#!/usr/bin/env python3
"""Generate mod content with simple built-in models.

Models reference vanilla textures by ID and contain no upstream texture bytes.
"""
import json
from common import ROOT, NATIVE

CONTENT = ROOT / "content"
BLOCKS = ["creative_motor", "shaft", "cogwheel", "large_cogwheel", "mechanical_press", "depot"]
SHEETS = ["iron", "golden", "copper"]


def write(path, value):
    target = CONTENT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def model(name, boxes, texture="minecraft:block/iron_block", *, rotation=None):
    sides = ("down", "up", "north", "south", "west", "east")
    elements = []
    for start, end in boxes:
        face = {"texture": "#all"}
        # Generated UVs can sample neighbouring atlas sprites beyond a block.
        if any(value < 0 or value > 16 for value in [*start, *end]):
            face["uv"] = [0, 0, 16, 16]
        element = {"from": start, "to": end, "faces": dict.fromkeys(sides, face)}
        if rotation is not None:
            element["rotation"] = rotation
        elements.append(element)
    write("assets/create/models/block/" + name + ".json", {
        "textures": {"all": texture, "particle": texture},
        "elements": elements,
    })


def processors():
    # State and collision definitions translated from MIT MillstoneBlock,
    # DirectionalKineticBlock and AllShapes. Geometry below is our provisional
    # design using vanilla textures, not a copy of the original ARR models.
    shapes = {"default": [[0, 0, 0, 1, 6 / 16, 1], [2 / 16, 6 / 16, 2 / 16, 14 / 16, 1, 14 / 16]]}
    write("data/create/horizon/block/millstone.json", {
        "template": "minecraft:glass", "item": True,
        "properties": {"hardness": 1.5, "explosion_resistance": 6, "requires_tool": True, "map_color": 6,
                       "render_shape": "MODEL", "sounds": "minecraft:andesite"},
        "shapes": shapes, "collision_shapes": shapes,
    })
    full = {"default": [[0, 0, 0, 1, 1, 1]]}
    write("data/create/horizon/block/encased_fan.json", {
        "template": "minecraft:glass", "item": True,
        "properties": {"hardness": 1.5, "explosion_resistance": 6, "requires_tool": True, "map_color": 34,
                       "render_shape": "MODEL", "sounds": "minecraft:andesite"},
        "state_properties": {"facing": {"type": "enum", "values": ["down", "up", "north", "south", "west", "east"]}},
        # 1.21.1 DirectionProperty's implicit first value is DOWN. The source
        # fan does not override it; placement supplies its actual direction.
        "default_state": {"facing": "down"}, "shapes": full, "collision_shapes": full,
    })
    drill_shapes = {"property": "facing", "values": {
        "up": [[0, 0, 0, 1, .75, 1]], "down": [[0, .25, 0, 1, 1, 1]],
        "north": [[0, 0, .25, 1, 1, 1]], "south": [[0, 0, 0, 1, 1, .75]],
        "west": [[.25, 0, 0, 1, 1, 1]], "east": [[0, 0, 0, .75, 1, 1]],
    }}
    write("data/create/horizon/block/mechanical_drill.json", {
        "template": "minecraft:glass", "item": True,
        "properties": {"hardness": 1.5, "explosion_resistance": 6, "requires_tool": True,
                       "map_color": 34, "render_shape": "MODEL", "sounds": "minecraft:andesite"},
        "state_properties": {"facing": {"type": "enum", "values": ["down", "up", "north", "south", "west", "east"]},
                             "waterlogged": {"type": "boolean"}},
        "default_state": {"facing": "down", "waterlogged": "false"},
        "shapes": drill_shapes, "collision_shapes": drill_shapes,
    })
    model("millstone/block", [([0, 0, 0], [16, 5, 16]),
                               ([2, 5, 2], [5, 16, 14]), ([11, 5, 2], [14, 16, 14]),
                               ([5, 5, 2], [11, 16, 5]), ([5, 5, 11], [11, 16, 14])], "minecraft:block/andesite")
    model("millstone/inner", [([5, 5, 5], [11, 12, 11]), ([6, 12, 7], [10, 15, 9])], "minecraft:block/smooth_stone")
    # The source static fan model faces up before blockstate rotations.
    model("encased_fan/block", [([0, 0, 0], [16, 3, 16]),
                                 ([0, 3, 0], [3, 16, 16]), ([13, 3, 0], [16, 16, 16]),
                                 ([3, 3, 0], [13, 16, 3]), ([3, 3, 13], [13, 16, 16])], "minecraft:block/stripped_spruce_log")
    # The native renderer faces the moving partial along the rear shaft.
    model("encased_fan/propeller", [([3, 6, 7], [13, 10, 9]), ([6, 3, 7], [10, 13, 9]), ([6, 6, 5], [10, 10, 11])])
    model("mechanical_drill/block", [([0, 0, 0], [16, 10, 16]), ([2, 10, 2], [14, 12, 14])], "minecraft:block/andesite")
    model("mechanical_drill/head", [([4, 12, 4], [12, 13, 12]), ([5, 13, 5], [11, 14, 11]), ([7, 14, 7], [9, 16, 9])])
    for block in ["millstone", "encased_fan", "mechanical_drill"]:
        variants = {"": {"model": "create:block/" + block + "/block"}}
        if block != "millstone":
            variants = {"facing=" + face: {"model": "create:block/" + block + "/block", **rotation}
                        for face, rotation in {"down": {"x": 180}, "up": {}, "north": {"x": 90},
                                               "south": {"x": 90, "y": 180}, "west": {"x": 90, "y": 270},
                                               "east": {"x": 90, "y": 90}}.items()}
            if block == "mechanical_drill":
                variants = {key + ",waterlogged=" + wet: value for key, value in variants.items() for wet in ["false", "true"]}
        write("assets/create/blockstates/" + block + ".json", {"variants": variants})
        write("assets/create/models/item/" + block + ".json", {"parent": "create:block/" + block + "/block"})
        write("assets/create/items/" + block + ".json", {"model": {"type": "minecraft:model", "model": "create:item/" + block}})
        write("data/create/loot_table/blocks/" + block + ".json", {"type": "minecraft:block", "pools": [{"rolls": 1, "entries": [{"type": "minecraft:item", "name": "create:" + block}]}]})


