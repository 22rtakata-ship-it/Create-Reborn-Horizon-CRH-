// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::{Game, native_block_entity_draw::BlockEntityDraw};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModContainer},
    items::ItemStack,
};
use create_engine::{
    chute::{self, ENTRY, Source, Step},
    kinetics::Pos,
};
use horizon_core::geometry::Direction;
use horizon_items::item_nbt::{item_from_nbt, item_to_nbt};
use horizon_world::{
    attachments::{AttachmentType, Holder},
    nbt::{Compound, Tag},
};

pub(super) const BLOCK: &str = "create:chute";

const PREVIOUS: AttachmentType<f32> = AttachmentType {
    namespace: "create",
    name: "chute_previous_position",
    default: || 0.0,
    codec: None,
    copy_on_death: false,
    sync: false,
};

pub(super) fn default_nbt() -> Compound {
    Compound::from([
        ("Item".into(), Tag::Compound(Compound::new())),
        ("ItemPosition".into(), Tag::Float(0)),
        ("Pull".into(), Tag::Float(0)),
        ("Push".into(), Tag::Float(0)),
        ("BottomAirFlowDistance".into(), Tag::Float(0)),
    ])
}

pub(super) fn container() -> ModContainer {
    ModContainer {
        slots: 1,
        faces: std::array::from_fn(|_| vec![0]),
        inputs: vec![0],
        outputs: vec![0],
        menu: None,
    }
}

fn dry(game: &Game<'_>, pos: Pos) -> bool {
    game.native_block(pos) == BLOCK
        && game.native_block_property(pos, "facing").as_deref() == Some("down")
        && game.native_block_property(pos, "waterlogged").as_deref() == Some("false")
}

fn empty(entity: &ModBlockEntity) -> bool {
    entity
        .items
        .first()
        .is_none_or(|slot| slot.as_ref().is_none_or(|stack| stack.count == 0))
}

fn set_position(entity: &mut ModBlockEntity, value: f32) {
    entity
        .nbt
        .insert("ItemPosition".into(), Tag::Float(value.to_bits()));
}

fn item_data(entity: &ModBlockEntity) -> Tag {
    entity
        .items
        .first()
        .and_then(Option::as_ref)
        .filter(|stack| stack.count > 0)
        .map_or_else(|| Tag::Compound(Compound::new()), item_to_nbt)
}

fn set_item(entity: &mut ModBlockEntity, stack: Option<ItemStack>, position: f32) {
    if entity.items.is_empty() {
        entity.items.push(stack);
    } else {
        entity.items[0] = stack;
    }
    let item = item_data(entity);
    entity.nbt.insert("Item".into(), item);
    set_position(entity, position);
}

fn remember(game: &mut Game<'_>, pos: Pos, previous: Option<f32>) {
    if let Ok(map) = game.attachment_map_quiet(Holder::BlockEntity(pos)) {
        if let Some(previous) = previous {
            let _ = map.set(&PREVIOUS, previous);
        } else {
            let _ = map.remove(&PREVIOUS);
        }
    }
}

fn previous(game: &mut Game<'_>, pos: Pos, current: f32) -> f32 {
    game.attachment_map_quiet(Holder::BlockEntity(pos))
        .ok()
        .and_then(|map| map.get_existing(&PREVIOUS).ok().flatten())
        .unwrap_or(current)
}

fn entity_at<'g>(game: &'g Game<'_>, [x, y, z]: Pos) -> Option<&'g ModBlockEntity> {
    match game.world.block_entities.get(&(x, y, z))? {
        BlockEntity::Mod(entity) if entity.id == BLOCK => Some(entity),
        _ => None,
    }
}

fn enter(game: &mut Game<'_>, pos: Pos, stack: ItemStack) -> bool {
    let accepted = game.native_with_mod_block_entity(pos, |entity| {
        if entity.id != BLOCK || !empty(entity) {
            return false;
        }
        set_item(entity, Some(stack), ENTRY);
        true
    }) == Some(true);
    if accepted {
        remember(game, pos, None);
        game.native_block_entity_sync(pos);
    }
    accepted
}

