// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Game-side adapter. The engine crate contains the portable algorithms.
use super::{Game, native_block_entity_draw::BlockEntityDraw};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModContainer},
    items::ItemStack,
};
use create_engine::{
    kinetics::{self, Axis, Kind, Node},
    placement::{self, Direction},
    pressing::{Action, PressCycle, head_offset, tick_speed},
};
use horizon_world::{
    nbt::{Compound, Tag},
    registry::BlockStateId,
};
use serde_json::Value;

static ORIGINAL_ASSETS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Feed bundled content through Horizon's ordinary content loader, before it
/// registers blocks or announces mods. The native mod uses the same enable flag.
pub(super) fn prepare_content(enabled: &std::collections::BTreeSet<String>) {
    if enabled.contains("create") {
        match create_assets::bundled_content(
            &Game::game_dir(),
            horizon_client::native_data!("create", "content.json"),
        ) {
            Ok(path) => {
                log::info!("Create: bundled content enabled in {}", path.display());
            }
            Err(error) => log::error!("Create: bundled content could not be loaded: {error}"),
        }
    }
}

/// Imports the user's JAR before the initial load, and again on pack reload.
/// Uses Horizon's own pack loader/cache; the installed content archive is untouched.
pub(super) fn asset_root(root: std::path::PathBuf) -> std::path::PathBuf {
    ORIGINAL_ASSETS.store(false, std::sync::atomic::Ordering::Relaxed);
    // Horizon consumes registry declarations while building the root; their
    // source data/*/horizon files are deliberately absent from that overlay.
    if !root
        .join("data/create/recipe/pressing/iron_ingot.json")
        .is_file()
        || !root.join("assets/create/items/shaft.json").is_file()
    {
        return root;
    }
    let home = Game::game_dir();
    let result = (|| -> std::io::Result<Option<std::path::PathBuf>> {
        let Some(imported) = create_assets::prepare(&home, horizon_client::packs::RESOURCE_FORMAT)?
        else {
            log::warn!(
                "Create: using provisional resources. Place the original Create {} JAR in {}",
                create_assets::CREATE_VERSION,
                home.join(create_assets::INPUT_DIR).display()
            );
            return Ok(None);
        };
        let packs = horizon_client::packs::folder(
            imported.pack.parent().expect("pack parent"),
            horizon_client::packs::RESOURCE_FORMAT,
        );
        if packs.len() != 1 {
            return Err(std::io::Error::other(
                "The locally imported Create resource pack could not be read",
            ));
        }
        let overlay = horizon_client::packs::resource_root(
            &root,
            &packs,
            &home.join(".horizon-cache/create-assets/overlay"),
        )?;
        log::info!(
            "Create: {} local resources from Create {} ({}); JAR and installed mod unchanged",
            imported.files,
            create_assets::CREATE_VERSION,
            if imported.cached {
                "cache verified"
            } else {
                "imported"
            }
        );
        Ok(Some(overlay))
    })();
    match result {
        Ok(Some(overlay)) => {
            ORIGINAL_ASSETS.store(true, std::sync::atomic::Ordering::Relaxed);
            overlay
        }
        Ok(None) => root,
        Err(error) => {
            log::warn!(
                "Create: local resource import failed; using provisional resources: {error}"
            );
            root
        }
    }
}

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
        let id = match block {
            "create:creative_motor" => "create:motor",
            "create:shaft" | "create:cogwheel" | "create:large_cogwheel" => "create:simple_kinetic",
            _ => block,
        };
        let mut nbt = Compound::new();
        if block == "create:creative_motor" {
            nbt.insert("ScrollValue".into(), Tag::Int(16));
        }
        *entity = Some((id.to_owned(), nbt));
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
        Some(Tag::Byte(n)) => i32::from(*n),
        _ => 0,
    }
}

thread_local! {
    static PLACEMENT_HIT: std::cell::Cell<Option<([i32; 3], [i32; 3])>> = const { std::cell::Cell::new(None) };
}