def basin_mixer():
    # MIT AllShapes and BasinBlock/MechanicalMixerBlock context dispatch.
    basin = {'default': [[0, 2/16, 0, 2/16, 1, 1],
                         [14/16, 2/16, 0, 1, 1, 1],
                         [2/16, 2/16, 0, 14/16, 1, 2/16],
                         [2/16, 2/16, 14/16, 14/16, 1, 1],
                         [2/16, 0, 2/16, 14/16, 2/16, 14/16]]}
    mixer = {'default': [[0, 0, 0, 4/16, 1, 1], [12/16, 0, 0, 1, 1, 1],
                         [4/16, 0, 0, 12/16, 1, 4/16], [4/16, 0, 12/16, 12/16, 1, 1]]}
    basin_item = {'default': [[0, 2/16, 0, 2/16, 13/16, 1],
                              [14/16, 2/16, 0, 1, 13/16, 1],
                              [2/16, 2/16, 0, 14/16, 13/16, 2/16],
                              [2/16, 2/16, 14/16, 14/16, 13/16, 1],
                              [2/16, 0, 2/16, 14/16, 5/16, 14/16]]}
    basin_interaction = {'default': [[0, 2/16, 0, 1, 1, 1],
                                     [2/16, 0, 2/16, 14/16, 2/16, 14/16]]}
    write('data/create/horizon/block/basin.json', {
        'template': 'minecraft:glass', 'item': True,
        'properties': {'hardness': 1.5, 'explosion_resistance': 6, 'requires_tool': True,
                       'map_color': 21, 'sounds': 'minecraft:netherite_block', 'render_shape': 'MODEL'},
        'state_properties': {'facing': {'type': 'enum', 'values': ['down', 'north', 'south', 'west', 'east']}},
        'default_state': {'facing': 'down'}, 'shapes': basin, 'collision_shapes': basin,
        'collision_context_shapes': {'item': basin_item},
        'interaction_shapes': basin_interaction,
    })
    write('data/create/horizon/block/mechanical_mixer.json', {
        'template': 'minecraft:glass', 'item': True,
        'properties': {'hardness': 1.5, 'explosion_resistance': 6, 'requires_tool': True,
                       'map_color': 11, 'sounds': 'minecraft:stone', 'render_shape': 'MODEL'},
        'shapes': mixer, 'collision_shapes': mixer,
        'outline_context_shapes': {'player': {'default': [[0, 2/16, 0, 1, 1, 1]]}},
    })
    # Our provisional geometry uses vanilla textures and contains no original
    # model bytes. The local JAR importer replaces it with the genuine parts.
    bowl = [([1, 2, 1], [3, 15, 15]), ([13, 2, 1], [15, 15, 15]),
            ([3, 2, 1], [13, 15, 3]), ([3, 2, 13], [13, 15, 15]), ([3, 0, 3], [13, 2, 13])]
    model('basin/block', bowl, 'minecraft:block/iron_block')
    model('basin/block_directional', bowl + [([5, 9, 14], [11, 12, 18])], 'minecraft:block/iron_block')
    model('mechanical_mixer/block', [([0, 10, 0], [4, 16, 16]), ([12, 10, 0], [16, 16, 16]),
                                     ([4, 10, 0], [12, 16, 4]), ([4, 10, 12], [12, 16, 16])], 'minecraft:block/andesite')
    model('mechanical_mixer/pole', [([7, 2, 7], [9, 15, 9])])
    model('mechanical_mixer/head', [([4, 0, 7], [12, 2, 9]), ([7, 0, 4], [9, 2, 12]), ([7, 2, 7], [9, 8, 9])])
    model('cogwheel_shaftless', [([2, 6, 2], [14, 10, 14]), ([0, 5, 6], [16, 11, 10]),
                                ([6, 5, 0], [10, 11, 16])], 'minecraft:block/oak_planks')
    write('assets/create/blockstates/basin.json', {'variants': {
        'facing=down': {'model': 'create:block/basin/block'},
        **{'facing=' + face: {'model': 'create:block/basin/block_directional', **({'y': angle} if angle else {})}
           for face, angle in [('south', 0), ('west', 90), ('north', 180), ('east', 270)]},
    }})
    write('assets/create/blockstates/mechanical_mixer.json', {'variants': {'': {'model': 'create:block/mechanical_mixer/block'}}})
    for block in ['basin', 'mechanical_mixer']:
        write('assets/create/models/item/' + block + '.json', {'parent': 'create:block/' + block + '/block'})
        write('assets/create/items/' + block + '.json', {'model': {'type': 'minecraft:model', 'model': 'create:item/' + block}})
        write('data/create/loot_table/blocks/' + block + '.json', {'type': 'minecraft:block', 'pools': [
            {'rolls': 1, 'entries': [{'type': 'minecraft:item', 'name': 'create:' + block}]},
        ]})


