// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// AirCurrent / FanProcessing adapter. No item components are used as entity data.
use super::{Game, native_block_entity_draw::BlockEntityDraw};
use crate::{containers::ModBlockEntity, items::ItemStack};
use create_engine::{
    fan::{self, Processing, ProcessingType as Type, Segment},
    placement::Direction,
};
use horizon_core::{geometry::Vec3, random::RandomSource};
use horizon_world::nbt::{Compound, Tag};
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Default)]
struct Current {
    facing: Option<Direction>,
    rpm: f32,
    cooldown: i32,
    search: i32,
    distance: f32,
    segments: Vec<Segment>,
    caught: Vec<u32>,
    handlers: Vec<([i32; 3], Type)>,
}
#[derive(Default)]
struct Cache {
    cleanup: BTreeMap<String, i64>,
    currents: BTreeMap<(String, [i32; 3]), Current>,
    recipes: BTreeMap<Type, BTreeMap<String, Vec<(ItemStack, f32)>>>,
    recipes_ready: bool,
}
thread_local! {static CACHE:RefCell<Cache>=RefCell::new(Cache::default());}
pub(super) fn reset(reload: bool) {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if !reload {
            *cache = Cache::default();
        } else {
            cache.recipes_ready = false;
        }
    });
}
pub(super) fn changed(dimension: &str, pos: [i32; 3]) {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache
            .currents
            .retain(|(dim, origin), _| dim != dimension || *origin != pos);
        for ((dim, origin), current) in &mut cache.currents {
            if dim != dimension {
                continue;
            }
            if origin
                .iter()
                .zip(pos)
                .map(|(a, b)| (a - b).abs())
                .sum::<i32>()
                <= 1
            {
                current.cooldown = 0;
            }
        }
    });
}
pub(super) fn maintenance(game: &Game<'_>) {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.cleanup.get(game.dim.id()) == Some(&game.world.game_time) {
            return;
        }
        cache
            .cleanup
            .insert(game.dim.id().into(), game.world.game_time);
        if game.world.game_time.rem_euclid(20) != 0 {
            return;
        }
        cache.currents.retain(|(dim, pos), _| {
            dim != game.dim.id() || game.native_block(*pos) == "create:encased_fan"
        });
    });
}
fn facing(game: &Game<'_>, pos: [i32; 3]) -> Direction {
    let value = game.native_block_property(pos, "facing");
    Direction::ALL
        .into_iter()
        .find(|d| Some(d.name()) == value.as_deref())
        .unwrap_or(Direction::Down)
}
fn add(pos: [i32; 3], d: Direction, n: i32) -> [i32; 3] {
    let v = d.vector();
    std::array::from_fn(|i| pos[i] + v[i] * n)
}
fn block_tag(game: &Game<'_>, pos: [i32; 3], tag: &str) -> bool {
    game.pack
        .assets
        .registry
        .state(game.world.block(pos[0], pos[1], pos[2]))
        .is_some_and(|s| game.native_server_block_in_tag(s, tag).unwrap_or(false))
}
fn media(game: &Game<'_>, pos: [i32; 3]) -> Option<Type> {
    let name = game.native_block(pos);
    // Match priority Splashing > Haunting > Smoking > Blasting.
    let water = name == "minecraft:water"
        || name == "minecraft:bubble_column"
        || game.native_block_property(pos, "waterlogged").as_deref() == Some("true");
    if water || block_tag(game, pos, "create:fan_processing_catalysts/splashing") {
        return Some(Type::Splashing);
    }
    let lit = game.native_block_property(pos, "lit").as_deref() != Some("false");
    if lit && block_tag(game, pos, "create:fan_processing_catalysts/haunting") {
        return Some(Type::Haunting);
    }
    if lit && block_tag(game, pos, "create:fan_processing_catalysts/smoking") {
        return Some(Type::Smoking);
    }
    if name == "minecraft:lava" || block_tag(game, pos, "create:fan_processing_catalysts/blasting")
    {
        return Some(Type::Blasting);
    }
    None
}
fn flow_limit(game: &Game<'_>, pos: [i32; 3], d: Direction, max: f32) -> f32 {
    for i in 0..max.ceil() as i32 {
        let p = add(pos, d, i + 1);
        if !game.world.loaded(p[0], p[2]) {
            return i as f32;
        }
        if block_tag(game, p, "create:fan_transparent") {
            continue;
        }
        let boxes = game.native_block_collision(p);
        if boxes.is_empty() {
            continue;
        }
        if boxes == [[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]] {
            return i as f32;
        }
        let depth = fan::shape_depth(&boxes, d);
        if depth.is_finite() {
            return ((f64::from(i) + depth + 1.0 / 32.0) as f32).min(max);
        }
    }
    max
}
fn bounds(pos: [i32; 3], d: Direction, distance: f32) -> ([f64; 3], [f64; 3]) {
    let mut min = pos.map(f64::from);
    let mut max = min.map(|v| v + 1.0);
    let axis = d.axis().index();
    if d.vector()[axis] > 0 {
        min[axis] += 1.0;
        max[axis] = min[axis] + f64::from(distance);
    } else {
        max[axis] -= 1.0;
        min[axis] = max[axis] - f64::from(distance);
    }
    (min, max)
}
fn refresh(game: &Game<'_>) {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.recipes_ready {
            return;
        }
        cache.recipes.clear();
        for kind in [Type::Splashing, Type::Haunting] {
            let mut recipes = BTreeMap::new();
            for r in game.native_recipes_of_type(kind.id()) {
                let Some(input) = r.json["ingredients"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(super::create_port::ingredient)
                    .and_then(|i| game.native_recipe_ingredient(&i))
                else {
                    continue;
                };
                let Some(results) = r.json["results"].as_array() else {
                    continue;
                };
                let output: Option<Vec<_>> = results
                    .iter()
                    .map(|j| {
                        let mut j = j.clone();
                        if j.get("id").is_none() {
                            j["id"] = j["item"].clone();
                        }
                        Some((
                            game.native_recipe_result(&j)?,
                            j["chance"].as_f64().unwrap_or(1.0) as f32,
                        ))
                    })
                    .collect();
                let Some(output) = output else {
                    continue;
                };
                for item in input.order {
                    recipes.entry(item).or_insert_with(|| output.clone());
                }
            }
            cache.recipes.insert(kind, recipes);
        }
        cache.recipes_ready = true;
    });
}
fn plan(game: &Game<'_>, stack: &ItemStack, kind: Type) -> Option<Vec<(ItemStack, f32)>> {
    match kind {
        Type::Haunting | Type::Splashing => CACHE.with(|cache| {
            cache
                .borrow()
                .recipes
                .get(&kind)?
                .get(stack.item.as_ref())
                .cloned()
        }),
        Type::Smoking => game
            .recipes()
            .cook("minecraft:smoking", stack)
            .filter(|r| !r.id.ends_with("_manual_only"))
            .map(|r| vec![(r.result.clone(), 1.0)]),
        Type::Blasting => {
            let recipe = game
                .recipes()
                .cook("minecraft:smelting", stack)
                .filter(|r| !r.id.ends_with("_manual_only"))
                .or_else(|| {
                    game.recipes()
                        .cook("minecraft:blasting", stack)
                        .filter(|r| !r.id.ends_with("_manual_only"))
                });
            if let Some(recipe) = recipe {
                if game
                    .recipes()
                    .cook("minecraft:smoking", stack)
                    .filter(|r| !r.id.ends_with("_manual_only"))
                    .is_none_or(|smoke| smoke.result.item != recipe.result.item)
                {
                    return Some(vec![(recipe.result.clone(), 1.0)]);
                }
            }
            // Blasting destroys smokeable food and non-resistant items with no furnace recipe.
            if recipe.is_some()
                || !game
                    .items()
                    .get(&stack.item)
                    .is_some_and(|i| i.fire_resistant)
            {
                Some(Vec::new())
            } else {
                None
            }
        }
    }
}
fn roll(
    game: &mut Game<'_>,
    stack: &ItemStack,
    outputs: &[(ItemStack, f32)],
    remainder: bool,
) -> Vec<ItemStack> {
    let mut result = Vec::new();
    for _ in 0..stack.count {
        for (output, chance) in outputs {
            let mut output = output.clone();
            output.count = create_engine::milling::roll_count(output.count, *chance, || {
                game.world.random.next_float()
            });
            if output.count > 0 {
                let max = game.native_max_stack_size(&output.item).unwrap_or(64);
                if let Some(rest) = super::create_port_inventory::insert(&mut result, output, max) {
                    result.push(Some(rest));
                }
            }
        }
        if remainder
            && let Some(item) = game
                .items()
                .get(&stack.item)
                .and_then(|i| i.remainder.clone())
        {
            let output = ItemStack::new(item.as_str(), 1);
            let max = game.native_max_stack_size(&output.item).unwrap_or(64);
            if let Some(rest) = super::create_port_inventory::insert(&mut result, output, max) {
                result.push(Some(rest));
            }
        }
    }
    result.into_iter().flatten().collect()
}
fn held_state(nbt: &Compound) -> Processing {
    let kind = match nbt.get(&"FanProcessingType".into()) {
        Some(Tag::String(s)) => s.to_string().ok().as_deref().and_then(Type::parse),
        _ => None,
    };
    Processing {
        kind,
        time: super::create_port::integer(nbt, "FanProcessingTime"),
    }
}
fn dropped_state(game: &Game<'_>, id: u32) -> Processing {
    let tag = game
        .persist
        .raw
        .get(&id)
        .and_then(|t| t.get("NeoForgeData"))
        .and_then(|t| t.get("CreateData"))
        .and_then(|t| t.get("Processing"));
    let Some(Tag::Compound(tag)) = tag else {
        return Processing::default();
    };
    let kind = match tag.get(&"Type".into()) {
        Some(Tag::String(s)) => s.to_string().ok().as_deref().and_then(Type::parse),
        _ => None,
    };
    Processing {
        kind,
        time: super::create_port::integer(tag, "Time"),
    }
}
fn store_dropped(game: &mut Game<'_>, id: u32, state: Processing) {
    let Some(kind) = state.kind else {
        return;
    };
    // Horizon already preserves unknown entity NBT in persist.raw. Use the
    // original NeoForge entity namespace; item components and stacking stay intact.
    let mut tag = game
        .persist
        .raw
        .entry(id)
        .or_insert_with(|| Tag::Compound(Compound::new()));
    for key in ["NeoForgeData", "CreateData", "Processing"] {
        if !matches!(tag, Tag::Compound(_)) {
            *tag = Tag::Compound(Compound::new());
        }
        let Tag::Compound(compound) = tag else {
            unreachable!()
        };
        tag = compound
            .entry(key.into())
            .or_insert_with(|| Tag::Compound(Compound::new()));
    }
    if !matches!(tag, Tag::Compound(_)) {
        *tag = Tag::Compound(Compound::new());
    }
    let Tag::Compound(tag) = tag else {
        unreachable!()
    };
    tag.insert("Type".into(), Tag::String(kind.id().into()));
    tag.insert("Time".into(), Tag::Int(state.time));
}
fn store_held(nbt: &mut Compound, state: Processing) {
    if let Some(kind) = state.kind {
        nbt.insert("FanProcessingType".into(), Tag::String(kind.id().into()));
        nbt.insert("FanProcessingTime".into(), Tag::Int(state.time));
    }
}
fn process_depot(game: &mut Game<'_>, pos: [i32; 3], kind: Type) {
    let Some((stack, mut state)) = game
        .native_with_mod_block_entity(pos, |e| {
            Some((
                e.items.first()?.clone()?,
                held_state(match e.nbt.get(&"HeldItem".into()) {
                    Some(Tag::Compound(tag)) => tag,
                    _ => return None,
                }),
            ))
        })
        .flatten()
    else {
        return;
    };
    let output = plan(game, &stack, kind);
    let convert = state.transported(kind, stack.count, output.is_some());
    let mut results = if convert {
        output.as_ref().map(|o| {
            roll(
                game,
                &stack,
                o,
                matches!(kind, Type::Splashing | Type::Haunting),
            )
        })
    } else {
        None
    };
    let maxima: Vec<_> = results
        .as_ref()
        .into_iter()
        .flatten()
        .map(|s| game.native_max_stack_size(&s.item).unwrap_or(64))
        .collect();
    let overflow = game
        .native_with_mod_block_entity(pos, |e| {
            if e.items.first() != Some(&Some(stack.clone())) {
                return Vec::new();
            }
            let mut overflow = Vec::new();
            if let Some(ref mut results) = results {
                e.items[0] = None;
                for (i, result) in results.drain(..).enumerate() {
                    if i == 0 {
                        e.items[0] = Some(result);
                    } else if let Some(rest) =
                        super::create_port_inventory::insert(&mut e.items[1..], result, maxima[i])
                    {
                        overflow.push(rest);
                    }
                }
            }
            super::create_port::depot_data(e);
            if let Some(Tag::Compound(tag)) = e.nbt.get_mut(&"HeldItem".into()) {
                store_held(tag, state);
            }
            overflow
        })
        .unwrap_or_default();
    for stack in overflow {
        game.spawn_item(
            stack,
            [
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + 1.0,
                f64::from(pos[2]) + 0.5,
            ],
            [0.0; 3],
            10,
        );
    }
    if convert {
        game.native_block_entity_sync(pos);
    }
}
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], _entity: &mut ModBlockEntity, rpm: f32) {
    refresh(game);
    let dim = game.dim.id().to_owned();
    let key = (dim.clone(), pos);
    let mut current =
        CACHE.with(|cache| cache.borrow_mut().currents.remove(&key).unwrap_or_default());
    let facing = facing(game, pos);
    let push = fan::pushing(rpm, facing);
    let rebuild = current.facing != Some(facing) || current.rpm != rpm || current.cooldown <= 0;
    current.cooldown -= 1;
    if rebuild {
        current.facing = Some(facing);
        current.rpm = rpm;
        current.cooldown = 30;
        current.distance = if rpm == 0.0 {
            0.0
        } else {
            flow_limit(game, pos, facing, fan::max_distance(rpm))
        };
        current.segments =
            fan::segments(current.distance, push, |i| media(game, add(pos, facing, i)));
        current.handlers.clear();
        for i in 1..=current.distance.ceil() as i32 {
            let kind = fan::type_at(current.distance, push, &current.segments, (i - 1) as f32);
            for offset in 0..=i32::from(facing.axis() != create_engine::kinetics::Axis::Y) {
                let mut p = add(pos, facing, i);
                p[1] -= offset;
                if game.native_block(p) == "create:depot"
                    && let Some(kind) = media(game, p).or(kind)
                {
                    current.handlers.push((p, kind));
                }
            }
        }
    }
    if rpm != 0.0 && current.distance >= 0.25 {
        let (min, max) = bounds(pos, facing, current.distance);
        if rebuild || current.search <= 0 {
            current.caught = game
                .native_item_entities(&dim, min, max)
                .iter()
                .map(|e| e.id)
                .collect();
            current.search = 5;
        } else {
            current.search -= 1;
        }
        let flow = if push {
            facing.vector()
        } else {
            facing.opposite().vector()
        };
        let origin = pos.map(|v| f64::from(v) + 0.5);
        let caught = std::mem::take(&mut current.caught);
        for id in &caught {
            let Some(index) = game
                .dropped
                .iter()
                .position(|e| e.id == *id && e.stack.count > 0)
            else {
                continue;
            };
            let e = &mut game.dropped[index];
            let p = [e.position.x, e.position.y, e.position.z];
            if (0..3).any(|i| {
                p[i] + if i == 1 { 0.25 } else { 0.125 } <= min[i]
                    || p[i] - if i == 1 { 0.0 } else { 0.125 } >= max[i]
            }) {
                continue;
            }
            current.caught.push(*id);
            let velocity = fan::accelerated(
                [e.velocity.x, e.velocity.y, e.velocity.z],
                p,
                origin,
                flow,
                rpm,
                current.distance,
                false,
            );
            e.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
            e.needs_sync = true;
            let stack = e.stack.clone();
            let axis = facing.axis().index();
            let offset = if facing.vector()[axis] > 0 {
                p[axis] - f64::from(pos[axis]) - 1.0
            } else {
                f64::from(pos[axis]) - p[axis]
            };
            let Some(kind) = fan::type_at(current.distance, push, &current.segments, offset as f32)
            else {
                continue;
            };
            let output = plan(game, &stack, kind);
            let mut processing = dropped_state(game, *id);
            let convert = processing.dropped(kind, stack.count, output.is_some());
            store_dropped(game, *id, processing);
            if convert && let Some(output) = output {
                let mut results = roll(
                    game,
                    &stack,
                    &output,
                    matches!(kind, Type::Splashing | Type::Haunting),
                )
                .into_iter();
                let first = results
                    .next()
                    .unwrap_or_else(|| ItemStack::new("minecraft:air", 0));
                if game.native_set_item_entity(&dim, *id, first) {
                    for stack in results {
                        game.spawn_item(stack, p, velocity, 0);
                    }
                }
            }
        }
        for (p, kind) in &current.handlers {
            process_depot(game, *p, *kind);
        }
    }
    CACHE.with(|cache| cache.borrow_mut().currents.insert(key, current));
}
pub(super) fn draw(
    game: &Game<'_>,
    pos: [i32; 3],
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    let facing = facing(game, pos);
    let axis = facing.axis();
    let speed = super::create_port::number(&entity.nbt, "Speed");
    let time = draw.game_time as f64 + f64::from(draw.partial);
    for (model, angle) in [
        (
            "create:block/shaft_half",
            create_engine::visual::angle(
                time,
                speed,
                create_engine::visual::rotation_offset(pos, axis, false),
            ),
        ),
        (
            "create:block/encased_fan/propeller",
            create_engine::visual::angle(time, fan::propeller_speed(speed), 0.0),
        ),
    ] {
        draw.push();
        draw.rotate_about(create_engine::visual::positive_axis(axis), angle, [0.5; 3]);
        super::create_port::face_south(draw, facing.opposite());
        game.native_draw_model(draw, model, 0xFFFFFF);
        draw.pop();
    }
}
