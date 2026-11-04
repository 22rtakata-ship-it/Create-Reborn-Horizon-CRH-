# Current phase: original-reference comparison and factory foundations

The current review checkpoint is **0.6.0-alpha.2**, branch `stage/parity-06`.
[REPORT_06.md](REPORT_06.md) records the result at the current stopping point,
the installed build, and known differences. A separate user instance,
**Create Factory 0.6**, was created; earlier worlds and Prism instances are preserved.
The history of alpha.1 and later immutable snapshots is recorded separately below:
their results are not automatically attributed to a later binary.

The reference is actual Create 6.0.10 on Minecraft 1.21.1, with source pinned in
`compatibility.json`. Original assets are still imported only from the player's
local JAR. The runtime is entirely Rust; the Java bridge and oracles are used
only during development.

## Combined-build validation

The verified `0.6.0-alpha.1` checkpoint used Horizon
`8438f415fb0fe3bee40c9cd0f29d4ca2ffe869be`
from `Delete228/horizon:create-port-runtime-06`. This includes the generic fluid/
rendering contract, state-dependent block-entity creation, identity preservation
across `/data`, and item interaction after ordinary mob interaction.
The standard Launcher downloaded this public commit and built
`native-8438f41-48caff0b4ee8` with the performance profile. Executable SHA256:
`04cd9af73820dc9a808624d936d8cbf85e0eb4d4b7ba8440a2e05ce804aa8e63`.
First, unmodified `native-build` built the local source bundle. Then the standard
Launcher installed published Create commit
`a7d00881d4ac9385e32adfaae89885c67fcd8210` by GitHub URL into a new temporary
instance, `Create GitHub Pipeline 0.6`. Its `native-build --profile performance`
selected the same key and ready binary from the Launcher's shared cache. This
confirms a clean GitHub source-package installation; this second invocation does
not claim recompilation with an empty cache. User installation was not yet complete.

This performance build passed 14 ordinary burner scenarios with Metal:
flint-and-steel/wear, shovel/extinguishing, soul fire without consumption, water
bucket, spawner variants, living-blaze capture, both hands on tool breakage, and
correct wrench loot. A real exit/new process preserved soul-fire state and inventory.
Both native PNGs were viewed; this is not acceptance of pixel equality with Java.

The closed CreateFactory06 source world passed an independent run on a disposable
copy using this performance build: 14 check groups across 12 stations, six actual
crafts through survival menus, operation of an authored schematic, and two verified
exits/new processes with exact inventory/component persistence.
The 31 source-save files and 14 authored schematic files remained unchanged.
Acceptance fixed three fixture problems: ordinary hopper transfer of wheat during
startup, a cow blocking the bucket-use ray, and unloaded client columns near the
fans. Previously checked machines were neither restocked nor repaired. The final
result used two scenario continuations and five process starts, so it does not
claim an uninterrupted first-attempt run. All 16 PNGs were viewed. Overview frames
are limited by render distance; they do not establish the full map's appearance,
audio, or pixel parity with Java. The walkthrough is in [TESTING_06.md](TESTING_06.md).

A separate actual-draw regression on the same build confirmed that the last sheet
disappears from the client depot immediately after ordinary hopper extraction,
before restart and without waiting for another machine. Both frames were viewed.

## Changes after the verified checkpoint

A separate development build using Horizon
`edd52a269fb920d8531abcfa9e37c4d442b34694` and SHA256
`13809ebef2b850ebc9c92cab0b8eac9a0e15aaaa9b94c399a60d27ae782e1036`
passed comparison of eight Mixer tooltips in actual Java/native clients.
Goggles show the source stress: 128 SU at 32 RPM, 64 SU at ±16 RPM,
120 SU at 30 RPM. The insufficient-speed warning remains without goggles;
it is absent without goggles when stopped, frozen, or running fast enough.
Text, inherited styles, font widths, wrapping, position, and border matched.
All eight PNG pairs were viewed. Separate 576 comparisons of source methods with
actual Game providers and CPU widgets cover overload, RPM boundaries, and special
float values; another eight earlier UI scenarios remain as regressions. Overload
is not claimed as a separate new live Java scenario: seeding it through `/data`
reset the original's kinetic state. Full-frame, lighting, animation-phase, and
tutorial-message equality are not claimed.

This is historical Stage9, not the final build: a broader Game run found an extra
contents-change notification during the first automatic expansion of an empty
native inventory. The following snapshot includes the generic fix for that defect.

