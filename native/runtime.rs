// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Game-side adapter. The engine crate contains the portable algorithms.
use super::{Game, native_block_entity_draw::BlockEntityDraw};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModContainer},
    items::ItemStack,
    mod_api::NativeScreen,
};
use create_engine::{
    kinetics::{self, Axis, Kind, Node},
    pressing::{Action, PressCycle, tick_speed},
};
use horizon_world::{
    nbt::{Compound, Tag},
    registry::BlockStateId,
};
use serde_json::Value;

fn owns(block: &str) -> bool {
    matches!(
        block,
        "create:creative_motor"
            | "create:shaft"
            | "create:cogwheel"
            | "create:large_cogwheel"
            | "create:mechanical_press"
            | "create:depot"
    )
}

pub(super) fn entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if owns(block) && entity.is_none() {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if owns(block) {
        *ticks = true;
    }
}

fn number(nbt: &Compound, key: &str) -> f32 {
    match nbt.get(&key.into()) {
        Some(Tag::Float(n)) => f32::from_bits(*n),
        Some(Tag::Int(n)) => *n as f32,
        Some(Tag::Long(n)) => *n as f32,
        _ => 0.0,
    }
}

fn integer(nbt: &Compound, key: &str) -> i32 {
    match nbt.get(&key.into()) {
        Some(Tag::Int(n)) => *n,
        _ => 0,
    }
}

fn facing(value: &str) -> [i32; 3] {
    match value {
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        "up" => [0, 1, 0],
        "down" => [0, -1, 0],
        "south" => [0, 0, 1],
        _ => [0, 0, -1],
    }
}

fn node(game: &Game<'_>, pos: [i32; 3]) -> Option<Node> {
    let block = game.native_block(pos);
    let axis_property = game.native_block_property(pos, "axis");
    let direction = facing(
        &game
            .native_block_property(pos, "facing")
            .unwrap_or_else(|| "north".into()),
    );
    let axis = match axis_property.as_deref() {
        Some("x") => Axis::X,
        Some("y") => Axis::Y,
        Some("z") => Axis::Z,
        _ if direction[0] != 0 => Axis::X,
        _ if direction[1] != 0 => Axis::Y,
        _ => Axis::Z,
    };
    let kind = match block.as_str() {
        "create:creative_motor" => Kind::Motor {
            facing: direction,
            rpm: game
                .native_block_property(pos, "speed")
                .and_then(|s| s.parse().ok())
                .unwrap_or(16.0),
            capacity: 16384.0,
        },
        "create:shaft" => Kind::Shaft,
        "create:cogwheel" => Kind::Cog { large: false },
        "create:large_cogwheel" => Kind::Cog { large: true },
        "create:mechanical_press" => Kind::Press,
        _ => return None,
    };
    Some(Node { pos, axis, kind })
}

fn ingredient(json: &Value) -> Option<Value> {
    if json.is_string() {
        return Some(json.clone());
    }
    if let Some(id) = json.get("item").and_then(Value::as_str) {
        return Some(id.into());
    }
    if let Some(tag) = json.get("tag").and_then(Value::as_str) {
        return Some(format!("#{tag}").into());
    }
    json.as_array()
        .map(|values| Value::Array(values.iter().filter_map(ingredient).collect()))
}

fn result(game: &Game<'_>, stack: &ItemStack) -> Option<ItemStack> {
    for recipe in game.native_recipes_of_type("create:pressing") {
        let Some(inputs) = recipe.json["ingredients"].as_array() else {
            continue;
        };
        let Some(outputs) = recipe.json["results"].as_array() else {
            continue;
        };
        // The first milestone supports deterministic, single-input/single-output bulk pressing.
        if inputs.len() != 1
            || outputs.len() != 1
            || outputs[0]
                .get("chance")
                .and_then(Value::as_f64)
                .is_some_and(|c| c != 1.0)
        {
            continue;
        }
        let Some(input) = ingredient(&inputs[0]).and_then(|i| game.native_recipe_ingredient(&i))
        else {
            continue;
        };
        if !input.test(stack) {
            continue;
        }
        let Some(mut output) = game.native_recipe_result(&outputs[0]) else {
            continue;
        };
        output.count = output.count.checked_mul(stack.count)?;
        if output.count <= game.native_max_stack_size(&output.item)? {
            return Some(output);
        }
    }
    None
}

