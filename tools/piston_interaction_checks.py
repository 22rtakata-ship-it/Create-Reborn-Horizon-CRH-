#!/usr/bin/env python3
# Copyright (c) 2026 create-horizon contributors
# SPDX-License-Identifier: MIT
"""Check survival slime application and wrench actions through the Horizon client."""
import argparse
import json
from pathlib import Path

from bearing_native_checks import Batch
from native_check_session import NativeCheckSession, wait_loaded

PISTON = (935, 100, 88)
EXTENDED = (939, 100, 88)
CREATIVE = (935, 100, 91)


def review(session):
    batch = Batch()
    batch.load((936, 101, 89))
    batch.command('fill 929 100 83 942 106 95 minecraft:air')
    batch.command('fill 929 99 83 942 99 95 minecraft:stone')
    batch.put(PISTON, 'create:mechanical_piston[facing=south,axis_along_first=false]')
    batch.put(EXTENDED, 'create:mechanical_piston[facing=south,axis_along_first=false,state=extended]')
    batch.put(CREATIVE, 'create:mechanical_piston[facing=east,axis_along_first=true]')
    batch.run(session, 'piston-hand-actions')
    with session:
        call = session.call
        command = lambda line: call('command', line)

        def click(pos, face, vantage):
            command('tp @s ' + ' '.join(map(str, vantage)))
            call('wait', 5)
            call('look-at', 'block', *pos, face)
            looking = call('looking-at')['block']
            assert looking['pos'] == list(pos) and looking['face'] == face, looking
            call('mouse', 'right', 'click')
            call('wait', 8)

        def slime_count():
            return sum(slot.get('count', 0) for slot in call('inventory')['hotbar']
                       if slot.get('item') == 'minecraft:slime_ball')

        def expect(pos, name, **properties):
            state = call('block', *pos)
            assert state['block'] == 'create:' + name, state
            assert all(f'{key}={value}' in state['state'] for key, value in properties.items()), state

        wait_loaded(call, [PISTON, EXTENDED, CREATIVE])
        command('gamemode survival')
        command('clear @s')
        command('give @s minecraft:slime_ball 3')
        call('select', 1)
        assert slime_count() == 3
        click(PISTON, 'west', (932.5, 100, 88.5))
        expect(PISTON, 'mechanical_piston', facing='south', axis_along_first='false')
        assert slime_count() == 3
        click(PISTON, 'south', (935.5, 100, 90.5))
        expect(PISTON, 'sticky_mechanical_piston', facing='south', axis_along_first='false')
        assert slime_count() == 2
        click(EXTENDED, 'south', (939.5, 100, 90.5))
        expect(EXTENDED, 'mechanical_piston', state='extended')
        assert slime_count() == 2
        command('gamemode creative')
        click(CREATIVE, 'east', (938.5, 100, 91.5))
        expect(CREATIVE, 'sticky_mechanical_piston', facing='east', axis_along_first='true')
        assert slime_count() == 2
        command('clear @s')
        command('give @s create:wrench')
        call('select', 1)
        click(PISTON, 'south', (935.5, 100, 90.5))
        expect(PISTON, 'sticky_mechanical_piston', facing='south', axis_along_first='true')
        click(EXTENDED, 'west', (937.5, 100, 88.5))
        expect(EXTENDED, 'mechanical_piston', state='extended', facing='south', axis_along_first='false')
        assert call('inventory')['hotbar'][0]['item'] == 'create:wrench'
        assert 'the integrated server panicked' not in session.log.read_text()
        command('clear @s')
        call('select', 9)
        base, first, second, head, rear = [(x, 100, 90) for x in (938, 939, 940, 941, 937)]
        for pos, block in [
            (base, 'mechanical_piston[facing=east,state=extended]{Offset:3f}'),
            (first, 'piston_extension_pole[facing=east]'),
            (second, 'piston_extension_pole[facing=east]'),
            (head, 'mechanical_piston_head[facing=east,type=normal]'),
            (rear, 'piston_extension_pole[facing=east]'),
        ]:
            command('setblock ' + ' '.join(map(str, pos)) + ' create:' + block)
        command('tp @s 939.5 100 92.5')
        call('wait', 4)
        call('look-at', 'block', *first, 'south')
        target = call('looking-at')['block']
        assert target['pos'] == list(first), target
        call('mouse', 'left', 'click')
        call('wait', 10)
        assert call('block', *first)['block'] == 'minecraft:air', 'Player did not break the pole'
        expect(base, 'mechanical_piston', state='retracted', facing='east')
        for pos in (first, second, head):
            assert call('block', *pos)['block'] == 'minecraft:air', pos
        expect(rear, 'piston_extension_pole', facing='east')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ('game', 'resources', 'ctl'):
        parser.add_argument('--' + flag, type=Path, required=True)
    args = parser.parse_args()
    session = NativeCheckSession(args.game, args.resources, args.ctl, None,
                                 name='piston-hand-actions', graphics=True,
                                 world='CreatePistonInteractionChecks', username='BearingReview')
    print(session.scratch, flush=True)
    try:
        review(session)
    finally:
        session.control_temp.cleanup()
    print(json.dumps(dict(passed=True, world=str(session.scratch))))


if __name__ == '__main__':
    main()
