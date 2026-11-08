#!/usr/bin/env python3
"""Build and verify a clean world for comparing the fallback and JAR art."""

import argparse
import json
from pathlib import Path
import shutil
from textwrap import wrap

from bearing_native_checks import Batch
from catalog import BLOCKS, INTERNAL_BLOCKS, ITEMS
from common import ROOT
from contraption_review_stations import sign
from native_check_session import NativeCheckSession


Y = 160
COLUMNS = 12
BLOCK_STEP = 6
FIRST_ITEM_ROW = 68
SPAWN = (1.5, Y + 2, -7.5)

VISIBLE_INTERNAL = [
    name for name in INTERNAL_BLOCKS if name not in ('water_wheel_structure', 'crushing_wheel_controller')
]
SPECIAL = {
    'belt': 'facing=east,slope=horizontal,part=middle',
    'fluid_pipe': 'east=true,west=true',
    'hand_crank': 'facing=south',
    'large_water_wheel': 'axis=x',
    'sail_frame': 'facing=south',
    'shaft': 'axis=x',
}
VARIANTS = [
    ('blaze_burner', 'blaze=kindled', 'Kindled Burner'),
    ('blaze_burner', 'blaze=seething', 'Seething Burner'),
    ('fluid_tank', 'shape=plain', 'Solid Fluid Tank'),
    ('mechanical_piston_head', 'facing=south,type=sticky', 'Sticky Head'),
    ('smart_chute', 'powered=true', 'Powered Smart Chute'),
    ('sticker', 'powered=true', 'Powered Sticker'),
    ('adjustable_chain_gearshift', 'powered=true', 'Powered Chain Gearshift'),
]
STATES = ROOT / 'content/data/create/horizon/block'


