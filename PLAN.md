# Port plan and review checkpoints

Agreed with the project owner on 2026-11-03. The goal is a complete, unofficial
Rust implementation of Create for Horizon, delivered in reviewable increments.

## Working agreement

Each increment has a bounded feature list, automated checks, a reproducible
source commit and build, known limitations and a short visual checklist. Follow
the latest owner instruction below for when to hand over or continue across
increments. Automated verification is the developer's responsibility; the owner
tests appearance and interactions. Record every observation as a defect, an
unimplemented feature or a question; retain observations about later stages.

## Milestones

1. **Installation and builds (accepted).** Build through the official launcher CLI
   without local tool patches; carry code and public content in one native
   bundle; test a clean instance and GitHub source installation. Preserve normal
   Horizon instances. Deliver a launchable review instance and exact instructions.
2. **Appearance of the initial set.** Original models and textures from the
   player-supplied JAR; inventory, held, dropped and placed items; all supported
   orientations, lighting and animation. Review a visual test gallery.
3. **Behavior of the initial set.** Pin matching Create source/assets/reference
   versions. Compare Java and Rust kinetics, connection rules, direction, stress,
   pressing and depot interactions. Review a working small factory.
4. **Complete initial release.** Saves, automation, client/server synchronization,
   performance, recovery and repeated clean installation. Publish an accurately
   scoped experimental release after the review checkpoint.
5. **Expand by subsystem.** Mechanical transmissions and belts; processing;
   logistics; fluids; moving contraptions; trains. Split each subsystem into
   bounded increments with the same tests and visual review gates.
6. **Tutorials and completeness.** Ponder, remaining interfaces/effects,
   compatibility and an inventory of every original feature and known difference.

## Architecture and verification

Keep portable behavior in Rust crates and Horizon integration in a small adapter.
Preserve observable behavior and data conventions while using efficient native
Rust structures; mirroring every Java class is not a requirement. Cache compiled
recipe inputs and graph results with explicit invalidation rather than repeating
registry/recipe work for each item on every tick.
Use documented named hooks where possible; raw source mixins pin the Horizon
commit and must be revalidated on updates. Prefer internal workspace libraries;
extract reusable public libraries when their interfaces are established.

Use unit and boundary tests, Java-reference comparisons, stateful world scenarios,
save/reload checks, render checks, and actual launcher installation tests.
Test counts alone are not a completeness claim. CI records exact versions.

Plan a later shared-schematic test suite: construct the same small factory in
original Create and Horizon, supply the same inputs and actions, and compare
outputs, item components, timing, direction, failure conditions and saved state.
Use this to close gaps left by isolated method oracles and Horizon-only tests.
This cross-runtime runner is future work, not part of the current delivery and
not a promise of Java/Horizon world or schematic-format interchange.

## Distribution

The target is a GitHub URL or a reviewed Horizon registry entry followed by the
launcher's normal native build. Original Create assets are not redistributed;
the player supplies the supported JAR locally. Registry acceptance is the
maintainers' decision and does not make this an official Horizon/Create project.

## Completed increment: original interactions (0.2.0)

The owner accepted the installation checkpoint. Reported differences: shafts only
place vertically; the depot opens a chest-like menu. Implement source-derived
placement orientations and direct top-face depot exchange, preserve original
translations from the local JAR, and deliver another review build.

The owner clarified that cross-version Java/Horizon world interoperability is
not required. Preserve original Create IDs, state names and NBT conventions
where implemented. Record exceptions and avoid promising untested world compatibility.
The matching behavior/source/assets baseline is Create 6.0.10 for Minecraft 1.21.1.

Delivery: version 0.2.0, source bundle `b839dcb`, official performance build
`native-fc02103-0554886ea08b`, instance **Create Review 0.2**, new world
**CreateReview02**. Placement, direct depot exchange, all upstream locales and
the identifier baseline are implemented and checked. Stop for the owner's visual
review; this is not acceptance of the remaining appearance/behavior milestones.
See `LAUNCHER.md`, `VALIDATION.md` and `COMPATIBILITY.md` for the checklist,
evidence and explicit data-format exceptions.

## Completed increment: playable foundation (0.3.0)

The owner accepted placement, original graphics and names, and requested a larger
development interval. Reported defects: stale depot rendering after pickup,
unretrievable processing output, bulk pressing instead of the default per-item
processing, missing speed controls and incorrect/missing sound accompaniment.

1. Reproduce these failures; distinguish server inventories, client update tags,
   rendered state and player inventory. Test clicks during a processing cycle,
   partial extraction, output buffers, stack components, break/drop and reload.
2. Implement the original default pressing policy (`bulkPressing = false`),
   a proper depot processing/output model and explicit client snapshot clearing.
3. Extend the kinetic core with perpendicular large gears, gearboxes, clutches,
   gearshifts and a hand crank. Add persistent motor speed/direction control,
   wrench interactions and redstone switching. Compare source-derived rules.
4. Add the material/casing/crafting foundation for the implemented machines,
   retaining upstream IDs and recipe identities. Import required original models,
   textures and sounds only from the player's JAR.
