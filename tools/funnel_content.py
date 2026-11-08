"""Authored fallback geometry for the ordinary Andesite Funnel."""

BLOCK = 'andesite_funnel'
FACES = ('down', 'up', 'north', 'south', 'west', 'east')
NORTH = [(2, 2, 14, 14, 14, 18), (1, 8, 12, 15, 15, 14),
         (0.1, 13, 7, 15.9, 15, 11), (0.1, 9, 8, 15.9, 13, 12),
         (0.1, 5, 9, 15.9, 9, 13), (0.1, 1, 10, 15.9, 5, 14),
         (0.1, -1, 11, 15.9, 1, 15)]
UP = [(2, -2, 2, 14, 8, 14), (1, 1, 1, 15, 8, 15), (0, 4, 0, 16, 10, 16)]
Y_ROTATION = {'north': 0, 'east': 90, 'south': 180, 'west': 270}


def clockwise(box):
    x1, y1, z1, x2, y2, z2 = box
    return 16 - z2, y1, x1, 16 - z1, y2, x2


def shape(facing):
    if facing == 'up':
        return UP
    if facing == 'down':
        return [(x1, 16 - y2, z1, x2, 16 - y1, z2) for x1, y1, z1, x2, y2, z2 in UP]
    boxes = NORTH
    for _ in range(Y_ROTATION[facing] // 90):
        boxes = [clockwise(box) for box in boxes]
    return boxes


def funnel(write, model):
    shapes = {'property': 'facing', 'values': {
        face: [[value / 16 for value in box] for box in shape(face)] for face in FACES
    }}
    write(f'data/create/horizon/block/{BLOCK}.json', {
        'template': 'minecraft:stone', 'item': True,
        'properties': {'occludes': False, 'requires_tool': True},
        'state_properties': {'facing': {'type': 'enum', 'values': FACES},
                             **{name: {'type': 'boolean'} for name in ('extracting', 'powered', 'waterlogged')}},
        'default_state': {'facing': 'north', 'extracting': 'false', 'powered': 'false', 'waterlogged': 'false'},
        'shapes': shapes, 'collision_shapes': shapes,
    })
    for kind, geometry in [('horizontal', NORTH), ('vertical', UP)]:
        for powered, texture in [('off', 'minecraft:block/andesite'), ('on', 'minecraft:block/polished_andesite')]:
            model(f'funnel/{kind}_{powered}', [(box[:3], box[3:]) for box in geometry], texture)
    variants = {}
    for face in FACES:
        orientation = ({'y': Y_ROTATION[face]} if face in Y_ROTATION else {'x': 180} if face == 'down' else {})
        kind = 'horizontal' if face in Y_ROTATION else 'vertical'
        for powered, suffix in [('false', 'off'), ('true', 'on')]:
            variants[f'facing={face},powered={powered}'] = {
                'model': f'create:block/funnel/{kind}_{suffix}', **orientation,
            }
    write(f'assets/create/blockstates/{BLOCK}.json', {'variants': variants})
    write(f'assets/create/models/item/{BLOCK}.json', {'parent': 'create:block/funnel/horizontal_off'})
    write(f'assets/create/items/{BLOCK}.json', {'model': {'type': 'minecraft:model', 'model': f'create:item/{BLOCK}'}})