def coordinates(index, start_z=0):
    return (index % COLUMNS) * BLOCK_STEP, Y + 1, start_z + (index // COLUMNS) * BLOCK_STEP


def artwork_state(name):
    if name in SPECIAL:
        return SPECIAL[name]
    if name.endswith('_sail'):
        return 'facing=south'
    fields = json.loads((STATES / f'{name}.json').read_text()).get('state_properties', {})
    if 'south' in fields.get('facing', {}).get('values', []):
        return 'facing=south'
    return ''


def label(batch, pos, name):
    lines = wrap(name, width=14)[:4]
    lines += [''] * (4 - len(lines))
    sign(batch, pos, lines)
    messages = json.dumps(lines)
    batch.command(f'data merge block {pos[0]} {pos[1]} {pos[2]} '
                  f'{{back_text:{{messages:{messages},color:"black"}}}}')


def place_block(batch, index, name, state, title):
    x, y, z = coordinates(index)
    if name == 'hand_crank':
        batch.put((x, y, z - 1), 'minecraft:polished_deepslate')
    block = f'create:{name}'
    batch.put((x, y, z), block + (f'[{state}]' if state else ''))
    label(batch, (x, y, z + 2), title)
    batch.expect((x, y, z), block)


def create_gallery(session):
    batch = Batch()
    batch.load((32, Y + 3, 39))
    batch.command('forceload add -16 -16 79 95')
    batch.tick(30)
    batch.command(f'fill -8 {Y} -10 74 {Y} 92 minecraft:smooth_stone')
    batch.command(f'fill -8 {Y} 65 74 {Y} 66 minecraft:polished_deepslate')
    batch.put((8, Y + 6, 11), 'minecraft:barrier')
    batch.put((8, Y + 3, 78), 'minecraft:barrier')
    batch.command('gamerule minecraft:advance_time false')
    batch.command('time set noon')
    batch.command('weather clear')
    for index, name in enumerate(BLOCKS + VISIBLE_INTERNAL):
        place_block(batch, index, name, artwork_state(name), name.replace('_', ' ').title())
    for index, (name, state, title) in enumerate(VARIANTS, len(BLOCKS + VISIBLE_INTERNAL)):
        place_block(batch, index, name, state, title)
    for index, name in enumerate(ITEMS):
        x, y, z = coordinates(index, FIRST_ITEM_ROW)
        batch.put((x, y + 1, z), 'minecraft:polished_deepslate')
        batch.command(f'summon minecraft:item_frame {x + .5} {y + 1.5} {z + 1.05} '
                      f'{{Facing:3b,Fixed:1b,Tags:["gallery_{index}"],Item:{{id:"create:{name}",count:1}}}}')
        label(batch, (x, y, z + 2), name.replace('_', ' ').title())
        batch.command(f'data get entity @e[type=minecraft:item_frame,tag=gallery_{index},limit=1] Item.id', True)
    label(batch, (-2, Y + 1, -5), 'Create Texture Gallery')
    label(batch, (4, Y + 1, -5), 'Blocks Ahead Items Far Side')
    batch.command(f'setworldspawn 1 {Y + 2} -8')
    batch.command(f'spawnpoint @s 1 {Y + 2} -8')
    batch.command(f'tp @s {SPAWN[0]} {SPAWN[1]} {SPAWN[2]} 0 7')
    batch.command('forceload remove all')
    rows = batch.run(session, 'build-gallery')
    expected = len(BLOCKS + VISIBLE_INTERNAL + VARIANTS)
    assert len(rows) == expected + len(ITEMS) and all('bearing_check' in r for r in rows[:expected]), rows
    assert all(f'"create:{name}"' in row for name, row in zip(ITEMS, rows[expected:], strict=True)), rows
    return dict(blocks=len(BLOCKS), additional_samples=expected - len(BLOCKS), items=len(ITEMS))


def capture(session, name):
    call = session.call
    call('key', 'f1', 'tap')
    call('command', f'tp @s 8.5 {Y + 7} 11.5')
    call('look-at', 22, Y + 1, 8)
    call('wait', 35)
    call('screenshot', session.scratch / f'{name}-blocks.png')
    call('command', f'tp @s 8.5 {Y + 4} 78.5')
    call('look-at', 21, Y + 2, 70)
    call('wait', 20)
    call('screenshot', session.scratch / f'{name}-items.png')
    call('key', 'f1', 'tap')
    call('command', f'tp @s {SPAWN[0]} {SPAWN[1]} {SPAWN[2]} 0 7')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ('game', 'resources', 'ctl'):
        parser.add_argument('--' + arg, type=Path, required=True)
    parser.add_argument('--create-jar', type=Path)
    args = parser.parse_args()
    session = NativeCheckSession(args.game, args.resources, args.ctl, None,
                                 name='texture-gallery', world='Create Texture Gallery',
                                 size=(1536, 960), username='BearingReview')
    print(session.scratch, flush=True)
    try:
        report = create_gallery(session)
        local_jar = session.scratch / 'horizon-mods/create-resources/create.jar'
        try:
            session.start()
            capture(session, 'fallback')
            log = session.log.read_text()
            assert 'Create: using provisional resources' in log
            assert 'bundled content could not be loaded' not in log
            if args.create_jar:
                shutil.copyfile(args.create_jar, local_jar)
                start = session.log.stat().st_size
                session.restart()
                capture(session, 'original')
                assert 'local resources from Create' in session.log.read_text()[start:]
                local_jar.unlink()
                start = session.log.stat().st_size
                session.restart()
                assert 'Create: using provisional resources' in session.log.read_text()[start:]
        finally:
            session.stop()
            local_jar.unlink(missing_ok=True)
        world = session.scratch / 'saves' / session.world
        assert (world / 'level.dat').is_file(), world
        archive = shutil.make_archive(str(session.scratch / 'Create Texture Gallery'), 'zip',
                                      world.parent, world.name)
        print(json.dumps(dict(**report, archive=archive, world=str(world))))
    finally:
        session.control_temp.cleanup()
        for path in (session.resources, session.scratch / '.horizon-cache', session.scratch / 'horizon-mods'):
            shutil.rmtree(path, ignore_errors=True)


if __name__ == '__main__':
    main()