5. Check unit/state-machine tests, Java oracles, actual crafting, hopper/player
   exchanges, client snapshots/rendering, power changes and saves through Horizon's
   control API. Inspect the matching Prism Create installation as a reference;
   never alter its existing worlds.
6. Deliver another native performance build with a new review world and an exact
   feature/limitation list, then stop for visual review. Unimplemented mechanisms
   are not counted as features merely because an item can be spawned.


0.3 automated checkpoint: implemented the 17-block/26-entry foundation, 30 source
recipes, default per-item pressing, cleared client snapshots, motor controls,
transmissions, wrench/casings and item/hopper/world IO. The performance Metal
scenario passed without skipped checks; 34 Rust tests and four Java oracles pass.
The Prism reference JAR/config were inspected read-only; Java gameplay itself was
not run. The delivered native package corresponds to source `12d83a0`.

Review instance: **Create Foundation 0.3**, world **CreateFoundation03**, official
performance build `native-fc02103-7ea49a8f5494`. Remain at this checkpoint for the
owner's visual/audio feedback before expanding to belts and further processing.


## Previous increment: engineer tools, feedback and survival power (0.4.0)

The owner visually accepted depot IO/default pressing, mechanical propagation,
wrench rotation and speed limits in 0.3, then requested a wider next increment.
Observed defects: opposing visible angular motion across adjacent split-shaft
ports, hand-crank handle facing/animation and non-rotating base, incomplete depot
item presentation. Missing scope: vertical gearbox item, brass pressing, goggles,
speed/stress instruments, placement assistance, particles and survival generators.

1. Compare rendering transforms and crank interpolation to pinned Java, including
   all axis signs. Verify actual rendered poses, not only server RPM fields.
2. Add original vertical gearbox item/conversions, brass sheet/pressing, engineer
   goggles and speed/stress gauges with their recipes, states and saved readings.
3. Add a reusable translated information HUD and in-world connection/rotation
   feedback, plus source-derived processing/kinetic particles through Horizon.
4. Add small/large water wheels with flow generation, capacity, placement and
   physical occupancy rules derived from Java. Keep moving contraptions as their
   own later subsystem; do not simulate them with decorative placeholder items.
5. Extend source oracles and real-game interaction/render/save tests. Preserve the
   owner's finished 0.3 world; prepare a separate native review instance/world.
6. Publish a tested build and stop again for visual/audio review. Scope and timing
   are governed by a coherent working increment, not a percentage-of-mod claim.

## Current increment: milling, bulk fan processing and stationary drilling (0.5.0)

Extend the usable processing chain with the source millstone and encased fan,
including washing, smoking, haunting and blasting, plus stationary drill block
breaking. Keep the existing engineering
and foundation scenarios as regression coverage. The public catalog now contains
24 blocks, 15 standalone items, 123 source recipes and 35 tags; the local JAR
importer prepares 324 private resource files with cache schema 7.

1. Port millstone kinetics, timer, probabilistic output, filtered inventory,
   fallen-item intake and output-first empty-hand collection. Preserve original
   inventory names and timer on save/reload, and render the original inner part.
2. Port the fan's six-way shaft/placement behavior, push/pull range, obstruction
   and media segments, item acceleration, stack timing and depot processing.
   Use actual Minecraft cooking recipes for smoking/blasting and source
   Create recipes for washing/haunting.
3. Add the genuine propeller, wheat-flour and dough ingredients, source crafting
   recipes and supported vanilla processing results. Keep original assets private
   and import only JSON or Horizon-supported OBJ resources from the supplied JAR.
4. Compare millstone/fan algorithms with pinned Java method bodies; check real
   item IO, tagged ingredients, blocked outputs, power changes, all orientations,
   saves and client presentation in native Horizon. Complete the full regression
   and launcher build before describing a tested 0.5 delivery.
5. Add the stationary mechanical drill with source facing/waterlogged states,
   front-block break progress, rear shaft, ordinary loot and rotating head. Keep
   contraption drilling, entity damage and automatic block placement out of scope.
6. Prepare a separate review world and record its exact source/build/instance in
   `VALIDATION.md`. Preserve the owner's previous worlds. The visual checklist is
   in `TESTING_05.md`; native verification and an independent actual-map audit
   have passed on performance build `native-fc02103-722bc455fcbe`.

This increment is ready for the owner's next visual review. The full Metal/audio
regression ran 2,729 control commands without skipped scenarios; the independent
map audit checked eight station groups and five real-menu crafts on an isolated
copy. Future increments should first cover the reported visual differences, then
extend coherent processing/transport systems with the same source-oracle and
native integration approach. Do not register decorative stand-ins for unported
contraptions, belts or fluid machinery.

Explicit current omissions: fan airflow on mobs/players, full audiovisual parity,
belts and nozzle behavior. Dropped-item processing progress is now represented in
the canonical `NeoForgeData.CreateData.Processing.Type/Time` entity fields using
Horizon's existing persistent unknown-field storage; native save/reload validation
has passed, including exact remaining time and unknown sibling fields.
Depot-held progress uses its saved transported-item fields. Neither
path stores entity state in item components or requires another raw hook.

