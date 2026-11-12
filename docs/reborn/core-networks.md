# Core checkpoint — event-driven networks, 2026-11-12

## Implemented

The canonical `mixins/create/engine/src/core/networks.rs` now supplies one game-independent service per world session, exported by create-core and the native engine facade. It indexes loaded components by dimension, position and explicit host unload group; caches immutable network views; invalidates affected local components on node/source/load edits; and supports regional/dimension teardown. Cached queries reuse results without a tick-driven rebuild. Snapshot IDs are opaque and transient; old/foreign snapshots can be rejected with is_current. Powered Clutch halves retain independent caches.

The graph solver shares its existing candidate footprint with invalidation and now uses checked coordinate addition. Traversal stops immediately at its existing 4096-node limit. Truncated TooLarge results are deliberately uncacheable, so an edit beyond a partial frontier cannot leave a stale cached rejection.

Added eleven regression scenarios and a release benchmark. The verification entry point now runs both solver and cache benchmarks. Updated architecture, gaps, feature matrix and project map. Repaired roadmap.md, which previously contained the initial validation report, into a real phase roadmap.

## Actual checks in this restored environment

| Check | Result |
|---|---|
| cargo build --locked --workspace | PASS |
| python tools/reborn.py verify | PASS: format, 307 Rust tests, module isolation, adapter feature combinations, Clippy with warnings denied, distribution contract |
| Independent native engine cargo test --offline | PASS: 252 inherited regression tests |
| Python build/recipe tooling | PASS: 15 tests |
| Cache regression tests | PASS: 11, including 160 source/topology edits compared with fresh solver results |
| Cache benchmark | PASS: 100/1000 nodes, 1000 cached queries, exactly two solves including one after the edit; 10000 nodes returns TooLarge |

Logs: evidence/core-networks-*.log. Measurements: ../../benchmarks/network-cache-baseline.csv. The initial checkpoint's full Horizon type-check evidence is historical and was not rerun for this increment. No native hook, inventory/recipe logic, save codec, asset or dependency pin was changed. The shared simulation and native engine crate were compiled/tested, not a new game executable.

## Remaining boundaries

This service is functional and available to Rust callers, but the existing native kinetic_runtime.rs has NOT switched to it. Game event timing, real chunk lifecycle, survival, visuals, save/restart, actual in-game performance and multiplayer remain unvalidated. Host region fixtures are not tests of a Horizon chunk API. The 4096-node bound remains, as do the inherited graph equations and balancing.

Next: map the pinned engine's real lifecycle callbacks into this service; publish effective generator state in the same order as existing source events; preserve the removed-current-BE read path and synchronous neighbor publication. Then run the retained Java/native timing and gameplay fixtures in an isolated instance before enabling it in the live ticker. Phase 1 remains in progress; do not mark the native cache gap solved based on headless tests.