### Stage10: physical item intake

The immutable development snapshot uses Horizon
`e0002eecd2f53f98423a7a85e6c3adcc130e1b2f`; binary SHA256
`2291da322fdb8b879f96a50574bd9db3626ade162c3430a4f3c51c7323d4c4bc`.
It was built from a local source checkout with official Horizon Mixins,
not installed as a new Launcher version.

It passed 16 ordinary item-physics scenarios compared with actual Create 6.0.10:
all 128 consecutive server snapshots of input inventories, stack remainders,
components, and individual slots matched. Cases checked the Basin center/rim,
refusal when supported by a slab above it, Millstone selection here/below, refusal
by a full first handler, and an unsuitable item. A real exit and second process
preserved both partial transfers: a full input of 64 and a remainder of 12 with a
component. The check uses movement and ordinary commands; it neither calls the
intake adapter directly nor searches for items by AABB.

The initial-empty-inventory fix was separately verified through a real raw-mixin
call with an empty trait registry: the previous snapshot order produces a false
event; the new order does not, while actual insertion/extraction still notifies
the handler. Both client crates passed strict Clippy, formatting, and public-tree
checks for this generic fix. This does not claim lint success for the entire
additionally injected Create build.

Stage10 also fixes the material of five burner OBJ parts and registers the imported
hat models. The actual Game/CPU check passed 14 source/native models and 128 frames:
geometry, UVs, color, light, JSON normals, and NBT were preserved while shading
changed. Two new raw PNG pairs, 05/10, were viewed: OBJ brightness is now close to
Java, and gold and red/white TrainHat details are visible. Background, floor/cage
lighting, and particle/animation phases differ; complete equality of the JSON
head/goggles, materials, all orientations, or the full frame is not claimed.
The detailed evidence boundary is in
[tools/BURNER_PRESENTATION.md](tools/BURNER_PRESENTATION.md).

The working tree after this snapshot was preparing the next separate phase:
a shared Basin recipe cache for Mixer and Press, compacting, and the Press client
cycle driven by actual messages. These changes are not attributed to Stage10.

### Stage11: Press above a Basin

The immutable development snapshot uses Horizon
`9c7f9b2215dfed497aba5522c44f38926628648d`; binary SHA256
`b7a6e07899b4fa1ab59ae0f9ef1e28ab3c32f1d32793c609635bff778e8d40a6`.
Build receipt: `.local/dev-native-drbmbtkf/result.json`. This is a local build with
official Horizon Mixins, not the result of the final Launcher installation.

[tools/press_basin_native_checks.py](tools/press_basin_native_checks.py) recorded
four sequences of 180 ordinary server ticks at the same coordinates and with the
same setup commands as actual Create 6.0.10. The source receipt is
`.local/press-basin-java-l4yhlfdi/result.json`; the native receipt is
`.local/press-basin-native-zquk3sv2/result.json`. Both Basin/Press creation orders
matched final quantities: eight snowballs yielded two snow blocks, and nine snow
blocks yielded one ice. `Running`, `Ticks`, `Mode`, `Finished`, `NeedsUpdate`,
`Speed`, and both item inventories were checked on every tick.

Strict comparison of these 720 snapshots **failed**: all 414 mismatches in the
checked fields/inventories are retained without shifting the sequences. Native
starts the cycle on the first tick (`Speed=32`, `Running=1`, `Ticks=4`, `NeedsUpdate=0`);
on the first tick Java still has `Running=0`, `Ticks=0`, `NeedsUpdate=1` at the
same 32 RPM and starts on the second. Native impacts therefore occur on ticks
31/93 versus Java's 32/94; for ice, 31 versus 32. Repeat impacts are 62 ticks apart
in both clients. Block creation order does not establish their internal tick
order; absolute startup equality is not claimed.

The same complete native run separately passed:

- stopping at zero RPM, operation at −32 RPM, and actual overload of a crank
  driving two presses with zero effective speed;
- preservation of remaining-input components and fresh output items without the
  input stack's component;
- an actual compacting recipe: two flint, gravel, and 100 mB lava yielded andesite
  without a burner; ingredients were fully consumed. This Press rejected the
  Mixer-only alloy recipe without consumption;
