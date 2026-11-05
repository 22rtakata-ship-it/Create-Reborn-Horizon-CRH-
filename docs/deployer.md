# Deployer engine and host boundary

The engine follows the pinned Create 6.0.10 Deployer. It covers the stationary
stroke timer and hand pose, moving active point and trigger, item and target
checks, filtered inventory routing, and moving block-break retries.
The original block and entity ID is `create:deployer`.

| Original code | Rust responsibility |
| --- | --- |
| `DeployerBlockEntity` | Stationary phases, RPM timing, reach ray and hand extension |
| `DeployerHandler.shouldActivate` | Same block and filled bucket checks, and the downward belt distinction |
| `DeployerMovementBehaviour` | Active point two blocks ahead, cell entry, reverse-pass activation, mounted input and stall timing |
| `DeployerRenderer` and `DeployerActorVisual` | Extension along the current hand axis and the original stalled oscillation |
| `DeployerItemHandler` | Filtered insertion, matching hand stacks, overflow-first slots and protected filtered extraction |

Preserve the Java data keys `Mode`, `State`, `Timer`, `Powered`, `Owner`,
`Inventory`, `HeldItem`, `Overflow`, `Filter`, `FilterAmount` and `UpTo` when wiring the block entity. `Reach`
and `Fistbump` are transient client update values. The moving context has its
own `HeldItem` and optional stall `Timer`.

## Horizon integration

The pinned Horizon `Game::clip_ray` takes an explicit origin, direction and
reach. The exported `native_raycast` uses the window player's camera. The
Deployer needs the former with the two prescribed segments, and needs the
action to retain the prescribed target cell when the ray returns a different
cell. Bucket and sandpaper actions also need a temporary ray-world mask.

Horizon already has the normal block-item placement path in
`LevelBehaviour::place_block`. `Game::use_hand` constructs its `PlaceContext`,
applies item-supplied block state and block entity components, then handles
placement callbacks, events, sound and item consumption. That path can supply
the actual placement rules for a sapling. Its acting-player inventory, GUI and
event handling are currently tied to the live player. A server automation entry
point must carry its own hand and inventory, pose, owner identity and
permission context through the complete placement action.

The same boundary is needed for use-first, block use, item use, entity
interaction and attack. The existing native click and damage hooks observe
those operations. They do not execute them from an automation context. The
adapter needs the original consuming-result order and a way to suppress menus
for an actor while retaining the gameplay effects of block and entity use.
Block and entity loot must flow back to the actor inventory and its mounted
storage, with the source distinction for experience from mob kills. A mining
adapter also needs access to ordinary tool wear, harvest checks and block crack
stages under the actor identity.

The engine does not register a playable Deployer. Mounted storage binding,
normal item/entity dispatch, schematic printing, belt recipes and presentation
remain host integrations. Run `python3 tools/deployer_reference_checks.py`
against the isolated Java reference for the original sapling fixtures.
