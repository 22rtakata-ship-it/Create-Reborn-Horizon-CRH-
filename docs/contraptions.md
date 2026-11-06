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
blocks, idle tools and a single ordinary chest and Deployer on a vertical
Mechanical Bearing. The three chassis variants can expand capture on a
vertical Mechanical Bearing, Gantry, Rope Pulley or Mechanical Piston.
Other inventories and fluid-containing payloads still need adapters. The
supported Gantry path has a fixed loaded rail span.
The Rope Pulley supports dry rope and cargo. The clockwork hands are captured
and restored together and currently accept vanilla payloads only.

Collision uses exact boxes at quarter turns, narrow strips for oblique rotation
and the registered shapes of piston parts and tools. Tests cover dropped items
and grounded riders on full block tops. Horizontal translating shapes resolve
player overlap using the minimum-separation axes from Java's
`ContinuousOBBCollider`, then use Horizon's terrain collision resolver. The
wall fixture checks sideways separation in both directions. A vertical
Mechanical Bearing also separates the local player from the sides of moving
full cubes. The adapter uses the current pose, Create's contracted player body,
the original six separating axes and the ordinary terrain resolver. Existing
flat-platform support and linear actuator paths retain their own rules.
The upright resolver matches the pinned JAR for separation, surface and time
of impact. The native adapter applies the lateral overlap response. Arbitrary
tilt, contact with partial shapes, moving block bounce, exact client sub-tick
pose timing, mounted mobs and downward lock behavior remain open. Sable
integration remains a separate milestone.

The Harvester handles wheat, carrots, potatoes, beetroot, Nether Wart, cocoa,
sweet berries, sugar cane and supported leaves or growing plants. A moving Saw
pauses its platform while cutting supported vanilla trees, fungi and stalks.
A horizontal stationary Saw also fells trees. The moving Drill breaks eligible
vanilla blocks and drops their loot. The Plough tills exposed soil, clears
snow and protects crops rooted in farmland. Cut progress on moving tools resets
across restarts. Plough entity throws and block breaking damage to passengers
remain open.

An upward stationary Saw strips logs, bark blocks, stems, hyphae and Bamboo
Blocks, cuts stripped wood into planks, and cycles through plank shapes and
stonecutting results. It also cuts Andesite Alloy into six shafts. Drop an
item on the powered blade. The default Saw takes one input at a time and
ejects its output in the direction set by the shaft and rotation. The saved
`RecipeIndex` and ordinary output filter select the next recipe in the pinned
Java order. A Saw with no matching recipe passes the item through unchanged.

The Saw uses Horizon's loaded stonecutting recipes with 1.21.1 ID aliases
where needed. Four additional 26.3 outputs from ordinary stone are excluded
from the Saw's cycle. Vanilla stonecutters retain their complete recipe list.
The source recipe order is checked for the original pack set. Extra datapacks
can change that order in Java. Missing Create decorative outputs, the original
filter control, automation and cutting particles remain open.

A downward Deployer on a vertical Mechanical Bearing plants ordinary saplings
from its hand and refills one at a time from a single mounted chest. The plain
sapling filter can be set in saved data. Stop the bearing to swap the held
stack by clicking the hand. The owner, filter and remaining stock survive a
restart. A stationary downward Deployer plants saplings in the cell two
blocks below its body. Punching, general item use, complex
filters and their interface remain open.
[Deployer engine and host boundary](deployer.md)

## Super Glue

Use Super Glue on two connected blocks to store the selected volume. Sneak-use
on a block or change the held item to clear the first corner. With glue in hand,
attack a block through an existing volume to remove that volume. Survival uses
the two durability calls in Create 6.0.10. Creative leaves the item intact.

Volumes currently fit in one loaded chunk column. Mechanical and Windmill
Bearings transfer them with their payload and restore them at the snapped
landing pose. Gantry, dry Rope Pulley and Mechanical Pistons transfer volumes
relative to their current cargo origin, including starts away from zero stroke.
A piston carries glue on its cargo. A volume that crosses its head or pole
is refused before assembly.
A volume that would be split or include a stationary block stops capture.
The mounted volume shares the chunk save with the structure. The original
selection outlines, particles, offhand placement and transfer on other drivers
remain open.

