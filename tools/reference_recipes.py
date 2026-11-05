#!/usr/bin/env python3
"""Export MIT recipes/tags/selected loot from pinned Create, never assets."""
import json
import math
import subprocess
from common import CREATE, PINS, ROOT
from catalog import BLOCKS, ITEMS, INTERNAL_BLOCKS


def dry_mixing(recipe):
    """The native Basin adapter's explicit item-only, unheated JSON contract.

    Do not drop unsupported fluid ingredients or accept a malformed ingredient
    as a simpler one. Original fluids/heated recipes belong to their own port.
    """
    def item_ingredient(value):
        if isinstance(value, str): return ':' in value
        if isinstance(value, list): return bool(value) and all(item_ingredient(v) for v in value)
        return (isinstance(value, dict) and set(value) in ({'item'}, {'tag'})
                and isinstance(next(iter(value.values())), str))
    def output(value):
        return (isinstance(value, dict) and set(value) <= {'id', 'count', 'chance', 'components'}
                and isinstance(value.get('id'), str) and ':' in value['id']
                and type(value.get('count', 1)) is int and 1 <= value.get('count', 1) <= 99
                and type(value.get('chance', 1)) in (int, float)
                and math.isfinite(value.get('chance', 1)) and 0 <= value.get('chance', 1) <= 1
                and isinstance(value.get('components', {}), dict))
    inputs, outputs = recipe.get('ingredients'), recipe.get('results')
    return (recipe.get('heat_requirement', 'none') == 'none'
            and type(recipe.get('processing_time', 0)) is int
            and -(1 << 31) <= recipe.get('processing_time', 0) < (1 << 31)
            and isinstance(inputs, list) and 1 <= len(inputs) <= 64
            and all(item_ingredient(i) for i in inputs)
            and isinstance(outputs, list) and 1 <= len(outputs) <= 4
            and all(output(o) for o in outputs))


def fluid_mixing(recipe):
    """Bounded unheated mixing: exact water/lava singles, ordinary items or
    NeoForge block tags. Keep the original predicate, never rewrite block tags
    as item tags or silently strip fluid/components/unknown codec fields.
    """
    ingredients = recipe.get('ingredients')
    if not isinstance(ingredients, list): return False
    items, fluids = [], []
    for ingredient in ingredients:
        if isinstance(ingredient, dict) and ingredient.get('type') == 'neoforge:single':
            if (set(ingredient) != {'type', 'fluid', 'amount'}
                    or ingredient['fluid'] not in ('minecraft:water', 'minecraft:lava')
                    or type(ingredient['amount']) is not int
                    or not 1 <= ingredient['amount'] < (1 << 31)): return False
            fluids.append(ingredient)
        elif isinstance(ingredient, dict) and ingredient.get('type') == 'neoforge:block_tag':
            if (set(ingredient) != {'type', 'tag'} or not isinstance(ingredient['tag'], str)
                    or ':' not in ingredient['tag']): return False
            # Only the validation copy uses a plain tag; the exported original
            # retains neoforge:block_tag for native BlockItem/default-state tests.
            items.append({'tag': ingredient['tag']})
        else: items.append(ingredient)
    return (1 <= len(fluids) <= 2
            and dry_mixing(dict(recipe, ingredients=items)))


def heated_mixing(recipe):
    """Source item-only heated/superheated codecs, using actual below-Basin heat.
    Fluid recipes retain their separately bounded unheated export policy.
    """
    return (recipe.get('heat_requirement') in ('heated','superheated')
            and dry_mixing(dict(recipe,heat_requirement='none')))


def compacting(recipe):
    """The same bounded Basin codec, selected by the real compacting type.

    Unimplemented fluid tags/results and unavailable Create ingredients remain
    rejected; no recipe is converted to a simpler predicate or fake output.
    """
    return (recipe.get('type') == 'create:compacting'
            and (dry_mixing(recipe) or fluid_mixing(recipe) or heated_mixing(recipe)))


