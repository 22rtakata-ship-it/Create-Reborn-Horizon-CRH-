# Stage report: 0.6.0-alpha.2

The current stage is complete; development is paused for the owner's visual
review. This is an experimental factory foundation, not all of Create.
The rough overall porting estimate is **8–10%**; it does not replace acceptance
of individual mechanics. The overall readiness table is [ROADMAP.md](ROADMAP.md).

## Changes since 0.5

- Bounded creative paste of original `.nbt`, strictly rejecting unknown blocks/data
  before writing. An authored press schematic was tested in Java and Horizon.
- Bare belts with items, connector, Depot/hopper output and press/fan
  processing. Complete belt automation is not yet implemented.
- Basin, Mixer, original inventories and bounded recipes; water/lava, buckets,
  dough, mud, burner, fuel and heated brass.
- Press over Basin: ordinary square compacting and four original compacting recipes,
  including ice and stone types with lava. The recipe cache is shared with the Mixer;
  operator pools are separate, and an active cycle retains its recipe through `/reload`.
- Ordinary dropped-item intake into the Basin/Millstone through Horizon physics;
  preservation of components, partial remainders and canonical inventories.
- Press client cycle driven by actual updates: pause at the bottom, interpolation,
  WORLD/BELT/BASIN modes and original item-particle request order. One-shot
  particle data is separate from saved data.
- Basin/Mixer tooltips checked against Java; burner OBJ part materials,
  hat registration and a spurious first-empty-native-inventory event fixed.

The catalog contains **27 public blocks, 20 standalone items, 137 recipes**.
Original models, textures, sounds and translations come only from the supported
local JAR. They are not published on GitHub.

## Validation and evidence boundaries

| Check | Result |
|---|---|
| Portable Rust libraries | 124 tests, PASS |
| Recipe export/unsupported-recipe refusal | 10 Python tests, PASS |
| Workspace Clippy `-D warnings`, formatting, package contract | PASS |
| Compacting predicate against unchanged MIT Java methods | 3015 bounded contexts, PASS |
| Press client against original Java methods | 6034 bounded contexts, PASS |
| Actual Game: recipes, press server/client, mixer schedule/reload, item contact | 6 resource-backed integration tests, PASS |
| Physical Basin/Millstone intake against Java | 16 scenes, 128 sequential snapshots, components and actual restart, PASS |
| Press over Basin, strict absolute timing | 4×180 ticks: startup one tick earlier than Java; full equality FAIL |
| Press: quantities, stopping, negative RPM, overstress, lava, components, restart | PASS |
| Burner material | 14 models/128 CPU frames, two image pairs inspected; full pixel parity not accepted |
| Independent review-world dev-copy run | 15 groups, 6 crafts, 2 restarts, 19 images, PASS |
| Final performance client | 3 new stations, manual collection and restart, PASS; separate scope |
| Full injected client: strict Clippy | 6 previously recorded injected-client diagnostics; overall PASS not claimed |

The new press over a Basin hits on native ticks **31/93**, versus **32/94** in Java.
The repeated interval of 62 ticks and final quantities match. All 414 field differences
are preserved without shifting time. Before the first tick, neither inputs nor the motor
have set `NeedsUpdate`; the difference relates to kinetic resolution
and machine startup ordering. The exact internal cause is not yet confirmed by a separate
Java observation; no artificial delay was added.

Isolated oracles verify selected methods in the stated contexts.
They do not prove the entire gameplay experience, audio, network lifecycle or frame parity.
Detailed history and limits are in [DEVELOPMENT_06.md](DEVELOPMENT_06.md).

## Review world and installation

A separate instance **Create Factory 0.6**, mod **0.6.0-alpha.2**,
world **CreateFactory06**, performance build **native-9c7f9b2-b81e43cfc1d2** is installed.
The Launcher was restarted and shows this instance and the intended version. Press
**Play**, then open **CreateFactory06**. Older worlds and instances are preserved.

The 13 stations were checked by another agent on a separate dev copy: **15 groups PASS**,
six survival crafts and two actual restarts. All 19 PNGs were inspected.
Three new stations and another actual restart then passed on the final
performance binary with Metal and audio enabled; its three PNGs were inspected.
This is a separate, bounded performance check, not a repeat of all 15 groups.
It proves neither matching sound nor pixel parity with Java.
The first performance scenario stopped before machine startup because distant client
columns were unloaded; the camera position was corrected, and a new copy passed.

All 31 source-map files and 14 authored schematic files remained byte-identical;
that untouched source was copied into the user instance. Processing motors
are off; ingredients and lava are waiting for you to start them.

This alpha.2 `add-github` installation was blocked by the anonymous API rate limit.
The verified local native archive was compared with the downloaded public commit
across all 143 files; the standard Launcher built the client and confirmed the ready
key in the user instance. A fresh successful installation specifically through GitHub
is not claimed. Source package:
[d6a82f5](https://github.com/Delete228/create-horizon/commit/d6a82f59e7622d9ad74e5cc200699776fa7e0e66).

Client SHA256: `c640101923ecabea1400982c22935008a78d138e771700965b9b3d418d4eb9a3`.
Source `level.dat` SHA256: `b6dd3157e24c550459fff61f655c8d8dbbbd1b612f603f405ef9aa78c3f3f99a`.

Route: [TESTING_06.md](TESTING_06.md). At station 13 (`68 64 27`), set
three motors to 32 RPM: obtain two snow blocks, one ice and one andesite;
the third Basin retains 900 mB lava. Collect with an empty hand.

Create reference: `ac0c444d9828da3453ae8cc65338e8de063286fb`, 6.0.10 / MC1.21.1.
Horizon: `9c7f9b2215dfed497aba5522c44f38926628648d`, MC26.3,
branch [create-port-runtime-06](https://github.com/Delete228/horizon/tree/create-port-runtime-06).
Mixins: `a1e13326306c6bc074b11e6a4926deb551c4083e`.
The client builds through the standard Launcher but currently requires a pinned Horizon fork
with generic API fixes. Compatibility with arbitrary new Nightlies and acceptance
into the official mod registry are not claimed.

## Remaining work

Complete belts and logistics, pipes and tanks, other machines, moving
contraptions, trains, Ponder and complete schematic tools are unfinished.
Differences remain in lighting, burner goggles, effect phases, some UI and
CJK line wrapping. Not all sound, particle and network edge cases are covered.
World compatibility across Java/Horizon versions is not promised; original IDs
and data structure are preserved within the implemented limits. Sable and Aeronautics
have not yet been ported.

[Horizon PR #9076](https://github.com/horizon-rust/horizon/pull/9076) concerns the
generic damage-type catalog needed by the fan, among other uses. Fixes on
`data-driven-damage` were submitted, the description and results updated, and reopening
requested. The PR remains closed pending the maintainers' decision; merging
is not claimed. Nine selected vanilla harnesses passed 69,531 comparisons,
but the overall upstream self-check has separately documented pre-existing failures.