pub(super) fn external_change(game: &mut Game<'_>, (x, y, z): (i32, i32, i32)) {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get_mut(&(x, y, z)) else {
        return;
    };
    if entity.id != BLOCK {
        return;
    }
    let item = item_data(entity);
    if entity.nbt.get(&"Item".into()) == Some(&item) {
        return;
    }
    entity.nbt.insert("Item".into(), item);
    set_position(entity, ENTRY);
    remember(game, [x, y, z], None);
    game.native_block_entity_sync([x, y, z]);
}

fn pull(game: &mut Game<'_>, [x, y, z]: Pos) -> Option<ItemStack> {
    let source = [x, y + 1, z];
    if !game.chute_source_available((x, y + 1, z)) {
        return None;
    }
    let dim = game.dim.id();
    let slots = game.native_container_slots(dim, source, Direction::Down)?;
    let slots = slots
        .iter()
        .filter(|slot| slot.can_take)
        .filter_map(|slot| {
            let stack = slot.stack.as_ref()?;
            Some(Source {
                slot: slot.slot,
                stack,
                count: stack.count,
                maximum: super::create_port::stack_maximum(game, stack),
            })
        });
    let plan = chute::pull_plan(slots, ItemStack::same_item_same_components);
    let mut result: Option<ItemStack> = None;
    for (slot, count) in plan {
        let Some(item) = game.native_extract_item(dim, source, Direction::Down, slot, count) else {
            continue;
        };
        if let Some(stack) = &mut result {
            stack.count += item.count;
        } else {
            result = Some(item);
        }
    }
    result
}

#[derive(Clone, Copy)]
enum Route {
    Blocked,
    Chute,
    Container,
    Drop,
}

fn route(game: &mut Game<'_>, below: Pos, stack: &ItemStack) -> Route {
    if !super::create_port_contraption::resident(game, below) {
        return Route::Blocked;
    }
    if game.native_block(below) == BLOCK {
        return if dry(game, below) && entity_at(game, below).is_some_and(empty) {
            Route::Chute
        } else {
            Route::Blocked
        };
    }
    if let Some(moved) = game.chute_insert((below[0], below[1], below[2]), stack, true) {
        return if moved > 0 {
            Route::Container
        } else {
            Route::Blocked
        };
    }
    let state = game.world.block(below[0], below[1], below[2]);
    if game
        .res
        .behaviours()
        .is_face_sturdy_rigid(state, Direction::Up)
    {
        Route::Blocked
    } else {
        Route::Drop
    }
}

