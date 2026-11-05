# Create Horizon: overall porting roadmap

The goal is the **Create 6.0.10** experience pinned in `compatibility.json`: the same
machines, interactions, data, interfaces, sound and effects. Rust code may have a
different internal structure. Matching models or individual formulas alone does
not mean a subsystem is complete. Java addon JARs are not executed here.

Legend: `[x]` means a verified, bounded item; `[ ]` means pending or only partly
ported. A large subsystem will be checked off only after all its listed behavior
has been verified. Current detailed limits are in
[COMPATIBILITY.md](compatibility.md), with evidence in [VALIDATION.md](../VALIDATION.md).
Completion percentages are rough estimates, not a count of the rows below.

The 0.7 checkpoints cover Encased Chain Drive, generator timing, fan component
handling and the built-in resources. [Gameplay report](reviews/0.7-report.md)
· [Review route](reviews/0.7-test-world.md)

| Subsystem | Verified bounded foundation | Next acceptance criterion |
|---|---|---|
| Installation and resources | Working native pipeline, local JAR | Verify the next release through the same route |
| Kinetic network | Basic shafts, gears, gearboxes, stress and Encased Chain Drive; bounded source-event traces | Source ownership, propagation transients, effects and complex connections |
| Item processing | Depot, press, millstone, airflow; press/fan over belts, bounded compacting over a Basin | Accurate presentation and complete factory chains |
| Rotation sources | Creative Motor, Hand Crank, water wheels | Other sources, control and power systems |
| UI and visual feedback | Java comparisons of motor and Basin tooltips; other UI/effects are partial | Complete layout and lifecycle of supported screens, original lighting |
| Schematics | Bounded `.nbt` preflight and creative paste; one shared press fixture | Preview/UI, transformations, other machine support |
| Transport and automation | Bare belts, Minecraft hoppers and basic IO | Complete belts, funnels/tunnels, mechanical arms |
| Mixing and heating | Dry Basin/Mixer; heated brass; burner and fuel in separate native runs | Combined regression, missing interactions and presentation |
| Fluids | Water/lava in canonical Basin tanks; buckets, dough and mud | Complete processing, pipe networks, pumps, tanks |
| Contraptions | Pending | Shared assembly, transformations, collisions and actors |
| Trains and logistics | Pending | Tracks, trains, schedules, cargo |
| Ponder and guides | Pending | Original scenes, controls and presentation |
| Addons | Architectural requirement | Public Rust contracts; then Sable and Aeronautics |

## 1. Distribution, identifiers and validation

- [x] Build the mod from GitHub through the standard Horizon Launcher with pinned versions.
- [x] Load original resources from the player's local JAR without publishing them.
- [x] Built-in models work without the JAR. An actual game switches back when
  the JAR is removed, even with the cache still present.
- [x] Original IDs for supported blocks, items and recipes; import 61 localizations.
- [x] Verified separate native worlds for 0.3, 0.4 and 0.5, preserving older worlds.
- [x] Independent Java oracles for selected algorithms and real native checks.
- [x] Separate actual Java Create 6.0.10 client and local comparison bridge.
- [ ] Complete set of shared comparison scenarios for both clients.
- [ ] All supported screens, effects, sounds and animations match the reference.
- [ ] LAN/dedicated-server, data reload and resource reload regression tests.
- [ ] Complete table of original save fields and migration tests between port releases.
- [x] Separate 0.6 world: 13 stations, independent run of 15 groups, 6 survival crafts,
  2 actual restarts and inspection of 19 images. Binary boundaries are in the report.
