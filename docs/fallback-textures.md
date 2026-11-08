# Built-in texture register

The default art uses new pixel textures on the port's own block and item models.
The visual reference is Create 6.0.10. Read its shapes, use cases, material
families and color cues for the brief. Draw independent artwork. Keep original
Create artwork in the player-supplied JAR. The original resource pack takes
priority when the JAR is installed.

The shipped tiles are 16 × 16 pixels. Block textures use the existing
model's 0–16 face UV coordinates. The tank glass preserves transparency.
Item sprites are RGBA with transparent surroundings and map to `layer0`
of the generated item model. Models with geometry, including the Empty
Blaze Burner, use their block materials in inventory. Adjacent parts can
share a material tile. Model geometry, face selection and state rotations
define the visible shape. The animated fire effect uses Minecraft's fire
sprite in fallback mode.

## Adding a texture

1. Add a row to the register with the output ID, dimensions, material,
   identifying feature and intended alpha. Choose an existing tile for a
   genuinely shared surface.
2. Generate a sheet with the built-in ImageGen tool. Prompt for flat
   steam-age voxel pixel art, exact equal grid cells, the table's order
   from left to right, and no margins, labels, perspective or lighting.
   Block cells fill the square. Item cells contain one centered icon
   surrounded by transparency. The individual prompts use the briefs
   in the corresponding rows below.
3. Export each cell at 16 × 16 using nearest-neighbor sampling. Center
   item art at its original aspect ratio and keep clean alpha edges.
   Put the PNGs in `content/assets/create/textures/{block,item}/fallback/`.
4. Bind the model in `tools/fallback_textures.py`, run `tools/content.py`
   to regenerate the model JSON and native archive, and run
   `tools/check_distribution.py` to check coverage and dimensions.
5. Inspect the updated gallery once with bundled art and once with the
   local JAR. Remove the JAR and check that the built-in art returns.

The art in `fallback/` is licensed under the repository's MIT license.

## Kinetics sheet

Output folder `block/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `steel_axle.png` | 16 × 16 | Cool steel shaft stock with straight machined longitudinal highlights and graphite seams. |
| 1, 2 | `oak_cog.png` | 16 × 16 | Honey-brown laminated hardwood with radial grain and dark tooth roots. |
| 1, 3 | `andesite_casing.png` | 16 × 16 | Slate-gray mineral panel with beveled pale border and small steel rivets. |
| 1, 4 | `brass_casing.png` | 16 × 16 | Antique honey brass frame with warm wooden center and dark fasteners. |
| 2, 1 | `copper_casing.png` | 16 × 16 | Burnished orange copper sheet with dark border seams and four matching rivets. |
| 2, 2 | `creative_motor.png` | 16 × 16 | Red-orange motor casing with brass corners and black industrial vent slots. |
| 2, 3 | `gearbox.png` | 16 × 16 | Gray steel gear housing with brass bearing race and black recessed ring. |
| 2, 4 | `clutch.png` | 16 × 16 | Cool gray clutch housing with cream disengagement stripe and bronze center rim. |
| 3, 1 | `gearshift.png` | 16 × 16 | Graphite transmission housing with muted vermilion direction chevrons. |
| 3, 2 | `chain_drive.png` | 16 × 16 | Gray steel housing with a narrow black chain track and repeating silver links. |
| 3, 3 | `hand_crank.png` | 16 × 16 | Warm spruce wood and dark ferrules with ribbed grip grain. |
| 3, 4 | `flywheel.png` | 16 × 16 | Dark bronzed cast metal with narrow polished edge and dark wheel groove. |
| 4, 1 | `speedometer.png` | 16 × 16 | Cream gauge plate with green and amber tick marks around a dark hub. |
| 4, 2 | `stressometer.png` | 16 × 16 | Dark gauge plate with red and cream load marks around a dark hub. |
| 4, 3 | `gauge_needle.png` | 16 × 16 | Saturated vermilion enamel with pale red center and dark border. |
| 4, 4 | `water_wheel.png` | 16 × 16 | Dark warm wooden paddles, parallel narrow plank seams and bronze pins. |

## Processing sheet

Output folder `block/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `press_case.png` | 16 × 16 | Charcoal steel press housing with dark hydraulic vents and safety ochre band. |
| 1, 2 | `depot_rim.png` | 16 × 16 | Burnt orange warehouse work platform with a dark framed tray and bolt heads. |
| 1, 3 | `millstone.png` | 16 × 16 | Pale basalt-gray millstone with shallow grooves and chipped worn edges. |
| 1, 4 | `fan_case.png` | 16 × 16 | Andesite-colored square fan shroud with dark vent grille and four bolts. |
| 2, 1 | `fan_blade.png` | 16 × 16 | Pale brushed steel fan blades with light blue highlights along long edge. |
| 2, 2 | `drill_case.png` | 16 × 16 | Deep dark gray drill housing with diagonal muted bronze guard bands. |
| 2, 3 | `cutting_steel.png` | 16 × 16 | Satin gray hardened steel with sharply beveled pale cutting edges. |
| 2, 4 | `basin.png` | 16 × 16 | Deep charcoal metal with broad dark inside and warm brown rubbed rim. |
| 3, 1 | `mixer_case.png` | 16 × 16 | Medium brown riveted steel with gold spindle collars and dark lower rim. |
| 3, 2 | `burner_case.png` | 16 × 16 | Dark graphite iron fire cage with soot and desaturated rust corner studs. |
| 3, 3 | `burner_hot.png` | 16 × 16 | Flat golden fire creature surface, dark ember eye-like square apertures and orange edges. |
| 3, 4 | `crusher_wheel.png` | 16 × 16 | Heavy dark taupe stone with carved diagonal tooth bands and worn lighter ridges. |
| 4, 1 | `brass_cog.png` | 16 × 16 | Warm brass gear tooth metal with dark bronze roots and bright edge. |
| 4, 2 | `cold_flame.png` | 16 × 16 | Orange and golden angular ember pattern with intense bright core. |
| 4, 3 | `superheated.png` | 16 × 16 | Blue-white hot metal with cyan vents and indigo edges. |
| 4, 4 | `cranial_goggles.png` | 16 × 16 | Dark brown leather, bronze rim and cool blue glass detail. |