def burner_data_maps(names, read):
    """Actual MIT data maps declared by CreateDataMaps (not item fuel tags).

    The pinned release has only the superheated file; regular fuel lookup uses
    the ordinary furnace fallback unless a pack provides its declared data map.
    Missing files stay missing. Do not synthesize an empty original definition.
    """
    result={}
    for kind in ['regular', 'superheated']:
        path='create/data_maps/item/'+kind+'_blaze_burner_fuels.json'
        source='src/generated/resources/data/'+path
        if source not in names: continue
        data=read(source)
        if set(data)!={'values'} or not isinstance(data['values'],dict):
            raise ValueError('Unsupported burner data map structure: '+source)
        for key,value in data['values'].items():
            if not isinstance(key,str) or ':' not in key:
                raise ValueError('Invalid burner fuel key: '+repr(key))
            time=value.get('burn_time') if isinstance(value,dict) and set(value)=={'burn_time'} else value
            if type(time) is not int or not 1<=time<(1<<31):
                raise ValueError('Unsupported burner fuel value: '+repr(value))
        result[path]=data
    return result


def main():
    base = 'src/generated/resources/data/'
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', PINS['create_revision']], cwd=CREATE, text=True).splitlines()
    known = {'create:' + n for n in BLOCKS + ITEMS}
    known_blocks = {'create:' + n for n in BLOCKS + INTERNAL_BLOCKS}
    def read(name):
        return json.loads(subprocess.check_output(['git', 'show', PINS['create_revision'] + ':' + name], cwd=CREATE))
    def dependencies(value):
        if isinstance(value, dict):
            for key, child in value.items():
                if key == 'item' and isinstance(child, str): yield child
                else: yield from dependencies(child)
        elif isinstance(value, list):
            for child in value: yield from dependencies(child)
    recipes = {}
    processing = {'create:milling', 'create:splashing', 'create:haunting', 'create:mixing', 'create:compacting'}
    cooking = {'minecraft:smelting', 'minecraft:smoking', 'minecraft:blasting'}
    for name in names:
        if not name.startswith(base + 'create/recipe/') or not name.endswith('.json'): continue
        r = read(name)
        kind = r.get('type', '')
        if kind not in {'minecraft:crafting_shaped', 'minecraft:crafting_shapeless', 'create:pressing', 'create:item_application'} | processing | cooking: continue
        if kind == 'create:mixing' and not (dry_mixing(r) or heated_mixing(r)):
            # These two actual source flows are the bounded first native fluid
            # integration. Other fluid/heat/component recipe codecs stay out.
            if name not in {base+'create/recipe/mixing/dough_by_mixing.json',
                            base+'create/recipe/mixing/mud_by_mixing.json'} or not fluid_mixing(r): continue
        if kind == 'create:compacting' and not compacting(r): continue
        # Conditional recipes depend on other mods and their ingredient codecs.
        if 'neoforge:conditions' in r: continue
        output = r.get('result', {})
        output = output if isinstance(output, str) else output.get('id')
        output = output or (r.get('results') or [{}])[0].get('id')
        outputs = [v['id'] for v in r.get('results', [])] or [output]
        if any(not isinstance(v, str) or not (v.startswith('minecraft:') or v in known) for v in outputs): continue
        deps = list(dependencies(r))
        if any(not (d.startswith('minecraft:') or d in known) for d in deps): continue
        # Processing recipes legitimately return vanilla items. Also retain the
        # real vanilla cooking recipes for supported Create ingredients (dough).
        if output not in known and kind not in processing and not (kind in cooking and any(d in known for d in deps)): continue
        recipes[name[len(base):]] = r
    # Retain upstream tag contents. Horizon's loader drops unregistered entries,
    # as documented; this does not register or pretend to implement other items.
    tags = {}
    conventional = {}
    for metal in ['iron', 'gold', 'copper']:
        conventional['c:ingots/' + metal] = ['minecraft:' + metal + '_ingot']
        conventional['c:storage_blocks/' + metal] = ['minecraft:' + metal + '_block']
    for metal in ['iron', 'gold']:
        conventional['c:nuggets/' + metal] = ['minecraft:' + metal + '_nugget']
    conventional['c:rods/wooden'] = ['minecraft:stick']
    conventional['c:dusts/redstone'] = ['minecraft:redstone']
    conventional['c:strings'] = ['minecraft:string']
    # NeoForge's verified 1.21.1 convention contains this vanilla block item;
    # its optional legacy #forge alias is unavailable here and is omitted.
    conventional['c:netherracks'] = ['minecraft:netherrack']
    colors = ['white','orange','magenta','light_blue','yellow','lime','pink','gray','light_gray','cyan','purple','blue','brown','green','red','black']
    conventional['c:glass_blocks'] = ['minecraft:glass', 'minecraft:tinted_glass'] + ['minecraft:' + color + '_stained_glass' for color in colors]
    conventional['c:glass_panes'] = ['minecraft:glass_pane'] + ['minecraft:' + color + '_stained_glass_pane' for color in colors]
    conventional['c:stones'] = ['minecraft:' + stone for stone in ['stone', 'granite', 'diorite', 'andesite', 'deepslate', 'tuff']]
    conventional['c:cobblestones'] = ['minecraft:cobblestone', 'minecraft:mossy_cobblestone', 'minecraft:cobbled_deepslate']
    conventional['c:gems/lapis'] = ['minecraft:lapis_lazuli']
    conventional['create:not_upright_on_belt'] = []
    # NeoForge 21.1.227 convention tags used by Create's manual casing recipes.
    woods = ['acacia', 'birch', 'cherry', 'dark_oak', 'jungle', 'mangrove', 'oak', 'spruce']
    conventional['c:stripped_logs'] = [
        *[f'minecraft:stripped_{wood}_log' for wood in woods],
        'minecraft:stripped_bamboo_block', 'minecraft:stripped_crimson_stem', 'minecraft:stripped_warped_stem',
    ]
    conventional['c:stripped_woods'] = [
        *[f'minecraft:stripped_{wood}_wood' for wood in woods],
        'minecraft:stripped_crimson_hyphae', 'minecraft:stripped_warped_hyphae',
    ]
    missing = []
    intrinsic_tags = ['create:upright_on_belt', 'create:not_upright_on_belt', 'c:flours', 'c:flours/wheat', 'c:foods/dough', 'c:foods/dough/wheat']
    pending = list(intrinsic_tags)
    def tag_refs(value):
        if isinstance(value, dict):
            for k,v in value.items():
                if k == 'tag': yield v
                else: yield from tag_refs(v)
        elif isinstance(value, list):
            for v in value: yield from tag_refs(v)
        elif isinstance(value,str) and value.startswith('#') and ':' in value: yield value[1:]
    for r in recipes.values(): pending.extend(tag_refs(r))
    while pending:
        namespace, path = pending.pop().split(':', 1)
        if namespace == 'minecraft': continue
        dest = namespace + '/tags/item/' + path + '.json'
        if dest in tags: continue
        name = base + dest
        if name not in names:
            fallback = conventional.get(namespace + ':' + path)
            if fallback is None:
                missing.append(namespace + ':' + path)
                continue
            tags[dest] = {'values': fallback}
        else:
            tags[dest] = read(name)
            # Upstream's datagen contains Create's additions only; supply the
            # vanilla side of convention tags alongside those additions.
            for value in conventional.get(namespace + ':' + path, []):
                if value not in tags[dest]['values']: tags[dest]['values'].append(value)
        pending.extend(tag_refs(tags[dest]))
    if missing: raise SystemExit('Missing convention definitions: ' + ', '.join(sorted(set(missing))))
    # Do not advertise furnace recipes whose only inputs are unimplemented ores.
    def available(value, seen=()):
        if isinstance(value, list): return any(available(v, seen) for v in value)
        if isinstance(value, dict):
            if value.get('type') == 'neoforge:single':
                return value.get('fluid') in ('minecraft:water', 'minecraft:lava')
            if 'item' in value: return available(value['item'], seen)
            if 'tag' in value: return available('#'+value['tag'], seen)
            if 'id' in value: return available(value['id'], seen)
            return False
        if value.startswith('#'):
            tag=value[1:]
            if tag.startswith('minecraft:'): return True
            if tag in seen: return False
            ns,path=tag.split(':',1)
            return available(tags.get(ns+'/tags/item/'+path+'.json',{}).get('values',[]),(*seen,tag))
        return value.startswith('minecraft:') or value in known
    def ingredients(r):
        if 'key' in r: return r['key'].values()
        return r.get('ingredients', [r.get('ingredient')])
    recipes={name:r for name,r in recipes.items() if all(available(i) for i in ingredients(r))}
    required=set()
    pending=intrinsic_tags + [tag for r in recipes.values() for tag in tag_refs(r)]
    while pending:
        ns,path=pending.pop().split(':',1)
        dest=ns+'/tags/item/'+path+'.json'
        if dest in required or dest not in tags: continue
        required.add(dest);pending.extend(tag_refs(tags[dest]))
    tags={name:t for name,t in tags.items() if name in required}
    for tool in ['axe', 'pickaxe']:
        dest = 'minecraft/tags/block/mineable/' + tool + '.json'
        tag = read(base + dest)
        tag['values'] = [v for v in tag['values'] if isinstance(v, str) and v in known_blocks]
        tags[dest] = tag
    for kind in ['block', 'item']:
        dest = f'create/tags/{kind}/casing.json'
        tag = read(base + dest)
        tag['values'] = [value for value in tag['values'] if value in known_blocks]
        tags[dest] = tag
    # Fan catalyst and permeability tags are MIT behavior data. Unimplemented
    # Create blocks must not become required tag entries in the port.
    for kind, paths in {
        'block': ['fan_transparent', 'fan_processing_catalysts/blasting', 'passive_boiler_heaters',
                  'fan_processing_catalysts/haunting', 'fan_processing_catalysts/smoking'],
        'fluid': ['fan_processing_catalysts/blasting', 'fan_processing_catalysts/splashing'],
        'entity_type': ['blaze_burner_capturable'],
    }.items():
        for path in paths:
            dest = 'create/tags/' + kind + '/' + path + '.json'
            tag = read(base + dest)
            def implemented(entry):
                name = entry if isinstance(entry, str) else entry.get('id', '')
                return name.startswith(('minecraft:', '#minecraft:')) or (kind == 'block' and name in known_blocks)
            tag['values'] = [entry for entry in tag['values'] if implemented(entry)]
            tags[dest] = tag
    # 1.21.1's source ID uses "convertable"; 26.3 renamed the vanilla tag to
    # "convertible". Keep the original recipe's actual BLOCK-tag predicate.
    # This is our convention definition for its three verified source members.
    tags['minecraft/tags/block/convertable_to_mud.json']={'values':[
        'minecraft:dirt','minecraft:coarse_dirt','minecraft:rooted_dirt']}
    maps=burner_data_maps(names,read)
    # Keep the complete source pool and explosion/random-sequence policy for
    # this newly supported block, rather than a generic unconditional drop.
    loot = {path: read(base + path) for path in [
        'create/loot_table/blocks/encased_chain_drive.json',
        'create/loot_table/blocks/adjustable_chain_gearshift.json',
        'create/loot_table/blocks/copper_casing.json']}
    result = {'reference': PINS['create_revision'], 'license': 'MIT recipe/data-map/loot data; original definitions of vanilla convention tags', 'recipes': recipes, 'tags': tags, 'data_maps':maps, 'loot_tables':loot}
    dest = ROOT / 'tools/data/recipes.json'; dest.parent.mkdir(exist_ok=True); dest.write_text(json.dumps(result, indent=2) + '\n')
    print('Exported', len(recipes), 'recipes,', len(tags), 'tags and',len(maps),'data maps from MIT datapack sources')


if __name__ == '__main__': main()
