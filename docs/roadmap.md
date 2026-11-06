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

The 0.8.0-alpha.7 checkpoint adds chassis and glue on mechanical pistons,
stationary sapling planting, and item unloading through a portable interface.
[Contraption review route](reviews/0.8-contraption-world.md)

## Current priority

Build the shared contraption engine alongside the remaining stationary machines.
Keep the Java comparisons as regressions and schedule presentation refinements
with the systems they depend on.

1. Expand the supported bearings and linear actuators with original Java
   comparisons. [Architecture and results](contraptions.md)
2. Extend capture to Super Glue, chassis and block entities. Define a durable
   cross-chunk write protocol before allowing a structure to span columns.
3. Expand contact, mounted storage and farm tools. Add shared schematic
   fixtures for repeating the same machine checks in both games.

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
| Contraptions | Bearings, pistons, Gantry and Rope Pulley with bounded cargo, riders and farm tools | Broader payloads, cross-chunk ownership, more drives, contact and portable storage |
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
- [x] Complete registry review and publish the 0.7.0-alpha.2 listing.
  [Create for Horizon](https://playhorizon.dev/mods/create)

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
- [ ] Adjustable Chain Gearshift transient timing and full visual parity.
- [x] Original Rose Quartz, Sand Paper, Electron Tube and Brass Hand recipes.
  Hand and ground polishing unlock the Deployer and Adjustable Chain Gearshift recipes.
- [ ] Remaining redstone controls, belt and gearbox connection variants.
- [x] Straight Clutch and Gearshift lines with either input end. Compare the
  immediate detach, next-tick reattachment and short pulses in Java and
  Horizon. [Transmission review](reviews/transmission.md)
- [ ] Clutch/Gearshift and Hand Crank in all network/animation edge cases.
- [ ] Original goggles tooltips and value settings, including RU/EN, colors, units and fading.
- [ ] Original speed/overstress particles, placement assistance, outlines and sound loops.
- [ ] Speed Controller, Rotational Speed Controller, Sequenced Gearshift and redstone modules.
- [x] Windmill Bearing: wool sails, assembly, rear shaft output and restoration.
- [ ] Complete Windmill controls, sail blocks, Steam Engine, boiler and other stress sources/consumers.

## 3. Item processing and transport

- [x] Depot: top interaction, collection, output buffer, dropped items and hoppers.
- [x] Bounded Depot slide and press lifecycle: canonical pose, lock release,
  two cycles and two sheets. Compared 101 unshifted lifecycle samples against
  the pinned Java client.
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
- [x] Belt connector green and red dust preview between clicks. Real Java and
  Horizon captures and accepted/rejected connection checks.
  [Review route](reviews/belt-preview.md)
- [x] Six belt layouts assembled through the connector in each client.
  Item travel matched in 16 of 18 powered, stopped and reversed phases.
  The remaining two phases share a Java source loop on one horizontal lane.
  Replacing a belt preserves the new chain and ejects the old load once.
  [Motion review](reviews/belt-motion.md)
- [ ] Belts: frame-by-frame scrolling, item shadows, complete lifecycle and IO.
- [ ] Belt passenger motion through a contact layer shared with moving surfaces.
  The static belt shapes stay in the block registry. Moving surfaces need
  transformed contact geometry and a world-space movement request.
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
- [x] Core captures connected blocks with host-provided attachment rules and
  preserves their snapshots through a failed restoration preflight.
- [x] Bearing angle trace and snapped log placement on X, Y and Z verified
  against the isolated Java client. [Architecture and results](contraptions.md)
- [x] Linear, Secondary Linear and Radial Chassis on a vertical Mechanical
  Bearing, including sticky faces, saved ranges and blocked landing recovery.
- [x] Chassis on a dry Gantry Carriage, including sticky capture, translated
  landing and saved ranges after an obstructed restart.
- [x] Chassis on a dry Rope Pulley with the selected range restored after
  restart and landing.
- [x] Chassis and glue on regular and sticky Mechanical Pistons, including
  forced linear rows, radial cargo and saved ranges after a cold restart.
- [x] Wrench range board for the three chassis variants, with single and Ctrl
  group settings and the saved value on reopen.
- [ ] Chassis attached-block outlines, other drivers and complete block movement rules.
- [x] Mechanical Bearing: native assembly, rotation, restart and orphan recovery.
- [x] Windmill Sail Frame and sixteen sail colors. Dye, shear, placement,
  windmill capture, generated speed and restart verified.
- [ ] Windmill sail placement preview and canvas contact behavior.
- [x] Clockwork Bearing: both hands and three modes, with atomic restoration.
- [x] Regular and sticky Mechanical Pistons: Java tick comparison, retraction,
  terrain stop, restart and recovery in one column.
- [x] Rope Pulley, dry rope and cargo. Lowering, capture, retraction, restart
  and rope break, checked against Java.
- [x] Gantry, bounded loaded rail and carriage. Motion, redstone stop, power
  transfer and restart checked in game.
- [x] Cart Assembler rail actions, mounted cart assembly rules and coupling
  corrections in the engine, with single and coupled Java reference fixtures.
- [ ] Complete Rope Pulley and Gantry presentation, wet lines, coupled pulleys,
  sequenced movement and elevator controls.
- [ ] Native Cart Assembler rails, cart coupling, mounted payload and renderer.
- [x] Dropped-item collision, grounded rotary and linear rider transfer, and
  collision of narrow piston poles in Horizon.
- [x] Unobstructed and wall-constrained side pushes from a horizontal piston,
  including mirrored wall exits checked in Java.
- [x] Side pushes and wall sliding from full cubes on a vertical Mechanical
  Bearing, with a pinned JAR geometry oracle and separate live-client tests.
- [x] One ordinary mounted chest on a vertical Mechanical Bearing, with crop
  and cutter loot, item components, restart and landing recovery.
- [ ] Full entity contact, multiple mounted inventories, moving chest menus
  and cross-chunk persistence.
- [x] Harvester on supported crops, moving Saw and Drill with cutting stalls,
  and horizontal stationary Saw tree felling.
- [ ] First active-cell timing and cutting order for newly assembled Saw rotors.
- [x] Moving Plough tilling, snow clearing and protection of farmland crops.
- [x] Downward Deployer planting from its own stock on a vertical Mechanical
  Bearing, with forty matching Java tick samples and restart coverage.
- [x] Deployer refill from a single mounted chest and a moving Saw on the same
  farm. First matching slot, blocked planting and restart checked.
- [x] Stationary downward Deployer planting with plain saplings, redstone
  lock, speed pause and cold-stroke resume.
- [x] Super Glue selection and transfer planner. Deployer stroke, target and
  storage rules checked against Java traces.
- [x] Two-click Super Glue and single-column bearing transfer, with durability
  and saved volume checks.
- [x] Super Glue transfer on dry Gantry and Rope Pulley, with off-origin
  capture and bounds restored after a cold native restart.
- [ ] Original Glue outlines and particles, offhand placement and transfer on other drivers.
- [x] Upward stationary Saw wood processing through native item collision,
  with saved progress and Java comparisons of the cutting and ejection cycle.
- [ ] Remaining Saw recipes, automation, filter and cutting particles.
- [x] Portable Storage Interface unloading from one moving chest into a
  vanilla hopper, with redstone, comparator, item components and restart.
- [ ] Portable Storage Interface input, generic item automation, fluid interfaces,
  Deployer filter interface and Plough entity throws.
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

Keep Java comparisons beside each new subsystem. Playable stages receive a
build, validation evidence and a separate review world.
