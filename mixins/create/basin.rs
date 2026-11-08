// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Create 6.0.10 BasinBlockEntity/BasinInventory/BasinRenderer adapter.
// Heat-aware, unfiltered, downward basin; two canonical water/lava tank halves.
// Unknown tanks, filters and spouts remain unsupported; never discard them.
use super::create_port_block_entity_revision::EntityRevision as _;
use super::{
    Game, GameLevel, create_port_basin_recipes::item_drain, create_port_fluid_codec as fluids,
    native_block_entity_draw::BlockEntityDraw,
};
use crate::{
    containers::{BlockEntity, ModBlockEntity, ModContainer, ModRevision},
    items::ItemStack,
};
use create_engine::{basin as rules, transport::SessionCache};
use horizon_client::mod_api::{
    NativeFluidAction, NativeFluidDrain, NativeFluidRenderOptions, NativeFluidStack,
    NativeFluidTankView,
};
use horizon_core::random::{LegacyRandom, RandomSource};
use horizon_world::nbt::{Compound, Tag};
use std::{cell::RefCell, collections::BTreeMap};

const ID: &str = "create:basin";
type Pos = [i32; 3];

#[derive(Clone, Debug)]
pub(super) struct Key(pub ItemStack);
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0.same_item_same_components(&other.0)
    }
}
impl Eq for Key {}
pub(super) type Basin = rules::Basin<Key, fluids::Key>;

#[derive(Default)]
struct Motion {
    owner: Option<ModRevision>,
    tick: Option<i64>,
    previous: f32,
    angle: f32,
    speed: f32,
    lazy: i32,
    moving: bool,
    fluid_levels: [rules::FluidLevel; 4],
    received_fluids: [Option<fluids::Level>; 4],
    rendered_fluids: [Option<rules::Fluid<fluids::Key>>; 4],
}
struct Server {
    owner: Option<ModRevision>,
    initialized: bool,
    contents_changed: bool,
    tick: Option<i64>,
    lazy: i32,
    backup: i32,
    heat: rules::HeatCache,
}
impl Default for Server {
    fn default() -> Self {
        Self {
            owner: None,
            initialized: false,
            contents_changed: false,
            tick: None,
            lazy: 10,
            backup: 20,
            heat: rules::HeatCache::default(),
        }
    }
}
impl Server {
    fn bind(&mut self, owner: ModRevision) {
        if self.owner.is_some_and(|old| !old.is_same_create_entity(owner)) {
            *self = Self::default();
        }
        self.owner = Some(owner);
    }
}
thread_local! {
    static MOTION: RefCell<SessionCache<BTreeMap<Pos, Motion>>> = RefCell::new(SessionCache::default());
    static SERVER: RefCell<SessionCache<BTreeMap<Pos, Server>>> = RefCell::new(SessionCache::default());
}

pub(super) fn reset(reload: bool) {
    if !reload {
        MOTION.with(|m| m.borrow_mut().clear());
        SERVER.with(|m| m.borrow_mut().clear());
        tank::reset();
    }
}

/// BasinInventory.whenContentsChanged/notifyChangeOfContents only raises a
/// flag. The next ordinary Basin tick schedules its operator checker.
pub(super) fn notify_contents(game: &Game<'_>, pos: Pos) {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return;
    };
    if entity.id != ID || !crate::game::create_port::inventory_supported(entity) {
        return;
    }
    SERVER.with(|cache| {
        let mut cache = cache.borrow_mut();
        let server = cache
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .entry(pos)
            .or_default();
        server.bind(entity.revision());
        server.contents_changed = true;
    });
}

pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if block == ID && container.is_none() {
        *container = Some(ModContainer {
            slots: 18,
            faces: std::array::from_fn(|_| (0..18).collect()),
            inputs: (0..9).collect(),
            // CombinedInvWrapper exposes BOTH input and output to extraction.
            outputs: (0..18).collect(),
            menu: None,
        });
    }
}

fn inventory(slots: &[Option<ItemStack>]) -> Tag {
    Tag::Compound(Compound::from([
        ("Size".into(), Tag::Int(9)),
        (
            "Items".into(),
            Tag::List(
                slots
                    .iter()
                    .enumerate()
                    .filter_map(|(i, s)| {
                        let stack = s.as_ref().filter(|s| s.count > 0)?;
                        let Tag::Compound(mut nbt) = horizon_items::item_nbt::item_to_nbt(stack)
                        else {
                            return None;
                        };
                        nbt.insert("Slot".into(), Tag::Int(i as i32));
                        Some(Tag::Compound(nbt))
                    })
                    .collect(),
            ),
        ),
    ]))
}
fn restore(entity: &mut ModBlockEntity) {
    // Canonical inventory is decoded before publication. Never re-read an old
    // bridge snapshot after ordinary automation has already changed live slots.
    if crate::game::create_port::inventory_supported(entity) {
        entity.items.resize(18, None);
    }
}
pub(super) fn data(entity: &mut ModBlockEntity) -> bool {
    if entity.id != ID || !crate::game::create_port::inventory_supported(entity) {
        return false;
    }
    entity.items.resize(18, None);
    let mut changed = false;
    for (key, range) in [("InputItems", 0..9), ("OutputItems", 9..18)] {
        let Tag::Compound(updated) = inventory(&entity.items[range]) else {
            unreachable!();
        };
        let mut value = match entity.nbt.get(&key.into()) {
            Some(Tag::Compound(existing)) => existing.clone(),
            _ => Compound::new(),
        };
        value.extend(updated);
        let value = Tag::Compound(value);
        changed |= entity.nbt.get(&key.into()) != Some(&value);
        entity.nbt.insert(key.into(), value.clone());
    }
    changed
}
fn empty_tanks() -> Tag {
    let segment = Tag::Compound(Compound::from([
        (
            "Level".into(),
            Tag::Compound(Compound::from([
                ("Force".into(), Tag::Byte(1)),
                ("Speed".into(), Tag::Float(0.25f32.to_bits())),
                ("Target".into(), Tag::Float(0.0f32.to_bits())),
                ("Value".into(), Tag::Float(0.0f32.to_bits())),
            ])),
        ),
        ("TankContent".into(), Tag::Compound(Compound::new())),
    ]));
    Tag::List(vec![segment.clone(), segment])
}
pub(super) fn initialize(entity: &mut ModBlockEntity) {
    if entity.id != ID || !crate::game::create_port::inventory_supported(entity) {
        return;
    }
    restore(entity);
    for key in ["InputTanks", "OutputTanks"] {
        entity.nbt.entry(key.into()).or_insert_with(empty_tanks);
    }
    for key in ["DisabledSpoutput", "Overflow", "FluidOverflow"] {
        entity
            .nbt
            .entry(key.into())
            .or_insert_with(|| Tag::List(Vec::new()));
    }
    entity
        .nbt
        .entry("Filter".into())
        .or_insert_with(|| Tag::Compound(Compound::new()));
    entity
        .nbt
        .entry("FilterAmount".into())
        .or_insert(Tag::Int(64));
    entity.nbt.entry("UpTo".into()).or_insert(Tag::Byte(1));
    data(entity);
}

