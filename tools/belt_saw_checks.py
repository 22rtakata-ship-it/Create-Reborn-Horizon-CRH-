#!/usr/bin/env python3
# Copyright (c) 2026 create-horizon contributors
# SPDX-License-Identifier: MIT
"""Compare a Belt feeding an upward Saw while stopped, then through a Chute."""

import argparse
import json
from pathlib import Path
import shutil
import tempfile
import time

from bearing_native_checks import Batch
from burner_heat_java_checks import field, items, number
from chain_drive_checks import Original, xyz
from common import PINS, ROOT
from native_check_session import NativeCheckSession
import reference_java


X, Y, Z = 2237, 126, 1348
BELT, SAW, CHUTE, CHEST = (X, Y, Z), (X+3, Y, Z), (X+4, Y, Z), (X+4, Y-1, Z)
PROBES = (BELT, SAW, CHUTE, CHEST)
REGION = '2234 124 1345 2244 130 1351'
FORCE = '2224 1344 2255 1359'
LOG, STRIPPED = 'minecraft:oak_log', 'minecraft:stripped_oak_log'
MARK = 'belt_saw_input'
WARM, RUN = 16, 86


def setup(command, advance):
    command(f'fill {REGION} minecraft:air')
    command(f'setblock {xyz(CHEST)} minecraft:chest')
    command(f'setblock {xyz(CHUTE)} create:chute[facing=down,shape=window]')
    command(f'setblock {xyz(SAW)} create:mechanical_saw[facing=up,axis_along_first=false]')
    for index, part in enumerate(('start', 'middle', 'end')):
        command(f'setblock {X+index} {Y} {Z} create:belt[facing=east,slope=horizontal,part={part}]')
    command(f'setblock {X} {Y} {Z-1} create:creative_motor[facing=south]{{ScrollValue:-32}}')
    advance(24)
    command(f'data merge block {xyz(BELT)} {{Inventory:{{PositiveOrder:1b,Items:[{{'
            f'Item:{{id:"{LOG}",count:3,components:{{"minecraft:custom_data":{{{MARK}:37}}}}}},'
            'Pos:2.65f,PrevPos:2.65f,InDirection:5,InSegment:2,Angle:180}]}}')


def start_saw(command):
    command(f'setblock {SAW[0]} {Y} {Z-1} create:creative_motor[facing=south]{{ScrollValue:128}}')


def snapshot(raw):
    belt, saw, chute, chest = raw
    keys = ('Inventory', 'Inventory', 'Item', 'Items')
    inventories = [items(data, key) if f'{key}:' in data else {} for data, key in zip(raw, keys, strict=True)]
    for data, key, content in zip(raw[:2], keys[:2], inventories[:2], strict=True):
        if LOG in content:
            assert number({'nbt': field(data, key)}, MARK) == 37, data
    timer = [number({'nbt': saw}, key, 0) for key in ('ProcessingTime', 'RecipeTime', 'AppliedRecipe')]
    position = number({'nbt': field(belt, 'Inventory')}, 'Pos') if inventories[0] else None
    return dict(belt=inventories[0], saw=inventories[1], chute=inventories[2], chest=inventories[3],
                timer=timer, position=position)


def verify(warm, cold, running):
    assert warm[-1] == cold, (warm[-1], cold)
    assert cold == dict(belt={LOG: 2}, saw={LOG: 1}, chute={}, chest={},
                        timer=[50, 50, 0], position=2.75), cold
    for row in warm + running:
        assert sum(sum(row[key].values()) for key in ('belt', 'saw', 'chute', 'chest')) == 3, row
        assert sum(row['saw'].values()) <= 1, row
    assert running[-1]['chest'] == {STRIPPED: 3} and not running[-1]['belt'], running[-1]


def compare(report, reference):
    assert report['warm'] == reference['warm']
    assert report['cold'] == reference['cold']
    running = report['running']
    original = reference['running']
    handoffs = []
    for index, (actual, expected) in enumerate(zip(running, original, strict=True)):
        if actual == expected:
            continue
        previous = original[index-1] if index else reference['cold']
        following = original[index+1] if index+1 < len(original) else expected
        allowed = expected.copy()
        # On cold load the engines can visit the Belt, Saw and Chute in
        # different orders. Cutting and export must still happen at the
        # same time, even if the Belt refills the Saw on the export tick.
        if STRIPPED in previous['saw'] and not expected['saw'] and LOG in following['saw']:
            for key in ('belt', 'saw', 'position'):
                allowed[key] = following[key]
            duration = following['timer'][1]
            allowed['timer'] = [duration, duration, 0]
        # The Chute may reach the Chest one tick later.
        if previous['chest'] != expected['chest']:
            for key in ('chute', 'chest'):
                allowed[key] = previous[key]
        assert actual == allowed, (index+1, actual, expected)
        handoffs.append(index+1)
    return handoffs


