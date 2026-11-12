# CRH development artifacts

Development candidates, not a gameplay-certified release. Main source branch:
https://github.com/22rtakata-ship-it/Create-Reborn-Horizon-CRH-/tree/main

- **create-0.9.0.hnmod**: officially validated native source package; use this for a NEW Horizon 26.3 development instance.
- **crh-linux-x86_64-dev.tar.gz**: linked Linux x86_64 developer client, horizon-ctl, native metadata/data and licensing notices. This is not a Windows executable or a normal full-feature/performance build.
- **create-0.9.0.hmod** and **create-content-0.9.0.hmod**: alternative combined/content-only packages for compatibility/tooling. Do not co-install all variants. Content-only does not implement native machines.
- **SHA256SUMS** and **build-manifest.json**: exact build pins, settings, sizes and hashes.

No Minecraft/Create JARs or prepared game resources are included. The build environment lacked prepared Minecraft 26.3 resources: actual startup fails at missing horizon-resources.json. No world, rendering or save/restart acceptance is marked passed. Phase 1 remains in progress.

Installation and the prepared isolated-world test workflow:
https://github.com/22rtakata-ship-it/Create-Reborn-Horizon-CRH-/blob/main/docs/core-acceptance.md

This separate artifact branch keeps binaries out of main's source history. GitHub release asset uploads returned HTTP 401 while Git/API operations remained available, so files are provided here. Download individual files with GitHub's Download raw file action; a source-only checkout can use --single-branch --branch main.