fn nonempty(tag: Option<&Tag>) -> bool {
    match tag {
        None => false,
        Some(Tag::Compound(c)) => !c.is_empty(),
        Some(Tag::List(l)) => !l.is_empty(),
        _ => true,
    }
}
pub(super) fn unsupported(
    game: &Game<'_>,
    pos: Pos,
    entity: &ModBlockEntity,
) -> Option<&'static str> {
    if !crate::game::create_port::inventory_supported(entity) {
        return Some("basin inventory sizes are not supported");
    }
    if game.native_block_property(pos, "facing").as_deref() != Some("down") {
        return Some("basin side spouts are not implemented");
    }
    if nonempty(entity.nbt.get(&"Filter".into())) {
        return Some("basin recipe filters are not implemented");
    }
    for key in ["InputTanks", "OutputTanks"] {
        if let Err(fluids::Unsupported(reason)) = read_tanks(entity, key) {
            return Some(reason);
        }
    }
    if [
        "Overflow",
        "FluidOverflow",
        "PreferredSpoutput",
        "DisabledSpoutput",
    ]
    .into_iter()
    .any(|key| nonempty(entity.nbt.get(&key.into())))
    {
        return Some("basin spout buffers are not implemented");
    }
    None
}
fn component_maximum(stack: &ItemStack) -> Option<u32> {
    super::create_port::stack_maximum_override(stack)
}
pub(super) fn max_stack(game: &Game<'_>, stack: &ItemStack) -> u32 {
    super::create_port::stack_maximum(game, stack)
}
pub(super) fn has_analog(
    game: &GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    output: &mut bool,
) {
    if matches!(game.facts.get(state).name.as_ref(), ID | item_drain::BLOCK | tank::BLOCK) {
        *output = true;
    }
}
pub(super) fn analog(
    game: &GameLevel<'_>,
    pos: horizon_core::position::BlockPos,
    state: horizon_world::registry::BlockStateId,
    output: &mut i32,
) {
    let block = game.facts.get(state).name.as_ref();
    if !matches!(block, ID | item_drain::BLOCK | tank::BLOCK) {
        return;
    }
    *output = 0;
    let Some(BlockEntity::Mod(e)) = game.world.block_entities.get(&(pos.x, pos.y, pos.z)) else {
        return;
    };
    if block == item_drain::BLOCK {
        *output = item_drain::comparator(e);
        return;
    }
    if block == tank::BLOCK {
        *output = tank::comparator(e);
        return;
    }
    // Unsupported native-created inventories may not have canonical slots yet.
    // Keep their payload opaque and fail closed before any comparator indexing.
    if !crate::game::create_port::inventory_supported(e) {
        return;
    }
    let mut fullness = 0.0f32;
    let mut occupied = false;
    for stack in e.items.iter().take(9).flatten().filter(|s| s.count > 0) {
        let maximum = component_maximum(stack)
            .unwrap_or_else(|| game.items.get(&stack.item).map_or(64, |i| i.max_stack))
            .min(64);
        fullness += stack.count as f32 / maximum as f32;
        occupied = true;
    }
    *output = (fullness / 9.0 * 14.0).floor() as i32 + i32::from(occupied);
}
pub(super) fn can_survive(
    game: &GameLevel<'_>,
    pos: horizon_core::position::BlockPos,
    state: horizon_world::registry::BlockStateId,
    survives: &mut Option<bool>,
) {
    let block = game.facts.get(state).name.as_ref();
    if block == "create:mechanical_mixer" {
        *survives = Some(
            !matches!(game.world.block_entities.get(&(pos.x,pos.y-1,pos.z)),Some(BlockEntity::Mod(e)) if e.id == ID),
        );
        return;
    }
    if block != ID {
        return;
    }
    *survives = Some(
        !matches!(game.world.block_entities.get(&(pos.x,pos.y+1,pos.z)),Some(BlockEntity::Mod(e)) if matches!(e.id.as_str(),"create:mechanical_mixer"|"create:mechanical_press")),
    );
}
pub(super) fn core_stack(game: &Game<'_>, stack: &ItemStack) -> rules::Item<Key> {
    let mut key = stack.clone();
    key.count = 1;
    rules::Item {
        key: Key(key),
        count: stack.count,
        maximum: max_stack(game, stack),
    }
}
pub(super) fn native_stack(item: &rules::Item<Key>) -> ItemStack {
    ItemStack {
        count: item.count,
        ..item.key.0.clone()
    }
}
pub(super) fn read(game: &Game<'_>, pos: Pos) -> Option<Basin> {
    let BlockEntity::Mod(entity) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))? else {
        return None;
    };
    if entity.id != ID || unsupported(game, pos, entity).is_some() {
        return None;
    }
    Some(Basin {
        input: std::array::from_fn(|i| {
            entity
                .items
                .get(i)
                .and_then(Option::as_ref)
                .filter(|s| s.count > 0)
                .map(|s| core_stack(game, s))
        }),
        output: std::array::from_fn(|i| {
            entity
                .items
                .get(9 + i)
                .and_then(Option::as_ref)
                .filter(|s| s.count > 0)
                .map(|s| core_stack(game, s))
        }),
        input_fluids: read_tanks(entity, "InputTanks").ok()?,
        output_fluids: read_tanks(entity, "OutputTanks").ok()?,
        ..Basin::default()
    })
}
pub(super) fn heat(game: &Game<'_>, pos: Pos) -> rules::Heat {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return rules::Heat::None;
    };
    if entity.id != ID {
        return rules::Heat::None;
    }
    SERVER.with(|cache| {
        let mut cache = cache.borrow_mut();
        let server = cache
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .entry(pos)
            .or_default();
        server.bind(entity.revision());
        server.heat.get_or_read(|| {
            let state = game.world.block(pos[0], pos[1] - 1, pos[2]);
            let Some(state) = game.pack.assets.registry.state(state) else {
                return Some(rules::Heat::None);
            };
            let blaze = state
                .properties
                .get("blaze")
                .map(|value| match value.as_str() {
                    "smouldering" => rules::Heat::Smouldering,
                    "fading" => rules::Heat::Fading,
                    "kindled" => rules::Heat::Kindled,
                    "seething" => rules::Heat::Seething,
                    _ => rules::Heat::None,
                });
            // Same source priority and short-circuit: a real blaze level
            // wins before passive tags or campfire lit state are tested.
            if let Some(blaze) = blaze {
                return Some(blaze);
            }
            let passive = game
                .native_server_block_in_tag(state, "create:passive_boiler_heaters")
                .unwrap_or(false);
            let campfire = passive
                && game
                    .native_server_block_in_tag(state, "minecraft:campfires")
                    .unwrap_or(false);
            let lit = state.properties.get("lit").map(|v| v == "true");
            Some(rules::heat_for_block(blaze, passive, campfire, lit))
        })
    })
}
pub(super) fn write(game: &mut Game<'_>, pos: Pos, basin: &Basin) -> bool {
    let Some(changed) = game.native_with_mod_block_entity(pos, |entity| {
        if entity.id != ID || !crate::game::create_port::inventory_supported(entity) {
            return false;
        }
        // Preflight both canonical halves before any item/fluid write. Unsupported
        // original data is retained; it cannot become an empty tank by accident.
        let Ok((input, output)) = replacement_tanks(entity, basin) else {
            return false;
        };
        let fluid_changed = set_tanks(entity, input, output);
        entity.items = basin
            .input
            .iter()
            .chain(&basin.output)
            .map(|s| s.as_ref().map(native_stack))
            .collect();
        data(entity) || fluid_changed
    }) else {
        return false;
    };
    if changed {
        game.native_block_entity_sync(pos);
        notify_contents(game, pos);
    }
    changed
}