- save, exit, and a second process: a pause at `Ticks=200` preserved eight
  snowballs, one finished block, components, an unknown NBT field, and
  `Running`/`Mode`/`Finished`. Resuming yielded exactly three blocks from the
  original twelve snowballs.

Both processes exited normally. The receipt explicitly distinguishes
`completed=true`, `controls_passed=true`, and `passed=false`: successful controls
do not turn the difference from the source sequence into an overall PASS.
Two earlier attempts are retained as fixture errors: incomplete `Level`, then
missing `Fluid` inside `TankContent`; native correctly refused the unsupported
format. The final run uses canonical fluid fields.

An additional observation before the first tick,
`.local/press-basin-prestart-2vwd19k_/result.json`, confirmed both creation orders:
after input loading and motor replacement, native still has
`Speed=0`, `NeedsUpdate=0`, `Running=0`, `Ticks=0`. RPM resolution, notification,
and startup occur on the first ordinary tick. This locates the difference in the
network/operator startup phase but does not establish Java block entities'
internal order. No artificial delay was added; further investigation of the
shared graph phase was left outside this completed run. These checks do not
establish audio, particles, rendering, or whole-client lint success.

### Stage11: Game regression and final delivery

Snapshot `b7a6e078…` passed all six actual prepared-resource Game tests:
shared recipe compiler, Press client with real particle admission, server
Press/Basin lifecycle, Mixer scheduling/reload, and physical item contact.
Receipt `.local/press-basin-adapter-bquz33pp/result.json`; the stage was restored
byte for byte. Corrected fixture errors are retained separately: a missing
ItemStack import after module extraction and a motor facing away from the Press.
Checks were not weakened; explicit confirmation of actual RPM was added.

Portable libraries: 124 Rust tests, 10 Python tests, and workspace strict Clippy
PASS. The complete injected client still reports six previously recorded lint
diagnostics outside the new modules; no overall strict PASS is claimed
(`.local/final-stage-clippy-result.json`).

Another agent checked the new 13-station map on a disposable copy using this
dev binary: 15 groups, six survival crafts, two real restarts, and all 19 PNGs
viewed. All 31 source-world files and 14 schematics remained unchanged. The three
new Basins yielded 2 snow blocks, 1 ice, and 1 andesite; lava went
1000→900 mB, with no duplicate result over 190 ordinary powered ticks.
Receipts: `.local/review06-audit-ao91u2a8/aggregate-result.json` and separate
`image-review.json`. The overview's vanilla recipe toast and the visible fluid
surface geometry from the chosen angle are not claimed as pixel parity.

The standard Launcher built performance client `native-9c7f9b2-b81e43cfc1d2`,
SHA256 `c640101923ecabea1400982c22935008a78d138e771700965b9b3d418d4eb9a3`.
It separately passed station13: three results, ordinary manual pickup, a
900 mB remainder, no duplication, and one real restart with exact inventories/
tanks. Three PNGs were viewed; the audio device opened and source Press events
were recorded. This is not acoustic/pixel parity or a repeat of all 15 dev groups.
Receipt `.local/final-performance-check-ejbjii2j/result.json` and `image-review.json`.
The first performance scenario stopped before machine startup because distant
columns were unloaded; a new copy passed after the camera-position correction.
The complete 15-group dev run is not relabeled as a performance run. A new GitHub
installation was blocked by the anonymous API limit; the verified local package
was compared with downloaded public commit
`d6a82f59e7622d9ad74e5cc200699776fa7e0e66` across all 143 files.
The user's native-build confirmed the ready key without recompilation.
The new map was copied byte for byte; level.dat SHA256
`b6dd3157e24c550459fff61f655c8d8dbbbd1b612f603f405ef9aa78c3f3f99a`.

## Other checkpoint checks

Before the build, separate checks covered actual Mixer datapack reloads, tag
updates without custom mixing recipes, completion of an already started cycle
using the old recipe object, and the correct client-level RNG for particle
requests. Two checks with the fixes disabled failed on the intended assertions.
The defensive Basin comparator preserves an unsupported short native inventory
without changing NBT or identity. Portable libraries: 120 Rust tests and workspace
Clippy `-D warnings`; recipe export: eight Python tests. This scope is separate
from whole-client lint.

The combined development build with SHA256
`a277bbd824ea4ce052fae43c98bdac1b2cc4e56a216253a391d01a0c34ca76bb`
checked:

- canonical Basin/Millstone inventories before the first machine tick, clearing
  both halves, ordinary hopper `3 + 5 = 8`, preservation of components, and full
  typed NBT across a real exit/new process;
- refusal to alter inventories with an unsupported explicit `Size`, including
  ticks, breaking, and wrench use; restoring the original size restores items;
- five actual rendered Basin tooltips: empty, items, items/fluids EN/RU, and no
  goggles. Text, styles, units, font widths, and layout matched; all five PNGs
  were viewed. Basin-content geometry is checked separately; full-frame equality
  is not claimed;
- an independent run on a copy of the new map: 13 machinery groups, six survival
  crafts through ordinary menus, and exact inventory/component persistence across
  restart. The prepared source world's 31 files remained unchanged.

That earlier run was **not yet release acceptance**. It found a sheet left on the
client depot after hopper emptying and water leaking from the review station.
A separate blaze-capture scenario found that a removed mob remained visible to
selectors until the next tick. The depot was fixed by immediately synchronizing
the last HeldItem and passed a separate actual-draw regression without restart;
mob removal was fixed in the runtime and checked through immediate commands/save.
Physical water containment passed separate ordinary fluid ticks. A new source
world with these fixes and the schematic platform was rebuilt and separately
checked on a copy as described above; earlier-map results are retained as history.

The burner and heated brass had already passed separate native checks of fuel,
placement, components, heat loss, `2 copper + 2 zinc → 4 brass`, and a real restart.
For its appearance, 12 Java/native frame pairs were captured: differences in
dynamic-part lighting and goggles visibility were found. Full visual burner
equality is therefore not accepted. Isolated Java oracles check selected numeric
methods but do not replace these observations.

The shared inventory codec and its checks are described in
[tools/INVENTORY_CHECKS.md](tools/INVENTORY_CHECKS.md); burner interactions and
network definitions are described in [tools/BURNER_INTERACTIONS.md](tools/BURNER_INTERACTIONS.md)
and [tools/BURNER_SYNC.md](tools/BURNER_SYNC.md).

The bounds and evidence from earlier intermediate builds are retained below;
they do not mean the entire new set has passed a combined final gate.

## What had been checked

| Area | Verified scope |
|---|---|
| Schematics | Bounded GZIP/typed NBT, modified UTF-8, strict preflight, bounded creative paste; 13 malformed/unsupported files rejected without world changes |
| Shared schematic | Identical file SHA256 `fe9cad9a8fcfa6895e84bce24335adfa7fad3826da7618dc4c26897ebd220d54`: motor, shaft, Press, Depot, and support; paste and one copper sheet in both clients |
| Bare belts | Connector and segments, source shapes, RPM, full-stack transport, output to a Depot/through a hopper, reverse motion, dismantling, and persistence across process close/start |
| Motor settings | Actual native screen: hold, confirm on release, directions, 128/-64 RPM, source zero application, Escape, held item, and removal of the previous wheel shortcut |
| Goggles and motor | Nine scenarios: eight visible EN/RU tooltips and the motor screen at the source camera pose; text, font widths, inherited styles, anchor, and UI geometry matched; both clients' frames were viewed |
| Sounds | Actual audio-engine channels: crank, motor, and cog; three start/stop cycles, disposal on Nether transition, and recovery on return. A separate NO_AUDIO run checks the exact retained-instance count |
| Particles | Actual Metal checks of rotation bursts, recovery, delivery of 800 particles, airflow motion, media, sprite/lifetime/light, and cleanup; numeric observations of actual Java particles were checked separately |
| Fan | Player movement force/direction/limits checked; mobs: fire, resistances, media, and horse conversion. The complete custom player-damage scenario passed on the corrected runtime |
| Press above a belt | Eight ingots yield eight sheets individually; drive stop, unsupported input, independent Press power loss, and a real mid-cycle restart checked |
| Fan above a belt | Stopped belt, washing 17 items, unsupported input, media change, and blasting destruction; a real restart preserves timer, position, and components. The queue of two simultaneously active fans was checked separately |
| Context shapes | Actual ItemEntity and player use different source Basin shapes; hollow Mixer/top-panel selection, an empty default shape with an override, section boundary, and face-only interaction checked, including restart |
| Dry Basin/Mixer | One complete eight-group native run and a real restart passed; a separate ordinary-server harness confirmed source 83/62 ticks |
| Portable logic | 82 Rust mechanic tests, 16 asset-importer tests, and 10 schematic tests pass; independent Java oracles supplement gameplay checks |

