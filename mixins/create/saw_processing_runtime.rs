// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::{
    Game, create_port as create, create_port_belt as belt,
    native_block_entity_draw::BlockEntityDraw,
};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModContainer},
    items::ItemStack,
};
use create_engine::{
    kinetics::Pos,
    milling::roll_count,
    placement::Direction,
    saw_processing::{self, Action, CuttingCycle},
    transport::Transported,
};
use horizon_core::random::RandomSource;
use horizon_items::item_nbt::item_from_nbt;
use horizon_world::nbt::{Compound, Tag};

const SLOTS: usize = 32;
const INVENTORY: &str = "Inventory";

/// MechanicalCrafterBlockEntity and RecipeGridHandler. The kinetic and
/// transport adapters in this source share the host's ordinary ticker.
pub(super) mod crafter {
    use super::{
        BlockEntity, BlockEntityDraw, Compound, Game, ItemStack, ModBlockEntity, Pos, Tag,
        Transported, belt, create,
    };
    use create_engine::{
        crafter::{self, Cycle, GroupedItems, Layout, Phase, Pointing, Tick},
        kinetics::Axis,
        placement::Direction,
    };
    use horizon_core::random::RandomSource;
    use horizon_items::item_nbt::{item_from_nbt, item_to_nbt};

    pub(in crate::game) const BLOCK: &str = "create:mechanical_crafter";
    const SLOT_COVER: &str = "create:crafter_slot_cover";
    const INVENTORY: &str = "Inventory";
    const GROUPED: &str = "GroupedItems";
    const PHASE: &str = "Phase";
    const COUNTDOWN: &str = "CountDown";

    fn layout(game: &Game<'_>, pos: Pos) -> Option<Layout> {
        if game.native_block(pos) != BLOCK {
            return None;
        }
        let name = game.native_block_property(pos, "facing")?;
        let facing = Direction::ALL
            .into_iter()
            .find(|side| side.axis() != Axis::Y && side.name() == name)?;
        let pointing = Pointing::from_name(&game.native_block_property(pos, "pointing")?)?;
        Some(Layout { facing, pointing })
    }

    fn entity<'a>(game: &'a Game<'_>, [x, y, z]: Pos) -> Option<&'a ModBlockEntity> {
        match game.world.block_entities.get(&(x, y, z)) {
            Some(BlockEntity::Mod(entity))
                if entity.id == BLOCK && create::inventory_supported(entity) =>
            {
                Some(entity)
            }
            _ => None,
        }
    }

    fn cycle(nbt: &Compound) -> Cycle {
        let phase = match nbt.get(&PHASE.into()) {
            Some(Tag::String(name)) => name
                .to_string()
                .ok()
                .and_then(|value| Phase::from_name(&value))
                .unwrap_or_default(),
            _ => Phase::default(),
        };
        Cycle {
            phase,
            countdown: create::integer(nbt, COUNTDOWN),
        }
    }

    fn set_cycle(nbt: &mut Compound, value: Cycle) {
        nbt.insert(PHASE.into(), Tag::String(value.phase.name().into()));
        nbt.insert(COUNTDOWN.into(), Tag::Int(value.countdown));
    }

    fn grid(nbt: &Compound) -> GroupedItems<ItemStack> {
        let Some(Tag::List(rows)) = nbt.get(&GROUPED.into()).and_then(|tag| tag.get("Grid")) else {
            return GroupedItems::default();
        };
        GroupedItems {
            grid: rows
                .iter()
                .filter_map(|row| {
                    let (Tag::Int(x), Tag::Int(y)) = (row.get("x")?, row.get("y")?) else {
                        return None;
                    };
                    Some(((*x, *y), row.get("item").and_then(item_from_nbt)))
                })
                .collect(),
        }
    }

    fn set_grid(nbt: &mut Compound, grouped: &GroupedItems<ItemStack>) {
        let cells = grouped
            .grid
            .iter()
            .map(|(&(x, y), stack)| {
                Tag::Compound(Compound::from([
                    ("x".into(), Tag::Int(x)),
                    ("y".into(), Tag::Int(y)),
                    (
                        "item".into(),
                        stack
                            .as_ref()
                            .map_or_else(|| Tag::Compound(Compound::new()), item_to_nbt),
                    ),
                ]))
            })
            .collect();
        nbt.insert(
            GROUPED.into(),
            Tag::Compound(Compound::from([("Grid".into(), Tag::List(cells))])),
        );
    }

    fn inventory(item: Option<&ItemStack>) -> Tag {
        let mut items = Vec::new();
        if let Some(Tag::Compound(mut row)) = item.map(item_to_nbt) {
            row.insert("Slot".into(), Tag::Int(0));
            items.push(Tag::Compound(row));
        }
        Tag::Compound(Compound::from([
            ("Size".into(), Tag::Int(1)),
            ("Items".into(), Tag::List(items)),
        ]))
    }

    fn save_inventory(entity: &mut ModBlockEntity) {
        let saved = inventory(entity.items.first().and_then(Option::as_ref));
        entity.nbt.insert(INVENTORY.into(), saved);
    }

    pub(in crate::game) fn default_nbt() -> Compound {
        let mut fields = Compound::from([
            (INVENTORY.into(), inventory(None)),
            ("Cover".into(), Tag::Byte(0)),
            (
                "ConnectedInput".into(),
                Tag::Compound(Compound::from([
                    ("Controller".into(), Tag::Byte(1)),
                    (
                        "Data".into(),
                        Tag::List(vec![Tag::Compound(
                            ["X", "Y", "Z"]
                                .into_iter()
                                .map(|axis| (axis.into(), Tag::Int(0)))
                                .collect(),
                        )]),
                    ),
                ])),
            ),
        ]);
        set_cycle(&mut fields, Cycle::default());
        set_grid(&mut fields, &GroupedItems::default());
        fields
    }

    fn chain(game: &Game<'_>, pos: Pos) -> Option<Vec<Pos>> {
        let chain = crafter::chain(pos, |at| layout(game, at)).ok()?;
        let column = |[x, _, z]: Pos| (x.div_euclid(16), z.div_euclid(16));
        chain
            .iter()
            .all(|&at| column(at) == column(pos))
            .then_some(chain)
    }

    #[derive(Clone, Copy)]
    enum Outlet {
        Open,
        Depot,
        Container,
    }

