#!/usr/bin/env python3
"""Export MIT datapack recipes/tags from the pinned Create source, never assets."""
import json
import subprocess
from common import CREATE, PINS, ROOT
from catalog import BLOCKS, ITEMS


def main():
    base = 'src/generated/resources/data/'
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', PINS['create_revision']], cwd=CREATE, text=True).splitlines()
    known = {'create:' + n for n in BLOCKS + ITEMS}
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
    processing = {'create:milling', 'create:splashing', 'create:haunting'}
    cooking = {'minecraft:smelting', 'minecraft:smoking', 'minecraft:blasting'}
    for name in names:
        if not name.startswith(base + 'create/recipe/') or not name.endswith('.json'): continue
        r = read(name)
        kind = r.get('type', '')
        if kind not in {'minecraft:crafting_shaped', 'minecraft:crafting_shapeless', 'create:pressing', 'create:item_application'} | processing | cooking: continue
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
    colors = ['white','orange','magenta','light_blue','yellow','lime','pink','gray','light_gray','cyan','purple','blue','brown','green','red','black']
    conventional['c:glass_blocks'] = ['minecraft:glass', 'minecraft:tinted_glass'] + ['minecraft:' + color + '_stained_glass' for color in colors]
    conventional['c:glass_panes'] = ['minecraft:glass_pane'] + ['minecraft:' + color + '_stained_glass_pane' for color in colors]
    conventional['c:stones'] = ['minecraft:' + stone for stone in ['stone', 'granite', 'diorite', 'andesite', 'deepslate', 'tuff']]
    conventional['c:cobblestones'] = ['minecraft:cobblestone', 'minecraft:mossy_cobblestone', 'minecraft:cobbled_deepslate']
    conventional['c:gems/lapis'] = ['minecraft:lapis_lazuli']
    conventional['create:not_upright_on_belt'] = []
    for kind in ['logs', 'woods']:
        conventional['c:stripped_' + kind] = ['minecraft:stripped_' + wood + ('_log' if kind == 'logs' else '_wood')
            for wood in ['oak','spruce','birch','jungle','acacia','dark_oak','mangrove','cherry']]
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
    # Fan catalyst and permeability tags are MIT behavior data. Unimplemented
    # Create blocks must not become required tag entries in the port.
    for kind, paths in {
        'block': ['fan_transparent', 'fan_processing_catalysts/blasting',
                  'fan_processing_catalysts/haunting', 'fan_processing_catalysts/smoking'],
        'fluid': ['fan_processing_catalysts/blasting', 'fan_processing_catalysts/splashing'],
    }.items():
        for path in paths:
            dest = 'create/tags/' + kind + '/' + path + '.json'
            tag = read(base + dest)
            def implemented(entry):
                name = entry if isinstance(entry, str) else entry.get('id', '')
                return name.startswith(('minecraft:', '#minecraft:')) or (kind == 'block' and name in known)
            tag['values'] = [entry for entry in tag['values'] if implemented(entry)]
            tags[dest] = tag
    result = {'reference': PINS['create_revision'], 'license': 'MIT recipe data; original definitions of vanilla convention tags', 'recipes': recipes, 'tags': tags}
    dest = ROOT / 'tools/data/recipes.json'; dest.parent.mkdir(exist_ok=True); dest.write_text(json.dumps(result, indent=2) + '\n')
    print('Exported', len(recipes), 'recipes and', len(tags), 'tags from MIT datapack sources')


if __name__ == '__main__': main()
