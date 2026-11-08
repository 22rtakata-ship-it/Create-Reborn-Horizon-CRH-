#!/usr/bin/env python3
"""Generate mod content with simple built-in models.

Models reference independent fallback art. The original artwork stays in the local JAR.
"""
import json
import zipfile
from fallback_textures import retarget
from common import ROOT, NATIVE
from catalog import HIDDEN_CREATIVE_ITEMS, IRON_PALETTE, PALETTE_BLOCKS, STONE_PALETTES, ZINC_BLOCKS
from pulley_content import rope_pulley
from gantry_content import gantry
from sail_content import COLORED_SAILS, sails
from chassis_content import chassis
from deployer_content import deployer
from portable_storage_content import PSI, portable_storage
from chute_content import CHUTE, chute
from smart_chute_content import BLOCK as SMART_CHUTE, smart_chute
from funnel_content import BLOCK as FUNNEL, BRASS as BRASS_FUNNEL, funnel
from crushing_content import WHEEL as CRUSHER, crushing
from crafter_content import CRAFTER, crafter
from item_drain_content import BLOCK as ITEM_DRAIN, item_drain
from fluid_tank_content import BLOCK as FLUID_TANK, fluid_tank
from fluid_pipe_content import PIPE, PUMP, VALVE, fluid_pipes
from sticker_content import STICKER, sticker
from encased_cog_content import ENGLISH as ENCASED_COGS_EN, RUSSIAN as ENCASED_COGS_RU, encased_cogs

CONTENT = ROOT / "content"
BLOCKS = ["creative_motor", "shaft", "cogwheel", "large_cogwheel", "mechanical_press", "depot"]
SHEETS = ["iron", "golden", "copper"]
FACING_AXES = {"down": 1, "up": 1, "north": 2, "south": 2, "west": 0, "east": 0}
STONE_APPEARANCE = {
    "andesite": ("stone_bricks", "tuff_bricks", "андезит", "андезитовые"),
    "granite": ("mud_bricks", "bricks", "гранит", "гранитные"),
    "diorite": ("quartz_bricks", "calcite", "диорит", "диоритовые"),
}


def write(path, value):
    retarget(path, value)
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


def axial_boxes(facing, sections):
    """Build centered block-shape boxes from pixel bounds along their facing."""
    boxes = []
    axis = FACING_AXES[facing]
    for inset, low, high in sections:
        start, end = [inset] * 3, [16 - inset] * 3
        if facing in ("down", "north", "west"):
            low, high = 16 - high, 16 - low
        start[axis], end[axis] = low, high
        boxes.append([coordinate / 16 for coordinate in start + end])
    return boxes


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
        "default_state": {"facing": "north"}, "shapes": full, "collision_shapes": full,
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
        "default_state": {"facing": "north", "waterlogged": "false"},
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


def bearings():
    rotations = {
        "up": {}, "down": {"x": 180}, "north": {"x": 90},
        "east": {"x": 90, "y": 90}, "south": {"x": 90, "y": 180},
        "west": {"x": 90, "y": 270},
    }
    model("bearing/top", [([1, 13, 1], [15, 16, 15])], "minecraft:block/smooth_stone")
    model("bearing/top_wooden", [([1, 13, 1], [15, 16, 15])], "minecraft:block/oak_planks")
    for name in ("mechanical_bearing", "windmill_bearing", "clockwork_bearing"):
        write(f"data/create/horizon/block/{name}.json", {
            "template": "minecraft:glass", "item": True,
            "properties": {"hardness": 1.5, "explosion_resistance": 6, "requires_tool": True,
                           "render_shape": "MODEL", "sounds": "minecraft:andesite"},
            "state_properties": {"facing": {"type": "enum", "values": list(rotations)}},
            "default_state": {"facing": "north"},
        })
        casing = "minecraft:block/copper_block" if name == "clockwork_bearing" else "minecraft:block/andesite"
        model(name, [([0, 0, 0], [16, 13, 16])], casing)
        write(f"assets/create/blockstates/{name}.json", {
            "variants": {f"facing={face}": {"model": f"create:block/{name}", **rotation}
                         for face, rotation in rotations.items()},
        })
        write(f"assets/create/models/item/{name}.json", {"parent": f"create:block/{name}"})
        write(f"assets/create/items/{name}.json", {
            "model": {"type": "minecraft:model", "model": f"create:item/{name}"},
        })