fn read_tanks(entity: &ModBlockEntity, key: &str) -> Result<fluids::Tanks, fluids::Unsupported> {
    // A fresh factory-created entity has not yet had its first initialization
    // tick. Reading/simulating that default must not write or dirty the entity.
    let empty = empty_tanks();
    fluids::read_half(Some(entity.nbt.get(&key.into()).unwrap_or(&empty)))
}
fn replacement_tanks(
    entity: &ModBlockEntity,
    basin: &Basin,
) -> Result<(Tag, Tag), fluids::Unsupported> {
    let empty = empty_tanks();
    Ok((
        fluids::write_half(
            Some(entity.nbt.get(&"InputTanks".into()).unwrap_or(&empty)),
            &basin.input_fluids,
        )?,
        fluids::write_half(
            Some(entity.nbt.get(&"OutputTanks".into()).unwrap_or(&empty)),
            &basin.output_fluids,
        )?,
    ))
}
fn set_tanks(entity: &mut ModBlockEntity, input: Tag, output: Tag) -> bool {
    let changed = entity.nbt.get(&"InputTanks".into()) != Some(&input)
        || entity.nbt.get(&"OutputTanks".into()) != Some(&output);
    if changed {
        entity.nbt.insert("InputTanks".into(), input);
        entity.nbt.insert("OutputTanks".into(), output);
    }
    changed
}
fn fluid_state(game: &Game<'_>, pos: Pos) -> Option<Basin> {
    if game.options.execution_role.is_client() {
        return None;
    }
    let BlockEntity::Mod(entity) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))? else {
        return None;
    };
    if entity.id != ID || !crate::game::create_port::inventory_supported(entity) {
        return None;
    }
    Some(Basin {
        input_fluids: read_tanks(entity, "InputTanks").ok()?,
        output_fluids: read_tanks(entity, "OutputTanks").ok()?,
        ..Basin::default()
    })
}
fn commit_fluids(game: &mut Game<'_>, pos: Pos, state: &Basin) {
    let changed = game
        .native_with_mod_block_entity(pos, |entity| {
            if entity.id != ID || !crate::game::create_port::inventory_supported(entity) {
                return false;
            }
            let Ok((input, output)) = replacement_tanks(entity, state) else {
                return false;
            };
            set_tanks(entity, input, output)
        })
        .unwrap_or(false);
    if changed {
        game.native_block_entity_sync(pos);
        notify_contents(game, pos);
    }
}
fn core_fluid(stack: &NativeFluidStack) -> Option<rules::Fluid<fluids::Key>> {
    (stack.amount > 0 && matches!(stack.fluid.as_str(), "minecraft:water" | "minecraft:lava")).then(
        || rules::Fluid {
            key: fluids::Key {
                id: stack.fluid.clone(),
                components: stack.components.clone(),
            },
            amount: stack.amount as u32,
        },
    )
}
fn native_fluid(stack: &rules::Fluid<fluids::Key>) -> NativeFluidStack {
    NativeFluidStack {
        fluid: stack.key.id.clone(),
        amount: stack.amount as i32,
        components: stack.key.components.clone(),
    }
}
/// Use the shared Horizon fluid helpers for sided automation.
pub(super) fn custom_fluids(block: &str, custom: &mut bool) {
    if matches!(block, ID | tank::BLOCK) {
        *custom = true;
    }
}
pub(super) fn fluid_tanks(
    game: &Game<'_>,
    pos: Pos,
    _side: Option<horizon_core::geometry::Direction>,
    out: &mut Option<Vec<NativeFluidTankView>>,
) {
    if game.native_block(pos) == tank::BLOCK {
        *out = tank::view(game, pos);
        return;
    }
    let Some(state) = fluid_state(game, pos) else {
        return;
    };
    *out = Some(
        (0..4)
            .map(|tank| NativeFluidTankView {
                tank,
                content: state.fluid(tank).map(native_fluid),
                capacity: rules::TANK_CAPACITY as i32,
                can_fill: tank >= 2,
                can_drain: true,
            })
            .collect(),
    );
}
pub(super) fn fluid_fill(
    game: &mut Game<'_>,
    pos: Pos,
    _side: Option<horizon_core::geometry::Direction>,
    resource: &NativeFluidStack,
    action: NativeFluidAction,
    out: &mut Option<i32>,
) {
    if game.native_block(pos) == tank::BLOCK {
        *out = tank::fill(game, pos, resource, action);
        return;
    }
    let Some(mut state) = fluid_state(game, pos) else {
        return;
    };
    *out = Some(0);
    let Some(resource) = core_fluid(resource) else {
        return;
    };
    // Injected into the defining crate today; reject future external variants.
    #[allow(unreachable_patterns)]
    let simulate = match action {
        NativeFluidAction::Simulate => true,
        NativeFluidAction::Execute => false,
        _ => return,
    };
    *out = Some(fill_input(game, pos, &mut state, &resource, simulate) as i32);
}
fn fill_input(
    game: &mut Game<'_>,
    pos: Pos,
    state: &mut Basin,
    resource: &rules::Fluid<fluids::Key>,
    simulate: bool,
) -> u32 {
    let amount = state.fill_input_fluid(resource, simulate);
    if !simulate && amount > 0 {
        commit_fluids(game, pos, state);
    }
    amount
}
pub(super) fn fluid_drain(
    game: &mut Game<'_>,
    pos: Pos,
    _side: Option<horizon_core::geometry::Direction>,
    request: &NativeFluidDrain,
    action: NativeFluidAction,
    out: &mut Option<NativeFluidStack>,
) {
    if game.native_block(pos) == tank::BLOCK {
        *out = tank::drain(game, pos, request, action);
        return;
    }
    let Some(mut state) = fluid_state(game, pos) else {
        return;
    };
    let selected;
    let request = match request {
        NativeFluidDrain::Any(maximum) if *maximum > 0 => rules::FluidRequest::Any(*maximum as u32),
        NativeFluidDrain::Exact(stack) => {
            let Some(stack) = core_fluid(stack) else {
                return;
            };
            selected = stack;
            rules::FluidRequest::Exact(&selected)
        }
        _ => return,
    };
    // Injected into the defining crate today; reject future external variants.
    #[allow(unreachable_patterns)]
    let simulate = match action {
        NativeFluidAction::Simulate => true,
        NativeFluidAction::Execute => false,
        _ => return,
    };
    *out = drain(game, pos, &mut state, request, simulate)
        .as_ref()
        .map(native_fluid);
}
fn drain(
    game: &mut Game<'_>,
    pos: Pos,
    state: &mut Basin,
    request: rules::FluidRequest<'_, fluids::Key>,
    simulate: bool,
) -> Option<rules::Fluid<fluids::Key>> {
    let report = state.drain_fluid(request, simulate);
    if !simulate && report.consumed > 0 {
        commit_fluids(game, pos, state);
    }
    report.returned
}
/// `BasinInventory.insertItem`: unique component-bearing stack per half and
/// only its first empty slot. Normal hopper transfers are one item at a time.
pub(super) fn can_place(
    slots: &[Option<ItemStack>],
    local: usize,
    stack: &ItemStack,
    maximum: u32,
) -> bool {
    let maximum = component_maximum(stack).unwrap_or(maximum);
    if local >= 9 || slots.len() < 9 || stack.count == 0 {
        return false;
    }
    let input = &slots[..9];
    if input.iter().enumerate().any(|(i, s)| {
        i != local
            && s.as_ref()
                .is_some_and(|s| s.count > 0 && s.same_item_same_components(stack))
    }) {
        return false;
    }
    match input[local].as_ref().filter(|s| s.count > 0) {
        Some(held) => held.count < maximum.min(64) && held.same_item_same_components(stack),
        None => {
            input
                .iter()
                .position(|s| s.as_ref().is_none_or(|s| s.count == 0))
                == Some(local)
        }
    }
}