pub(super) fn use_on(game: &Game<'_>, stack: &ItemStack, pos: [i32; 3], face: [i32; 3]) {
    PLACEMENT_HIT.set((game.native_on("create") && owns(&stack.item)).then_some((pos, face)));
}

// Direction.orderedByNearest, adapted from Horizon's MIT level/place.rs.
// Uses the engine's vanilla-compatible trigonometric table.
fn nearest(yaw: f32, pitch: f32) -> Direction {
    use horizon_core::geometry::trig;
    let pitch = pitch * (std::f32::consts::PI / 180.0);
    let yaw = -yaw * (std::f32::consts::PI / 180.0);
    let (ps, pc) = (trig::sin(f64::from(pitch)), trig::cos(f64::from(pitch)));
    let (ys, yc) = (trig::sin(f64::from(yaw)), trig::cos(f64::from(yaw)));
    let (x, y, z) = (ys.abs(), ps.abs(), yc.abs());
    let ax = if ys > 0.0 {
        Direction::East
    } else {
        Direction::West
    };
    let ay = if ps < 0.0 {
        Direction::Up
    } else {
        Direction::Down
    };
    let az = if yc > 0.0 {
        Direction::South
    } else {
        Direction::North
    };
    if x > z {
        if y > x * pc { ay } else { ax }
    } else if y > z * pc {
        ay
    } else {
        az
    }
}

pub(super) fn placed(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3]) {
    let Some((clicked, face)) = PLACEMENT_HIT.take() else {
        return;
    };
    if !owns(&stack.item) {
        return;
    }
    let Some(face) = Direction::from_vector(face) else {
        return;
    };
    let ports = Direction::ALL.map(|d| {
        let offset = d.vector();
        node(game, std::array::from_fn(|i| pos[i] + offset[i]))
            .is_some_and(|n| n.shaft_port(offset.map(|v| -v)))
    });
    let sneaking = game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft);
    let look = nearest(game.yaw, game.pitch);
    let axis = match stack.item.as_ref() {
        "create:shaft" => Some(placement::shaft(look, face, sneaking, ports)),
        "create:cogwheel" | "create:large_cogwheel" => {
            let against = if game.native_block(clicked) == "create:cogwheel" {
                node(game, clicked).map(|n| n.axis)
            } else {
                None
            };
            Some(placement::cog(face, sneaking, ports, against))
        }
        _ => None,
    };
    if let Some(axis) = axis {
        game.native_set_block_property(
            pos,
            "axis",
            match axis {
                Axis::X => "x",
                Axis::Y => "y",
                Axis::Z => "z",
            },
        );
    } else {
        let facing = match stack.item.as_ref() {
            "create:creative_motor" => Some(placement::motor(look, sneaking, ports)),
            "create:mechanical_press" => Some(
                [
                    Direction::North,
                    Direction::East,
                    Direction::South,
                    Direction::West,
                ][((f64::from(game.yaw) / 90.0 + 0.5).floor() as i64 & 3) as usize],
            ),
            _ => None,
        };
        if let Some(facing) = facing {
            game.native_set_block_property(pos, "facing", facing.name());
        }
    }
}

pub(super) fn depot_reacts(game: &Game<'_>, pos: [i32; 3], face: [i32; 3]) -> bool {
    if !game.native_on("create") || game.native_block(pos) != "create:depot" || face != [0, 1, 0] {
        return false;
    }
    if game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
        && (game
            .inventory
            .get(game.selected)
            .is_some_and(Option::is_some)
            || game.offhand.is_some())
    {
        return false;
    }
    true
}

pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    if !depot_reacts(game, pos, face) {
        return;
    }
    let held = game.held();
    let put = held.is_some_and(|s| s.count > 0 && s.item.as_ref() != "create:mechanical_arm");
    let Some(taken) = game.native_with_mod_block_entity(pos, |e| {
        let taken = e
            .items
            .iter_mut()
            .filter_map(Option::take)
            .collect::<Vec<_>>();
        e.nbt.remove(&"HeldItem".into());
        taken
    }) else {
        return;
    };
    let uuid = game.native_acting_id();
    for stack in taken {
        if let Some(rest) = game.native_give_stack(uuid, stack)
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
    // Read the hand after returning the old stack: Inventory.add may have
    // merged equal items into it. Creative players also empty their hand.
    if put {
        let incoming = game.held_mut().take();
        game.native_with_mod_block_entity(pos, |e| {
            e.items[0] = incoming;
        });
    }
    game.native_with_mod_block_entity(pos, |e| {
        depot_data(e);
    });
    game.native_block_entity_sync(pos);
    game.swing_arm();
    *canceled = true;
}

/// Match DepotBehaviour/TransportedItemStack's public data names. Horizon also
/// saves its generic Items slots; Create ignores that additional container field.
fn depot_data(entity: &mut ModBlockEntity) -> bool {
    let old = entity.nbt.clone();
    if let Some(item) = entity
        .items
        .first()
        .and_then(Option::as_ref)
        .map(horizon_items::item_nbt::item_to_nbt)
    {
        let mut held = match entity.nbt.remove(&"HeldItem".into()) {
            Some(Tag::Compound(tag)) => tag,
            _ => Compound::from([
                ("Pos".into(), Tag::Float(0.25f32.to_bits())),
                ("PrevPos".into(), Tag::Float(0.25f32.to_bits())),
                ("Offset".into(), Tag::Float(0.0f32.to_bits())),
                ("PrevOffset".into(), Tag::Float(0.0f32.to_bits())),
                ("InSegment".into(), Tag::Int(0)),
                ("Angle".into(), Tag::Int(0)),
                ("InDirection".into(), Tag::Int(3)),
            ]),
        };
        held.insert("Item".into(), item);
        entity.nbt.insert("HeldItem".into(), Tag::Compound(held));
    } else {
        entity.nbt.remove(&"HeldItem".into());
    }
    let buffer = entity
        .items
        .iter()
        .skip(1)
        .enumerate()
        .filter_map(|(slot, item)| {
            let Tag::Compound(mut tag) = horizon_items::item_nbt::item_to_nbt(item.as_ref()?)
            else {
                return None;
            };
            tag.insert("Slot".into(), Tag::Int(slot as i32));
            Some(Tag::Compound(tag))
        })
        .collect();
    entity.nbt.insert(
        "OutputBuffer".into(),
        Tag::Compound(Compound::from([
            ("Size".into(), Tag::Int(8)),
            ("Items".into(), Tag::List(buffer)),
        ])),
    );
    entity.nbt.remove(&"DisplayItem".into());
    entity.nbt.remove(&"DisplayCount".into());
    old != entity.nbt
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
            rpm: {
                let speed = match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
                    Some(BlockEntity::Mod(e)) => e
                        .nbt
                        .get(&"ScrollValue".into())
                        .map(|_| integer(&e.nbt, "ScrollValue") as f32)
                        .unwrap_or(16.0),
                    _ => 16.0,
                };
                if direction.iter().any(|v| *v < 0) {
                    -speed
                } else {
                    speed
                }
            },
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
    let block = game.native_block(pos);
    if !owns(&block) {
        return;
    }
    if block == "create:depot" {
        if depot_data(entity) {
            game.native_block_entity_sync(pos);
        }
        return;
    }
    let Some(mut start) = node(game, pos) else {
        return;
    };
    if let Kind::Motor { rpm, facing, .. } = &mut start.kind {
        let value = entity
            .nbt
            .get(&"ScrollValue".into())
            .map(|_| integer(&entity.nbt, "ScrollValue") as f32)
            .unwrap_or(16.0);
        *rpm = if facing.iter().any(|v| *v < 0) {
            -value
        } else {
            value
        };
    }
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
    if block == "create:mechanical_press" {
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
            .insert("Running".into(), Tag::Byte(i8::from(cycle.running)));
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
    if (owns(&entity.id) || matches!(entity.id.as_str(), "create:motor" | "create:simple_kinetic"))
        && tag.is_none()
    {
        *tag = Some(entity.nbt.clone());
    }
}

pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if block == "create:depot" && container.is_none() {
        *container = Some(ModContainer {
            slots: 9,
            faces: std::array::from_fn(|_| (0..9).collect()),
            inputs: vec![0],
            outputs: (0..9).collect(),
            menu: None,
        });
    }
}

pub(super) fn models(models: &mut Vec<String>) {
    models.extend(
        [
            "shaft",
            "shaft_half",
            "cogwheel",
            "large_cogwheel",
            "creative_motor/block",
            "creative_motor/block_vertical",
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
    let original = ORIGINAL_ASSETS.load(std::sync::atomic::Ordering::Relaxed);
    if block == "create:depot" {
        // The depot's exposed top and displayed item are lit from the air above it.
        let (block_light, sky_light) = game.native_light([pos[0], pos[1] + 1, pos[2]]);
        draw.light =
            horizon_client::block_entity_render::vertex_light((sky_light << 4) | block_light);
        game.native_draw_model(draw, "create:block/depot/block", 0xFFFFFF);
        if let Some(held) = entity.nbt.get(&"HeldItem".into()) {
            if let Some(stack) = held
                .get("Item")
                .and_then(horizon_items::item_nbt::item_from_nbt)
            {
                draw.push();
                draw.translate([0.5, 13.0 / 16.0, 0.5]);
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
    let direction = game.native_block_property(pos, "facing");
    if matches!(node.kind, Kind::Motor { .. } | Kind::Press) {
        draw.push();
        let model = if node.kind == Kind::Press {
            let yaw = match direction.as_deref() {
                Some("east") => -90.0,
                Some("south") => 180.0,
                Some("west") => 90.0,
                _ => 0.0,
            };
            draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.5; 3]);
            "create:block/mechanical_press/block"
        } else if matches!(direction.as_deref(), Some("up" | "down")) {
            if direction.as_deref() == Some("down") {
                draw.rotate_about([1.0, 0.0, 0.0], 180.0, [0.5; 3]);
            }
            "create:block/creative_motor/block_vertical"
        } else {
            let yaw = match direction.as_deref() {
                Some("east") => 90.0,
                Some("north") => 180.0,
                Some("west") => -90.0,
                _ => 0.0,
            };
            draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.5; 3]);
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
            let travel = if original {
                // Create's belt/depot head reaches the depot's 13/16 top two blocks below.
                head_offset(ticks) * 19.0 / 16.0
            } else if ticks <= 120.0 {
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
    let angle = ((draw.game_time as f64 + f64::from(draw.partial))
        * f64::from(number(&entity.nbt, "Speed"))
        * 0.3)
        .rem_euclid(360.0) as f32;
    if matches!(node.kind, Kind::Motor { .. }) {
        // The half shaft's untransformed output points south (+Z).
        match direction.as_deref() {
            Some("up") => draw.rotate_about([1.0, 0.0, 0.0], -90.0, [0.5; 3]),
            Some("down") => draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.5; 3]),
            Some("east") => draw.rotate_about([0.0, 1.0, 0.0], 90.0, [0.5; 3]),
            Some("west") => draw.rotate_about([0.0, 1.0, 0.0], -90.0, [0.5; 3]),
            Some("north") => draw.rotate_about([0.0, 1.0, 0.0], 180.0, [0.5; 3]),
            _ => {}
        }
        draw.rotate_about([0.0, 0.0, 1.0], angle, [0.5; 3]);
        game.native_draw_model(draw, "create:block/shaft_half", 0xFFFFFF);
        draw.pop();
        return;
    }
    orient(draw, node.axis);
    draw.rotate_about([0.0, 1.0, 0.0], angle, [0.5; 3]);
    let model = match node.kind {
        Kind::Cog { large: false } => "create:block/cogwheel",
        Kind::Cog { large: true } => "create:block/large_cogwheel",
        _ => "create:block/shaft",
    };
    game.native_draw_model(draw, model, 0xFFFFFF);
    draw.pop();
}
