"""State and vanilla-textured fallback for ordinary fluid pipes and pumps."""

from itertools import product

PIPE, PUMP = 'fluid_pipe', 'mechanical_pump'
FACES = ('down', 'up', 'north', 'south', 'west', 'east')
AXES = dict(down=1, up=1, north=2, south=2, west=0, east=0)


def arms():
    for index, face in enumerate(FACES):
        start, end = [4, 4, 4], [12, 12, 12]
        axis = AXES[face]
        start[axis], end[axis] = (12, 16) if index % 2 else (0, 4)
        yield face, [start, end]


def fluid_pipes(write, model, axial_boxes):
    tube = {face: box for face, box in arms()}
    core = [[4, 4, 4], [12, 12, 12]]
    property_names = list(FACES)
    values = {}
    for selected in product((False, True), repeat=len(FACES)):
        key = ','.join(str(flag).lower() for flag in selected)
        boxes = [core] + [tube[side] for side, flag in zip(FACES, selected, strict=True) if flag]
        values[key] = [[coordinate / 16 for corner in box for coordinate in corner] for box in boxes]
    pipe_shape = {'properties': property_names, 'values': values}
    physical = {'hardness': 3.0, 'explosion_resistance': 6.0, 'sounds': 'minecraft:copper_block',
                'requires_tool': True, 'occludes': False}
    write(f'data/create/horizon/block/{PIPE}.json', {
        'template': 'minecraft:glass', 'item': True,
        'properties': {**physical, 'map_color': 15},
        'state_properties': {name: {'type': 'boolean'} for name in (*FACES, 'waterlogged')},
        'default_state': dict.fromkeys((*FACES, 'waterlogged'), 'false'),
        'shapes': pipe_shape, 'collision_shapes': pipe_shape,
    })
    pump_shape = {'property': 'facing', 'values': {face: axial_boxes(face, [(2, 0, 16)]) for face in FACES}}
    write(f'data/create/horizon/block/{PUMP}.json', {
        'template': 'minecraft:glass', 'item': True,
        'properties': {**physical, 'map_color': 11},
        'state_properties': {'facing': {'type': 'enum', 'values': FACES}, 'waterlogged': {'type': 'boolean'}},
        'default_state': {'facing': 'down', 'waterlogged': 'false'},
        'shapes': pump_shape, 'collision_shapes': pump_shape,
    })
    model(f'{PIPE}/fallback_core', [core], 'minecraft:block/copper_block')
    for face, box in tube.items():
        model(f'{PIPE}/fallback_{face}', [box], 'minecraft:block/copper_block')
    write(f'assets/create/blockstates/{PIPE}.json', {'multipart': [
        {'apply': {'model': f'create:block/{PIPE}/fallback_core'}},
        *({'when': {face: 'true'}, 'apply': {'model': f'create:block/{PIPE}/fallback_{face}'}} for face in FACES),
    ]})
    model(f'{PIPE}/fallback_item', [core, tube['west'], tube['east']], 'minecraft:block/copper_block')
    write(f'assets/create/models/block/{PUMP}/fallback_body.json', {
        'textures': {'body': 'minecraft:block/copper_block', 'outlet': 'minecraft:block/exposed_copper',
                     'particle': '#body'},
        'elements': [{'from': low, 'to': high, 'faces': dict.fromkeys(FACES, {'texture': '#' + material})}
                     for low, high, material in [([5, 0, 5], [11, 16, 11], 'body'),
                                                  ([3, 5, 3], [13, 11, 13], 'body'),
                                                  ([4, 13, 4], [12, 15, 12], 'outlet')]],
    })
    model(f'{PUMP}/cog', [([2, 2, 7], [14, 14, 9]), ([0, 5, 7], [16, 11, 9]), ([5, 0, 7], [11, 16, 9])],
          'minecraft:block/oak_planks')
    rotations = {'down': (180, 0), 'up': (0, 0), 'north': (90, 0), 'south': (90, 180),
                 'west': (90, 270), 'east': (90, 90)}
    write(f'assets/create/blockstates/{PUMP}.json', {'variants': {
        f'facing={face}': {'model': f'create:block/{PUMP}/fallback_body', 'x': x, 'y': y}
        for face, (x, y) in rotations.items()
    }})
    model(f'{PUMP}/fallback_item', [([5, 0, 5], [11, 16, 11]), ([2, 7, 2], [14, 9, 14])],
          'minecraft:block/copper_block')
    for name in (PIPE, PUMP):
        write(f'assets/create/models/item/{name}.json', {'parent': f'create:block/{name}/fallback_item'})
        write(f'assets/create/items/{name}.json', {
            'model': {'type': 'minecraft:model', 'model': f'create:item/{name}'},
        })
