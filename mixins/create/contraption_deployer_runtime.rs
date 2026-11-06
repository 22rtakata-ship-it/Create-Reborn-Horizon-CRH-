// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

// Bounded stationary item use and mounted sapling planting.

use super::{
    Game, create_port as kinetics, create_port_contraption as contraption,
    create_port_contraption_storage as storage, native_block_entity_draw::BlockEntityDraw,
};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModBlockEntityFields},
    items::ItemStack,
};
use create_engine::{
    contraption::QuarterTurn,
    deployer::{
        Aim, Held, Mode, MovingAction, Phase, Segment, Stationary, Target, TickAction, Waiting,
        grabs_from_storage, moving_action, should_use, stationary_aim, timer_speed,
    },
    kinetics::{Axis, Pos},
    placement::Direction,
};
use glam::DVec3;
use horizon_assets::item_model::DisplayContext;
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
const UPDATE_TICK: &str = "create_horizon:DeployerTick";
const BONE_MEAL: &str = "minecraft:bone_meal";
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
    let mut update: Compound = ["Mode", "HeldItem", "State", "Timer", "Speed", "Powered"]
        .into_iter()
        .filter_map(|key| get(&entity.nbt, key).map(|tag| (key.into(), tag.clone())))
        .collect();
    let reach = entity.data.first().copied().unwrap_or(0) as u32;
    update.insert("Reach".into(), Tag::Float(reach));
    if let Some(&tick) = entity.data.get(1) {
        update.insert(UPDATE_TICK.into(), Tag::Int(tick));
    }
    update
}

fn draw_arm(
    game: &Game<'_>,
    draw: &mut BlockEntityDraw,
    facing: Direction,
    first: bool,
    shaft_angle: f32,
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
    draw.rotate_about(
        create_engine::visual::positive_axis(shaft_axis),
        shaft_angle,
        [0.5; 3],
    );
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
    let rpm = kinetics::number(&entity.nbt, "Speed");
    let arm = station_state(&entity.nbt).map_or(0.0, |mut cycle| {
        cycle.reach = kinetics::number(&entity.nbt, "Reach");
        if let Some(Tag::Int(sample)) = get(&entity.nbt, UPDATE_TICK) {
            let elapsed = (draw.game_time as i32).wrapping_sub(*sample).clamp(0, 128);
            cycle.timer = (cycle.timer - elapsed * timer_speed(rpm)).max(0);
        }
        cycle.hand_extension(rpm, draw.partial, holding)
    });
    let shaft_angle = kinetics::node(game, pos).map_or(0.0, |node| {
        create_engine::visual::angle(
            draw.game_time as f64 + f64::from(draw.partial),
            rpm,
            create_engine::visual::rotation_offset(pos, node.axis, false),
        )
    });
    draw_arm(game, draw, facing, first, shaft_angle, arm, holding);
    if get(&entity.nbt, "Mode") == Some(&Tag::String("USE".into()))
        && let Some(stack) = get(&entity.nbt, "HeldItem")
            .and_then(item_from_nbt)
            .filter(supported_item)
    {
        draw_held_item(
            game,
            draw,
            facing,
            kinetics::running_speed(game, &entity.nbt),
            arm,
            &stack,
        );
    }
}

// DeployerRenderer.renderItem: the supported items use flat generated models.
fn draw_held_item(
    game: &Game<'_>,
    draw: &mut BlockEntityDraw,
    facing: Direction,
    rpm: f32,
    extension: f32,
    stack: &ItemStack,
) {
    draw.push();
    draw.translate(
        facing
            .vector()
            .map(|component| 0.5 + component as f32 * extension),
    );
    let yaw = match facing {
        Direction::North => 0.0,
        Direction::East => -90.0,
        Direction::West => 90.0,
        _ => 180.0,
    };
    draw.rotate_about([0., 1., 0.], yaw, [0.; 3]);
    let display = if facing == Direction::Up && rpm == 0.0 {
        draw.translate([0., 11. / 16., 0.]);
        // Catnip's client animation clock advances one degree per tick.
        let spin = super::create_port_particles::animation_time(draw.partial);
        draw.rotate_about([0., 1., 0.], spin, [0.; 3]);
        DisplayContext::Ground
    } else {
        let pitch = match facing {
            Direction::Up => 90.0,
            Direction::Down => -90.0,
            _ => 0.0,
        };
        draw.rotate_about([1., 0., 0.], pitch, [0.; 3]);
        draw.translate([0., 0., -11. / 16.]);
        draw.scale([0.5; 3]);
        DisplayContext::Fixed
    };
    game.native_draw_item(draw, stack, display, None, 0);
    draw.pop();
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
    draw_arm(game, draw, facing, first, 0.0, extension, false);
}

