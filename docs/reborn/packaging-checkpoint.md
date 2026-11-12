# Executable, package and acceptance preparation — 2026-11-12

## Actual completed work

- Linked the injected Linux x86_64 Horizon client at the supported official source pin. Build took 5m 19s. Native simulation source is commit 930d0d4ecdf6ee7670e6e716ede27de4d95d2a00; the later changes in this checkpoint are tooling/documentation only.
- Built horizon-ctl from the same transformed source. Both executables run --help.
- Staged the client with native-mods.json and native-data using the existing dev_build.package_artifacts helper; native source digest is recorded in the build sidecar. These files must remain together.
- Built the official cargo-horizon-mod from the unmodified pinned Horizon checkout. It built and accepted create-0.9.0.hnmod, create-0.9.0.hmod (combined native/content) and create-content-0.9.0.hmod. Exact SHA256/size data is in build-manifest.json and SHA256SUMS.
- Archived the Linux developer client, control binary, public native data, build metadata and notices. The archive contains no original Minecraft/Create JARs or prepared game resources. Available registry dependency, vendored/theme and Rust notices accompany the engine and mod licenses.
- Added smoke.py --core-only, reusing the existing isolated-process/world harness, and a preflight/report wrapper. The new scenario targets real water flow, cogs, press processing, overload recovery, clutch, power loss, cross-column unload/reload, restart and optional rotational rendering.
- Python tooling suite: 20 tests passed, including five guards that prevent missing prerequisites or preflight-only success from being labeled as gameplay. New scenario modules also pass Python compilation. The scenario code itself has NOT been run in a world.

## Build configuration

Official Horizon 4e5296d1800b3f89f2ae48dfd99d431801df4d46, Mixins e8cd0aa38cffef49cb2e088ef716baa107148c02 and Rust 1.99.1 are unchanged. Used the already verified pristine official Mixins stage. Compiled --locked -p horizon-client --bin horizon-client --no-default-features --jobs 2 with CARGO_PROFILE_DEV_DEBUG=0 and CARGO_INCREMENTAL=0. This is a developer/headless-capable build, not the Launcher's normal performance/full-feature profile. Audio, friends/controllers and Windows binary compatibility are not implied. Native Windows installation has not been tested.

Only our generated check/test compiler caches were removed to reclaim disk space before linking. No source history, user instance/world or prepared game resource was deleted. Source checkouts remain unmodified inputs. The validator/build outputs are under ignored dist/; large executables are not committed as Git blobs.

## Observed practical blocker

The official horizon-prepare --check says the requested resource folder is not prepared. An actual invocation of the linked headless client exits with:

```
Horizon's prepared game files are missing: no horizon-resources.json in /workspace/work/minecraft-resources
```

The game prints its normal preparation/Launcher guidance and exits before a world can run. The acceptance wrapper separately records status=blocked, gameplay_attempted=false, gameplay_passed=false in core-acceptance-status.json. That file describes the acceptance wrapper; the separate failed executable startup probe is retained in evidence/package-launch-probe.log.

No facts.json/prepared resource folder or Minecraft installation is present in the workspace or normal checked game locations. The user has been asked for an accessible prepared-resource path. No licensed game inputs were invented, downloaded through an unauthenticated shortcut or replaced by fake resources. Original Create artwork remains optional; prepared vanilla Minecraft resources are still required. Resource preparation needs Java 25; Java 21 is installed here. Running an already prepared resource folder does not need Java.

## Acceptance status and next action

Compilation/linking: PASS. Package schemas: PASS. CLI startup/help: PASS. Actual world startup: BLOCKED by missing resources. Wheel/shaft/cog/press, stress, chunk lifecycle, cold restart, graphics, survival and multiplayer acceptance: NOT RUN. Phase 1 remains in progress.

Use docs/core-acceptance.md to install into a new development instance or run the prepared isolated-world workflow after supplying resources. The optional offscreen mode checks submitted wheel matrices and captures a frame; a headless-only run cannot validate rendering. The private original asset importer is not exercised without a player-supplied JAR. No registry entry is published by this checkpoint; any GitHub artifact draft is a development candidate for the repository owner to review.
