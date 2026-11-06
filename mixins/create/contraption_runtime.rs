// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::create_port_chassis as chassis;
use super::create_port_contraption_actors as actors;
use super::create_port_contraption_storage as storage;
use super::create_port_deployer as deployer;
use super::create_port_portable_storage as psi;
use super::create_port_sticker as sticker;
use super::{
    Game, create_port_clockwork as clockwork, create_port_pulley as pulley,
    native_block_entity_draw::BlockEntityDraw, native_world_draw::NativeWorldDraw,
};
use crate::containers::{BlockEntity, ModBlockEntity};
use create_engine::{
    contraption::actors::{Tool, harvester::Blade, points_backwards},
    contraption::bearing::{Bearing, Mode, TickAction},
    contraption::windmill::{Config as WindmillConfig, RotationDirection, Windmill},
    contraption::{
        Adhesion, Assembly, BlockRules, CaptureView, Cell, QuarterTurn, Rotation, plan_landing,
    },
    kinetics::{Axis, Pos},
    placement::Direction,
};
use glam::DVec3;
use horizon_client_render::create_contraption_scene::{
    ACTIVE, ClockHand, HARVESTER_KEY, KEY, MAX_BLOCKS, MAX_RECORDS, RECORDS, Record, StrokeKind,
    read,
};
use horizon_core::position::BlockPos;
use horizon_world::{
    attachments::{
        AttachmentError, AttachmentType, Attachments, Holder,
        create_contraption_attachment::decoded,
    },
    chunk::BlockState,
    level::flags,
    nbt::Tag,
    registry::BlockStateId,
};
use std::{collections::BTreeMap, sync::Arc};

pub(super) const BLOCK: &str = "create:mechanical_bearing";
pub(super) const WINDMILL: &str = "create:windmill_bearing";
pub(super) const LAST_GENERATED: &str = "LastGenerated";
const WINDMILL_REQUEST: &str = "QueueAssembly";

pub(super) fn is_bearing(block: &str) -> bool {
    matches!(block, BLOCK | WINDMILL | clockwork::BLOCK)
}
const RUNNING: &str = "Running";
const POWER: &str = "create_horizon:AssemblyPower";
const REQUEST: &str = "create_horizon:AssembleNextTick";

pub(super) fn column(pos: Pos) -> [i32; 2] {
    [pos[0] >> 4, pos[2] >> 4]
}

pub(super) fn state(game: &Game<'_>, p: Pos) -> BlockStateId {
    game.world.block(p[0], p[1], p[2])
}

pub(super) fn resident(game: &Game<'_>, p: Pos) -> bool {
    let [x, z] = column(p);
    game.world
        .chunks
        .get(&(x, z))
        .is_some_and(|chunk| chunk.section_index(p[1]).is_some())
}

fn axis(index: usize) -> Axis {
    [Axis::X, Axis::Y, Axis::Z][index]
}

pub(super) fn facing(game: &Game<'_>, p: Pos) -> Option<Direction> {
    let properties = &game.pack.assets.registry.state(state(game, p))?.properties;
    let facing = properties.get("facing")?;
    Direction::ALL
        .into_iter()
        .find(|direction| direction.name() == facing)
}

fn controller_present(game: &Game<'_>, record: &Record) -> bool {
    let [x, y, z] = record.controller;
    let name = game.native_block(record.controller);
    let rope_pulley = record.stroke_kind() == Some(StrokeKind::RopePulley);
    let correct = if rope_pulley {
        name == pulley::BLOCK
    } else if record.stroke_kind() == Some(StrokeKind::Piston) {
        super::create_port_piston::is_piston(&name)
            && game
                .native_block_property(record.controller, "state")
                .as_deref()
                == Some("moving")
    } else if record.clock_hand.is_some() {
        name == clockwork::BLOCK
    } else {
        name == if record.return_home { WINDMILL } else { BLOCK }
    };
    correct
        && (rope_pulley
            || facing(game, record.controller).is_some_and(|f| {
                let normal = f.vector();
                let distance = if record.clock_hand == Some(ClockHand::Minute) {
                    i64::from(record.controller[record.axis].abs_diff(record.anchor[record.axis]))
                } else {
                    1
                };
                record
                    .controller
                    .map(i64::from)
                    .into_iter()
                    .zip(normal)
                    .zip(record.anchor)
                    .all(|((p, delta), anchor)| {
                        p + i64::from(delta) * distance == i64::from(anchor)
                    })
            }))
        && matches!(game.world.block_entities.get(&(x, y, z)),
            Some(BlockEntity::Mod(entity)) if entity.nbt.get(&RUNNING.into()) == Some(&Tag::Byte(1)))
}

pub(super) fn entries(game: &Game<'_>, [x, z]: [i32; 2]) -> Result<Vec<Record>, AttachmentError> {
    Ok(read(
        game.world
            .chunks
            .get(&(x, z))
            .ok_or(AttachmentError::MissingHolder)?,
    )?
    .into_owned())
}

pub(super) fn set_records(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
) -> Result<(), AttachmentError> {
    // Update the transient visibility hint first so an error cannot leave a
    // saved record for block edits the caller has not committed.
    if !records.is_empty() {
        game.world.attachments.set(&ACTIVE, true)?;
    }
    let map = game.native_attachments(Holder::Chunk(key))?;
    // An empty replacement also clears the native client on the same channel.
    if records.is_empty() {
        map.remove(&RECORDS)?;
    } else {
        map.set(&RECORDS, records)?;
    }
    Ok(())
}