fn get<'a>(nbt: &'a Compound, key: &str) -> Option<&'a Tag> {
    nbt.get(&key.into())
}

fn supported_item(item: &ItemStack) -> bool {
    (SAPLINGS.contains(&item.item.as_ref()) || item.item.as_ref() == BONE_MEAL)
        && (1..=64).contains(&item.count)
        && item.damage == 0
        && item.data().is_none()
}

fn mounted_item(item: &ItemStack) -> bool {
    item.item.as_ref() != BONE_MEAL && supported_item(item)
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
                Ok("Timer") => matches!(val, Tag::Int(-511..=5120)),
                Ok("Powered") => *val == Tag::Byte(0),
                Ok("Owner") => matches!(val, Tag::IntArray(uuid) if uuid.len() == 4),
                Ok("Overflow") => matches!(val, Tag::List(items) if items.is_empty()),
                Ok("Filter") => true,
                Ok("FilterAmount") => matches!(val, Tag::Int(0..=64)),
                Ok("UpTo") => matches!(val, Tag::Byte(0 | 1)),
                Ok("Speed") => *val == Tag::Float(0),
                Ok("Stress" | "Capacity") => matches!(val, Tag::Float(_)),
                Ok("NetworkStatus") => matches!(val, Tag::String(_)),
                Ok("create_horizon") => detached_ports(val),
                _ => false,
            })
        && read_filter(nbt).is_some_and(|filter| {
            read_hand(nbt).is_some_and(|hand| {
                hand.as_ref().is_none_or(mounted_item)
                    && filter.as_ref().is_none_or(|filter| {
                        mounted_item(filter)
                            && hand.as_ref().is_none_or(|held| held.item == filter.item)
                    })
            })
        })
}

fn detached_ports(tag: &Tag) -> bool {
    let Tag::Compound(data) = tag else {
        return false;
    };
    data.len() == 2
        && get(data, "TheoreticalSpeed") == Some(&Tag::Float(0))
        && matches!(get(data, "Ports"), Some(Tag::Compound(ports)) if ports.len() == 6
            && Direction::ALL.into_iter().all(|face| get(ports, face.name()) == Some(&Tag::Float(0)))
        )
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

fn consistent_hand(entity: &ModBlockEntity) -> Option<Option<ItemStack>> {
    let hand = read_hand(&entity.nbt)?;
    // Java-shaped commands may initialize only Inventory. The native slot
    // tracks hand swaps and is spillable when an ordinary block breaks.
    (entity.items.len() <= 1
        && entity
            .items
            .first()
            .is_none_or(|item| item.is_none() || *item == hand))
    .then_some(hand)
}

fn entity<'a>(game: &'a Game<'_>, pos: Pos) -> Option<&'a ModBlockEntity> {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return None;
    };
    (entity.id == BLOCK && consistent_hand(entity).is_some()).then_some(entity)
}

fn station_state(nbt: &Compound) -> Option<Stationary> {
    if get(nbt, "Mode") != Some(&Tag::String("USE".into())) {
        return None;
    }
    let phase = match get(nbt, "State") {
        None => Phase::Waiting,
        Some(Tag::String(phase)) => match phase.to_string().ok()?.as_str() {
            "WAITING" => Phase::Waiting,
            "EXPANDING" => Phase::Expanding,
            "RETRACTING" => Phase::Retracting,
            _ => return None,
        },
        _ => return None,
    };
    let timer = match get(nbt, "Timer") {
        None => 0,
        Some(Tag::Int(timer)) if (-511..=5120).contains(timer) => *timer,
        _ => return None,
    };
    Some(Stationary {
        phase,
        timer,
        ..Stationary::default()
    })
}