def pistons():
    # MIT MechanicalPistonBlock, MechanicalPistonGenerator and AllShapes.
    # Built-in models below are simplified geometry using vanilla textures.
    rotations = {
        "down": {"x": 180}, "up": {}, "north": {"x": 90},
        "south": {"x": 90, "y": 180}, "west": {"x": 90, "y": 270},
        "east": {"x": 90, "y": 90},
    }
    facing_property = {"type": "enum", "values": list(rotations)}
    states = ["retracted", "moving", "extended"]

    piston_shapes = {
        "properties": ["facing", "state"],
        "values": {
            f"{facing},{state}": axial_boxes(facing, sections)
            for facing in rotations
            for state, sections in [
                ("moving", [(0, 0, 12)]),
                ("extended", [(0, 0, 12), (6, 12, 16)]),
            ]
        },
        "default": [[0, 0, 0, 1, 1, 1]],
    }
    path = "mechanical_piston"
    body = [([0, 0, 0], [16, 12, 16])]
    pole = [([6, 0, 6], [10, 16, 10])]
    head = [([6, 0, 6], [10, 12, 10]), ([0, 12, 0], [16, 16, 16])]
    for folder, boxes, texture in [
        ("normal", body + head[1:], "minecraft:block/stripped_spruce_log"),
        ("sticky", body + head[1:], "minecraft:block/mossy_cobblestone"),
        ("moving", body, "minecraft:block/stripped_spruce_log"),
        ("extended", body + [([6, 12, 6], [10, 16, 10])], "minecraft:block/stripped_spruce_log"),
    ]:
        model(f"{path}/{folder}/block", boxes, texture)
        write(f"assets/create/models/block/{path}/{folder}/block_rotated.json",
              {"parent": f"create:block/{path}/{folder}/block"})
    model(f"{path}/normal/head", head, "minecraft:block/stripped_spruce_log")
    model(f"{path}/sticky/head", head, "minecraft:block/moss_block")
    model("piston_extension_pole", pole, "minecraft:block/stripped_spruce_log")

    for name, folder in [("mechanical_piston", "normal"), ("sticky_mechanical_piston", "sticky")]:
        write(f"data/create/horizon/block/{name}.json", {
            "template": "minecraft:glass", "item": True,
            "properties": {"hardness": 1.5, "explosion_resistance": 6, "occludes": False,
                           "render_shape": "MODEL", "sounds": "minecraft:andesite"},
            "state_properties": {"facing": facing_property, "axis_along_first": {"type": "boolean"},
                                 "state": {"type": "enum", "values": states}},
            "default_state": {"facing": "north", "axis_along_first": "true", "state": "retracted"},
            "shapes": piston_shapes,
        })
        variants = {}
        for facing, rotation in rotations.items():
            for along_first in (False, True):
                model_name = "block_rotated" if (FACING_AXES[facing] == 0) != along_first else "block"
                for state in states:
                    subfolder = folder if state == "retracted" else state
                    key = f"axis_along_first={str(along_first).lower()},facing={facing},state={state}"
                    variants[key] = {"model": f"create:block/{path}/{subfolder}/{model_name}", **rotation}
        write(f"assets/create/blockstates/{name}.json", {"variants": variants})
        write(f"assets/create/models/block/{path}/{folder}/item.json", {
            "parent": f"create:block/{path}/{folder}/block",
        })
        write(f"assets/create/models/item/{name}.json", {"parent": f"create:block/{path}/{folder}/item"})

    for name, sections, sound, facing, item in [
        ("piston_extension_pole", [(6, 0, 16)], "minecraft:scaffolding", "up", True),
        ("mechanical_piston_head", [(6, 0, 12), (0, 12, 16)], "minecraft:piston", "north", False),
    ]:
        properties = {"facing": facing_property, "waterlogged": {"type": "boolean"}}
        defaults = {"facing": facing, "waterlogged": "false"}
        if not item:
            properties["type"] = {"type": "enum", "values": ["normal", "sticky"]}
            defaults["type"] = "normal"
        write(f"data/create/horizon/block/{name}.json", {
            "template": "minecraft:glass", "item": item,
            "properties": {"hardness": 1.5, "explosion_resistance": 1.5, "occludes": False,
                           "render_shape": "MODEL", "sounds": sound},
            "state_properties": properties, "default_state": defaults,
            "shapes": {"property": "facing", "values": {
                side: axial_boxes(side, sections) for side in rotations
            }},
        })
        variants = {}
        for side, rotation in rotations.items():
            for wet in ("false", "true"):
                key = f"facing={side},waterlogged={wet}"
                if item:
                    variants[key] = {"model": f"create:block/{name}", **rotation}
                else:
                    for kind in ("normal", "sticky"):
                        variants[f"{key},type={kind}"] = {
                            "model": f"create:block/{path}/{kind}/head", **rotation,
                        }
        write(f"assets/create/blockstates/{name}.json", {"variants": variants})
    write("assets/create/models/item/piston_extension_pole.json", {
        "parent": "create:block/piston_extension_pole",
    })
    for name in ("mechanical_piston", "sticky_mechanical_piston", "piston_extension_pole"):
        write(f"assets/create/items/{name}.json", {
            "model": {"type": "minecraft:model", "model": f"create:item/{name}"},
        })