type Edit = (Pos, BlockStateId, BlockStateId);

/// All edits and the saved attachment share one column. No save or network
/// flush runs between the writes. Supported states need no on-place callbacks.
pub(super) fn commit(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
    edits: &[Edit],
) -> bool {
    let mut positions = std::collections::BTreeSet::new();
    if edits.iter().any(|&(p, before, _)| {
        !positions.insert(p) || column(p) != key || !resident(game, p) || state(game, p) != before
    }) {
        return false;
    }
    if super::create_port_glue_transfer::blocked(
        game,
        key,
        edits.iter().filter_map(|&(position, before, after)| {
            (before != after && game.res.facts().is_air(after)).then_some(position)
        }),
    ) {
        return false;
    }
    let original_entities: Vec<_> = edits
        .iter()
        .filter_map(|&(p, _, _)| {
            let key = (p[0], p[1], p[2]);
            game.world
                .block_entities
                .get(&key)
                .map(|entity| (key, entity.clone()))
        })
        .collect();
    game.ensure_stateful_entity_table();
    let holder = Holder::Chunk(key);
    let Ok(map) = game.native_attachments(holder) else {
        return false;
    };
    let previous = map.clone();
    if set_records(game, key, records).is_err() {
        return false;
    }
    let flags = flags::UPDATE_CLIENTS | flags::UPDATE_SKIP_ALL_SIDEEFFECTS;
    let applied = game.with_level(|behaviour, level, ticks| {
        for (index, &(p, _, after)) in edits.iter().enumerate() {
            if !behaviour.set_block(level, ticks, BlockPos::new(p[0], p[1], p[2]), after, flags) {
                for &(done, before, _) in edits[..index].iter().rev() {
                    behaviour.set_block(
                        level,
                        ticks,
                        BlockPos::new(done[0], done[1], done[2]),
                        before,
                        flags,
                    );
                }
                return false;
            }
        }
        true
    });
    if !applied {
        if let Ok(map) = game.native_attachments(holder) {
            *map = previous;
        }
        for (key, entity) in original_entities {
            game.world.block_entities.insert(key, entity);
        }
        return false;
    }
    // Like Horizon's native set-block helper, rebuild default entities when
    // the block type changes. A driver changing only its state is often in
    // the middle of its tick, with the original entity held by the caller.
    let registry = game.pack.assets.registry;
    for &(p, before, after) in edits {
        let key = (p[0], p[1], p[2]);
        if registry.state(before).map(|s| &s.name) != registry.state(after).map(|s| &s.name)
            && !game.world.block_entities.contains_key(&key)
            && let Some(entity) = game.new_block_entity_for_state(after)
        {
            game.world.block_entities.insert(key, entity);
            game.native_block_entity_sync(p);
        }
    }
    true
}

// These 1.21.1 wood families carry the same plain or pillar states in Horizon.
const WOOD_FAMILIES: &[&str] = &[
    "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove", "cherry", "bamboo",
    "crimson", "warped",
];

fn supported(state: &BlockState) -> bool {
    if super::create_port_sail::from_state(state).is_some() {
        return true;
    }
    match state.name.as_str() {
        "create:mechanical_harvester" | "create:mechanical_drill" | actors::PLOUGH => state
            .properties
            .get("waterlogged")
            .is_some_and(|wet| wet == "false"),
        "create:mechanical_saw" => true,
        sticker::BLOCK => sticker::decode(state).is_some(),
        name => {
            let Some(name) = name.strip_prefix("minecraft:") else {
                return false;
            };
            if state.properties.is_empty() {
                return matches!(
                    name,
                    "slime_block"
                        | "honey_block"
                        | "stone"
                        | "smooth_stone"
                        | "cobblestone"
                        | "andesite"
                        | "polished_andesite"
                        | "granite"
                        | "diorite"
                        | "bricks"
                        | "stone_bricks"
                        | "glass"
                        | "terracotta"
                ) || name
                    .strip_suffix("_planks")
                    .is_some_and(|wood| WOOD_FAMILIES.contains(&wood))
                    || name.ends_with("_wool");
            }
            let wood = name.strip_prefix("stripped_").unwrap_or(name);
            let pillar = ["_log", "_wood", "_stem", "_hyphae", "_block"]
                .iter()
                .any(|suffix| {
                    wood.strip_suffix(suffix).is_some_and(|kind| {
                        if *suffix == "_block" {
                            kind == "bamboo"
                        } else {
                            WOOD_FAMILIES.contains(&kind)
                        }
                    })
                });
            pillar
                && state.properties.len() == 1
                && state
                    .properties
                    .get("axis")
                    .is_some_and(|axis| matches!(axis.as_str(), "x" | "y" | "z"))
        }
    }
}

/// The original events layer existing Minecraft samples at these gains.
pub(super) fn movement_sound(game: &mut Game<'_>, pos: Pos, assembled: bool) {
    let at = pos.map(|value| f64::from(value) + 0.5);
    game.with_level(|_, level, _| {
        if assembled {
            level.native_play_sound("minecraft:block.wooden_trapdoor.open", at, 4, 0.5, 0.5);
            level.native_play_sound("minecraft:block.chest.open", at, 4, 0.045, 0.74);
        } else {
            level.native_play_sound("minecraft:block.iron_trapdoor.close", at, 4, 0.35, 0.75);
        }
    });
}

