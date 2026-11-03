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
    for name in names:
        if not name.startswith(base + 'create/recipe/') or not name.endswith('.json'): continue
        r = read(name)
        kind = r.get('type', '')
        if kind not in ['minecraft:crafting_shaped', 'minecraft:crafting_shapeless', 'minecraft:smelting', 'minecraft:blasting', 'create:pressing', 'create:item_application']: continue
        output = r.get('result', {})
        output = output if isinstance(output, str) else output.get('id')
        output = output or (r.get('results') or [{}])[0].get('id')
        if output not in known: continue
        if any(d.startswith('create:') and d not in known for d in dependencies(r)): continue
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
    conventional['c:glass_blocks'] = ['minecraft:glass', 'minecraft:tinted_glass'] + ['minecraft:' + color + '_stained_glass' for color in ['white','orange','magenta','light_blue','yellow','lime','pink','gray','light_gray','cyan','purple','blue','brown','green','red','black']]
    conventional['create:not_upright_on_belt'] = []
    for kind in ['logs', 'woods']:
        conventional['c:stripped_' + kind] = ['minecraft:stripped_' + wood + ('_log' if kind == 'logs' else '_wood')
            for wood in ['oak','spruce','birch','jungle','acacia','dark_oak','mangrove','cherry']]
    missing = []
    display_tags = ['create:upright_on_belt', 'create:not_upright_on_belt']
    pending = list(display_tags)
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
    pending=display_tags + [tag for r in recipes.values() for tag in tag_refs(r)]
    while pending:
        ns,path=pending.pop().split(':',1)
        dest=ns+'/tags/item/'+path+'.json'
        if dest in required or dest not in tags: continue
        required.add(dest);pending.extend(tag_refs(tags[dest]))
    tags={name:t for name,t in tags.items() if name in required}
    result = {'reference': PINS['create_revision'], 'license': 'MIT recipe data; original definitions of vanilla convention tags', 'recipes': recipes, 'tags': tags}
    dest = ROOT / 'tools/data/recipes.json'; dest.parent.mkdir(exist_ok=True); dest.write_text(json.dumps(result, indent=2) + '\n')
    print('Exported', len(recipes), 'recipes and', len(tags), 'tags from MIT datapack sources')


if __name__ == '__main__': main()