## Mounted chest

An ordinary dry single chest on a vertical Mechanical Bearing keeps all 27
slots and item components while the structure is moving. Harvesters, Saws and
Drills on that structure insert loot from the same chunk column in ascending
slot order, matching Java's `ChestMountedStorage` and
`ItemHandlerHelper.insertItem`. Overflow and loot from neighboring columns
drop at the tool. The server saves the inventory with the structure's chunk
and restores it at landing, including after a restart or controller removal.

A Portable Storage Interface can dock that chest with a stationary interface
and a vanilla hopper below. The tested path unloads items and preserves their
components. [Portable Storage Interface](portable-storage.md)

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

## Chassis

Linear, Secondary Linear and Radial Chassis expand capture on vertical
Mechanical Bearings, dry Gantry Carriages, dry Rope Pulleys and Mechanical
Pistons. Sticky faces and ranges are restored after a quarter turn or a linear
translation, including recovery after a restart and an obstructed landing.
[Chassis rules and verification](chassis.md)

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
- `tools/gantry_chassis_checks.py` checks sticky capture and translated landing
  in Java and Horizon. It also exercises native save recovery and refused
  snapshots.
- `tools/gantry_glue_checks.py` and `tools/pulley_payload_checks.py` check
  ordinary Super Glue clicks, capture away from the zero-stroke position, a
  cold native restart and restored bounds. The pulley also carries a chassis
  with its selected range and sticky row.
- `tools/cart_assembler_reference_checks.py` checks a regular minecart, a
  powered controller rail with a chest cart, a rejected TNT cart and a coupled
  payload released onto two original assembler rails in Java.
- `tools/cart_coupling_math_checks.py` compares 722 rail corrections bit for
  bit with `CouplingPhysics.followLinkOnRail` in the original JAR.
- `tools/super_glue_reference_checks.py` observes the original selection,
  durability and removal. `tools/super_glue_native_checks.py --transfer` reads
  saved volumes before and after restart, removal and bearing landing.
- `tools/contraption_collision_checks.py --player --piston --side` verifies
  player and dropped-item movement in a disposable Horizon world.
  `tools/contraption_contact_reference_checks.py` records the unobstructed
  push and both wall exits on the original Java client.
- `tools/piston_interaction_checks.py` verifies applying slime, preserving the
  facing and shaft axis, wrenching retracted pistons and breaking a pole in
  both clients.
- `tools/piston_payload_checks.py` checks chassis and glue on ordinary and
  sticky strokes, including a saved moving payload and a refused glue volume
  crossing the piston head.
- `tools/contraption_contact_oracle.py` runs 945 comparisons against the
  discrete horizontal resolver in the pinned original JAR.
- `tools/rotary_contact_oracle.py` compares 17,280 upright contact cases with
  the pinned JAR. `tools/rotary_contact_checks.py` checks mirrored lateral pushes,
  a wall slide and an untouched player in the real clients.
- `tools/plough_checks.py` compares forward and reverse passes, including
  grass, coarse and rooted dirt, farmland crops, three snow layers and drops.
- `tools/deployer_farm_checks.py` and `tools/deployer_refill_checks.py --combined`
  compare planting ticks and chest slots, then test a restart and a shared
  Saw and Deployer rotor in Horizon.
- `tools/deployer_grounded_farm_checks.py` grows an oak through vanilla
  worldgen, joins a low Saw to the rotor by ordinary Super Glue clicks and
  checks same-cell replanting, stock and the cut tree across a restart.
- `tools/saw_processing_checks.py` compares 24 ticks for three stationary Saw
  inputs and checks ejection and resumption after a process restart.
- `tools/saw_recipe_checks.py` compares 29 cycles and 1,254 per-Saw tick
  observations for alloy, planks, stone and saved filters, including a restart.
- `tools/portable_farm_checks.py` uses the same field and wood-cutting scenes in
  the two clients. It checks the crop or log left at the first swept cell and
  the exact item delivered to the stationary chest.
- `tools/loot_dialect_checks.py` checks ordinary drops after converting the
  pinned Create loot condition format to the Horizon runtime's format.
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