## Logistics sheet

Output folder `block/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `belt_rubber.png` | 16 × 16 | Nearly black brown fabric rubber with dark brown evenly spaced horizontal tread bars. |
| 1, 2 | `chute.png` | 16 × 16 | Warm gray steel funnel plates with vertical seams and a dark mouth lip. |
| 1, 3 | `smart_chute.png` | 16 × 16 | Gray and brass sorting chute plates with a dark filter slot and amber status stud. |
| 1, 4 | `brass_funnel.png` | 16 × 16 | Gold brass funnel plates with tan edge lip and dark throat stripes. |
| 2, 1 | `andesite_funnel.png` | 16 × 16 | Gray stone and steel funnel plates with a dark throat and cool rim. |
| 2, 2 | `crafter.png` | 16 × 16 | Deep wooden mechanism panel with inset brass grid and dark edge frame. |
| 2, 3 | `pipe_copper.png` | 16 × 16 | Deep orange aged copper tubing with darker seam and narrow ochre bands. |
| 2, 4 | `pump.png` | 16 × 16 | Orange copper pump face with steel flange, round inlet detail and small brass marks. |
| 3, 1 | `valve.png` | 16 × 16 | Dark brown copper valve collar with red-orange accent and steel spindle markings. |
| 3, 2 | `fluid_tank.png` | 16 × 16 | Copper tank panel with vertical riveted seams and very dark bottom border. |
| 3, 3 | `drain.png` | 16 × 16 | Copper collecting tray with repeating dark gray metal grate pattern. |
| 3, 4 | `cargo_dock.png` | 16 × 16 | Gray iron telescoping dock with thin gold collar bands and dark central bumper. |
| 4, 1 | `piston.png` | 16 × 16 | Wooden linear drive casing with dark steel bands and a gray compression collar. |
| 4, 2 | `sticky_piston.png` | 16 × 16 | Dark warm timber with sage green tack pad accent and iron piston bands. |
| 4, 3 | `gantry.png` | 16 × 16 | Narrow iron rail motif with alternating bronze rack teeth and dark channel. |
| 4, 4 | `sticker.png` | 16 × 16 | Gunmetal clamp with a lime green tack accent and inset red signal square. |

## Motion sheet

Output folder `block/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `bearing.png` | 16 × 16 | Andesite mounting plate with a pale central bearing race and four dark bolts. |
| 1, 2 | `windmill.png` | 16 × 16 | Golden timber hub face with dark cast-metal ring and pale cloth accent. |
| 1, 3 | `clockwork.png` | 16 × 16 | Bronze mounting plate with toothed gold ring and pale clockface indices. |
| 1, 4 | `pulley.png` | 16 × 16 | Warm charcoal cast-metal winch case with two slim brass cheeks and copper pins. |
| 2, 1 | `rope.png` | 16 × 16 | Muted tan twisted hemp rope with visible repeating diagonal strand texture. |
| 2, 2 | `chassis.png` | 16 × 16 | Spruce structural timber slats with dark iron mounting straps and pale pin heads. |
| 2, 3 | `sail_canvas.png` | 16 × 16 | Off-white woven cotton with tan stitched perimeter and subtle fiber dither. |
| 2, 4 | `harvester.png` | 16 × 16 | Pale steel teeth and tan grain guard on a dark brown frame. |
| 3, 1 | `saw.png` | 16 × 16 | Silver toothed cutting steel with a black hub and dark-gray directional bands. |
| 3, 2 | `plough.png` | 16 × 16 | Blackened steel plow plate with pale scraped leading edge and dark earth tint. |
| 3, 3 | `deployer.png` | 16 × 16 | Brass actuator panel with ribbed gold sleeve and blue-black cuff. |
| 3, 4 | `zinc_ore.png` | 16 × 16 | Neutral gray rock with cool pale blue-white angular zinc nodules. |
| 4, 1 | `deepslate_zinc_ore.png` | 16 × 16 | Dark charcoal layered rock with cool gray-blue angular zinc nodules. |
| 4, 2 | `raw_zinc_block.png` | 16 × 16 | Irregular cool slate-blue chunks with pale zinc highlights. |
| 4, 3 | `industrial_iron.png` | 16 × 16 | Neutral medium graphite industrial panel, riveted edges and pale seam. |
| 4, 4 | `weathered_iron.png` | 16 × 16 | Dark muted iron plating with worn slate patina, oxidized speckling and seam. |

