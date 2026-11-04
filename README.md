# Create for Horizon — experimental port

**Unofficial experimental port**, maintained by [Delete228](https://github.com/Delete228).
Based on [Create by the Creators of Create](https://github.com/Creators-of-Create/Create).
This project is not an official Create or Horizon release. MIT code and datapack
logic are adapted to Rust; original Create assets are supplied locally by the player.
See [third-party notices](THIRD_PARTY_NOTICES.md).

Branch `stage/parity-06`, version **0.6.0-alpha.2**: bounded schematics,
bare belts, Basin/Mixer, water/lava, burner and heated brass, press over a Basin.
Physical item intake and the packet-driven press client cycle were added.
[Stage report and known differences](REPORT_06.md),
[13-station route](TESTING_06.md), [complete TODO table](ROADMAP.md).
This is not yet the whole of Create; rough overall porting estimate: **8–10%**.

Candidate **0.6.0-alpha.3** fixes belt X rotation and development packaging,
and pins the already accepted Mixins fix. It is built and has passed focused checks,
but the user instance has not yet been updated: a separate macOS Launcher bug
was found that loses built-in mod metadata when starting from `.app`.
The generic fix was tested and submitted in [PR #9529](https://github.com/horizon-rust/horizon/pull/9529);
updating the installed Launcher and cleanly installing the candidate are still pending.
[Candidate report and remaining checks](REPORT_063.md),
[upstream PR explanations and status](UPSTREAM_STATUS.md).
Candidate source checkpoint:
[`8b7cca1`](https://github.com/Delete228/create-horizon/commit/8b7cca1ce486e8b4297a46eabe817d3b7ed1a407).

Current development candidate **0.7.0-alpha.1** adds enclosed chain transmission
and works on source-event timing and effective item-component behavior.
[Development scope, checks and pending native acceptance](DEVELOPMENT_07.md).
The installed review instance has not been replaced with this candidate.

For a reproducible installation of the experimental stage, paste this into **Add from GitHub**:

```text
https://github.com/Delete228/create-horizon/commit/d6a82f59e7622d9ad74e5cc200699776fa7e0e66
```

Version **0.5.0** adds the Millstone, fan processing and stationary drill to kinetics,
the press, Depot, engineer's gauges and water wheels. Its performance build passed
a full gameplay run with Metal, audio and restart; the prepared map was checked
in a separate audit. Evidence and limits are in [VALIDATION.md](VALIDATION.md).
Complete contraptions, trains, Ponder, pipes and other logistics are still pending.
The limits of the new factory foundation are described in the 0.6 report.

- [Horizon Launcher installation and visual checks](LAUNCHER.md).
- [Original resources from your JAR](ASSETS.md).
- [Plan and review checkpoints](PLAN.md), [validation results](VALIDATION.md).
- [Overall TODO table for all Create subsystems](ROADMAP.md).
- [Complete visual checklist for the 0.5 map](TESTING_05.md).
- [Original IDs, data and compatibility limits](COMPATIBILITY.md).

## Foundation in released version 0.5

**24 blocks:** Creative Motor, Shaft, small and large Cogwheels, Mechanical Press,
Depot, Gearbox, Clutch, Gearshift, Hand Crank, Andesite/Brass Encased Shafts,
Andesite/Brass Casings and blocks of Andesite Alloy, Brass and Zinc,
Speedometer, Stressometer, small and large Water Wheels, Millstone, Encased Fan and Mechanical Drill.
**15 standalone items:** four sheet types, Andesite Alloy, Brass and Zinc
Ingots/Nuggets, Wrench, Vertical Gearbox, Engineer's Goggles, Propeller, Wheat Flour
and Dough. There are 39 entries
in the Create tab. The Vertical Gearbox places the original `create:gearbox` block.

Connections include coaxial shafts, small cogwheels, 2:1 large/small cogwheel gearing,
perpendicular large cogwheels and four gearbox ports. The Clutch disconnects its
halves under redstone power; the Gearshift reverses direction. The Hand Crank supplies 32 RPM,
with Shift reversing it. The Wrench rotates machines; Shift removes them and returns
their contents. Casings apply to stripped logs and shafts.

The motor defaults to 16 RPM. Open settings on its side face with an empty hand;
two rows select direction and 1–256 RPM. Scrolling while pointing at it
changes speed by 1 RPM. The value is stored in `ScrollValue`, validated by the server
and saved with the world. The interface is drawn with Horizon's UI tools.

The Depot accepts a stack from a top click and returns it to an empty hand without a menu. The press
processes **one item** per hit, like Create with `bulkPressing=false`:
the remainder stays on top, results collect in eight output slots, and the last
result takes the top position. One click collects the top stack and output buffer.
Hoppers insert/extract items; the Depot accepts dropped items. The press also
processes items lying on a solid block without a Depot.

The Millstone accepts eligible dropped items and hopper input, mills one input
item at a time and stores results in nine output slots. An empty hand collects
finished output first; if output is empty, it returns the input. Drive it with a shaft
from below or meshing cogwheels; its rotation axis is always vertical. Speed and
chance results follow original recipes; the timer and inventories persist.

The fan moves dropped items and processes them or a stack on a Depot.
Water enables washing, fire and lit campfires enable smoking, soul fire and soul campfires
enable haunting, and lava enables blasting. Airflow direction depends on rotation;
obstructions and original tags limit the current. A working chain is wheat → Millstone →
flour → washing → dough → smoking → bread. Smoking/blasting use
actual Minecraft recipes, including Create's added dough recipes.

The stationary drill breaks the block in front and uses ordinary Minecraft loot
tables. Its shaft connects at the rear; speed, target hardness and loss of drive
control the cycle. At the review station, a Clutch keeps the drill off until the lever
is toggled, and a hopper collects drops into a chest. Automatic block replacement,
drills on moving contraptions and entity damage are outside this stage.

Right-click equips the goggles; pointing at machines shows RPM and network stress.
The equipped model uses Horizon's standard `head` context. Gauges have
smoothed needles and original comparator output. The small wheel gives 8 RPM / 256 SU,
the large wheel 4 RPM / 512 SU; the sign depends on water flow. Planks change the material
without being consumed. A large wheel reserves a 3×3 plane; breaking a structural part
dismantles the whole wheel. Press/overstress particles and initial connection hints were added.
The HUD and hints still use port-native presentation; this is not a finished Create interface.

There are **123 original recipes** and 35 required tags: 37 crafting recipes, four
pressing recipes, four item applications, 47 milling recipes, ten washing recipes,
19 haunting recipes and two bread cooking recipes. Zinc generation
and brass mixing are not yet implemented: these materials are available creatively for now.
Original models, textures, 61 languages and the sound bank are imported from the supported
local JAR. Placeholder resources are used without it.

## Installation and building

Create an instance in Horizon Launcher, choose **Mods → Add from GitHub** and paste
`https://github.com/Delete228/create-horizon`. Build native mods, place the JAR
as [instructed](ASSETS.md), and press Play. No separate content archive is required.

From a working copy, with the official Launcher installed and an instance already created:

```sh
python3 tools/launcher.py --instance "Create Native" --github https://github.com/Delete228/create-horizon --profile performance
```

To rebuild the installed package, omit `--github`. For development, use `--source`.
The Launcher manages the compiler, Horizon source, Mixins and caches. Pinned versions
are in `compatibility.json`; updating Horizon requires revalidating the adapter.
No manual patching of the original Horizon is needed. The published Mixins fix
for `recursion_limit` is used; the next alpha.3 pins the already accepted
fix from [PR #25](https://github.com/horizon-rust/horizon-mixins/pull/25).
Installed alpha.2 retains its previously verified revision; it is a historical release.

## Development and validation

`mixins/create/engine` contains independent Rust logic. `runtime.rs` and neighboring
modules connect it to Horizon's server items, network, NBT, sounds and rendering.
The port aims for matching observable behavior and efficient operation in Horizon.
Rust may combine or split original Java classes; one-to-one class mapping
is not required. Recipes and eligible inputs are cached and
refreshed when server data loads.
Named hooks are preferred; pinned raw mixins cover content/resource loading,
client interaction results, insertion restrictions and reset of transported
state after automation extracts items. A separate opt-in hook
observes actual rendering matrices in tests.
`mixins/create/assets` is the Rust JAR importer. `content/` holds public declarations
and placeholder resources; `tools/catalog.py` defines only the implemented set.

```sh
python3 tools/bootstrap.py
python3 tools/cargo.py test --locked --workspace
python3 tools/cargo.py clippy --locked --workspace --all-targets -- -D warnings
python3 tools/check_distribution.py
python3 tools/oracle.py
python3 tools/placement_oracle.py
python3 tools/rotation_oracle.py
python3 tools/processing_oracle.py
python3 tools/visual_oracle.py
python3 tools/milling_oracle.py
python3 tools/fan_oracle.py
python3 tools/drilling_oracle.py
```

Oracles require a JDK and Create source in `../Create` or `CREATE_SOURCE`.
They extract methods from the pinned release with the JDK AST without switching the
working copy. This compares algorithms with context adapters, not all of NeoForge.
The next validation level uses identical test schematics in original Create and
Horizon: identical inputs and actions, comparing outputs, timing, directions and
saved state. The development branch already verifies bounded import
and the same result from one shared schematic; a broad mechanism set and exact
timing comparisons are still pending. This is not a promise of world interchange.
`tools/smoke.py --help` describes actual gameplay checks through `horizon-ctl`;
menu and rendering checks require `--offscreen`. They use disposable worlds.
`tools/content.py` updates public files from the catalog and MIT recipe data;
`tools/reference_recipes.py` reproduces the export from pinned Create.

The initial `horizon-port-pack` import/inventory/carry/report/check is complete:
8913 files are marked `partial`. Its complete import remains in `.local/` and does
not register hundreds of nonfunctional machines. JAR Java code is not executed.
Low-level builds/packages require a separate Horizon checkout at the pinned
commit, `tools/stage.py` and `tools/package.py`; normal installation does not.

## Limits of the latest 0.5 release (historical)

The network is limited to 4096 loaded nodes; components are cached per game tick.
Breaking on conflict/overspeed does not yet match Create: the component
stops. Large networks, chunk unloading and multiple network clients need
separate runs. Water, exact particles/animations/soundscapes and the original
motor interface are unfinished. Special recipe schemas, Basins and belts are absent.
The fan currently affects only dropped items and Depots: airflow movement of players
and mobs is unimplemented. Original dropped-item fields
`NeoForgeData.CreateData.Processing.Type/Time` persist through Horizon entity
persistent data; timer continuation and preservation of unknown sibling keys
were checked with an actual world restart.
Depot progress is stored in transported-item data. Exact effects,
sound and all fan interaction variants still need verification.
Identifiers follow Create, but Java 1.21.1 ↔ Horizon 26.3 world interchange
is not claimed. Item availability and passing tests do not mean the entire mod is ready.

Official guides: [Native mods](https://github.com/horizon-rust/horizon/blob/main/docs/native-mods.md),
[Mods](https://github.com/horizon-rust/horizon/blob/main/docs/mods.md),
[archive formats](https://github.com/horizon-rust/horizon/blob/main/docs/api/mod-files.md).
Registry submission is described in [REGISTRY_READINESS.md](REGISTRY_READINESS.md).