    fn outlet(game: &Game<'_>, pos: Pos, layout: Layout) -> Option<Outlet> {
        let target @ [x, y, z] = layout.target_position(pos);
        let block = game.native_block(target);
        if block == "create:depot" {
            return Some(Outlet::Depot);
        }
        if layout.target_direction() == Direction::Down
            && matches!(
                block.as_str(),
                "minecraft:chest"
                    | "minecraft:trapped_chest"
                    | "minecraft:barrel"
                    | "minecraft:hopper"
            )
            && game.chute_source_available((x, y, z))
        {
            return Some(Outlet::Container);
        }
        game.res
            .behaviours()
            .is_air(game.world.block(x, y, z))
            .then_some(Outlet::Open)
    }

    fn start_plan(
        game: &Game<'_>,
        pos: Pos,
        ticking: Option<&ModBlockEntity>,
        powered: bool,
    ) -> Option<Vec<(Pos, bool)>> {
        let chain = chain(game, pos)?;
        let mut plan = Vec::with_capacity(chain.len());
        let mut nonempty = false;
        for at in chain {
            let member = if at == pos {
                ticking.or_else(|| entity(game, at))
            } else {
                entity(game, at)
            }?;
            let has_item = member.items.first().is_some_and(Option::is_some);
            nonempty |= has_item;
            if cycle(&member.nbt).phase != Phase::Idle
                || (!powered && !has_item && create::integer(&member.nbt, "Cover") == 0)
                || create::running_speed(game, &member.nbt) == 0.0
            {
                return None;
            }
            let state = layout(game, at)?;
            if crafter::target(at, state, &mut |p| layout(game, p)).is_none()
                && outlet(game, at, state).is_none()
            {
                return None;
            }
            let waits = !crafter::preceding(at, state, &mut |p| layout(game, p)).is_empty();
            plan.push((at, waits));
        }
        nonempty.then_some(plan)
    }

    fn begin(entity: &mut ModBlockEntity, waits: bool) {
        let item = entity.items.first_mut().and_then(Option::take);
        save_inventory(entity);
        set_grid(&mut entity.nbt, &GroupedItems::single(item));
        let before = cycle(&entity.nbt);
        set_cycle(&mut entity.nbt, before.begin(waits));
    }

    fn start(
        game: &mut Game<'_>,
        pos: Pos,
        mut ticking: Option<&mut ModBlockEntity>,
        powered: bool,
    ) -> bool {
        let Some(plan) = start_plan(game, pos, ticking.as_deref(), powered) else {
            return false;
        };
        for (at, waits) in plan {
            if at == pos
                && let Some(entity) = ticking.as_deref_mut()
            {
                begin(entity, waits);
            } else {
                game.native_with_mod_block_entity(at, |entity| begin(entity, waits));
                game.native_block_entity_sync(at);
            }
        }
        true
    }

    fn drop_position(pos: Pos, facing: Direction) -> [f64; 3] {
        let normal = facing.vector();
        std::array::from_fn(|axis| f64::from(pos[axis]) + 0.5 + 0.75 * f64::from(normal[axis]))
    }

    fn eject(entity: &mut ModBlockEntity) -> Vec<ItemStack> {
        let mut stacks = grid(&entity.nbt)
            .grid
            .into_values()
            .flatten()
            .collect::<Vec<_>>();
        stacks.extend(entity.items.iter_mut().filter_map(Option::take));
        let countdown = cycle(&entity.nbt).countdown;
        set_cycle(
            &mut entity.nbt,
            Cycle {
                phase: Phase::Idle,
                countdown,
            },
        );
        set_grid(&mut entity.nbt, &GroupedItems::default());
        save_inventory(entity);
        stacks
    }

    fn eject_chain(game: &mut Game<'_>, pos: Pos, mut ticking: Option<&mut ModBlockEntity>) {
        for at in chain(game, pos).unwrap_or_else(|| vec![pos]) {
            let Some(state) = layout(game, at) else {
                continue;
            };
            let stacks = if at == pos
                && let Some(entity) = ticking.as_deref_mut()
            {
                eject(entity)
            } else {
                let Some(stacks) = game.native_with_mod_block_entity(at, eject) else {
                    continue;
                };
                game.native_block_entity_sync(at);
                stacks
            };
            let spawn = drop_position(at, state.facing);
            for stack in stacks {
                let speed = &mut game.world.random;
                let velocity = [
                    speed.next_double() * 0.2 - 0.1,
                    0.2,
                    speed.next_double() * 0.2 - 0.1,
                ];
                game.spawn_item(stack, spawn, velocity, 10);
            }
        }
    }

    pub(in crate::game) fn removed(game: &mut Game<'_>, pos: Pos) {
        let Some(member) = entity(game, pos) else {
            return;
        };
        if create::integer(&member.nbt, "Cover") != 0 {
            game.spawn_item(
                ItemStack::new(SLOT_COVER, 1),
                pos.map(|v| f64::from(v) + 0.5),
                [0.0, 0.2, 0.0],
                10,
            );
        }
        eject_chain(game, pos, None);
    }

    fn return_stack(game: &mut Game<'_>, item: ItemStack) {
        let uuid = game.native_acting_id();
        if let Some(rest) = game.native_give_stack(uuid, item)
            && let Some(player) = game.native_player()
        {
            game.spawn_item(
                rest,
                [player.x, player.y + 1.0, player.z],
                [0.0, 0.2, 0.0],
                10,
            );
        }
    }

    fn set_cover(game: &mut Game<'_>, pos: Pos, value: bool) {
        game.native_with_mod_block_entity(pos, |entity| {
            entity
                .nbt
                .insert("Cover".into(), Tag::Byte(i8::from(value)));
        });
        game.native_block_entity_sync(pos);
    }

    pub(in crate::game) fn predicts_use(game: &Game<'_>, pos: Pos, face: Pos) -> bool {
        game.native_on("create")
            && layout(game, pos).is_some_and(|state| state.facing.vector() == face)
    }

    pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: Pos, face: Pos) -> bool {
        if !predicts_use(game, pos, face) {
            return false;
        }
        let Some(member) = entity(game, pos) else {
            return false;
        };
        let Some(state) = layout(game, pos) else {
            return false;
        };
        let phase = cycle(&member.nbt).phase;
        if game
            .held()
            .is_some_and(|stack| stack.item.as_ref() == "create:wrench")
        {
            if game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft) {
                return false;
            }
            // A live group keeps its topology until its cargo is recovered.
            if phase == Phase::Idle {
                game.native_set_block_property(pos, "pointing", state.pointing.next().name());
            }
            return true;
        }
        if phase != Phase::Idle {
            eject_chain(game, pos, None);
            return true;
        }
        let occupied = member.items.first().is_some_and(Option::is_some);
        let covered = create::integer(&member.nbt, "Cover") != 0;
        let held = game.held();
        if held.is_some_and(|stack| stack.item.as_ref() == SLOT_COVER) {
            if occupied || covered {
                return false;
            }
            set_cover(game, pos, true);
            if !game.creative {
                game.consume_held();
            }
            return true;
        }
        if covered {
            if held.is_none() {
                set_cover(game, pos, false);
                if !game.creative {
                    return_stack(game, ItemStack::new(SLOT_COVER, 1));
                }
            }
            return true;
        }
        match (occupied, held.cloned()) {
            (false, Some(mut stack)) => {
                stack.count = 1;
                if game
                    .native_with_mod_block_entity(pos, |entity| {
                        entity.items.resize(1, None);
                        entity.items[0] = Some(stack);
                        save_inventory(entity);
                    })
                    .is_some()
                {
                    let held = game.held_mut();
                    if let Some(stack) = held {
                        stack.count -= 1;
                        if stack.count == 0 {
                            *held = None;
                        }
                    }
                    game.native_block_entity_sync(pos);
                    crate::game::create_port_sounds::crafter_feed(game, pos);
                    start(game, pos, None, false);
                }
            }
            (true, None) => {
                if let Some(Some(stack)) = game.native_with_mod_block_entity(pos, |entity| {
                    let item = entity.items.first_mut().and_then(Option::take);
                    save_inventory(entity);
                    item
                }) {
                    return_stack(game, stack);
                    game.native_block_entity_sync(pos);
                }
            }
            (false, None) => return false,
            (true, Some(_)) => {}
        }
        true
    }

    fn recipe(game: &Game<'_>, grouped: &GroupedItems<ItemStack>) -> Option<ItemStack> {
        let grid = grouped.recipe_grid()?;
        for candidate in game.native_recipes_of_type("create:mechanical_crafting") {
            let data = &candidate.json;
            // Only the first recipe is wired until crafting remainders and
            // other recipe types have their native behavior.
            if data["result"]["id"] != "create:crushing_wheel" {
                continue;
            }
            let Some(rows) = data["pattern"]
                .as_array()
                .filter(|rows| rows.len() == grid.height)
            else {
                continue;
            };
            let fits = rows.iter().enumerate().all(|(y, row)| {
                row.as_str()
                    .filter(|row| row.len() == grid.width)
                    .is_some_and(|row| {
                        row.chars().enumerate().all(|(x, symbol)| {
                            let item = grid.get(x, y);
                            if symbol == ' ' {
                                return item.is_none();
                            }
                            let Some(item) = item else { return false };
                            create::ingredient(&data["key"][symbol.to_string()])
                                .and_then(|value| game.native_recipe_ingredient(&value))
                                .is_some_and(|ingredient| ingredient.test(item))
                        })
                    })
            });
            if fits {
                return game.native_recipe_result(&data["result"]);
            }
        }
        None
    }

    fn lazy_turn(entity: &mut ModBlockEntity) -> bool {
        const RATE: i32 = 20;
        let Some(counter) = entity.data.first_mut() else {
            // SmartBlockEntity.initialize invokes lazyTick before tick's
            // initial post-decrement. This counter is not saved.
            entity.data.extend([RATE - 1, 1]);
            return true;
        };
        if *counter == 0 {
            *counter = RATE;
            true
        } else {
            *counter -= 1;
            false
        }
    }

    fn powered_edge(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) -> bool {
        let dimension = game.dim.id().to_owned();
        let powered = game.native_neighbor_signal(&dimension, pos) > 0;
        let previous = std::mem::replace(&mut entity.data[1], i32::from(powered));
        powered && previous == 0
    }

    fn try_output(
        game: &mut Game<'_>,
        pos: Pos,
        entity: &mut ModBlockEntity,
        layout: Layout,
        outlet: Option<Outlet>,
    ) -> bool {
        let Some(outlet @ (Outlet::Depot | Outlet::Container)) = outlet else {
            eject_chain(game, pos, Some(entity));
            return true;
        };
        let destination @ [x, y, z] = layout.target_position(pos);
        let mut grouped = grid(&entity.nbt);
        let mut changed = false;
        grouped.grid.retain(|_, stack| {
            let Some(stack) = stack else { return false };
            let moved = match outlet {
                Outlet::Depot => {
                    let item = Transported::new(
                        belt::core_stack(game, stack.clone()),
                        belt::angle(game, stack),
                    );
                    belt::insert_depot_from_side(
                        game,
                        destination,
                        &item,
                        layout.target_direction(),
                    )
                }
                Outlet::Container => game.chute_insert((x, y, z), stack, false).unwrap_or(0),
                Outlet::Open => unreachable!("only accepting outlets reach insertion"),
            };
            changed |= moved != 0;
            stack.count -= moved;
            stack.count != 0
        });
        let phase = cycle(&entity.nbt).phase;
        if !changed && phase == Phase::Inserting {
            return false;
        }
        set_grid(&mut entity.nbt, &grouped);
        if grouped.grid.is_empty() {
            eject_chain(game, pos, Some(entity));
        } else {
            set_cycle(
                &mut entity.nbt,
                Cycle {
                    phase: Phase::Inserting,
                    countdown: 0,
                },
            );
        }
        true
    }

    pub(in crate::game) fn tick(
        game: &mut Game<'_>,
        pos: Pos,
        entity: &mut ModBlockEntity,
        rpm: f32,
    ) -> bool {
        let Some(layout_here) = layout(game, pos) else {
            return false;
        };
        let before = cycle(&entity.nbt);
        let lazy = lazy_turn(entity);
        if lazy
            && before.phase == Phase::Idle
            && rpm != 0.0
            && entity.items.first().is_some_and(Option::is_some)
        {
            start(game, pos, Some(entity), false);
        }
        if cycle(&entity.nbt).phase != Phase::Accepting && powered_edge(game, pos, entity) {
            start(game, pos, Some(entity), true);
        }
        let mut state = cycle(&entity.nbt);
        if state.phase == Phase::Inserting {
            let target = outlet(game, pos, layout_here);
            return (lazy || matches!(target, Some(Outlet::Depot)))
                && try_output(game, pos, entity, layout_here, target);
        }
        let action = state.tick(rpm);
        if state != before {
            set_cycle(&mut entity.nbt, state);
        }
        match action {
            Tick::None => return state.phase != before.phase,
            Tick::AssemblyComplete => {
                let grouped = grid(&entity.nbt);
                if crafter::target(pos, layout_here, &mut |p| layout(game, p)).is_some() {
                    set_cycle(&mut entity.nbt, Cycle::exporting(grouped.only_empty()));
                } else if let Some(output) = recipe(game, &grouped) {
                    set_grid(&mut entity.nbt, &GroupedItems::single(Some(output)));
                    set_cycle(&mut entity.nbt, Cycle::crafting());
                } else {
                    eject_chain(game, pos, Some(entity));
                }
            }
            Tick::ExportComplete => {
                let Some(at) = crafter::target(pos, layout_here, &mut |p| layout(game, p)) else {
                    eject_chain(game, pos, Some(entity));
                    return true;
                };
                let mut source = grid(&entity.nbt);
                let empty = source.only_empty();
                let Some(receiver) = self::entity(game, at) else {
                    eject_chain(game, pos, Some(entity));
                    return true;
                };
                let mut next = grid(&receiver.nbt);
                if cycle(&receiver.nbt).phase != Phase::Accepting
                    || !source.merge_onto(&mut next, layout_here.pointing)
                {
                    eject_chain(game, pos, Some(entity));
                    return true;
                }
                set_grid(&mut entity.nbt, &source);
                set_cycle(
                    &mut entity.nbt,
                    Cycle {
                        phase: Phase::Waiting,
                        countdown: 0,
                    },
                );
                let receiver_layout = layout(game, at).expect("targeting crafter has a layout");
                let ready = crafter::preceding(at, receiver_layout, &mut |p| layout(game, p))
                    .into_iter()
                    .all(|previous| {
                        previous == pos
                            || self::entity(game, previous)
                                .is_some_and(|e| cycle(&e.nbt).phase == Phase::Waiting)
                    });
                game.native_with_mod_block_entity(at, |receiver| {
                    set_grid(&mut receiver.nbt, &next);
                    if ready {
                        set_cycle(&mut receiver.nbt, Cycle::assembling());
                    }
                });
                game.native_block_entity_sync(at);
                if !empty {
                    crate::game::create_port_sounds::crafter_click(
                        game,
                        pos,
                        next.grid.len() as f32 / 16.0 + 0.5,
                    );
                }
            }
            Tick::CraftMiddle => {
                crate::game::create_port_sounds::crafter_craft(game, pos);
                return false;
            }
            Tick::CraftComplete => {
                let target = outlet(game, pos, layout_here);
                try_output(game, pos, entity, layout_here, target);
            }
        }
        true
    }

    pub(in crate::game) fn draw(
        game: &Game<'_>,
        pos: Pos,
        entity: &ModBlockEntity,
        draw: &mut BlockEntityDraw,
    ) {
        let Some(state) = layout(game, pos) else {
            return;
        };
        let axis = state.facing.axis();
        draw.push();
        create::orient(draw, axis);
        let time = draw.game_time as f64 + f64::from(draw.partial);
        let angle = create_engine::visual::angle(
            time,
            create::number(&entity.nbt, "Speed"),
            create_engine::visual::rotation_offset(pos, axis, false),
        );
        draw.rotate_about([0.0, 1.0, 0.0], angle, [0.5; 3]);
        game.native_draw_model(draw, "create:block/cogwheel_shaftless", 0xFFFFFF);
        draw.pop();

        draw.push();
        create::face_south(draw, state.facing);
        draw.rotate_about([0.0, 1.0, 0.0], 90.0, [0.5; 3]);
        let angle = match state.pointing {
            Pointing::Up => 0.0,
            Pointing::Left => 270.0,
            Pointing::Down => 180.0,
            Pointing::Right => 90.0,
        };
        draw.rotate_about([1.0, 0.0, 0.0], angle, [0.5; 3]);
        let connected = crafter::target(pos, state, &mut |p| layout(game, p)).is_some();
        let base = "create:block/mechanical_crafter/";
        if connected {
            game.native_draw_model(draw, &format!("{base}belt"), 0xFFFFFF);
            game.native_draw_model(draw, &format!("{base}belt_animated"), 0xFFFFFF);
        } else {
            game.native_draw_model(draw, &format!("{base}arrow"), 0xFFFFFF);
        }
        draw.pop();

        let running = cycle(&entity.nbt);
        let grouped = if running.phase == Phase::Idle {
            GroupedItems::single(entity.items.first().cloned().flatten())
        } else {
            grid(&entity.nbt)
        };
        if !matches!(running.phase, Phase::Crafting | Phase::Inserting)
            && (running.phase != Phase::Idle || create::integer(&entity.nbt, "Cover") != 0)
        {
            draw.push();
            create::face_south(draw, state.facing);
            draw.rotate_about([0.0, 1.0, 0.0], 90.0, [0.5; 3]);
            game.native_draw_model(draw, &format!("{base}lid"), 0xFFFFFF);
            draw.pop();
        }
        if running.phase == Phase::Crafting && running.countdown >= 1000 {
            return;
        }
        let advance = if running.phase == Phase::Exporting {
            let progress = ((1000 - running.countdown) as f32
                + crafter::countdown_speed(create::running_speed(game, &entity.nbt)) as f32
                    * draw.partial)
                / 1000.0;
            0.75 * progress.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let local = state.pointing.direction(Direction::South).vector();
        draw.push();
        create::face_south(draw, state.facing);
        draw.translate([
            0.5 + local[0] as f32 * advance,
            0.5 + local[1] as f32 * advance,
            1.08,
        ]);
        draw.scale([0.5; 3]);
        for ((x, y), stack) in grouped.grid {
            let Some(stack) = stack else { continue };
            draw.push();
            draw.translate([
                x as f32 * 0.5,
                y as f32 * 0.5,
                (x as f32 + y as f32 * 3.0) / 1024.0,
            ]);
            game.native_draw_item(
                draw,
                &stack,
                horizon_assets::item_model::DisplayContext::Fixed,
                None,
                0,
            );
            draw.pop();
        }
        draw.pop();
    }
}

