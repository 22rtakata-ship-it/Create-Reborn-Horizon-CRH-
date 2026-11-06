// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

// SuperGlueSelectionHandler and SuperGlueSelectionPacket on Horizon's server.

use super::{
    Game,
    create_port_contraption::{Capture, column},
};
use create_engine::{
    contraption::{
        CaptureView, Cell,
        glue::{Search, SelectionError, Volume, selection},
    },
    kinetics::Pos,
    particles::{glue_edges, glue_face},
    placement::Direction,
};
use horizon_client::{
    item_render::cit::stack_max_damage,
    items::ItemStack,
    mod_api::Holder,
    particles::{ParticleOptions, ParticleType},
    text::Text,
};
use horizon_core::{
    geometry::{Aabb, Vec3},
    random::RandomSource,
};
use horizon_world::{
    attachments::{AttachmentCodec, AttachmentError, AttachmentType, Attachments},
    nbt::{Compound, Tag},
    player_inventory::EquipmentSlot,
};
use std::borrow::Cow;

const ITEM: &str = "create:super_glue";
const ATTACK_CHANNEL: &str = "create:remove_super_glue";
const FACE_CHANNEL: &str = "create:super_glue_face";
pub(super) const MAX_VOLUMES: usize = 128;
const MAX_SELECTION_BLOCKS: usize = 32_768;
const KEY: &str = "super_glue";

pub(super) const VOLUMES: AttachmentType<Vec<Volume>> = AttachmentType {
    namespace: "create",
    name: KEY,
    default: Vec::new,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: false,
    sync: true,
};

const FIRST: AttachmentType<Option<(String, Pos)>> = AttachmentType {
    namespace: "create",
    name: "glue_first_corner",
    default: || None,
    codec: None,
    copy_on_death: false,
    sync: false,
};

const ATTACK: AttachmentType<bool> = AttachmentType {
    namespace: "create",
    name: "glue_attack_requested",
    default: || false,
    codec: None,
    copy_on_death: false,
    sync: false,
};

pub(super) fn encode(volumes: &Vec<Volume>) -> Result<Compound, AttachmentError> {
    if volumes.len() > MAX_VOLUMES {
        return Err(AttachmentError::Encode);
    }
    let entries = volumes
        .iter()
        .map(|volume| {
            let [min, max] = volume.corners();
            Tag::IntArray([min, max].concat())
        })
        .collect();
    Ok(Compound::from([
        ("Version".into(), Tag::Int(1)),
        ("Volumes".into(), Tag::List(entries)),
    ]))
}

pub(super) fn decode(saved: &Compound) -> Result<Vec<Volume>, AttachmentError> {
    let error = AttachmentError::Decode;
    if !matches!(saved.get(&"Version".into()), Some(Tag::Int(1))) {
        return Err(error);
    }
    let Some(Tag::List(entries)) = saved.get(&"Volumes".into()) else {
        return Err(error);
    };
    if entries.len() > MAX_VOLUMES {
        return Err(error);
    }
    entries
        .iter()
        .map(|entry| {
            let Tag::IntArray(corners) = entry else {
                return Err(error);
            };
            let &[a, b, c, d, e, f] = corners.as_slice() else {
                return Err(error);
            };
            let min = [a, b, c];
            let max = [d, e, f];
            let span_squared = (0..3)
                .map(|i| (i128::from(max[i]) - i128::from(min[i])).pow(2))
                .sum::<i128>();
            if (0..3).any(|i| min[i] > max[i]) || span_squared >= 625 {
                return Err(error);
            }
            Ok(Volume::span(min, max))
        })
        .collect()
}

pub(super) fn read<'g>(
    game: &'g Game<'_>,
    key: [i32; 2],
) -> Result<Cow<'g, [Volume]>, AttachmentError> {
    let Some(chunk) = game.world.chunks.get(&(key[0], key[1])) else {
        return Err(AttachmentError::MissingHolder);
    };
    let Some(saved) = chunk.saved.as_ref() else {
        return Ok(Cow::Borrowed(&[]));
    };
    read_map(&saved.attachments, key)
}

