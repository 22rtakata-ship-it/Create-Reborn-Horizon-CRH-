// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::{
    Game, create_port_contraption as contraption, native_block_entity_draw::BlockEntityDraw,
};
use create_engine::{contraption::sticker::State, kinetics::Pos, placement::Direction};
use horizon_world::{
    chunk::BlockState,
    level::flags,
    nbt::{Compound, Tag},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

pub(super) const BLOCK: &str = "create:sticker";

thread_local! {
    static CHANGED: RefCell<BTreeMap<&'static str, BTreeSet<Pos>>> =
        const { RefCell::new(BTreeMap::new()) };
}

pub(super) fn default_nbt() -> Compound {
    Compound::from([("HasAttachedComputer".into(), Tag::Byte(0))])
}

fn facing(block: &BlockState) -> Option<Direction> {
    let name = block.properties.get("facing")?;
    Direction::ALL.into_iter().find(|face| face.name() == name)
}

pub(super) fn decode(block: &BlockState) -> Option<State> {
    if block.name != BLOCK {
        return None;
    }
    let flag = |name| match block.properties.get(name)?.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    };
    Some(State {
        facing: facing(block)?,
        powered: flag("powered")?,
        extended: flag("extended")?,
    })
}

pub(super) fn target_supports(block: &BlockState, side: Direction) -> bool {
    if let Some(sticker) = decode(block) {
        return sticker.supported_face(side);
    }
    if let Some(sail) = super::create_port_sail::from_state(block) {
        return sail.attaches(side);
    }
    // Directional actors expose a non-supportive working face in
    // BlockMovementChecksImpl.isNotSupportiveFallback.
    !matches!(
        block.name.as_str(),
        "create:mechanical_bearing"
            | "create:mechanical_drill"
            | "create:mechanical_saw"
            | "create:mechanical_harvester"
            | "create:mechanical_plough"
            | "create:portable_storage_interface"
    ) || facing(block) != Some(side)
}

pub(super) fn neighbor(pos: Pos, face: Direction) -> Option<Pos> {
    let delta = face.vector();
    let [x, y, z] = std::array::from_fn(|axis| pos[axis].checked_add(delta[axis]));
    Some([x?, y?, z?])
}

pub(super) fn can_capture(game: &Game<'_>, [x, y, z]: Pos, block: &BlockState) -> bool {
    use crate::containers::BlockEntity;
    decode(block).is_some()
        && matches!(game.world.block_entities.get(&(x, y, z)), Some(BlockEntity::Mod(entity))
            if entity.id == BLOCK
                && entity.items.iter().all(Option::is_none)
                && (entity.nbt.is_empty()
                    || entity.nbt.len() == 1
                        && entity.nbt.get(&"HasAttachedComputer".into()) == Some(&Tag::Byte(0))))
}

pub(super) fn changed(dimension: &'static str, pos: Pos) {
    CHANGED.with(|pending| {
        pending
            .borrow_mut()
            .entry(dimension)
            .or_default()
            .insert(pos)
    });
}

fn update(game: &mut Game<'_>, pos: Pos, placement: bool) {
    let dimension = game.dim.id();
    let id = contraption::state(game, pos);
    let registry = game.pack.assets.registry;
    let Some(block) = registry.state(id) else {
        return;
    };
    let Some(old) = decode(block) else { return };
    let powered = game.native_neighbor_signal(dimension, pos) != 0;
    let next = if placement {
        State {
            powered,
            extended: false,
            ..old
        }
    } else {
        old.with_signal(powered)
    };
    if old == next {
        return;
    }
    let mut updated = block.clone();
    updated
        .properties
        .insert("powered".into(), next.powered.to_string());
    updated
        .properties
        .insert("extended".into(), next.extended.to_string());
    if let Some(id) = registry.id(&updated) {
        game.native_set_block(dimension, pos, id, flags::UPDATE_CLIENTS);
    }
}

pub(super) fn refresh(game: &mut Game<'_>) {
    let Some(pending) = CHANGED.with(|changed| changed.borrow_mut().remove(game.dim.id())) else {
        return;
    };
    for pos in pending {
        update(game, pos, false);
    }
}

pub(super) fn placed(game: &mut Game<'_>, pos: Pos) {
    update(game, pos, true);
}

pub(super) fn draw(game: &Game<'_>, draw: &mut BlockEntityDraw, block: &BlockState) {
    let Some(state) = decode(block) else { return };
    draw.push();
    match state.facing {
        Direction::Up => {}
        Direction::Down => draw.rotate_about([1.0, 0.0, 0.0], 180.0, [0.5; 3]),
        side => {
            super::create_port::face_south(draw, side);
            draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.5; 3]);
        }
    }
    if state.extended {
        draw.translate([0.0, 0.25, 0.0]);
    }
    game.native_draw_model(draw, "create:block/sticker/head", 0xFFFFFF);
    draw.pop();
}