- [x] Alpha.3 candidate: accepted upstream Mixins, native package and developer build.
- [x] macOS `.app` metadata and native-data fix merged upstream.
  [Horizon #9529](https://github.com/horizon-rust/horizon/pull/9529)
- [ ] Retest a clean installation with the updated Launcher and official Horizon.
- [x] Separate 0.7 developer review world: 23 independent audit groups, 16 inspected
  frames and two actual restart comparisons; resumed fixture correction disclosed.
- [x] Submit the 0.7.0-alpha.2 archive to the native mod registry.
  [Registry submission #252](https://github.com/horizon-rust/horizon-mods/pull/252)
- [ ] Complete registry review and publish an approved listing.

## 2. Rotation and engineer's tools

- [x] Shafts, small/large cogwheels, basic gear ratios and speed limit.
- [x] Gearbox, Clutch, Gearshift, original directions and rotating parts.
- [x] Hand Crank, motor, small and large water wheels: basic mechanics and persistence.
- [x] Wrench, rotation/removal, encased shafts; basic Speedometer/Stressometer.
- [x] Manual Copper Casing recipes and the NeoForge convention tags for
  stripped logs and wood. Connected textures and advancements remain pending.
- [x] Encased Chain Drive: 24 source states, placement/shape updates, ordinary Wrench,
  two recipes, six actual Java/native settled scenes and a native restart.
- [x] Bounded generator publication from initialization, GUI, crank and wrench events;
  raw NBT reads remain distinct. Per-component publication and lifetime cache tests.
- [ ] Complete source ownership, flicker/breakage, loaded-network and instantaneous
  detach semantics from RotationPropagator.
- [x] Adjustable Chain Gearshift. Settled shaft speeds and block states for all
  16 redstone strengths from either input end, plus native save/reload.
- [ ] Adjustable Chain Gearshift transient timing, full visual parity and
  survival crafting through rose quartz polishing and the Electron Tube.
- [ ] Remaining redstone controls, belt and gearbox connection variants.
- [ ] Clutch/Gearshift and Hand Crank in all network/animation edge cases.
- [ ] Original goggles tooltips and value settings, including RU/EN, colors, units and fading.
- [ ] Original speed/overstress particles, placement assistance, outlines and sound loops.
- [ ] Speed Controller, Rotational Speed Controller, Sequenced Gearshift and redstone modules.
- [ ] Windmill, bearings, Steam Engine, boiler and other stress sources/consumers.

## 3. Item processing and transport

- [x] Depot: top interaction, collection, output buffer, dropped items and hoppers.
- [x] Press: single-item cycle, basic recipes, acceleration, blocked output and save/reload.
- [x] Millstone: input, timer, chance outputs, collection and persistence.
- [x] Fan: item movement, processing media and item/Depot processing.
- [x] Bounded effective fire-resistance components: prototype/removal, canonical
  full-fire tag and recipe priority; actual Java/native scenes and native restart.
- [ ] General damage-selector/data-fixer equivalence across Minecraft versions.
- [x] Stationary drill: breaking the block in front, drops, speed and stopping.
- [ ] Fan: players/mobs/other entities, ignition/damage/effects and exact particles/sound.
- [ ] Full observable parity of already supported machines, including item animation.
- [x] Bounded press over a bare belt: single-item processing, holding the remainder,
  output through a Depot/hopper, independent power loss and an actual restart.
- [x] Bounded fan over a bare belt: washing, media changes,
  unsupported input, stopped belt, two-current queue and an actual restart.
- [x] Press/belt retain their cycle and items through 20 frozen steps;
  processing resumes after unfreezing, as in actual Java Create.
- [x] Bounded dry Basin/Mixer: original inventories, hopper IO,
  empty-hand collection, 30 RPM threshold, power loss, blocked output,
  recipes/bucket remainder and an actual restart. Cycle times of 83/62 ticks were
  checked against actual Java Create.
- [x] Canonical Depot/Basin/Millstone inventories: load before the first tick,
  removal of inventory halves, hopper IO and components; unsupported explicit `Size` is preserved
  opaquely without losing contents. Actual restart and empty
  network-root no-op were checked; this is not yet an arbitrary Java data fixer.
- [x] Bounded Basin: water/lava in two input and two output segments,
  ordinary buckets, dough for 1000 mB and mud for 250 mB; persistence and real native rendering.
- [x] Bounded heating: a filled burner with coal and a running Mixer
  turn `2 copper + 2 zinc → 4 brass`; power/heat loss does not consume input.
- [ ] Complete burner: combined ordinary-interaction run, networking, exact lighting,
  retained visual lifecycle, goggles and all effects without known differences.
- [ ] Belts: geometry, transported-item lifecycle, directions, slopes and IO.
- [x] Bounded surface geometry: X rotation fixed; 1600 source cases,
  60 actual Game/CPU draw cases and inspection of limited Java/native views.
- [x] Bounded bare-belt foundation: connector, original shapes, RPM transmission,
  items, output Depots/hoppers and persistence through an actual restart.
- [ ] Funnels/tunnels/chutes, filters, brass/andesite rules and batch transfer.
- [x] Ordinary physical item intake into the Basin and Millstone: original scenes,
  partial transfer, components and an actual restart.
- [x] Bounded Press/Basin: separate compacting pool, square crafting recipes,
  ice and three stone types with lava; repetition, stopping, overstress and persistence.
- [x] Bounded Press/Basin startup: 1,488 unshifted per-tick observations match,
  respecting actual source events and ticker order; no timer offset.
- [ ] Authentic cold-loaded Java/network traces and full propagation-order coverage.
- [x] Press client foundation: packets, pause/interpolation, three modes and
  item-particle order checked by oracles and actual Game tests.
- [ ] Complete Basin/Mixer: filters, side outputs, other compacting recipes,
  other fluids/heated recipes and network scenarios.
- [ ] Crushing Wheels, Mechanical Saw, Deployer, Spout, Item Drain, Mechanical Crafter and Sand Paper.
- [ ] Sequenced Assembly, tools/components of incomplete items and random results.
- [ ] Smart Observer, Stockpile Switch, Packager, packages and factory logistics.

## 4. Schematics and contraptions

- [x] Bounded GZIP/NBT reader, strict palette/state/data checks before writing.
- [x] Bounded creative paste of supported schematics, safe NBT and reported refusal;
  belts, entities, rotations and unsupported data are currently rejected.
- [x] Identical bytes of a five-block schematic produced one Copper Sheet in Java and Horizon;
  result collection was checked within a bounded time, without claiming exact timing.
- [ ] Rotations/mirrors with verified block and block entity transformations.
- [ ] Original Schematic and Quill, preview, UI, deployment and Schematicannon.
- [ ] Shared assembled-contraption model, glue/chassis and attachment rules.
- [ ] Bearings, piston, Rope Pulley, gantry, elevator and Cart Assembler.
- [ ] Collisions, players/passengers, moving inventories and contraption persistence.
- [ ] Deployer/drill/saw/plough/harvester actors and portable storage/fluid interfaces.
- [ ] Shared-schematic checks: identical machines produce identical results in both clients.

## 5. Fluids, railways and other content

- [ ] Pipes/pumps/valves, connectivity and flow direction.
- [ ] Fluid Tanks, Hose Pulley, Item Drain/Spout, displays and fluid levels.
- [ ] Trains: tracks, signals, stations, assembly, controls, schedules and persistence.
- [ ] Logistics networks, Redstone Links, Display Links and Display Boards.
- [ ] Materials/ores/world generation, decorative families and the full crafting set.
- [ ] Other equipment, tools, consumables and entity interactions.
- [ ] Original configurations, achievements/advancements, recipe integration and Ponder tutorials.

## 6. Extensibility and future mods

- [ ] Documented Rust APIs for kinetics, recipes, transported items, UI and contraptions.
- [ ] Stable contraption identity, coordinate systems and world/transform boundary.
- [ ] Separate Sable physics adapter with force, collision, persistence and network tests.
- [ ] Create Aeronautics on verified Create/Sable, without decorative substitutes for physics.
- [ ] Example external Rust addon and API version policy.

Order: reproducible comparison → accuracy of current systems → practical factories →
contraptions → fluids/trains/other systems → Sable/Aeronautics.
Each completed stage receives a build, validation evidence and a separate review world.