pub(super) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) -> bool {
    if entity.id != ID || !crate::game::create_port::inventory_supported(entity) {
        return false;
    }
    restore(entity);
    let changed = data(entity);
    initialize(entity);
    let owner = entity.revision();
    let already_ticked = SERVER.with(|cache| {
        let mut cache = cache.borrow_mut();
        let server = cache
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .entry(pos)
            .or_default();
        server.bind(owner);
        server.tick == Some(game.world.game_time)
    });
    if already_ticked {
        return changed;
    }
    SERVER.with(|cache| {
        cache
            .borrow_mut()
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .entry(pos)
            .or_default()
            .heat
            .reset()
    });
    for key in ["InputTanks", "OutputTanks"] {
        if let Some(tanks) = entity.nbt.get_mut(&key.into()) {
            let _ = fluids::tick_half(tanks);
        }
    }
    let scheduled = SERVER.with(|cache| {
        let mut cache = cache.borrow_mut();
        let s = cache
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .entry(pos)
            .or_default();
        if s.tick == Some(game.world.game_time) {
            return false;
        }
        if s.tick.is_some_and(|t| t > game.world.game_time) {
            *s = Server {
                owner: Some(owner),
                ..Server::default()
            };
        }
        // The duplicate-tick return above deliberately leaves this flag set.
        let mut scheduled = std::mem::take(&mut s.contents_changed) || !s.initialized;
        if !s.initialized {
            s.initialized = true;
            s.backup -= 1;
        }
        if s.lazy <= 0 {
            s.lazy = 10;
            if s.backup <= 0 {
                s.backup = 20;
                scheduled |= entity.items.iter().flatten().any(|s| s.count > 0)
                    || ["InputTanks", "OutputTanks"].into_iter().any(|key| {
                        read_tanks(entity, key)
                            .is_ok_and(|tanks| tanks.iter().flatten().any(|f| f.amount > 0))
                    });
            } else {
                s.backup -= 1;
            }
        } else {
            s.lazy -= 1;
        }
        s.tick = Some(game.world.game_time);
        scheduled
    });
    // Canonical serialization may normalize a supported loaded inventory's
    // order/numeric tags. Basin.read does not notify contents: only genuine
    // inventory/fluid events, initialization and the lazy backup wake it.
    if scheduled {
        crate::game::create_port_mixer::schedule(game, [pos[0], pos[1] + 2, pos[2]]);
        crate::game::create_port_press_basin::schedule(game, [pos[0], pos[1] + 2, pos[2]]);
    }
    if changed {
        let dim = game.dim.id().to_owned();
        game.native_update_output_neighbors(&dim, pos);
    }
    let reason = unsupported(game, pos, entity);
    match reason {
        Some(reason) => {
            entity.nbt.insert(
                "create_horizon:Unsupported".into(),
                Tag::String(reason.into()),
            );
        }
        None => {
            entity.nbt.remove(&"create_horizon:Unsupported".into());
        }
    }
    // Falling-item intake deliberately waits for the entity-sensitive basin
    // collision API; no guessed contact height or direct teleport is used.
    changed
}
pub(super) fn reacts(game: &Game<'_>, pos: Pos) -> bool {
    game.native_on("create")
        && game.native_block(pos) == ID
        && game.held().is_none_or(|s| {
            s.count == 0
                || matches!(
                    s.item.as_ref(),
                    "minecraft:air"
                        | "minecraft:water_bucket"
                        | "minecraft:lava_bucket"
                        | "minecraft:bucket"
                        | "minecraft:sponge"
                )
        })
}
pub(super) fn give_container(game: &mut Game<'_>, pos: Pos, stack: ItemStack) {
    let player = game.native_acting_id();
    if let Some(rest) = game.native_give_stack(player, stack) {
        if let Some(eye) = game.eye_position() {
            game.throw_from_eyes(rest, eye, false);
        } else {
            game.spawn_item(rest, pos.map(|v| f64::from(v) + 0.5), [0.0, 0.2, 0.0], 10);
        }
    }
}
fn bucket_click(game: &mut Game<'_>, pos: Pos, held: &ItemStack) -> bool {
    match held.item.as_ref() {
        "minecraft:water_bucket" | "minecraft:lava_bucket" => {
            let Some((fluid, amount, remainder)) = game.native_item_fluid_drain(held, 1000) else {
                return true;
            };
            let Some(mut state) = fluid_state(game, pos) else {
                return true;
            };
            let fluid = rules::Fluid {
                key: fluids::Key { id: fluid, components: Compound::new() },
                amount: amount as u32,
            };
            if fill_input(game, pos, &mut state, &fluid, true) != fluid.amount {
                return true;
            }
            fill_input(game, pos, &mut state, &fluid, false);
            if !game.creative {
                if held.count == 1 {
                    *game.held_mut() = Some(remainder);
                } else {
                    if let Some(s) = game.held_mut() {
                        s.count -= 1;
                    }
                    give_container(game, pos, remainder);
                }
            }
            true
        }
        "minecraft:bucket" => {
            let Some(mut state) = fluid_state(game, pos) else {
                return true;
            };
            for tank in 0..4 {
                let Some(mut fluid) = state.fluid(tank).cloned() else {
                    continue;
                };
                // API item containers cannot carry an opaque fluid component
                // patch. Refuse it explicitly rather than flattening it.
                if !fluid.key.components.is_empty() || fluid.amount < 1000 {
                    continue;
                }
                let Some((amount, filled)) = game.native_item_fluid_fill(held, &fluid.key.id, 1000)
                else {
                    continue;
                };
                if amount != 1000 {
                    continue;
                }
                fluid.amount = amount as u32;
                drain(game, pos, &mut state, rules::FluidRequest::Exact(&fluid), false);
                if !game.creative {
                    game.consume_held();
                    give_container(game, pos, filled);
                }
                return true;
            }
            true // GenericItemFilling claims even a refused/insufficient bucket.
        }
        "minecraft:sponge" => fluid_state(game, pos).is_some_and(|mut state| {
            drain(game, pos, &mut state, rules::FluidRequest::Any(i32::MAX as u32), false).is_some()
        }),
        _ => false,
    }
}
pub(super) fn right_click(game: &mut Game<'_>, pos: Pos) -> bool {
    if crate::game::create_port::inventory_blocked(game, pos) {
        return true;
    }
    if !reacts(game, pos) {
        return false;
    }
    if let Some(held) = game
        .held()
        .filter(|s| s.count > 0 && s.item.as_ref() != "minecraft:air")
        .cloned()
    {
        return bucket_click(game, pos, &held);
    }
    let stacks = game
        .native_with_mod_block_entity(pos, |entity| {
            restore(entity);
            let stacks = entity
                .items
                .iter_mut()
                .filter_map(Option::take)
                .collect::<Vec<_>>();
            data(entity);
            stacks
        })
        .unwrap_or_default();
    let player = game.native_acting_id();
    let success = !stacks.is_empty();
    for stack in stacks {
        if let Some(rest) = game.native_give_stack(player, stack) {
            if let Some(eye) = game.eye_position() {
                game.throw_from_eyes(rest, eye, false);
            } else {
                game.spawn_item(rest, pos.map(|v| f64::from(v) + 0.5), [0.0, 0.2, 0.0], 10);
            }
        }
    }
    if success {
        let pitch = 1.0 + game.world.random.next_float();
        game.with_level(|_, level, _| {
            level.native_play_sound(
                "minecraft:entity.item.pickup",
                pos.map(|v| f64::from(v) + 0.5),
                7,
                0.2,
                pitch,
            );
        });
    }
    game.native_block_entity_sync(pos);
    crate::game::create_port_mixer::basin_removed(game, [pos[0], pos[1] + 2, pos[2]]);
    crate::game::create_port_press_basin::basin_removed(game, [pos[0], pos[1] + 2, pos[2]]);
    true
}