def burners():
    # MIT BlazeBurnerBlock/LitBlazeBurnerBlock and AllShapes. The static
    # fallback geometry below is ours; original models stay in the local JAR.
    heats = ['none', 'smouldering', 'fading', 'kindled', 'seething']
    outline = {'default': [[1/16, 0, 1/16, 15/16, 14/16, 15/16]]}
    collision = {'default': [[0, 0, 0, 1, 4/16, 1]]}
    common = {'template': 'minecraft:glass', 'shapes': outline, 'collision_shapes': collision,
              'collision_context_shapes': {kind: outline for kind in ['player', 'item', 'living', 'other']}}
    # Source emission is compiled once by Horizon's state_light selector.
    write('data/create/horizon/block/blaze_burner.json', {
        **common, 'item': True,
        'properties': {'hardness': 3, 'explosion_resistance': 6, 'requires_tool': True,
                       'map_color': 21, 'sounds': 'minecraft:gold_block', 'render_shape': 'MODEL'},
        'state_properties': {'blaze': {'type': 'enum', 'values': heats},
                             'facing': {'type': 'enum', 'values': ['north', 'south', 'west', 'east']}},
        'default_state': {'blaze': 'none', 'facing': 'north'},
        'state_light': {'property': 'blaze', 'values': dict(zip(heats, [0, 8, 15, 15, 15]))},
    })
    write('data/create/horizon/block/lit_blaze_burner.json', {
        **common, 'item': False,
        'properties': {'hardness': 3, 'explosion_resistance': 6, 'requires_tool': True,
                       'map_color': 22, 'sounds': 'minecraft:gold_block', 'render_shape': 'MODEL'},
        'state_properties': {'flame_type': {'type': 'enum', 'values': ['regular', 'soul']}},
        'default_state': {'flame_type': 'regular'},
        'state_light': {'property': 'flame_type', 'values': {'regular': 12, 'soul': 9}},
    })
    cage = [([0, 0, 0], [16, 2, 16])]
    cage += [([x, 2, z], [x+2, 14, z+2]) for x in [1, 13] for z in [1, 13]]
    model('blaze_burner/block', cage)
    model('blaze_burner/block_with_blaze', cage + [([5, 5, 5], [11, 11, 11])])
    for state in ['fire', 'soul_fire']:
        model('blaze_burner/block_with_' + state, cage)
    for part in ['inert', 'idle', 'active', 'super', 'super_active']:
        model('blaze_burner/blaze/' + part, [([5, 5, 5], [11, 11, 11])], 'minecraft:block/gold_block')
    for part in ['rods_small', 'rods_large', 'superheated_rods_small', 'superheated_rods_large']:
        model('blaze_burner/' + part, [([3, 0, 3], [5, 8, 5]), ([11, 0, 11], [13, 8, 13])], 'minecraft:block/gold_block')
    for part in ['goggles', 'goggles_small']:
        model('blaze_burner/' + part, [([4, 0, 3], [7, 3, 4]), ([9, 0, 3], [12, 3, 4])], 'minecraft:block/gold_block')
    model('blaze_burner/flame', [([7.99, 0, 0], [8.01, 16, 16])], 'minecraft:block/fire_0')
    for hat in ['train_hat', 'logistics_hat']:
        write('assets/create/models/entity/' + hat + '.json', {
            'parent': 'minecraft:block/block', 'textures': {'all': 'minecraft:block/black_wool'},
            'elements': [{'from': [3, 0, 3], 'to': [13, 3, 13], 'faces': {
                face: {'texture': '#all'} for face in ['down', 'up', 'north', 'south', 'west', 'east']}}],
        })
    write('assets/create/blockstates/blaze_burner.json', {'variants': {'': {'model': 'create:block/blaze_burner/block'}}})
    write('assets/create/blockstates/lit_blaze_burner.json', {'variants': {
        'flame_type=regular': {'model': 'create:block/blaze_burner/block_with_fire'},
        'flame_type=soul': {'model': 'create:block/blaze_burner/block_with_soul_fire'},
    }})
    for item, part in [('blaze_burner', 'block_with_blaze'), ('empty_blaze_burner', 'block')]:
        write('assets/create/models/item/' + item + '.json', {'parent': 'create:block/blaze_burner/' + part})
        write('assets/create/items/' + item + '.json', {'model': {'type': 'minecraft:model', 'model': 'create:item/' + item}})
    # BlazeBurnerBlock.buildLootTable predicates are unchanged. Minecraft 26.3
    # encodes each condition object's discriminator as `type`; the 1.21.1
    # `condition` discriminator silently fails Horizon's ordinary evaluator.
    write('data/create/loot_table/blocks/blaze_burner.json', {'type': 'minecraft:block', 'pools': [{
        'rolls': 1, 'entries': [{'type': 'minecraft:item',
            'name': 'create:empty_blaze_burner' if heat == 'none' else 'create:blaze_burner',
            'conditions': [{'type': 'minecraft:survives_explosion'}, {
                'type': 'minecraft:block_state_property', 'block': 'create:blaze_burner',
                'properties': {'blaze': heat}}]} for heat in heats],
    }]})
    write('data/create/loot_table/blocks/lit_blaze_burner.json', {'type': 'minecraft:block', 'pools': [{
        'rolls': 1, 'conditions': [{'type': 'minecraft:survives_explosion'}],
        'entries': [{'type': 'minecraft:item', 'name': 'create:empty_blaze_burner'}],
    }]})


