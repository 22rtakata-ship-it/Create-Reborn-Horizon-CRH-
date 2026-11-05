# Checks and evidence

## 0.8.0-alpha.3 moving cargo and sails

The original Create 6.0.10 client and Horizon agree on slot order and rotated
landing for the single mounted chest. Horizon also preserved item components
through a real restart, recaptured the chest after landing, recovered it after
an obstructed controller removal, and collected Saw and Drill drops. Full
inventory overflow and loot from a neighboring column drop into the world.

The sail fixture matches the Java dye and shear interactions and the eight-sail
windmill's 1 RPM. The axial sail stays outside the assembly. Both native
resource modes passed assembly, process restart and restoration. A separate
offscreen review world checks the original sail meshes and the ordinary
closed chest on the moving structure.

The player interaction fixture covers Survival slime consumption, Creative
slime retention, wrenching the retracted piston, refusal while extended and
teardown through a real left click on a pole. It also detects a server crash
when an invalid wrench click reaches the Depot handler. The translated
full-block push covers unobstructed contact. The blocked push still differs
from the sideways displacement in the Java fixture.

Four Java cart-assembler scenes and 722 bitwise coupling comparisons cover
the new cart engine. Cart behavior is not connected to Horizon in this build.
The workspace passed 232 Rust tests, 14 Python tests, strict Clippy and official
native archive validation.

## 0.8.0-alpha.2 contraptions

The native build was checked against Create 6.0.10 on three Mechanical
Bearing axes, three Clockwork modes, regular and sticky piston rigs,
a Rope Pulley with dry cargo, and Gantry Carriage motion and redstone stops.
The comparisons include 120 pairs of hand angles, 176 piston samples,
48 pulley ticks with 80 block observations, and 273 Gantry assertions.

Separate Horizon runs cover saving a moving structure, removing its driver,
retrying an obstructed placement and restoring the payload once. Offscreen
checks cover items on moving surfaces, rotary and linear player transfer,
and clipping against a wall. Native actor fixtures exercise crop loot and
replanting, tree felling, drilling and resumed motion. A separate run matches
32 ticks of stationary horizontal Saw progress.

The review world includes native stations with original local resources. The
native bundle also works with the included models. Read the supported scope
and the in-game route before extending the fixtures.

[Contraption checks](docs/contraptions.md) · [Review route](docs/reviews/0.8-contraption-world.md)

## Belt motion and Depot pressing

Six identical connector-built lanes cover east and south orientation with
level, upward and downward belts. The real Java and Horizon captures cover
forward, stopped and reversed drive. RPM, item step and side offset agreed in
16 of 18 phases. In the south-facing horizontal lane, Java retained +32 RPM
through a cyclic source reference while Horizon stopped and reversed. Item
shadows and continuous texture phase remain open.

Two native rebuild checks cover a level belt replaced with an upward belt and
a full replacement with identical block states. The new load stays on the
replacement and the old load drops once. The real press, belt, Depot, hopper
and chest route also passed, including its freeze regression.

The Depot now slides the held item before delivering it to the same press
callbacks used by belts. In the source comparison, the first ten Depot pose
records and all 101 Press lifecycle samples matched Java without aligning
the clocks. The two cycles began at ticks 9 and 28 and produced two sheets.
This fixture uses a single copper-ingot stack at the edge of the Depot.

The Basin/Mixer retained its 83-tick first cycle and 62-tick repeat cycle.
Eight Mixer HUD scenes passed their text, style and layout comparisons.
The source sound-method oracle passed 2,776 cases. The latter measures
parameters and lifecycle rather than audio output.

[Review world and remaining limits](docs/reviews/belt-motion.md)

## Belt connector preview

The same right-click sequence in Create 6.0.10 and Horizon produced green dust
between compatible shafts and red dust when the second shaft had a different
axis. The incompatible click left both shafts intact. The compatible click
formed the five belt segments. Horizon's client trace verified that emission
starts after the first shaft is selected and ends after a valid connection.
Both clients were rendered and the captures were inspected.

The real native press/belt integration check passed all five groups without
the original JAR. The workspace passed 133 Rust tests, 14 Python tests and
strict Clippy. The [belt review route](docs/reviews/belt-preview.md) describes
the delivered scene and its boundaries.

## Chain gearshift and copper casing in 0.8.0-alpha.1

- `tools/adjustable_chain_checks.py` compared 36 settled cases in the pinned
  Java client and the native client, including all 16 signal strengths from
  each end of a mixed chain, changes while the motor is running and power
  removal. An actual Horizon restart retained `Signal`, block states and RPM.
