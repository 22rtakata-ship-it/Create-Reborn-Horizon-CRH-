# Incremental roadmap

The original requested scope and gameplay acceptance criteria are in reborn/requirements.pt-BR.md. Historical upstream plans are preserved in reborn/upstream-roadmap.md. This file tracks the current fork; validation results belong in reborn/validation.md.

## Phase 0 — initial audited checkpoint

Completed: preserve upstream history; inspect licenses and compatibility pins; inventory source and native hooks; separate headless simulation into five crates; compile/test the workspace; plan/apply the native bundle and type-check the injected client at the fixed Horizon revision. See reborn/checkpoint.md for the first delivery and its evidence.

Completed in the packaging checkpoint: linked Linux developer executable and official validation of the native, combined and content archives.

Remaining gates: complete behavioral source review, real Launcher installation and fresh-world acceptance. Current Horizon main, Windows native builds and multiplayer are not certified.

## Phase 1 — Core development (in progress)

Retained existing tested shafts/cogs/gearboxes, source generation and kinetic/stress rules. Added the game-independent event-driven network registry/cache with dimension isolation, indexed unload groups, opaque snapshots and localized invalidation. Eleven new regression scenarios cover cache behavior and compare repeated event sequences with fresh solver results. See networks.md and reborn/core-networks.md.

The native adapter now calls the service and retains its source generation/attachment flow. Added confirmed chunk load/unload callbacks and seven host/publication helper regressions. See reborn/native-networks.md.

A linked Linux developer client, officially validated packages and an executable --core-only acceptance workflow are prepared. World startup was attempted but blocked by absent prepared Minecraft resources (horizon-resources.json); no gameplay scenario is marked passed. See core-acceptance.md and reborn/packaging-checkpoint.md.

Next: supply prepared resources, then validate source timing and the wheel/shaft/cog/press chain, rotational rendering, chunk unload/reload and save/restart in an isolated game; then localize conservative source/chunk invalidation where fixtures support it. Type-checking and helper tests do not complete Phase 1.

## Phase 2 — reliable Core API

Complete registration/consumption notifications, persistent-state rebuild/migration, source ownership and chunk boundary regression fixtures. Preserve existing tools, clutches, gearshifts, chain drives and speed controller while testing their real-world interactions.

## Phases 3–5 — Mechanics

Retain industrial recipe, crusher, fan, belt, basin and logistics implementations. Validate a full ore-processing line with inventories, power interruption, backpressure and restart before broadening machine coverage. Maintain data-defined balance from Create 6.0.10.

## Phases 6–9 — Contraptions

Retain assembly, drivers, windmills and actors. Test transactional capture/restoration, collision, mounted storage and unloading; address single-column limits without risking existing worlds. Verify custom windmill factories, harvesters and mobile drills in real worlds.

## Phase 10 — release gates

Measure industrial/contraption/rendering baselines, profile actual TPS and frametime, validate packaging and installation, and run real client/server tests. Make no FPS, Windows or multiplayer claim from pure Rust tests.

## Phase 11 — expansions

Advanced fluids, arms, trains, elevators, Sequenced Assembly and Ponder remain later work. They must not delay the first playable survival acceptance chain.