def contraption_actors():
    # MIT AttachedActorBlock, SawBlock, SawGenerator and AllShapes.
    properties = {"hardness": 1.5, "explosion_resistance": 6, "render_shape": "MODEL",
                  "sounds": "minecraft:andesite"}
    harvester = "mechanical_harvester"
    plough = "mechanical_plough"
    saw = "mechanical_saw"
    horizontal = {"north": 0, "east": 90, "south": 180, "west": 270}
    harvester_bounds = {
        "south": [0, 2, 0, 16, 14, 3], "north": [0, 2, 13, 16, 14, 16],
        "east": [0, 2, 0, 3, 14, 16], "west": [13, 2, 0, 16, 14, 16],
    }
    attached = {
        "template": "minecraft:glass", "item": True, "properties": properties,
        "state_properties": {"facing": {"type": "enum", "values": list(horizontal)},
                             "waterlogged": {"type": "boolean"}},
        "default_state": {"facing": "north", "waterlogged": "false"},
        "shapes": {"property": "facing", "values": {
            side: [[coordinate / 16 for coordinate in box]] for side, box in harvester_bounds.items()
        }},
    }
    for name, appearance in ((harvester, f"{harvester}/block"), (plough, plough)):
        write(f"data/create/horizon/block/{name}.json", attached)
        write(f"assets/create/blockstates/{name}.json", {
            "variants": {f"facing={side},waterlogged={wet}": {
                "model": f"create:block/{appearance}", **({"y": yaw} if yaw else {}),
            } for side, yaw in horizontal.items() for wet in ("false", "true")},
        })
    model(f"{harvester}/block", [([0, 2, 13], [16, 14, 16]), ([0, 3, 8], [16, 5, 13])])
    model(f"{harvester}/blade", [([0, 3, 8], [16, 5, 13])])
    model(plough, [([0, 2, 13], [16, 14, 16]), ([0, 2, 1], [16, 5, 13])], "minecraft:block/anvil")
    write(f"assets/create/models/item/{plough}.json", {"parent": f"create:block/{plough}"})
    write(f"assets/create/items/{plough}.json", {
        "model": {"type": "minecraft:model", "model": f"create:item/{plough}"},
    })

    write(f"data/create/horizon/block/{saw}.json", {
        "template": "minecraft:glass", "item": True, "properties": properties,
        "state_properties": {"facing": {"type": "enum", "values": list(FACING_AXES)},
                             "axis_along_first": {"type": "boolean"}, "flipped": {"type": "boolean"}},
        "default_state": {"facing": "north", "axis_along_first": "true", "flipped": "true"},
        "shapes": {"property": "facing", "values": {
            side: axial_boxes(side, [(0, 0, 12)]) for side in FACING_AXES
        }},
    })
    saw_variants = {}
    for side in FACING_AXES:
        for first in ("false", "true"):
            for flipped in ("false", "true"):
                if side in horizontal:
                    model_type, pitch, yaw = "horizontal", 0, (horizontal[side] + 180) % 360
                else:
                    model_type, pitch = "vertical", 180 if side == "down" else 0
                    yaw = ((270 if first == "true" else 0) + (180 if flipped == "true" else 0)) % 360
                entry = {"model": f"create:block/{saw}/{model_type}"}
                if pitch:
                    entry["x"] = pitch
                if yaw:
                    entry["y"] = yaw
                saw_variants[f"axis_along_first={first},facing={side},flipped={flipped}"] = entry
    write(f"assets/create/blockstates/{saw}.json", {"variants": saw_variants})
    blade = {
        "horizontal": [([7, 3, 12], [9, 13, 16])],
        "vertical": [([3, 12, 7], [13, 16, 9])],
    }
    for orientation, base in [("horizontal", [([0, 0, 0], [16, 16, 12])]),
                              ("vertical", [([0, 0, 0], [16, 12, 16])])]:
        model(f"{saw}/{orientation}", base + blade[orientation], "minecraft:block/andesite")
        for motion in ("active", "inactive", "reversed"):
            model(f"{saw}/blade_{orientation}_{motion}", blade[orientation])

    for name, appearance in [(harvester, "block"), (saw, "vertical")]:
        write(f"assets/create/models/block/{name}/item.json", {"parent": f"create:block/{name}/{appearance}"})
        write(f"assets/create/models/item/{name}.json", {"parent": f"create:block/{name}/item"})
        write(f"assets/create/items/{name}.json", {
            "model": {"type": "minecraft:model", "model": f"create:item/{name}"},
        })


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
    # Simple fallback cases and cover. Original variants come from the local JAR.
    for slope, box in [('horizontal', ([0, 0, 0], [16, 11, 16])),
                       ('diagonal', ([0, 2, -3.314], [16, 11, 19.314])),
                       ('sideways', ([5, 0, 0], [11, 16, 16]))]:
        rotation = {'origin': [8, 8, 8], 'axis': 'x', 'angle': 45} if slope == 'diagonal' else None
        name = f'belt_casing/{slope}_middle'
        model(name, [box], 'minecraft:block/cut_copper', rotation=rotation)
        for part in ('start', 'end', 'pulley'):
            write(f'assets/create/models/block/belt_casing/{slope}_{part}.json',
                  {'parent': 'create:block/' + name})
        for part in ('start', 'middle', 'end', 'pulley'):
            write(f'assets/create/models/block/belt_casing/andesite_{slope}_{part}.json', {
                'parent': f'create:block/belt_casing/{slope}_{part}',
                'textures': {'all': 'minecraft:block/andesite', 'particle': 'minecraft:block/andesite'},
            })
    for material, texture in [('andesite', 'polished_andesite'), ('brass', 'cut_copper')]:
        name = f'belt_cover/{material}_belt_cover'
        model(name + '_x', [([0, 11, 0], [16, 16, 16])], 'minecraft:block/' + texture)
        write(f'assets/create/models/block/{name}_z.json', {'parent': 'create:block/' + name + '_x'})
    write('data/create/loot_table/blocks/belt.json', {'type': 'minecraft:block', 'pools': [
        {'rolls': 1, 'entries': [{'type': 'minecraft:item', 'name': 'create:belt_connector'}]},
        {'rolls': 1, 'conditions': [{'type': 'minecraft:inverted', 'term': {
            'type': 'minecraft:block_state_property', 'block': 'create:belt',
            'properties': {'part': 'middle'}}}],
         'entries': [{'type': 'minecraft:item', 'name': 'create:shaft'}]},
    ]})