fn read_map(map: &Attachments, key: [i32; 2]) -> Result<Cow<'_, [Volume]>, AttachmentError> {
    use horizon_world::attachments::create_contraption_attachment::{decoded, encoded};
    if let Some(volumes) = decoded(map, &VOLUMES) {
        return Ok(Cow::Borrowed(volumes));
    }
    if let Some(value) = encoded(map, "create", KEY) {
        let volumes = decode(value)?;
        if volumes
            .iter()
            .any(|volume| volume.corners().into_iter().any(|p| column(p) != key))
        {
            return Err(AttachmentError::Decode);
        }
        return Ok(Cow::Owned(volumes));
    }
    if map.has(&VOLUMES) {
        return Err(AttachmentError::Decode);
    }
    Ok(Cow::Borrowed(&[]))
}

struct GlueView<'g, 'a>(Capture<'g, 'a>);

impl CaptureView for GlueView<'_, '_> {
    type Block = ();

    fn inspect(&self, pos: Pos) -> Cell {
        let inspected = self.0.inspect(pos);
        if !matches!(inspected, Cell::Block(_)) {
            return inspected;
        }
        let game = self.0.0;
        let state = game.world.block(pos[0], pos[1], pos[2]);
        if game.res.facts().get(state).replaceable {
            return Cell::Empty;
        }
        inspected
    }

    fn snapshot(&self, _: Pos) -> Option<()> {
        Some(())
    }

    fn glued(&self, pos: Pos, face: Direction) -> bool {
        glued(self.0.0, pos, face)
    }
}

fn glued(game: &Game<'_>, pos: Pos, face: Direction) -> bool {
    read(game, column(pos)).is_ok_and(|volumes| volumes.iter().any(|v| v.glues(pos, face)))
}

fn sticky(game: &Game<'_>, pos: Pos, side: Direction) -> bool {
    matches!(
        game.native_block(pos).as_str(),
        "create:sticky_mechanical_piston"
            | "create:gantry_carriage"
            | "create:mechanical_bearing"
            | "create:windmill_bearing"
            | "create:clockwork_bearing"
    ) && game.native_block_property(pos, "facing").as_deref() == Some(side.name())
}

fn holds_glue(game: &Game<'_>) -> bool {
    game.stack_in(game.selected)
        .is_some_and(|stack| stack.item.as_ref() == ITEM)
}

pub(super) fn predicts(game: &Game<'_>, held: Option<&ItemStack>) -> bool {
    game.native_on("create")
        && holds_glue(game)
        && held.is_some_and(|stack| stack.item.as_ref() == ITEM)
}

fn available(game: &Game<'_>) -> Option<(usize, ItemStack, u32)> {
    std::iter::once(game.selected)
        .chain(0..game.inventory.len())
        .find_map(|slot| {
            let stack = game.stack_in(slot)?;
            if stack.item.as_ref() != ITEM || stack.count == 0 {
                return None;
            }
            let max = stack_max_damage(
                stack,
                game.items().get(ITEM).map_or(0, |info| info.max_damage),
            );
            (stack.damage < max).then(|| (slot, stack.clone(), max))
        })
}

fn spend(game: &mut Game<'_>) -> bool {
    if game.creative {
        return true;
    }
    let Some((slot, mut stack, max)) = available(game) else {
        return false;
    };
    stack.damage = stack.damage.saturating_add(game.wear(&stack, 1));
    if stack.damage < max {
        *game.stack_in_mut(slot) = Some(stack);
    } else {
        *game.stack_in_mut(slot) = None;
        game.award_stat("broken", ITEM, 1);
        game.player_equipment_broke(EquipmentSlot::MainHand, &stack);
    }
    true
}

fn status(game: &mut Game<'_>, key: &str) {
    let player = game.native_acting_id();
    game.native_action_bar(player, Text::translate(key, &[]));
}

fn sound(game: &mut Game<'_>, pos: Pos, pitch: f32) {
    let at = pos.map(|value| f64::from(value) + 0.5);
    game.with_level(|_, level, _| {
        level.native_play_sound("minecraft:block.slime_block.place", at, 4, 0.5, pitch);
        level.native_play_sound("minecraft:entity.item_frame.add_item", at, 4, 0.75, 1.0);
    });
}

