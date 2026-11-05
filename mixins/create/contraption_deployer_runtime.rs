// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

// Bounded Deployer USE adapter for tree saplings on vertical-axis bearings.

use super::{
    Game, create_port_contraption as contraption, create_port_contraption_storage as storage,
    native_block_entity_draw::BlockEntityDraw,
};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModBlockEntityFields},
    items::ItemStack,
};
use create_engine::{
    contraption::QuarterTurn,
    deployer::{Aim, Held, Mode, MovingAction, Target, grabs_from_storage, moving_action},
    kinetics::{Axis, Pos},
    placement::Direction,
};
use glam::DVec3;
use horizon_client_render::create_contraption_scene::{MAX_RECORDS, Record};
use horizon_core::{
    geometry::{Direction as Face, Vec3},
    position::BlockPos,
};
use horizon_items::item_nbt::{encode_items, item_from_nbt, item_to_nbt};
use horizon_world::{
    attachments::{AttachmentCodec, AttachmentError, AttachmentType, Holder},
    chunk::BlockState,
    level::{PlaceContext, PlaceOutcome},
    nbt::{Compound, Tag},
    registry::BlockStateId,
};
use std::collections::BTreeMap;

pub(super) const BLOCK: &str = "create:deployer";
const SAPLINGS: &[&str] = &[
    "minecraft:oak_sapling",
    "minecraft:spruce_sapling",
    "minecraft:birch_sapling",
    "minecraft:jungle_sapling",
    "minecraft:acacia_sapling",
    "minecraft:dark_oak_sapling",
    "minecraft:cherry_sapling",
];

#[derive(Clone)]
pub(super) struct Cargo {
    local: Pos,
    nbt: Compound,
}

type Store = BTreeMap<Pos, Cargo>;
type Edit = (Pos, BlockStateId, BlockStateId);

const CARGO: AttachmentType<Store> = AttachmentType {
    namespace: "create",
    name: "mounted_deployer",
    default: Store::new,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: false,
    sync: false,
};

pub(super) fn block_state(state: &BlockState) -> bool {
    state.name == BLOCK
        && state.properties.len() == 2
        && state
            .properties
            .get("facing")
            .is_some_and(|v| Direction::ALL.into_iter().any(|d| d.name() == v))
        && state
            .properties
            .get("axis_along_first")
            .is_some_and(|v| matches!(v.as_str(), "true" | "false"))
}

pub(super) fn default_nbt() -> Compound {
    Compound::from([
        ("Mode".into(), Tag::String("USE".into())),
        ("State".into(), Tag::String("WAITING".into())),
        ("Timer".into(), Tag::Int(0)),
        ("Powered".into(), Tag::Byte(0)),
    ])
}

pub(super) fn update_tag(entity: &ModBlockEntity) -> Compound {
    ["Mode", "HeldItem"]
        .into_iter()
        .filter_map(|key| get(&entity.nbt, key).map(|tag| (key.into(), tag.clone())))
        .collect()
}

fn draw_arm(
    game: &Game<'_>,
    draw: &mut BlockEntityDraw,
    facing: Direction,
    first: bool,
    extension: f32,
    holding: bool,
) {
    let shaft_axis = match facing.axis() {
        Axis::X => {
            if first {
                Axis::Y
            } else {
                Axis::Z
            }
        }
        Axis::Y => {
            if first {
                Axis::X
            } else {
                Axis::Z
            }
        }
        Axis::Z => {
            if first {
                Axis::X
            } else {
                Axis::Y
            }
        }
    };
    draw.push();
    match shaft_axis {
        Axis::X => draw.rotate_about([0., 0., 1.], 90., [0.5; 3]),
        Axis::Z => draw.rotate_about([1., 0., 0.], 90., [0.5; 3]),
        Axis::Y => {}
    }
    game.native_draw_model(draw, "create:block/shaft", 0xffffff);
    draw.pop();
    draw.push();
    super::create_port::face_south(draw, facing);
    draw.translate([0.0, 0.0, extension]);
    draw.push();
    if first ^ (facing.axis() == Axis::Z) {
        draw.rotate_about([0., 0., 1.], 90., [0.5; 3]);
    }
    game.native_draw_model(draw, "create:block/deployer/pole", 0xffffff);
    draw.pop();
    let model = if holding {
        "create:block/deployer/hand_holding"
    } else {
        "create:block/deployer/hand_pointing"
    };
    game.native_draw_model(draw, model, 0xffffff);
    draw.pop();
}