/// Ordinary singleton FluidTankBlockEntity. A controller or non-singleton save
/// never falls back to empty storage. Towers, boiler and moving tanks need their
/// own topology adapter.
pub(super) mod tank {
    use super::*;
    use horizon_world::level::PlacementContext;

    pub(in crate::game) const BLOCK: &str = "create:fluid_tank";
    const CAPACITY: u32 = 8000;
    const BUCKET: i32 = 1000;
    const TOTAL_HEIGHT: f32 = 7.0 / 16.0;
    type Fluid = rules::Fluid<fluids::Key>;

    #[derive(Default)]
    struct RenderLevel {
        owner: Option<ModRevision>,
        tick: Option<i64>,
        chase: rules::FluidLevel,
    }

    thread_local! {
        static RENDER: RefCell<SessionCache<BTreeMap<Pos, RenderLevel>>> = RefCell::new(SessionCache::default());
    }

    pub(super) fn reset() {
        RENDER.with(|cache| cache.borrow_mut().clear());
    }

    pub(in crate::game) fn default_nbt() -> Compound {
        Compound::from([
            (
                "Boiler".into(),
                Tag::Compound(Compound::from([
                    ("ActiveHeat".into(), Tag::Int(0)),
                    ("Engines".into(), Tag::Int(0)),
                    ("PassiveHeat".into(), Tag::Byte(0)),
                    ("Supply".into(), Tag::Float(0)),
                    ("Update".into(), Tag::Byte(0)),
                    ("Whistles".into(), Tag::Int(0)),
                ])),
            ),
            ("Height".into(), Tag::Int(1)),
            ("Size".into(), Tag::Int(1)),
            ("Luminosity".into(), Tag::Int(0)),
            ("Window".into(), Tag::Byte(1)),
            ("TankContent".into(), Tag::Compound(Compound::new())),
        ])
    }

    fn read(entity: &ModBlockEntity) -> Option<(bool, Option<Fluid>)> {
        let nbt = &entity.nbt;
        if entity.id != BLOCK
            || nbt.contains_key(&"Controller".into())
            || !matches!(nbt.get(&"Size".into()), Some(Tag::Int(1)))
            || !matches!(nbt.get(&"Height".into()), Some(Tag::Int(1)))
        {
            return None;
        }
        let Tag::Compound(boiler) = nbt.get(&"Boiler".into())? else {
            return None;
        };
        if ["Engines", "Whistles"]
            .iter()
            .any(|key| !matches!(boiler.get(&(*key).into()), Some(Tag::Int(0))))
        {
            return None;
        }
        let window = match nbt.get(&"Window".into())? {
            Tag::Byte(value @ (0 | 1)) => *value != 0,
            _ => return None,
        };
        Some((
            window,
            fluids::read_content(nbt.get(&"TankContent".into()), CAPACITY).ok()?,
        ))
    }

    fn entity_at<'a>(game: &'a Game<'_>, [x, y, z]: Pos) -> Option<&'a ModBlockEntity> {
        match game.world.block_entities.get(&(x, y, z))? {
            BlockEntity::Mod(entity) => Some(entity),
            _ => None,
        }
    }

    fn current(game: &Game<'_>, pos: Pos) -> Option<(bool, Option<Fluid>)> {
        if game.options.execution_role.is_client() || !game.native_on("create") {
            return None;
        }
        let state = game
            .pack
            .assets
            .registry
            .state(game.world.block(pos[0], pos[1], pos[2]))?;
        if state.name != BLOCK
            || state.properties.get("top").is_none_or(|v| v != "true")
            || state.properties.get("bottom").is_none_or(|v| v != "true")
            || !matches!(
                state.properties.get("shape").map(String::as_str),
                Some("window" | "plain")
            )
        {
            return None;
        }
        read(entity_at(game, pos)?)
    }

