// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// IWrenchable, EncasableBlock, ManualApplicationRecipe and HandCrankBlock.
use super::Game;
use crate::{containers::BlockEntity, items::ItemStack};
use create_engine::{kinetics::Axis, placement::Direction};
use horizon_world::nbt::Tag;

fn secondary(game: &Game<'_>) -> bool {
    game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
}
fn bypass(game: &Game<'_>) -> bool {
    secondary(game)
        && (game
            .inventory
            .get(game.selected)
            .is_some_and(Option::is_some)
            || game.offhand.is_some())
}
fn manual(game: &Game<'_>, pos: [i32; 3]) -> Option<(String, bool)> {
    let held = game.held()?;
    let block = game.native_block(pos);
    let block_stack = ItemStack::new(block.as_str(), 1);
    for recipe in game.native_recipes_of_type("create:item_application") {
        let ingredients = recipe.json["ingredients"].as_array()?;
        if ingredients.len() != 2 {
            continue;
        }
        let Some(a) = crate::game::create_port::ingredient(&ingredients[0])
            .and_then(|v| game.native_recipe_ingredient(&v))
        else {
            continue;
        };
        let Some(b) = crate::game::create_port::ingredient(&ingredients[1])
            .and_then(|v| game.native_recipe_ingredient(&v))
        else {
            continue;
        };
        if a.test(&block_stack) && b.test(held) {
            return Some((
                recipe.json["results"][0]["id"].as_str()?.to_owned(),
                !recipe.json["keepHeldItem"].as_bool().unwrap_or(false),
            ));
        }
    }
    None
}
pub(super) fn reacts(game: &Game<'_>, pos: [i32; 3]) -> bool {
    if !game.native_on("create") {
        return false;
    }
    let block = game.native_block(pos);
    let held = game.held().map(|s| s.item.as_ref());
    if held == Some("create:wrench") && crate::game::create_port::owns(&block) {
        return true;
    }
    if bypass(game) {
        return false;
    }
    block == "create:hand_crank"
        || (block == "create:shaft"
            && matches!(held, Some("create:andesite_casing" | "create:brass_casing")))
        || manual(game, pos).is_some()
}
fn give(game: &mut Game<'_>, stack: ItemStack, pos: [i32; 3]) {
    let uuid = game.native_acting_id();
    if let Some(rest) = game.native_give_stack(uuid, stack) {
        game.spawn_item(rest, pos.map(|v| f64::from(v) + 0.5), [0.0, 0.2, 0.0], 10);
    }
}
fn switch(game: &mut Game<'_>, pos: [i32; 3], block: &str) -> bool {
    let Some(state) = game.pack.assets.registry.default_state(block) else {
        return false;
    };
    let axis = game.native_block_property(pos, "axis");
    let nbt = match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(e)) => Some(e.nbt.clone()),
        _ => None,
    };
    let dim = game.dim.id().to_owned();
    if !game.native_set_block(&dim, pos, state, 3) {
        return false;
    }
    if let Some(axis) = axis {
        game.native_set_block_property(pos, "axis", &axis);
    }
    if let Some(nbt) = nbt {
        game.native_with_mod_block_entity(pos, |e| e.nbt = nbt);
    }
    crate::game::create_port_kinetic_runtime::invalidate();
    game.native_block_entity_sync(pos);
    true
}
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3]) -> bool {
    if !reacts(game, pos) {
        return false;
    }
    let block = game.native_block(pos);
    let wrench = game
        .held()
        .is_some_and(|s| s.item.as_ref() == "create:wrench");
    if wrench {
        if secondary(game) {
            if block.ends_with("encased_shaft") {
                switch(game, pos, "create:shaft");
            } else {
                let contents = game
                    .native_with_mod_block_entity(pos, |e| {
                        e.items
                            .iter_mut()
                            .filter_map(Option::take)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if switch(game, pos, "minecraft:air") {
                    for stack in contents {
                        give(game, stack, pos);
                    }
                    if !game.creative {
                        give(game, ItemStack::new(block.as_str(), 1), pos);
                    }
                } else {
                    game.native_with_mod_block_entity(pos, |e| {
                        for (slot, item) in e.items.iter_mut().zip(contents) {
                            *slot = Some(item);
                        }
                    });
                }
            }
            crate::game::create_port_sounds::wrench(game, pos, true);
        } else if let Some(face) = Direction::from_vector(face) {
            if let Some(axis) = game.native_block_property(pos, "axis") {
                let axis = match axis.as_str() {
                    "x" => Axis::X,
                    "y" => Axis::Y,
                    _ => Axis::Z,
                };
                let direction = match axis {
                    Axis::X => Direction::East,
                    Axis::Y => Direction::Up,
                    Axis::Z => Direction::South,
                };
                let rotated = direction.clockwise(face.axis()).axis();
                let changed = game.native_set_block_property(
                    pos,
                    "axis",
                    match rotated {
                        Axis::X => "x",
                        Axis::Y => "y",
                        Axis::Z => "z",
                    },
                );
                if changed {
                    crate::game::create_port_sounds::wrench(game, pos, false);
                }
            } else if let Some(direction) = game.native_block_property(pos, "facing") {
                if block != "create:mechanical_press" || face.axis() == Axis::Y {
                    let direction = Direction::ALL
                        .into_iter()
                        .find(|d| d.name() == direction)
                        .unwrap();
                    if game.native_set_block_property(
                        pos,
                        "facing",
                        direction.clockwise(face.axis()).name(),
                    ) {
                        crate::game::create_port_sounds::wrench(game, pos, false);
                    }
                }
            }
            crate::game::create_port_kinetic_runtime::invalidate();
        }
        return true;
    }
    if block == "create:hand_crank" {
        let back = secondary(game);
        game.native_with_mod_block_entity(pos, |e| {
            e.nbt.insert("InUse".into(), Tag::Int(10));
            e.nbt.insert("Backwards".into(), Tag::Byte(i8::from(back)));
        });
        crate::game::create_port_kinetic_runtime::invalidate();
        game.native_block_entity_sync(pos);
        crate::game::create_port_sounds::crank(game, pos);
        if let Some(player) = game.player.as_mut() {
            player.cause_food_exhaustion(32.0 * 0.01);
        }
        return true;
    }
    if block == "create:shaft" {
        let item = game.held().map(|s| s.item.to_string()).unwrap_or_default();
        let target = match item.as_str() {
            "create:andesite_casing" => Some("create:andesite_encased_shaft"),
            "create:brass_casing" => Some("create:brass_encased_shaft"),
            _ => None,
        };
        if let Some(target) = target {
            switch(game, pos, target);
            crate::game::create_port_sounds::casing(game, pos);
            return true;
        }
    }
    if let Some((result, consume)) = manual(game, pos) {
        if !game.may_build() {
            return true;
        }
        let dim = game.dim.id().to_owned();
        if let Some(state) = game.pack.assets.registry.default_state(&result) {
            if game.native_set_block(&dim, pos, state, 3) {
                if consume {
                    game.consume_held();
                }
                crate::game::create_port_sounds::casing(game, pos);
            }
        }
        return true;
    }
    false
}