pub(super) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
    if !dry(game, pos)
        || entity.id != BLOCK
        || entity.items.len() > 1
        || super::create_port::number(&entity.nbt, "Pull") != 0.0
        || super::create_port::number(&entity.nbt, "Push") != 0.0
    {
        return;
    }
    let Some(stack) = entity
        .items
        .first()
        .and_then(Option::as_ref)
        .filter(|stack| stack.count > 0)
    else {
        let above = [pos[0], pos[1] + 1, pos[2]];
        let pulled = if game.native_block(above) == super::create_port_portable_storage::BLOCK {
            super::create_port_portable_storage::pull_chute(game, above, |_, item| {
                set_item(entity, Some(item.clone()), ENTRY);
                true
            })
        } else if let Some(item) = pull(game, pos) {
            set_item(entity, Some(item), ENTRY);
            true
        } else {
            false
        };
        if pulled {
            remember(game, pos, None);
            game.native_block_entity_sync(pos);
        }
        return;
    };
    let offset = super::create_port::number(&entity.nbt, "ItemPosition");
    if !offset.is_finite() {
        return;
    }
    let below = [pos[0], pos[1] - 1, pos[2]];
    let mut output = Route::Blocked;
    match chute::step(offset, || {
        output = route(game, below, stack);
        !matches!(output, Route::Blocked)
    }) {
        Step::Position(next) => {
            remember(game, pos, Some(offset));
            if next != offset {
                set_position(entity, next);
                if game.native_block_property(pos, "shape").as_deref() == Some("window") {
                    game.native_block_entity_sync(pos);
                }
            }
        }
        Step::Output => {
            let stack = stack.clone();
            match output {
                Route::Chute if enter(game, below, stack.clone()) => set_item(entity, None, ENTRY),
                Route::Container => {
                    let mut rest = stack;
                    let moved = game
                        .chute_insert((below[0], below[1], below[2]), &rest, false)
                        .unwrap_or(0);
                    rest.count -= moved;
                    let previous = previous(game, pos, offset);
                    set_item(entity, (rest.count != 0).then_some(rest), previous);
                }
                Route::Drop => {
                    game.spawn_item(
                        stack,
                        [
                            f64::from(pos[0]) + 0.5,
                            f64::from(pos[1]) - 0.25,
                            f64::from(pos[2]) + 0.5,
                        ],
                        [0.0, -0.25, 0.0],
                        10,
                    );
                    set_item(entity, None, ENTRY);
                }
                _ => return,
            }
            remember(game, pos, None);
            game.native_block_entity_sync(pos);
        }
    }
}

pub(super) fn landed(game: &mut Game<'_>, pos: Pos, stack: &mut ItemStack) -> bool {
    let above = [pos[0], pos[1] + 1, pos[2]];
    if !game.native_on("create")
        || !dry(game, pos)
        || game.native_block(above) == BLOCK
        || stack.count == 0
    {
        return false;
    }
    let count = stack
        .count
        .min(super::create_port::stack_maximum(game, stack));
    if count == 0
        || !enter(
            game,
            pos,
            ItemStack {
                count,
                ..stack.clone()
            },
        )
    {
        return false;
    }
    stack.count -= count;
    true
}

pub(super) fn right_click(game: &mut Game<'_>, pos: Pos) -> bool {
    if !game.native_on("create") || !dry(game, pos) {
        return false;
    }
    if let Some(item) = game.held() {
        if !super::create_port_chassis::wrench(game, item)
            || game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
            || !game.may_build()
        {
            return false;
        }
        let shape = match game.native_block_property(pos, "shape").as_deref() {
            Some("normal") => "window",
            Some("window") => "normal",
            _ => return false,
        };
        if game.native_set_block_property(pos, "shape", shape) {
            super::create_port_sounds::wrench(game, pos, false);
            return true;
        }
        return false;
    }
    let item = game
        .native_with_mod_block_entity(pos, |entity| {
            let item = entity.items.first_mut()?.take()?;
            set_item(entity, None, ENTRY);
            Some(item)
        })
        .flatten();
    if let Some(item) = item {
        game.give_or_drop(item);
        remember(game, pos, None);
        game.native_block_entity_sync(pos);
        true
    } else {
        false
    }
}

pub(super) fn draw(game: &Game<'_>, pos: Pos, entity: &ModBlockEntity, draw: &mut BlockEntityDraw) {
    if !dry(game, pos) || game.native_block_property(pos, "shape").as_deref() != Some("window") {
        return;
    }
    let Some(stack) = entity.nbt.get(&"Item".into()).and_then(item_from_nbt) else {
        return;
    };
    let offset = super::create_port::number(&entity.nbt, "ItemPosition");
    if !offset.is_finite() {
        return;
    }
    draw.push();
    draw.translate([0.5, offset, 0.5]);
    draw.scale([0.5; 3]);
    draw.rotate_about([1.0, 0.0, 0.0], offset * 180.0, [0.0; 3]);
    draw.rotate_about([0.0, 1.0, 0.0], offset * 180.0, [0.0; 3]);
    game.native_draw_item(
        draw,
        &stack,
        horizon_assets::item_model::DisplayContext::Fixed,
        None,
        0,
    );
    draw.pop();
}