pub(super) struct Capture<'g, 'a>(pub(super) &'g Game<'a>);
impl CaptureView for Capture<'_, '_> {
    type Block = BlockState;

    fn inspect(&self, p: Pos) -> Cell {
        let game = self.0;
        if !game.world.loaded(p[0], p[2]) {
            return Cell::Unloaded;
        }
        if !resident(game, p) {
            return Cell::OutsideWorld;
        }
        let id = state(game, p);
        if game.res.facts().is_air(id) {
            return Cell::Empty;
        }
        let Some(block) = game.pack.assets.registry.state(id) else {
            return Cell::Block(BlockRules::default());
        };
        if let Some(sail) = super::create_port_sail::from_state(block) {
            let attached = Direction::ALL.map(|face| sail.attaches(face));
            return Cell::Block(BlockRules {
                attached,
                unsupported_faces: attached.map(|face| !face),
                ..BlockRules::default()
            });
        }
        if let Some(sticker_state) = sticker::decode(block) {
            let attached = sticker_state.attachment().filter(|&face| {
                sticker::neighbor(p, face)
                    .and_then(|target| game.pack.assets.registry.state(state(game, target)))
                    .is_some_and(|target| sticker::target_supports(target, face.opposite()))
            });
            return Cell::Block(BlockRules {
                attached: Direction::ALL.map(|face| attached == Some(face)),
                unsupported_faces: Direction::ALL.map(|face| !sticker_state.supported_face(face)),
                ..BlockRules::default()
            });
        }
        let rules = BlockRules {
            movable: !matches!(
                block.name.as_str(),
                "minecraft:bedrock" | "minecraft:barrier" | "minecraft:obsidian"
            ),
            adhesion: match block.name.as_str() {
                "minecraft:slime_block" => Adhesion::Slime,
                "minecraft:honey_block" => Adhesion::Honey,
                _ => Adhesion::None,
            },
            ..BlockRules::default()
        };
        Cell::Block(rules)
    }

    fn snapshot(&self, p: Pos) -> Option<BlockState> {
        let game = self.0;
        let state = game.pack.assets.registry.state(state(game, p))?;
        if !supported(state) || state.name == sticker::BLOCK {
            return None;
        }
        if let Some(entity) = game.world.block_entities.get(&(p[0], p[1], p[2])) {
            let BlockEntity::Mod(entity) = entity else {
                return None;
            };
            // Only ordinary, idle tools with no inventory or foreign data
            // can move while block entity snapshots are still unsupported.
            let empty_tool = matches!(
                (state.name.as_str(), entity.id.as_str()),
                ("create:mechanical_saw", "create:saw")
                    | ("create:mechanical_harvester", "create:harvester")
                    | (
                        "create:mechanical_drill",
                        "create:drill" | "create:mechanical_drill"
                    )
            );
            if !empty_tool
                || entity.items.iter().any(Option::is_some)
                || super::create_port::integer(&entity.nbt, "Progress") != 0
                || entity.nbt.keys().any(|key| {
                    !matches!(
                        key.to_string().as_deref(),
                        Ok("Progress"
                            | "NextTick"
                            | "Breaking"
                            | "Speed"
                            | "Capacity"
                            | "Stress"
                            | "NetworkStatus"
                            | "create_horizon")
                    )
                })
            {
                return None;
            }
        }
        Some(state.clone())
    }

    fn glued(&self, _: Pos, _: Direction) -> bool {
        false
    }
}

pub(super) struct MountedCapture<'g, 'a> {
    base: Capture<'g, 'a>,
    allow_chest: bool,
    allow_actor_data: bool,
    allow_chassis: bool,
    allow_sticker: bool,
    glue: &'g [create_engine::contraption::glue::Volume],
}

impl<'g, 'a> MountedCapture<'g, 'a> {
    pub(super) fn for_linear(
        game: &'g Game<'a>,
        glue: &'g [create_engine::contraption::glue::Volume],
    ) -> Self {
        Self {
            base: Capture(game),
            allow_chest: true,
            allow_actor_data: false,
            allow_chassis: true,
            allow_sticker: false,
            glue,
        }
    }
}

/// Transfer saved inventories, chassis ranges and glue with the blocks.
/// The current payload origin can differ from the driver's zero-stroke anchor.
pub(super) fn commit_linear_capture(
    game: &mut Game<'_>,
    key: [i32; 2],
    mut records: Vec<Record>,
    record: Record,
    origin: Pos,
    edits: &[Edit],
) -> bool {
    use super::create_port_glue_transfer as glue;
    let Ok(ranges) = chassis::capture(game, origin, &record.blocks) else {
        return false;
    };
    let Ok(chest) = storage::capture(game, origin, &record.blocks) else {
        return false;
    };
    let Ok(update) = glue::capture(game, key, &record, origin) else {
        return false;
    };
    let controller = record.controller;
    records.push(record);
    glue::commit(game, key, update, |game| {
        chassis::commit_capture(game, key, controller, ranges, |game| {
            storage::commit_capture(game, key, records, edits, controller, chest)
        })
    })
}

