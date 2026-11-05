// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::{Game, create_port as create, native_block_entity_draw::BlockEntityDraw};
use crate::{
    containers::{ModBlockEntity, ModContainer},
    items::ItemStack,
};
use create_engine::{
    kinetics::Pos,
    saw_processing::{self, Action, CuttingCycle},
};
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

// RuntimeDataGenerator.cuttingRecipes provides these recipes for the original
// vanilla logs and bark blocks. The ordinary default cycle uses 50 ticks.
fn wood_recipe(stack: &ItemStack) -> Option<ItemStack> {
    const WOODS: &[&str] = &[
        "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove", "cherry",
    ];
    let item = stack.item.strip_prefix("minecraft:")?;
    let stripped = item.strip_prefix("stripped_");
    let (wood, suffix) = stripped.unwrap_or(item).rsplit_once('_')?;
    let count = match (wood, suffix) {
        ("bamboo", "block") => 3,
        ("crimson" | "warped", "stem" | "hyphae") => 6,
        (wood, "log" | "wood") if WOODS.contains(&wood) => 6,
        _ => return None,
    };
    let (output, count) = if stripped.is_some() {
        (format!("minecraft:{wood}_planks"), count)
    } else {
        (format!("minecraft:stripped_{item}"), 1)
    };
    Some(ItemStack::new(output.as_str(), count))
}

fn unfiltered(entity: &ModBlockEntity) -> bool {
    match entity.nbt.get(&"Filter".into()) {
        None => true,
        Some(Tag::Compound(filter)) => filter.is_empty(),
        _ => false,
    }
}

fn start(cycle: &mut CuttingCycle, input: &ItemStack) {
    let durations: &[u32] = if wood_recipe(input).is_some() {
        &[50]
    } else {
        &[]
    };
    cycle.start(input.count, durations);
}

pub(super) fn landed(game: &mut Game<'_>, pos: Pos, stack: &mut ItemStack) -> bool {
    if stack.count == 0
        || !game.native_on("create")
        || game.native_block(pos) != super::create_port_drill::SAW
        || game.native_block_property(pos, "facing").as_deref() != Some("up")
    {
        return false;
    }
    let frozen = game.clock.frozen();
    let mut inserted = stack.clone();
    inserted.count = 1;
    let accepted = game.native_with_mod_block_entity(pos, |entity| {
        if frozen
            || create::number(&entity.nbt, "Speed") == 0.0
            || !unfiltered(entity)
            || entity.items.len() != SLOTS
            || entity.items.iter().any(Option::is_some)
        {
            return false;
        }
        let Some(mut cycle) = cycle(entity) else {
            return false;
        };
        start(&mut cycle, &inserted);
        entity.items[0] = Some(inserted);
        save(entity, cycle);
        true
    });
    if accepted != Some(true) {
        return false;
    }
    stack.count -= 1;
    game.native_block_entity_sync(pos);
    true
}

pub(super) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity, rpm: f32) -> bool {
    if entity.items.len() != SLOTS || !unfiltered(entity) {
        return false;
    }
    let Some(mut cycle) = cycle(entity) else {
        return false;
    };
    let first = game
        .native_block_property(pos, "axis_along_first")
        .as_deref()
        == Some("true");
    let motion = saw_processing::item_motion(first, rpm).map(f64::from);
    let Some(input) = entity.items[0]
        .as_ref()
        .filter(|item| item.count > 0)
        .cloned()
        .or_else(|| entity.items.iter().flatten().next().cloned())
    else {
        return false;
    };
    match cycle.tick(rpm, entity.items[0].is_some()) {
        Action::Idle => return false,
        Action::Start => start(&mut cycle, &input),
        Action::Advance => {}
        Action::Apply => {
            if let Some(mut output) = wood_recipe(&input) {
                output.count *= input.count;
                entity.items[0] = None;
                let max = create::stack_maximum(game, &output);
                super::create_port_inventory::insert(&mut entity.items[1..], output, max);
            }
            cycle.finish_application();
            activation_sound(game, pos, &input);
        }
        Action::Export => {
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