pub(super) fn draw_station(
    game: &Game<'_>,
    pos: Pos,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    let Some(facing) = contraption::facing(game, pos) else {
        return;
    };
    let first = game
        .native_block_property(pos, "axis_along_first")
        .as_deref()
        == Some("true");
    let holding = get(&entity.nbt, "HeldItem").is_some_and(|held| held.get("id").is_some());
    draw_arm(game, draw, facing, first, 0.0, holding);
}

pub(super) fn draw_moving(
    game: &Game<'_>,
    draw: &mut BlockEntityDraw,
    record: &Record,
    local: Pos,
    state: &BlockState,
) {
    let Some(facing) = state
        .properties
        .get("facing")
        .and_then(|name| Direction::ALL.into_iter().find(|face| face.name() == name))
    else {
        return;
    };
    let active =
        DVec3::from_array(create_engine::deployer::active_point(local, facing)) - DVec3::splat(0.5);
    let sample = |angle| {
        DVec3::from_array(record.origin_at(1.0))
            + record.orientation(angle) * active
            + DVec3::splat(0.5)
    };
    let point = sample(record.angle);
    let previous = sample(record.previous_angle);
    let extension = create_engine::deployer::moving_extension(
        Some(point.to_array()),
        (point - previous).to_array(),
        false,
        false,
        draw.game_time as f32 + draw.partial,
        draw.partial,
    );
    let first = state
        .properties
        .get("axis_along_first")
        .is_some_and(|value| value == "true");
    draw_arm(game, draw, facing, first, extension, false);
}

fn get<'a>(nbt: &'a Compound, key: &str) -> Option<&'a Tag> {
    nbt.get(&key.into())
}

fn supported_item(item: &ItemStack) -> bool {
    SAPLINGS.contains(&item.item.as_ref())
        && (1..=64).contains(&item.count)
        && item.damage == 0
        && item.data().is_none()
}

fn read_hand(nbt: &Compound) -> Option<Option<ItemStack>> {
    let hand = match get(nbt, "Inventory") {
        None => None,
        Some(Tag::List(items)) if items.is_empty() => None,
        Some(Tag::List(items)) if items.len() == 1 => {
            let item = item_from_nbt(&items[0])?;
            if !supported_item(&item)
                || encode_items(&[Some(item.clone())]) != Tag::List(items.clone())
            {
                return None;
            }
            Some(item)
        }
        _ => return None,
    };
    if let Some(display) = get(nbt, "HeldItem") {
        let expected = hand
            .as_ref()
            .map_or_else(|| Tag::Compound(Compound::new()), item_to_nbt);
        if *display != expected {
            return None;
        }
    }
    Some(hand)
}

fn valid_nbt(nbt: &Compound) -> bool {
    get(nbt, "Mode") == Some(&Tag::String("USE".into()))
        && nbt
            .iter()
            .all(|(key, val)| match key.to_string().as_deref() {
                Ok("Mode" | "Inventory" | "HeldItem") => true,
                Ok("State") => *val == Tag::String("WAITING".into()),
                Ok("Timer") => *val == Tag::Int(0),
                Ok("Powered") => *val == Tag::Byte(0),
                Ok("Owner") => matches!(val, Tag::IntArray(uuid) if uuid.len() == 4),
                Ok("Overflow") => matches!(val, Tag::List(items) if items.is_empty()),
                Ok("Filter") => true,
                Ok("FilterAmount") => matches!(val, Tag::Int(0..=64)),
                Ok("UpTo") => matches!(val, Tag::Byte(0 | 1)),
                _ => false,
            })
        && read_filter(nbt).is_some_and(|filter| {
            read_hand(nbt).is_some_and(|hand| {
                filter
                    .as_ref()
                    .is_none_or(|filter| hand.as_ref().is_none_or(|held| held.item == filter.item))
            })
        })
}

