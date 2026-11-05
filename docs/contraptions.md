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
blocks and idle tools. Super Glue, chassis, inventories and fluid-containing
payloads still need adapters. The supported Gantry path has a fixed loaded rail
span. The Rope Pulley supports dry rope and cargo. The clockwork hands are captured and restored
together and currently accept vanilla payloads only.

Collision uses exact boxes at quarter turns, narrow strips for oblique rotation
and the registered shapes of piston parts and tools. Tests cover dropped items
and grounded riders on full block tops. Side impacts, arbitrary tilted contact,
rider transfer on partial shapes, mounted mobs and downward lock behavior remain
open. Sable integration remains a separate milestone.

The Harvester handles wheat, carrots, potatoes, beetroot, Nether Wart, cocoa,
sweet berries, sugar cane and supported leaves or growing plants. A moving Saw
pauses its platform while cutting supported vanilla trees, fungi and stalks.
A horizontal stationary Saw also fells trees. The moving Drill breaks eligible
vanilla blocks and drops their loot. Cut progress on moving tools resets across
restarts. Mounted storage, block breaking damage to passengers, stationary Saw
processing recipes and planting trees with a Deployer remain open.
[Deployer engine and host boundary](deployer.md)

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
- `tools/contraption_collision_checks.py --player --piston` verifies normal
  player and dropped-item movement in a disposable Horizon world.
- `tools/actor_reference_checks.py` runs the original Java Create and Catnip
  classes against the Rust actor rules. `tools/actor_native_checks.py` verifies
  crop resets and drops, tree felling, drilling and motion after a cut.
  `tools/stationary_saw_checks.py` compares the horizontal Saw over 32 Java ticks.
- `tools/bearing_review.py` prepares a separate in-game review world. The
  checked world and screenshots are local artifacts, with no changes to the
  player's normal worlds.

[Original Java source](https://github.com/Creators-of-Create/Create/tree/ac0c444d9828da3453ae8cc65338e8de063286fb/src/main/java/com/simibubi/create/content/contraptions)