## Next increment: Java experience parity and shared schematic fixtures

The owner's review of 0.5 found that matching block models and selected machine
algorithms does not yet reproduce the Create experience. Exact observable
behavior and presentation are the highest-priority acceptance criteria, including
all interfaces. The arrow-model speed hints and simplified goggles overlay are
reported mismatches, not accepted substitutes. `AGENTS.md` records these rules.

1. Establish an isolated original reference: Create 6.0.10, Minecraft 1.21.1,
   NeoForge 21.1.227 and Java 21, reusing the verified Prism caches. Keep the
   existing CreateAERO instance and its worlds unchanged. Add a development-only
   control bridge for commands, tick stepping, state/NBT, camera and captures.
2. Implement strict original `.nbt` schematic reading and creative placement for
   shared fixtures. Preserve source placement/safe-NBT policy. Preflight all
   IDs/properties/required data and reject unsupported constructs before writes.
   Produce comparable fixture reports for both clients. Initial scope is direct
   creative placement; full schematic UI/printer/cannon features remain separate.
3. Complete existing-system parity before counting catalog expansion: original
   goggles tooltips and value settings, source rotation/airflow particles and
   sounds, fan effects on players/mobs and source damage/effect semantics.
   Compare exact UI content/layout and deterministic effect traces, with matching
   camera, GUI scale, language, machine state and observation time.
4. Refine the module boundaries into kinetic simulation, processing recipes,
   transported items, UI/presentation, schematics and moving-structure adapters.
   Preserve original identifiers and useful extension contracts for future Rust
   addons. Plan a separate Sable physics backend and Create Aeronautics port;
   do not implement speculative physics stubs or load Java addons as Rust code.
5. Expand coherent factory systems, prioritizing item transport and basin/mixer
   processing after the comparison foundation is operational. Port actual
   mechanics and their UI/effects together, with original-Java and native checks.
6. Deliver a substantial reviewed increment with a new map, complete checklist,
   source/build identity, actual cross-client evidence and explicit remaining
   differences. The requested 20–25% overall coverage is an informal scope goal;
   report measurable completed systems rather than inventing a precise total.

Historical Create releases are context rather than port milestones: their feature
sets and behavior differ from the pinned modern reference. Passing the existing
157,204 isolated Java comparisons does not replace full-client comparisons.

### Work budget and final handoff

Continue beyond the informal 20–25% scope estimate while useful work and account
quota remain. Monitor the actual shared account limit; reserve time for a coherent
build, regression checks, a separate review world, publication and the final
checklist. Stop around 10% remaining quota as requested; do not consume reset
credits. Publish a repository-wide subsystem checklist linked from README, using
checked items only for completed, verified behavior and explicitly marking
partial implementations and presentation differences. Preserve every existing
player instance/world.

## Previous owner instruction: finish this checkpoint and stop

On 2026-11-04 the owner replaced the earlier quota/percentage work target with
finishing the current development increment and handing it over. Close Stage10
item contact/material fixes and Stage11 Press/Basin/client pressing, preserve
measured differences, run native regressions, publish reproducible source and
prepare the review world. Do not start another subsystem after this handoff.
The repository-wide acceptance checklist remains in ROADMAP.md.

## Current owner instruction: resume systematic development

On 2026-11-05 the owner resumed development. Maintain all public documentation
in English and commit/push the documentation and implementation. Preserve the
original game localizations. Defer a registry proposal to a later owner review;
do not submit one during this development session.

First verify the build, Java-reference, game-control and GitHub tools. Record
available paths and concrete limitations; a failed optional browser/API write
must not halt independent implementation or tests. During ongoing work, inspect
the existing upstream PRs roughly once per hour, without creating a scheduled
automation or merging changes automatically.

Continue through coherent increments under the existing source-parity rules.
Close measured behavior/data gaps before expanding the affected system. The
historical Press/Basin startup discrepancy has now been traced to source events
and actual ticker order: 1,488 bounded unshifted native/original observations match
in 0.7. Full source ownership, cold-loaded networks and propagation transients
remain open; those trace results are not complete kinetic-network acceptance.
Then add the next supported processing/transport features from the subsystem
plan, each with source-derived tests and actual integration checks. Moving
contraptions were suggested as future scope, not the priority for this session.

Do not stop merely because the previous checkpoint was delivered. Continue until
the owner interrupts or a coherent handoff is necessary, with all partial work
and remaining checks recorded accurately. Preserve existing worlds and instances.

### 0.7 review checkpoint

Encased Chain Drive, bounded generator source events, placement shaft contracts
and effective fan fire-resistance mapping are implemented. The final native
binary and an independently exercised clone of CreateFactory07 are ready for
owner review. See REPORT_07.md and TESTING_07.md; ROADMAP.md remains the overall
checklist. The original failed audit click and the resumed acceptance are both
retained, rather than relabeling the first run as successful.

Next: review the visual feedback, expand shared Java/native network scenarios,
close supported transport/processing gaps, then select another bounded original
feature. Contraptions remain later work. Every new feature keeps source-derived
tests, ordinary interactions, persistence and presentation review as acceptance
requirements.