def belts():
    from belt_shapes import geometry
    shapes, collisions = geometry()
    write('data/create/horizon/block/belt.json', {
        'template': 'minecraft:glass', 'item': False,
        'properties': {'hardness': .8, 'explosion_resistance': .8,
                       'sounds': 'minecraft:white_wool', 'render_shape': 'ENTITYBLOCK_ANIMATED',
                       'map_color': 22},
        'state_properties': {
            'facing': {'type': 'enum', 'values': ['north', 'south', 'west', 'east']},
            'slope': {'type': 'enum', 'values': ['horizontal', 'upward', 'downward', 'vertical', 'sideways']},
            'part': {'type': 'enum', 'values': ['start', 'middle', 'end', 'pulley']},
            'casing': {'type': 'boolean'}, 'waterlogged': {'type': 'boolean'}},
        'default_state': {'facing': 'north', 'slope': 'horizontal', 'part': 'start',
                          'casing': 'false', 'waterlogged': 'false'},
        'shapes': shapes, 'collision_shapes': collisions,
    })
    # Bare source belts have an empty static particle model; the native renderer
    # supplies original moving parts. Do not run Create's Java BeltModel loader.
    write('assets/create/blockstates/belt.json', {
        'variants': {'': {'model': 'create:block/belt/particle'}}})
    write('assets/create/models/block/belt/particle.json', {
        'textures': {'particle': 'minecraft:block/black_wool'}, 'elements': []})
    # The renderer reuses these parts for all directions. The built-in belt
    # surface is plain wool; only the optional JAR supplies scrolling textures.
    model('belt/middle', [([0, 11, 0], [16, 13, 16])], 'minecraft:block/black_wool')
    model('belt/middle_bottom', [([0, 3, 0], [16, 5, 16])], 'minecraft:block/black_wool')
    model('belt/diagonal_middle', [([0, 4, -3.314], [16, 12, 19.314])],
          'minecraft:block/black_wool', rotation={'origin': [8, 8, 8], 'axis': 'x', 'angle': 45})
    for prefix, suffix in [('', ''), ('', '_bottom'), ('diagonal_', '')]:
        for part in ['start', 'end']:
            write(f'assets/create/models/block/belt/{prefix}{part}{suffix}.json',
                  {'parent': f'create:block/belt/{prefix}middle{suffix}'})
    model('belt_pulley', [([2, 6, 2], [14, 10, 14]), ([5, 0, 5], [11, 16, 11])],
          'minecraft:block/iron_block')
    write('data/create/loot_table/blocks/belt.json', {'type': 'minecraft:block', 'pools': [
        {'rolls': 1, 'entries': [{'type': 'minecraft:item', 'name': 'create:belt_connector'}]},
        {'rolls': 1, 'conditions': [{'condition': 'minecraft:inverted', 'term': {
            'condition': 'minecraft:block_state_property', 'block': 'create:belt',
            'properties': {'part': 'middle'}}}],
         'entries': [{'type': 'minecraft:item', 'name': 'create:shaft'}]},
    ]})


def chain_drives():
    # MIT ChainDriveBlock / RotatedPillarKineticBlock: 24 states, full collision.
    # Original visual resources are imported privately from the player's JAR.
    write('data/create/horizon/block/encased_chain_drive.json', {
        'template': 'minecraft:glass', 'item': True,
        'properties': {'hardness': 1.5, 'explosion_resistance': 6, 'map_color': 34,
                       'render_shape': 'MODEL', 'sounds': 'minecraft:stone'},
        'state_properties': {'axis': {'type': 'enum', 'values': ['x', 'y', 'z']},
                             'axis_along_first': {'type': 'boolean'},
                             'part': {'type': 'enum', 'values': ['start', 'middle', 'end', 'none']}},
        # The pinned original registry's implicit BooleanProperty default is
        # true, verified in the actual Java client (not inferred from Rust).
        'default_state': {'axis': 'y', 'axis_along_first': 'true', 'part': 'none'},
        'shapes': {'default': [[0, 0, 0, 1, 1, 1]]},
        'collision_shapes': {'default': [[0, 0, 0, 1, 1, 1]]},
    })
    variants = {}
    # ChainDriveGenerator rotations. These fallback boxes are our own simple
    # geometry; no original asset JSON is copied into the public bundle.
    for axis in ['x', 'y', 'z']:
        for first in [False, True]:
            for part in ['start', 'middle', 'end', 'none']:
                if part == 'none':
                    suffix, x, y = 'single', 90 if axis == 'y' else 0, 90 if axis == 'x' else 0
                else:
                    suffix = ('middle' if part == 'middle' else 'end') + ('_vertical' if axis == 'y' else '_horizontal')
                    if axis == 'x':
                        x = (90 if first else 0) + (180 if part == 'start' else 0)
                    elif axis == 'z':
                        x = 0 if first else (270 if part == 'start' else 90)
                    else:
                        x = 0
                    flip = part == 'end' and not first or part == 'start' and first
                    y = (270 if first and part == 'end' else 90) if axis == 'z' else ((90 if first else 0) + (180 if flip else 0) if axis == 'y' else 0)
                key = f'axis={axis},axis_along_first={str(first).lower()},part={part}'
                variants[key] = {'model': 'create:block/encased_chain_drive/' + suffix,
                                 **({'x': x} if x else {}), **({'y': y} if y else {})}
    for suffix in ['single', 'middle_vertical', 'end_vertical', 'middle_horizontal', 'end_horizontal']:
        model('encased_chain_drive/' + suffix, [([0, 0, 0], [16, 16, 16])], 'minecraft:block/andesite')
    write('assets/create/blockstates/encased_chain_drive.json', {'variants': variants})
    write('assets/create/models/item/encased_chain_drive.json', {'parent': 'create:block/encased_chain_drive/single'})