pub(super) mod crushing {
    use super::{
        BlockEntity, Compound, Game, INVENTORY, ItemStack, ModBlockEntity, Pos, SLOTS, Tag,
        Transported, belt, create, processing_inventory, roll_count,
    };
    use create_engine::{
        crushing::{Action, Cycle, direction},
        kinetics::Axis,
        placement::Direction,
    };
    use horizon_core::random::RandomSource;
    use horizon_world::{chunk::BlockState, level::flags, registry::BlockStateId};
    use std::collections::BTreeMap;

    pub(in crate::game) const WHEEL: &str = "create:crushing_wheel";
    pub(in crate::game) const CONTROLLER: &str = "create:crushing_wheel_controller";

    const GAPS: [Direction; 2] = [Direction::East, Direction::South];

    fn offset(pos: Pos, side: Direction) -> Pos {
        let normal = side.vector();
        std::array::from_fn(|axis| pos[axis] + normal[axis])
    }

    fn axis(game: &Game<'_>, pos: Pos) -> Option<Axis> {
        if game.native_block(pos) != WHEEL {
            return None;
        }
        match game.native_block_property(pos, "axis").as_deref() {
            Some("x") => Some(Axis::X),
            Some("z") => Some(Axis::Z),
            _ => None,
        }
    }