The latest development build checked through the standard Launcher at this point:
`native-f6cdb73-56f914c2efb2`, Horizon
`f6cdb73881546165f94bbdc4dc790248018dd701`. It passed nine UI scenarios,
audio lifecycle, and six numeric belt stack-capacity variants.
Earlier `native-cee7e60-a56e3358ca1c` passed the main belt, strict schematics,
save/restart, identical coordinates in two dimensions, dismantling after chunk
unload, and shaft return with a full inventory.
The next development build, `native-b45ffd5-0d0847ae8f31`, was successfully built
by the standard Launcher on Horizon `b45ffd5a670f20e66db7fc01f53fd94907563adc`.
It passed the complete fan/player-damage scenario, numeric item-component
transmission to the client, Press above a belt, and eight dry Basin/Mixer check
groups split between two fresh worlds.

Development build `native-08821fd-e710896f4b86`, Horizon
`08821fdbce6b5d081f1e6cc5c4d0da5438861222`, was also successfully built by the
official Launcher. It passed a complete fan-above-belt run with two processes,
ten context-shape checks, and Press above a belt with early-cycle persistence
across a real restart. A separate complete Press run also checked
20 frozen-world steps: exact Inventory/Running/Ticks/Mode/Finished remained
unchanged with saved 128/32 RPM, while ordinary time continued processing exactly
three sheets. The same behavior was checked in actual Java Create.
This was not yet a combined release regression gate.
One complete Basin/Mixer run on this build passed all eight groups,
a real restart, and a new post-load cycle. A separate built-in
`HORIZON_COMMAND_SCRIPT` harness recorded 200 consecutive ordinary server ticks:
first result after 83 ticks, repeat after 62. In this harness the shared schematic's
geometry was placed with 40 ordinary `setblock` calls; file paste is checked by a
separate gameplay scenario. Ingredients are supplied through canonical NBT here;
this does not prove the Basin's then-unimplemented fallen-item intake.

## Recorded differences and limits of those snapshots

- The ninth UI scenario at the exact Java camera pose found an incorrect motor
  selection/collision shape. The source shape was corrected, verified for six
  directions, and rechecked in the native client. Full pixel parity of every
  interface is not claimed. Invalid old screenshots showing the loading screen
  were excluded from visual evidence.
- Original `create:fan_fire` / `create:fan_lava` were missing from Horizon's player
  damage-type catalog: ignition worked but the airflow hit was skipped. The
  generic world damage-registry/tag loading fix passed the complete Horizon Rust
  run; the new build confirmed an actual `create:fan_lava` hit losing four health.
  The complete scenario passed with natural regeneration disabled: health 20→16,
  source bypass of equipped diamond armor, and no extra damage. Other entities,
  including boats/minecarts/projectiles, were not yet fully covered.
- Audio state machines and lifecycle were checked separately. Subjective accuracy
  of the sound mix and all other Create sounds was not yet confirmed.
- Belt checks across dimensions, with chain parts unloaded, and with overridden
  `max_stack_size` passed. A separate loss of this component during Horizon's
  client inventory synchronization was fixed and verified: six numeric forms and
  explicit component removal survive on the client. Casings, dyes, passengers,
  and funnels/tunnels remain future work.
- Belt rendering did not yet reproduce all legacy item-presentation and source
  client-prediction details. The bounded schematic importer explicitly rejects
  belts; this is not arbitrary Create-schematic compatibility.
- Basin/Mixer: the portable core and Java oracle worked, and the native adapter
  entered the combined build. Checks covered 18 slots, full-component hopper IO,
  both-hand interaction, dry recipes/bucket remainder, the 30 RPM threshold, power
  loss, blocked output, and inventory persistence. In actual Java the same
  40-block schematic file yielded two andesite alloys; after cycle start, the
  first result appeared on ordinary tick 83 and the second another 62 later.
  Exact server timing comparison with Horizon confirmed 83/62 ticks.
  `/tick sprint N`
  advanced N+1 ticks in the tested clients, so the old experimental sprint scenario
  was excluded from exact timing evidence. Fluid filling/rendering, heat, filters,
  side outputs, and fallen-item intake were not yet implemented in that snapshot.
  Unknown data is preserved and rejected. The one-shot `Level.Force` field needs
  the correct serialization boundary. At that point the catalog contained
  26 public blocks, 17 standalone items, and 129 recipes. Registration alone does
  not count as a mechanic.
