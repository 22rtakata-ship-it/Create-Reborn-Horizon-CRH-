// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Loaded connected-component equations are cached per tick. Source generation
// is published by initialize/updateGeneratedRotation, not by reading a setting.
// This preserves supported stationary-machine event ordering; it is not a
// complete RotationPropagator implementation (source ownership/flicker/breakage).
use super::Game;
use crate::containers::{BlockEntity, ModBlockEntity, ModRevision};
use create_engine::{
    kinetics::{self, Kind, Network, Node, Pos},
    transport::SessionCache,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
#[derive(Default)]
struct Cache {
    world: Option<(usize, i64, String)>,
    components: BTreeMap<(Pos, u8), Rc<Network>>,
}
#[derive(Default)]
struct Lifetime {
    owner: Option<ModRevision>,
    attached: bool,
    waiting_for_neighbour: bool,
    initialized_source: bool,
    generation: Option<f32>,
}
const MAX_PENDING_TRANSMISSIONS: usize = 2048;
#[derive(Default)]
struct Pending {
    positions: BTreeSet<Pos>,
    scan_loaded: bool,
}
thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
    static CHANGED: RefCell<BTreeMap<String, Pending>> = RefCell::new(BTreeMap::new());
    static LIFETIMES: RefCell<SessionCache<BTreeMap<Pos, Lifetime>>> = RefCell::new(SessionCache::default());
}
fn with_lifetime<R>(
    game: &Game<'_>,
    pos: Pos,
    entity: &ModBlockEntity,
    action: impl FnOnce(&mut Lifetime) -> R,
) -> R {
    LIFETIMES.with(|cell| {
        let mut cache = cell.borrow_mut();
        let world = cache.dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id());
        let state = world.entry(pos).or_default();
        let owner = entity.revision();
        if state.owner.is_some_and(|old| !old.same_entity(owner)) {
            *state = Lifetime::default();
        }
        state.owner = Some(owner);
        action(state)
    })
}
pub(super) fn reset() {
    invalidate();
    LIFETIMES.with(|cell| cell.borrow_mut().clear());
    CHANGED.with(|cell| cell.borrow_mut().clear());
}
pub(super) fn invalidate() {
    CACHE.with(|c| c.borrow_mut().components.clear());
}
fn theoretical(entity: &ModBlockEntity) -> f32 {
    match entity.nbt.get(&"create_horizon".into()) {
        Some(horizon_world::nbt::Tag::Compound(data)) => {
            super::create_port::number(data, "TheoreticalSpeed")
        }
        _ => super::create_port::number(&entity.nbt, "Speed"),
    }
}
fn generation(game: &Game<'_>, pos: Pos, entity: &ModBlockEntity) -> f32 {
    with_lifetime(game, pos, entity, |state| {
        *state.generation.get_or_insert_with(|| theoretical(entity))
    })
}
fn published_node(game: &Game<'_>, mut node: Node) -> Node {
    if let Some(BlockEntity::Mod(entity)) =
        game.world
            .block_entities
            .get(&(node.pos[0], node.pos[1], node.pos[2]))
        && !entity.id.is_empty()
        && let Kind::Motor { rpm, .. } | Kind::WaterWheel { rpm, .. } = &mut node.kind
    {
        *rpm = generation(game, node.pos, entity);
    }
    node
}
/// CreativeMotor.initialize publishes its setting. Other generators require
/// their own real turn/expiry/flow callback even on their first BE turn. Reading ScrollValue via /data is not that event.
pub(super) fn source_turn(
    game: &Game<'_>,
    pos: Pos,
    entity: &ModBlockEntity,
    requested: f32,
    initialize_motor: bool,
    changed: bool,
) -> (f32, bool) {
    let result = with_lifetime(game, pos, entity, |state| {
        state.generation.get_or_insert_with(|| theoretical(entity));
        let publish = !state.initialized_source && initialize_motor || changed;
        state.initialized_source = true;
        if publish {
            state.generation = Some(requested);
        }
        (state.generation.unwrap_or(0.0), publish)
    });
    if result.1 {
        invalidate();
    }
    result
}
/// A newborn passive node attaches from an immediate neighbour's *actual*
/// rotation. A distant loaded motor behind an unticked shaft is not a shortcut.
/// Synchronous propagation may already have supplied this node before its turn.
pub(super) fn attach(
    game: &Game<'_>,
    node: Node,
    entity: &ModBlockEntity,
    potential: f32,
) -> (bool, bool) {
    let (first, waiting) = with_lifetime(game, node.pos, entity, |state| {
        let first = !state.attached;
        state.attached = true;
        (first, state.waiting_for_neighbour)
    });
    if first && std::env::var_os("CREATE_HORIZON_TRACE").is_some() {
        log::info!(
            "Create kinetic turn: {}",
            serde_json::json!({
                "time": game.world.game_time, "pos": node.pos,
                "speed": super::create_port::number(&entity.nbt, "Speed"),
                "needs_update": super::create_port::integer(&entity.nbt, "NeedsUpdate"),
            })
        );
    }
    if (!first && !waiting) || theoretical(entity) != 0.0 {
        with_lifetime(game, node.pos, entity, |state| {
            state.waiting_for_neighbour = false
        });
        return (first, false);
    }
    if potential != 0.0 {
        for x in -1i32..=1 {
            for y in -1i32..=1 {
                for z in -1i32..=1 {
                    if !(1..=2).contains(&(x.abs() + y.abs() + z.abs())) {
                        continue;
                    }
                    let pos = [node.pos[0] + x, node.pos[1] + y, node.pos[2] + z];
                    let Some(other) = super::create_port::node(game, pos) else {
                        continue;
                    };
                    if kinetics::ratio(node, other).is_none() {
                        continue;
                    }
                    if let Some(BlockEntity::Mod(neighbour)) =
                        game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
                        && !neighbour.id.is_empty()
                        && theoretical(neighbour) != 0.0
                    {
                        with_lifetime(game, node.pos, entity, |state| {
                            state.waiting_for_neighbour = false
                        });
                        return (true, false);
                    }
                }
            }
        }
    }
    with_lifetime(game, node.pos, entity, |state| {
        state.waiting_for_neighbour = true
    });
    (first, true)
}
pub(super) fn network(game: &Game<'_>, start: Node, half: u8) -> Rc<Network> {
    let start = published_node(game, start);
    CACHE.with(|cell| {
        let mut cache = cell.borrow_mut();
        let world = (
            std::ptr::from_ref(&game.world) as usize,
            game.world.game_time,
            game.dim.id().to_owned(),
        );
        if cache.world.as_ref() != Some(&world) {
            cache.world = Some(world);
            cache.components.clear();
        }
        if let Some(network) = cache.components.get(&(start.pos, half)) {
            return Rc::clone(network);
        }
        let network = Rc::new(kinetics::solve_half(start, half, |pos| {
            if pos == start.pos {
                Some(start)
            } else {
                super::create_port::node(game, pos).map(|node| published_node(game, node))
            }
        }));
        for key in &network.vertices {
            cache.components.insert(*key, Rc::clone(&network));
        }
        network
    })
}
/// updateGeneratedRotation/attachKinetics publishes to neighbours immediately,
/// including machines whose behaviour turn already ran in this level pass.
pub(super) fn publish(game: &mut Game<'_>, network: &Network, current: Pos) {
    for &(pos, _) in &network.vertices {
        if pos == current {
            continue;
        }
        let Some(node) = super::create_port::node(game, pos) else {
            continue;
        };
        let other = matches!(node.kind, Kind::Clutch { powered: true }).then(|| {
            let half = network
                .vertices
                .iter()
                .find(|(p, _)| *p == pos)
                .map_or(0, |(_, h)| 1 - *h);
            self::network(game, node, half)
        });
        let Some(BlockEntity::Mod(entity)) =
            game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
        else {
            continue;
        };
        if entity.id.is_empty()
            || super::create_port::network_matches(entity, network, other.as_deref(), pos)
        {
            continue;
        }
        game.native_with_mod_block_entity(pos, |entity| {
            super::create_port::apply_network(entity, network, other.as_deref(), pos);
        });
        game.native_block_entity_sync(pos);
    }
}
/// Ordinary source-setting callback while the source BE is in the world.
pub(super) fn generated_changed(game: &mut Game<'_>, pos: Pos) {
    let Some(mut node) = super::create_port::node(game, pos) else {
        return;
    };
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return;
    };
    let (Kind::Motor { rpm, .. } | Kind::WaterWheel { rpm, .. }) = &mut node.kind else {
        return;
    };
    (*rpm, _) = source_turn(game, pos, entity, *rpm, false, true);
    let network = self::network(game, node, 0);
    game.native_with_mod_block_entity(pos, |entity| {
        super::create_port::apply_network(entity, &network, None, pos);
    });
    publish(game, &network, pos);
    game.native_block_entity_sync(pos);
}