pub(super) fn commit_linear_landing(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
    record: &Record,
    origin: Pos,
    edits: &[Edit],
) -> bool {
    use super::create_port_glue_transfer as glue;
    let turn = QuarterTurn::IDENTITY;
    let Ok(update) = glue::landing(game, key, record, origin, turn) else {
        return false;
    };
    glue::commit(game, key, update, |game| {
        chassis::commit_landing(game, key, record, origin, turn, edits, |game| {
            storage::commit_landing(game, key, records, edits, record, origin, turn)
        })
    })
}

impl CaptureView for MountedCapture<'_, '_> {
    type Block = BlockState;

    fn inspect(&self, p: Pos) -> Cell {
        self.base.inspect(p)
    }

    fn snapshot(&self, p: Pos) -> Option<BlockState> {
        let game = self.base.0;
        let block = game.pack.assets.registry.state(state(game, p))?;
        if (self.allow_chest && storage::can_capture(game, p, block))
            || (self.allow_actor_data
                && (deployer::can_capture(game, p, block) || psi::can_capture(game, p, block)))
            || (self.allow_chassis && chassis::can_capture(game, p, block))
            || (self.allow_sticker && sticker::can_capture(game, p, block))
        {
            return Some(block.clone());
        }
        self.base.snapshot(p)
    }

    fn glued(&self, p: Pos, face: Direction) -> bool {
        self.glue.iter().any(|volume| volume.glues(p, face))
    }

    fn expand(
        &self,
        p: Pos,
        forced: Option<Direction>,
    ) -> Result<Vec<Pos>, create_engine::contraption::CaptureError> {
        if self.allow_sticker {
            let game = self.base.0;
            let registry = game.pack.assets.registry;
            if let Some(sticker_state) = registry.state(state(game, p)).and_then(sticker::decode) {
                let Some(face) = sticker_state.attachment() else {
                    return Ok(Vec::new());
                };
                let target = sticker::neighbor(p, face)
                    .ok_or(create_engine::contraption::CaptureError::CoordinateOverflow)?;
                let side = face.opposite();
                return match self.base.inspect(target) {
                    Cell::Unloaded => {
                        Err(create_engine::contraption::CaptureError::Unloaded(target))
                    }
                    Cell::Block(_)
                        if registry
                            .state(state(game, target))
                            .is_some_and(|block| sticker::target_supports(block, side)) =>
                    {
                        Ok(vec![target])
                    }
                    _ => Ok(Vec::new()),
                };
            }
        }
        if !self.allow_chassis {
            return Ok(Vec::new());
        }
        chassis::expansion(self.base.0, p, forced, |target| self.base.inspect(target))
    }
}

fn assemble(
    game: &mut Game<'_>,
    controller: Pos,
    facing: Direction,
    windmill: bool,
    reversed: bool,
) -> Option<f32> {
    let vector = facing.vector();
    let anchor = std::array::from_fn(|i| controller[i] + vector[i]);
    let key = column(controller);
    if key != column(anchor) {
        return None;
    }
    let Ok(mut records) = entries(game, key) else {
        return None;
    };
    if records.len() == MAX_RECORDS || records.iter().any(|r| r.controller == controller) {
        return None;
    }
    let shape = {
        let glue = super::create_port_super_glue::read(game, key).ok()?;
        let allow_mounted_data = !windmill && facing.axis() == Axis::Y;
        let capture = MountedCapture {
            base: Capture(game),
            allow_chest: allow_mounted_data,
            allow_actor_data: allow_mounted_data,
            allow_chassis: allow_mounted_data,
            allow_sticker: !windmill,
            glue: &glue,
        };
        Assembly::capture(&capture, anchor, Some(controller), None, MAX_BLOCKS).ok()?
    };
    if shape.blocks().is_empty()
        || shape
            .blocks()
            .iter()
            .filter(|(_, block)| block.name == psi::BLOCK)
            .count()
            > 1
    {
        return None;
    }
    let generated = if windmill {
        let sails = shape
            .blocks()
            .iter()
            .filter(|(_, block)| {
                game.native_server_block_in_tag(block, create_engine::sail::WINDMILL_TAG)
                    .unwrap_or(false)
            })
            .count();
        let config = WindmillConfig::default();
        config.check_sails(sails).ok()?;
        let direction = if reversed {
            RotationDirection::CounterClockwise
        } else {
            RotationDirection::Clockwise
        };
        config.rpm(sails, direction)
    } else {
        0.0
    };
    let axis = facing.axis();
    for degrees in [0.0, 90.0, 180.0, 270.0] {
        let turn = Rotation::new(anchor, axis, degrees).unwrap().snapped();
        if shape.blocks().iter().any(|&(local, _)| {
            turn.place(anchor, local)
                .is_none_or(|p| column(p) != key || !resident(game, p))
                || local.iter().any(|v| v.unsigned_abs() > 15)
        }) {
            return None;
        }
    }
    let registry = game.pack.assets.registry;
    let Some(edits) = shape
        .blocks()
        .iter()
        .map(|&(local, ref block)| {
            let p = std::array::from_fn(|i| anchor[i] + local[i]);
            Some((p, registry.id(block)?, game.world.air))
        })
        .collect::<Option<Vec<_>>>()
    else {
        return None;
    };
    let chest = storage::capture(game, anchor, shape.blocks()).ok()?;
    let deployer = deployer::capture(game, anchor, shape.blocks()).ok()?;
    let chassis = chassis::capture(game, anchor, shape.blocks()).ok()?;
    let record = Record {
        controller,
        anchor,
        axis: axis.index(),
        angle: 0.0,
        previous_angle: 0.0,
        rpm: generated,
        return_home: windmill,
        stroke: None,
        clock_hand: None,
        blocks: shape.blocks().to_vec(),
    };
    let glue = super::create_port_glue_transfer::capture(game, key, &record, anchor).ok()?;
    records.push(record);
    if !super::create_port_glue_transfer::commit(game, key, glue, |game| {
        chassis::commit_capture(game, key, controller, chassis, |game| {
            deployer::commit_capture(game, key, records, &edits, controller, chest, deployer)
        })
    }) {
        return None;
    }
    movement_sound(game, controller, true);
    Some(generated)
}

