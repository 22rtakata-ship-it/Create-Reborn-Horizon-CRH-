# Visual checks for Create 0.5

Open instance **Create Processing 0.5**, world **CreateProcessing05**.
This is a separate world; the previous 0.3/0.4 worlds are preserved. Coordinates below
are machine positions; use F3 to see your coordinates. Engineer's Goggles are already equipped,
and the Wrench and basic blocks are in the inventory. Materials are in barrels.
New materials are in the barrel at `43 64 3`; the remaining three catalog items are
in the barrel at `45 64 7`. Older materials are in two barrels near `27 64 10`.

Try other constructions too: the stations are convenient starting examples.
Useful bug reports include coordinates, input items, RPM and the expected
result. A construction that behaves differently in Create 6.0.10 is especially useful.

| Station / coordinates | Action | Expected result |
| --- | --- | --- |
| Press, `2 66 2`, Depot `2 64 2` | Collect the prepared 8 Brass Sheets with an empty hand; insert 8 Copper Ingots. Increase RPM on the motor to the left. | One ingot per hit; all 8 Copper Sheets can be collected. The item image disappears after extraction. Press speed changes with RPM. |
| Gauges, `2 66 8` and `3 66 8` | Look at them while wearing goggles; open motor `0 66 8` and change 16 → 256 RPM. | The HUD shows the network, RPM and stress; the speed needle changes. The comparator behind the gauge changes its signal from 4 to 15. |
| Gearbox, `4 66 8` | Compare rotation of the connected half shafts; add a side output. | Neighboring connected shafts agree; the side output direction follows gearbox rules. |
| Clutch, `2 64 14` | Toggle the lever above it. | The input keeps rotating; the Clutch output disconnects and reconnects. |
| Gearshift, `2 64 18` | Toggle the lever above it. | The output reverses while the input retains its direction. |
| Hand Crank, `18 66 12` | Turn with right-click; then Shift+right-click; try other placement faces. | The crank changes direction smoothly, and its shaft drives the press to the right. The crank and base are oriented correctly. |
| Vertical Gearbox, `9 64 12` | Rotate with the Wrench; place another from the inventory. | The same `create:gearbox` block is used with another axis; the correct four faces connect. |
| Gear transmission | Connect a large and small cogwheel diagonally, then add another pair. | 2:1 ratio, meshing reversal and 256 RPM limit. An incompatible closed network stops. |
| Small Water Wheel, `13 67 2` | Place ingots on Depot `17 64 3`; replace magma in the channel with soul sand. | The wheel gives 8 RPM / 256 SU; after gearing, the press receives 16 RPM. Changing the flow reverses direction. |
| Large Water Wheel, `24 67 2` | Click with different plank types; check rotation and place your own wheel nearby. | 4 RPM / 512 SU, original geometry and wood replacement. A large wheel needs room for its structural parts. |
| Millstone, `36 65 0` | Put wheat into the upper hopper. Collect results from chest `37 64 0`. | Wheat is processed one item at a time; flour and chance bonus outputs are collected below. Diamonds in the input hopper stay there. |
| Manual Millstone | Drop wheat onto a separate Millstone, then click with an empty hand. | The item is captured when it lands. Outputs are collected first, then the remaining input. No chest menu opens; item right-click insertion does not replace dropping. |
| Washing, fan `35 65 7`, Depot `38 64 7` | Place flour on the Depot. | Water in the enclosure turns flour into dough. Up to 16 items take about 7.5 s; 17–32 take about 15 s. RPM affects range, not recipe time. |
| Smoking, `35 65 11`, Depot `38 64 11` | Insert dough or raw meat. | Airflow through a lit campfire makes bread or cooked meat. An extinguished campfire does not process recipes. |
| Haunting, `35 65 15`, Depot `38 64 15` | Insert sand or cobblestone. | Produces soul sand or blackstone. Reversing airflow changes the order of passing through different media. |
| Blasting, `35 65 19`, Depot `38 64 19` | Insert raw iron. | Airflow through lava produces iron ingots. Food that should be smoked is destroyed by blasting, matching the original behavior. |
| Airflow | Drop an item in front of a fan; change the RPM sign and place a solid obstruction. | The item is pushed or pulled; a solid obstruction limits the current. A Depot may sit one block below the air channel. |
| Drill, `45 65 13` | Supply rotation with the lever on Clutch `44 66 13`. Place stone in front of the head at `46 65 13`. | Breaking progress appears, the block breaks, and drops collect in chest `47 64 13`. Progress pauses without rotation. Bedrock is not broken. |
| Crafting and items | Use the crafting table and materials; inspect names and models in hand/inventory, changing language if desired. | Implemented items use original IDs, translations and locally imported models/textures. New machines have ordinary recipes, rather than only command access. |
| Saving | Stop unfinished processing, leave the world, reopen it and restore power. | Inventories and supported progress fields are restored without extra items appearing. |

Wheat has chance bonuses of flour and seeds; their counts need not match between
separate world runs. For strict quantity comparison, use the deterministic
Millstone recipe: sandstone → sand.

This version does not yet include belts, contraptions, trains, Ponder,
fluid machines or complete schematics. Fan effects on mobs and
players and drills on moving contraptions are also unimplemented. Formats and IDs
are checked against Create 6.0.10; transferring a complete Java world between versions is not promised.
Verified automated scenarios and exact limits are listed in
[VALIDATION.md](VALIDATION.md) and [COMPATIBILITY.md](COMPATIBILITY.md).
