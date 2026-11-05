# Contraptions

Create for Horizon uses one structure format for moving blocks and keeps the
driver rules in `create_engine::contraption`. The reference is Create 6.0.10
for Minecraft 1.21.1. The implementations share local coordinates and motion
rules while using Horizon's block, item and player APIs.

| Layer | Responsibility |
|---|---|
| Capture | Discover a payload through block movement and attachment rules. Store original identifiers and full block states in local coordinates. |
| Drive | Mechanical and Windmill Bearings, both hands of the Clockwork Bearing, and the regular and sticky Mechanical Piston advance their own poses. Rope Pulleys share the linear actuator, while Gantry Carriages follow a loaded rail. |
| Ownership | The structure belongs to the chunk, independently of the driver. It is saved with the removal of the original blocks. Removing a driver leaves a recoverable record if a block obstructs the landing. |
| Presentation | The client receives the captured structure and successive poses. Models are drawn in the same coordinate system used for collision. Original resources remain an optional local import. |
| Contact and actors | Horizon's normal collision paths handle moving geometry. A rider on a flat platform can follow horizontal rotation or translation and be lifted. Moving harvesters, saws and drills use the original crop or block rules and Horizon loot. |

The current native adapter limits each structure and its swept footprint to one
loaded chunk column. This keeps the source, target and persistent record in the
same chunk save. It refuses unsupported snapshots and occupied landing cells
instead of discarding blocks. Capture is limited to an allowlist of stateless
blocks, idle tools and a single ordinary chest on a vertical Mechanical
Bearing. Super Glue, chassis, other inventories and fluid-containing payloads
still need adapters. The supported Gantry path has a fixed loaded rail span.
The Rope Pulley supports dry rope and cargo. The clockwork hands are captured
and restored together and currently accept vanilla payloads only.

Collision uses exact boxes at quarter turns, narrow strips for oblique rotation
and the registered shapes of piston parts and tools. Tests cover dropped items
and grounded riders on full block tops, including unobstructed side pushes from
a translating full block. A blocked push clips against the terrain. The Java
wall fixture also moves the player sideways. Other side impacts, arbitrary
tilted contact, rider transfer on partial shapes, mounted mobs and downward
lock behavior remain open. Sable integration remains a separate milestone.

The Harvester handles wheat, carrots, potatoes, beetroot, Nether Wart, cocoa,
sweet berries, sugar cane and supported leaves or growing plants. A moving Saw
pauses its platform while cutting supported vanilla trees, fungi and stalks.
A horizontal stationary Saw also fells trees. The moving Drill breaks eligible
vanilla blocks and drops their loot. Cut progress on moving tools resets across
restarts. Block breaking damage to passengers, stationary Saw processing
recipes and planting trees with a Deployer remain open.
[Deployer engine and host boundary](deployer.md)

## Mounted chest

An ordinary dry single chest on a vertical Mechanical Bearing keeps all 27
slots and item components while the structure is moving. Harvesters, Saws and
Drills on that structure insert loot from the same chunk column in ascending
slot order, matching Java's `ChestMountedStorage` and
`ItemHandlerHelper.insertItem`. Overflow and loot from neighboring columns
drop at the tool. The server saves the inventory with the structure's chunk
and restores it at landing, including after a restart or controller removal.

Access the inventory when the chest has landed. The moving chest menu,
multiple or double chests, other drivers, named, locked and loot-table chests
are outside the current adapter.

## Windmill sails

Windmill Sail Frames and all sixteen sail colors use the original block IDs,
placement orientation, dye and shear rules. The item placed in Survival counts
as one sail, and recoloring preserves the dye stack. A Windmill Bearing counts
the attached sails in its rotation plane. Eight sails or frames generate 1 RPM.
The optional local Create JAR supplies original models and textures. The built-in
pack supplies sail shapes and vanilla-derived materials. Placement previews,
canvas bounce and fall damping remain open.

## Minecart contraptions

The Cart Assembler engine covers the five rail modes, assembler placement and
rail swaps, the pass queue and cooldown, and assembly and landing decisions for
single and coupled carts. It also computes single-cart turning and both
coupling corrections. The pinned Java sources are `CartAssemblerBlock`,
`CartAssemblerBlockEntity`, `MountedContraption`, `OrientedContraptionEntity`,
`CouplingHandler` and `CouplingPhysics`.

The native adapter still needs cart-aware rail lookup and collision shapes,
an authoritative rail-pass callback, and a moving payload saved and synced
with the cart. Coupled movement also needs the two carts' persistent links and
controller motion. The current implementation is exercised by Rust unit tests.

## Verification

- `tools/bearing_reference_checks.py` and `tools/bearing_native_checks.py` compare
  three axes at 32 RPM against Java. The Horizon test also reloads a running
  structure and recovers it after the driver is removed and an obstruction is
  cleared.
- `tools/piston_checks.py` compares extension and retraction across four rigs,
  including opposite axes, a sticky payload and blocked travel: 176 tick
  samples plus final block states. The native test reloads mid-stroke and
  recovers a blocked orphan on a second restart.
- `tools/clockwork_checks.py` compares the three original modes and both hand
  angles, with native checks for grouped restoration, restart and an obstruction.
- `tools/pulley_reference_checks.py` and `tools/pulley_native_checks.py` compare
  48 ticks and 80 block observations against Java. The native run also checks
  restart, controller removal and breaking a placed rope.
- `tools/gantry_reference_checks.py` and `tools/gantry_native_checks.py` compare
  Java motion and stopped blocks. The native run checks powered pinion output,
  restart, blocked placement, reversed travel and vertical orientation.
- `tools/cart_assembler_reference_checks.py` checks a regular minecart, a
  powered controller rail with a chest cart, a rejected TNT cart and a coupled
  payload released onto two original assembler rails in Java.
- `tools/cart_coupling_math_checks.py` compares 722 rail corrections bit for
  bit with `CouplingPhysics.followLinkOnRail` in the original JAR.
- `tools/contraption_collision_checks.py --player --piston --side` verifies
  player and dropped-item movement in a disposable Horizon world. The separate
  Java contact tool records the unobstructed push and wall response.
- `tools/piston_interaction_checks.py` verifies applying slime, preserving the
  facing and shaft axis, wrenching retracted pistons and breaking a pole.
- `tools/actor_reference_checks.py` runs the original Java Create and Catnip
  classes against the Rust actor rules. `tools/actor_native_checks.py` verifies
  crop resets and drops, tree felling, drilling and motion after a cut.
  `tools/stationary_saw_checks.py` compares the horizontal Saw over 32 Java ticks.
- `tools/mounted_storage_checks.py` checks Java slot order, rotated landing,
  custom item components, a full chest's overflow and Horizon recovery after
  an obstructed landing and real process restarts.
- `tools/sail_reference_checks.py` and `tools/sail_native_checks.py` verify
  placement, dye, shear, axial exclusion and 1 RPM generation with eight
  sails or frames. The native runs cover restart, landing and both art packs.
- `tools/bearing_review.py` prepares a separate in-game review world. The
  checked world and screenshots are local artifacts, with no changes to the
  player's normal worlds.

[Original Java source](https://github.com/Creators-of-Create/Create/tree/ac0c444d9828da3453ae8cc65338e8de063286fb/src/main/java/com/simibubi/create/content/contraptions)