fn read_filter(nbt: &Compound) -> Option<Option<ItemStack>> {
    let Some(filter) = get(nbt, "Filter") else {
        return Some(None);
    };
    if matches!(filter, Tag::Compound(parts) if parts.is_empty()) {
        return Some(None);
    }
    let stack = item_from_nbt(filter)?;
    (supported_item(&stack) && item_to_nbt(&stack) == *filter).then_some(Some(stack))
}

fn write_hand(nbt: &mut Compound, hand: Option<ItemStack>) {
    nbt.insert(
        "HeldItem".into(),
        hand.as_ref()
            .map_or_else(|| Tag::Compound(Compound::new()), item_to_nbt),
    );
    nbt.insert("Inventory".into(), encode_items(&[hand]));
}

fn entity<'a>(game: &'a Game<'_>, pos: Pos) -> Option<&'a ModBlockEntity> {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return None;
    };
    if entity.id != BLOCK || !valid_nbt(&entity.nbt) || entity.items.len() > 1 {
        return None;
    }
    // Java-shaped commands may initialize only Inventory. Native hand swaps
    // and restoration also fill the host's one private, spillable slot.
    (entity
        .items
        .first()
        .is_none_or(|item| item.is_none() || read_hand(&entity.nbt).as_ref() == Some(item)))
    .then_some(entity)
}

pub(super) fn placed(game: &mut Game<'_>, pos: Pos) {
    let uuid = game.acting_player_uuid().to_vec();
    game.native_with_mod_block_entity(pos, |entity| {
        entity.nbt.insert("Owner".into(), Tag::IntArray(uuid));
    });
}

fn exchange_target(game: &Game<'_>, pos: Pos, hit: [f64; 3], held: Option<&ItemStack>) -> bool {
    if !game.native_on("create")
        || game.native_block(pos) != BLOCK
        || !game.may_build()
        || game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
        || held.is_some_and(|held| !supported_item(held))
    {
        return false;
    }
    contraption::facing(game, pos).is_some_and(|facing| {
        let axis = facing.axis().index();
        let back = f64::from(pos[axis]) + 0.5 - f64::from(facing.vector()[axis]) * 0.5;
        (hit[axis] - back).abs() >= 0.75
    })
}

pub(super) fn predicts(game: &Game<'_>, hit: &super::Hit, held: Option<&ItemStack>) -> bool {
    exchange_target(game, hit.pos, hit.point, held)
}

pub(super) fn right_click(game: &mut Game<'_>, pos: Pos) -> bool {
    let held = game.held().cloned();
    let Some(hit) = game
        .pick()
        .filter(|hit| hit.pos == pos && exchange_target(game, pos, hit.point, held.as_ref()))
    else {
        return false;
    };
    let Some(entity) = entity(game, hit.pos) else {
        return false;
    };
    let Some(previous) = read_hand(&entity.nbt) else {
        return false;
    };
    if let (Some(item), Some(Some(filter))) = (held.as_ref(), read_filter(&entity.nbt))
        && item.item != filter.item
    {
        return false;
    }
    if held.is_none() && previous.is_none() {
        return true;
    }
    if game
        .native_with_mod_block_entity(pos, |entity| {
            entity.items = vec![held.clone()];
            write_hand(&mut entity.nbt, held);
        })
        .is_none()
    {
        return false;
    }
    *game.held_mut() = previous;
    game.native_block_entity_sync(pos);
    true
}

pub(super) fn can_capture(game: &Game<'_>, pos: Pos, state: &BlockState) -> bool {
    if !block_state(state)
        || state
            .properties
            .get("facing")
            .is_none_or(|face| face != "down")
        || entity(game, pos).is_none()
    {
        return false;
    }
    let key = (pos[0], pos[1], pos[2]);
    let [x, z] = contraption::column(pos);
    !game.world.block_names.contains_key(&key)
        && game.world.block_lock(key).is_none()
        && !game
            .world
            .block_entities
            .chunk_attachments(x, z)
            .iter()
            .any(|(p, _)| *p == key)
        && game
            .world
            .chunks
            .get(&(x, z))
            .and_then(|c| c.saved.as_ref())
            .is_none_or(|saved| {
                !saved
                    .block_entity_attachments
                    .iter()
                    .any(|(p, _)| *p == key)
                    && !saved.rolled_loot.iter().any(|loot| loot.pos == key)
                    && saved
                        .block_entity_sources
                        .iter()
                        .find(|(p, _)| *p == key)
                        .is_none_or(|(_, source)| source_ok(source))
            })
}