## Masonry sheet

Output folder `block/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `cut_andesite.png` | 16 × 16 | Medium slate-gray squared natural andesite face with fine mineral flecks. |
| 1, 2 | `polished_cut_andesite.png` | 16 × 16 | Smooth cool gray stone slab tile with a fine beveled border. |
| 1, 3 | `cut_andesite_bricks.png` | 16 × 16 | Large medium gray rectangular andesite bricks in horizontal staggered courses. |
| 1, 4 | `small_andesite_bricks.png` | 16 × 16 | Small slate-gray andesite masonry in staggered courses. |
| 2, 1 | `layered_andesite.png` | 16 × 16 | Thin stacked bands of gray stone in alternating charcoal and light gray. |
| 2, 2 | `cut_granite.png` | 16 × 16 | Warm brown rose granite tile with mottled fine speckles. |
| 2, 3 | `polished_cut_granite.png` | 16 × 16 | Polished warm reddish granite tile with a thin edge. |
| 2, 4 | `cut_granite_bricks.png` | 16 × 16 | Large desaturated reddish granite bricks with dark horizontal mortar. |
| 3, 1 | `small_granite_bricks.png` | 16 × 16 | Small reddish gray granite bricks arranged in staggered rows. |
| 3, 2 | `cut_diorite.png` | 16 × 16 | Chalky ivory speckled stone tile with gray flecks and subdued edge. |
| 3, 3 | `polished_cut_diorite.png` | 16 × 16 | Bright ivory diorite tile with a smooth beveled trim. |
| 3, 4 | `cut_diorite_bricks.png` | 16 × 16 | Large cream white stone bricks with pale gray joints in staggered rows. |
| 4, 1 | `small_diorite_bricks.png` | 16 × 16 | Small ivory and pale gray diorite masonry. |
| 4, 2 | `zinc_block.png` | 16 × 16 | Cool desaturated blue-gray metallic ingot block with thin polished seams. |
| 4, 3 | `brass_block.png` | 16 × 16 | Warm golden honey brass metallic storage block with bevel and dark seam. |
| 4, 4 | `alloy_block.png` | 16 × 16 | Granular dark gray andesite alloy with pale steel flecks and bevel. |

## Materials sheet

Output folder `item/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `iron_sheet.png` | 16 × 16 | Flat diagonal thin gray rectangular plate with a pale steel edge. |
| 1, 2 | `golden_sheet.png` | 16 × 16 | Flat diagonal thin bright gold rectangular plate with a cream highlight. |
| 1, 3 | `copper_sheet.png` | 16 × 16 | Flat diagonal thin copper-orange rectangular plate with pale rim. |
| 1, 4 | `brass_sheet.png` | 16 × 16 | Flat diagonal thin antique mustard-brass plate with dark ochre edge. |
| 2, 1 | `andesite_alloy.png` | 16 × 16 | Short rough dark gray andesite-steel bar with pale flecks. |
| 2, 2 | `brass_ingot.png` | 16 × 16 | Slanted trapezoid of deep honey brass with gold edges. |
| 2, 3 | `zinc_ingot.png` | 16 × 16 | Slanted trapezoid of cool blue-gray zinc with pale blue highlight. |
| 2, 4 | `brass_nugget.png` | 16 × 16 | Small angular dark gold nugget with golden edge. |
| 3, 1 | `zinc_nugget.png` | 16 × 16 | Small angular cool pale blue-gray zinc nugget. |
| 3, 2 | `copper_nugget.png` | 16 × 16 | Small angular copper-orange nugget, russet shadow and pale rim. |
| 3, 3 | `raw_zinc.png` | 16 × 16 | Three rough pale blue-gray ore chunks joined in a low uneven cluster. |
| 3, 4 | `crushed_raw_copper.png` | 16 × 16 | Pile of small crushed terracotta copper fragments with dark edge. |
| 4, 1 | `crushed_raw_gold.png` | 16 × 16 | Pile of small ochre-gold crushed ore grains with pale yellow flecks. |
| 4, 2 | `crushed_raw_iron.png` | 16 × 16 | Pile of crushed warm gray iron ore with taupe and white flecks. |
| 4, 3 | `crushed_raw_zinc.png` | 16 × 16 | Pile of pale cool blue-gray crushed zinc grains. |
| 4, 4 | `powdered_obsidian.png` | 16 × 16 | Small angular nearly black violet powder mound with lavender glints. |

