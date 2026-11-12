# Initial modular checkpoint

## Implemented

Preserved the entire upstream history and native mod. Added five workspace crates and a backward-compatible root facade. Separated Gantry transmission math and shared particle RNG into Core, exposed the mounted actor extension method, and made Mechanics/Contraptions composition optional in the adapter. Added three cross-module integration fixtures, a solver baseline and a portable verification entry point.

## Files

Public module manifests/entry points are under crates/. Canonical algorithms remain in mixins/create/engine/src/core; only extraction and public-boundary adjustments were made. Existing native hooks/content/importer were retained. New architecture, compatibility, feature matrix, source inventory, hook map, testing, performance and roadmap documents are indexed in project-map.md. Original guides and AGENTS rules were preserved under docs/reborn/upstream-*.

## Verification

Workspace build, 296 Rust tests, 15 Python tests, format/Clippy, isolated module builds, native engine's 252 regression tests and distribution contracts passed. The pinned official Mixins tool planned and applied all 104 injections; cargo check passed for the complete injected Horizon client. See validation.md and raw evidence. These checks do not establish playable survival, asset fidelity or multiplayer.

## Next work

Link and launch an isolated native client, validate the wheel/shaft/cog/press and save/reload scenarios, then implement the dimension-aware persistent topology cache while preserving source-event ordering. Continue adapting the retained Core components rather than rebuilding their tested algorithms. Follow gaps.md before broadening machines/contraptions or independent native packaging.

Subsequent Core development is recorded in [core-networks.md](core-networks.md).