fn rotate_state(state: &BlockState, turn: QuarterTurn) -> Option<BlockState> {
    if chassis::is_chassis(&state.name) {
        return chassis::rotate_state(state, turn);
    }
    if !supported(state)
        && !storage::chest_state(state)
        && !deployer::block_state(state)
        && !psi::block_state(state)
    {
        return None;
    }
    let mut rotated = state.clone();
    if state.name == deployer::BLOCK
        && turn.axis(Axis::X) != Axis::X
        && let Some(first) = rotated.properties.get_mut("axis_along_first")
    {
        *first = if first == "true" { "false" } else { "true" }.into();
    }
    if let Some(value) = rotated.properties.get_mut("axis") {
        let axis = match value.as_str() {
            "x" => Axis::X,
            "y" => Axis::Y,
            "z" => Axis::Z,
            _ => return None,
        };
        *value = ["x", "y", "z"][turn.axis(axis).index()].into();
    }
    if let Some(value) = rotated.properties.get_mut("facing") {
        let facing = Direction::ALL.into_iter().find(|f| f.name() == value)?;
        let direction = turn.place([0; 3], facing.vector())?;
        let facing = Direction::ALL
            .into_iter()
            .find(|f| f.vector() == direction)?;
        // The Java horizontal-facing state is left alone if the quarter turn
        // would point it vertically.
        if !actors::attached_actor(&state.name) || facing.axis() != Axis::Y {
            *value = facing.name().into();
        }
    }
    Some(rotated)
}

fn land(game: &mut Game<'_>, controller: Pos, angle: f32) -> bool {
    let key = column(controller);
    let Ok(mut records) = entries(game, key) else {
        return false;
    };
    let Some(index) = records.iter().position(|r| r.controller == controller) else {
        return false;
    };
    let record = records.remove(index);
    let Some(rotation) = Rotation::new(
        record.anchor,
        axis(record.axis),
        if record.return_home { 0.0 } else { angle },
    ) else {
        return false;
    };
    let Ok(plan) = plan_landing(
        &record.blocks,
        record.anchor,
        rotation.snapped(),
        rotate_state,
        |p, _| column(p) == key && resident(game, p) && game.res.facts().is_air(state(game, p)),
    ) else {
        return false;
    };
    let registry = game.pack.assets.registry;
    let Some(edits) = plan
        .iter()
        .map(|(p, block)| Some((*p, state(game, *p), registry.id(block)?)))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let turn = rotation.snapped();
    let Ok(glue) =
        super::create_port_glue_transfer::landing(game, key, &record, record.anchor, turn)
    else {
        return false;
    };
    if !super::create_port_glue_transfer::commit(game, key, glue, |game| {
        chassis::commit_landing(game, key, &record, record.anchor, turn, &edits, |game| {
            deployer::commit_landing(game, key, records, &edits, &record, turn)
        })
    }) {
        return false;
    }
    actors::remove(game, controller);
    psi::remove(game, controller);
    movement_sound(game, controller, false);
    true
}

pub(super) fn write_controller(entity: &mut ModBlockEntity, running: bool, angle: f32) {
    entity
        .nbt
        .insert(RUNNING.into(), Tag::Byte(i8::from(running)));
    entity
        .nbt
        .insert("Angle".into(), Tag::Float(angle.to_bits()));
}