fn send_face(game: &mut Game<'_>, pos: Pos, face: [i32; 3]) {
    let Some(index) = Direction::ALL.iter().position(|d| d.vector() == face) else {
        return;
    };
    let mut data = [0; 13];
    for (bytes, coord) in data[..12].chunks_exact_mut(4).zip(pos) {
        bytes.copy_from_slice(&coord.to_be_bytes());
    }
    data[12] = index as u8;
    let player = game.native_acting_id();
    let _ = game.native_net_send_to_player(player, FACE_CHANNEL, &data);
}

pub(super) fn receive_face(game: &mut Game<'_>, channel: &str, data: &[u8]) {
    if channel != FACE_CHANNEL
        || !game.native_window()
        || !game.native_on("create")
        || data.len() != 13
    {
        return;
    }
    let Some(&face) = Direction::ALL.get(usize::from(data[12])) else {
        return;
    };
    let pos =
        std::array::from_fn(|i| i32::from_be_bytes(data[4 * i..4 * i + 4].try_into().unwrap()));
    let options = game.item_particle(&ItemStack::new("minecraft:slime_ball", 1));
    let particles = glue_face(pos, face, || game.particles.level_random().next_float());
    for (at, velocity) in particles {
        game.particles.add(&options, at, velocity);
    }
}

fn send_edges(game: &mut Game<'_>, volume: Volume) {
    let options = ParticleOptions::simple(ParticleType::ItemSlime);
    game.with_level(|_, level, _| {
        glue_edges(volume, |at| {
            level.native_send_particles(&options, None, at, [0.0; 3], 0.0, 1);
        });
    });
}

fn vector([x, y, z]: [f64; 3]) -> Vec3 {
    Vec3::new(x, y, z)
}

fn nearest_volume(game: &Game<'_>) -> Option<([i32; 2], usize, [f64; 3])> {
    let (origin, direction) = game.aim()?;
    let eye = vector(origin);
    let end = eye.add_vector(vector(direction).scale(game.reach() + 1.0));
    let x = origin[0].floor() as i32;
    let z = origin[2].floor() as i32;
    let columns = |axis: i32| (axis - 32).div_euclid(16)..=(axis + 32).div_euclid(16);
    let mut nearest = None;
    let mut distance = f64::INFINITY;
    for x in columns(x) {
        for z in columns(z) {
            let column = [x, z];
            let volumes = if let Some(remote) = &game.remote {
                remote.attachments.get(&Holder::Chunk(column)).map_or_else(
                    || Ok(Cow::<[Volume]>::Borrowed(&[])),
                    |map| read_map(map, column),
                )
            } else {
                read(game, column)
            };
            let Ok(volumes) = volumes else {
                continue;
            };
            for (index, volume) in volumes.iter().enumerate() {
                let [min, max] = volume.corners();
                let lower = vector(min.map(f64::from));
                let upper = vector(max.map(|value| f64::from(value) + 1.0));
                let Some(hit) = Aabb::new(lower, upper).clip(eye, end) else {
                    continue;
                };
                let distance_to_hit = eye.distance_squared(hit);
                if distance_to_hit <= distance {
                    nearest = Some((column, index, hit.components()));
                    distance = distance_to_hit;
                }
            }
        }
    }
    nearest
}

/// An empty request names no world object. The server raycasts independently.
pub(super) fn attack_input(game: &mut Game<'_>, button: i32) -> bool {
    if button != 0
        || !game.native_window()
        || !game.native_on("create")
        || !holds_glue(game)
        || !game.may_build()
    {
        return false;
    }
    if game.entity_targeted() && nearest_volume(game).is_none() {
        return false;
    }
    game.native_net_send_to_server(ATTACK_CHANNEL, &[]).is_ok()
}

pub(super) fn receive_attack(game: &mut Game<'_>, player: u128, channel: &str, data: &[u8]) {
    if channel == ATTACK_CHANNEL
        && data.is_empty()
        && game.native_on("create")
        && let Ok(map) = game.attachment_map_quiet(Holder::Player(player))
    {
        let _ = map.set(&ATTACK, true);
    }
}

