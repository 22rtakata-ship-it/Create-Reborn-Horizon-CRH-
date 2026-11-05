# Contraption engine

The first stage implements the shared rules in `create_engine::contraption`.
Create 6.0.10 supplies the reference. Each driver will use the same captured
structure and restoration planner.

| Layer | Responsibility |
|---|---|
| Capture | Discover blocks from the anchor using movement, adhesion and attachment rules supplied by the host. Store full snapshots at local positions. The excluded controller stays in the world. |
| Pose | Convert points between the assembly and the world using the centre of the anchor block. Keep the continuous angle for motion and use the nearest quarter turn when planning block placement. |
| Restoration | Resolve all target positions and rotated block snapshots. Return a plan only when every target passes the host's preflight. |
| Mechanical Bearing | React to power and manual input, advance the angle, and request assembly or disassembly. The host acknowledges disassembly only after committing the plan. |

Horizon supplies block rules and the snapshot codec. The core can represent
slime and honey adhesion, explicit face bonds and block attachments. Connecting
those rules to the in-game Super Glue boxes, chassis and block-specific behavior
is separate work. The current restoration preflight can refuse an occupied
position and leave the snapshot intact. Create's replacement and item drop
rules will need their own tests in the native adapter.

## Native integration

Store one authoritative structure record per moving assembly in the level.
Keep the controller reference separate so breaking the driver cannot discard the
blocks it holds. Define save and chunk unload behavior before making the capture
remove world blocks. A server-side commit must revalidate the captured positions
and check every placement before applying it. A rejected placement keeps the
structure available for retry or recovery. The initial world test should cover
normal stops, obstructions, restart and removal of the controller.

Send the snapshot and pose to the client through the native game path. Render
and resolve contact from the same local coordinates. Actors and inventories
should operate on captured blocks through the structure's own view of the world.
Further drives can supply their own initial frontier and motion without changing
the ownership or restoration contract. Sable can be evaluated against that
boundary once an ordinary Create contraption works end to end.

## Verification

`tools/bearing_reference_checks.py` uses the isolated Java reference client.
It runs a Mechanical Bearing at 32 RPM on X, Y and Z. A slime block picks up an
oak log, the angle advances by 9.6 degrees per tick, and cutting power returns
the log to the corresponding quarter-turn position with its axis rotated.
The Rust tests use the recorded trace and the observed target on each axis.
They also exercise capture limits, an unsupported snapshot, an unloaded cell,
and retrying a refused restoration without losing its data.

This stage runs in the engine. A player can try moving structures once the
native ownership, write and rendering paths are connected and verified in a
disposable Horizon world.

[Create source revision](https://github.com/Creators-of-Create/Create/tree/ac0c444d9828da3453ae8cc65338e8de063286fb/src/main/java/com/simibubi/create/content/contraptions)