pub(super) fn tick(
    game: &mut Game<'_>,
    pos: Pos,
    entity: &mut ModBlockEntity,
    raw_rpm: f32,
    rpm: f32,
) {
    let Some(facing) = facing(game, pos) else {
        return;
    };
    let key = column(pos);
    let Ok(records) = entries(game, key) else {
        return;
    };
    let current = records.iter().find(|r| r.controller == pos);
    let owned = entity.nbt.get(&RUNNING.into()) == Some(&Tag::Byte(1));
    // A new block at the same position must not take ownership of a surviving assembly.
    if current.is_some() && !owned {
        return;
    }
    let previous_raw = super::create_port::number(&entity.nbt, POWER);
    let previous_rpm = current.map_or(previous_raw, |r| r.rpm);
    let mut controller = current.map_or_else(Bearing::default, |r| Bearing::resume(r.angle));
    if (raw_rpm != previous_raw || rpm != previous_rpm)
        && controller.speed_changed(previous_rpm, rpm, actors::stalled(game, pos), None)
    {
        actors::remove(game, pos);
    }
    if entity.nbt.remove(&REQUEST.into()).is_some() && controller.toggle() {
        if land(game, pos, controller.angle()) {
            write_controller(entity, false, 0.0);
            entity
                .nbt
                .insert(POWER.into(), Tag::Float(raw_rpm.to_bits()));
        }
        return;
    }
    let mode = match super::create_port::integer(&entity.nbt, "ScrollValue") {
        1 => Mode::PlaceReturned,
        2 => Mode::NeverPlace,
        _ => Mode::Place,
    };
    let stalled = current.is_some_and(|record| {
        actors::tick_bearing_tools(game, record);
        let cutting = actors::stalled(game, pos);
        let docked = psi::advance(game, record);
        cutting || docked
    });
    let action = controller.tick(raw_rpm, rpm, mode, stalled, || {
        assemble(game, pos, facing, false, false).is_some()
    });
    if action == TickAction::Disassemble {
        if land(game, pos, controller.angle()) {
            write_controller(entity, false, 0.0);
            entity
                .nbt
                .insert(POWER.into(), Tag::Float(raw_rpm.to_bits()));
        }
        return;
    }
    entity
        .nbt
        .insert(POWER.into(), Tag::Float(raw_rpm.to_bits()));
    if controller.running() {
        let Ok(mut next) = entries(game, key) else {
            return;
        };
        let Some(record) = next.iter_mut().find(|r| r.controller == pos) else {
            return;
        };
        let previous = record.clone();
        record.previous_angle = record.angle;
        record.angle = controller.angle();
        record.rpm = rpm;
        let current = record.clone();
        if set_records(game, key, next).is_ok() {
            write_controller(entity, true, controller.angle());
            actors::tick_bearing_deployer(game, &previous, &current);
        }
    } else if owned {
        write_controller(entity, false, 0.0);
    }
}

pub(super) fn right_click(game: &mut Game<'_>, pos: Pos) -> bool {
    let block = game.native_block(pos);
    if !is_bearing(&block)
        || !game.native_on("create")
        || game.held().is_some()
        || game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
        || !game.may_build()
    {
        return false;
    }
    if block == clockwork::BLOCK {
        return clockwork::right_click(game, pos);
    }
    game.native_with_mod_block_entity(pos, |entity| {
        let request = if block == WINDMILL {
            WINDMILL_REQUEST
        } else {
            REQUEST
        };
        entity.nbt.insert(request.into(), Tag::Byte(1));
    })
    .is_some()
}

pub(super) fn recover_orphans(game: &mut Game<'_>) {
    let mut active = false;
    let mut orphans = Vec::new();
    for chunk in game.world.chunks.values() {
        let Ok(records) = read(chunk) else { continue };
        active |= !records.is_empty();
        orphans.extend(records.iter().filter_map(|record| {
            (record.clock_hand != Some(ClockHand::Minute)
                && record.stroke_kind() != Some(StrokeKind::Gantry)
                && !controller_present(game, record))
            .then_some((
                record.controller,
                record.angle,
                record.stroke_kind(),
                record.clock_hand.is_some(),
            ))
        }));
    }
    if active {
        let _ = game.world.attachments.set(&ACTIVE, true);
    } else {
        let _ = game.world.attachments.remove(&ACTIVE);
    }
    for (pos, angle, stroke, clock) in orphans {
        if stroke == Some(StrokeKind::RopePulley) {
            pulley::recover(game, pos);
        } else if stroke == Some(StrokeKind::Piston) {
            super::create_port_piston::recover(game, pos);
        } else if clock {
            clockwork::recover(game, pos);
        } else {
            land(game, pos, angle);
        }
    }
}

#[derive(Clone, Default)]
struct HarvesterBlades {
    tick: u64,
    blades: BTreeMap<(Pos, Pos), (DVec3, Blade)>,
}

const HARVESTER_BLADES: AttachmentType<HarvesterBlades> = AttachmentType {
    namespace: "create",
    name: HARVESTER_KEY,
    default: HarvesterBlades::default,
    codec: None,
    copy_on_death: false,
    sync: false,
};

fn actor_facing(state: &BlockState) -> Option<Direction> {
    let facing = state.properties.get("facing")?;
    Direction::ALL.into_iter().find(|d| d.name() == facing)
}

fn update_harvester_blades(map: &mut Attachments, records: &[Record], tick: u64, reset: bool) {
    let old = if reset {
        None
    } else {
        decoded(map, &HARVESTER_BLADES)
    };
    let mut blades = BTreeMap::new();
    for record in records {
        let rotation = record.orientation(record.angle);
        let pivot = DVec3::splat(0.5);
        let origin = DVec3::from_array(record.origin_at(1.0)) + pivot;
        for (local, block) in &record.blocks {
            if block.name != "create:mechanical_harvester" {
                continue;
            }
            let Some(facing) = actor_facing(block) else {
                continue;
            };
            let point = origin
                + rotation
                    * (DVec3::from_array(Tool::Harvester.active_point(*local, facing)) - pivot);
            let key = (record.controller, *local);
            let mut blade = Blade::default();
            if let Some(&(previous, phase)) = old.and_then(|v| v.blades.get(&key)) {
                let motion = point - previous;
                blade = phase;
                blade.tick(
                    motion.length(),
                    !points_backwards((rotation.conjugate() * motion).to_array(), facing),
                );
            }
            blades.insert(key, (point, blade));
        }
    }
    if blades.is_empty() {
        let _ = map.remove(&HARVESTER_BLADES);
    } else {
        let _ = map.set(&HARVESTER_BLADES, HarvesterBlades { tick, blades });
    }
}