def fallback_instruments_and_wheels():
    for gauge in ['speedometer', 'stressometer']:
        model(f'gauge/{gauge}/head', [([15.8, 3, 3], [16.5, 13, 13])],
              'minecraft:block/smooth_stone')
    model('gauge/dial', [([16.5, 5.25, 5.25], [17, 10.5, 6.25])],
          'minecraft:block/red_concrete')

    small = [([1, 6, 5], [15, 10, 11]), ([5, 6, 1], [11, 10, 15]),
             ([5, 0, 5], [11, 16, 11])]
    large = [([-16, 6, 4], [32, 10, 12]), ([4, 6, -16], [12, 10, 32]),
             ([5, 0, 5], [11, 16, 11])]
    model('water_wheel/wheel', small, 'minecraft:block/spruce_planks')
    model('large_water_wheel/block', large, 'minecraft:block/spruce_planks')
    write('assets/create/models/block/large_water_wheel/block_extension.json',
          {'parent': 'create:block/large_water_wheel/block'})
    write('assets/create/models/block/water_wheel/block.json',
          {'textures': {'particle': 'minecraft:block/spruce_planks'}, 'elements': []})
    write('assets/create/models/item/water_wheel.json', {'parent': 'create:block/water_wheel/wheel'})
    write('assets/create/models/item/large_water_wheel.json', {'parent': 'create:block/water_wheel/wheel'})
    woods = ['oak', 'spruce', 'birch', 'jungle', 'acacia', 'dark_oak', 'mangrove', 'cherry',
             'bamboo', 'crimson', 'warped', 'pale_oak']
    for part in ['water_wheel/wheel', 'large_water_wheel/block', 'large_water_wheel/block_extension']:
        for wood in woods:
            texture = f'minecraft:block/{wood}_planks'
            write(f'assets/create/models/block/{part}_material_{wood}.json',
                  {'parent': f'create:block/{part}', 'textures': {'all': texture, 'particle': texture}})


