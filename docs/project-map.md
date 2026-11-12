# Project map

| System | Canonical implementation / entry |
|---|---|
| Core public modules | crates/create-core/src/lib.rs |
| Event-driven registry, cache, dimension/region lifecycle | mixins/create/engine/src/core/networks.rs; docs/networks.md |
| Native lazy query / publication / lifecycle integration | mixins/create/kinetic_runtime.rs; mixins/create/block_changed.rs; mixins/create/kinetic_chunk_changed.rs |
| Host callback and publication regressions | crates/create-testkit/tests/host_networks.rs |
| Cache event regression scenarios | crates/create-testkit/tests/network_events.rs |
| RPM, ratios, stress, graph traversal | mixins/create/engine/src/core/kinetics.rs |
| Source water sampling | mixins/create/engine/src/core/waterwheel.rs |
| Placement / directions | mixins/create/engine/src/core/placement.rs |
| Shared presentation random interface | mixins/create/engine/src/core/particle_random.rs |
| Shared gantry transmission | mixins/create/engine/src/core/gantry_transmission.rs |
| Mechanics public modules | crates/create-mechanics/src/lib.rs |
| Crushing / fan / recipes / transport | mixins/create/engine/src/core/{crushing,fan,basin,transport}.rs |
| Native recipe compilation | mixins/create/basin_recipes.rs; mixins/create/fan.rs; mixins/create/saw_processing_runtime.rs |
| Contraptions public modules | crates/create-contraptions/src/lib.rs |
| Assembly, transforms, drivers, collision, actors | mixins/create/engine/src/core/contraption/ |
| Native hooks and integration | mixins/create/horizon-mixin.toml; reborn/hooks.md |
| Compatibility facade | crates/create-horizon-adapter/src/lib.rs; src/lib.rs |
| Integration fixtures | crates/create-testkit/tests/modules.rs |
| Local assets importer | mixins/create/assets/ |
| Schematics | mixins/create/schematics/ |
| Data and recipes | content/; mixins/create/data/ |
| Phase-1 acceptance / preflight | tools/core_checks.py; tools/core_validation.py; docs/core-acceptance.md |
| Packaging | tools/package.py; packaging/README.md |
| Full inventory and provenance | docs/reborn/source-inventory.md; provenance.json |

Paths are deliberately based on the actual inherited tree; future edits start here and in AGENTS.md. Path attributes avoid duplicate implementation during packaging migration.
