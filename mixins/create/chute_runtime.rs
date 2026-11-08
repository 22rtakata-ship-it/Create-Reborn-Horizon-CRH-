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
const CASING: &str = "create:industrial_iron_block";

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
    PortableStorage,
    Drop,
}

fn route(game: &mut Game<'_>, pos: Pos, below: Pos, stack: &ItemStack) -> Route {
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
    if game.native_block(below) == super::create_port_portable_storage::BLOCK {
        let moved = super::create_port_portable_storage::push_chute(game, below, pos, stack, true);
        return if moved != 0 {
            Route::PortableStorage
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
        output = route(game, pos, below, stack);
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
                Route::Container | Route::PortableStorage => {
                    let mut rest = stack;
                    let moved = if matches!(output, Route::PortableStorage) {
                        super::create_port_portable_storage::push_chute(
                            game, below, pos, &rest, false,
                        )
                    } else {
                        game.chute_insert((below[0], below[1], below[2]), &rest, false)
                            .unwrap_or(0)
                    };
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

fn shape_change(game: &Game<'_>, pos: Pos, item: &ItemStack) -> Option<(String, &'static str)> {
    if game.spectator() || game.native_sneaking() {
        return None;
    }
    let shape = game.native_block_property(pos, "shape")?;
    let next = if item.item.as_ref() == CASING {
        match shape.as_str() {
            "normal" | "window" => "encased",
            _ => return None,
        }
    } else {
        if !super::create_port_chassis::wrench(game, item) || !game.may_build() {
            return None;
        }
        match shape.as_str() {
            "normal" => "window",
            "window" | "encased" => "normal",
            _ => return None,
        }
    };
    Some((shape, next))
}

pub(super) fn predicts_use(game: &Game<'_>, pos: Pos) -> bool {
    game.native_on("create")
        && dry(game, pos)
        && game
            .held()
            .is_some_and(|item| shape_change(game, pos, item).is_some())
}

pub(super) fn right_click(game: &mut Game<'_>, pos: Pos) -> bool {
    if !game.native_on("create") || !dry(game, pos) {
        return false;
    }
    if let Some(item) = game.held() {
        let Some((shape, next)) = shape_change(game, pos, item) else {
            return false;
        };
        if !game.native_set_block_property(pos, "shape", next) {
            return false;
        }
        if next == "encased" {
            let center = pos.map(|v| f64::from(v) + 0.5);
            game.native_play_sound_in_place(
                "minecraft:block.netherite_block.hit",
                center,
                4,
                0.5,
                1.05,
            );
        } else if shape == "encased"
            && let Some(state) = game.pack.assets.registry.default_state(CASING)
        {
            game.level_event_at(2001, pos, state.0 as i32);
        }
        return true;
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

pub(in crate::game) mod funnel {
    use crate::game::{Game, GameLevel, create_port_belt::world_direction};
    use crate::{containers::ModBlockEntity, items::ItemStack};
    use create_engine::{kinetics::Pos, placement::Direction};
    use horizon_core::{geometry::Vec3, position::BlockPos, random::RandomSource};
    use horizon_world::{
        fluid::FluidKind,
        level::{PlaceContext, PlaceOutcome, PlacementContext, flags},
        nbt::{Compound, Tag},
        registry::BlockStateId,
    };

    pub(in crate::game) const BLOCK: &str = "create:andesite_funnel";
    pub(in crate::game) const ENTITY: &str = "create:funnel";
    // CLogistics.defaultExtractionTimer in Create 6.0.10.
    const EXTRACTION_COOLDOWN: i32 = 8;

    pub(in crate::game) fn default_nbt() -> Compound {
        Compound::from([
            ("TransferCooldown".into(), Tag::Int(0)),
            ("Filter".into(), Tag::Compound(Compound::new())),
            ("FilterAmount".into(), Tag::Int(64)),
            ("UpTo".into(), Tag::Byte(1)),
        ])
    }

    fn property(game: &Game<'_>, pos: Pos, name: &str) -> bool {
        game.native_block_property(pos, name).as_deref() == Some("true")
    }

    fn direction(game: &Game<'_>, pos: Pos) -> Option<Direction> {
        let name = game.native_block_property(pos, "facing")?;
        Direction::ALL.into_iter().find(|d| d.name() == name)
    }

    pub(in crate::game) fn placement(
        game: &GameLevel<'_>,
        state: BlockStateId,
        context: &PlacementContext,
        face: Direction,
    ) -> Option<BlockStateId> {
        let facing = if context.replace_clicked {
            crate::game::create_port::nearest(context.yaw, context.pitch).opposite()
        } else {
            face
        };
        let at = context.pos;
        let existing = game.world.block(at.x, at.y, at.z);
        // This release implements the dry vertical forms. A horizontal
        // placement may also resolve to a separate Belt Funnel in Java.
        if !matches!(facing, Direction::Up | Direction::Down)
            || game.behaviours.fluid_kind(existing) != FluidKind::Empty
        {
            return None;
        }
        let [dx, dy, dz] = facing.vector();
        let behind = game.world.block(at.x - dx, at.y - dy, at.z - dz);
        if game.native_state_block(behind) == Some(BLOCK) {
            return None;
        }
        let state = game.native_state_with_property(state, "facing", facing.name())?;
        game.native_state_with_property(
            state,
            "extracting",
            if context.sneaking { "false" } else { "true" },
        )
    }

    pub(in crate::game) fn refresh_power(game: &mut Game<'_>, pos: Pos) -> bool {
        let powered = game.native_neighbor_signal(game.dim.id(), pos) != 0;
        if powered != property(game, pos, "powered") {
            game.native_set_block_property_with(
                pos,
                "powered",
                if powered { "true" } else { "false" },
                flags::UPDATE_CLIENTS,
            );
        }
        powered
    }

    pub(in crate::game) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
        if entity.id != ENTITY || property(game, pos, "waterlogged") {
            return;
        }
        let cooldown = crate::game::create_port::integer(&entity.nbt, "TransferCooldown");
        let next = if refresh_power(game, pos) {
            0
        } else if cooldown > 0 {
            cooldown - 1
        } else if property(game, pos, "extracting") && extract(game, pos) {
            EXTRACTION_COOLDOWN
        } else {
            0
        };
        if next != cooldown {
            entity.nbt.insert("TransferCooldown".into(), Tag::Int(next));
        }
    }

    fn extract(game: &mut Game<'_>, pos: Pos) -> bool {
        let Some(facing @ (Direction::Up | Direction::Down)) = direction(game, pos) else {
            return false;
        };
        let dim = game.dim.id();
        let source = [pos[0], pos[1] - facing.vector()[1], pos[2]];
        let side = world_direction(facing);
        let Some(slot) = game
            .native_container_slots(dim, source, side)
            .and_then(|slots| slots.into_iter().find(|slot| slot.can_take))
        else {
            return false;
        };
        let up = facing == Direction::Up;
        let origin = pos.map(f64::from);
        let mut scan_min = origin;
        if !up {
            scan_min[1] -= 1.0;
        }
        if !game
            .native_item_entities(dim, scan_min, origin.map(|v| v + 1.0))
            .is_empty()
        {
            return false;
        }
        let Some(stack) = game.native_extract_item(dim, source, side, slot.slot, 1) else {
            return false;
        };
        let mut spawn = origin.map(|v| v + 0.5);
        spawn[1] = if up {
            spawn[1] + f64::from(0.15_f32)
        } else {
            origin[1]
        };
        game.spawn_item(stack, spawn, [0.0, if up { 0.25 } else { 0.0 }, 0.0], 10);
        let at = origin.map(|v| v + 0.5);
        game.native_play_sound_in_place(
            "minecraft:entity.item_frame.rotate_item",
            at,
            4,
            0.125,
            1.5,
        );
        game.native_play_sound_in_place("minecraft:block.wool.break", at, 4, 0.0425, 0.75);
        true
    }

    fn collecting(game: &Game<'_>, pos: Pos, face: Pos, stack: &ItemStack) -> bool {
        face == [0, 1, 0]
            && direction(game, pos) == Some(Direction::Up)
            && !property(game, pos, "powered")
            && !property(game, pos, "extracting")
            && stack.item.as_ref() != "create:mechanical_arm"
    }

    enum Use {
        Attach,
        Wrench,
        Pickup,
        Collect,
    }

    fn action(game: &Game<'_>, pos: Pos, face: Pos, stack: &ItemStack) -> Option<Use> {
        if !game.native_on("create") || game.spectator() {
            return None;
        }
        let sneaking = game.native_sneaking();
        let block = game.native_block(pos);
        if stack.item.as_ref() == BLOCK {
            return (!sneaking
                && matches!(face, [0, -1, 0] | [0, 1, 0])
                && matches!(
                    block.as_str(),
                    "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel"
                ))
            .then_some(Use::Attach);
        }
        if block != BLOCK || property(game, pos, "waterlogged") {
            return None;
        }
        if game.may_build() && crate::game::create_port_chassis::wrench(game, stack) {
            return Some(if sneaking { Use::Pickup } else { Use::Wrench });
        }
        (!sneaking && collecting(game, pos, face, stack)).then_some(Use::Collect)
    }

    pub(in crate::game) fn predicts_use(game: &Game<'_>, pos: Pos, face: Pos) -> bool {
        game.held()
            .is_some_and(|stack| action(game, pos, face, stack).is_some())
    }

    pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: Pos, face: Pos) -> bool {
        let Some(stack) = game.held().cloned() else {
            return false;
        };
        match action(game, pos, face, &stack) {
            None => return false,
            Some(Use::Attach) => attach(game, pos, face, &stack),
            Some(Use::Pickup) => pickup(game, pos, &stack),
            Some(Use::Wrench) => {
                let extracting = property(game, pos, "extracting");
                game.native_set_block_property(
                    pos,
                    "extracting",
                    if extracting { "false" } else { "true" },
                );
            }
            Some(Use::Collect) => {
                let inserted = game
                    .chute_insert((pos[0], pos[1] - 1, pos[2]), &stack, false)
                    .unwrap_or(0);
                if inserted > 0 {
                    let count = stack.count - inserted;
                    *game.held_mut() = (count > 0).then_some(ItemStack { count, ..stack });
                }
            }
        }
        true
    }

    fn pickup(game: &mut Game<'_>, pos: Pos, wrench: &ItemStack) {
        let [x, y, z] = pos;
        let state = game.world.block(x, y, z);
        let mut canceled = false;
        game.native_destroy_block(pos, state, &mut canceled);
        if canceled || game.world.block(x, y, z) != state {
            return;
        }
        let drops = if game.creative {
            Vec::new()
        } else {
            let entity = game.loot_block_entity(pos, BLOCK);
            game.block_loot_at(pos, state, Some(wrench), entity)
        };
        if game.with_level(|b, level, ticks| {
            b.destroy_block(level, ticks, BlockPos::new(x, y, z), false)
        }) {
            let _ = game.process_level_events();
            for stack in drops {
                game.give_or_drop(stack);
            }
            let pitch = 0.5 + game.world.random.next_float() * 0.5;
            crate::game::create_port_sounds::wrench_remove(game, pos, pitch);
        }
    }

    fn attach(game: &mut Game<'_>, pos: Pos, face: Pos, stack: &ItemStack) {
        // FunnelItem bypasses the clicked container's own menu before running
        // BlockItem.place. Keep its ordinary validation and placed-item effects.
        if !game.may_build() && !game.can_place_on_in_adventure(pos) {
            game.use_failed = true;
            return;
        }
        let ctx = PlaceContext {
            clicked_pos: BlockPos::new(pos[0], pos[1], pos[2]),
            face: world_direction(if face[1] > 0 {
                Direction::Up
            } else {
                Direction::Down
            }),
            hit: Vec3::new(
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + if face[1] > 0 { 1.0 } else { 0.0 },
                f64::from(pos[2]) + 0.5,
            ),
            inside: false,
            yaw: game.yaw,
            pitch: game.pitch,
            sneaking: false,
            item: stack.item.to_string(),
            can_use_game_master_blocks: game.can_use_game_master_blocks(),
        };
        if game.block_place_canceled(stack, &ctx) {
            game.use_failed = true;
            game.guest_inventory_resend = true;
            return;
        }
        let outcome = game.with_level(|b, level, ticks| b.place_block(level, ticks, &ctx));
        if let PlaceOutcome::Placed { pos, state } = outcome {
            let target = [pos.x, pos.y, pos.z];
            let state = game
                .apply_item_block_state(target, state, stack)
                .unwrap_or(state);
            if !game.world.block_entities.stateful_entity(state.0)
                && let Some(entity) = game.new_block_entity_for_state(state)
            {
                game.world
                    .block_entities
                    .insert((pos.x, pos.y, pos.z), entity);
            }
            game.apply_placed_item(target, stack, ctx.can_use_game_master_blocks);
            game.award_stat("used", &stack.item, 1);
            game.trigger_block("minecraft:placed_block", target, stack);
            game.native_place_block(stack, target, state);
            game.consume_held();
        } else {
            game.use_failed = true;
        }
        if let Err(error) = game.process_level_events() {
            log::warn!("Create Funnel placement: {error}");
        }
    }
}