def foundation():
    from catalog import BLOCKS as all_blocks, ITEMS, TRANSMISSIONS, STATIC_BLOCKS, INSTRUMENTS, GENERATORS, INTERNAL_BLOCKS, rust_catalog
    for block in TRANSMISSIONS + STATIC_BLOCKS + INSTRUMENTS + GENERATORS:
        declaration = {"template": "minecraft:glass", "item": True,
                       "properties": {"hardness": 1.5, "sounds": "minecraft:andesite"}}
        if block in TRANSMISSIONS and block != "hand_crank":
            declaration["state_properties"] = {"axis": {"type": "enum", "values": ["x", "y", "z"]}}
            declaration["default_state"] = {"axis": "y"}
            if block in ["clutch", "gearshift"]:
                declaration["state_properties"]["powered"] = {"type": "boolean"}
                declaration["default_state"]["powered"] = "false"
        if block in INSTRUMENTS:
            declaration['state_properties']={'facing':{'type':'enum','values':['down','up','north','south','west','east']},'axis_along_first':{'type':'boolean'}}
            declaration['default_state']={'facing':'north','axis_along_first':'false'}
        if block in GENERATORS:
            declaration['properties']['render_shape']='ENTITYBLOCK_ANIMATED' if block=='large_water_wheel' else 'MODEL'
            declaration['properties']['sounds']='minecraft:stripped_spruce_wood'
            key='axis' if block=='large_water_wheel' else 'facing'
            values=['x','y','z'] if key=='axis' else ['down','up','north','south','west','east']
            declaration['state_properties']={key:{'type':'enum','values':values}}
            declaration['default_state']={key:'x' if key=='axis' else 'east'}
            if key=='axis':
                declaration['state_properties']['extension']={'type':'boolean'}
                declaration['default_state']['extension']='false'
        if block == "hand_crank":
            declaration["properties"]["sounds"] = "minecraft:stripped_spruce_wood"
            declaration["properties"]["render_shape"] = "ENTITYBLOCK_ANIMATED"
            declaration["state_properties"] = {"facing": {"type": "enum", "values": ["up", "down", "north", "south", "west", "east"]}, "waterlogged": {"type": "boolean"}}
            declaration["default_state"] = {"facing": "up", "waterlogged": "false"}
        write("data/create/horizon/block/" + block + ".json", declaration)
        texture = "minecraft:block/stripped_spruce_log" if "casing" in block else "minecraft:block/iron_block"
        model(block + "/block", [([0, 0, 0], [16, 16, 16])], texture)
        write("assets/create/blockstates/" + block + ".json", {"variants": {"": {"model": "create:block/" + block + "/block"}}})
        write("assets/create/models/item/" + block + ".json", {"parent": "create:block/" + block + "/block"})
        write("assets/create/items/" + block + ".json", {"model": {"type": "minecraft:model", "model": "create:item/" + block}})
        drop = "shaft" if "encased_shaft" in block else block
        write("data/create/loot_table/blocks/" + block + ".json", {"type": "minecraft:block", "pools": [{"rolls": 1, "entries": [{"type": "minecraft:item", "name": "create:" + drop}]}]})
    write('data/create/horizon/block/water_wheel_structure.json', {'template':'minecraft:glass','item':False,'properties':{'hardness':1.5,'render_shape':'INVISIBLE','sounds':'minecraft:stripped_spruce_wood'},'state_properties':{'facing':{'type':'enum','values':['down','up','north','south','west','east']}},'default_state':{'facing':'north'}})
    write('assets/create/blockstates/water_wheel_structure.json',{'variants':{'':{'model':'minecraft:block/air'}}})
    write('data/create/loot_table/blocks/water_wheel_structure.json',{'type':'minecraft:block','pools':[]})
    for item in ITEMS:
        if item in [name + '_sheet' for name in SHEETS]: continue
        item_decl={"template":"minecraft:iron_ingot","max_stack_size":1 if item in ['wrench','goggles'] else 64}
        if item=='goggles':item_decl['components']={'minecraft:equippable':{'slot':'head','equip_sound':'minecraft:item.armor.equip_generic','dispensable':True,'swappable':True,'damage_on_hurt':False}}
        if item=='creative_blaze_cake':item_decl['rarity']='epic'
        if item == 'empty_blaze_burner':
            item_decl['place_block'] = 'create:blaze_burner'
        write("data/create/horizon/item/" + item + ".json", item_decl)
        sprite = {"wheat_flour": "sugar", "dough": "bread"}.get(item, "iron_ingot")
        write("assets/create/models/item/" + item + ".json", {"parent": "minecraft:item/generated", "textures": {"layer0": "minecraft:item/" + sprite}})
        write("assets/create/items/" + item + ".json", {"model": {"type": "minecraft:model", "model": "create:item/" + item}})
        if item=='goggles':
            write('assets/create/items/goggles.json',{'model':{'type':'minecraft:select','property':'minecraft:display_context','cases':[{'when':'head','model':{'type':'minecraft:model','model':'create:block/goggles'}}],'fallback':{'type':'minecraft:model','model':'create:item/goggles'}}})
    vertical={'condition':'minecraft:block_state_property','block':'create:gearbox','properties':{'axis':'y'}}
    write('data/create/loot_table/blocks/gearbox.json',{'type':'minecraft:block','pools':[{'rolls':1,'entries':[{'type':'minecraft:alternatives','children':[{'type':'minecraft:item','name':'create:gearbox','conditions':[vertical]},{'type':'minecraft:item','name':'create:vertical_gearbox'}]}]}]})
    model("hand_crank/handle", [([6, 5, 6], [10, 10, 16])], "minecraft:block/oak_planks")
    model('goggles',[([4,6,3],[7,9,4]),([9,6,3],[12,9,4])],'minecraft:block/gold_block')
    processors()
    basin_mixer()
    burners()
    belts()
    chain_drives()
    fallback_instruments_and_wheels()
    (CONTENT / 'assets/create/models/block/port_hint.json').unlink(missing_ok=True)
    # Only MIT recipe/tag data is distributed; original assets stay in the JAR.
    reference = json.loads((ROOT / "tools/data/recipes.json").read_text())
    def current(value):
        if isinstance(value, dict):
            if set(value) == {"item"}: return value["item"]
            if set(value) == {"tag"}: return "#" + value["tag"]
            return {k: current(v) for k,v in value.items()}
        if isinstance(value, list): return [current(v) for v in value]
        return value
    for folder, retained in [(CONTENT / "data/create/recipe", reference["recipes"]),
                             (CONTENT / "data/create/tags", reference["tags"]),
                             (CONTENT / "data/c/tags/item", reference["tags"])]:
        for path in folder.rglob("*.json"):
            if path.relative_to(CONTENT / "data").as_posix() not in retained: path.unlink()
    for path, value in reference["recipes"].items(): write("data/" + path, current(value))
    for path, value in reference["tags"].items(): write("data/" + path, value)
    for path, value in reference.get('data_maps', {}).items(): write('data/' + path, value)
    for path, value in reference.get('loot_tables', {}).items(): write('data/' + path, value)
    # MIT Create AllDamageTypes / generated data, pinned in compatibility.json.
    # Keep source IDs and data-driven fire/protection policy on both sides.
    for kind in ("fan_fire", "fan_lava"):
        write("data/create/damage_type/" + kind + ".json", {
            "effects": "burning", "exhaustion": 0.0,
            "message_id": "create." + kind,
            "scaling": "when_caused_by_living_non_player",
        })
    for tag in ("is_fire", "bypasses_armor", "bypasses_shield"):
        write("data/minecraft/tags/damage_type/" + tag + ".json", {
            "values": ["create:fan_fire", "create:fan_lava"],
        })
    write("data/create/horizon/creative_tab/kinetics.json", {"title": "itemGroup.create.base", "icon": "create:cogwheel", "items": ["create:" + name for name in all_blocks + ITEMS]})
    # These fallback labels are ours; the JAR replaces them with all upstream locales.
    extra_blocks = {"gearbox":"Gearbox","clutch":"Clutch","gearshift":"Gearshift","hand_crank":"Hand Crank","andesite_encased_shaft":"Andesite Encased Shaft","brass_encased_shaft":"Brass Encased Shaft","andesite_casing":"Andesite Casing","brass_casing":"Brass Casing","andesite_alloy_block":"Block of Andesite Alloy","brass_block":"Block of Brass","zinc_block":"Block of Zinc","speedometer":"Speedometer","stressometer":"Stressometer","water_wheel":"Water Wheel","large_water_wheel":"Large Water Wheel","millstone":"Millstone","encased_fan":"Encased Fan","mechanical_drill":"Mechanical Drill"}
    extra_items = {"andesite_alloy":"Andesite Alloy","brass_ingot":"Brass Ingot","brass_nugget":"Brass Nugget","zinc_ingot":"Zinc Ingot","zinc_nugget":"Zinc Nugget","wrench":"Wrench","brass_sheet":"Brass Sheet","vertical_gearbox":"Vertical Gearbox","goggles":"Engineer’s Goggles","propeller":"Propeller","wheat_flour":"Wheat Flour","dough":"Dough"}
    extra_blocks.update({'basin': 'Basin', 'mechanical_mixer': 'Mechanical Mixer'})
    extra_blocks.update({'blaze_burner': 'Blaze Burner', 'lit_blaze_burner': 'Lit Blaze Burner'})
    extra_blocks['encased_chain_drive'] = 'Encased Chain Drive'
    extra_items['whisk'] = 'Whisk'
    extra_items.update({'empty_blaze_burner': 'Empty Blaze Burner', 'blaze_cake': 'Blaze Cake', 'creative_blaze_cake': 'Creative Blaze Cake'})
    for locale in ["en_us","ru_ru"]:
        path=CONTENT / ("assets/create/lang/"+locale+".json")
        lang=json.loads(path.read_text())
        lang.update({"block.create."+k:v for k,v in extra_blocks.items()})
        lang.update({"item.create."+k:v for k,v in extra_items.items()})
        if locale == "ru_ru":
            lang.update({"block.create.millstone": "Мельничный жернов", "block.create.encased_fan": "Вентилятор в корпусе", "block.create.mechanical_drill": "Механический бур",
                         "item.create.propeller": "Пропеллер", "item.create.wheat_flour": "Пшеничная мука", "item.create.dough": "Тесто"})
            lang.update({'block.create.basin': 'Чаша', 'block.create.mechanical_mixer': 'Механический миксер', 'item.create.whisk': 'Венчик'})
            lang.update({'block.create.blaze_burner': 'Горелка всполоха', 'block.create.lit_blaze_burner': 'Зажжённая горелка всполоха',
                         'item.create.empty_blaze_burner': 'Пустая горелка всполоха', 'item.create.blaze_cake': 'Пирог всполоха',
                         'item.create.creative_blaze_cake': 'Творческий пирог всполоха'})
        lang["create.kinetics.creative_motor.rotation_speed"]="Generated Speed in RPM"
        path.write_text(json.dumps(lang,ensure_ascii=False,indent=2)+"\n")
    (NATIVE / "assets/src/catalog.rs").write_text(rust_catalog())
    # Provisional aliases derived from MIT AllSoundEvents registrations.
    sounds = {key: {"sounds": [{"type": "event", "name": event}]} for key,event in {
        "depot_slide": "minecraft:block.sand.break", "depot_plop": "minecraft:entity.item_frame.add_item",
        "mechanical_press_activation": "minecraft:block.anvil.land", "cranking": "minecraft:block.wood.place",
        "wrench_rotate": "minecraft:block.wooden_trapdoor.close", "wrench_remove": "minecraft:entity.item.pickup",
    }.items()}
    # Exact MIT AllSoundEvents.MIXING registration, not bundled sound bytes.
    sounds['mixing'] = {'sounds': [{'type': 'event', 'name': 'minecraft:block.' + name + '.break', 'volume': .125, 'pitch': .5}
                                  for name in ['gilded_blackstone', 'netherrack']]}
    write('assets/create/sounds.json', sounds)


