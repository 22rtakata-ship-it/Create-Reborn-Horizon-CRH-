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
    let items = entity
        .items
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
    entity.nbt.insert(
        INVENTORY.into(),
        Tag::Compound(Compound::from([
            ("Size".into(), Tag::Int(SLOTS as i32)),
            ("Items".into(), Tag::List(items)),
            (
                "ProcessingTime".into(),
                Tag::Float(cycle.remaining.to_bits()),
            ),
            ("RecipeTime".into(), Tag::Float(cycle.duration.to_bits())),
            ("AppliedRecipe".into(), Tag::Byte(i8::from(cycle.applied))),
        ])),
    );
    entity
        .nbt
        .insert("RecipeIndex".into(), Tag::Int(cycle.recipe_index as i32));
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
            let depot = target_block == "create:depot";
            if depot || target_block == "create:millstone" {
                let from = Direction::from_vector(motion).expect("horizontal saw output");
                let mut changed = false;
                for slot in &mut entity.items {
                    let Some(stack) = slot else { continue };
                    let remaining = if depot {
                        let item = Transported::new(
                            belt::core_stack(game, stack.clone()),
                            belt::angle(game, stack),
                        );
                        stack.count - belt::insert_depot_from_side(game, target, &item, from)
                    } else {
                        // The Millstone's default DirectBeltInputBehaviour
                        // goes through its recipe-filtered item capability.
                        game.native_insert_item(target, belt::world_direction(from), stack.clone())
                            .map_or(0, |remainder| remainder.count)
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
                // The receiver may have already ticked this turn. Publish its
                // canonical input before the next Millstone tick or a save.
                if changed
                    && !depot
                    && game.native_with_mod_block_entity(target, super::create_port_millstone::data)
                        == Some(true)
                {
                    game.native_block_entity_sync(target);
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