- `tools/copper_casing_checks.py` compared seven real player interactions,
  including stripped bamboo, a stem and hyphae, Creative item retention and
  rejection of an unstripped log. Converted blocks and copper counts matched.
- Both blocks rendered in separate Horizon review worlds with and without
  the player-supplied JAR. A real lever click changed the left rig's output
  from 32 to 64 RPM. The screenshots cover model loading and the powered
  appearance. They do not measure animation or lighting parity.
- 132 Rust tests, 14 Python tests, strict workspace Clippy and official
  native archive validation passed.

Native build SHA-256
`d7fcf9df49c7729afca036e06dc8f5efe0b8c24636d886ef9493e040e9a64fa4`.
The `create-0.8.0-alpha.1.hnmod` archive SHA-256 is
`5c4d4eab4a0e4f3a32f5bfa4a04dd422feb7df518943a0ee0fcf800515e5d6f8`.
The reviewed mechanics and known differences are in the
[0.8 review world guide](docs/reviews/0.8-test-world.md).

## Optional resources in 0.7.0-alpha.2

A disposable Horizon world was opened three times with the same native build.
On the first launch without the JAR, a powered belt, both wheel sizes, a gauge
and the press rendered with built-in models. The press turned one Copper Ingot
into one Copper Sheet. Adding the supported JAR switched the models to the
original art. Removing it switched the models back even with the import cache
present. The sheet remained in the Depot across both restarts. The built-in
burner uses vanilla fire while the atlas may still log the optional original
flame sprites as missing.

Native build SHA-256
`dd4786638fd3a943256519b9c4e7818f8c6fdc88b2506e3c1b0dbc32c659a352`.
The matching `create-0.7.0-alpha.2.hnmod` passed the pinned Horizon archive and
registry release checks. Archive SHA-256
`13638c5571ef7ac32be5bb19ddac90959796d6c102122de09f6a81d39be74d97`.
All 131 Rust tests, 14 Python tests and portable strict Clippy passed.

## Launcher review for 0.6.0-alpha.2

Source bundle `d6a82f59e7622d9ad74e5cc200699776fa7e0e66`; Horizon
`9c7f9b2215dfed497aba5522c44f38926628648d`; official Mixins
`a1e13326306c6bc074b11e6a4926deb551c4083e`. The unmodified Launcher built
`native-9c7f9b2-b81e43cfc1d2` with the performance profile. Executable SHA256:
`c640101923ecabea1400982c22935008a78d138e771700965b9b3d418d4eb9a3`.

Installed **Create Factory 0.6**, world **CreateFactory06**. The GitHub
installation attempt hit the anonymous API limit. The validated local native
archive was compared byte for byte with all 143 corresponding files in the
public commit; no GitHub origin receipt was fabricated. The user instance's
official native-build confirmed Up to date. This is not a fresh successful
alpha.2 add-github installation. Previous instances and saves were preserved.

- Portable Rust: 124 tests; recipe exporter: 10 Python tests; workspace strict
  Clippy, formatting, package contracts and pinned official archive checks PASS.
- Shared compression predicate: 3015 pinned MIT source contexts PASS. Client
  PressingBehaviour oracle: 6034 contexts / 13 original methods PASS, scoped
  authored contexts and request traces, not full original gameplay.
- Complete woven Game: 6 prepared-resource tests PASS, including actual recipe
  IO/reload, press client receipt/RNG, press server lifecycle and previous
  item-contact/Mixer regressions. Stage restored byte for byte.
- Real item physics: 16 Java scenes / 128 native snapshots match, components,
  partial transfer and a real restart PASS.
- Press/Basin:four 180-tick original/native scenes have exact final outputs,
  controls and a true cold restart PASS. Strict absolute equality FAIL: native
  starts one tick earlier (impacts 31/93 vs 32/94; repeat interval 62). All 414
  differing fields remain unshifted. No artificial startup delay was added.
- New source world independently checked on a disposable dev-binary clone:
  15 groups, 6 survival crafts, 2 verified process restarts PASS; all 19 PNGs viewed.
  Source 31 save files and 14 schematic fixtures unchanged.
- Final performance binary independently opens another copy: 3 new rigs,
  ordinary pickup, native captures, audio-enabled device/events and a real
  process restart PASS. Three PNGs viewed. This does not relabel the full
  15-group dev audit as a performance audit or establish Java acoustic/pixel parity.
- Full injected production client strict Clippy still reports 6 previously
  recorded diagnostics outside the new modules; a whole-client green claim
  is intentionally absent.