## Tools sheet

Output folder `item/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `wrench.png` | 16 × 16 | Diagonal brass open-jaw wrench with brown wrapped grip and dark outline. |
| 1, 2 | `goggles.png` | 16 × 16 | Two connected circular teal lenses with gold rims and brown head strap. |
| 1, 3 | `belt_connector.png` | 16 × 16 | Dark charcoal rubber loop wound into a ring with chestnut tread bands. |
| 1, 4 | `propeller.png` | 16 × 16 | Four broad gray silver propeller blades around bronze center cap. |
| 2, 1 | `whisk.png` | 16 × 16 | Silver wire loop whisk head with a short bronze handle. |
| 2, 2 | `super_glue.png` | 16 × 16 | Small squat mint-green glue bottle with cream band and dark cap. |
| 2, 3 | `sand_paper.png` | 16 × 16 | Square bent sheet of pale ochre abrasive paper with dark speckles. |
| 2, 4 | `red_sand_paper.png` | 16 × 16 | Square bent sheet of brick-red abrasive paper with dark speckles. |
| 3, 1 | `vertical_gearbox.png` | 16 × 16 | Compact cool gray cubic gearbox with upright brass bearing and dark shaft. |
| 3, 2 | `brass_hand.png` | 16 × 16 | Small golden mechanical hand with articulated fingers and dark wrist cuff. |
| 3, 3 | `crafter_slot_cover.png` | 16 × 16 | Thin rectangular brass slotted panel with dark engraved inner border. |
| 3, 4 | `precision_mechanism.png` | 16 × 16 | Complex gold gear with inset gray spokes and copper pins. |
| 4, 1 | `incomplete_precision_mechanism.png` | 16 × 16 | Partly assembled dark brass plate with missing pieces and one exposed cog. |
| 4, 2 | `electron_tube.png` | 16 × 16 | Small upright pale cyan glass tube with two dark metallic pins and rose core. |
| 4, 3 | `rose_quartz.png` | 16 × 16 | Rough coral-pink shard with angular facet and dark red outline. |
| 4, 4 | `polished_rose_quartz.png` | 16 × 16 | Smooth compact coral-pink faceted prism with pale diagonal highlight. |

## Supplies sheet

Output folder `item/fallback/`. The source grid has 2 columns and 3 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `experience_nugget.png` | 16 × 16 | Tiny bright lime faceted droplet with yellow highlight and dark green rim. |
| 1, 2 | `wheat_flour.png` | 16 × 16 | Small soft mound of ivory flour with warm shadow and loose grains. |
| 2, 1 | `dough.png` | 16 × 16 | Lumpy pale cream dough ball with beige folded shadow. |
| 2, 2 | `blaze_cake.png` | 16 × 16 | Compact brown fuel pastry with golden orange fiery top and ember glints. |
| 3, 1 | `creative_blaze_cake.png` | 16 × 16 | Compact dark pastry with vivid cyan-blue flame and violet glowing rim. |
| 3, 2 | Empty Blaze Burner study | Source sheet only | Small black wrought iron cage with copper rim and hollow dark interior. The shipped item uses its 3D cage model. |

## Cloth sheet

Output folder `block/fallback/`. The source grid has 4 columns and 4 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `white_canvas.png` | 16 × 16 | white sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 1, 2 | `orange_canvas.png` | 16 × 16 | orange sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 1, 3 | `magenta_canvas.png` | 16 × 16 | magenta sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 1, 4 | `light_blue_canvas.png` | 16 × 16 | light blue sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 2, 1 | `yellow_canvas.png` | 16 × 16 | yellow sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 2, 2 | `lime_canvas.png` | 16 × 16 | lime sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 2, 3 | `pink_canvas.png` | 16 × 16 | pink sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 2, 4 | `gray_canvas.png` | 16 × 16 | gray sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 3, 1 | `light_gray_canvas.png` | 16 × 16 | light gray sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 3, 2 | `cyan_canvas.png` | 16 × 16 | cyan sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 3, 3 | `purple_canvas.png` | 16 × 16 | purple sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 3, 4 | `blue_canvas.png` | 16 × 16 | blue sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 4, 1 | `brown_canvas.png` | 16 × 16 | brown sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 4, 2 | `green_canvas.png` | 16 × 16 | green sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 4, 3 | `red_canvas.png` | 16 × 16 | red sail cloth with low-contrast woven cotton pattern and stitches along edges. |
| 4, 4 | `black_canvas.png` | 16 × 16 | black sail cloth with low-contrast woven cotton pattern and stitches along edges. |

## Wood sheet

Output folder `block/fallback/`. The source grid has 4 columns and 3 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `oak_paddles.png` | 16 × 16 | gold tan oak timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 1, 2 | `spruce_paddles.png` | 16 × 16 | dark resin brown spruce timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 1, 3 | `birch_paddles.png` | 16 × 16 | pale golden cream birch timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 1, 4 | `jungle_paddles.png` | 16 × 16 | warm medium brown jungle timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 2, 1 | `acacia_paddles.png` | 16 × 16 | warm orange red acacia timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 2, 2 | `dark_oak_paddles.png` | 16 × 16 | dark chocolate dark oak timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 2, 3 | `mangrove_paddles.png` | 16 × 16 | dark muted mahogany mangrove timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 2, 4 | `cherry_paddles.png` | 16 × 16 | warm pale blush cherry timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 3, 1 | `bamboo_paddles.png` | 16 × 16 | pale warm straw bamboo timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 3, 2 | `crimson_paddles.png` | 16 × 16 | deep burgundy red crimson timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 3, 3 | `warped_paddles.png` | 16 × 16 | muted turquoise teal warped timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |
| 3, 4 | `pale_oak_paddles.png` | 16 × 16 | pale muted cool beige pale oak timber water-wheel paddle grain, several narrow vertical planks and two dark pinheads. |

## Glazing sheet

Output folder `block/fallback/`. The source grid has 1 columns and 1 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `tank_glass.png` | 16 × 16 | Nearly clear glass inspection pane with pale cool steel-blue edge bevel and two subdued diagonal reflection lines, transparent middle. |

## Status and end-grain sheet

Output folder `block/fallback/`. The source grid has 3 columns and 2 rows.

| Cell | PNG | Pixels | Shape and surface |
| --- | --- | --- | --- |
| 1, 1 | `smart_chute_powered.png` | 16 × 16 | Slate iron and antique brass Smart Chute sorting face, dark slot and distinctly glowing small red square over it. |
| 1, 2 | `sticker_powered.png` | 16 × 16 | Gunmetal clamp panel with a red status lamp and a sage tack pad against black recessed strips. |
| 1, 3 | `cargo_dock_powered.png` | 16 × 16 | Grey iron telescoping cargo dock, copper-orange bright horizontal collar stripes and dark central bumper. |
| 2, 1 | `andesite_funnel_active.png` | 16 × 16 | Cool pale ash gray andesite hopper panel with steel edges and broad dark lower opening. |
| 2, 2 | `brass_funnel_active.png` | 16 × 16 | Burnished copper-orange and honey-brass hopper panel with black lower throat and amber rim. |
| 2, 3 | `chassis_end.png` | 16 × 16 | Light honey-brown cut timber end face with concentric squared growth rings and a small steel center pin. |
