// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Client adapter for crank interpolation, depot piles, goggles and kinetic feedback.
use super::{Game, native_block_entity_draw::BlockEntityDraw};
use crate::game::create_port::{integer, number};
use crate::{
    containers::{BlockEntity, ModBlockEntity},
    hud::HudBatch,
    items::ItemStack,
    particles::{ParticleOptions, ParticleType},
};
use create_engine::{
    kinetics::{Axis, Pos},
    placement::Direction,
    visual::{self, CrankMotion},
};
use horizon_core::random::{LegacyRandom, RandomSource};
use horizon_world::nbt::{Compound, Tag};
use std::{cell::RefCell, collections::BTreeMap};
#[derive(Default)]
struct Motion {
    tick: i64,
    block: String,
    crank: CrankMotion,
    dial: f32,
    previous_dial: f32,
    speed: f32,
    status: String,
    applied: bool,
    indicator: i64,
}
thread_local! {static MOTION: RefCell<BTreeMap<(String,Pos),Motion>> = const {RefCell::new(BTreeMap::new())};}
fn text(nbt: &Compound, key: &str) -> String {
    match nbt.get(&key.into()) {
        Some(Tag::String(s)) => s.to_string().unwrap_or_default(),
        _ => String::new(),
    }
}
fn in_tag(game: &Game<'_>, item: &ItemStack, tag: &str) -> bool {
    game.native_recipe_ingredient(&serde_json::Value::String(format!("#{tag}")))
        .is_some_and(|i| i.test(item))
}
fn key(game: &Game<'_>, pos: Pos) -> (String, Pos) {
    (game.dim.id().to_owned(), pos)
}
pub(super) fn crank_angle(game: &Game<'_>, pos: Pos, partial: f32) -> f32 {
    MOTION.with(|m| {
        m.borrow()
            .get(&key(game, pos))
            .map_or(0.0, |v| v.crank.rendered(partial))
    })
}
pub(super) fn dial(game: &Game<'_>, pos: Pos, partial: f32) -> f32 {
    MOTION.with(|m| {
        m.borrow().get(&key(game, pos)).map_or(0.0, |v| {
            v.previous_dial + (v.dial - v.previous_dial) * partial
        })
    })
}
pub(super) fn tick(game: &mut Game<'_>) {
    if !game.native_window() || !game.native_on("create") {
        return;
    }
    let time = game.world.game_time;
    let entries: Vec<_> = game
        .world
        .block_entities
        .iter()
        .filter_map(|(&(x, y, z), entity)| {
            let BlockEntity::Mod(e) = entity else {
                return None;
            };
            crate::game::create_port::owns(&game.native_block([x, y, z]))
                .then(|| ([x, y, z], game.native_block([x, y, z]), e.nbt.clone()))
        })
        .collect();
    for (pos, block, nbt) in entries {
        let speed = number(&nbt, "Speed");
        let status = text(&nbt, "NetworkStatus");
        let applied = integer(&nbt, "Applied") != 0;
        let (particles, stress_effect) = MOTION.with(|m| {
            let mut m = m.borrow_mut();
            let state = m.entry(key(game, pos)).or_default();
            if state.block != block {
                *state = Motion {
                    tick: -1,
                    block: block.clone(),
                    indicator: -100,
                    ..Motion::default()
                };
            }
            if state.tick == time {
                return (false, None);
            }
            let previous_status = state.status.clone();
            if state.speed != speed && speed != 0.0 {
                state.indicator = time;
            }
            state.speed = speed;
            state.status = status.clone();
            state.crank.tick(speed);
            state.previous_dial = state.dial;
            state.dial += (number(&nbt, "Value") - state.dial) * 0.125;
            if state.dial > 1.0 && game.world.random.next_float() < 0.5 {
                state.dial -= (state.dial - 1.0) * game.world.random.next_float();
            }
            let emit = applied && !state.applied;
            state.applied = applied;
            state.tick = time;
            let stress = if previous_status != status
                && (previous_status == "Overstressed" || status == "Overstressed")
            {
                Some(status == "Overstressed")
            } else {
                None
            };
            (emit, stress)
        });
        if particles {
            if let Some(Tag::List(items)) = nbt.get(&"ParticleItems".into()) {
                for item in items
                    .iter()
                    .filter_map(horizon_items::item_nbt::item_from_nbt)
                {
                    let options = game.item_particle(&item);
                    let y = if integer(&nbt, "Mode") == 1 {
                        f64::from(pos[1]) - 1.25
                    } else {
                        f64::from(pos[1]) - 1.0
                    };
                    for _ in 0..15 {
                        let velocity = [
                            (game.world.random.next_double() - 0.5) * 0.25,
                            0.125,
                            (game.world.random.next_double() - 0.5) * 0.25,
                        ];
                        game.particles.add(
                            &options,
                            [f64::from(pos[0]) + 0.5, y, f64::from(pos[2]) + 0.5],
                            velocity,
                        );
                    }
                    trace(
                        "press_particles",
                        pos,
                        serde_json::json!({"item":item.item.as_ref(),"count":15}),
                    );
                }
            }
        }
        if let Some(over) = stress_effect {
            let options = ParticleOptions::simple(if over {
                ParticleType::Smoke
            } else {
                ParticleType::Cloud
            });
            let amount = if over { 5 } else { 2 };
            let spread = if over { 0.2 } else { 0.075 };
            for _ in 0..amount {
                let velocity =
                    std::array::from_fn(|_| (game.world.random.next_double() - 0.5) * spread * 2.0);
                game.particles
                    .add(&options, pos.map(|v| f64::from(v) + 0.5), velocity);
            }
            trace(
                "stress_particles",
                pos,
                serde_json::json!({"overstressed":over,"count":amount}),
            );
        }
    }
    MOTION.with(|m| {
        m.borrow_mut()
            .retain(|(dimension, _), v| dimension == game.dim.id() && time - v.tick < 200)
    });
}
fn trace(event: &str, pos: Pos, details: serde_json::Value) {
    if std::env::var_os("CREATE_HORIZON_TRACE").is_some() {
        log::info!(
            "Create visual: {}",
            serde_json::json!({"event":event,"pos":pos,"details":details})
        );
    }
}
fn saved_stack(nbt: &Compound) -> Option<ItemStack> {
    nbt.get(&"Item".into())
        .and_then(horizon_items::item_nbt::item_from_nbt)
}
fn stack(game: &Game<'_>, draw: &mut BlockEntityDraw, item: &ItemStack, angle: f32) {
    let block = game
        .pack
        .assets
        .registry
        .default_state(&item.item)
        .is_some();
    let upright = in_tag(game, item, "create:upright_on_belt")
        && !in_tag(game, item, "create:not_upright_on_belt");
    draw.rotate_about([0.0, 1.0, 0.0], angle, [0.0; 3]);
    let mut random = LegacyRandom::new(0);
    if upright {
        draw.translate([0.0, 3.0 / 32.0, -1.0 / 16.0]);
    }
    for i in 0..visual::depot_layers(item.count) {
        draw.push();
        if block {
            draw.translate([
                random.next_float() * 0.0625 * i as f32,
                0.0,
                random.next_float() * 0.0625 * i as f32,
            ]);
        }
        draw.scale([0.5; 3]);
        if !block && !upright {
            draw.translate([0.0, -3.0 / 16.0, 0.0]);
            draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.0; 3]);
        }
        game.native_draw_item(
            draw,
            item,
            horizon_assets::item_model::DisplayContext::Fixed,
            None,
            0,
        );
        draw.pop();
        if !upright {
            if !block {
                draw.rotate_about([0.0, 1.0, 0.0], 10.0, [0.0; 3]);
            }
            draw.translate([0.0, if block { 1.0 / 64.0 } else { 1.0 / 16.0 }, 0.0]);
        } else {
            draw.translate([0.0, 0.0, -1.0 / 16.0]);
        }
    }
}
pub(super) fn depot(game: &Game<'_>, _: Pos, entity: &ModBlockEntity, draw: &mut BlockEntityDraw) {
    draw.push();
    draw.translate([0.5, 15.0 / 16.0, 0.5]);
    if let Some(Tag::Compound(held)) = entity.nbt.get(&"HeldItem".into()) {
        if let Some(item) = saved_stack(held) {
            draw.push();
            stack(game, draw, &item, integer(held, "Angle") as f32);
            draw.pop();
        }
    }
    if let Some(Tag::List(items)) = entity
        .nbt
        .get(&"OutputBuffer".into())
        .and_then(|t| t.get("Items"))
    {
        for (i, tag) in items.iter().enumerate() {
            let Some(item) = horizon_items::item_nbt::item_from_nbt(tag) else {
                continue;
            };
            let slot = tag
                .get("Slot")
                .and_then(|t| match t {
                    Tag::Byte(v) => Some(*v as usize),
                    Tag::Int(v) => Some(*v as usize),
                    _ => None,
                })
                .unwrap_or(i);
            draw.push();
            draw.rotate_about([0.0, 1.0, 0.0], 45.0 * slot as f32, [0.0; 3]);
            draw.translate([0.35, 0.0, 0.0]);
            stack(
                game,
                draw,
                &item,
                (LegacyRandom::new(slot as i64 + 1).next_float() * 360.0) as i32 as f32,
            );
            draw.pop();
        }
    }
    draw.pop();
}
pub(super) fn hints(game: &Game<'_>, pos: Pos, draw: &mut BlockEntityDraw) {
    let Some(node) = crate::game::create_port::node(game, pos) else {
        return;
    };
    let age = MOTION.with(|m| {
        m.borrow()
            .get(&key(game, pos))
            .map(|v| draw.game_time - v.indicator)
    });
    if let Some(age) = age.filter(|a| *a >= 0 && *a < 10) {
        let radius = 0.5 + age as f32 / 10.0 * 0.125;
        let speed = number_from_game(game, pos, "Speed");
        if speed != 0.0 {
            for i in 0..20 {
                let a = (i as f32 * 18.0 + age as f32 * speed.signum() * 8.0).to_radians();
                let (u, v) = (radius * a.cos(), radius * a.sin());
                let offset = match node.axis {
                    Axis::X => [0.0, u, v],
                    Axis::Y => [u, 0.0, v],
                    Axis::Z => [u, v, 0.0],
                };
                draw.push();
                draw.translate(offset.map(|v| v + 0.5));
                draw.scale([0.10; 3]);
                draw.translate([-0.5; 3]);
                game.native_draw_model(draw, "create:block/port_hint", 0xB4F06A);
                draw.pop();
            }
        }
    }
    let Some(hit) = game.native_raycast(6.0) else {
        return;
    };
    if [hit.block_x, hit.block_y, hit.block_z] != pos {
        return;
    }
    let held = game.held().map(|s| s.item.as_ref()).unwrap_or("");
    if !matches!(
        held,
        "create:shaft"
            | "create:cogwheel"
            | "create:large_cogwheel"
            | "create:wrench"
            | "create:vertical_gearbox"
            | "create:gearbox"
    ) {
        return;
    }
    for d in Direction::ALL {
        if node.shaft_port(d.vector()) {
            draw.push();
            super::create_port::face_south(draw, d);
            draw.translate([0.0, 0.0, 0.35]);
            game.native_draw_model(draw, "create:block/port_hint", 0x80D8FF);
            draw.pop();
        }
    }
}
fn number_from_game(game: &Game<'_>, pos: Pos, name: &str) -> f32 {
    match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(e)) => number(&e.nbt, name),
        _ => 0.0,
    }
}
pub(super) fn hud(game: &Game<'_>, batch: &mut HudBatch) {
    if game.native_mod_screen().is_some() || !game.native_on("create") {
        return;
    }
    if !game.armor[0]
        .as_ref()
        .is_some_and(|s| s.item.as_ref() == "create:goggles")
    {
        return;
    }
    let Some(hit) = game.native_raycast(8.0) else {
        return;
    };
    let pos = [hit.block_x, hit.block_y, hit.block_z];
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return;
    };
    if crate::game::create_port::node(game, pos).is_none() {
        return;
    }
    let rpm = number(&entity.nbt, "Speed");
    let capacity = number(&entity.nbt, "Capacity");
    let stress = number(&entity.nbt, "Stress");
    let tr = |key: &str, fallback: &str| horizon_client::lang::get_or(key, fallback);
    let mut lines = vec![
        tr("create.gui.goggles.kinetic_stats", "Kinetic Stats:"),
        format!(
            "{}: {:.0} {}",
            tr("create.gui.speedometer.title", "Rotation Speed"),
            rpm.abs(),
            tr("create.generic.unit.rpm", "RPM")
        ),
        format!(
            "{}: {:.0} / {:.0} {}",
            tr("create.gui.stressometer.title", "Network Stress"),
            stress,
            capacity,
            tr("create.generic.unit.stress", "su")
        ),
    ];
    if text(&entity.nbt, "NetworkStatus") == "Overstressed" {
        lines.push(tr("create.gui.stressometer.overstressed", "Overstressed"));
    }
    let x = batch.width / 2.0 + 18.0;
    let y = batch.height / 2.0 - 18.0;
    let width = lines.iter().map(|s| s.chars().count()).max().unwrap_or(20) as f32 * 6.0 + 16.0;
    game.native_fill(
        batch,
        [x, y, width, lines.len() as f32 * 12.0 + 12.0],
        0xE0101729,
    );
    for (i, line) in lines.iter().enumerate() {
        game.native_text(
            batch,
            line,
            [x + 8.0, y + 6.0 + i as f32 * 12.0],
            if i == 0 { 0xFFDA94 } else { 0xD3DBEE },
            true,
        );
    }
    trace(
        "goggles_hud",
        pos,
        serde_json::json!({"rpm":rpm,"capacity":capacity,"stress":stress}),
    );
}