pub(super) fn left_click(game: &mut Game<'_>) -> bool {
    if !game.native_on("create") || !holds_glue(game) || !game.may_build() {
        return false;
    }
    if game.player_attachments.has(&FIRST) {
        return true;
    }
    let Some((column, index, at)) = nearest_volume(game) else {
        return true;
    };
    let Ok(volumes) = read(game, column) else {
        return true;
    };
    let mut volumes = volumes.into_owned();
    let volume = volumes.remove(index);
    let Ok(map) = game.native_attachments(Holder::Chunk(column)) else {
        return true;
    };
    let removed = if volumes.is_empty() {
        map.remove(&VOLUMES).map(|_| ())
    } else {
        map.set(&VOLUMES, volumes)
    };
    if removed.is_ok() {
        send_edges(game, volume);
        let sound_pos = at.map(|value| value.floor() + 0.5);
        game.with_level(|_, level, _| {
            level.native_play_sound("minecraft:block.slime_block.place", sound_pos, 4, 0.5, 0.5);
        });
    }
    true
}

pub(super) fn player_tick(game: &mut Game<'_>) {
    if game.player_attachments.remove(&ATTACK).ok().flatten() == Some(true) {
        left_click(game);
    }
    if !game.player_attachments.has(&FIRST) {
        return;
    }
    if !game.native_on("create") || !holds_glue(game) || !game.may_build() {
        let _ = game.player_attachments.remove(&FIRST);
        status(game, "create.super_glue.abort");
    }
}

pub(super) fn right_click(game: &mut Game<'_>, pos: Pos, face: [i32; 3]) -> bool {
    if !game.native_on("create")
        || game.held_slot() != game.selected
        || !holds_glue(game)
        || !game.may_build()
    {
        return false;
    }
    let first = game
        .player_attachments
        .get_existing(&FIRST)
        .ok()
        .flatten()
        .flatten()
        .filter(|(dimension, _)| dimension == game.dim.id());
    if game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft) {
        if first.is_some() {
            let _ = game.player_attachments.remove(&FIRST);
            status(game, "create.super_glue.abort");
        }
        return true;
    }
    let Some((_, first)) = first else {
        if game
            .player_attachments
            .set(&FIRST, Some((game.dim.id().into(), pos)))
            .is_ok()
        {
            status(game, "create.super_glue.first_pos");
            sound(game, pos, 0.85);
            send_face(game, pos, face);
        }
        return true;
    };
    let view = GlueView(Capture(game));
    let planned = selection(
        &view,
        first,
        pos,
        Search::Preview,
        MAX_SELECTION_BLOCKS,
        |pos, side| sticky(game, pos, side),
    );
    match planned {
        Err(SelectionError::TooFar) => {
            status(game, "create.super_glue.too_far");
            return true;
        }
        Ok(cluster) if cluster.can_commit() => {}
        _ => {
            status(game, "create.super_glue.cannot_reach");
            return true;
        }
    }
    if !game.creative && available(game).is_none() {
        status(game, "create.super_glue.not_enough");
        return true;
    }

    let planned = selection(
        &view,
        first,
        pos,
        Search::Commit,
        MAX_SELECTION_BLOCKS,
        |pos, side| sticky(game, pos, side),
    );
    let _ = game.player_attachments.remove(&FIRST);
    let Ok(cluster) = planned else { return true };
    if !cluster.can_commit() {
        return true;
    }
    let volume = cluster.volume;
    let [min, max] = volume.corners();
    let key = column(min);
    if column(max) != key {
        status(game, "create.super_glue.too_far");
        return true;
    }
    let Ok(volumes) = read(game, key) else {
        return true;
    };
    let mut volumes = volumes.into_owned();
    if volumes.len() == MAX_VOLUMES {
        return true;
    }
    volumes.push(volume);
    let Ok(map) = game.native_attachments(Holder::Chunk(key)) else {
        return true;
    };
    if map.set(&VOLUMES, volumes).is_err() {
        return true;
    }
    // 6.0.10's `collectGlueFromInventory` uses its durability path for both
    // calls. The first can break the selected glue. The second then selects
    // the next glue in ordinary inventory order, if one remains.
    spend(game);
    spend(game);
    send_edges(game, volume);
    status(game, "create.super_glue.success");
    sound(game, pos, 0.95);
    true
}
