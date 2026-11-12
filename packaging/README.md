# Native build and packaging

Keep one self-contained native bundle. The new Cargo modules do not yet provide independently installable Launcher packages or native module toggles. The existing `create` namespace and 0.9.0 manifest are inherited; this checkpoint is not a new registry release.

Headless Linux/Windows (PowerShell also accepts these commands):

```
cargo build --locked --workspace
cargo test --locked --workspace
python tools/reborn.py verify
python tools/reborn.py compatibility --horizon PATH_TO_PINNED_HORIZON
python tools/reborn.py bench
```

Install official Rust 1.99.1 through rustup; rust-toolchain.toml pins the version. The audited session used local CARGO_HOME and RUSTUP_HOME under work/toolchain. No player installation is required for these tests.

Inherited native development pipeline on Linux/macOS:

```
python3 tools/bootstrap.py
python3 tools/stage.py check
python3 tools/stage.py build
python3 tools/package.py --cli PATH_TO_PINNED_CARGO_HORIZON_MOD
```

Set HORIZON_SOURCE to a clean checkout at compatibility.json's Horizon pin. The bootstrap and setup_mixins scripts obtain official pinned tools. Review tools/common.py for their local cache layout. tools/dev_build.py --help documents the separate immutable-source builder. Run the official validator for every output; do not rename an arbitrary ZIP to .hnmod. Native source injection and compilation are separate operations.

Native Windows development is not certified here: inherited tools/dev_build.py imports fcntl and the setup/stage paths have Unix assumptions. Use an isolated WSL development environment for source builds until a Windows path is tested. Do not advertise native Windows build support from the headless Cargo checks.

A standalone Linux developer client has now been staged outside Launcher-owned directories; actual Launcher installation is still untested. Follow the preserved upstream README's Launcher workflow only with a validated package and a new development instance. Do not use an existing player's world. Do not publish an updated registry entry without authorization.

## Prepared development artifacts

The packaging checkpoint now links a Linux x86_64 developer client and validates all three archives with the official cargo-horizon-mod built from the pinned engine. See docs/reborn/packaging-checkpoint.md for exact hashes and results. The client is built with no default features, debug info disabled and incremental compilation disabled; it is not the Launcher's full desktop/performance feature profile.

The Linux archive preserves native-mods.json, native-data and licensing notices beside the executable. It contains no Minecraft resources or original Create artwork. Its --help and the control binary execute, but world startup is blocked until the player supplies prepared resources. Prefer the source .hnmod/normal Launcher build for Windows. Installation into a real Launcher instance remains unvalidated.

The executable Phase-1 test workflow is documented in ../docs/core-acceptance.md. It creates only a new isolated world and reports missing prerequisites without marking gameplay as passed. The original full smoke suite remains available without --core-only.
