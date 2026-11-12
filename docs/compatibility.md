# Compatibility — audited 2026-11-12

| Component | Fixed reference |
|---|---|
| Fork source | Delete228/create-horizon `4ae7340c06da5e77828e12494509fe9a70b8eaa4` (development 0.9.0) |
| Registry version | 0.7.0-alpha.2, source `236c87a889cd76f7e663d1d182c28dc1145f7b14` |
| Horizon supported target | `4e5296d1800b3f89f2ae48dfd99d431801df4d46` |
| Horizon current inspected | `f24ee1accf1c816736e6e067939d04f3853f474e`; not certified |
| Mixins | `e8cd0aa38cffef49cb2e088ef716baa107148c02` |
| Behavior reference | Create 6.0.10, `ac0c444d9828da3453ae8cc65338e8de063286fb`, Minecraft 1.21.1 |
| Native game target | Minecraft 26.3 |
| Rust | 1.99.1 |

The catalog and development branch are different revisions. Registry macOS evidence and upstream VALIDATION.md are author reports, not tests reproduced in this environment. No Java resource JAR or existing player world was used here.

Current Horizon is not automatically compatible. Use the exact official target, not main. Build on a pristine exported source tree; never patch the user's engine. Official Mixins plan/apply and the complete injected Horizon client cargo check passed locally at the supported target. This confirms source/API compatibility for the tested no-default-features configuration, not a linked or running game. See reborn/hooks.md, reborn/provenance.json and reborn/validation.md for local results.

Horizon native mods are source mixin bundles compiled with the game, not a stable Rust binary ABI. `.hnmod` packages native source; `.hmod` can contain data/content (and the inherited combined package). Actual schemas and validators live in Horizon crates/cargo-horizon-mod and launcher/src/native_mods.rs. The existing tools/package.py uses those formats. Retain one bundle until independent native dependency behavior is validated. Cargo feature isolation alone does not establish Launcher support.

Code: MIT (Horizon, Mixins, Create code and Create for Horizon). Create src/main/resources/assets is All Rights Reserved, confirmed in the pinned LICENSE.md. Original models/textures must remain player-supplied local imports. Preserve LICENSE, NOTICE, THIRD_PARTY_NOTICES.md and native notices. No original asset files or JAR are included in the new deliverable.

The native Core integration checkpoint adds two confirmed version-1 chunk lifecycle hooks at the same pins (106 injections total). Current full-client and test-target type-check evidence is in [reborn/native-networks.md](reborn/native-networks.md). It does not certify a linked/running game or change the supported game version.