    fn pair(game: &Game<'_>, center: Pos) -> Option<(Direction, Axis)> {
        let mut matches = GAPS.into_iter().filter_map(|side| {
            let a = offset(center, side.opposite());
            let b = offset(center, side);
            let axis = axis(game, a)?;
            (axis != side.axis() && self::axis(game, b) == Some(axis)).then_some((side, axis))
        });
        let pair = matches.next()?;
        matches.next().is_none().then_some(pair)
    }

    fn rpm(game: &Game<'_>, pos: Pos, update: (Pos, f32)) -> f32 {
        if pos == update.0 {
            return update.1;
        }
        match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
            Some(BlockEntity::Mod(entity)) if entity.id == WHEEL => {
                create::number(&entity.nbt, "Speed")
            }
            _ => 0.0,
        }
    }

    fn state(facing: Direction, valid: bool, game: &Game<'_>) -> Option<BlockStateId> {
        game.pack.assets.registry.id(&BlockState {
            name: CONTROLLER.into(),
            properties: BTreeMap::from([
                ("facing".to_owned(), facing.name().into()),
                ("valid".to_owned(), valid.to_string()),
            ])
            .into(),
        })
    }

    pub(in crate::game) fn wheel_updated(game: &mut Game<'_>, pos: Pos, axis: Axis, rpm: f32) {
        if axis == Axis::Y {
            return;
        }
        let sides = if axis == Axis::X {
            [Direction::North, Direction::South]
        } else {
            [Direction::West, Direction::East]
        };
        for side in sides {
            let center = offset(pos, side);
            let Some((gap, wheel_axis)) = pair(game, center) else {
                continue;
            };
            let current = game.world.block(center[0], center[1], center[2]);
            if !game.res.behaviours().is_air(current) && game.native_block(center) != CONTROLLER {
                continue;
            }
            // Iterate.directions sees the negative Z/X side before the positive side.
            let speed = self::rpm(game, offset(center, gap.opposite()), (pos, rpm));
            let other = self::rpm(game, offset(center, gap), (pos, rpm));
            let facing = direction(wheel_axis, gap, speed, other);
            let Some(target) = state(facing.unwrap_or(Direction::Down), facing.is_some(), game)
            else {
                continue;
            };
            if current != target {
                let dim = game.dim.id().to_owned();
                if !game.native_set_block(&dim, center, target, flags::UPDATE_ALL) {
                    continue;
                }
            }
            let speed = if facing.is_some() {
                (speed / 50.0).abs()
            } else {
                0.0
            };
            if game.native_with_mod_block_entity(center, |entity| {
                let changed = create::number(&entity.nbt, "Speed") != speed;
                if changed {
                    entity
                        .nbt
                        .insert("Speed".into(), Tag::Float(speed.to_bits()));
                }
                changed
            }) == Some(true)
            {
                game.native_block_entity_sync(center);
            }
        }
    }

    fn save(entity: &mut ModBlockEntity, cycle: Cycle) {
        let inventory = processing_inventory(
            &entity.items,
            cycle.remaining,
            cycle.recipe_time,
            cycle.applied,
        );
        entity.nbt.insert(INVENTORY.into(), inventory);
    }

    pub(in crate::game) fn default_nbt() -> Compound {
        Compound::from([
            (INVENTORY.into(), processing_inventory(&[], 0.0, 0.0, false)),
            ("Speed".into(), Tag::Float(0)),
        ])
    }

    fn cycle(entity: &ModBlockEntity) -> Option<Cycle> {
        let Tag::Compound(saved) = entity.nbt.get(&INVENTORY.into())? else {
            return None;
        };
        let remaining = create::number(saved, "ProcessingTime");
        let recipe_time = create::number(saved, "RecipeTime");
        (create::integer(saved, "Size") == SLOTS as i32
            && entity.items.len() == SLOTS
            && remaining.is_finite()
            && remaining >= 0.0
            && recipe_time.is_finite()
            && recipe_time >= 0.0)
            .then_some(Cycle {
                remaining,
                recipe_time,
                applied: create::integer(saved, "AppliedRecipe") != 0,
            })
    }

    fn down(game: &Game<'_>, pos: Pos) -> bool {
        game.native_block_property(pos, "facing").as_deref() == Some("down")
    }

    fn recipe(game: &Game<'_>, stack: &ItemStack) -> Option<(i32, Vec<(ItemStack, f32)>)> {
        let kind = match stack.item.as_ref() {
            "minecraft:cobblestone" => "create:milling",
            "minecraft:gravel" => "create:crushing",
            _ => return None,
        };
        for recipe in game.native_recipes_of_type(kind) {
            let json = &recipe.json;
            let Some(input) = json["ingredients"]
                .as_array()
                .filter(|list| list.len() == 1)
                .and_then(|list| create::ingredient(&list[0]))
                .and_then(|value| game.native_recipe_ingredient(&value))
            else {
                continue;
            };
            if !input.test(stack) {
                continue;
            }
            let duration = i32::try_from(json["processing_time"].as_i64()?).ok()?;
            let results = json["results"]
                .as_array()?
                .iter()
                .map(|row| {
                    let id = row["id"].as_str()?;
                    let count = row["count"].as_u64().unwrap_or(1);
                    let chance = row["chance"].as_f64().unwrap_or(1.0);
                    (count > 0 && count <= 64 && (0.0..=1.0).contains(&chance))
                        .then(|| (ItemStack::new(id, count as u32), chance as f32))
                })
                .collect::<Option<Vec<_>>>()?;
            return (duration > 0 && !results.is_empty() && results.len() < SLOTS)
                .then_some((duration, results));
        }
        None
    }

    pub(in crate::game) fn insert(
        game: &mut Game<'_>,
        pos: Pos,
        stack: &ItemStack,
        simulate: bool,
    ) -> u32 {
        if !down(game, pos) || pair(game, pos).is_none() {
            return 0;
        }
        let Some(BlockEntity::Mod(entity)) =
            game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
        else {
            return 0;
        };
        if entity.id != CONTROLLER
            || cycle(entity).is_none()
            || entity.items.iter().any(Option::is_some)
        {
            return 0;
        }
        let Some((duration, _)) = recipe(game, stack) else {
            return 0;
        };
        let moved = stack.count.min(create::stack_maximum(game, stack));
        if moved == 0 || simulate {
            return moved;
        }
        let accepted = game.native_with_mod_block_entity(pos, |entity| {
            let mut stack = stack.clone();
            stack.count = moved;
            entity.items[0] = Some(stack);
            let mut cycle = Cycle::default();
            cycle.start(duration);
            save(entity, cycle);
        });
        if accepted.is_some() {
            game.native_block_entity_sync(pos);
            moved
        } else {
            0
        }
    }

    fn eject(game: &mut Game<'_>, pos: Pos, stack: ItemStack) {
        let center = pos.map(|value| f64::from(value) + 0.5);
        let location = [center[0], center[1] - f64::from(0.55_f32), center[2]];
        let id = game.next_entity + 1;
        game.spawn_item(stack, location, [0.0; 3], 0);
        if !game.dropped.iter().rev().any(|item| item.id == id) {
            return;
        }
        let root = game
            .persist
            .raw
            .entry(id)
            .or_insert_with(|| Tag::Compound(Compound::new()));
        if let Tag::Compound(root) = root
            && let Tag::Compound(data) = root
                .entry("NeoForgeData".into())
                .or_insert_with(|| Tag::Compound(Compound::new()))
        {
            data.insert("BypassCrushingWheel".into(), Tag::IntArray(pos.to_vec()));
        }
    }

    pub(in crate::game) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
        let Some(mut cycle) = cycle(entity) else {
            return;
        };
        let resident = |at| crate::game::create_port_contraption::resident(game, at);
        if GAPS
            .into_iter()
            .any(|side| !resident(offset(pos, side)) || !resident(offset(pos, side.opposite())))
        {
            return;
        }
        if pair(game, pos).is_none() {
            let dim = game.dim.id().to_owned();
            if let Some(air) = game.pack.assets.registry.default_state("minecraft:air")
                && game.native_set_block(&dim, pos, air, flags::UPDATE_ALL)
            {
                for stack in entity.items.iter_mut().filter_map(Option::take) {
                    game.pop_resource(pos, stack);
                }
            }
            return;
        }
        let speed = create::running_speed(game, &entity.nbt);
        if !down(game, pos)
            || speed <= 0.0
            || !speed.is_finite()
            || entity.items.iter().all(Option::is_none)
        {
            return;
        }
        let count = entity.items[0].as_ref().map_or(0, |stack| stack.count);
        if !cycle.applied && count == 0 {
            return;
        }
        let previous = cycle;
        match cycle.tick(speed, count) {
            Action::Idle => {
                if let Some(Tag::Compound(saved)) = entity.nbt.get_mut(&INVENTORY.into()) {
                    saved.insert(
                        "ProcessingTime".into(),
                        Tag::Float(cycle.remaining.to_bits()),
                    );
                }
            }
            Action::Apply => {
                let Some((_, results)) = entity.items[0]
                    .as_ref()
                    .and_then(|stack| recipe(game, stack))
                else {
                    // A data reload can remove a recipe while it is in progress.
                    return;
                };
                let outputs = results
                    .into_iter()
                    .map(|(stack, chance)| {
                        let limit = create::stack_maximum(game, &stack);
                        (stack, chance, limit)
                    })
                    .collect::<Vec<_>>();
                entity.items.fill(None);
                for _ in 0..count {
                    for (stack, chance, limit) in &outputs {
                        let count =
                            roll_count(stack.count, *chance, || game.world.random.next_float());
                        if count != 0 {
                            let mut output = stack.clone();
                            output.count = count;
                            super::super::create_port_inventory::insert(
                                &mut entity.items[1..],
                                output,
                                *limit,
                            );
                        }
                    }
                }
                save(entity, cycle);
                game.native_block_entity_sync(pos);
            }
            Action::Eject => {
                let below = offset(pos, Direction::Down);
                let state = game.world.block(below[0], below[1], below[2]);
                if game.res.facts().get(state).name.as_ref() == "create:depot" {
                    let mut changed = false;
                    for slot in &mut entity.items {
                        let Some(stack) = slot else { continue };
                        let item = Transported::new(
                            belt::core_stack(game, stack.clone()),
                            belt::angle(game, stack),
                        );
                        let moved =
                            belt::insert_depot_from_side(game, below, &item, Direction::Down);
                        if moved != 0 {
                            stack.count -= moved;
                            if stack.count == 0 {
                                *slot = None;
                            }
                            changed = true;
                        }
                    }
                    // Direct input keeps the applied phase, even when the receiver
                    // takes the final result. Open-space ejection clears the cycle.
                    if changed || cycle != previous {
                        save(entity, cycle);
                    }
                    if changed {
                        game.native_block_entity_sync(pos);
                    }
                    return;
                }
                if !game.res.behaviours().is_air(state) {
                    return;
                }
                for stack in entity.items.iter_mut().filter_map(Option::take) {
                    eject(game, pos, stack);
                }
                save(entity, Cycle::default());
                game.native_block_entity_sync(pos);
            }
        }
    }
}