def java(work):
    host = Original()
    state = host.call('state', {})
    if state.get('screen') == 'PauseScreen':
        host.call('input', {'kind': 'escape'})
        state = host.call('state', {})
    assert state['ready'] and state['data_version'] == 3955

    def advance(ticks):
        return host.call('observe_ticks', {'positions': PROBES, 'ticks': ticks})['samples']

    def sample(ticks):
        return [snapshot([block['nbt'] for block in row['blocks']]) for row in advance(ticks)]

    host.call('tick', {'frozen': True})
    try:
        host.command(f'forceload add {FORCE}')
        host.command(f'tp @s {X+3}.5 130 {Z+1}.5')
        advance(23)
        setup(host.command, advance)
        warm = sample(WARM)
        host.call('quit', {})
        deadline = time.monotonic() + 60
        while reference_java.process_alive(reference_java.BASE):
            if time.monotonic() >= deadline:
                raise RuntimeError('The reference Java world did not quit')
            time.sleep(.2)
        assert reference_java.launch(reference_java.BASE)['ready']
        host.call('tick', {'frozen': True})
        for _ in range(10):
            blocks = [host.call('block', {'pos': pos}) for pos in PROBES]
            if all(block['id'] != 'minecraft:air' for block in blocks):
                break
            advance(5)
        else:
            raise RuntimeError('Saved Saw fixture did not reload')
        cold = snapshot([block['nbt'] for block in blocks])
        start_saw(host.command)
        running = sample(60) + sample(RUN - 60)
        return warm, cold, running
    finally:
        host.command(f'fill {REGION} minecraft:air')
        host.command(f'forceload remove {FORCE}')
        host.call('camera', {key: state[key] for key in ('position', 'yaw', 'pitch')})
        host.call('tick', {'frozen': state['server']['frozen']})
        (work / 'transcript.json').write_text(json.dumps(host.trace, indent=2) + '\n')


def native(session):
    def probe(batch):
        for pos in PROBES:
            batch.data(pos)

    def rows(batch, label):
        raw = batch.run(session, label)
        return [snapshot(raw[i:i+len(PROBES)]) for i in range(0, len(raw), len(PROBES))]

    first = Batch()
    first.load((X+3, 129, Z+1))
    setup(first.command, first.tick)
    for _ in range(WARM):
        first.tick()
        probe(first)
    warm = rows(first, 'before-restart')

    resumed = Batch()
    resumed.load((X+3, 129, Z+1))
    probe(resumed)
    start_saw(resumed.command)
    for _ in range(RUN):
        resumed.tick()
        probe(resumed)
    cold, *running = rows(resumed, 'after-restart')
    return warm, cold, running


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ('game', 'resources', 'ctl', 'reference'):
        parser.add_argument('--' + key, type=Path)
    args = parser.parse_args()
    if args.game:
        if not all((args.resources, args.ctl, args.reference)):
            parser.error('Horizon requires --game, --resources, --ctl and --reference')
        reference = json.loads(args.reference.read_text())
        assert reference['passed'] and reference['create'] == PINS['create_revision']
        session = NativeCheckSession(args.game, args.resources, args.ctl, None, graphics=False,
                                     name='belt-saw-native', world='BeltSaw')
        work = session.scratch
        try:
            warm, cold, running = native(session)
        finally:
            session.control_temp.cleanup()
            for folder in (session.resources, work / '.horizon-cache', work / 'horizon-mods'):
                shutil.rmtree(folder, ignore_errors=True)
    else:
        work = Path(tempfile.mkdtemp(prefix='belt-saw-java-', dir=ROOT / '.local'))
        warm, cold, running = java(work)
    path = work / 'result.json'
    report = dict(passed=False, create=PINS['create_revision'], warm=warm, cold=cold, running=running)
    path.write_text(json.dumps(report, indent=2) + '\n')
    verify(warm, cold, running)
    if args.game:
        report['one_tick_handoffs'] = compare(report, reference)
    report['passed'] = True
    path.write_text(json.dumps(report, indent=2) + '\n')
    print(path)


if __name__ == '__main__':
    main()