    pub(in crate::game) fn placement(game: &GameLevel<'_>, ctx: &PlacementContext) -> bool {
        let [x, y, z] = [ctx.pos.x, ctx.pos.y, ctx.pos.z];
        let is_tank = |x, y, z| game.native_state_block(game.world.block(x, y, z)) == Some(BLOCK);
        if is_tank(x, y - 1, z) || is_tank(x, y + 1, z) {
            return false;
        }
        // Vertical groups always occupy the same column. For wider groups,
        // completing a horizontal 2×2 is the first point Java could merge.
        for dx in [-1, 1] {
            for dz in [-1, 1] {
                if !game.world.loaded(x + dx, z + dz)
                    || (is_tank(x + dx, y, z)
                        && is_tank(x, y, z + dz)
                        && is_tank(x + dx, y, z + dz))
                {
                    return false;
                }
            }
        }
        true
    }

    pub(in crate::game) fn tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
        if entity.nbt.contains_key(&"LastKnownPos".into()) || read(entity).is_none() {
            return;
        }
        entity
            .nbt
            .insert("LastKnownPos".into(), Tag::IntArray(pos.to_vec()));
        game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
    }

    fn fraction(fluid: Option<&Fluid>) -> f32 {
        fluid.map_or(0.0, |fluid| fluid.amount as f32 / CAPACITY as f32)
    }

    pub(super) fn comparator(entity: &ModBlockEntity) -> i32 {
        read(entity).map_or(0, |(_, fluid)| {
            let fraction = f64::from(fraction(fluid.as_ref()));
            (fraction * 14.0 + f64::from(u8::from(fraction > 0.0))).floor() as i32
        })
    }

    pub(super) fn view(game: &Game<'_>, pos: Pos) -> Option<Vec<NativeFluidTankView>> {
        let (_, fluid) = current(game, pos)?;
        Some(vec![NativeFluidTankView {
            tank: 0,
            content: fluid.as_ref().map(native_fluid),
            capacity: CAPACITY as i32,
            can_fill: true,
            can_drain: true,
        }])
    }

    fn commit(game: &mut Game<'_>, pos: Pos, fluid: Option<&Fluid>) -> bool {
        let Ok(content) = fluids::write_content(fluid, CAPACITY) else {
            return false;
        };
        let luminosity = fluid.map_or(0, |fluid| {
            if fluid.key.id == "minecraft:lava" {
                12
            } else {
                0
            }
        });
        if game
            .native_with_mod_block_entity(pos, |entity| {
                entity.nbt.insert("TankContent".into(), content);
                entity.nbt.insert("Luminosity".into(), Tag::Int(luminosity));
            })
            .is_none()
        {
            return false;
        }
        game.native_block_entity_sync(pos);
        game.native_update_output_neighbors(game.dim.id(), pos);
        true
    }

    pub(super) fn fill(
        game: &mut Game<'_>,
        pos: Pos,
        resource: &NativeFluidStack,
        action: NativeFluidAction,
    ) -> Option<i32> {
        let (_, current) = current(game, pos)?;
        let Some(mut fluid) = core_fluid(resource) else {
            return Some(0);
        };
        let stored = match current {
            Some(current) if current.key == fluid.key => current.amount,
            Some(_) => return Some(0),
            None => 0,
        };
        let accepted = fluid.amount.min(CAPACITY - stored);
        #[allow(unreachable_patterns)]
        match action {
            NativeFluidAction::Simulate => {}
            NativeFluidAction::Execute => {
                fluid.amount = stored + accepted;
                if accepted != 0 && !commit(game, pos, Some(&fluid)) {
                    return Some(0);
                }
            }
            _ => return Some(0),
        }
        Some(accepted as i32)
    }

    pub(super) fn drain(
        game: &mut Game<'_>,
        pos: Pos,
        request: &NativeFluidDrain,
        action: NativeFluidAction,
    ) -> Option<NativeFluidStack> {
        let (_, fluid) = current(game, pos)?;
        let mut fluid = fluid?;
        let maximum = match request {
            NativeFluidDrain::Any(maximum) => *maximum,
            NativeFluidDrain::Exact(wanted)
                if wanted.fluid == fluid.key.id && wanted.components == fluid.key.components =>
            {
                wanted.amount
            }
            _ => return None,
        };
        if maximum <= 0 {
            return None;
        }
        let drained = fluid.amount.min(maximum as u32);
        #[allow(unreachable_patterns)]
        match action {
            NativeFluidAction::Simulate => {}
            NativeFluidAction::Execute => {
                fluid.amount -= drained;
                if !commit(game, pos, (fluid.amount > 0).then_some(&fluid)) {
                    return None;
                }
            }
            _ => return None,
        }
        fluid.amount = drained;
        Some(native_fluid(&fluid))
    }

    pub(in crate::game) fn predicts_use(game: &Game<'_>, pos: Pos) -> bool {
        game.native_on("create")
            && game.native_block(pos) == BLOCK
            && !game.native_sneaking()
            && game
                .held()
                .filter(|held| held.count > 0)
                .is_some_and(|held| {
                    game.may_build() && crate::game::create_port_chassis::wrench(game, held)
                        || game.creative
                            && matches!(
                                held.item.as_ref(),
                                "minecraft:bucket"
                                    | "minecraft:water_bucket"
                                    | "minecraft:lava_bucket"
                            )
                })
    }

    pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: Pos) -> bool {
        if !predicts_use(game, pos) {
            return false;
        }
        let Some((window, before)) = current(game, pos) else {
            return true;
        };
        let Some(held) = game.held().cloned() else {
            return false;
        };
        if crate::game::create_port_chassis::wrench(game, &held) {
            let next = if window { "plain" } else { "window" };
            if game.native_set_block_property_with(
                pos,
                "shape",
                next,
                horizon_world::level::flags::UPDATE_CLIENTS,
            ) {
                game.native_with_mod_block_entity(pos, |entity| {
                    entity
                        .nbt
                        .insert("Window".into(), Tag::Byte(i8::from(!window)));
                });
                game.native_block_entity_sync(pos);
            }
            return true;
        }
        let (fluid, amount, emptying) = if held.item.as_ref() == "minecraft:bucket" {
            let Some(mut resource) = before
                .as_ref()
                .map(native_fluid)
                .filter(|f| f.components.is_empty())
            else {
                return true;
            };
            if resource.amount < BUCKET
                || game
                    .native_item_fluid_fill(&held, &resource.fluid, BUCKET)
                    .is_none_or(|(amount, _)| amount != BUCKET)
            {
                return true;
            }
            let remaining = resource.amount - BUCKET;
            resource.amount = BUCKET;
            let request = NativeFluidDrain::Exact(resource);
            let mut drained = None;
            game.native_custom_fluid_drain(
                pos,
                None,
                &request,
                NativeFluidAction::Execute,
                &mut drained,
            );
            let Some(drained) = drained else {
                return true;
            };
            (drained.fluid, remaining, false)
        } else {
            let Some((fluid, amount, _)) = game.native_item_fluid_drain(&held, BUCKET) else {
                return true;
            };
            let resource = NativeFluidStack {
                fluid,
                amount,
                components: Compound::new(),
            };
            let mut accepted = None;
            game.native_custom_fluid_fill(
                pos,
                None,
                &resource,
                NativeFluidAction::Simulate,
                &mut accepted,
            );
            if accepted != Some(amount) {
                return true;
            }
            accepted = None;
            game.native_custom_fluid_fill(
                pos,
                None,
                &resource,
                NativeFluidAction::Execute,
                &mut accepted,
            );
            if accepted != Some(amount) {
                return true;
            }
            (
                resource.fluid,
                before.as_ref().map_or(0, |f| f.amount as i32) + amount,
                true,
            )
        };
        play_bucket(game, pos, &fluid, amount, emptying);
        true
    }

    fn play_bucket(game: &mut Game<'_>, pos: Pos, fluid: &str, amount: i32, emptying: bool) {
        let event = match (emptying, fluid) {
            (true, "minecraft:lava") => "minecraft:item.bucket.empty_lava",
            (false, "minecraft:lava") => "minecraft:item.bucket.fill_lava",
            (true, _) => "minecraft:item.bucket.empty",
            (false, _) => "minecraft:item.bucket.fill",
        };
        game.with_level(|_, level, _| {
            let Some(random) = level.native_level_random() else {
                return;
            };
            let pitch = (1.0 - amount as f32 / (CAPACITY as f32 * 16.0)).clamp(0.0, 1.0) / 1.5
                + 0.5
                + (random.next_float() - 0.5) / 4.0;
            level.native_play_sound(
                event,
                pos.map(|coordinate| f64::from(coordinate) + 0.5),
                4,
                0.5,
                pitch,
            );
        });
    }

    pub(super) fn client_tick(
        session: usize,
        dimension: &str,
        time: i64,
        entries: &[(Pos, &ModBlockEntity)],
    ) {
        RENDER.with(|cache| {
            let mut cache = cache.borrow_mut();
            let cache = cache.dimension(session, dimension);
            let mut live = std::collections::BTreeSet::new();
            for &(pos, entity) in entries.iter().filter(|(_, e)| e.id == BLOCK) {
                let Some((_, fluid)) = read(entity) else {
                    continue;
                };
                live.insert(pos);
                let owner = entity.revision();
                let target = fraction(fluid.as_ref());
                let motion = cache.entry(pos).or_default();
                if motion.tick.is_some_and(|old| old > time)
                    || motion
                        .owner
                        .is_none_or(|old| !old.is_same_create_entity(owner))
                {
                    *motion = RenderLevel {
                        owner: Some(owner),
                        tick: None,
                        chase: rules::FluidLevel {
                            previous: target,
                            value: target,
                            target,
                            speed: 0.5,
                        },
                    };
                }
                motion.chase.target = target;
                if motion.tick != Some(time) {
                    motion.chase.tick();
                    motion.tick = Some(time);
                }
            }
            cache.retain(|pos, _| live.contains(pos));
        });
    }

    pub(in crate::game) fn draw(
        game: &Game<'_>,
        pos: Pos,
        entity: &ModBlockEntity,
        draw: &mut BlockEntityDraw,
    ) {
        let Some((true, Some(fluid))) = read(entity) else {
            return;
        };
        let level = RENDER.with(|cache| {
            cache
                .borrow_mut()
                .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
                .get(&pos)
                .filter(|motion| {
                    motion
                        .owner
                        .is_some_and(|old| old.is_same_create_entity(entity.revision()))
                })
                .map_or_else(
                    || fraction(Some(&fluid)),
                    |motion| motion.chase.at(draw.partial),
                )
        });
        if level < 1.0 / (512.0 * TOTAL_HEIGHT) {
            return;
        }
        let hull = 1.0 / 16.0 + 1.0 / 128.0;
        let floor = 5.0 / 16.0;
        let height = (level * TOTAL_HEIGHT).clamp(0.0, TOTAL_HEIGHT);
        game.native_draw_fluid_box(
            draw,
            &native_fluid(&fluid),
            [hull, floor, hull, 1.0 - hull, floor + height, 1.0 - hull],
            NativeFluidRenderOptions {
                argb: None,
                render_bottom: false,
                uv_shrink_ratio: 4.0 / 2048.0,
            },
        );
    }
}