pub(super) fn container() -> ModContainer {
    ModContainer {
        slots: SLOTS,
        faces: std::array::from_fn(|_| Vec::new()),
        inputs: Vec::new(),
        outputs: Vec::new(),
        menu: None,
    }
}

fn cycle(entity: &ModBlockEntity) -> Option<CuttingCycle> {
    let Some(saved) = entity.nbt.get(&INVENTORY.into()) else {
        return Some(CuttingCycle::default());
    };
    let Tag::Compound(saved) = saved else {
        return None;
    };
    if create::integer(saved, "Size") != SLOTS as i32 {
        return None;
    }
    let remaining = create::number(saved, "ProcessingTime");
    let duration = create::number(saved, "RecipeTime");
    if !remaining.is_finite() || remaining < -1.0 || !duration.is_finite() || duration < 0.0 {
        return None;
    }
    Some(CuttingCycle {
        remaining,
        duration,
        applied: create::integer(saved, "AppliedRecipe") != 0,
        recipe_index: create::integer(&entity.nbt, "RecipeIndex").max(0) as usize,
    })
}

fn save(entity: &mut ModBlockEntity, cycle: CuttingCycle) {
    let inventory = processing_inventory(
        &entity.items,
        cycle.remaining,
        cycle.duration,
        cycle.applied,
    );
    entity.nbt.insert(INVENTORY.into(), inventory);
    entity
        .nbt
        .insert("RecipeIndex".into(), Tag::Int(cycle.recipe_index as i32));
}

