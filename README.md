# Create Reborn for Horizon

Incremental modular fork of [Create for Horizon](https://github.com/Delete228/create-horizon), preserving upstream history and native code. This checkpoint separates headless simulation into Core, Mechanics and Contraptions; it is not a certified playable release.

```sh
cargo build --locked --workspace
cargo test --locked --workspace
python tools/reborn.py verify
```

Rust 1.99.1 is pinned. See [architecture](docs/architecture.md), [compatibility](docs/compatibility.md), [audit and feature matrix](docs/reborn/feature-matrix.md), [actual validation](docs/reborn/validation.md), [roadmap](docs/roadmap.md), [project map](docs/project-map.md) and [native packaging](packaging/README.md). Original documentation is preserved under docs/reborn/upstream-README.md.

Original Create artwork is not redistributed. Default independent artwork and the upstream optional local JAR importer are preserved. LICENSE, NOTICE and THIRD_PARTY_NOTICES.md apply.