pub(super) fn mirror_sync(game: &mut Game<'_>, data: &[u8]) {
    let Ok(message) = horizon_protocol::attachment_sync::decode(data) else {
        return;
    };
    let Holder::Chunk([x, z]) = message.holder else {
        return;
    };
    for change in message.changes {
        if change.namespace != "create".into() || change.name != KEY.into() {
            continue;
        }
        let Some(chunk) = game.world.chunks.get_mut(&(x, z)) else {
            return;
        };
        let chunk = Arc::make_mut(chunk);
        let extras = Arc::make_mut(chunk.saved.get_or_insert_with(Default::default));
        extras.attachments.apply_sync(change);
        if let Ok(records) = extras.attachments.get_existing(&RECORDS) {
            update_harvester_blades(
                &mut extras.attachments,
                records.as_deref().unwrap_or_default(),
                game.ticks,
                message.replace,
            );
        }
        let _ = game.world.attachments.set(&ACTIVE, true);
    }
}

fn draw_actor(
    game: &Game<'_>,
    draw: &mut BlockEntityDraw,
    record: &Record,
    local: Pos,
    state: &BlockState,
    blades: Option<&HarvesterBlades>,
) {
    if state.name == sticker::BLOCK {
        sticker::draw(game, draw, state);
        return;
    }
    if state.name == deployer::BLOCK {
        deployer::draw_moving(game, draw, record, local, state);
        return;
    }
    if state.name == psi::BLOCK {
        psi::draw_moving(game, draw, record, local, state);
        return;
    }
    let tool = match state.name.as_str() {
        "create:mechanical_harvester" => Tool::Harvester,
        "create:mechanical_saw" => Tool::Saw,
        "create:mechanical_drill" => Tool::Drill,
        _ => return,
    };
    let Some(facing) = actor_facing(state) else {
        return;
    };
    super::create_port::face_south(draw, facing);
    if tool == Tool::Harvester {
        let angle = blades
            .and_then(|v| {
                let (_, blade) = v.blades.get(&(record.controller, local))?;
                let partial = if game.ticks <= v.tick.saturating_add(1) {
                    draw.partial
                } else {
                    1.0
                };
                Some(blade.angle(partial))
            })
            .unwrap_or_default();
        draw.rotate_about([1.0, 0.0, 0.0], angle, [0.5, 6.0 / 16.0, 9.0 / 16.0]);
        game.native_draw_model(
            draw,
            "create:block/mechanical_harvester/moving_blade",
            0xFFFFFF,
        );
        return;
    }
    let pivot = DVec3::splat(0.5);
    let point = DVec3::from_array(tool.active_point(local, facing)) - pivot;
    let before = record.orientation(record.previous_angle);
    let after = record.orientation(record.angle);
    let motion = DVec3::from_array(record.origin_at(1.0))
        - DVec3::from_array(record.origin_at(0.0))
        + after * point
        - before * point;
    let backwards = points_backwards((after.conjugate() * motion).to_array(), facing);
    let length = motion.length();
    let moving = length >= 1.0 / 512.0;
    match tool {
        Tool::Plough | Tool::Harvester => {}
        Tool::Drill => {
            let speed = if !moving && record.rpm != 0.0 {
                700.0
            } else if moving && !backwards {
                ((-length * 1000.0 - 100.0) as i32 / 100 * 100) as f64
            } else {
                0.0
            };
            let time = (draw.game_time as f64 + f64::from(draw.partial)) / 20.0;
            draw.rotate_about([0.0, 0.0, 1.0], ((time * speed) % 360.0) as f32, [0.5; 3]);
            game.native_draw_model(draw, "create:block/mechanical_drill/head", 0xFFFFFF);
        }
        Tool::Saw => {
            let vertical = facing.axis() == Axis::Y;
            if vertical
                && state
                    .properties
                    .get("axis_along_first")
                    .is_some_and(|s| s == "true")
            {
                draw.rotate_about([0.0, 0.0, 1.0], 90.0, [0.5; 3]);
            }
            let direction = after * DVec3::from_array(facing.vector().map(f64::from));
            let horizontal_in_world = direction.x.abs().max(direction.z.abs()) > direction.y.abs();
            // For the moving structures supported here, a powered saw that
            // has halted is waiting for its cut to finish.
            let cutting = !moving && record.rpm != 0.0 && horizontal_in_world;
            let active = cutting || (moving && !backwards);
            let model = match (vertical, active) {
                (true, true) => "create:block/mechanical_saw/moving_vertical",
                (true, false) => "create:block/mechanical_saw/moving_vertical_idle",
                (false, true) => "create:block/mechanical_saw/moving_horizontal",
                (false, false) => "create:block/mechanical_saw/moving_horizontal_idle",
            };
            game.native_draw_model(draw, model, 0xFFFFFF);
        }
    }
}