fn source_ok(source: &Tag) -> bool {
    let Tag::Compound(fields) = source else {
        return false;
    };
    if fields
        .iter()
        .any(|(key, tag)| match key.to_string().as_deref() {
            Ok("id") => *tag != Tag::String(BLOCK.into()),
            Ok("x" | "y" | "z") => !matches!(tag, Tag::Int(_)),
            Ok("keepPacked") => !matches!(tag, Tag::Byte(0)),
            _ => false,
        })
    {
        return false;
    }
    let mut nbt = fields.clone();
    nbt.retain(|key, _| {
        !matches!(
            key.to_string().as_deref(),
            Ok("id" | "x" | "y" | "z" | "keepPacked")
        )
    });
    if nbt.get(&"Items".into()).is_some_and(|tag| {
        matches!(tag, Tag::List(items) if items.is_empty()) || get(&nbt, "Inventory") == Some(tag)
    }) {
        nbt.remove(&"Items".into());
    }
    if nbt
        .get(&"components".into())
        .is_some_and(|tag| matches!(tag, Tag::Compound(parts) if parts.is_empty()))
    {
        nbt.remove(&"components".into());
    }
    valid_nbt(&nbt)
}

fn local_deployer(blocks: &[(Pos, BlockState)]) -> Result<Option<Pos>, AttachmentError> {
    let mut result = None;
    for (local, block) in blocks.iter().filter(|(_, block)| block.name == BLOCK) {
        if !block_state(block) || result.replace(*local).is_some() {
            return Err(AttachmentError::Decode);
        }
    }
    Ok(result)
}

pub(super) fn capture(
    game: &Game<'_>,
    anchor: Pos,
    blocks: &[(Pos, BlockState)],
) -> Result<Option<Cargo>, AttachmentError> {
    let Some(local) = local_deployer(blocks)? else {
        return Ok(None);
    };
    // Centered actors use the source's separate StationaryTimer path.
    if local[0] == 0 && local[2] == 0 {
        return Err(AttachmentError::Encode);
    }
    let pos = std::array::from_fn(|i| anchor[i] + local[i]);
    let entity = entity(game, pos).ok_or(AttachmentError::Encode)?;
    Ok(Some(Cargo {
        local,
        nbt: entity.nbt.clone(),
    }))
}

pub(super) fn commit_capture(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
    edits: &[Edit],
    controller: Pos,
    chest: Option<storage::Chest>,
    cargo: Option<Cargo>,
) -> bool {
    let Some(cargo) = cargo else {
        return storage::commit_capture(game, key, records, edits, controller, chest);
    };
    transaction(
        game,
        key,
        |store| {
            if store.insert(controller, cargo).is_some() {
                return false;
            }
            true
        },
        |game| storage::commit_capture(game, key, records, edits, controller, chest),
    )
}

pub(super) fn commit_landing(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
    edits: &[Edit],
    record: &Record,
    turn: QuarterTurn,
) -> bool {
    let local = match local_deployer(&record.blocks) {
        Ok(Some(local)) => local,
        Ok(None) => return storage::commit_landing(game, key, records, edits, record, turn),
        Err(_) => return false,
    };
    let Some(pos) = turn.place(record.anchor, local) else {
        return false;
    };
    let entity_pos = (pos[0], pos[1], pos[2]);
    if game.world.block_entities.contains_key(&entity_pos)
        || !edits.iter().any(|&(at, _, state)| {
            at == pos
                && game
                    .pack
                    .assets
                    .registry
                    .state(state)
                    .is_some_and(block_state)
        })
    {
        return false;
    }
    let mut cargo = None;
    if !transaction(
        game,
        key,
        |store| {
            cargo = store
                .remove(&record.controller)
                .filter(|cargo| cargo.local == local);
            cargo.is_some()
        },
        |game| storage::commit_landing(game, key, records, edits, record, turn),
    ) {
        return false;
    }
    if let Some(cargo) = cargo {
        let items = vec![read_hand(&cargo.nbt).flatten()];
        game.world.block_entities.insert(
            entity_pos,
            BlockEntity::Mod(Box::new(ModBlockEntity::new(ModBlockEntityFields {
                id: BLOCK.into(),
                nbt: cargo.nbt,
                items,
                ..ModBlockEntityFields::default()
            }))),
        );
        game.native_block_entity_sync(pos);
    }
    true
}

