# Phase-1 real-client acceptance

The linked Linux developer client and validated source packages are development artifacts. Gameplay acceptance is still blocked in the build environment because prepared Minecraft 26.3 resources are absent. The launch probe exits with `no horizon-resources.json`; no world scenario has passed. This document describes executable scenarios, not completed results.

## Install in a new development instance

For a desktop machine, create a separate Minecraft 26.3 instance in Horizon Launcher. The inherited supported workflow is **Mods → Add from GitHub** using this fork:

```
https://github.com/22rtakata-ship-it/Create-Reborn-Horizon-CRH-/tree/main
```

Build the native mod and start that new instance using the Launcher's normal preparation through your own Minecraft Java installation/account. Alternatively use the validated create-0.9.0.hnmod through the Launcher's native-mod installation path. Do not install upstream Create and this fork together: both intentionally retain the create namespace. Do not install all three alternative archives at once. The separate content-only .hmod does not supply the native machines.

The source package fixes Horizon 4e5296d1800b3f89f2ae48dfd99d431801df4d46 and Mixins e8cd0aa38cffef49cb2e088ef716baa107148c02. The registry release is a different build. Keep any existing player instances/worlds separate. Windows native compilation/installation has not been tested here; the provided executable archive is Linux x86_64 only.

## Isolated automated run

Extract the Linux developer archive, keeping horizon-client, native-mods.json and native-data together. It includes horizon-ctl and notices but no Minecraft resources or original Create JAR. Supply a previously prepared resource folder from your own installation. Java is not needed to run prepared resources; preparing them from JARs requires the pinned Java 25 (the build environment has Java 21).

From this repository:

```
python tools/core_validation.py \
  --game /path/to/crh-linux-x86_64-dev/horizon-client \
  --ctl /path/to/crh-linux-x86_64-dev/horizon-ctl \
  --resources /your/prepared/26.3/resources \
  --report .local/core-acceptance.json
```

Add `--offscreen` for real rendered frames and a moving-wheel pose check; it requires a working graphics backend. Without it, the report explicitly skips rendering. Add `--preflight-only` to check prerequisite file/metadata presence without launching a client. Preflight success never means gameplay passed. The optional `--preparer /path/to/horizon-prepare --horizon /path/to/pinned/horizon` runs the official completeness/version check before launch. Without that option, the real client remains responsible for rejecting incomplete prepared data.

Exit codes: 0 requested check passed, 1 acceptance failed, 2 prerequisites blocked. The JSON records gameplay_attempted and gameplay_passed separately. Tests of this reporter use synthetic files and are not game tests.

The wrapper delegates to the existing tools/smoke.py harness with the new --core-only scenario. That harness creates a unique .local/smoke-* directory, copies resources privately, isolates Horizon/XDG directories and stores its new world there. It never opens an existing player world. Logs, control-command transcript and result.json remain in that directory. Only successful assertions plus the cold-restart checks produce passed=true.

## Scenario coverage and expected checks

| Scenario | Observable acceptance |
|---|---|
| Real fluid source | Bubble-current Water Wheel supplies absolute 8 RPM; reversing Soul Sand/Magma reverses it |
| Shaft / large-to-small cog / Press | Shaft matches source RPM; large-to-small reverses and doubles it |
| Press / Depot | Two iron ingots become exactly two iron sheets |
| Overload and recovery | Three presses at 16 RPM consume 384 SU against a 256 SU source; removing extra load restores rotation |
| Clutch / redstone | Powered clutch stops the downstream press; release recovers |
| Power loss | Removing the shaft preserves two copper ingots unprocessed; reconnection yields exactly two sheets |
| Cross-column factory | Source and downstream components straddle X chunks 128/129 |
| Actual unload/reload | Teleport away; both block.loaded flags must become false, then true on return; RPM and inventory recover |
| Cold restart during a cycle | Observe running Press ticks, request quit, restart the same isolated world; exactly two golden sheets eventually remain |
| Rendering, optional | A screenshot and at least two different submitted wheel matrices; not merely a placed block |

Between independent item scenarios the fixture replaces its depot and clears dropped fixture items in this new world. This cleanup is not an item-conservation assertion across scenarios. The restart request follows an observed running cycle; exact final saved-tick timing still needs the real trace. Survival acquisition/progression, full source timing parity, multiplayer and sounds are outside this focused fixture. No scenario above has been executed successfully in this environment yet.

The scenario code uses the pinned control API confirmed in docs/horizon-ctl-commands.md and game/agent.rs (`block.loaded`, command/data, container, wait, teleport, quit). Balancing and the bubble-current fixture come from the retained Create 6.0.10 rules and engineering_checks.py. Do not replace absent resources with placeholder facts or claim fixture compilation as in-game validation.

Development artifact downloads: [artifacts/crh-dev-2026-11-12](https://github.com/22rtakata-ship-it/Create-Reborn-Horizon-CRH-/tree/artifacts/crh-dev-2026-11-12). The `.hnmod` is the native source package; the `.tar.gz` is the Linux developer client. They are not gameplay-certified releases.