The original world was copied unchanged to the user instance; level.dat SHA256
`b6dd3157e24c550459fff61f655c8d8dbbbd1b612f603f405ef9aa78c3f3f99a`.
Evidence: `.local/final-port-checks.json`,
`.local/press-basin-adapter-bquz33pp/result.json`,
`.local/press-basin-native-zquk3sv2/result.json`,
`.local/review06-audit-ao91u2a8/aggregate-result.json` and separate image review,
`.local/final-performance-check-ejbjii2j/result.json` and separate image review,
`.local/final-user-install-result.json`. Failed fixture attempts are retained.

The route and known differences are in [TESTING_06.md](https://github.com/Delete228/create-horizon/blob/eb8b96e8ea1db698b79bb61fd1caf6c963f31652/TESTING_06.md),
[REPORT_06.md](https://github.com/Delete228/create-horizon/blob/eb8b96e8ea1db698b79bb61fd1caf6c963f31652/REPORT_06.md) and [DEVELOPMENT_06.md](https://github.com/Delete228/create-horizon/blob/eb8b96e8ea1db698b79bb61fd1caf6c963f31652/DEVELOPMENT_06.md).
The entries below describe earlier binaries and are not silently transferred
to this checkpoint.


## Production machinery 0.5.0 (4 November 2026)

The pinned Horizon API 359 / Minecraft 26.3 and Create 6.0.10 are unchanged.
Official performance build: `native-fc02103-722bc455fcbe`.

- **56 Rust tests**: 41 for independent gameplay logic and 15 for assets.
  Formatting, Clippy with warnings denied, and source-package contracts passed.
- **157 204 Java comparisons** across eight oracles: the previous 75 145,
  millstone 24 409, fan 17 680, and stationary drill 39 970. The fan oracle also
  checks 566 956 timer transitions. Selected method bodies are extracted through
  the JDK AST; world/recipe contexts use explicitly bounded test adapters.
- The native millstone/fan run passed:
  `.local/smoke-omefnu7i/result.json`. It checked input filtering, components,
  hoppers, probabilistic outputs, full-output blocking, rotation transmission,
  four media, dropped items, obstructions, sign changes, and a real restart.
- Independent fan audit: `.local/smoke-audit05-comjxyyz/audit-result.json`
  and `.local/smoke-audit05-an8iwh2i/audit-result.json`. All six input faces,
  reversed media order, obstructions/slabs, fresh progress after reinsertion,
  exact 153/303 calls for 16/17 items, and hopper extraction/filling within one
  tick without carrying over the old timer.
- Dropped-item persistence was checked with a real restart:
  `.local/smoke-ri_t_bv9/result.json` and its transcript. The fields
  `NeoForgeData.CreateData.Processing.Type/Time` resume the remaining time,
  preserve unknown sibling keys, and complete processing exactly once.
- Independent native drill: `.local/drill-smoke-g4o3ujex/result.json`.
  All six directions, 4 SU/RPM, ordinary stone/diamond-ore loot without a tool
  and regardless of the player's creative mode, bedrock, `block_drops`, clutch
  pause, progress clearing on an unbreakable target, and a restart yielding
  exactly one obsidian. This checks the stationary drill.
- Asset import: 324 private files, including 288 original files and 36 derived
  wheel-material variants. All 61 locales are preserved byte for byte;
  root models/textures for the new machines were compared with the original JAR.
  `.local/content-05-import/validation.json`, cache schema 7.
- All three archives were accepted by Horizon's standard validator; they contain
  no original Create or Minecraft assets.
- **The complete final Metal/audio run passed without skips**:
  `.local/smoke-hd1a1k8a/result.json`, 2 729 control commands. It includes all
  previous scenarios, 24 actual crafts, the millstone, four airflow types,
  stationary drill, press reset of fan progress, and a real restart. It checked
  exact remaining timers/unknown sibling NBT fields on a dropped item, saved
  inventories, and completion exactly once. The second launch reused the verified
  cache of all 324 assets. Both launches opened the normal MacBook Air Speakers
  audio output; the log contains no errors or warnings.
  This checks audio events/startup, not perceived sound quality.

Additional review found a difference at the press/fan boundary:
`BeltPressingCallbacks.whenItemHeld` clears previous fan progress before creating
results and remainders. The adapter now performs the same reset while preserving
unrelated `HeldItem` keys. A new native scenario reproduces the bug on the earlier
`389d0c06ae5e` build and checks a fresh full timer after a real press impact.
It passed on `722bc455fcbe` for a single input and a stack remainder:
`.local/smoke-audit05-frwj19pm/audit-result.json`. In both cases the new timer
runs 151→141 over ten ticks; the transported item's unrelated field is preserved.

A separate audit of the prepared **CreateProcessing05** map also passed on the
exact `722bc455fcbe` build: `.local/review-map-clone3-i_0qdnkc/review-map-result.json`.
Eight check groups include all 39 catalog entries, press/render clearing,
instruments/comparator, reversal of both wheels, millstone with real hoppers,
four airflows with eight outputs each, drill with one new cobblestone,
five crafts through the actual menu, and empty-bucket return. Seven gameplay
screenshots were viewed. Checks ran on a copy: `final-integrity.json` confirms
that all 29 source-map files are unchanged. The user walkthrough is in
[TESTING_05.md](https://github.com/Delete228/create-horizon/blob/eb8b96e8ea1db698b79bb61fd1caf6c963f31652/TESTING_05.md).

The standard **Add from GitHub** installed the published source commit
`16c5d454b1b9860da01b78474ea5206f0664d384` into a new empty test instance.
`native-build --profile performance` confirmed **Up to date / 0 s** and the exact
`722bc455fcbe` build. All 69 source files matched the local package byte for
byte; manifest JSON fields are identical, with the official exporter removing
the final newline. SHA-256 of the local `.hnmod`:
`f49534cc3d558eb6baca9b1479e48a8c1d2a33031b098c5fa037fa1c4d4e0c49`.
SHA-256 of the GitHub archive installed by the Launcher:
`6e88ee5b86f21fd756b09e8d9511516896d5cb4e2051214fbb73577ac2aa53f5`.
[Source-commit CI](https://github.com/Delete228/create-horizon/actions/runs/41732420020)
passed, including all eight Java oracles.

The mod was also installed into the user's **Create Processing 0.5** using the
actual `mods add-github` command with a link to this exact commit; a genuine
origin receipt exists. The ordinary repository URL hit the anonymous GitHub API
limit in this attempt; the `/commit/…` URL worked. The installed package again
reported **Up to date / 0 s**, selecting `native-fc02103-722bc455fcbe`; the previous
global Nightly and old instances were preserved. After restarting the GUI, its
window confirmed **Create Processing 0.5**, one mod, one world, and the correct
native version. Evidence: `.local/05-github-fresh-install.log`,
`.local/05-github-fresh-build.log`, `.local/05-github-byte-audit.json`,
`.local/05-user-github-install.log`, `.local/05-installed-build.log`,
`.local/05-install.json`.

The official Launcher's normal windowed `play` opened **CreateProcessing05**
on the final build. It confirmed 36 carried entries, equipped goggles,
original models/Russian labels, the verified 324-asset cache, and the normal
audio device. The game was left paused. The gameplay screenshot was viewed;
evidence is in `.local/visible05-bpomhiqr/result.json`, `review.png`, and
`.local/05-visible.json`. Audit copies do not replace the original user world.

Comparison of the new machines' matrices actually submitted for rendering uses
Java angles at **the same fractional frame times**. Computing an ideal
instantaneous speed from two frames gives small deviations due to the source
`float`; this was checked on saved millstone matrices. The test uses the original
Java method, rather than weakening the direction check.

Evidence: `.local/05-{core,clippy,performance,package}.log`,
`.local/05-oracles.json`; gameplay transcripts are beside the referenced results.
These checks do not establish full Create support, networked multi-client play,
moving contraptions, or cross-version Java/Horizon world exchange.

## Instruments and water wheels 0.4.0 (4 November 2026)

The same pinned Horizon API 359 / Minecraft 26.3 and Create 6.0.10 apply. The new
performance build `native-fc02103-6533e4389288` was produced by the official Launcher.

- **39 Rust tests**, formatting, Clippy, and source-package contracts passed.
- **75 145 Java comparisons**: the previous four oracles plus 6 388 new cases
  covering phases, angles (including large times), crank acceleration/reversal/
  coast-down, the speed scale, and wheel RPM limits. Method bodies come from the
  pinned source through the JDK AST. These are isolated contexts, not a full NeoForge run.
- The instrument gameplay scenario `.local/smoke-7t9g60e4/result.json` passed with
  Metal and audio: three gauge axes, comparator output 4→15, actual equipping of
  goggles, HUD, actual matrices of three transmissions and the crank in all six
  orientations, eight brass sheets, wheel currents/reversal/power, wood changes
  without consumption, placement refusal without item loss, retry above a floor,
  breaking a structural part with one wheel returned and all parts cleared,
  eight new actual crafts, and save/restart.
- 258 imported files matched the original JAR byte for byte, including 61 languages,
  four OBJ and three MTL files. Another 36 material variants are created only in
  the private cache. Total: 294 files, schema 5. Public archives omit these assets.
- Three 0.4.0 archives were accepted by Horizon's standard validator. `.hnmod` SHA-256:
  `b7e8c6ba112d02d25c572dbee12f8796b1608a5e8d07dd178337b6cfb41b8021`.

Additional evidence: `.local/04-{core,clippy,oracles,performance,packages}.log`,
`.local/04-resource-audit.json`, `.local/visual-oracle/result.json`.
The complete Metal/audio regression also passed:
`.local/smoke-a6vb0yx3/result.json`, without skips. A separate **Create Engineering 0.4**
instance and **CreateEngineering04** world were prepared; the package was compared
byte for byte with the published source commit `2c9b9d8`.
This intermediate version is preserved; the owner chose to review 0.5 directly.


## Foundation 0.3.0 (3 November 2026)

Base environment: macOS arm64 / Apple M4, Rust 1.99.1, Horizon API 359 / Minecraft
26.3. Create 6.0.10, source `ac0c444d9828da3453ae8cc65338e8de063286fb`.
Exact dependencies are pinned in `compatibility.json`.

| Check | Result and scope |
| --- | --- |
| Rust | 34 tests: kinetic components/clutch halves, source-speed priority, gearbox, transmissions, overload, traversal bounds, press cycle, quantities/buffer, motor controls, asset import/recovery/isolation. Clippy without warnings. |
| Original Java | 75 capacity/stress scenarios, 27 648 placements, 31 752 connection/port equations, 9 282 press-result routing/speed/head-curve scenarios. Original method bodies were extracted through the JDK AST; Minecraft/world/recipe contexts use explicit adapters. Full NeoForge was not run. |
| Assets | All 217 `assets/create` files from the set of 218, including 61 languages, `sounds.json`, and 38 OGG files, matched the local JAR byte for byte. Cache schema 3. Neither these files nor the JAR are published. |
| Depot/client | Top exchange, replacement/merging, buffer, components, Shift/side click. Item selection is checked in the actual render callback: it is empty after extraction, beyond just the server slot. The wire snapshot explicitly clears `HeldItem`. |
| Transmissions | Real redstone switches the clutch and gearshift direction. All four gearbox ports and perpendicular large cogs transmit the expected speeds. |
| Interactions | Crank 32 RPM / Shift −32 / stop; Survival casings consume material, encasing does not consume the casing, wrench rotation/removal and return of depot contents. |
| Automation | A hopper does not merge an input into a busy depot's stack; top-slot/buffer extraction preserves quantities. A fallen item is picked up and returned with `custom_data`. Three dropped ingots were pressed without a depot. |
| Actual crafts | 11 recipes executed through the 3×3 crafting-table menu and server click-slot: alloy, shaft, two cogs, crank, gearbox, clutch, gearshift, press, depot, wrench. Result and final inventory were checked. |
| Motor | Actual native-screen clicks select +128 / −64 RPM, scrolling changes it to −63, and speed reaches the shaft. The value is checked after restart. |
| Packages | Three 0.3.0 archives were accepted by standard `cargo-horizon-mod check`. The native package includes all public content; no separate `.hmod` is needed. |

Latest complete intermediate Metal run: `.local/smoke-0u4woeb4/result.json`.
Hoppers and world pressing were also checked in `.local/smoke-8rr__rei/transcript.json`;
that headless run stopped at the crafting-table menu (`no renderer yet`). Headless
mode now explicitly skips menu checks; Metal executes them. Results are retained
and are not counted as a complete headless pass.

Performance build: `native-fc02103-7ea49a8f5494`, built by the unmodified official
Launcher 0.2.37. The final expanded Metal run passed without skips: `.local/smoke-ncu3notg/result.json`.
It checked extraction after the first impact (7 inputs + 1 sheet), no duplicate
result, eight sound events/impacts for eight inputs, return of all eight sheets
and cleared rendering, power loss/restoration, and a gold cycle across restart.
Standard audio opened MacBook Air Speakers; the log contains no errors.
This checks startup/events/assets, not perceived sound quality. Test worlds were
disposable; original user worlds were not used.

Local evidence: `.local/foundation-unit.log`, `.local/foundation-clippy.log`,
`.local/{network,placement,rotation,processing}-oracle/result.json`,
`.local/foundation-resource-audit.json`, `.local/foundation-packages.log`,
`.local/foundation-performance.log`. Installation commands and tests are in the
repository; local game assets and results are not in Git.

Source package `12d83a0233317484b697292af0c7623ced074ec9` was published;
[CI](https://github.com/Delete228/create-horizon/actions/runs/41701109989) passed
all stages, including four Java oracles. Native archive SHA-256:
`0dd5758a7db5e7955b295e881f8ebe51a24f01a947e4d7c809e546f0bd8b086d`.

`mods add-github` for 0.3 was blocked by the anonymous GitHub API limit in both
the normal and isolated launcher homes. Instead, every file in the verified local
`.hnmod` was compared with the source archive of this published commit. The package
was installed into a new **Create Foundation 0.3** without a fabricated GitHub
origin receipt. Automatic GitHub updates for this specific instance are therefore
not yet configured; ordinary `mods add-github` can be used once the limit clears.
This is not a successful end-to-end GitHub installation for 0.3; the standard path
was previously verified for 0.2 (history below).

The exact performance build was transferred from the official native-build cache.
A new `native-build --instance "Create Foundation 0.3" --profile performance` read
the installed package and confirmed **Up to date (no build needed)** / the same
version. The new **CreateFoundation03** world was prepared through standard `play`
and the control API: 26 items, redstone transmissions, gearbox, manually driven
press, casings, crafting table, and 8 finished sheets. Old worlds were neither
opened nor replaced. Local evidence: `.local/foundation-source-install-audit.json`,
`.local/foundation-installed-build.log`, `.local/foundation-review-world-result.json`.

Original gearbox, clutch, gearshift, wrench and depot models with a finished sheet
were viewed in gameplay screenshots of the new gallery. The final normal windowed
launch was confirmed through the API: world `CreateFoundation03`, then `paused`.
macOS was locked, so CUA could not check the Launcher window/instance list;
the normal window did not return a screenshot while locked. A separate offscreen
launch of **the same new world** was used for inspection, then the normal windowed
client was left paused. Evidence: `.local/foundation-visible-state.json`,
screenshots `.local/foundation-{gallery,transmissions,press-results}.png`.

Automated tests do not establish complete Create support. The owner's listening/
visual review remains separate; exact particles, random sound variations,
kinetic atmosphere, smoothing, and the original UI were not yet complete at this stage.

# Prototype and installation checks

## Iteration 0.2.0: placement and direct depot interaction

Native package: `b839dcb0c9b6674367b6f05eaf50dc75d7234854`.
Behavior/asset reference: Create **6.0.10**, source
`ac0c444d9828da3453ae8cc65338e8de063286fb`. The adjacent Java checkout was not
switched: tools read pinned files through Git.

| Check | Result and scope |
| --- | --- |
| Rust and CI | 24 tests (14 core, 10 importer), formatting, package contract, and Clippy passed. [Source-package CI](https://github.com/Delete228/create-horizon/actions/runs/41687537157) passed. [Next CI](https://github.com/Delete228/create-horizon/actions/runs/41688429604) also ran both Java oracles successfully on JDK 25. |
| Placement versus Java | 27 648 combinations for shaft, cogs, and motor matched the AST-extracted original methods. Context, neighboring ports, and Minecraft types use adapters. |
| Stress versus Java | 75 scenarios again matched unchanged `KineticNetwork.java` from the same release. This is not a complete graph/NeoForge check. |
| In-game placement | 12 scenarios: X/Y/Z axes, adjacent shaft, Shift, inherited cog axis, motor, and horizontal press. |
| Direct depot | A top click places the whole stack without a GUI, an empty hand collects it, another item replaces it; identical items merge after return. Quantity is conserved. Side face, Shift, and return of eight output slots were checked. |
| Components and persistence | The candle's `custom_data` survived placement/return and save, restart, and extraction again. |
| Client result | A cancellable HEAD on `remote_block_reacts` returns Success for a top depot click. Without it the client sent an offhand action and immediately took back the deposited item. Inventory exchange remains on the named server hook. |
| Original assets | All 110 imported files, including 61 languages, were compared with the JAR byte for byte. Cache schema 2 refreshes the previous set. Original files were not added to Git or port archives. |
| Complete gameplay runs | Headless without the JAR and Metal with the JAR passed: placement/depot, iron, power loss, copper after restoration, and a gold cycle across restart. Metal uses final performance build `native-fc02103-0554886ea08b`. Screenshots were viewed: original depot, candle, and copper sheet are visible in the world. |
| Standard pipeline | Unmodified Launcher 0.2.37 downloaded the package by GitHub commit and built the installed `.hnmod`. Its SHA-256: `0f648187005c30879b40340b08b3401e63cc3356882923f65e8a41823a81ef80`. All 0.2.0 archives were accepted by the standard validator. |
| User gallery | New **Create Review 0.2** instance, **CreateReview02** world: the factory produced 8 iron sheets; orientations, depots, and items were prepared. Standard performance `native-build` confirmed Up to date. A normal windowed launch through the same Launcher opened the world with original graphics and Russian names; the game was left paused. |

While preparing the user instance, the anonymous GitHub API hit its rate limit:
the exact package downloaded through the standard pipeline, its genuine origin,
and the verified build cache were reused. Previous instances/worlds were not
opened or replaced. macOS was locked during the final CUA check of the Launcher
window; its refreshed instance list was not visually confirmed. Selection and
version were confirmed by the standard CLI, and the open new world by the
control API and a gameplay screenshot.

The fixture sends rotation/Shift early and waits for server delivery before
clicking, then waits for synchronization after the click. The Java oracle checks
the algorithm; the gameplay run checks the adapter. The headless run preceded
the depot collision-height correction to the original 13/16;
the final Metal run includes that correction.

Local results, not published in Git: `.local/placement-oracle/result.json`,
`.local/network-oracle/result.json`, `.local/locale-check-result.json`,
`.local/smoke-mhi4i1uo/result.json` (headless), `.local/smoke-yvme8bge/result.json`
(Metal), `.local/review02-world-result.json`, `.local/review02-visible-state.json`.

Persistence was checked within Horizon. Cross-version Java-world transfer is
outside the requirement; the full Create system had not yet been ported.
Exceptions are listed in [COMPATIBILITY.md](docs/compatibility.md). Animations, water,
sound, multiplayer, and the remaining machinery required later iterations.

## First-stage results (0.1.0)

Date: 3 November 2026. macOS arm64, Apple M4, Rust 1.99.1.
Exact commits and versions are listed in [compatibility.json](compatibility.json).

| Check | Result and scope |
| --- | --- |
| `horizon-port-pack` | Ran `import`, `inventory --offline`, `carry`, `report`, `check`. 8913 files; Create coverage marked `partial`. This is data import; Java code is not executed. |
| Rust core | 12 unit tests passed: connections, speeds, conflicting sources, overload, disconnection, press cycle, and head motion curve. |
| Asset importer | 9 unit tests passed: public-content preparation, recovery and isolation, model dependencies and PNG metadata, repeated import and cache recovery, no JAR, multiple JARs, wrong SHA-256, path traversal, model cycles, missing assets, Java loaders, size limit, and input-file symlink. All fixtures are synthetic. |
| Clippy | `--workspace --all-targets -- -D warnings` passed for the core and importer. |
| Java comparison | 75 power/stress scenarios matched unchanged `KineticNetwork.java`. Block/world use minimal adapters; this is not a full NeoForge test. |
| Native integration | Horizon with eight named hooks and three pinned mixins for content/resource loading passed `cargo check` and built into an executable client. |
| Archives | Official `cargo-horizon-mod check` accepted the content `.hmod`, native `.hnmod`, and combined `.hmod`. The package now pins the published corrected Mixins commit; no local patch is needed. |
| Headless game | Motor → shaft → two cogs → press → depot produced 8 iron sheets. |
| Power loss | With the shaft removed, 4 copper ingots remained unprocessed; restoring the shaft yielded 4 sheets. |
| World persistence | The game closed during a cycle with 3 gold ingots; restarting the same test world yielded 3 gold sheets. |
| Graphics and screen | The same scenario passed with Metal in `--offscreen`; the depot interface opened. Screenshots were viewed: provisional models and the slotted item are visible. Mouse item dragging was not separately checked. |
| Automatic in-game import | A new isolated Metal scenario ran with normal public content and the JAR in `horizon-mods/create-resources/`. The game imported and enabled 49 files itself; each was also compared with the original JAR byte for byte. Models and the copper-sheet texture were checked in screenshots. Iron, copper, power loss, and the gold cycle across save passed again. The second launch used the verified cache. |
| Resource reload | F3+T passed with the original JAR, after its removal, and after its return. Logs/screenshots confirm switching to provisional models and back. The original JAR was unchanged. |
| Public archives | After adding the importer, all three archives passed the official validator again. They contain no PNG, OGG, JAR, or Java classes; original assets are not packaged. |
| Official Launcher | A separate Create Prototype was created, the verified client imported, and the public content archive installed. New world CreateDemo launched through `horizon-launcher play`; Create content loaded, Metal worked, and the player's selected item was `create:creative_motor`. |
| Importer in the installed instance | Create Prototype updated to `imported-2026-11-03-78bafb5`. Official `play --instance create-prototype` opened the title screen offscreen: 49 assets imported from the JAR beside the mod, and Metal rendered its first frame. State `screen: title`, `world: null`; CreateDemo was not opened. The normal selected Nightly was preserved. |

## Standard GitHub installation (3 November 2026)

Source package at commit `73a9ccba62f619c4a7d0288a7d42c1146a3a8f52` was checked.
[CI for this commit](https://github.com/Delete228/create-horizon/actions/runs/41677935703)
passed: formatting, 21 Rust tests, source-installation contract, and Clippy.

- Unmodified installed Horizon Launcher **0.2.37** built the client with standard
  `native-build`, profile `dev`, and the full client feature set.
  The Launcher itself installed Mixins through `cargo install --git` at commit
  `a1e13326306c6bc074b11e6a4926deb551c4083e`. Tool source was not edited.
- `mods add-github` installed the exact published commit into a new isolated
  instance. Both revisions, all public content, and the absence of original
  PNG/OGG/JAR/classes were checked in the installed `.hnmod`.
- That installed package was then built, without specifying the working checkout
  directory. The result was `native-fc02103-0ac3c457cc78`. Repeating the command
  reported **Up to date (no build needed)** and used the same build.
- Both complete gameplay scenarios used this client **without `--dev-mod` and
  without a separate content archive**. Without the JAR, iron/copper pressing,
  power loss, restoration, and the gold cycle across restart passed.
- The Metal/JAR scenario also passed all operations, opened the depot, and reused
  the verified cache. The interface screenshot was viewed: the original copper
  sheet is visible in its slot. Before checking the menu, the fixture clears the
  hand and waits for the server-thread response, avoiding dependence on picking
  up items dropped during setup.

- Separate title-screen launches checked an invalid JAR, `mod.create` disabled
  after earlier enablement, and the original JAR. An invalid JAR produces a
  diagnosable fallback; a disabled mod enables neither content nor the importer.
- The standard **performance** profile was also built:
  `native-fc02103-1ba8788c96b6`. It was installed into new **Create Native** and
  launched with the ordinary Launcher `play` command. New **CreateInstallCheck**
  produced 8 iron sheets; the world, rig, and nine items were saved for visual review.
- The Launcher window was refreshed after CLI installation. Its UI confirmed the
  selected Native build and GitHub mod `create` with **Up to date** status.
  Later user-instance preparation reused the genuine package, GitHub receipt,
  and verified build cache because of the anonymous API limit; this was not an
  import of a separately hand-built game executable.

Isolation: new launcher/game/XDG homes and disposable worlds; assets, verified
compiler, and Cargo cache were cloned locally on APFS. This is a clean mod install
and a standard-build-path check with warm caches, not a measurement of first-time
dependency downloads on an empty computer. Gameplay tests disable audio through
the environment; the standard build retains audio/controllers features.

This iteration does not claim accuracy of every model/mechanic, multiplayer, or
the remaining Create blocks. The verified chain remains the bounded prototype
from the plan.

Local evidence, excluded from Git:

- `.local/native-startup-hqgsf5ag/result.json` — invalid JAR, disabling, and enabling.
- `.local/user-native-world-result.json` — performance, Launcher launch, and saved rig.
- `.local/user-native-build.log` — repeated standard build using the ready cache.

- `.local/official-launcher-check.log` — first official Launcher build.
- `.local/github-install-result.json` — installation response for the published commit.
- `.local/native-cache-check.log` — reuse of the completed build.
- `.local/smoke-346h0dtw/result.json` — complete cycle without the original JAR.
- `.local/smoke-menojh3s/result.json` — complete cycle with the original JAR and Metal.
- `.local/smoke-menojh3s/depot.png` — open interface and original copper sheet.

Local evidence, excluded from Git:

- [Import report](.local/port-work/coverage.md).
- [Headless result](.local/smoke-3507mg3u/result.json).
- [Graphics result](.local/smoke-8ekcjs2r/result.json).
- [Machinery](.local/smoke-8ekcjs2r/factory.png) and [depot screen](.local/smoke-8ekcjs2r/depot.png).
- [Automatic import and working machinery](.local/smoke-5_mrnj1k/result.json).
- [Resource switching through F3+T](.local/smoke-5_mrnj1k/asset-cycle-result.json).
- [Original models](.local/smoke-5_mrnj1k/reload-restored.png) and [provisional models after JAR removal](.local/smoke-5_mrnj1k/reload-without-jar.png).

Games ran in separate working directories/homes with copied assets. Each scenario
created its own world in its working directory. Existing user worlds were not
used. Source repositories `../Create` and `../horizon` remained unchanged.

At this point, the next stage still needed two-client checks, large networks and
chunk unloading, full depot/hopper interactions, and further Java animation/render
comparisons. Remaining Create machinery had not yet been ported. A working chain
does not establish that the entire mod is ready or fully compatible with Java.