/// The block notification hook has no Game borrow. Retain bounded nearby
/// transmission candidates for the next ordinary level's block phase.
pub(super) fn changed(dimension: &str, pos: Pos) {
    invalidate();
    CHANGED.with(|cell| {
        let mut changes = cell.borrow_mut();
        let pending = changes.entry(dimension.to_owned()).or_default();
        if pending.scan_loaded {
            return;
        }
        if pending.positions.len() + 7 > MAX_PENDING_TRANSMISSIONS {
            pending.positions.clear();
            pending.scan_loaded = true;
            return;
        }
        pending.positions.insert(pos);
        for direction in create_engine::placement::Direction::ALL {
            let delta = direction.vector();
            pending
                .positions
                .insert(std::array::from_fn(|i| pos[i] + delta[i]));
        }
    });
}
/// Clutch/Gearshift reopening is a scheduled block tick before the BE pass.
/// This preserves that next-tick machine boundary. It does not claim immediate
/// between-tick neighbour callback parity: the current hook has no Game owner.
pub(super) fn refresh_transmissions(game: &mut Game<'_>) {
    let pending = CHANGED.with(|cell| cell.borrow_mut().remove(game.dim.id()).unwrap_or_default());
    let positions: Vec<Pos> = if pending.scan_loaded {
        game.world
            .block_entities
            .iter()
            .filter_map(|(&(x, y, z), _)| {
                let pos = [x, y, z];
                matches!(
                    game.native_block(pos).as_str(),
                    "create:clutch" | "create:gearshift"
                )
                .then_some(pos)
            })
            .collect()
    } else {
        pending.positions.into_iter().collect()
    };
    for pos in positions {
        let block = game.native_block(pos);
        if !matches!(block.as_str(), "create:clutch" | "create:gearshift") {
            continue;
        }
        let dimension = game.dim.id().to_owned();
        let powered = game.native_neighbor_signal(&dimension, pos) > 0;
        if !game.native_set_block_property(pos, "powered", if powered { "true" } else { "false" }) {
            continue;
        }
        invalidate();
        let Some(node) = super::create_port::node(game, pos) else {
            continue;
        };
        let network = self::network(game, node, 0);
        let other = matches!(node.kind, Kind::Clutch { powered: true })
            .then(|| self::network(game, node, 1));
        game.native_with_mod_block_entity(pos, |entity| {
            super::create_port::apply_network(entity, &network, other.as_deref(), pos);
        });
        publish(game, &network, pos);
        if let Some(other) = other {
            publish(game, &other, pos);
        }
        game.native_block_entity_sync(pos);
    }
}