pub(super) fn tick_station(
    game: &mut Game<'_>,
    pos: Pos,
    entity: &mut ModBlockEntity,
    rpm: f32,
) -> bool {
    if contraption::facing(game, pos) != Some(Direction::Down)
        || !matches!(get(&entity.nbt, "Powered"), None | Some(Tag::Byte(0 | 1)))
        || !get(&entity.nbt, "Overflow")
            .is_none_or(|v| matches!(v, Tag::List(items) if items.is_empty()))
    {
        return false;
    }
    let (Some(aim), Some(mut cycle), Some(hand), Some(filter)) = (
        stationary_aim(pos, Direction::Down),
        station_state(&entity.nbt),
        consistent_hand(entity),
        read_filter(&entity.nbt),
    ) else {
        return false;
    };
    if filter.is_some_and(|filter| hand.as_ref().is_some_and(|held| held.item != filter.item)) {
        return false;
    }
    let target = aim.clicked;
    let dim = game.dim.id().to_owned();
    let powered = i8::from(game.native_neighbor_signal(&dim, pos) != 0);
    let mut changed = get(&entity.nbt, "Powered") != Some(&Tag::Byte(powered));
    if changed {
        entity.nbt.insert("Powered".into(), Tag::Byte(powered));
    }
    if let Some(stack) = &hand
        && entity.items.first().is_none_or(Option::is_none)
    {
        entity.items = vec![Some(stack.clone())];
        write_hand(&mut entity.nbt, hand.clone());
        changed = true;
    }
    cycle.reach = f32::from_bits(entity.data.first().copied().unwrap_or(0) as u32);
    let previous = cycle;
    let action = cycle.tick(rpm, |_| {
        if !contraption::resident(game, target) {
            return Waiting::Locked;
        }
        let block = game.native_block(target);
        if !should_use(
            hand.as_ref().map_or(Held::Empty, |stack| {
                if stack.item.as_ref() == BONE_MEAL {
                    Held::Other
                } else {
                    Held::Block(&stack.item)
                }
            }),
            Target {
                block: &block,
                fluid: None,
                transports_items: matches!(block.as_str(), "create:belt" | "create:depot"),
            },
            Some(Direction::Down),
        ) {
            return Waiting::Retry;
        }
        if powered != 0 {
            return Waiting::Locked;
        }
        let probe = aim.reach_probe();
        Waiting::Start {
            reach: aim.reach(clip(game, probe).map_or(probe.end, |hit| hit.point)),
        }
    });
    if cycle != previous || changed || entity.data.len() != 2 {
        entity.data = vec![cycle.reach.to_bits() as i32, game.world.game_time as i32];
        if cycle.timer != previous.timer || get(&entity.nbt, "Timer").is_none() {
            entity.nbt.insert("Timer".into(), Tag::Int(cycle.timer));
        }
        if cycle.phase != previous.phase || get(&entity.nbt, "State").is_none() {
            entity
                .nbt
                .insert("State".into(), Tag::String(cycle.phase.saved_name().into()));
        }
    }
    if matches!(action, TickAction::Activate { .. })
        && let Some(mut stack) = hand
        && use_stationary_item(game, aim, &stack)
    {
        stack.count -= 1;
        let remaining = (stack.count != 0).then_some(stack);
        entity.items = vec![remaining.clone()];
        write_hand(&mut entity.nbt, remaining);
    }
    changed || action != TickAction::None
}