fn depot_input(game: &Game<'_>, pos: [i32; 3]) -> Option<ItemStack> {
    if game.native_block(pos) != "create:depot" {
        return None;
    }
    let BlockEntity::Mod(depot) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))? else {
        return None;
    };
    depot.items.first()?.clone()
}

pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if !owns(&entity.id) {
        return;
    }
    if entity.id == "create:depot" {
        let (name, count) = entity
            .items
            .first()
            .and_then(Option::as_ref)
            .map(|s| (s.item.to_string(), s.count as i32))
            .unwrap_or_default();
        let changed = entity.nbt.get(&"DisplayItem".into())
            != Some(&Tag::String(name.as_str().into()))
            || integer(&entity.nbt, "DisplayCount") != count;
        if changed {
            entity
                .nbt
                .insert("DisplayItem".into(), Tag::String(name.as_str().into()));
            entity.nbt.insert("DisplayCount".into(), Tag::Int(count));
            game.native_block_entity_sync(pos);
        }
        return;
    }
    let Some(start) = node(game, pos) else { return };
    let network = kinetics::solve(start, |at| node(game, at));
    let rpm = network.speed(pos);
    let old_rpm = number(&entity.nbt, "Speed");
    entity.nbt.insert("Speed".into(), Tag::Float(rpm.to_bits()));
    entity
        .nbt
        .insert("Capacity".into(), Tag::Float(network.capacity.to_bits()));
    entity
        .nbt
        .insert("Stress".into(), Tag::Float(network.stress.to_bits()));
    entity.nbt.insert(
        "NetworkStatus".into(),
        Tag::String(format!("{:?}", network.status).as_str().into()),
    );
    let mut sync = old_rpm != rpm;
    if entity.id == "create:mechanical_press" {
        let depot = [pos[0], pos[1] - 2, pos[2]];
        let plan = depot_input(game, depot)
            .and_then(|input| result(game, &input).map(|output| (input, output)));
        let mut cycle = PressCycle {
            running: integer(&entity.nbt, "Running") != 0,
            ticks: integer(&entity.nbt, "Ticks"),
            applied: integer(&entity.nbt, "Applied") != 0,
        };
        let action = cycle.tick(rpm, plan.is_some());
        if action == Action::Press {
            if let Some((input, output)) = plan {
                let changed = game
                    .native_with_mod_block_entity(depot, |target| {
                        if target.id != "create:depot" || target.items.first() != Some(&Some(input))
                        {
                            return false;
                        }
                        target.items[0] = Some(output);
                        true
                    })
                    .unwrap_or(false);
                if changed {
                    game.native_block_entity_sync(depot);
                }
            }
        }
        entity
            .nbt
            .insert("Running".into(), Tag::Int(i32::from(cycle.running)));
        entity.nbt.insert("Ticks".into(), Tag::Int(cycle.ticks));
        entity
            .nbt
            .insert("Applied".into(), Tag::Int(i32::from(cycle.applied)));
        entity
            .nbt
            .insert("LastTick".into(), Tag::Long(game.world.game_time));
        sync |= action != Action::None;
    }
    if sync {
        game.native_block_entity_sync(pos);
    }
}

pub(super) fn update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    if owns(&entity.id) && tag.is_none() {
        *tag = Some(entity.nbt.clone());
    }
}

pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if block == "create:depot" && container.is_none() {
        *container = Some(ModContainer {
            slots: 1,
            faces: std::array::from_fn(|_| vec![0]),
            inputs: vec![0],
            outputs: vec![0],
            menu: Some("create:depot".into()),
        });
    }
}

