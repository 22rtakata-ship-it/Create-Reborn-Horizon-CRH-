> The source candidate is **0.6.0-alpha.3**, with accepted Mixins
> `e8cd0aa38cffef49cb2e088ef716baa107148c02` ([PR #25](https://github.com/horizon-rust/horizon-mixins/pull/25)).
> The alpha.2 instructions and receipts below refer to the installed version.
> The package and developer client are verified; a clean candidate installation and normal Play
> on macOS are not yet accepted. [Identified cause and next check](REPORT_063.md).
> The old instance was not replaced.

The verified candidate source package is
[`8b7cca1`](https://github.com/Delete228/create-horizon/commit/8b7cca1ce486e8b4297a46eabe817d3b7ed1a407).
This is a reproducible checkpoint link; it does not mean a new macOS Play run
has passed acceptance. Use the details below for the currently installed map.
The macOS bundle fix was submitted in [PR #9529](https://github.com/horizon-rust/horizon/pull/9529).
It changes the Launcher itself: changing the mod's Horizon pin does not update it.

# Current stage: Create Factory 0.6

A separate instance, **Create Factory 0.6**, and world, **CreateFactory06**, are prepared,
with mod **0.6.0-alpha.2**, performance build `native-9c7f9b2-b81e43cfc1d2`.
Select this instance, press **Play** and open **CreateFactory06**. If the
Launcher window still shows the old list, restart it. The 13-station route
is in [TESTING_06.md](TESTING_06.md); results and limits are in
[REPORT_06.md](REPORT_06.md). Older instances and worlds are preserved.

The new map is at `instances/create-factory-0-6/saves/CreateFactory06/`
inside the normal Horizon folder. Authored schematic files are in `schematics/`
in the same instance. The new press-over-Basin station is at `68 64 27`.

Stage source package:

```text
https://github.com/Delete228/create-horizon/commit/d6a82f59e7622d9ad74e5cc200699776fa7e0e66
```

For the next clean installation, create a new instance and add this link
through **Mods → Add from GitHub**, then build native mods and place your
supported Create JAR in `horizon-mods/create-resources/`. From the console:

```sh
horizon-launcher mods add-github https://github.com/Delete228/create-horizon/commit/d6a82f59e7622d9ad74e5cc200699776fa7e0e66 --instance "Create Factory 0.6" --trust-native-code
horizon-launcher native-build --instance "Create Factory 0.6" --profile performance --trust-native-code
```

In this installation, `add-github` hit the anonymous GitHub API rate limit. A
verified local `.hnmod` was therefore installed: all 143 files were compared byte
for byte with the downloaded public commit. The standard Launcher built a performance client,
and its `native-build` reported **Up to date** in the user instance.
GitHub origin metadata was not fabricated. A fresh alpha.2 installation specifically
through `add-github` remains unverified; it passed for the previous alpha.1.

This version requires the pinned Horizon fork
`9c7f9b2215dfed497aba5522c44f38926628648d` and Mixins
`a1e13326306c6bc074b11e6a4926deb551c4083e`. The Launcher obtains them automatically;
compatibility with any future Nightly is not promised. Fork source and the package
are published; original game resources remain only on the local computer.

Instructions and verified identifiers for the previous release are preserved below.

# Historical 0.5 installation and validation

Version 0.5 adds the Millstone, item processing by fan airflow and the stationary
drill. The verified performance build is `native-fc02103-722bc455fcbe`.
A full run with Metal, audio and restart passed without skips; a separate audit
checked the prepared map. Instance **Create Processing 0.5** and world
**CreateProcessing05** are prepared. The full review route is in [TESTING_05.md](TESTING_05.md),
with precise evidence in [VALIDATION.md](VALIDATION.md).

Create a separate instance, such as **Create Processing 0.5**, and a new world for
testing. Keep **Create Engineering 0.4**, world **CreateEngineering04**, and earlier
worlds for comparison. Repository GitHub URL:
`https://github.com/Delete228/create-horizon`. The Launcher downloads the source package
and builds a native client for the selected instance.

## New instance

1. In the official Horizon Launcher, click **+** beside **Instances**, create
   a normal Horizon instance and select it on the left.
2. Prepare a normal Horizon version with Play if Minecraft 26.3 resources
   have not yet been prepared. An already prepared version can be used.
3. Open **Mods → Add from GitHub**, paste
   `https://github.com/Delete228/create-horizon` and install mod `create`.
   The Launcher shows the exact commit and asks you to trust native code. Check the package
   version: **0.5.0**. For a reproducible source package, you can paste
   `https://github.com/Delete228/create-horizon/commit/16c5d454b1b9860da01b78474ea5206f0664d384`.
4. Start the native mod build. The first build obtains the compiler, source
   and dependencies; later builds reuse the cache. A separate `native-fc02103-…`
   version is assigned to the selected instance.
5. Open the instance folder and place one original **Create 6.0.10,
   NeoForge, Minecraft 1.21.1** JAR in `horizon-mods/create-resources/`.
   [Supported file and SHA-256](ASSETS.md).
6. Press **Play**. The mod prepares public content and imports original
   resources itself. No separate content archive needs to be copied.

Without the supported JAR, placeholder models are used; the reason is written
to the log. The GitHub package contains no original models, textures or sounds.
Java code from the JAR is not executed.

If a build or instance was added through the console while the Launcher window
was open, restart that window: the UI may retain the old build list.
Check the selected instance, mod version and Native marker on the launch page.
For a new review setup, `tools/review_world.py` creates world
`CreateProcessing05` by default; the script does not open an existing world of that name.
The new Millstone, four-current and drill area is at X=32…48; earlier
0.4 stations are preserved. The inventory holds 36 entries, with the other three
in an ordinary barrel beside the new materials; goggles are equipped separately.

## Building from the console

The official Launcher and an already created instance are required. A mod installed
from GitHub builds without neighboring repositories or a separate Rust installation from this working copy:

```sh
horizon-launcher mods add-github https://github.com/Delete228/create-horizon --instance "Create Processing 0.5" --trust-native-code
horizon-launcher native-build --instance "Create Processing 0.5" --profile dev --trust-native-code
```

On macOS, the binary is at
`/Applications/Horizon.app/Contents/MacOS/horizon-launcher` and may not be
in PATH. The helper invokes the same commands from a working copy:

```sh
python3 tools/launcher.py --instance "Create Processing 0.5" --github https://github.com/Delete228/create-horizon
```

To rebuild, omit `--github`. For development from a local working copy:

```sh
python3 tools/launcher.py --instance "Create Processing 0.5" --source --profile performance
```

For `--source`, Create must be installed and enabled in that instance.
The helper passes only the public native package, excluding `.git`, caches, JARs
and prepared Minecraft resources. This validates the working copy, not a fresh
GitHub installation of the published commit.

`dev` is convenient for frequent iterations: dependencies are optimized, while game
code builds faster. `performance` optimizes game code; its first build takes longer.
Normal builds do not disable audio or controllers. Automated runs may use
`HORIZON_NO_AUDIO=1`; audible playback needs a separate run.

Earlier installation checks reached the anonymous GitHub API rate limit.
An existing cache or local package allowed native-build to continue, but that route
does not prove a fresh GitHub download. If the Launcher reports this limit,
keep its message and repeat normal `mods add-github` once access recovers.
Do not manually fabricate an origin receipt. The history and limits of
completed checks are preserved in `VALIDATION.md`.
For 0.5, fresh installation through an exact `/commit/…` link passed in both an empty test
instance and the prepared user instance; the normal repository link
hit this limit in the same session.

## Pinned versions

- Horizon: `fc02103818c212920fd24aa34a2b2f91ba49e56f`, Minecraft 26.3, API 359.
- Mixins: `a1e13326306c6bc074b11e6a4926deb551c4083e`, published commit from
  [PR #26](https://github.com/horizon-rust/horizon-mixins/pull/26).
- Original Create: 6.0.10 for Minecraft 1.21.1,
  `ac0c444d9828da3453ae8cc65338e8de063286fb`.
- Machine-readable settings: [compatibility.json](compatibility.json) and
  [horizon-native.json](horizon-native.json).

The Launcher obtains the Mixins commit from the official Git repository. No manual
changes to the tool or engine source are required. After pinned versions change,
the adapter and all raw mixins are checked again; another Horizon Nightly
is not automatically considered compatible.

## Content inside the native package

`mixins/create/data/content.json` contains public declarations, our placeholder
JSON models/labels, MIT recipes and tags, and the manifest. `native_data!` embeds it
in the client. Before content loads, the mod prepares
`horizon-mods/.create-native-content/` inside the instance. This folder is rebuilt
from the current build; do not edit it manually. Other mods and enable/disable
settings are preserved. One `mod.create` flag enables both code and content.

Version 0.5 contains 24 blocks, 15 standalone items, 123 recipes and 35 tags:
39 entries in the Create tab. The local importer creates 324 resource files and
uses cache schema 7. The source package, original JAR and private cache serve
different purposes; the JAR and cache are not included in the published mod.

## Visual and gameplay checks for 0.5

This is an action checklist for a new build, not a report of completed checks.

1. Open the Create tab. Inspect the Millstone, fan, drill, Propeller, Flour and Dough
   in the inventory, hand and world. With the JAR, original models and languages should be used.
2. Build a Millstone driven from below or through a cogwheel. Drop wheat on top
   or feed it through a hopper. Compare inner-part rotation and processing
   startup/stopping as RPM changes. Flour and seed outputs are probabilistic.
3. Collect finished Millstone output with an empty hand, then the input remainder when
   output is empty. Check hoppers, full output, Wrench removal and ordinary breaking.
   Save during processing and check inventories/timer after loading.
4. Place a fan in all six directions. The shaft connects at the rear;
   reversal switches between pulling and pushing dropped items. Check
   Wrench rotation, solid obstructions and grilles allowed by original tags.
5. Blow through water onto flour on a Depot to make dough. Blow through a
   lit ordinary campfire onto dough to make bread. Repeat with dropped items;
   stacks up to 16 and larger than 16 take different processing times.
6. Check sand haunting through soul fire and cobblestone blasting
   through lava. Blasting can destroy unsuitable items and food intended for smoking;
   do not use valuable stacks for the first trial.
7. Check result collection and progress persistence on Depots and dropped
   items through a world restart. The new persistence route for
   `NeoForgeData.CreateData.Processing` fields is checked in a separate scenario with
   an actual restart and exact continuation of the remaining time.
   Airflow does not yet affect players or mobs.
8. Toggle the lever above the drill's Clutch: the stone in front should break and drops
   should enter the chest through a hopper. The chest holds spare stone/cobblestone for manual
   target replacement. Stop the drive mid-cycle and check continuation.
9. Repeat the main checks from earlier stages: individual press hits and
   Depot image clearing, Clutch/Gearshift, Hand Crank with Shift reversal, motor settings,
   goggles/gauges, and small/large Water Wheels.

Native HUDs, hints, sounds and effects still need comparison with the original.
Belts, Mixer, fan attachment, moving contraptions and Ponder are outside
this iteration. Data limits are described in [COMPATIBILITY.md](COMPATIBILITY.md).

Official guides: [native mods and GitHub](https://github.com/horizon-rust/horizon/blob/main/docs/native-mods.md),
[Launcher commands](https://github.com/horizon-rust/horizon/blob/main/docs/launcher-commands.md),
[instances](https://github.com/horizon-rust/horizon/blob/main/launcher/README.md).