fn transaction(
    game: &mut Game<'_>,
    key: [i32; 2],
    change: impl FnOnce(&mut Store) -> bool,
    commit: impl FnOnce(&mut Game<'_>) -> bool,
) -> bool {
    let holder = Holder::Chunk(key);
    let Ok(map) = game.attachment_map_quiet(holder) else {
        return false;
    };
    let Ok(mut store) = map.get(&CARGO) else {
        return false;
    };
    if !change(&mut store) {
        return false;
    }
    let before = map.clone();
    let saved = if store.is_empty() {
        map.remove(&CARGO).map(|_| ())
    } else {
        map.set(&CARGO, store)
    };
    if saved.is_err() {
        return false;
    }
    if commit(game) {
        return true;
    }
    if let Ok(map) = game.attachment_map_quiet(holder) {
        *map = before;
    }
    false
}

pub(super) fn supported_record(record: &Record) -> bool {
    record.axis == Axis::Y.index()
        && record.stroke.is_none()
        && record.clock_hand.is_none()
        && !record.return_home
}

pub(super) fn plant(game: &mut Game<'_>, record: &Record, local: Pos, aim: Aim) {
    let target = aim.clicked;
    let key = contraption::column(record.controller);
    if game.options.execution_role != super::execution_role::ExecutionRole::IntegratedServer
        || !supported_record(record)
    {
        return;
    }
    let holder = Holder::Chunk(key);
    let Ok(Some(store)) = game
        .attachment_map_quiet(holder)
        .and_then(|map| map.get_existing(&CARGO))
    else {
        return;
    };
    let Some(cargo) = store
        .get(&record.controller)
        .filter(|cargo| cargo.local == local)
    else {
        return;
    };
    let (Some(mut hand), Some(filter)) = (read_hand(&cargo.nbt), read_filter(&cargo.nbt)) else {
        return;
    };
    // The actor refills on a visit even if shouldActivate or placement later
    // refuses the target. In that case the one withdrawn item stays held.
    if grabs_from_storage(hand.is_none(), false)
        && let Some(mount) = storage::mounted_chest(record)
    {
        hand = storage::withdraw_one(
            game,
            mount,
            |item| {
                filter
                    .as_ref()
                    .is_none_or(|filter| item.item == filter.item)
            },
            |game, item| {
                supported_item(item)
                    && update_hand(
                        game,
                        key,
                        record.controller,
                        local,
                        Some(item.clone()),
                        |_| true,
                    )
            },
        );
    }
    let Some(mut hand) = hand else { return };
    if key != contraption::column(target)
        || !contraption::resident(game, target)
        || !game
            .border
            .contains(f64::from(target[0]), f64::from(target[2]))
        || !game.res.facts().is_air(contraption::state(game, target))
    {
        return;
    }
    let name = game.native_block(target);
    if moving_action(
        Mode::Use,
        Held::Block(&hand.item),
        Target {
            block: &name,
            fluid: None,
            transports_items: false,
        },
        false,
    ) != MovingAction::World(Mode::Use)
    {
        return;
    }
    let ray = aim.action_ray();
    let segment = DVec3::from_array(ray.end) - DVec3::from_array(ray.start);
    let vector = segment.normalize();
    let fallback = if vector.x.abs() > vector.z.abs() {
        if vector.x > 0.0 {
            Face::West
        } else {
            Face::East
        }
    } else if vector.z > 0.0 {
        Face::North
    } else {
        Face::South
    };
    let hit = game.clip_ray(ray.start, vector.to_array(), segment.length());
    let (face, point) = hit.map_or((fallback, ray.end), |hit| {
        (native_face(hit.face).unwrap_or(fallback), hit.point)
    });
    let pos = BlockPos::new(target[0], target[1], target[2]);
    let ctx = PlaceContext {
        clicked_pos: pos,
        face,
        hit: Vec3::new(point[0], point[1], point[2]),
        inside: false,
        yaw: aim.yaw,
        pitch: aim.pitch,
        sneaking: false,
        item: hand.item.to_string(),
        can_use_game_master_blocks: false,
    };
    if game
        .with_level(|b, l, t| b.placement_state(l, t, &ctx))
        .is_none_or(|(at, _)| at != pos)
    {
        return;
    }
    hand.count -= 1;
    update_hand(
        game,
        key,
        record.controller,
        local,
        (hand.count > 0).then_some(hand),
        |game| {
            let placed = game.with_level(|b, l, t| b.place_block(l, t, &ctx));
            let _ = game.process_level_events();
            matches!(placed, PlaceOutcome::Placed { .. })
        },
    );
}

fn update_hand(
    game: &mut Game<'_>,
    key: [i32; 2],
    controller: Pos,
    local: Pos,
    hand: Option<ItemStack>,
    commit: impl FnOnce(&mut Game<'_>) -> bool,
) -> bool {
    transaction(
        game,
        key,
        |store| {
            let Some(cargo) = store
                .get_mut(&controller)
                .filter(|cargo| cargo.local == local)
            else {
                return false;
            };
            write_hand(&mut cargo.nbt, hand);
            true
        },
        commit,
    )
}

fn native_face(normal: Pos) -> Option<Face> {
    Some(match normal {
        [0, -1, 0] => Face::Down,
        [0, 1, 0] => Face::Up,
        [0, 0, -1] => Face::North,
        [0, 0, 1] => Face::South,
        [-1, 0, 0] => Face::West,
        [1, 0, 0] => Face::East,
        _ => return None,
    })
}

fn encode(store: &Store) -> Result<Compound, AttachmentError> {
    if store.len() > MAX_RECORDS || store.values().any(|cargo| !valid_nbt(&cargo.nbt)) {
        return Err(AttachmentError::Encode);
    }
    let entries = store
        .iter()
        .map(|(controller, cargo)| {
            Tag::compound([
                ("Controller", Tag::IntArray(controller.to_vec())),
                ("Pos", Tag::IntArray(cargo.local.to_vec())),
                ("Data", Tag::Compound(cargo.nbt.clone())),
            ])
        })
        .collect();
    Ok(Compound::from([
        ("Version".into(), Tag::Int(1)),
        ("Deployers".into(), Tag::List(entries)),
    ]))
}

fn decode(saved: &Compound) -> Result<Store, AttachmentError> {
    let invalid = AttachmentError::Decode;
    let Some(Tag::List(entries)) = get(saved, "Deployers") else {
        return Err(invalid);
    };
    if saved.len() != 2
        || get(saved, "Version") != Some(&Tag::Int(1))
        || entries.len() > MAX_RECORDS
    {
        return Err(invalid);
    }
    let position = |entry: &Tag, key: &str| match entry.get(key) {
        Some(Tag::IntArray(values)) => <Pos>::try_from(values.as_slice()).map_err(|_| invalid),
        _ => Err(invalid),
    };
    let mut store = Store::new();
    for entry in entries {
        if !matches!(entry, Tag::Compound(fields) if fields.len() == 3) {
            return Err(invalid);
        }
        let Some(Tag::Compound(nbt)) = entry
            .get("Data")
            .filter(|tag| matches!(tag, Tag::Compound(nbt) if valid_nbt(nbt)))
        else {
            return Err(invalid);
        };
        let cargo = Cargo {
            local: position(entry, "Pos")?,
            nbt: nbt.clone(),
        };
        if cargo
            .local
            .into_iter()
            .any(|coordinate| coordinate.unsigned_abs() > 15)
            || store
                .insert(position(entry, "Controller")?, cargo)
                .is_some()
        {
            return Err(invalid);
        }
    }
    Ok(store)
}