pub(super) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    if block == "create:depot" && screen.is_none() {
        *screen = Some(NativeScreen {
            width: 176.0,
            height: 166.0,
            slots: vec![Some([80.0, 35.0])],
            inventory_x: 8.0,
            inventory_y: 84.0,
            ..NativeScreen::default()
        });
    }
}

pub(super) fn models(models: &mut Vec<String>) {
    models.extend(
        [
            "shaft",
            "cogwheel",
            "large_cogwheel",
            "creative_motor/block",
            "mechanical_press/block",
            "mechanical_press/head",
            "depot/block",
        ]
        .map(|name| format!("create:block/{name}")),
    );
}

fn orient(draw: &mut BlockEntityDraw, axis: Axis) {
    match axis {
        Axis::X => draw.rotate_about([0.0, 0.0, 1.0], -90.0, [0.5; 3]),
        Axis::Z => draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.5; 3]),
        Axis::Y => {}
    }
}

pub(super) fn draw(
    game: &Game<'_>,
    pos: [i32; 3],
    block: &str,
    _state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    if !owns(block) {
        return;
    }
    if block == "create:depot" {
        game.native_draw_model(draw, "create:block/depot/block", 0xFFFFFF);
        if let Some(Tag::String(item)) = entity.nbt.get(&"DisplayItem".into()) {
            if integer(&entity.nbt, "DisplayCount") > 0 {
                let Ok(item) = item.to_string() else { return };
                let stack = ItemStack::new(&item, 1);
                draw.push();
                draw.translate([0.5, 1.0, 0.5]);
                game.native_draw_item(
                    draw,
                    &stack,
                    horizon_assets::item_model::DisplayContext::Ground,
                    None,
                    0,
                );
                draw.pop();
            }
        }
        return;
    }
    let Some(node) = node(game, pos) else { return };
    if matches!(node.kind, Kind::Motor { .. } | Kind::Press) {
        let yaw = match game.native_block_property(pos, "facing").as_deref() {
            Some("east") => 90.0,
            Some("south") => 180.0,
            Some("west") => 270.0,
            _ => 0.0,
        };
        draw.push();
        draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.5; 3]);
        let model = if node.kind == Kind::Press {
            "create:block/mechanical_press/block"
        } else {
            "create:block/creative_motor/block"
        };
        game.native_draw_model(draw, model, 0xFFFFFF);
        if node.kind == Kind::Press {
            let last = match entity.nbt.get(&"LastTick".into()) {
                Some(Tag::Long(t)) => *t,
                _ => draw.game_time,
            };
            let elapsed = (draw.game_time - last).max(0) as f32 + draw.partial;
            let ticks = if integer(&entity.nbt, "Running") == 0 {
                0.0
            } else {
                (integer(&entity.nbt, "Ticks") as f32
                    + elapsed * tick_speed(number(&entity.nbt, "Speed")) as f32)
                    .min(240.0)
            };
            let travel = if ticks <= 120.0 {
                ticks / 120.0
            } else {
                (240.0 - ticks) / 120.0
            };
            draw.translate([0.0, -travel, 0.0]);
            game.native_draw_model(draw, "create:block/mechanical_press/head", 0xFFFFFF);
        }
        draw.pop();
    }
    draw.push();
    orient(draw, node.axis);
    let angle = ((draw.game_time as f64 + f64::from(draw.partial))
        * f64::from(number(&entity.nbt, "Speed"))
        * 0.3)
        .rem_euclid(360.0) as f32;
    draw.rotate_about([0.0, 1.0, 0.0], angle, [0.5; 3]);
    let model = match node.kind {
        Kind::Cog { large: false } => "create:block/cogwheel",
        Kind::Cog { large: true } => "create:block/large_cogwheel",
        _ => "create:block/shaft",
    };
    game.native_draw_model(draw, model, 0xFFFFFF);
    draw.pop();
}