fn use_stationary_item(game: &mut Game<'_>, aim: Aim, stack: &ItemStack) -> bool {
    if stack.item.as_ref() != BONE_MEAL {
        return sapling_context(game, aim, stack, Face::Up)
            .is_some_and(|context| place_sapling(game, &context));
    }
    let [x, y, z] = aim.clicked;
    if !contraption::resident(game, aim.clicked)
        || !game.border.contains(f64::from(x), f64::from(z))
        || !matches!(
            game.native_block(aim.clicked).as_str(),
            "minecraft:oak_sapling"
                | "minecraft:wheat"
                | "minecraft:carrots"
                | "minecraft:potatoes"
                | "minecraft:beetroots"
        )
    {
        return false;
    }
    // DeployerHandler forces the action ray onto the prescribed cell and
    // preserves the ray face. Vanilla growth reports use even on failed rolls.
    let face = clip(game, aim.action_ray())
        .and_then(|hit| native_face(hit.face))
        .unwrap_or(Face::Up);
    let used = game
        .with_level(|b, level, ticks| b.use_bone_meal(level, ticks, BlockPos::new(x, y, z), face));
    let _ = game.process_level_events();
    used
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
    let (Some(previous), Some(filter)) = (read_hand(&entity.nbt), read_filter(&entity.nbt)) else {
        return false;
    };
    if let (Some(item), Some(filter)) = (held.as_ref(), filter)
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
        || !entity(game, pos).is_some_and(|entity| valid_nbt(&entity.nbt))
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
    let entity = entity(game, pos)
        .filter(|entity| valid_nbt(&entity.nbt))
        .ok_or(AttachmentError::Encode)?;
    let mut nbt = entity.nbt.clone();
    for key in [
        "Speed",
        "Stress",
        "Capacity",
        "NetworkStatus",
        "create_horizon",
    ] {
        nbt.remove(&key.into());
    }
    Ok(Some(Cargo { local, nbt }))
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
        Ok(None) => {
            return storage::commit_landing(game, key, records, edits, record, record.anchor, turn);
        }
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
        |game| storage::commit_landing(game, key, records, edits, record, record.anchor, turn),
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
                mounted_item(item)
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
    if key != contraption::column(target) {
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
    let fallback = if (ray.end[0] - ray.start[0]).abs() > (ray.end[2] - ray.start[2]).abs() {
        if ray.end[0] > ray.start[0] {
            Face::West
        } else {
            Face::East
        }
    } else if ray.end[2] > ray.start[2] {
        Face::North
    } else {
        Face::South
    };
    let Some(ctx) = sapling_context(game, aim, &hand, fallback) else {
        return;
    };
    hand.count -= 1;
    update_hand(
        game,
        key,
        record.controller,
        local,
        (hand.count > 0).then_some(hand),
        |game| place_sapling(game, &ctx),
    );
}

fn clip(game: &Game<'_>, ray: Segment) -> Option<super::Hit> {
    let segment = DVec3::from_array(ray.end) - DVec3::from_array(ray.start);
    let length = segment.length();
    if !length.is_finite() || length == 0.0 {
        return None;
    }
    game.clip_ray(ray.start, (segment / length).to_array(), length)
}

fn sapling_context(
    game: &mut Game<'_>,
    aim: Aim,
    stack: &ItemStack,
    fallback: Face,
) -> Option<PlaceContext> {
    let target = aim.clicked;
    if !contraption::resident(game, target)
        || !game
            .border
            .contains(f64::from(target[0]), f64::from(target[2]))
        || !game.res.facts().is_air(contraption::state(game, target))
    {
        return None;
    }
    let ray = aim.action_ray();
    let (face, point) = clip(game, ray).map_or((fallback, ray.end), |hit| {
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
        item: stack.item.to_string(),
        can_use_game_master_blocks: false,
    };
    game.with_level(|b, l, t| b.placement_state(l, t, &ctx))
        .is_some_and(|(at, _)| at == pos)
        .then_some(ctx)
}

fn place_sapling(game: &mut Game<'_>, context: &PlaceContext) -> bool {
    let placed = game.with_level(|b, l, t| b.place_block(l, t, context));
    let _ = game.process_level_events();
    matches!(placed, PlaceOutcome::Placed { .. })
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
