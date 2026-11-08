#!/usr/bin/env python3
# Copyright (c) 2026 create-horizon contributors
# SPDX-License-Identifier: MIT
"""Check the pinned andesite palette data and the real Horizon Stonecutter."""

import argparse
import errno
import json
from pathlib import Path
import shutil
import subprocess
from zipfile import ZipFile

from burner_heat_java_checks import number
from catalog import ANDESITE_PALETTE
from chain_drive_checks import xyz
from common import CREATE, PINS, ROOT
from native_check_session import NativeCheckSession, wait_loaded
from reference_recipes import loot_for_horizon


STONECUTTER = (1200, 120, 301)
PALETTE = [f'create:{name}' for name in ANDESITE_PALETTE]
REVERSE = 'minecraft:andesite'
OUTPUTS = [*PALETTE, REVERSE]
TAG = 'create:stone_types/andesite'
PICKAXE = 'minecraft:iron_pickaxe'


def recipe_id(item):
    return f'create:{item.split(":", 1)[1]}_from_stone_types_andesite_stonecutting'


def pinned(path):
    return json.loads(subprocess.check_output(
        ['git', 'show', PINS['create_revision'] + ':src/generated/resources/data/' + path], cwd=CREATE,
    ))


def source_checks():
    def content(path):
        return json.loads((ROOT / 'content/data' / path).read_text())

    for item in OUTPUTS:
        path = f'create/recipe/{recipe_id(item).split(":", 1)[1]}.json'
        original = pinned(path)
        assert original == {'type': 'minecraft:stonecutting', 'ingredient': {'tag': TAG},
                            'result': {'count': 1, 'id': item}}, original
        assert content(path) == dict(original, ingredient='#' + TAG), path
    path = 'create/tags/item/stone_types/andesite.json'
    assert content(path) == pinned(path), path
    pickaxe = 'minecraft/tags/block/mineable/pickaxe.json'
    assert set(PALETTE) <= set(content(pickaxe)['values']) <= set(pinned(pickaxe)['values'])
    for name in ANDESITE_PALETTE:
        loot = f'create/loot_table/blocks/{name}.json'
        assert content(loot) == loot_for_horizon(pinned(loot)), loot


def menu_input(call, item):
    slots = [row['slot'] for row in call('menu')['slots']
             if row['slot'] < 36 and row['stack']['item'] == item]
    assert len(slots) == 1, (item, slots)
    call('click-slot', slots[0], 'shift')
    call('wait', 3)
    menu = call('menu')
    assert menu['station'] == 'Stonecutter' and menu['carried'] is None, menu
    assert any(row['slot'] == 600 and row['stack']['item'] == item and row['stack']['count'] == 1
               for row in menu['slots']), menu
    assert set(map(recipe_id, OUTPUTS)) <= set(menu['info']['recipes']), menu
    return menu['info']['recipes']