- Press processing on belts uses source holding/copying and queued transported
  results: separate Java oracles passed 17780 and 1448 cases. Actual Java machinery
  built with the ordinary connector and native machinery both produced eight
  sheets from eight ingots through a Depot and hopper. This checks a bounded
  output, without claiming equality on every tick.
- Fan above a belt has a separate AST oracle of 18500 cases. Its diagnostic frame
  with chat and water obstructing the scene was excluded from visual evidence;
  mechanical checks do not depend on that image.
- The 0.5 limits for canonical `Source`/`Network`, world item IO, cross-version
  item codec, and some collision shapes were retained. Effective `getSpeed` now
  returns 0 during debug-freeze while preserving network RPM; checking all client
  effects in that mode remained future work.

## Upstream and further validation

The port temporarily pins the required APIs in its own Horizon fork.
[PR #8872](https://github.com/horizon-rust/horizon/pull/8872) was opened for named
mob damage and owned particle controllers; the CI checks that ran passed.
A separate [draft PR #8918](https://github.com/horizon-rust/horizon/pull/8918)
adds world text and atlas quads. [Draft PR #8924](https://github.com/horizon-rust/horizon/pull/8924)
adds native-owned sounds. Datapack loading of damage types and tags was submitted
in [PR #9076](https://github.com/horizon-rust/horizon/pull/9076) for
[issue #8917](https://github.com/horizon-rust/horizon/issues/8917);
the maintainer temporarily closed the PR. Review fixes and an upstream-main merge
were pushed to `Delete228/horizon:data-driven-damage`
(`28b9f949ffe1dbff5cd1427f155f28611e8dbced`); revalidation and the reopen request
are handled separately. The closed PR still showed the earlier head at this point.
Pending API fragments receive no invented numbers: an exact source pin and narrow
raw adapter remain in use until then.

The official [tracker #4953](https://github.com/horizon-rust/horizon/issues/4953)
directs new mod-API work to `arismoko/horizon`. Its current API/package format
differs from the released Launcher; no Launcher release for that API was yet
available. The build therefore retained its own exact pin and previous installation
format. Context-dependent block shapes were separately proposed in
[fork issue #516](https://github.com/arismoko/horizon/issues/516).

The next gate was: corrected combined native build → UI/audio/fans/belt boundaries
→ factory Basin/Mixer → full regression → new review map and verified publication.
The module-wide TODO is in [ROADMAP.md](ROADMAP.md). Matching individual tests does
not establish completion of a subsystem or translate into an exact coverage percentage.


Burner and fluid development (the next unfinished phase in this earlier record):

- `[logic ready, integration under test]` Burner: 17 710 comparisons of four unchanged
  Java methods and 13 interactions in the actual Java client. An empty cage creates
  no `create:blaze_heater`; an existing BE survives a property change on the same
  block. The first lava bucket gives 20 000 ticks; ordinary-fuel overflow caps the
  top-up at 10 000.
- `[logic ready, gameplay run pending]` Fan mode selection:
  27 648 combinations of fluid/block tags, lit/flame_type/blaze matched four
  original `isValidAt` methods. A heated burner blasts; a smouldering burner smokes.
- `[checked in a separate build]` Fluid components are preserved in canonical
  `InputTanks`/`OutputTanks`. Nine ordinary bucket interactions, dough mixing, four
  mud portions, a real process restart, and Metal water/lava rendering passed.
  This does not claim pixel equality with Java. The
  [fluid-capability](https://github.com/Delete228/horizon/tree/create-native-fluid-capabilities)
  and [rendering](https://github.com/Delete228/horizon/tree/create-native-fluid-render)
  branches were published after the fork owner's explicit permission. The combined
  burner build was still under test; the previous phase's public pin had not yet
  been replaced.
- `[checked in the engine]` Separate branches were published for
  [state-dependent light](https://github.com/Delete228/horizon/tree/content-state-light)
  and [multiple items for one block](https://github.com/Delete228/horizon/tree/content-item-alias).
  The latter contract preserves the empty burner's source item ID during ordinary
  placement/item consumption, including client prediction. The specific burner
  checks in all directions were part of the next gameplay run.

Catalog registration and local model import alone do not make a completed burner
block. Heat, materials/animation, audio, and placement receive separate checks
before the next world is handed over for visual review.
