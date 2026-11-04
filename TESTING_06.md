# CreateFactory06 review route

Checklist for `0.6.0-alpha.2`: 13 stations, including the press over a Basin.
The exact final build, installed instance and run results are recorded
in the [stage report](REPORT_06.md). Earlier alpha.1 checks are preserved separately.
Older 0.5 worlds are kept separately. The source reference is Create 6.0.10 / Java
Minecraft 1.21.1; the port runs on Horizon / Minecraft 26.3.

Goggles are already equipped. The first nine slots contain a Wrench, motor, shaft,
cogwheel, Belt Connector, press, Basin, Mixer and burner. Two catalog barrels contain
all 47 supported public IDs. The inventory holds 36 distinct stacks; other items are available in the catalog.
Processing at the review stations is stopped until their motor RPM is changed.

| Check | Coordinates near the station | Action and expected result |
| --- | --- | --- |
| Catalog and resources | `3 64 4` | Items, blocks, hands and goggles use resources from your JAR. Switch RU/EN and check names. |
| Gauges and motor settings | `12 64 4` | Look at the gauges while wearing goggles. Change 32 RPM to 16/256 and back; check comparator output and direction. |
| Single-item press | `12 64 11` | Motor beside the press: 0→128 RPM. Eight ingots should become eight sheets, one per hit. Click the top of the Depot with an empty hand to collect everything. |
| Belt and press | `29 64 5` | Lower belt motor: 0→32 RPM. The press is already connected to 128 RPM. Eight ingots pass through the press, end Depot and ordinary hopper into the chest. An empty Depot leaves no sheet image. |
| Dry Basin and Mixer | `46 64 5` | Motor: 0→32 RPM. Four andesite and four iron nuggets become four alloy. Stopping and restarting preserves the input. |
| Water and dough | `57 64 11` | The Basin below the Mixer contains flour. Add a water bucket and enable 32 RPM: one flour and 1000 mB produce one dough. The other three flour remain. |
| Mud and block tag | `61 64 9` | Add water to the neighboring Basin and enable 32 RPM. Four original dirt blocks of different types produce four mud blocks at 250 mB each. |
| Basin contents and tooltip | `57 64 11` | A separate Basin contains 1000 mB each of water and lava. Use an empty bucket to collect water, then lava; return both. Check the tooltip with/without goggles, RU/EN. |
| Burner and brass | `72 64 5` | Add coal to the filled burner beneath the Mixer, then enable 32 RPM. Four copper and four zinc produce eight brass. Smouldering without fuel must not make brass. |
| Empty cage and fire | `75 64 8` | After acceptance of this scenario: Flint and Steel lights the empty cage, soul sand changes the fire without being consumed, a shovel extinguishes it; Wrench removal returns the correct item. |
| Five fan media | `9 64 28` | Lane motors: 0→32 RPM. Check exactly eight outputs: flour→dough, dough→bread, sand→soul sand, raw iron→ingot, beef→steak. Leave the water gates open. |
| Airflow and player | `18 64 27` | Enter the safe glass corridor without flying. 128 RPM pushes, −128 RPM pulls. Compare the feel of movement with Create. |
| Water wheels | `32 64 34` | The small wheel gives 8 RPM, gearing gives 16, and the large wheel gives 4. Remove/rotate with the Wrench; check the visual direction of every part. |
| Millstone and ordinary hoppers | `44 64 17` | Motor: 0→128 RPM. 32 wheat are milled; flour and chance seeds collect in the chest. Check stopping and a full output. |
| Stationary drill | `45 64 28` | Toggle the Clutch lever. The stone breaks and one new cobblestone enters the chest. Disable it and place the next stone manually. |
| Hand Crank and Gearbox | `44 64 37` | Right-click drives rotation, Shift reverses direction, and the lever toggles the Gearshift. Check the connected shaft animations. |
| Survival crafting | `16 64 39` | `/gamemode survival`; materials are in three barrels. Craft a whisk, basin, mechanical mixer, belt connector, empty blaze burner and dough. Crafting dough must return the bucket. |
| Schematic | `68 64 37` | In creative: `/create schematic inspect own-press.nbt`, then `/create schematic paste own-press.nbt 64 64 32`. Five blocks are pasted. Place one Copper Ingot on Depot `66 64 32`: it becomes one sheet. |
| Schematic refusal | `68 64 37` | `/create schematic paste unsupported-belt.nbt 70 64 32` must report unsupported data without changing blocks. Files are in `schematics/` beside the world. |
| Press over Basin | `68 64 27` | Motors `58/64/70 66 20`: 0→32 RPM. Basins `60/66/72 64 20` should produce two snow blocks, one ice and one andesite, respectively. The third Basin retains 900 mB lava. No burner is required. An empty hand collects the output. |
| Saving | Any station | Stop/start a process, leave the world and restart. Items, components, water/lava, settings and the constructed belt must persist. |

## Known limits

- Command-based schematic paste is limited to supported states and data.
  Preview, original schematic items, rotations/mirrors and Schematicannon are unfinished.
  Belts and unknown block entities are rejected before writing.
- The presence of 47 IDs does not make all of Create complete. Full belts, funnels,
  pipes, contraptions, trains and Ponder need separate stages.
- Coarse darkening of burner OBJ parts is fixed, and the hat renders. Full
  lighting parity for JSON parts, goggles and animation phases with Java is not accepted.
- The press over a Basin starts one tick earlier than Java after network startup;
  repeated-hit intervals and results match. This remains an open difference.
- Basin and Mixer tooltips were compared with Java; this does not validate every screen, effect,
  Basin content display or sound. Record small differences too.
- Do not close the gates in the washing lane: they provide the physical water boundary
  and an open path for airflow. Fluid around neighboring campfires is a bug.
- Ordinary item drops into the Basin and Millstone were checked in separate physics
  scenarios; this is not every form of automation. Complete world compatibility across
  Java/Horizon versions is not promised.

A report needs the station number, action, expected result and actual result.
Save the world after experimenting: it can help reproduce the difference.
The overall TODO table is [ROADMAP.md](ROADMAP.md); evidence and
clarifications are in [DEVELOPMENT_06.md](DEVELOPMENT_06.md).