pub(super) fn removed(game: &mut Game<'_>, pos: Pos) {
    SERVER.with(|cache| {
        cache
            .borrow_mut()
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .remove(&pos);
    });
    crate::game::create_port_mixer::basin_removed(game, [pos[0], pos[1] + 2, pos[2]]);
    crate::game::create_port_press_basin::basin_removed(game, [pos[0], pos[1] + 2, pos[2]]);
}
pub(super) fn maintenance(game: &mut Game<'_>) {
    SERVER.with(|cache| {
        cache
            .borrow_mut()
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .retain(|p, _| game.world.loaded(p[0], p[2]) && game.native_block(*p) == ID);
    });
}
pub(super) fn client_tick(game: &mut Game<'_>) {
    if !game.native_window() || game.remote.is_none() || !game.native_on("create") {
        return;
    }
    let time = game.world.game_time;
    let session = std::ptr::from_ref(&game.world) as usize;
    let dimension = game.dim.id().to_owned();
    let entries: Vec<_> = game
        .world
        .block_entities
        .iter()
        .filter_map(|(&(x, y, z), e)| match e {
            BlockEntity::Mod(e) if matches!(e.id.as_str(), ID | tank::BLOCK) => {
                Some(([x, y, z], e.as_ref()))
            }
            _ => None,
        })
        .collect();
    tank::client_tick(session, &dimension, time, &entries);
    let entries = entries.into_iter().filter_map(|(pos, e)| (e.id == ID).then(|| (pos, e.revision())));
    let entries: Vec<_> = entries.collect();
    let live: std::collections::BTreeSet<_> = entries.iter().map(|(pos, _)| *pos).collect();
    MOTION.with(|cache| {
        let mut cache = cache.borrow_mut();
        let cache = cache.dimension(session, &dimension);
        cache.retain(|p, _| live.contains(p));
        for (pos, owner) in entries {
            let m = cache.entry(pos).or_default();
            if m.tick.is_some_and(|t| t > time)
                || m.owner.is_some_and(|old| !old.is_same_create_entity(owner))
            {
                *m = Motion::default();
            }
            m.owner = Some(owner);
            if m.tick == Some(time) { continue; }
            // read(clientPacket) updates the chase target; the lazy tick is a
            // fallback for absent/removed mixers, as in BasinBlockEntity.
            let mixer = game.world.block_entities.get(&(pos[0],pos[1]+2,pos[2]));
            let moving = matches!(mixer, Some(BlockEntity::Mod(e)) if e.id == "create:mechanical_mixer" && crate::game::create_port::integer(&e.nbt,"Running") != 0 && crate::game::create_port::integer(&e.nbt,"Ticks") <= 20);
            if m.tick.is_none() || m.lazy <= 0 || m.moving != moving {
                m.moving = moving;
                m.lazy = 10;
            } else { m.lazy -= 1; }
            m.previous = m.angle;
            m.speed += ((if m.moving {20.0} else {0.0}) - m.speed) * 0.1;
            m.angle += m.speed;
            if let Some(BlockEntity::Mod(entity))=game.world.block_entities.get(&(pos[0],pos[1],pos[2])) {
                for (half,key) in ["InputTanks","OutputTanks"].into_iter().enumerate() {
                    let Ok(levels)=fluids::read_levels(entity.nbt.get(&key.into())) else {continue;};
                    let Ok(contents)=read_tanks(entity,key) else {continue;};
                    for (i,level) in levels.into_iter().enumerate() {
                        let index=half*2+i;
                        if let Some(fluid)=contents[i].as_ref(){m.rendered_fluids[index]=Some(fluid.clone());}
                        let chase=&mut m.fluid_levels[index];
                        if m.received_fluids[index]!=Some(level) {
                            if level.force {chase.value=level.value;chase.previous=level.value;}
                            chase.target=level.target;chase.speed=level.speed;
                            m.received_fluids[index]=Some(level);
                        }
                        chase.tick();
                    }
                }
            }
            m.tick = Some(time);
        }
    });
}
fn draw_fluids(
    game: &Game<'_>,
    pos: Pos,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) -> f32 {
    let rendered = MOTION.with(|cache| {
        let mut cache = cache.borrow_mut();
        let motion = cache
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .get(&pos);
        std::array::from_fn::<_, 4, _>(|i| {
            if let Some(m) = motion {
                return (
                    m.rendered_fluids[i].clone(),
                    m.fluid_levels[i].at(draw.partial) * rules::TANK_CAPACITY as f32,
                );
            }
            let key = if i < 2 { "InputTanks" } else { "OutputTanks" };
            let fluid = read_tanks(entity, key).ok().and_then(|t| t[i % 2].clone());
            let units = fluids::read_levels(entity.nbt.get(&key.into()))
                .map_or(0.0, |l| l[i % 2].value * rules::TANK_CAPACITY as f32);
            (fluid, units)
        })
    });
    let total: f32 = rendered.iter().map(|(_, units)| units).sum();
    if total < 1.0 {
        return 0.0;
    }
    let ratio = (total / 2000.0).clamp(0.0, 1.0);
    let height = 2.0 / 16.0 + 12.0 / 16.0 * (1.0 - (1.0 - ratio) * (1.0 - ratio));
    let mut x = 2.0 / 16.0;
    for (fluid, units) in rendered {
        let Some(fluid) = fluid else {
            continue;
        };
        if units < 1.0 {
            continue;
        }
        let next = x + (units / total).clamp(0.0, 1.0) * 12.0 / 16.0;
        game.native_draw_fluid_box(
            draw,
            &native_fluid(&fluid),
            [x, 2.0 / 16.0, 2.0 / 16.0, next, height, 14.0 / 16.0],
            NativeFluidRenderOptions {
                argb: None,
                render_bottom: false,
                // Actual source fixture's stitched blocks atlas is 2048x1024.
                // This is explicit because Horizon stores block sprites in arrays.
                uv_shrink_ratio: 4.0 / 2048.0,
            },
        );
        x = next;
    }
    height
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
    // Retained motion belongs to this live BE, including ordinary NBT changes.
    MOTION.with(|cache| {
        let mut cache = cache.borrow_mut();
        let motions = cache.dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id());
        if motions.get(&pos).is_some_and(|m| {
            m.owner
                .is_some_and(|old| !old.is_same_create_entity(entity.revision()))
        }) {
            motions.remove(&pos);
        }
    });
    // restore() truncates supported inventories to18, while opaque unsupported
    // inventories retain every live slot. Borrow the same visible range.
    let end = if crate::game::create_port::inventory_supported(entity) {
        entity.items.len().min(18)
    } else {
        entity.items.len()
    };
    let items = &entity.items[..end];
    let fluid_height = draw_fluids(game, pos, entity, draw);
    let item_height = (fluid_height - 0.3).clamp(0.125, 0.6);
    let mut count = items.iter().flatten().filter(|s| s.count > 0).count();
    if count == 0 {
        return true;
    }
    let rotation = MOTION.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache
            .dimension(std::ptr::from_ref(&game.world) as usize, game.dim.id())
            .get(&pos)
            .map_or(0.0, |m| m.previous + (m.angle - m.previous) * draw.partial)
    });
    // BlockPos.hashCode and RandomSource.create preserve stable item pile offsets.
    let hash = pos[1]
        .wrapping_add(pos[2].wrapping_mul(31))
        .wrapping_mul(31)
        .wrapping_add(pos[0]);
    let mut random = LegacyRandom::new(i64::from(hash));
    let partition = 360.0 / count as f32;
    let radius = if count == 1 { 0.0 } else { 0.125 };
    draw.push();
    draw.translate([0.5, 0.2, 0.5]);
    draw.rotate_about([0.0, 1.0, 0.0], rotation, [0.0; 3]);
    for item in items.iter().flatten().filter(|s| s.count > 0) {
        let angle = partition * count as f32;
        let at =
            crate::game::create_port_mixer::rotate_y([radius, f64::from(item_height), 0.0], angle);
        draw.push();
        if fluid_height > 0.0 {
            let phase = (draw.game_time as f32 + draw.partial) / 12.0 + angle;
            let bob = (crate::game::create_port_mixer::sin(phase) + 1.5) / 32.0;
            draw.translate([0.0, bob, 0.0]);
        }
        draw.translate(at.map(|v| v as f32));
        draw.rotate_about([0.0, 1.0, 0.0], angle + 35.0, [0.0; 3]);
        draw.rotate_about([1.0, 0.0, 0.0], 65.0, [0.0; 3]);
        for _ in 0..=item.count / 8 {
            draw.push();
            draw.translate(std::array::from_fn(|_| {
                (random.next_float() - 0.5) * 2.0 / 16.0
            }));
            game.native_draw_item(
                draw,
                item,
                horizon_assets::item_model::DisplayContext::Ground,
                None,
                0,
            );
            draw.pop();
        }
        draw.pop();
        count -= 1;
    }
    draw.pop();
    true
}
