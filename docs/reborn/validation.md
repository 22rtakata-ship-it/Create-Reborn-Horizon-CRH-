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