pub(super) fn draw_structures(game: &Game<'_>, draw: &mut NativeWorldDraw) {
    use horizon_client::block_entity_render::vertex_light;
    let camera = DVec3::from_array(draw.camera);
    for chunk in game.world.chunks.values() {
        if (f64::from(chunk.x) * 16.0 + 8.0 - camera.x).abs() > 80.0
            || (f64::from(chunk.z) * 16.0 + 8.0 - camera.z).abs() > 80.0
        {
            continue;
        }
        let Ok(records) = read(chunk) else { continue };
        let blades = chunk
            .saved
            .as_ref()
            .and_then(|saved| decoded(&saved.attachments, &HARVESTER_BLADES));
        for record in records.iter() {
            pulley::draw_line(game, draw, record);
            let anchor = record.origin_at(draw.partial);
            if DVec3::from_array(anchor).distance_squared(camera) > 64.0 * 64.0 {
                continue;
            }
            draw.push();
            draw.translate_world(anchor);
            let mut axis = [0.0; 3];
            axis[record.axis] = 1.0;
            let angle = record.angle_at(draw.partial);
            draw.rotate_about(axis, angle, [0.5; 3]);
            let [x, y, z] = record.anchor;
            let previous_light = draw.light;
            draw.light = vertex_light(game.world.light(x, y, z));
            for (local, block) in &record.blocks {
                let properties: Vec<_> = block
                    .properties
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                draw.push();
                draw.translate(local.map(|v| v as f32));
                let chest = block.name == "minecraft:chest"
                    && game.pack.assets.registry.id(block).is_some_and(|id| {
                        horizon_client_render::block_entity_render::create_contraption_chest::closed(
                            game.res.block_entity_render(),
                            &game.gui.atlas,
                            id.0,
                            game.dim == super::Dim::Nether,
                            draw.pose(),
                            draw.light,
                            draw.vertices_mut(),
                        )
                    });
                if !chest {
                    game.native_draw_block(draw, &block.name, &properties);
                }
                draw_actor(game, draw, record, *local, block, blades);
                draw.pop();
            }
            draw.light = previous_light;
            draw.pop();
        }
    }
}

pub(super) fn draw_bearing(game: &Game<'_>, pos: Pos, draw: &mut BlockEntityDraw) {
    let Some(facing) = facing(game, pos) else {
        return;
    };
    let angle = entries(game, column(pos))
        .ok()
        .and_then(|records| {
            records.into_iter().find(|record| {
                record.controller == pos && record.clock_hand != Some(ClockHand::Minute)
            })
        })
        .map_or(0.0, |record| record.angle_at(draw.partial));
    let mut axis = [0.0; 3];
    axis[facing.axis().index()] = 1.0;
    draw.push();
    draw.rotate_about(axis, angle, [0.5; 3]);
    match facing {
        Direction::Up => {}
        Direction::Down => draw.rotate_about([1.0, 0.0, 0.0], 180.0, [0.5; 3]),
        Direction::North => draw.rotate_about([1.0, 0.0, 0.0], -90.0, [0.5; 3]),
        Direction::South => draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.5; 3]),
        Direction::West => draw.rotate_about([0.0, 0.0, 1.0], 90.0, [0.5; 3]),
        Direction::East => draw.rotate_about([0.0, 0.0, 1.0], -90.0, [0.5; 3]),
    }
    let top = if game.native_block(pos) == WINDMILL {
        "create:block/bearing/top_wooden"
    } else {
        "create:block/bearing/top"
    };
    game.native_draw_model(draw, top, 0xffffff);
    draw.pop();
}

/// The windmill's queued assembly creates its own power before network evaluation.
pub(super) fn windmill_source(
    game: &mut Game<'_>,
    pos: Pos,
    entity: &mut ModBlockEntity,
) -> (f32, bool) {
    let previous = super::create_port::number(&entity.nbt, LAST_GENERATED);
    let mut generated = previous;
    if entity.nbt.remove(&WINDMILL_REQUEST.into()).is_some() {
        if entity.nbt.get(&RUNNING.into()) == Some(&Tag::Byte(1)) {
            if land(game, pos, 0.0) {
                write_controller(entity, false, 0.0);
                generated = 0.0;
            }
        } else if let Some(direction) = facing(game, pos) {
            let reversed = super::create_port::integer(&entity.nbt, "ScrollValue") == 1;
            if let Some(rpm) = assemble(game, pos, direction, true, reversed) {
                write_controller(entity, true, 0.0);
                generated = rpm;
            }
        }
    }
    if !matches!(entity.nbt.get(&RUNNING.into()), Some(Tag::Byte(1))) {
        generated = 0.0;
    }
    entity
        .nbt
        .insert(LAST_GENERATED.into(), Tag::Float(generated.to_bits()));
    (generated, generated != previous)
}

pub(super) fn tick_windmill(
    game: &mut Game<'_>,
    pos: Pos,
    entity: &mut ModBlockEntity,
    effective: f32,
) {
    let key = column(pos);
    let Ok(mut records) = entries(game, key) else {
        return;
    };
    let Some(record) = records
        .iter_mut()
        .find(|record| record.controller == pos && record.return_home)
    else {
        return;
    };
    if entity.nbt.get(&RUNNING.into()) != Some(&Tag::Byte(1)) {
        return;
    }
    let mut driver = Windmill::restored(true, record.angle, record.rpm, false);
    let previous = record.clone();
    driver.finish_tick(record.rpm, effective, actors::stalled(game, pos));
    record.previous_angle = record.angle;
    record.angle = driver.angle();
    let current = record.clone();
    if set_records(game, key, records).is_ok() {
        write_controller(entity, true, driver.angle());
        actors::tick(game, &previous, &current);
    }
}