def cut(call, item):
    menu = call('menu')
    i = menu['info']['recipes'].index(recipe_id(item))
    assert i < 12, menu
    widgets = call('ui')['widgets']
    x, y, _, _ = next(widget['bounds'] for widget in widgets if widget['id'] == 'slot-600')
    # The vanilla input starts at (20, 33), the four-column recipe grid at (52, 14).
    call('click-at', x - 20 + 60 + i % 4 * 16, y - 33 + 22 + i // 4 * 18)
    selected = call('menu')['info']
    assert selected['result'] == f'{item} x1', selected
    call('click-slot', 650, 'shift')
    call('wait', 3)
    menu = call('menu')
    assert menu['carried'] is None and all(row['slot'] < 600 for row in menu['slots']), menu
    assert sum(row['stack']['count'] for row in menu['slots'] if row['stack']['item'] == item) == 1, menu


def native_checks(call):
    def command(text):
        return call('command', text)

    command('gamemode spectator')
    command('tp @s 1200.5 125 304.5')
    call('wait-ready', 120)
    wait_loaded(call, [(1196, 119, 299), (1204, 125, 306)])
    command('fill 1196 120 299 1204 125 306 minecraft:air')
    command('fill 1196 119 299 1204 119 306 minecraft:smooth_stone')
    command('setblock ' + xyz(STONECUTTER) + ' minecraft:stonecutter')
    command('tp @s 1200.5 120 304.5')
    command('gamemode survival')
    command('clear @s')
    call('interact', *STONECUTTER, 'up')
    for item in PALETTE:
        command(f'give @s {REVERSE} 1')
        call('wait', 3)
        recipes = menu_input(call, REVERSE)
        cut(call, item)
    # Every registered variant must itself match the original family tag.
    for item in PALETTE[:-1]:
        menu_input(call, item)
        call('click-slot', 600, 'shift')
        call('wait', 3)
    menu_input(call, PALETTE[-1])
    cut(call, REVERSE)
    menu_input(call, REVERSE)
    cut(call, PALETTE[-1])
    call('close')
    assert call('inventory')['counts'] == dict.fromkeys(PALETTE, 1), call('inventory')

    call('open-inventory')
    for item in PALETTE:
        slot = next(row['slot'] for row in call('menu')['slots'] if row['stack']['item'] == item)
        if slot >= 9:
            call('click-slot', slot, 'shift')
            call('wait', 3)
    call('close')
    positions = [(x, 120, 302) for x in range(1198, 1203)]
    for pos, item in zip(positions, PALETTE, strict=True):
        x, y, z = pos
        command(f'tp @s {x}.5 {y} {z + 2}.5')
        call('wait', 3)
        call('select', item)
        call('look-at', 'block', x, y - 1, z, 'up')
        assert call('looking-at')['block']['pos'] == [x, y - 1, z], pos
        call('mouse', 'right', 'click')
        call('wait', 3)
        placed = call('block', *pos)
        assert placed['block'] == item and '=' not in placed['state'], placed
    command('give @s ' + PICKAXE)
    call('wait', 3)
    call('select', PICKAXE)
    for pos, item in zip(positions, PALETTE, strict=True):
        x, y, z = pos
        command(f'tp @s {x}.5 {y} {z + 2}.5')
        call('wait', 3)
        call('break', *pos)
        call('wait', 3)
        assert call('block', *pos)['block'] == 'minecraft:air', pos
        selector = ('@e[type=minecraft:item,distance=..8,limit=1,nbt={Item:{id:"' + item + '"}}]')
        raw = ' '.join(command(f'data get entity {selector} Item')['chat'])
        dropped = int(number({'nbt': raw}, 'count', 0)) if item in raw else 0
        picked_up = call('inventory')['counts'].get(item, 0)
        assert dropped + picked_up == 1, (item, dropped, picked_up, raw)
    return dict(stonecutter=recipes, outputs=OUTPUTS, placed_and_dropped=PALETTE)


def imported_assets(session, jar):
    target = session.scratch / 'horizon-mods/create-resources/create.jar'
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(jar, target)
    session.start()
    receipts = list((session.scratch / '.horizon-cache/create-assets/imports').glob('*/import.json'))
    assert len(receipts) == 1, receipts
    pack = receipts[0].parent / 'pack'
    paths = []
    with ZipFile(jar) as archive:
        for name in ANDESITE_PALETTE:
            base = f'assets/create/models/block/{name}.json'
            model = json.loads(archive.read(base))
            paths.extend([base, f'assets/create/models/item/{name}.json'])
            paths.extend('assets/create/textures/' + texture.removeprefix('create:') + '.png'
                         for texture in model['textures'].values())
        for path in set(paths):
            assert (pack / path).read_bytes() == archive.read(path), path
    return len(set(paths))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('game', 'resources', 'ctl', 'create-jar'):
        parser.add_argument('--' + name, type=Path)
    args = parser.parse_args()
    source_checks()
    if not any((args.game, args.resources, args.ctl, args.create_jar)):
        print('Pinned source recipes, item tags and loot match.')
        return
    if not all((args.game, args.resources, args.ctl)):
        parser.error('Native checks require --game, --resources and --ctl')
    session = NativeCheckSession(args.game, args.resources, args.ctl, None,
                                 name='andesite-palette-native', world='CreateAndesitePalette')
    print(session.scratch, flush=True)
    try:
        session.start()
        assert 'bundled content could not be loaded' not in session.log.read_text(), session.log
        result = native_checks(session.call)
        session.stop()
        if args.create_jar:
            result['imported_exact_files'] = imported_assets(session, args.create_jar)
    finally:
        session.stop()
        session.control_temp.cleanup()
        for folder in (session.resources, session.scratch / '.horizon-cache', session.scratch / 'horizon-mods'):
            if not folder.exists():
                continue
            try:
                shutil.rmtree(folder)
            except OSError as error:
                if error.errno != errno.ENOTEMPTY:
                    raise
                shutil.rmtree(folder)
    output = session.scratch / 'result.json'
    output.write_text(json.dumps(dict(passed=True, create=PINS['create_revision'], **result), indent=2) + '\n')
    print(output)


if __name__ == '__main__':
    main()
