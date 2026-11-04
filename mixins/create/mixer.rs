// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// MechanicalMixerBlockEntity + BasinOperatingBlockEntity, Create 6.0.10.
// Source heat-aware item/water/lava mixing and ordinary shapeless crafting. processingTicks and
// currentRecipe deliberately remain transient, exactly as the source serializer.
use super::{
    Game, create_port_basin as basin, create_port_fluid_codec as fluids,
    native_block_entity_draw::BlockEntityDraw,
};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModRevision},
    items::ItemStack,
};
use create_engine::{basin as rules, kinetics::Axis, transport::SessionCache};
use horizon_client::crafting::{Ingredient, Recipe as Crafting};
use horizon_client_render::sound_engine::{SoundEngine, SoundInstance, Sounds};
use horizon_core::random::RandomSource;
use horizon_world::nbt::Tag;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

const ID: &str = "create:mechanical_mixer";
type Pos = [i32; 3];

#[derive(Clone)]
enum Input {
    Item(Ingredient),
    BlockTag(String),
}
#[derive(Clone)]
struct Recipe {
    id: String,
    inputs: Vec<Input>,
    fluids: Vec<String>,
    core: rules::Recipe<basin::Key, fluids::Key>,
}
#[derive(Default)]
struct Recipes {
    generation: u64,
    source: Vec<Arc<serde_json::Value>>,
    vanilla: usize,
    compiled: Vec<Arc<Recipe>>,
}
struct Running {
    owner: Option<ModRevision>,
    cycle: rules::Mixer,
    recipe: Option<Arc<Recipe>>,
    pending: bool,
    rpm: f32,
    tick: Option<i64>,
    contents: Vec<Option<ItemStack>>,
    fluids: Vec<Option<rules::Fluid<fluids::Key>>>,
    removed: bool,
}
impl Default for Running {
    fn default() -> Self {
        Self {
            owner: None,
            cycle: rules::Mixer::default(),
            recipe: None,
            pending: true,
            rpm: 0.0,
            tick: None,
            contents: Vec::new(),
            fluids: Vec::new(),
            removed: false,
        }
    }
}
#[derive(Default)]
struct Client {
    owner: Option<ModRevision>,
    tick: Option<i64>,
    received: Option<(bool, i32)>,
    cycle: rules::Mixer,
}
#[derive(Default)]
struct World {
    recipes: Recipes,
    running: BTreeMap<Pos, Running>,
    client: BTreeMap<Pos, Client>,
    maintenance: Option<i64>,
}
thread_local! {
    static WORLDS: RefCell<SessionCache<World>> = RefCell::new(SessionCache::default());
    // The host replaces WorldTables in place on reload. A Recipes address
    // cannot identify that new table, especially when no custom recipes exist.
    static RECIPE_GENERATION: Cell<u64> = const { Cell::new(0) };
}
fn with_world<R>(game: &Game<'_>, action: impl FnOnce(&mut World) -> R) -> R {
    WORLDS.with(|cache| {
        action(
            cache
                .borrow_mut()
                .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id()),
        )
    })
}
pub(super) fn reset(reload: bool) {
    // A data-pack reload keeps the live source BE and its current recipe.
    // Chunk/world reload re-creates it, losing processingTicks/currentRecipe.
    if reload {
        RECIPE_GENERATION.with(|generation| generation.set(generation.get().wrapping_add(1)));
    } else {
        WORLDS.with(|cache| cache.borrow_mut().clear());
    }
}
pub(super) fn maintenance(game: &mut Game<'_>) {
    with_world(game, |world| {
        if world.maintenance == Some(game.world.game_time) {
            return;
        }
        world.maintenance = Some(game.world.game_time);
        let loaded = |p: &Pos| game.world.loaded(p[0], p[2]) && game.native_block(*p) == ID;
        world.running.retain(|p, _| loaded(p));
        world.client.retain(|p, _| loaded(p));
    });
}
pub(super) fn initialize(entity: &mut ModBlockEntity) {
    if entity.id != ID {
        return;
    }
    entity.nbt.entry("Running".into()).or_insert(Tag::Byte(0));
    entity.nbt.entry("Ticks".into()).or_insert(Tag::Int(0));
    entity
        .nbt
        .entry("NeedsUpdate".into())
        .or_insert(Tag::Byte(0));
}
pub(super) fn schedule(game: &mut Game<'_>, pos: Pos) {
    if game.native_block(pos) != ID {
        return;
    }
    with_world(game, |w| w.running.entry(pos).or_default().pending = true);
    game.native_with_mod_block_entity(pos, |e| {
        if e.id == ID {
            e.nbt.insert("NeedsUpdate".into(), Tag::Byte(1));
        }
    });
}
pub(super) fn basin_removed(game: &mut Game<'_>, pos: Pos) {
    if game.native_block(pos) == ID {
        with_world(game, |w| w.running.entry(pos).or_default().removed = true);
    }
}
fn saved(entity: &ModBlockEntity) -> (bool, i32) {
    (
        super::create_port::integer(&entity.nbt, "Running") != 0,
        super::create_port::integer(&entity.nbt, "Ticks"),
    )
}
fn compile(game: &Game<'_>, id: &str, json: &serde_json::Value) -> Option<Recipe> {
    // Strict bounded codecs: no unknown ingredient fields can be dropped to
    // turn an unsupported predicate into a simpler item/fluid match.
    let heat = match json.get("heat_requirement") {
        None => rules::HeatRequirement::None,
        Some(v) => match v.as_str()? {
            "none" => rules::HeatRequirement::None,
            "heated" => rules::HeatRequirement::Heated,
            "superheated" => rules::HeatRequirement::Superheated,
            _ => return None,
        },
    };
    if json["results"]
        .as_array()?
        .iter()
        .any(|i| i.get("fluid").is_some() || i.get("amount").is_some())
    {
        return None;
    }
    let mut inputs = Vec::new();
    let mut fluids = Vec::new();
    let mut fluid_amounts = Vec::new();
    for i in json["ingredients"].as_array()? {
        match i.get("type").and_then(serde_json::Value::as_str) {
            Some("neoforge:single") => {
                if i.as_object()?.len() != 3 {
                    return None;
                }
                let fluid = i["fluid"].as_str()?;
                let amount = i["amount"].as_u64()?;
                if !matches!(fluid, "minecraft:water" | "minecraft:lava")
                    || !(1..=i32::MAX as u64).contains(&amount)
                {
                    return None;
                }
                fluids.push(fluid.to_owned());
                fluid_amounts.push(amount as u32);
            }
            Some("neoforge:block_tag") => {
                if i.as_object()?.len() != 2 {
                    return None;
                }
                inputs.push(Input::BlockTag(i["tag"].as_str()?.to_owned()));
            }
            Some(_) => return None,
            None => inputs.push(Input::Item(
                game.native_recipe_ingredient(&plain_ingredient(i)?)?,
            )),
        }
    }
    let outputs: Option<Vec<_>> = json["results"]
        .as_array()?
        .iter()
        .map(|r| {
            let fields = r.as_object()?;
            if fields.keys().any(|k| {
                !matches!(
                    k.as_str(),
                    "id" | "item" | "count" | "chance" | "components"
                )
            }) || (fields.contains_key("id") && fields.contains_key("item"))
                || r.get("count")
                    .is_some_and(|v| v.as_u64().is_none_or(|v| !(1..=99).contains(&v)))
                || r.get("chance").is_some_and(|v| v.as_f64().is_none())
                || r.get("components").is_some_and(|v| !v.is_object())
            {
                return None;
            }
            let mut r = r.clone();
            if r.get("id").is_none() {
                r["id"] = r["item"].clone();
            }
            let stack = game.native_recipe_result(&r)?;
            let chance = r["chance"].as_f64().unwrap_or(1.0) as f32;
            (chance.is_finite() && (0.0..=1.0).contains(&chance)).then(|| rules::Roll {
                stack: basin::core_stack(game, &stack),
                chance,
            })
        })
        .collect();
    let outputs = outputs?;
    if inputs.len() + fluids.len() == 0
        || inputs.len() > 64
        || fluids.len() > 2
        || outputs.is_empty()
        || outputs.len() > 4
    {
        return None;
    }
    Some(Recipe {
        id: id.to_owned(),
        core: rules::Recipe {
            kind: rules::RecipeKind::Basin,
            item_ingredients: inputs.len(),
            fluid_amounts,
            outputs,
            fluid_outputs: Vec::new(),
            heat,
            duration: match json
                .get("processing_time")
                .or_else(|| json.get("processingTime"))
            {
                None => 0,
                Some(v) => i32::try_from(v.as_i64()?).ok()?,
            },
        },
        inputs,
        fluids,
    })
}
fn plain_ingredient(i: &serde_json::Value) -> Option<serde_json::Value> {
    if let Some(id) = i.as_str() {
        return id.contains(':').then(|| i.clone());
    }
    if let Some(children) = i.as_array() {
        if children.is_empty() {
            return None;
        }
        return children
            .iter()
            .map(plain_ingredient)
            .collect::<Option<Vec<_>>>()
            .map(serde_json::Value::Array);
    }
    if i.as_object()?.len() != 1 {
        return None;
    }
    super::create_port::ingredient(i)
}
fn recipes(game: &Game<'_>) -> Vec<Arc<Recipe>> {
    with_world(game, |world| {
        let custom = game.native_recipes_of_type("create:mixing");
        let table = game.recipes();
        let vanilla = std::ptr::from_ref(table) as usize;
        let cache = &mut world.recipes;
        let generation = RECIPE_GENERATION.with(Cell::get);
        if cache.generation == generation
            && cache.vanilla == vanilla
            && cache.source.len() == custom.len()
            && custom
                .iter()
                .zip(&cache.source)
                .all(|(r, old)| Arc::ptr_eq(&r.json, old))
        {
            return cache.compiled.clone();
        }
        let mut compiled: Vec<_> = custom
            .iter()
            .filter_map(|r| compile(game, &r.id, &r.json))
            .map(Arc::new)
            .collect();
        for (id, recipe) in &table.crafting {
            let Crafting::Shapeless {
                ingredients,
                result,
            } = recipe
            else {
                continue;
            };
            if ingredients.len() <= 1 || id.ends_with("_manual_only") {
                continue;
            }
            // MechanicalPress.canCompress + ItemHelper.matchAllIngredients:
            // compare the displayed ingredient item ORDER, not just a set.
            if matches!(ingredients.len(), 4 | 9)
                && ingredients.iter().all(|i| i.order == ingredients[0].order)
            {
                continue;
            }
            let output = result.create();
            if output.count == 0 || output.item.as_ref() == "minecraft:air" {
                continue;
            }
            compiled.push(Arc::new(Recipe {
                id: horizon_client::crafting::recipe_id(id),
                inputs: ingredients.iter().cloned().map(Input::Item).collect(),
                fluids: Vec::new(),
                core: rules::Recipe {
                    kind: rules::RecipeKind::Crafting,
                    item_ingredients: ingredients.len(),
                    fluid_amounts: Vec::new(),
                    outputs: vec![rules::Roll {
                        stack: basin::core_stack(game, &output),
                        chance: 1.0,
                    }],
                    fluid_outputs: Vec::new(),
                    heat: rules::HeatRequirement::None,
                    duration: 0,
                },
            }));
        }
        cache.generation = generation;
        cache.vanilla = vanilla;
        cache.source = custom.iter().map(|r| r.json.clone()).collect();
        cache.compiled = compiled;
        cache.compiled.clone()
    })
}
struct Context<'a, 'b> {
    game: &'a mut Game<'b>,
    inputs: &'a [Input],
    fluids: &'a [String],
}
impl rules::Context<basin::Key, fluids::Key> for Context<'_, '_> {
    fn item_matches(&mut self, ingredient: usize, item: &rules::Item<basin::Key>) -> bool {
        match &self.inputs[ingredient] {
            Input::Item(input) => input.test(&item.key.0),
            Input::BlockTag(tag) => self
                .game
                .items()
                .get(&item.key.0.item)
                .and_then(|info| info.block)
                .and_then(|state| self.game.pack.assets.registry.state(state))
                .is_some_and(|state| {
                    self.game
                        .native_server_block_in_tag(state, tag)
                        .unwrap_or(false)
                }),
        }
    }
    fn fluid_matches(&mut self, ingredient: usize, fluid: &rules::Fluid<fluids::Key>) -> bool {
        // NeoForge single fluid ingredients match the fluid holder, ignoring
        // the stack component patch; consumed storage still keeps that patch.
        self.fluids[ingredient] == fluid.key.id
    }
    fn remainder(&mut self, unit: &rules::Item<basin::Key>) -> Option<rules::Item<basin::Key>> {
        let item = self
            .game
            .items()
            .get(&unit.key.0.item)?
            .crafting_remainder
            .clone()?;
        Some(basin::core_stack(self.game, &ItemStack::new(&item, 1)))
    }
    fn random(&mut self) -> f32 {
        self.game.world.random.next_float()
    }
}
fn matching(game: &mut Game<'_>, pos: Pos, basin: &mut basin::Basin, recipe: &Recipe) -> bool {
    if recipe.core.kind == rules::RecipeKind::Basin {
        basin.heat = basin::heat(game, pos);
    }
    basin
        .match_recipe(
            &recipe.core,
            true,
            true,
            &mut Context {
                game,
                inputs: &recipe.inputs,
                fluids: &recipe.fluids,
            },
        )
        .success()
}
fn choose(game: &mut Game<'_>, pos: Pos, basin: &mut basin::Basin) -> Option<Arc<Recipe>> {
    if (basin.input.iter().chain(&basin.output).all(Option::is_none)
        && basin
            .input_fluids
            .iter()
            .chain(&basin.output_fluids)
            .all(Option::is_none))
        || !basin.can_continue_processing()
    {
        return None;
    }
    // Source tests EVERY candidate before sorting: even match simulations roll
    // probabilistic outputs. Do not short-circuit at the first successful one.
    let variants: std::collections::BTreeSet<_> = basin
        .input
        .iter()
        .chain(&basin.output)
        .flatten()
        .filter(|s| s.count > 0)
        .map(|s| s.key.0.item.to_string())
        .collect();
    let mut found: Vec<_> = recipes(game)
        .into_iter()
        // RecipeTrie indexes simple ingredients by item variant, without
        // counts/components. Its universal key admits non-simple predicates.
        .filter(|r| {
            r.inputs.iter().all(|i| match i {
                Input::BlockTag(_) => true,
                Input::Item(i) => {
                    i.custom.is_some()
                        || i.children.is_some()
                        || i.order.iter().any(|item| variants.contains(item.as_str()))
                }
            })
        })
        .filter(|r| matching(game, pos, basin, r))
        .collect();
    found.sort_by_key(|r| std::cmp::Reverse(r.inputs.len()));
    found.into_iter().next()
}
fn apply(game: &mut Game<'_>, pos: Pos, recipe: Option<&Recipe>) -> bool {
    let Some(recipe) = recipe else {
        return false;
    };
    let Some(mut state) = basin::read(game, pos) else {
        return false;
    };
    if recipe.core.kind == rules::RecipeKind::Basin {
        state.heat = basin::heat(game, pos);
    }
    let was_empty = state.can_continue_processing();
    let report = state.apply(
        &recipe.core,
        false,
        &mut Context {
            game,
            inputs: &recipe.inputs,
            fluids: &recipe.fluids,
        },
    );
    // Preserve known source partial-consumption/output-overbooking failures.
    if report.changed() {
        basin::write(game, pos, &state);
    }
    if !report.success() {
        return false;
    }
    was_empty && matching(game, pos, &mut state, recipe)
}
/// Runs once after the kinetic graph has resolved this mixer's actual RPM.
pub(super) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity, rpm: f32) -> bool {
    if entity.id != ID {
        return false;
    }
    initialize(entity);
    let now = game.world.game_time;
    let mut state = with_world(game, |w| w.running.remove(&pos).unwrap_or_default());
    let owner = entity.revision();
    let received = saved(entity);
    // Source transient fields belong to the BE object, not its coordinate or
    // saved counters. Ordinary load/update writes retain that object's timer.
    if state.tick.is_some_and(|t| t > now) || state.owner.is_some_and(|old| !old.same_entity(owner))
    {
        state = Running::default();
    }
    state.owner = Some(owner);
    if state.tick == Some(now) {
        with_world(game, |w| {
            w.running.insert(pos, state);
        });
        return false;
    }
    state.cycle.running = received.0;
    state.cycle.ticks = received.1;
    state.pending |=
        super::create_port::integer(&entity.nbt, "NeedsUpdate") != 0 || state.rpm != rpm;
    state.rpm = rpm;
    let basin_pos = [pos[0], pos[1] - 2, pos[2]];
    let mut basin = basin::read(game, basin_pos);
    let contents: Vec<_> = basin
        .as_ref()
        .map(|b| {
            b.input
                .iter()
                .chain(&b.output)
                .map(|s| s.as_ref().map(basin::native_stack))
                .collect()
        })
        .unwrap_or_default();
    state.pending |= state.contents != contents;
    state.contents = contents;
    let fluids = basin
        .as_ref()
        .map(|b| {
            b.input_fluids
                .iter()
                .chain(&b.output_fluids)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    state.pending |= state.fluids != fluids;
    state.fluids = fluids;
    let exists = matches!(game.world.block_entities.get(&(basin_pos[0],basin_pos[1],basin_pos[2])),Some(BlockEntity::Mod(e)) if e.id == "create:basin");
    let mut sync = false;
    if !exists && state.cycle.running {
        state.cycle.basin_removed();
        sync = true;
    }
    if state.removed {
        state.cycle.basin_removed();
        state.removed = false;
        sync = true;
    }
    if state.pending {
        // DeferralBehaviour runs before MechanicalMixerBlockEntity.tick.
        state.pending = false;
        if !state.cycle.running
            && rpm != 0.0
            && rpm.abs() >= rules::DEFAULT_MIXER_MINIMUM_RPM
            && let Some(basin) = basin.as_mut()
            && let Some(recipe) = choose(game, basin_pos, basin)
        {
            state.recipe = Some(recipe);
            sync |= state.cycle.start();
        }
    }
    let recipe = state.recipe.clone();
    let events = state.cycle.tick(
        rpm,
        rules::DEFAULT_MIXER_MINIMUM_RPM,
        recipe.as_ref().map_or(0, |r| r.core.duration),
        || apply(game, basin_pos, recipe.as_deref()),
    );
    if events.initialized_processing
        && basin.as_ref().is_some_and(|basin| {
            basin
                .input_fluids
                .iter()
                .chain(&basin.output_fluids)
                .flatten()
                .any(|fluid| fluid.amount > 0)
        })
    {
        // MechanicalMixerBlockEntity.tick: one authoritative sound when the
        // processing timer is initialized, in addition to the client mixing loop.
        game.with_level(|_, level, _| {
            level.native_play_sound(
                "minecraft:block.bubble_column.whirlpool_ambient",
                pos.map(|v| f64::from(v) + 0.5),
                4,
                0.75,
                if rpm.abs() < 65.0 { 0.75 } else { 1.5 },
            );
        });
    }
    if events.apply_called {
        sync = true;
        state.pending = true;
        if std::env::var_os("CREATE_HORIZON_TRACE").is_some() {
            log::info!(
                "Create mixing: {}",
                serde_json::json!({"position":pos,"recipe":recipe.as_ref().map(|r|&r.id),"running_ticks":state.cycle.ticks})
            );
        }
    }
    state.pending |= events.schedule_basin_check;
    entity
        .nbt
        .insert("Running".into(), Tag::Byte(i8::from(state.cycle.running)));
    entity
        .nbt
        .insert("Ticks".into(), Tag::Int(state.cycle.ticks));
    entity
        .nbt
        .insert("NeedsUpdate".into(), Tag::Byte(i8::from(state.pending)));
    state.tick = Some(now);
    with_world(game, |w| {
        w.running.insert(pos, state);
    });
    sync
}

// Minecraft1.21.1 Mth: float indexing is significant at lookup boundaries.
fn sine() -> &'static [f32; 65536] {
    static TABLE: OnceLock<[f32; 65536]> = OnceLock::new();
    TABLE.get_or_init(|| {
        std::array::from_fn(|i| (i as f64 * std::f64::consts::PI * 2.0 / 65536.0).sin() as f32)
    })
}
pub(super) fn cos(value: f32) -> f32 {
    sine()[((value * 10430.378 + 16384.0) as i32 & 65535) as usize]
}
pub(super) fn sin(value: f32) -> f32 {
    sine()[((value * 10430.378) as i32 & 65535) as usize]
}
pub(super) fn rotate_y(vector: [f64; 3], degrees: f32) -> [f64; 3] {
    if degrees == 0.0 {
        return vector;
    }
    let radians = (f64::from(degrees) / 180.0 * std::f64::consts::PI) as f32;
    let sin = f64::from(sine()[((radians * 10430.378) as i32 & 65535) as usize]);
    let cos = f64::from(cos(radians));
    let [x, y, z] = vector;
    [x * cos + z * sin, y, z * cos - x * sin]
}
pub(super) fn models(models: &mut Vec<String>) {
    models.extend(
        [
            "create:block/cogwheel_shaftless",
            "create:block/mechanical_mixer/pole",
            "create:block/mechanical_mixer/head",
        ]
        .map(str::to_owned),
    );
}
pub(super) fn bounds(entity: &ModBlockEntity, bounds: &mut [f32; 6]) {
    if entity.id == ID {
        *bounds = [0.0, -1.5, 0.0, 1.0, 1.0, 1.0];
    }
}
pub(super) fn draw(
    game: &Game<'_>,
    pos: Pos,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) -> bool {
    if entity.id != ID {
        return false;
    }
    let cycle = with_world(game, |w| {
        w.client
            .get(&pos)
            .filter(|c| {
                c.owner
                    .is_some_and(|old| old.same_entity(entity.revision()))
            })
            .map_or_else(
                || {
                    let (running, ticks) = saved(entity);
                    rules::Mixer {
                        running,
                        ticks,
                        ..rules::Mixer::default()
                    }
                },
                |c| c.cycle,
            )
    });
    let rpm = super::create_port::number(&entity.nbt, "Speed");
    let time = draw.game_time as f64 + f64::from(draw.partial);
    let angle = create_engine::visual::angle(
        time,
        rpm,
        create_engine::visual::rotation_offset(pos, Axis::Y, false),
    );
    draw.push();
    draw.rotate_about([0.0, 1.0, 0.0], angle, [0.5; 3]);
    game.native_draw_model(
        draw,
        "create:block/cogwheel_shaftless",
        super::create_port_presentation::kinetic_tint(game, pos),
    );
    draw.pop();
    let offset = cycle.rendered_head_offset(draw.partial, cos);
    draw.push();
    draw.translate([0.0, -offset, 0.0]);
    game.native_draw_model(draw, "create:block/mechanical_mixer/pole", 0xFFFFFF);
    draw.pop();
    let angle = ((time as f32 * cycle.rendered_rotation_speed(rpm) * 6.0 / 10.0) % 360.0) / 180.0
        * std::f32::consts::PI;
    draw.push();
    draw.rotate_about([0.0, 1.0, 0.0], angle.to_degrees(), [0.5; 3]);
    draw.translate([0.0, -offset, 0.0]);
    game.native_draw_model(draw, "create:block/mechanical_mixer/head", 0xFFFFFF);
    draw.pop();
    true
}
pub(super) fn client_tick(game: &mut Game<'_>) {
    if !game.native_window() || game.remote.is_none() || !game.native_on("create") {
        return;
    }
    let time = game.world.game_time;
    let entries: Vec<_> = game
        .world
        .block_entities
        .iter()
        .filter_map(|(&(x, y, z), e)| {
            let BlockEntity::Mod(e) = e else {
                return None;
            };
            (e.id == ID).then(|| {
                (
                    [x, y, z],
                    e.revision(),
                    saved(e),
                    super::create_port::number(&e.nbt, "Speed"),
                )
            })
        })
        .collect();
    let live: std::collections::BTreeSet<_> = entries.iter().map(|entry| entry.0).collect();
    with_world(game, |world| {
        world.client.retain(|pos, _| live.contains(pos))
    });
    for (pos, owner, received, rpm) in entries {
        let particles = with_world(game, |w| {
            let c = w.client.entry(pos).or_default();
            if c.tick.is_some_and(|t| t > time)
                || c.owner.is_some_and(|old| !old.same_entity(owner))
            {
                *c = Client::default();
            }
            c.owner = Some(owner);
            if c.tick == Some(time) {
                return false;
            }
            if c.received != Some(received) {
                c.cycle.running = received.0;
                c.cycle.ticks = received.1;
                c.received = Some(received);
            }
            let particles = c.cycle.running && c.cycle.ticks == 20;
            if c.cycle.ticks >= 40 {
                c.cycle.running = false;
                c.cycle.ticks = 0;
            } else if c.cycle.running {
                if rpm == 0.0 || rpm.abs() < rules::DEFAULT_MIXER_MINIMUM_RPM {
                    if c.cycle.ticks < 20 {
                        c.cycle.ticks = 40 - c.cycle.ticks;
                    } else if c.cycle.ticks == 20 {
                        c.cycle.ticks += 1;
                    }
                }
                if c.cycle.ticks != 20 {
                    c.cycle.ticks += 1;
                }
            }
            c.tick = Some(time);
            particles
        });
        if !particles {
            continue;
        }
        if rpm.abs() >= 65.0 || time % 2 != 0 {
            let at = pos.map(|v| f64::from(v) + 0.5);
            let distance: f64 = at
                .iter()
                .zip(game.audio.listener)
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            let delay = if distance > 100.0 {
                (distance.sqrt() / 40.0 * 20.0) as i32
            } else {
                0
            };
            let sound =
                SoundInstance::at("create:mixing", 4, 0.75, 1.0, SoundEngine::unseeded(), at);
            if delay > 0 {
                let id = game.audio.engine.add(sound, false);
                game.audio.engine.play_delayed(id, delay);
            } else {
                game.audio.play_instance(sound, false);
            }
        }
        let stacks = basin::read(game, [pos[0], pos[1] - 2, pos[2]])
            .map(|b| {
                b.input
                    .iter()
                    .chain(&b.output)
                    .flatten()
                    .filter(|s| s.count > 0)
                    .map(basin::native_stack)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for stack in stacks {
            let angle = game.particles.level_random().next_float() * 360.0;
            let offset = rotate_y([0.0, 0.0, 0.25], angle);
            let mut target = rotate_y(offset, if rpm > 0.0 { 25.0 } else { -25.0 });
            target[1] += 0.25;
            let velocity = std::array::from_fn(|i| {
                target[i] - offset[i]
                    + f64::from((game.particles.level_random().next_float() - 0.5) * 2.0 / 128.0)
            });
            let center = [
                f64::from(pos[0]) + 0.5 + offset[0],
                f64::from(pos[1]) + 0.5 + offset[1] - 1.75,
                f64::from(pos[2]) + 0.5 + offset[2],
            ];
            let options = game.item_particle(&stack);
            game.particles.add(&options, center, velocity);
        }
    }
}
