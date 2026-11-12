# Create Reborn architecture

Decision: preserve the complete Create for Horizon history and native bundle; incrementally separate its game-independent simulation. A fresh rewrite would discard tested processing, persistence adapters, import security and moving structures.

Cargo dependency graph: create-core <- create-contraptions; create-core <- create-mechanics. Mechanics optionally depends on Contraptions through public actor/volume APIs. create-horizon-adapter composes these crates; create-testkit supplies fixtures. Core has no dependency on either module. Contraptions builds without Mechanics.

Canonical simulation sources remain under mixins/create/engine/src/core, compiled through explicit path attributes in the new crates. This transitional layout preserves the native archive's self-contained engine crate and the original adapters. There is one implementation per system, not parallel copies. Root create-horizon-core is a backward-compatible facade, retaining the factory example and original imports.

The native package still compiles its monolithic engine facade. The new workspace boundaries apply to headless simulation builds; migrating the native engine to the three compiled crates requires a packaging change and a real client regression run. Do not claim that native module toggles or three independent Launcher bundles already work.

Gantry transmission geometry was extracted to Core to remove an inverted dependency. MotionTracker::sample_with_activation is now a documented public extension point used by Mechanics' mounted Deployer. Deployer, particles and burner visuals currently require the optional contraptions feature; standalone Mechanics retains other simulation modules. Split presentation-specific glue particles in a later tested increment.

Engine-specific server mutation, inventories, NBT, packet sync, rendering and mixins remain under mixins/create/*.rs. Simulation accepts world views or callbacks. Preserve server ownership; headless tests cannot prove network authority or multiplayer correctness.

The inherited kinetic solver is iterative and bounded to 4096 vertices. The new networks.rs service wraps this solver with a dimension-scoped registry and persistent event-invalidated results; see networks.md. It is exported by Core and the native engine facade but is not yet wired into the live native ticker. The native kinetic_runtime.rs cache key includes game_time, world pointer and dimension: components are solved again on a new tick, even without a topology change. This does not meet the requested persistent event-only topology cache. Preserve source-event ordering while introducing that service; never silently replace it with a forever-valid RPM cache. Do not remove the limit to advertise 10000-node support.
