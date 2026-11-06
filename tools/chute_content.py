"""Canonical vertical Chute states with built-in vanilla-textured models."""

CHUTE = 'chute'


def chute(write, model):
    normal = [([1, 8, 1], [15, 16, 15]), ([2, 0, 2], [14, 8, 14])]
    boxed = [([0, 0, 0], [16, 16, 16])]
    shapes = {'normal': normal, 'window': normal, 'intersection': boxed, 'encased': boxed}
    shape_names = {'normal': 'block', 'window': 'block_window',
                   'intersection': 'block_intersection', 'encased': 'block_intersection'}
    write('data/create/horizon/block/chute.json', {
        'template': 'minecraft:gold_block', 'item': True,
        'properties': {'hardness': 3.0, 'explosion_resistance': 6.0, 'occludes': False,
                       'requires_tool': True, 'sounds': 'minecraft:netherite_block'},
        'state_properties': {'facing': {'type': 'enum', 'values': ['down']},
                             'shape': {'type': 'enum', 'values': list(shapes)},
                             'waterlogged': {'type': 'boolean'}},
        'default_state': {'facing': 'down', 'shape': 'normal', 'waterlogged': 'false'},
        'shapes': {'property': 'shape', 'values': {
            name: [[coord / 16 for coord in start + end] for start, end in boxes]
            for name, boxes in shapes.items()
        }},
    })
    texture = 'minecraft:block/iron_block'
    model('chute/block', normal, texture)
    model('chute/block_intersection', boxed, 'minecraft:block/polished_andesite')
    model('chute/block_window', [([1, 14, 1], [15, 16, 15]), ([2, 0, 2], [14, 2, 14]),
                                 ([2, 2, 2], [4, 14, 14]), ([12, 2, 2], [14, 14, 14])], texture)
    write('assets/create/blockstates/chute.json', {'variants': {
        f'facing=down,shape={shape},waterlogged={wet}': {'model': f'create:block/chute/{name}'}
        for shape, name in shape_names.items() for wet in ('true', 'false')
    }})
    write('assets/create/models/item/chute.json', {'parent': 'create:block/chute/block'})
    write('assets/create/items/chute.json', {'model': {'type': 'minecraft:model', 'model': 'create:item/chute'}})