def main():
    CONTENT.mkdir(exist_ok=True)
    (CONTENT / "horizon-mod.toml").write_text('''id = "create"
name = "Create for Horizon"
version = "0.7.0-alpha.2"
description = "Build rotating machines and small factories in Horizon. The original Create JAR is optional and supplies its models and textures."
authors = ["create-horizon contributors"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''')
    for block in BLOCKS:
        declaration = {"template": "minecraft:glass", "item": True,
                       "properties": {"hardness": 1.5, "render_shape": "ENTITYBLOCK_ANIMATED", "sounds": "minecraft:andesite"}}
        if block in ["cogwheel", "large_cogwheel"]:
            declaration["properties"]["sounds"] = "minecraft:stripped_spruce_wood"
        if block in ["shaft", "cogwheel", "large_cogwheel"]:
            declaration["state_properties"] = {"axis": {"type": "enum", "values": ["x", "y", "z"]}}
            declaration["state_properties"]["waterlogged"] = {"type": "boolean"}
            declaration["default_state"] = {"axis": "y", "waterlogged": "false"}
            shapes = {"x": [[0, .375, .375, 1, .625, .625]],
                      "y": [[.375, 0, .375, .625, 1, .625]],
                      "z": [[.375, .375, 0, .625, .625, 1]]}
            if "cogwheel" in block:
                shapes = {"x": [[.375, 0, 0, .625, 1, 1]],
                          "y": [[0, .375, 0, 1, .625, 1]],
                          "z": [[0, 0, .375, 1, 1, .625]]}
            declaration["shapes"] = {"property": "axis", "values": shapes}
        elif block in ["creative_motor", "mechanical_press"]:
            values = ["north", "south", "east", "west"]
            if block == "creative_motor": values += ["up", "down"]
            declaration["state_properties"] = {"facing": {"type": "enum", "values": values}}
            declaration["default_state"] = {"facing": "east"}
            if block == "creative_motor":
                # AllShapes.MOTOR_BLOCK: box(3,0,3,13,14,13), forDirectional(UP).
                # Selection and collision must share the original shape: the
                # value-box hit test uses the selected face's actual hit point.
                shapes = {"property": "facing", "values": {
                    "down": [[3/16, 2/16, 3/16, 13/16, 1, 13/16]],
                    "up": [[3/16, 0, 3/16, 13/16, 14/16, 13/16]],
                    "north": [[3/16, 3/16, 2/16, 13/16, 13/16, 1]],
                    "south": [[3/16, 3/16, 0, 13/16, 13/16, 14/16]],
                    "west": [[2/16, 3/16, 3/16, 1, 13/16, 13/16]],
                    "east": [[0, 3/16, 3/16, 14/16, 13/16, 13/16]],
                }}
                declaration["shapes"] = shapes
                declaration["collision_shapes"] = shapes
        else:
            # Create's depot is not an inventory screen or a directional barrel.
            declaration["state_properties"] = {"waterlogged": {"type": "boolean"}}
            declaration["default_state"] = {"waterlogged": "false"}
            declaration["shapes"] = {"default": [[0, 0, 0, 1, 13 / 16, 1]]}
        write("data/create/horizon/block/" + block + ".json", declaration)
        # A blockstate model is still needed for the block item and model baking.
        name = {"creative_motor": "creative_motor/block", "mechanical_press": "mechanical_press/block", "depot": "depot/block"}.get(block, block)
        write("assets/create/blockstates/" + block + ".json", {"variants": {"": {"model": "create:block/" + name}}})
        write("assets/create/models/item/" + block + ".json", {"parent": "create:block/" + name})
        write("assets/create/items/" + block + ".json", {"model": {"type": "minecraft:model", "model": "create:item/" + block}})
        write("data/create/loot_table/blocks/" + block + ".json", {
            "type": "minecraft:block", "pools": [{"rolls": 1, "entries": [{"type": "minecraft:item", "name": "create:" + block}]}]})
    shaft = [([6, 0, 6], [10, 16, 10])]
    model("shaft", shaft)
    model("shaft_half", [([6, 6, 8], [10, 10, 16])])
    for name, low, high in [("cogwheel", 2, 14), ("large_cogwheel", -1, 17)]:
        boxes = shaft + [([low, 6, low], [high, 10, high])]
        for x, z in [(0, 6), (6, 0), (12, 6), (6, 12)]:
            boxes.append(([x, 5, z], [x + 4, 11, z + 4]))
        model(name, boxes, "minecraft:block/oak_planks")
    model("creative_motor/block", [([1, 0, 1], [15, 14, 15])], "minecraft:block/copper_block")
    model("creative_motor/block_vertical", [([1, 0, 1], [15, 14, 15])], "minecraft:block/copper_block")
    model("mechanical_press/block", [([0, 8, 0], [16, 16, 16])])
    model("mechanical_press/head", [([6, -12, 6], [10, 9, 10]), ([2, -14, 2], [14, -10, 14])])
    model("depot/block", [([1, 0, 1], [15, 12, 15]), ([0, 12, 0], [16, 13, 16])], "minecraft:block/smooth_stone")
    (CONTENT / "data/create/horizon/menu/depot.json").unlink(missing_ok=True)
    for name in SHEETS:
        write("data/create/horizon/item/" + name + "_sheet.json", {"template": "minecraft:iron_ingot", "max_stack_size": 64})
        vanilla = "gold" if name == "golden" else name
        write("assets/create/models/item/" + name + "_sheet.json", {"parent": "minecraft:item/generated", "textures": {"layer0": "minecraft:item/" + vanilla + "_ingot"}})
        write("assets/create/items/" + name + "_sheet.json", {"model": {"type": "minecraft:model", "model": "create:item/" + name + "_sheet"}})
        # Adapted from the three MIT Create pressing recipes. Convention tags are
        # narrowed to vanilla ingots for this initial, self-contained content set.
        write("data/create/recipe/pressing/" + vanilla + "_ingot.json", {
            "type": "create:pressing", "ingredients": ["minecraft:" + vanilla + "_ingot"],
            "results": [{"id": "create:" + name + "_sheet"}],
        })
    ids = BLOCKS + [name + "_sheet" for name in SHEETS]
    write("data/create/horizon/creative_tab/kinetics.json", {"title": "itemGroup.create.base", "icon": "create:cogwheel", "items": ["create:" + name for name in ids]})
    english = ["Creative Motor", "Shaft", "Cogwheel", "Large Cogwheel", "Mechanical Press", "Depot"]
    russian = ["Творческий мотор", "Вал", "Шестерня", "Большая шестерня", "Механический пресс", "Депо"]
    for language, names, title in [("en_us", english, "Create for Horizon"), ("ru_ru", russian, "Create для Horizon")]:
        lang = {"itemGroup.create.base": title}
        lang.update({"block.create." + block: name for block, name in zip(BLOCKS, names)})
        lang.update({"item.create." + sheet + "_sheet": name for sheet, name in zip(SHEETS,
                    ["Iron Sheet", "Golden Sheet", "Copper Sheet"] if language == "en_us" else ["Железная пластина", "Золотая пластина", "Медная пластина"])})
        write("assets/create/lang/" + language + ".json", lang)
    foundation()
    # Source installs carry only the native bundle. Its documented data folder
    # therefore contains the complete public content, with no player-side Python.
    files = {path.relative_to(CONTENT).as_posix(): path.read_text()
             for path in sorted(CONTENT.rglob("*")) if path.is_file()
             and "target" not in path.relative_to(CONTENT).parts}
    bundled = NATIVE / "data/content.json"
    bundled.parent.mkdir(parents=True, exist_ok=True)
    bundled.write_text(json.dumps(files, indent=2, ensure_ascii=False) + "\n")
    from catalog import BLOCKS as all_blocks, ITEMS
    print(f"Generated {len(all_blocks)} blocks, {len(ITEMS)} standalone items and bundled foundation content.")


if __name__ == "__main__":
    main()