fn processing_inventory(
    items: &[Option<ItemStack>],
    remaining: f32,
    duration: f32,
    applied: bool,
) -> Tag {
    let items = items
        .iter()
        .enumerate()
        .filter_map(|(slot, stack)| {
            let Tag::Compound(mut item) = horizon_items::item_nbt::item_to_nbt(stack.as_ref()?)
            else {
                return None;
            };
            item.insert("Slot".into(), Tag::Int(slot as i32));
            Some(Tag::Compound(item))
        })
        .collect();
    Tag::Compound(Compound::from([
        ("Size".into(), Tag::Int(SLOTS as i32)),
        ("Items".into(), Tag::List(items)),
        ("ProcessingTime".into(), Tag::Float(remaining.to_bits())),
        ("RecipeTime".into(), Tag::Float(duration.to_bits())),
        ("AppliedRecipe".into(), Tag::Byte(i8::from(applied))),
    ]))
}

struct CuttingRecipe {
    duration: u32,
    outputs: Vec<(ItemStack, f32)>,
}

pub(super) fn read_filter(game: &Game<'_>, entity: &ModBlockEntity) -> Option<Option<ItemStack>> {
    let Some(saved) = entity.nbt.get(&"Filter".into()) else {
        return Some(None);
    };
    if matches!(saved, Tag::Compound(values) if values.is_empty()) {
        return Some(None);
    }
    let item = item_from_nbt(saved)?;
    ordinary_filter_item(game, &item).then_some(Some(item))
}

pub(super) fn ordinary_filter_item(game: &Game<'_>, item: &ItemStack) -> bool {
    item.count != 0
        && !matches!(
            item.item.as_ref(),
            "create:filter" | "create:attribute_filter" | "create:package_filter"
        )
        && game.items().get(&item.item).is_some()
}

fn cutting_recipes(
    game: &Game<'_>,
    input: &ItemStack,
    filter: Option<&ItemStack>,
) -> Vec<CuttingRecipe> {
    let accepts = |item: &ItemStack| filter.is_none_or(|filter| filter.item == item.item);
    let custom = game.native_recipes_of_type("create:cutting");
    let mut recipes: Vec<_> = custom
        .iter()
        .filter_map(|recipe| {
            if !saw_processing::automated_recipe(&recipe.id) {
                return None;
            }
            let [ingredient] = recipe.json["ingredients"].as_array()?.as_slice() else {
                return None;
            };
            // Bundled single-item recipes use qualified strings. Skip
            // unrelated ones before resolving tags or allocating ingredients.
            if ingredient.as_str().is_some_and(|id| {
                !id.starts_with('#') && id.contains(':') && id != input.item.as_ref()
            }) {
                return None;
            }
            let ingredient = create::ingredient(ingredient)
                .and_then(|json| game.native_recipe_ingredient(&json))?;
            if !ingredient.test(input) {
                return None;
            }
            let duration = recipe
                .json
                .get("processing_time")
                .or_else(|| recipe.json.get("processingTime"))
                .map_or(Some(0), |value| u32::try_from(value.as_u64()?).ok())?;
            let results = recipe.json["results"].as_array()?;
            if !(1..=4).contains(&results.len()) {
                return None;
            }
            let outputs = results
                .iter()
                .map(|output| {
                    let chance = output
                        .get("chance")
                        .map_or(Some(1.0), |value| value.as_f64())?;
                    if !(0.0..=1.0).contains(&chance) {
                        return None;
                    }
                    let stack = if output.get("id").is_some() {
                        game.native_recipe_result(output)
                    } else {
                        let mut output = output.clone();
                        output["id"] = output["item"].clone();
                        game.native_recipe_result(&output)
                    }?;
                    Some((stack, chance as f32))
                })
                .collect::<Option<Vec<_>>>()?;
            accepts(&outputs[0].0)
                .then_some((recipe.id.as_str(), CuttingRecipe { duration, outputs }))
        })
        .collect();
    // The same authoritative provider as the native Stonecutter, including
    // merged world recipes and item tags. The original default enables these.
    recipes.extend(
        game.stations()
            .recipes
            .stonecutting
            .iter()
            .filter_map(|recipe| {
                let id = saw_processing::stonecutting_source_id(&recipe.id)?;
                (saw_processing::automated_recipe(id)
                    && recipe.input.test(input)
                    && accepts(&recipe.result)
                    && game.items().get(&recipe.result.item).is_some())
                .then(|| {
                    (
                        id,
                        CuttingRecipe {
                            duration: 50,
                            outputs: vec![(recipe.result.clone(), 1.0)],
                        },
                    )
                })
            }),
    );
    recipes.sort_by_key(|(id, _)| saw_processing::source_recipe_order(id));
    recipes.into_iter().map(|(_, recipe)| recipe).collect()
}

fn start(game: &Game<'_>, cycle: &mut CuttingCycle, input: &ItemStack, filter: Option<&ItemStack>) {
    let durations: Vec<_> = cutting_recipes(game, input, filter)
        .into_iter()
        .map(|recipe| recipe.duration)
        .collect();
    cycle.start(input.count, &durations);
}

fn apply(
    game: &mut Game<'_>,
    entity: &mut ModBlockEntity,
    input: &ItemStack,
    recipe: &CuttingRecipe,
) {
    entity.items.fill(None);
    let remainder = game
        .items()
        .get(&input.item)
        .and_then(|item| item.crafting_remainder.as_ref())
        .map(|id| ItemStack::new(id, 1));
    for _ in 0..input.count {
        for (stack, chance) in &recipe.outputs {
            let mut output = stack.clone();
            output.count = roll_count(output.count, *chance, || game.world.random.next_float());
            if output.count > 0 {
                let max = create::stack_maximum(game, &output);
                super::create_port_inventory::insert(&mut entity.items[1..], output, max);
            }
        }
        if let Some(output) = &remainder {
            let max = create::stack_maximum(game, output);
            super::create_port_inventory::insert(&mut entity.items[1..], output.clone(), max);
        }
    }
}

