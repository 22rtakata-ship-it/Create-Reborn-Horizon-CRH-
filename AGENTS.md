# Create Reborn development rules

Read docs/project-map.md, docs/compatibility.md and docs/roadmap.md before changes. Full user specification: docs/reborn/requirements.pt-BR.md. Preserved upstream guidance: docs/reborn/upstream-AGENTS.md.

Preserve Create for Horizon history, functionality, namespaced IDs, MIT notices and player worlds. Documentation is English; keep game localizations. Do not redistribute original Create assets or JARs. Import remains local and optional.

Core must not depend on Mechanics or Contraptions. Contraptions depends only on Core. Mechanics may use Contraptions through its optional public extension APIs. Engine-specific code stays in mixins/create. Shared canonical sources remain there until native packaging migration is tested. Do not duplicate them in crates.

Commands: cargo build --locked --workspace; cargo test --locked --workspace; cargo fmt --all --check; cargo clippy --locked --workspace --all-targets -- -D warnings. Run python tools/reborn.py verify for module isolation, distribution and tests. Rust is pinned in rust-toolchain.toml. Native integration and packaging commands are documented in packaging/README.md.

Implement small coherent increments; update project-map when ownership changes. Check real engine source at the exact revision before using APIs. Never treat mocks or unit tests as game, rendering or multiplayer validation. A feature is complete only after functional tests, required persistence/boundary checks and real-game acceptance where applicable. Record real commands, exit codes and blockers in docs/reborn/validation.md. Do not publish registry entries without explicit authorization.
