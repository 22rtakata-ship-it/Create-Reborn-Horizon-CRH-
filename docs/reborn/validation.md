# Local validation — 2026-11-12

Environment: Linux x86_64, official Rust 1.99.1 installed in session-local work/toolchain; no pre-existing Horizon/Create project, game installation or player world. Upstream checkouts are read-only inputs to the modified fork. Source history is retained. Documentation is not a substitute for gameplay acceptance.

| Command / check | Observed result |
|---|---|
| Upstream cargo test --locked --workspace | PASS: 293 tests (252 simulation, 20 assets/importer, 21 schematics) |
| cargo build --locked --workspace | PASS: modular headless workspace |
| python tools/reborn.py verify | PASS: fmt; 296 tests; four no-default-feature module builds; mechanics-only and contraptions-only adapter builds; Clippy with -D warnings; distribution contract |
| Native engine cargo test --offline --manifest-path mixins/create/engine/Cargo.toml | PASS: 252 tests, independently compiled package engine |
| Python tooling unit tests | PASS: 15 tests |
| Distribution contract | PASS: version pins, bundled content, 213 fallback textures and notices |
| Exact Horizon revision check | PASS for 4e5296d1800b3f89f2ae48dfd99d431801df4d46 |
| Pinned horizon-mixins-cli build | PASS |
| Official Mixins plan | PASS: 104 injections, 71 modules, 3 crates, 2 bridges, 1 data input |
| Official Mixins apply | PASS against pristine exported official Horizon pin |
| Injected Horizon client cargo check --locked, no-default-features, jobs=2 | PASS: complete injected client type-check, 4m 56s; no-default-features, not a link/run |
| Solver-only release baseline | PASS: 100 and 1000 nodes Running; 10000 TooLarge (4096-node bound) |
| In-game acceptance / renderer / real multiplayer | NOT RUN |
| Native .hnmod official validator / Launcher installation | NOT RUN; no distribution claim |

The three new integration tests cover source removal, windmill-to-Core energy and a Mechanics Press paused by power loss and resumed with one processing event. They do not test actual inventory/NBT or save/restart. The 252 simulation tests were redistributed as 34 Core, 110 Mechanics and 108 Contraptions; they were not replaced by a reduced new suite.

Local raw logs are under evidence/. Unit-test output from importer tests uses synthetic fixtures, not a redistributed original JAR. Full native plan JSON and exported engine source remain session intermediates under /workspace/work; they are not part of a game package.

Initial planning attempts caught a source symlink rejection and a missing Cargo PATH; both were corrected using the inherited immutable-source exporter (materialized documentation links) and the session toolchain PATH. No upstream tracked game code was modified. The native snapshot differs from final source only in whitespace formatting in gantry.rs and gantry_transmission.rs; the final canonical sources also passed the independent native engine suite.

Limits: no current-main compatibility claim; no full client link/run claim; no private original Create JAR/Java differential run; no actual TPS/FPS/RAM or allocation measurements. The original author evidence is retained under upstream documentation and VALIDATION.md but is not counted as a local pass. See gaps.md and feature-matrix.md.

To reproduce commands in this exact session, source `/workspace/work/activate-rust.sh` before entering `/workspace/create-reborn`. For a new machine use an official rustup installation or the preserved tools/bootstrap.py workflow; do not copy compiled caches.

The final windmill integration fixture uses the same Motor source representation as the native windmill adapter. Its three integration tests and testkit Clippy were rerun after that fixture adjustment (evidence/integration-final.log).

## Subsequent Core checkpoint

The event-driven network service, 307-test workspace result and native integration boundaries are recorded in [core-networks.md](core-networks.md). Initial checkpoint results above are historical.

## Native Core integration checkpoint

See [native-networks.md](native-networks.md) for the 314-test workspace result and the new complete injected-client type-check. Unlike the previous pure-Core checkpoint, the native ticker now calls the service. Game execution remains unvalidated.

## Executable and packaging checkpoint

See [packaging-checkpoint.md](packaging-checkpoint.md) for the linked Linux binary, officially validated archives, 20 Python tooling tests and the actual missing-resource startup failure. Gameplay acceptance remains blocked, not passed.

## Pinned Create parity checkpoint — 2026-11-13

See [beta-0.1.md](beta-0.1.md) for the verified stable reference, the successful local Java comparison counts, the Goggles impact correction and the required first-playable acceptance scope. The original private JAR's SHA-512 was checked against Modrinth metadata; neither it nor its assets are committed. Java algorithm comparisons now have local evidence, superseding the initial checkpoint's absence of Java runs. Native gameplay acceptance remains unrun.

Verification of this source correction: `cargo fmt --all --check`, `cargo test --locked --workspace` (314 tests), `cargo clippy --locked --workspace --all-targets -- -D warnings`, `python3 tools/check_distribution.py`, `python3 tools/reborn.py verify`, and `CREATE_SOURCE=/workspace/work/create-reference python3 tools/stress_defaults_oracle.py` all exited 0. The oracle was added to the existing pinned-reference CI job with its JSON report retained as an artifact. Local logs: `/workspace/work/beta-audit/`. Native Mixins apply exited 0 against the pristine pinned engine; only transformed `crates/horizon-client/src/game.rs` changed relative to the previous staging output.

The updated injected client passed `cargo check --locked --offline -p horizon-client --no-default-features -j 2` (exit 0, 2m 07s; `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`, session target `/workspace/work/game-build`). This is a complete client type-check of the correction, not a new linked release, equipment interaction test or game launch.
