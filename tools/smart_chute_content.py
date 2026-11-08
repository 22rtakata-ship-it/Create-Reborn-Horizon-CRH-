"""Canonical Smart Chute states and authored vanilla-textured fallback."""

BLOCK = 'smart_chute'


def smart_chute(write, model):
    shape = {'default': [[0, 0, 0, 1, 1, 1]]}
    write(f'data/create/horizon/block/{BLOCK}.json', {
        # Glass gives this full cube the original nonconducting/non-suffocating defaults.
        'template': 'minecraft:glass', 'item': True,
        'properties': {'hardness': 3.0, 'explosion_resistance': 6.0, 'occludes': False,
                       'requires_tool': True, 'sounds': 'minecraft:netherite_block', 'map_color': 21},
        'state_properties': {'powered': {'type': 'boolean'}},
        'default_state': {'powered': 'true'}, 'shapes': shape, 'collision_shapes': shape,
    })
    model(f'{BLOCK}/block', [([0, 0, 0], [16, 16, 16])], 'minecraft:block/polished_andesite')
    model(f'{BLOCK}/block_powered', [([0, 0, 0], [16, 16, 16])], 'minecraft:block/redstone_block')
    write(f'assets/create/blockstates/{BLOCK}.json', {'variants': {
        'powered=false': {'model': f'create:block/{BLOCK}/block'},
        'powered=true': {'model': f'create:block/{BLOCK}/block_powered'},
    }})
    write(f'assets/create/models/item/{BLOCK}.json', {'parent': f'create:block/{BLOCK}/block'})
    write(f'assets/create/items/{BLOCK}.json', {'model': {'type': 'minecraft:model', 'model': f'create:item/{BLOCK}'}})
