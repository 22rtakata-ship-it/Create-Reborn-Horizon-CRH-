# Presentation validation: 0.6.0-alpha.3 candidate

Status as of 5 November 2026. This is a **source candidate**, not a newly
installed release. The user instance **Create Factory 0.6**,
world **CreateFactory06** and its alpha.2 were not replaced. Implemented scope
remains unchanged: 27 blocks, 20 standalone items, 137 recipes.
The rough estimate for the entire port remains **8–10%**.

## Fixes

- Belt surface X-axis rotation: east now uses 90°, west uses
  270°. Direction follows original `Catnip AngleHelper`, used by
  Create 6.0.10's `BeltRenderer`. The correction is especially visible on sloped belts.
- Development builds now preserve truthful `native-mods.json` and public
  `native-data` beside the executable. Previously only the executable was copied,
  so world canvas could be disabled: the click hint appeared,
  but the motor's white RPM digits and frame were missing.
- The next package pins the accepted Horizon Mixins fix:
  `e8cd0aa38cffef49cb2e088ef716baa107148c02`, [PR #25](https://github.com/horizon-rust/horizon-mixins/pull/25).
  Our duplicate [PR #26](https://github.com/horizon-rust/horizon-mixins/pull/26)
  is closed as superseded by the accepted fix.
- Added a reproducible check of actual Game draw helpers with
  imported private models. It checks all four face vertices
  and their cyclic order, rather than just one normal.

## Findings from the belt report

The user's save was examined only through a separate copy. In the original
camera position, the eyes are below the belt's top surface. Original Java Create
also shows two side strips with an open center on a normal connector from this
view; the center is visible from above. This does not negate
the slope-surface X-rotation bug, which was fixed separately.
A missing section cannot be declared normal without comparing its shape,
direction and viewpoint.

## Results and validation limits

| Check | Result | What it establishes |
|---|---|---|
| Rust workspace | 124 tests; fmt and strict all-target Clippy passed | Portable logic, importer and bounded schematics |
| Python tools | 14 tests, including 4 new packaging regressions | Metadata/digest, artifact preservation and refusal when post-build is incomplete |
| Original `BeltRenderer`/Catnip against Rust | 1600 cases, 2560 surface frames | Part selection and matrices; old rotation rejected in 800 cases |
| Original motor settings side | 36 cases | Eligible hover side; not rendering output |
| Actual Game, CPU draw | 60 distinct combinations, 5088 vertices | 4 directions × 5 slopes × 3 RPM; positions, winding, light, unchanged NBT |
| Actual native client | 20 belt images inspected | Bounded horizontal, sloped, vertical and sideways constructions |
| Original Java client | 3 control views inspected | Top/side of horizontal belt and eastward ascent |
| Motor, one executable without/with metadata | 6 directions inspected | Metadata restores white world digits 128 and hover frame |
| Installed alpha.2 executable inside a private `.app`, without/with metadata | Both images inspected | Normal app-bundle path symptom reproduced; only the companion file changed |
| Accepted Mixins | Tool build and 3 focused tests passed | Embedded crate/tool attribute fix |
| `.hnmod` | Official check against exact Horizon revision passed | Package format; not a clean GitHub installation |

The CPU check compares imported native geometry with original Java rendering
matrices. It does not check the Java model baker, full UV/texture parity,
shaders, network callback or whole-frame pixels. The old build receipt did not yet
contain the output `game.rs` hash: this is explicitly recorded as
`build_stage_output_hash_verified=false`. The belt input module is linked to the build
and Java oracle by SHA; the test stage was restored byte for byte afterward.
Future builds also record the output `game.rs` hash.

Not all test groups form one uninterrupted run. Failed attempts and
original logs are preserved; additional checks do not rewrite their outcomes.
Full parity of every interface, animation, effect and sound is not claimed.

## Blocking macOS Launcher issue

A separate cause of the same symptom was found in normal **Play**. The Launcher
starts `Horizon.app/Contents/MacOS/horizon-client`, but `game_app::ensure`
puts only the executable and `horizon-ctl` there. `native-mods.json`
stays in the version root. The game looks beside the actual running executable.
This is confirmed by pinned source, the current upstream file
and the installed build: the root and `.app` executables share an inode, but metadata
inside `.app` is absent.

A separate private `.app` check with the same installed executable,
`c640101923ecabea1400982c22935008a78d138e771700965b9b3d418d4eb9a3`,
reproduced the missing digits/frame. Copying the truthful alpha.2 receipt beside that
executable restored `128` and the frame with the same camera and motor. This launched
the app-bundle executable itself, but did not automate Play in the Launcher window.

Development packaging is fixed, but this **does not fix the installed Launcher**.
[Issue #9521](https://github.com/horizon-rust/horizon/issues/9521) was published for the generic fix.
The same gap was found for `native-data`: the post-build API also looks for that
folder beside the actual executable. The generic fix was submitted in
[PR #9529](https://github.com/horizon-rust/horizon/pull/9529) and tested with both
companions, including creation, replacement and removal. Eight focused tests
in the actual Launcher, strict all-target Clippy and 524 Launcher tests passed,
with 7 pre-existing ignored tests. The full self-check retains an unchanged Python
installer-fixture failure, separately reproduced using main's files.

An additional probe compiled unchanged `game_app.rs` from commit `69e629890`
with declared localization/XML stubs. Actual `ensure` created the
companion resources inside a private `.app` itself. An app-side process read authored
fixture data through original `native_data::built`; the game executable from that `.app`
showed digits 128 and a white motor frame. The image was inspected, and the camera matched
the earlier negative control exactly. This did not exercise Play in the user's Launcher.
The actual Play route must still be repeated. The user's version cache
was not manually modified. Until that check, alpha.3 is not presented as a ready
fix for normal macOS installation.

This is a separate **Launcher** fix. Pinning a new Horizon commit in the mod
does not itself update the installed Launcher. The private corrected-app-helper check
and the normal installation-path update are tracked separately.

## Immutable artifacts and local evidence

- Candidate source:
  [`8b7cca1ce486e8b4297a46eabe817d3b7ed1a407`](https://github.com/Delete228/create-horizon/commit/8b7cca1ce486e8b4297a46eabe817d3b7ed1a407).
- Horizon: `9c7f9b2215dfed497aba5522c44f38926628648d` — a published fork,
  not a promise of arbitrary Nightly support.
- Create: `ac0c444d9828da3453ae8cc65338e8de063286fb`, 6.0.10 / Java 1.21.1.
- Package `dist/create-0.6.0-alpha.3.hnmod`, SHA-256
  `624b63d266131c5aecc803400a84f5c7e3c9da033e4a8ac030010fa19a0acd6e`.
- Development executable SHA-256
  `f45b91d055e81e5d1eaf1c72c2ea1992b6f140e634142a3e6d24bd07c77af7ef`.
  The complete developer folder includes its companion metadata; the executable alone
  is not a complete native-build artifact.
- All 142 current native-package files matched the immutable build snapshot by SHA.
  Original models, textures, sounds and JARs are not included in Git or the public package.

Private receipts (no resources published):
`alpha3-core-lcs9yl56/result.json`, `alpha3-package-result.json`,
`belt-render-oracle/result.json`, `belt-render-game-9vvqs0oi/result.json`,
`accepted-mixins-28jdf_40/fresh-result.json`,
`dev-native-hawp1q9n/companion-fix-result.json`,
`belt-fixed-native-rest-z7dapqow/companion-ab-image-review.json`,
`motor-app-ab-goxkp7fj/{result,image-review}.json`,
`pr9521-finish-7y3rmfbp/result.json`, `mac-ensure-probe-r8o3_j7o/result.json`,
`mac-ensure-game-positive-o_7ej57w/aggregate-result.json` — under `.local/`.

## Next visual review checkpoint

- [x] Fix X rotation and add independent source/native checks.
- [x] Rebuild with accepted Mixins and verify the complete developer artifact.
- [x] Submit the generic macOS Launcher fix and check the private `.app` it creates.
- [ ] Update the Launcher after acceptance and check the normal Play button.
- [ ] Complete a clean Add from GitHub and standard performance native-build of the candidate.
- [ ] Prepare a separate review instance/world, preserving alpha.2.
- [ ] Check motors in every direction: world digits, frame and menu.
- [ ] Check belts from above and beside, X/Z ascent/descent, stopping and reversal.
- [ ] Repeat the [13 factory stations](TESTING_06.md) and a world restart.

Generic PR status is in [UPSTREAM_STATUS.md](UPSTREAM_STATUS.md).
The full subsystem queue and completion criteria are in [ROADMAP.md](ROADMAP.md).
