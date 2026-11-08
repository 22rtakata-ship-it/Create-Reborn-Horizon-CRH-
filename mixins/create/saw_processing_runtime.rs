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
