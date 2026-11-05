# Chassis

The capture rules use Create 6.0.10. The pinned Java entry points are
`AbstractChassisBlock`, `LinearChassisBlock`, `RadialChassisBlock`,
`ChassisBlockEntity` and `Contraption.moveChassis`.

| Block | State and grouping | Included payload |
| --- | --- | --- |
| `create:linear_chassis` | `axis`, `sticky_top`, `sticky_bottom`. Adjacent blocks of this variant and axis form a group on all six sides. | Each sticky axial face scans a straight row. Forced movement along a face enables that row without slime. The scan stops before air or brittle blocks. It includes a solid non-supporting endpoint and stops there. |
| `create:secondary_linear_chassis` | Same properties as Linear Chassis, with a separate group. | Same row scan. |
| `create:radial_chassis` | `axis` and four reference-orientation sticky properties. Adjacent radial chassis join along the common axis. | Each sticky face floods its axis-perpendicular plane through supportive faces within the strict `range + 0.5` radius. The flood skips air and brittle blocks. |

All three variants use the `create:chassis` block entity and its
`ScrollValue` integer. The default is 8 with a range from 1 to 16.
`engine::contraption::chassis` computes group links, included blocks and
rotated sticky faces from the host's movement classification.

## Current support

The first native adapter supports a vertical Mechanical Bearing in one chunk
column. It saves the range of each captured chassis by local position and
restores it with the rotated state at landing. Unknown block-entity data and
unloaded cells fail capture before world edits. The installed mod provides
fallback models and loads the original models from the optional local JAR.

Use a slimeball or Super Glue on a glueable face to make it sticky. Clicking
an already sticky face covers the other eligible faces. Sneak-click with an
empty hand to clear the targeted face. Applying slime leaves the stack count
intact. The linear variants inherit the axis of either linear variant when
placed against it without sneaking. A same-variant, same-axis neighbor
clears the covered sticky end on placement.

The wrench settings board, group preview, connected textures and slime
particle remain open. Use `/data merge block x y z {ScrollValue:4}` to set
the range for a fixture. Other drivers and the full set of movement
classifications are later steps.

## Verification

`tools/chassis_reference_checks.py` exercises two original Java bearing rigs.
`tools/chassis_native_checks.py` repeats their group and payload layout,
then checks obstructed recovery across restarts, rotated states and drops.
Use `--hand` for ordinary survival interaction. The Rust tests also cover
forced linear movement, rotation and unavailable cells.
