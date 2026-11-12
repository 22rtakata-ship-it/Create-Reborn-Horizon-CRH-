# Native Core network integration — 2026-11-12

## Implemented behavior

The native kinetic_runtime.rs now queries the shared Core Networks service through query_with. Stable graph results survive tick changes. Cache misses discover authoritative loaded nodes in one solver pass, reading each candidate at most once. The explicit start node overrides the engine read because Horizon temporarily removes its ticking block entity from world.block_entities.

The existing source_turn, published_node/generation, attach, generated_changed, pending transmission and synchronous publish flows are retained. Queries still consume published generation instead of treating raw ScrollValue/FlowScore settings as immediate source callbacks. A separate, tested PublicationTracker resets on tick/dimension changes and retains shared snapshots until reset, preventing pointer reuse from incorrectly suppressing a publication. Legacy NBT/presentation adapters receive Arc<Network> snapshots without a full graph clone.

Ordinary block_changed events now evict/invalidate the relevant position, including an empty connection frontier that gained a bridge. Other existing invalidate calls (sources, transmissions, belts, bulk edits) remain conservative full resets. New confirmed chunk_load/chunk_unload callbacks also reset all host observations without reading departing blocks. World reset releases registry, publications and retained source/session state through the inherited reset path. Chunk/source invalidation is not yet fully localized.

## Confirmed engine contracts

Official unmodified Horizon pin: 4e5296d1800b3f89f2ae48dfd99d431801df4d46. Mixins pin unchanged: e8cd0aa38cffef49cb2e088ef716baa107148c02.

- native-hooks.json defines chunk_load and chunk_unload version 1, targeting Game::native_chunk_load/native_chunk_unload in game/native_hooks.rs. Both are server notifications. New helper: mixins/create/kinetic_chunk_changed.rs.
- native_chunk_unload documents that departing block entities can still be present; the callback must not inspect departing blocks. Our helper only clears cached observations.
- game/native_chunk_snapshot.rs defines chunk coordinates as block coordinates >> 4. The native region mapping uses that exact rule; Core does not assume a chunk size.
- Native source lifetime/attachment code remains in the same adapter. No source revision, dependency pin, save codec, recipe, balance, item or asset was changed. The original Horizon checkout is clean; only a pristine export was transformed by official Mixins.

## Actual validation

| Check | Result |
|---|---|
| Workspace build | PASS |
| python tools/reborn.py verify | PASS: 314 Rust tests, format, module isolation, adapter feature combinations, Clippy -D warnings, source distribution contract |
| New host/publication helper scenarios | PASS: 7 |
| Independent native engine regression suite | PASS: 252 tests |
| Python tooling suite | PASS: 15 tests |
| Official Mixins plan/apply | PASS: 106 injections, 71 modules, 3 crates, 2 bridges, 1 data input |
| Complete injected Horizon client cargo check, no default features | PASS: 3m 19s |
| Injected horizon-client-lib --lib --tests cargo check | PASS: 1m 23s; compile check only, not test execution |
| Native adapter rustfmt --check | PASS |
| Real-game source timing, chunk lifecycle, save/restart, rendering, multiplayer | NOT RUN |

The seven helper tests cover the missing ticking entity, cache survival across 100 publication turns, an empty frontier joining/splitting networks, published versus changed raw generation settings, publication snapshot lifetime/reset, dimension/unload behavior and one read per host candidate. They execute the actual Core helpers called by the adapter against controlled callbacks. They do not instantiate Game or prove NBT/gameplay parity. In-game timing remains an acceptance gate.

Raw compiler/test logs are evidence/native-networks-*.log. The immutable export, native input and transformed client live under /workspace/work as session intermediates. The mod source snapshot used by plan/apply matches the final source files.

## Next acceptance gate

Build/link and launch a fresh isolated native instance. Run the inherited source-event/ticker-order fixtures, motor/crank/water-wheel starts and stops, powered clutch detach/reattach, source conflicts, wheel/shaft/cog/press processing, chunk boundary unload/reload and mid-cycle restart. Do not publish a playable/optimized/multiplayer claim until those real-client checks pass. Refine coarse source/chunk invalidation with explicit frontier dependencies only after these timing fixtures are available. Phase 1 remains in progress.