pub(super) fn landed(game: &mut Game<'_>, pos: Pos, stack: &mut ItemStack) -> bool {
    if stack.count == 0
        || !game.native_on("create")
        || game.native_block(pos) != super::create_port_drill::SAW
        || game.native_block_property(pos, "facing").as_deref() != Some("up")
    {
        return false;
    }
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return false;
    };
    if game.clock.frozen()
        || create::number(&entity.nbt, "Speed") == 0.0
        || entity.items.len() != SLOTS
        || entity.items.iter().any(Option::is_some)
    {
        return false;
    }
    let (Some(filter), Some(mut cycle)) = (read_filter(game, entity), cycle(entity)) else {
        return false;
    };
    let mut inserted = stack.clone();
    inserted.count = 1;
    start(game, &mut cycle, &inserted, filter.as_ref());
    if game
        .native_with_mod_block_entity(pos, |entity| {
            entity.items[0] = Some(inserted);
            save(entity, cycle);
        })
        .is_none()
    {
        return false;
    }
    stack.count -= 1;
    game.native_block_entity_sync(pos);
    true
}

pub(super) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity, rpm: f32) -> bool {
    if entity.items.len() != SLOTS {
        return false;
    }
    let (Some(filter), Some(mut cycle)) = (read_filter(game, entity), cycle(entity)) else {
        return false;
    };
    let first = game
        .native_block_property(pos, "axis_along_first")
        .as_deref()
        == Some("true");
    let motion = saw_processing::item_motion(first, rpm);
    let Some(input) = entity.items[0]
        .as_ref()
        .filter(|item| item.count > 0)
        .cloned()
        .or_else(|| entity.items.iter().flatten().next().cloned())
    else {
        return false;
    };
    let previous = cycle;
    match cycle.tick(rpm, entity.items[0].is_some()) {
        Action::Idle => return false,
        Action::Start => start(game, &mut cycle, &input, filter.as_ref()),
        Action::Advance => {}
        Action::Apply => {
            let recipes = cutting_recipes(game, &input, filter.as_ref());
            if let Some(index) = cycle.select_current(recipes.len()) {
                apply(game, entity, &input, &recipes[index]);
            }
            cycle.finish_application();
            activation_sound(game, pos, &input);
        }
        Action::Export => {
            let target = std::array::from_fn(|axis| pos[axis] + motion[axis]);
            let target_block = game.native_block(target);
            let millstone = target_block == "create:millstone";
            if millstone || matches!(target_block.as_str(), "create:depot" | "create:belt") {
                let from = Direction::from_vector(motion).expect("horizontal saw output");
                let mut changed = false;
                for slot in &mut entity.items {
                    let Some(stack) = slot else { continue };
                    let remaining = if millstone {
                        // The Millstone's default DirectBeltInputBehaviour
                        // goes through its recipe-filtered item capability.
                        game.native_insert_item(target, belt::world_direction(from), stack.clone())
                            .map_or(0, |remainder| remainder.count)
                    } else {
                        let item = Transported::new(
                            belt::core_stack(game, stack.clone()),
                            belt::angle(game, stack),
                        );
                        if target_block == "create:depot" {
                            stack.count - belt::insert_depot_from_side(game, target, &item, from)
                        } else if belt::insert_belt_from_side(game, target, &item, from, false) {
                            0
                        } else {
                            stack.count
                        }
                    };
                    if remaining != stack.count {
                        if remaining == 0 {
                            *slot = None;
                        } else {
                            stack.count = remaining;
                        }
                        changed = true;
                    }
                }
                if changed && millstone {
                    super::create_port_millstone::publish_inventory(game, target);
                }
                // Direct input retains the finished phase. Keep rejected
                // stacks in their slots and try again on the next tick.
                if !changed && cycle == previous {
                    return false;
                }
            } else {
                let motion = motion.map(f64::from);
                let at = [
                    f64::from(pos[0]) + 0.5 + motion[0] * 0.5,
                    f64::from(pos[1]) + 1.0,
                    f64::from(pos[2]) + 0.5 + motion[2] * 0.5,
                ];
                let velocity = [motion[0] * 0.0625, 0.125, motion[2] * 0.0625];
                for stack in entity.items.iter_mut().filter_map(Option::take) {
                    game.spawn_item(stack, at, velocity, 0);
                }
                cycle.finish_export();
            }
        }
    }
    save(entity, cycle);
    true
}

fn activation_sound(game: &mut Game<'_>, pos: Pos, input: &ItemStack) {
    let is_wood = game
        .pack
        .assets
        .registry
        .default_state(&input.item)
        .is_some_and(|id| {
            game.res.facts().get(id).sounds.place.as_ref() == "minecraft:block.wood.place"
        });
    let (sound, volume, pitch) = if is_wood {
        ("minecraft:entity.boat.paddle_land", 2.25, 1.5)
    } else {
        ("minecraft:ui.stonecutter.take_result", 0.375, 1.25)
    };
    let at = pos.map(|v| f64::from(v) + 0.5);
    game.with_level(|_, level, _| {
        level.native_play_sound(sound, at, 4, volume, pitch);
    });
}

pub(super) fn draw(game: &Game<'_>, pos: Pos, entity: &ModBlockEntity, draw: &mut BlockEntityDraw) {
    let Some(Tag::List(stacks)) = entity
        .nbt
        .get(&INVENTORY.into())
        .and_then(|t| t.get("Items"))
    else {
        return;
    };
    // The supported default recipe emits a single output stack.
    let [saved] = stacks.as_slice() else { return };
    let Some(stack) = horizon_items::item_nbt::item_from_nbt(saved) else {
        return;
    };
    let Some(cycle) = cycle(entity) else { return };
    let rpm = create::number(&entity.nbt, "Speed");
    let along_z = game
        .native_block_property(pos, "axis_along_first")
        .as_deref()
        != Some("true");
    let mut offset = if rpm == 0.0 {
        0.5
    } else if cycle.duration > 0.0 {
        let speed = (rpm.abs() / 32.0).clamp(1.0, 128.0);
        let offset =
            ((cycle.remaining + (0.5 - draw.partial) * speed) / cycle.duration).clamp(0.125, 1.0);
        (offset + if cycle.applied { 0.0 } else { 1.0 }) / 2.0
    } else {
        0.0
    };
    if (rpm < 0.0) ^ along_z {
        offset = 1.0 - offset;
    }
    let block = game
        .pack
        .assets
        .registry
        .default_state(&stack.item)
        .is_some();
    draw.push();
    if along_z {
        draw.rotate_about([0.0, 1.0, 0.0], 90.0, [0.0; 3]);
    }
    draw.translate([
        if along_z { -0.5 } else { 0.5 },
        if block { 0.925 } else { 13.0 / 16.0 },
        offset,
    ]);
    draw.scale([0.5; 3]);
    draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.0; 3]);
    game.native_draw_item(
        draw,
        &stack,
        horizon_assets::item_model::DisplayContext::Fixed,
        None,
        0,
    );
    draw.pop();
}