def chain_drives():
    # MIT ChainDriveBlock / RotatedPillarKineticBlock: 24 states, full collision.
    # Original visual resources are imported privately from the player's JAR.
    declaration = {
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
    }
    write('data/create/horizon/block/encased_chain_drive.json', declaration)
    adjustable = 'adjustable_chain_gearshift'
    write(f'data/create/horizon/block/{adjustable}.json', {
        **declaration,
        'properties': {**declaration['properties'], 'map_color': 35},
        'state_properties': {**declaration['state_properties'], 'powered': {'type': 'boolean'}},
        'default_state': {**declaration['default_state'], 'powered': 'false'},
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
    adjustable_variants = {}
    for powered, texture in [('false', 'red_terracotta'), ('true', 'redstone_block')]:
        name = f'{adjustable}/{powered}'
        model(name, [([0, 0, 0], [16, 16, 16])], f'minecraft:block/{texture}')
        for key, variant in variants.items():
            adjustable_variants[f'{key},powered={powered}'] = {**variant, 'model': f'create:block/{name}'}
    write(f'assets/create/blockstates/{adjustable}.json', {'variants': adjustable_variants})
    write(f'assets/create/models/item/{adjustable}.json', {'parent': f'create:block/{adjustable}/false'})


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


def flywheel():
    # FlywheelBlock uses AllShapes.LARGE_GEAR, an axis-aligned slab and six-pixel shaft.
    shapes = {"property": "axis", "values": {
        axis: axial_boxes(face, [(0, 6, 10), (5, 0, 16)])
        for axis, face in (("x", "east"), ("y", "up"), ("z", "south"))
    }}
    write("data/create/horizon/block/flywheel.json", {
        "template": "minecraft:gold_block", "item": True,
        "properties": {"occludes": False, "map_color": 40, "render_shape": "ENTITYBLOCK_ANIMATED"},
        "state_properties": {"axis": {"type": "enum", "values": ["x", "y", "z"]}},
        "default_state": {"axis": "y"}, "shapes": shapes, "collision_shapes": shapes,
    })
    # Authored fallback rim and spokes. The runtime draws the shaft separately.
    wheel = [
        ([-3, 5, -12], [19, 11, -8]), ([-3, 5, 24], [19, 11, 28]),
        ([-12, 5, -3], [-8, 11, 19]), ([24, 5, -3], [28, 11, 19]),
        ([-8, 5, -8], [-2, 11, -2]), ([18, 5, -8], [24, 11, -2]),
        ([-8, 5, 18], [-2, 11, 24]), ([18, 5, 18], [24, 11, 24]),
        ([-8, 6, 6], [24, 10, 10]), ([6, 6, -8], [10, 10, 6]), ([6, 6, 10], [10, 10, 24]),
    ]
    texture = "minecraft:block/gold_block"
    model("flywheel/block", wheel, texture)
    model("flywheel/item", wheel + [([5, 0, 5], [11, 16, 11])], texture)
    model("flywheel/particle", [], texture)
    write("assets/create/blockstates/flywheel.json", {
        "variants": {"": {"model": "create:block/flywheel/particle"}},
    })
    write("assets/create/models/item/flywheel.json", {
        "parent": "create:block/flywheel/item",
        "display": {
            "gui": {"rotation": [30, 225, 0], "scale": [.4, .4, .4]},
            "fixed": {"rotation": [90, 0, 0], "translation": [0, 0, -1], "scale": [.375, .375, .375]},
        },
    })
    write("assets/create/items/flywheel.json", {
        "model": {"type": "minecraft:model", "model": "create:item/flywheel"},
    })


def stone_palettes():
    # These PaletteBlockPatterns copy their vanilla stone properties, add
    # the pickaxe requirement and use no state properties.
    for stone, variants in STONE_PALETTES.items():
        bricks, small_bricks, _, _ = STONE_APPEARANCE[stone]
        textures = [stone, f"polished_{stone}", bricks, small_bricks]
        for index, name in enumerate(variants):
            write(f"data/create/horizon/block/{name}.json", {
                "template": f"minecraft:{stone}", "item": True, "properties": {"requires_tool": True},
            })
            block_model = f"create:block/{name}"
            write(f"assets/create/blockstates/{name}.json", {"variants": {"": {"model": block_model}}})
            if name == "layered_andesite":
                model_data = {"parent": "minecraft:block/cube_column", "textures": {
                    "end": "minecraft:block/smooth_stone", "side": "minecraft:block/smooth_stone_slab_side",
                }}
            else:
                model_data = {"parent": "minecraft:block/cube_all", "textures": {
                    "all": f"minecraft:block/{textures[index]}",
                }}
            write(f"assets/create/models/block/{name}.json", model_data)
            write(f"assets/create/models/item/{name}.json", {"parent": block_model})
            write(f"assets/create/items/{name}.json", {
                "model": {"type": "minecraft:model", "model": f"create:item/{name}"},
            })


def solid_variant(name, template, texture, properties):
    write(f"data/create/horizon/block/{name}.json", {
        "template": f"minecraft:{template}", "item": True,
        "properties": dict(properties, requires_tool=True),
    })
    block_model = f"create:block/{name}"
    write(f"assets/create/blockstates/{name}.json", {"variants": {"": {"model": block_model}}})
    write(f"assets/create/models/block/{name}.json", {"parent": "minecraft:block/cube_all",
                                                    "textures": {"all": f"minecraft:block/{texture}"}})
    write(f"assets/create/models/item/{name}.json", {"parent": block_model})
    write(f"assets/create/items/{name}.json", {
        "model": {"type": "minecraft:model", "model": f"create:item/{name}"},
    })


def iron_palettes():
    for name, texture in zip(IRON_PALETTE, ("iron_block", "netherite_block"), strict=True):
        solid_variant(name, "gold_block", texture, {"map_color": 21, "sounds": "minecraft:netherite_block"})


def zinc_ores():
    templates = ("gold_ore", "deepslate_gold_ore", "raw_gold_block")
    textures = ("iron_ore", "deepslate_iron_ore", "raw_iron_block")
    for name, template, texture, color in zip(ZINC_BLOCKS, templates, textures, (6, 11, 61), strict=True):
        solid_variant(name, template, texture, {"map_color": color})


def palette_tab():
    write("data/create/horizon/creative_tab/palettes.json", {
        "title": "itemGroup.create.palettes", "icon": "create:cut_andesite",
        "items": [f"create:{name}" for name in PALETTE_BLOCKS],
    })


def foundation():
    from catalog import BLOCKS as all_blocks, ITEMS, TRANSMISSIONS, STATIC_BLOCKS, INSTRUMENTS, GENERATORS, INTERNAL_BLOCKS, SANDPAPERS, rust_catalog
    for block in TRANSMISSIONS + STATIC_BLOCKS + INSTRUMENTS + GENERATORS:
        if block == "flywheel":
            flywheel()
            continue
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
            declaration['default_state']={'facing':'north','axis_along_first':'true'}
        if block in GENERATORS:
            declaration['properties']['render_shape']='ENTITYBLOCK_ANIMATED' if block=='large_water_wheel' else 'MODEL'
            declaration['properties']['sounds']='minecraft:stripped_spruce_wood'
            key='axis' if block=='large_water_wheel' else 'facing'
            values=['x','y','z'] if key=='axis' else ['down','up','north','south','west','east']
            declaration['state_properties']={key:{'type':'enum','values':values}}
            declaration['default_state']={key:'y' if key=='axis' else 'north'}
            if key=='axis':
                declaration['state_properties']['extension']={'type':'boolean'}
                declaration['default_state']['extension']='false'
        if block == "hand_crank":
            # Inherit real water and bucket behavior, including the source
            # restored by the engine when a waterlogged crank breaks.
            declaration["template"] = "minecraft:mangrove_roots"
            declaration["properties"]["sounds"] = "minecraft:stripped_spruce_wood"
            declaration["properties"]["render_shape"] = "ENTITYBLOCK_ANIMATED"
            declaration["properties"]["occludes"] = False
            declaration["state_properties"] = {"facing": {"type": "enum", "values": list(FACING_AXES)}}
            declaration["default_state"] = {"facing": "north", "waterlogged": "false"}
            shape = {"property": "facing", "values": {
                facing: axial_boxes(facing, [(5, 0, 6), (1, 3, 8)]) for facing in FACING_AXES
            }}
            declaration["shapes"] = declaration["collision_shapes"] = shape
        if block in ("andesite_casing", "brass_casing"):
            declaration["properties"]["sounds"] = "minecraft:oak_planks"
        if block == "copper_casing":
            declaration = {"template": "minecraft:andesite", "item": True,
                           "properties": {"map_color": 44, "sounds": "minecraft:oak_planks"}}
        if block == "zinc_block":
            declaration = {"template": "minecraft:iron_block", "item": True,
                           "properties": {"map_color": 61, "requires_tool": True}}
        write("data/create/horizon/block/" + block + ".json", declaration)
        texture = "minecraft:block/stripped_spruce_log" if "casing" in block else "minecraft:block/iron_block"
        if block == "copper_casing":
            texture = "minecraft:block/copper_block"
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
        item_decl={"template":"minecraft:iron_ingot","max_stack_size":1 if item in ['wrench','goggles','super_glue', *SANDPAPERS] else 64}
        if item == 'raw_zinc':
            item_decl['template'] = 'minecraft:raw_iron'
        if item=='goggles':item_decl['components']={'minecraft:equippable':{'slot':'head','equip_sound':'minecraft:item.armor.equip_generic','dispensable':True,'swappable':True,'damage_on_hurt':False}}
        if item == 'super_glue':
            item_decl['components'] = {'minecraft:max_damage': 99, 'minecraft:damage': 0}
        if item == 'experience_nugget':
            item_decl['components'] = {'minecraft:enchantment_glint_override': True}
        if item in SANDPAPERS:
            item_decl['components'] = {'minecraft:max_damage': 8, 'minecraft:damage': 0,
                                       'minecraft:enchantable': {'value': 1}}
        if item=='creative_blaze_cake':item_decl['rarity']='epic'
        if item == 'empty_blaze_burner':
            item_decl['place_block'] = 'create:blaze_burner'
        write("data/create/horizon/item/" + item + ".json", item_decl)
        sprite = {"raw_zinc": "raw_iron", "wheat_flour": "sugar", "dough": "bread", "super_glue": "slime_ball",
                  "rose_quartz": "amethyst_shard", "polished_rose_quartz": "quartz",
                  "sand_paper": "paper", "red_sand_paper": "paper",
                  "electron_tube": "redstone_torch", "brass_hand": "gold_ingot",
                  "crafter_slot_cover": "gold_nugget", "incomplete_precision_mechanism": "iron_nugget",
                  "precision_mechanism": "clock", "crushed_raw_copper": "raw_copper",
                  "crushed_raw_gold": "raw_gold", "crushed_raw_iron": "raw_iron",
                  "crushed_raw_zinc": "raw_iron", "copper_nugget": "copper_ingot",
                  "powdered_obsidian": "gunpowder", "experience_nugget": "emerald"}.get(item, "iron_ingot")
        write("assets/create/models/item/" + item + ".json", {"parent": "minecraft:item/generated", "textures": {"layer0": "minecraft:item/" + sprite}})
        write("assets/create/items/" + item + ".json", {"model": {"type": "minecraft:model", "model": "create:item/" + item}})
        if item=='goggles':
            write('assets/create/items/goggles.json',{'model':{'type':'minecraft:select','property':'minecraft:display_context','cases':[{'when':'head','model':{'type':'minecraft:model','model':'create:block/goggles'}}],'fallback':{'type':'minecraft:model','model':'create:item/goggles'}}})
    vertical={'type':'minecraft:block_state_property','block':'create:gearbox','properties':{'axis':'y'}}
    write('data/create/loot_table/blocks/gearbox.json',{'type':'minecraft:block','pools':[{'rolls':1,'entries':[{'type':'minecraft:alternatives','children':[{'type':'minecraft:item','name':'create:gearbox','conditions':[vertical]},{'type':'minecraft:item','name':'create:vertical_gearbox'}]}]}]})
    model("hand_crank/handle", [([6, 5, 6], [10, 10, 16])], "minecraft:block/oak_planks")
    model('goggles',[([4,6,3],[7,9,4]),([9,6,3],[12,9,4])],'minecraft:block/gold_block')
    processors()
    bearings()
    pistons()
    rope_pulley(write, model)
    gantry(write, model)
    sails(write, model)
    chassis(write)
    portable_storage(write, model, axial_boxes)
    chute(write, model)
    smart_chute(write, model)
    funnel(write, model)
    funnel(write, model, BRASS_FUNNEL)
    crushing(write, model)
    crafter(write, model)
    item_drain(write)
    fluid_tank(write, model)
    fluid_pipes(write, model, axial_boxes)
    sticker(write, model)
    encased_cogs(write, model)
    deployer(write, model)
    contraption_actors()
    basin_mixer()
    burners()
    belts()
    chain_drives()
    fallback_instruments_and_wheels()
    stone_palettes()
    iron_palettes()
    zinc_ores()
    palette_tab()
    (CONTENT / 'assets/create/models/block/port_hint.json').unlink(missing_ok=True)
    # Only MIT recipe/tag data is distributed; original assets stay in the JAR.
    reference = json.loads((ROOT / "tools/data/recipes.json").read_text())
    for path, value in reference['worldgen'].items():
        write('data/' + path, value)
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
    write("data/create/horizon/creative_tab/kinetics.json", {
        "title": "itemGroup.create.base",
        "icon": "create:cogwheel",
        "items": [
            "create:" + name for name in all_blocks + ITEMS
            if name not in PALETTE_BLOCKS and name not in HIDDEN_CREATIVE_ITEMS
        ],
    })
    write('data/minecraft/tags/item/enchantable/durability.json', {'values': ['create:super_glue']})
    # These fallback labels are ours; the JAR replaces them with all upstream locales.
    extra_blocks = {"gearbox":"Gearbox","clutch":"Clutch","gearshift":"Gearshift","hand_crank":"Hand Crank","andesite_encased_shaft":"Andesite Encased Shaft","brass_encased_shaft":"Brass Encased Shaft","andesite_casing":"Andesite Casing","brass_casing":"Brass Casing","andesite_alloy_block":"Block of Andesite Alloy","brass_block":"Block of Brass","zinc_block":"Block of Zinc","speedometer":"Speedometer","stressometer":"Stressometer","water_wheel":"Water Wheel","large_water_wheel":"Large Water Wheel","millstone":"Millstone","encased_fan":"Encased Fan","mechanical_drill":"Mechanical Drill"}
    extra_items = {"andesite_alloy":"Andesite Alloy","brass_ingot":"Brass Ingot","brass_nugget":"Brass Nugget","zinc_ingot":"Zinc Ingot","zinc_nugget":"Zinc Nugget","wrench":"Wrench","brass_sheet":"Brass Sheet","vertical_gearbox":"Vertical Gearbox","goggles":"Engineer’s Goggles","propeller":"Propeller","wheat_flour":"Wheat Flour","dough":"Dough"}
    extra_items.update({'crafter_slot_cover': 'Crafter Slot Cover',
                        'incomplete_precision_mechanism': 'Incomplete Precision Mechanism',
                        'precision_mechanism': 'Precision Mechanism',
                        'crushed_raw_copper': 'Crushed Raw Copper',
                        'crushed_raw_gold': 'Crushed Raw Gold',
                        'crushed_raw_iron': 'Crushed Raw Iron',
                        'crushed_raw_zinc': 'Crushed Raw Zinc',
                        'copper_nugget': 'Copper Nugget',
                        'powdered_obsidian': 'Powdered Obsidian',
                        'experience_nugget': 'Nugget of Experience'})
    extra_blocks.update({'basin': 'Basin', 'mechanical_mixer': 'Mechanical Mixer'})
    extra_blocks.update({'blaze_burner': 'Blaze Burner', 'lit_blaze_burner': 'Lit Blaze Burner'})
    extra_blocks['encased_chain_drive'] = 'Encased Chain Drive'
    extra_blocks['adjustable_chain_gearshift'] = 'Adjustable Chain Gearshift'
    extra_blocks['flywheel'] = 'Flywheel'
    extra_blocks['copper_casing'] = 'Copper Casing'
    extra_blocks['mechanical_bearing'] = 'Mechanical Bearing'
    extra_blocks['windmill_bearing'] = 'Windmill Bearing'
    extra_blocks['clockwork_bearing'] = 'Clockwork Bearing'
    extra_blocks.update({'rope_pulley': 'Rope Pulley', 'rope': 'Rope', 'pulley_magnet': 'Pulley Magnet'})
    extra_blocks.update({'gantry_shaft': 'Gantry Shaft', 'gantry_carriage': 'Gantry Carriage'})
    extra_blocks.update({name: name.replace('_', ' ').title() for name in COLORED_SAILS})
    extra_blocks.update({'sail_frame': 'Windmill Sail Frame', 'white_sail': 'Windmill Sail'})
    extra_blocks['deployer'] = 'Deployer'
    extra_blocks[PSI] = 'Portable Storage Interface'
    extra_blocks[CHUTE] = 'Chute'
    extra_blocks[SMART_CHUTE] = 'Smart Chute'
    extra_blocks[FUNNEL] = 'Andesite Funnel'
    extra_blocks[BRASS_FUNNEL] = 'Brass Funnel'
    extra_blocks[CRUSHER] = 'Crushing Wheel'
    extra_blocks[CRAFTER] = 'Mechanical Crafter'
    extra_blocks[ITEM_DRAIN] = 'Item Drain'
    extra_blocks[FLUID_TANK] = 'Fluid Tank'
    extra_blocks.update({PIPE: 'Fluid Pipe', PUMP: 'Mechanical Pump', VALVE: 'Fluid Valve'})
    extra_blocks.update({'linear_chassis': 'Linear Chassis',
                         'secondary_linear_chassis': 'Secondary Linear Chassis',
                         'radial_chassis': 'Radial Chassis'})
    extra_blocks[STICKER] = 'Sticker'
    extra_blocks.update(ENCASED_COGS_EN)
    extra_blocks.update(zip(IRON_PALETTE, ('Block of Industrial Iron', 'Block of Weathered Iron'), strict=True))
    extra_blocks.update(zip(ZINC_BLOCKS, ('Zinc Ore', 'Deepslate Zinc Ore', 'Block of Raw Zinc'), strict=True))
    extra_items['raw_zinc'] = 'Raw Zinc'
    extra_blocks.update({"mechanical_piston": "Mechanical Piston", "sticky_mechanical_piston": "Sticky Mechanical Piston",
                         "mechanical_piston_head": "Mechanical Piston Head", "piston_extension_pole": "Piston Extension Pole"})
    extra_blocks.update({"mechanical_harvester": "Mechanical Harvester",
                         "mechanical_plough": "Mechanical Plough", "mechanical_saw": "Mechanical Saw"})
    palette_ru = {
        "block.create.industrial_iron_block": "Блок промышленного железа",
        "block.create.weathered_iron_block": "Состаренный блок промышленного железа",
    }
    for stone, variants in STONE_PALETTES.items():
        title = stone.title()
        _, _, name, adjective = STONE_APPEARANCE[stone]
        labels = [f"Cut {title}", f"Polished Cut {title}", f"Cut {title} Bricks", f"Small {title} Bricks"]
        russian = [f"Резной {name}", f"Полированный резной {name}",
                   f"Резные {adjective} кирпичи", f"Мелкие {adjective} кирпичи"]
        if stone == "andesite":
            labels.append("Layered Andesite")
            russian.append("Слоистый андезит")
        extra_blocks.update(zip(variants, labels, strict=True))
        palette_ru.update((f"block.create.{variant}", label) for variant, label in zip(variants, russian, strict=True))
    extra_items.update({'whisk': 'Whisk', 'super_glue': 'Super Glue'})
    extra_items.update({'rose_quartz': 'Rose Quartz', 'polished_rose_quartz': 'Polished Rose Quartz',
                        'sand_paper': 'Sand Paper', 'red_sand_paper': 'Red Sand Paper',
                        'electron_tube': 'Electron Tube', 'brass_hand': 'Brass Hand'})
    extra_items.update({'empty_blaze_burner': 'Empty Blaze Burner', 'blaze_cake': 'Blaze Cake', 'creative_blaze_cake': 'Creative Blaze Cake'})
    for locale in ["en_us","ru_ru"]:
        path=CONTENT / ("assets/create/lang/"+locale+".json")
        lang=json.loads(path.read_text())
        lang.update({"block.create."+k:v for k,v in extra_blocks.items()})
        lang.update({"item.create."+k:v for k,v in extra_items.items()})
        lang["itemGroup.create.palettes"] = "Create's Building Blocks"
        lang['create.contraptions.moving_container'] = (
            'Moving %1$s' if locale == 'en_us' else '%1$s на штуковине')
        filter_labels = {
            '': ('Filter', 'Фильтр'),
            '.click_to_set': ('Click with item to set', 'Кликните с предметом в руке для установки'),
            '.click_to_replace': ('Click with item to replace', 'Кликните с предметом в руке для замены'),
            '.extracted_amount': ('Extracted Amount', 'Извлекаемое количество'),
            '.up_to': ('Up to', 'До'),
            '.exactly': ('Exactly', 'Точно'),
            '.any_amount_short': ('Any', 'Любое'),
            '.hold_to_set_amount': ('Click and hold for amount', 'Удерживайте для изменения количества'),
        }
        lang.update({'create.logistics.filter' + key: values[locale == 'ru_ru']
                     for key, values in filter_labels.items()})
        lang['create.logistics.recipe_filter'] = (
            'Recipe Filter' if locale == 'en_us' else 'Фильтр рецепта')
        deployer_labels = {
            'header': ('Deployer Information', 'Информация об автономном активаторе:'),
            'using': ('Mode: Use', 'Режим: Использование/Размещение'),
            'punching': ('Mode: Attack', 'Режим: Атака/Разрушение'),
            'contains': ('Item: %1$s x%2$s', 'Держит: %1$s x%2$s'),
        }
        lang.update({'create.tooltip.deployer.' + key: values[locale == 'ru_ru']
                     for key, values in deployer_labels.items()})
        value_labels = {
            'hold_to_edit': ('Click and hold to edit', 'Удерживайте для изменения'),
            'release_to_confirm': ('Release %1$s to Confirm', 'Отпустите %1$s для подтверждения'),
        }
        lang.update({'create.gui.value_settings.' + key: values[locale == 'ru_ru']
                     for key, values in value_labels.items()})
        glue_status = {
            'first_pos': ('First corner selected', 'Первый угол выбран'),
            'abort': ('Selection cleared', 'Выделение сброшено'),
            'cannot_reach': ('The selected blocks must touch', 'Выбранные блоки должны соединяться'),
            'not_enough': ('You need more glue', 'Нужно больше клея'),
            'too_far': ('The selection is too large', 'Выделение слишком большое'),
            'success': ('Glue applied', 'Клей нанесён'),
        }
        lang.update({'create.super_glue.' + key: values[locale == 'ru_ru'] for key, values in glue_status.items()})
        chassis_labels = {
            'distance': ('Distance', 'Расстояние'),
            'range': ('Range of Sticky Sides', 'Радиус прикрепления липкой стороны'),
            'radius': ('Radius when Sticky', 'Радиус прикрепления липкой стороны'),
        }
        lang.update({'create.contraptions.chassis.' + key: values[locale == 'ru_ru']
                     for key, values in chassis_labels.items()})
        if locale == "ru_ru":
            lang.update({'block.create.linear_chassis': 'Линейное шасси',
                         'block.create.secondary_linear_chassis': 'Вторичное линейное шасси',
                         'block.create.radial_chassis': 'Радиальное шасси'})
            lang['item.create.super_glue'] = 'Суперклей'
            lang.update({'item.create.rose_quartz': 'Розовый кварц',
                         'item.create.polished_rose_quartz': 'Полированный розовый кварц',
                         'item.create.sand_paper': 'Наждачная бумага',
                         'item.create.red_sand_paper': 'Красная наждачная бумага',
                         'item.create.electron_tube': 'Электронная лампа',
                         'item.create.brass_hand': 'Латунная рука'})
            lang['block.create.adjustable_chain_gearshift'] = 'Регулируемая цепная коробка передач'
            lang['block.create.flywheel'] = 'Маховик'
            lang.update(palette_ru)
            lang.update({'block.create.zinc_ore': 'Цинковая руда',
                         'block.create.deepslate_zinc_ore': 'Цинконосный глубинный сланец',
                         'block.create.raw_zinc_block': 'Блок рудного цинка',
                         'item.create.raw_zinc': 'Рудный цинк'})
            lang["itemGroup.create.palettes"] = "Create: Строительные блоки"
            lang['block.create.copper_casing'] = 'Медный корпус'
            lang['block.create.mechanical_bearing'] = 'Механический вращатель'
            lang['block.create.windmill_bearing'] = 'Вращатель ветряной мельницы'
            lang['block.create.clockwork_bearing'] = 'Часовой механизм'
            lang.update({'block.create.rope_pulley': 'Верёвочная лебёдка', 'block.create.rope': 'Верёвка',
                         'block.create.pulley_magnet': 'Лебёдочный магнит'})
            lang.update({'block.create.gantry_shaft': 'Вал линейного привода',
                         'block.create.gantry_carriage': 'Каретка линейного привода'})
            lang.update({'block.create.sail_frame': 'Рама паруса ветряной мельницы',
                         'block.create.white_sail': 'Парус ветряной мельницы'})
            lang['block.create.deployer'] = 'Автономный активатор'
            lang['block.create.' + PSI] = 'Портативный интерфейс хранения'
            lang['block.create.' + CHUTE] = 'Жёлоб'
            lang['block.create.' + SMART_CHUTE] = 'Умный жёлоб'
            lang['block.create.' + FUNNEL] = 'Андезитовый шлюз'
            lang['block.create.' + BRASS_FUNNEL] = 'Латунный шлюз'
            lang['block.create.' + CRUSHER] = 'Дробильное колесо'
            lang['block.create.' + CRAFTER] = 'Механический сборщик'
            lang.update({'item.create.crafter_slot_cover': 'Заглушка слота сборщика',
                         'item.create.incomplete_precision_mechanism': 'Незаконченный механизм точности',
                         'item.create.precision_mechanism': 'Механизм точности',
                         'item.create.crushed_raw_copper': 'Дроблёная рудная медь',
                         'item.create.crushed_raw_gold': 'Дроблёное рудное золото',
                         'item.create.crushed_raw_iron': 'Дроблёное рудное железо',
                         'item.create.crushed_raw_zinc': 'Дроблёный рудный цинк',
                         'item.create.copper_nugget': 'Кусочек меди',
                         'item.create.powdered_obsidian': 'Обсидиановый порошок',
                         'item.create.experience_nugget': 'Кусочек опыта'})
            lang['block.create.' + ITEM_DRAIN] = 'Осушитель предметов'
            lang['block.create.' + FLUID_TANK] = 'Жидкостный бак'
            lang['block.create.' + PIPE] = 'Жидкостная труба'
            lang['block.create.' + PUMP] = 'Механическая помпа'
            lang['block.create.' + VALVE] = 'Жидкостный вентиль'
            lang['block.create.' + STICKER] = 'Блок-липучка'
            lang.update({f'block.create.{name}': text for name, text in ENCASED_COGS_RU.items()})
            lang.update({"block.create.mechanical_piston": "Механический поршень",
                         "block.create.sticky_mechanical_piston": "Липкий механический поршень",
                         "block.create.mechanical_piston_head": "Головка механического поршня",
                         "block.create.piston_extension_pole": "Удлинитель поршня"})
            lang.update({"block.create.mechanical_harvester": "Механический комбайн",
                         "block.create.mechanical_plough": "Механический плуг",
                         "block.create.mechanical_saw": "Механическая пила"})
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
version = "0.8.0-alpha.16"
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
            declaration["default_state"] = {"facing": "north"}
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
             for path in sorted(CONTENT.rglob("*"))
             if path.is_file() and path.suffix in {".json", ".toml"}
             and "target" not in path.relative_to(CONTENT).parts}
    bundled = NATIVE / "data/content.json"
    bundled.parent.mkdir(parents=True, exist_ok=True)
    bundled.write_text(json.dumps(files, indent=2, ensure_ascii=False) + "\n")
    with zipfile.ZipFile(bundled.with_name("textures.zip"), "w", zipfile.ZIP_DEFLATED) as archive:
        for texture in sorted((CONTENT / "assets/create/textures").glob("*/fallback/*.png")):
            entry = zipfile.ZipInfo(texture.relative_to(CONTENT).as_posix())
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, texture.read_bytes())
    from catalog import BLOCKS as all_blocks, ITEMS
    print(f"Generated {len(all_blocks)} blocks, {len(ITEMS)} standalone items and bundled foundation content.")


if __name__ == "__main__":
    main()
