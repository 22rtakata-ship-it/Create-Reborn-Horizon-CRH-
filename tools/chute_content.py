"""Canonical Chute states with built-in vanilla-textured models."""

from belt_shapes import clip, transform

CHUTE = 'chute'
ANGLES = {'south': 0, 'west': 90, 'north': 180, 'east': 270}


def chute(write, model):
    normal = [([1, 8, 1], [15, 16, 15]), ([2, 0, 2], [14, 8, 14])]
    boxed = [([0, 0, 0], [16, 16, 16])]
    shapes = {'normal': normal, 'window': normal, 'intersection': boxed, 'encased': boxed}
    shape_names = {'normal': 'block', 'window': 'block_window',
                   'intersection': 'block_intersection', 'encased': 'block_intersection'}
    # ChuteShapes.createSlope grows one panel at a time in +Y and +Z.
    slope = [[1, -15+i, i, 15, 4+i, i+1] for i in range(16)]
    cube = [[0, 0, 0, 16, 16, 16]]
    join = cube + clip(slope, [0, -16, 0], [16, 0, 16])
    outline = {f'down,{name}': [[coord / 16 for coord in start + end] for start, end in boxes]
               for name, boxes in shapes.items()}
    collision = dict(outline)
    for facing in ANGLES:
        for name in shapes:
            boxes = join if name in ('intersection', 'encased') else slope
            key = f'{facing},{name}'
            outline[key] = transform(boxes, facing)
            collision[key] = clip(outline[key], [0, 0, 0], [1, 24/16, 1])
    properties = ['facing', 'shape']
    write('data/create/horizon/block/chute.json', {
        'template': 'minecraft:gold_block', 'item': True,
        'properties': {'hardness': 3.0, 'explosion_resistance': 6.0, 'occludes': False,
                       'requires_tool': True, 'sounds': 'minecraft:netherite_block'},
        'state_properties': {'facing': {'type': 'enum', 'values': ['down', *ANGLES]},
                             'shape': {'type': 'enum', 'values': list(shapes)},
                             'waterlogged': {'type': 'boolean'}},
        'default_state': {'facing': 'down', 'shape': 'normal', 'waterlogged': 'false'},
        'shapes': {'properties': properties, 'values': outline},
        'collision_shapes': {'properties': properties, 'values': collision},
    })
    texture = 'minecraft:block/iron_block'
    model('chute/block', normal, texture)
    model('chute/block_intersection', boxed, 'minecraft:block/polished_andesite')
    model('chute/block_window', [([1, 14, 1], [15, 16, 15]), ([2, 0, 2], [14, 2, 14]),
                                 ([2, 2, 2], [4, 14, 14]), ([12, 2, 2], [14, 14, 14])], texture)
    model('chute/block_diagonal', [(box[:3], box[3:]) for box in slope], texture)
    model('chute/block_diagonal_intersection', [(box[:3], box[3:]) for box in join], texture)
    variants = {
        f'facing=down,shape={shape},waterlogged={wet}': {'model': f'create:block/chute/{name}'}
        for shape, name in shape_names.items() for wet in ('true', 'false')
    }
    for face, angle in ANGLES.items():
        for shape in shapes:
            suffix = '_intersection' if shape in ('intersection', 'encased') else ''
            for wet in ('true', 'false'):
                variants[f'facing={face},shape={shape},waterlogged={wet}'] = {
                    'model': f'create:block/chute/block_diagonal{suffix}', 'y': angle}
    write('assets/create/blockstates/chute.json', {'variants': variants})
    write('assets/create/models/item/chute.json', {'parent': 'create:block/chute/block'})
    write('assets/create/items/chute.json', {'model': {'type': 'minecraft:model', 'model': 'create:item/chute'}})
