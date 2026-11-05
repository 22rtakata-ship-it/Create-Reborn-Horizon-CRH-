#!/usr/bin/env python3
# Copyright (c) 2026 create-horizon contributors
# SPDX-License-Identifier: MIT
"""Observe a Mechanical Piston pushing the local Java client player."""
import json
from pathlib import Path
import tempfile

from common import PINS, ROOT
from chain_drive_checks import Original
from burner_heat_java_checks import number

def main():
    host = Original()
    host.call('tick', {'frozen': True})
    host.command('gamemode creative')
    host.command('tp @s 905.45 100 102.5')
    host.command('forceload add 896 96 911 111')
    pivot = (903, 100, 102)

    def step(n, commands=()):
        return host.call('observe_ticks', {'ticks': n, 'positions': [pivot], 'commands': commands})['samples'][-1]['blocks'][0]

    def pos():
        return host.call('state', {})['position']

    step(35)
    for command in ('fill 898 100 98 910 107 107 minecraft:air',
                    'fill 898 99 98 910 99 107 minecraft:stone',
                    'setblock 903 100 102 create:mechanical_piston[facing=east,axis_along_first=false]',
                    *(f'setblock {x} 100 102 create:piston_extension_pole[facing=east]' for x in (900, 901, 902)),
                    'setblock 904 100 102 minecraft:stone',
                    'gamemode creative'):
        host.command(command)
    host.call('camera', {'position': [905.45, 100, 102.5], 'target': [904.5, 100.5, 102.5]})
    step(8)
    before = pos()
    moving = step(20, ['setblock 903 100 101 create:creative_motor[facing=south]{ScrollValue:-32}'])
    after = pos()
    stopped = step(40, ['fill 907 101 101 907 104 103 minecraft:stone'])
    wall = pos()
    host.command('gamemode creative')
    assert after[0] > before[0] + 0.45
    assert abs(after[1] - before[1]) < 1e-6 and abs(after[2] - before[2]) < 1e-6
    report = dict(create=PINS['create_revision'], before=before, after=after, wall=wall,
                  offset=number(moving, 'Offset', 0), final_offset=number(stopped, 'Offset', 0))
    work = Path(tempfile.mkdtemp(prefix='side-contact-reference-', dir=ROOT / '.local'))
    path = work / 'result.json'
    path.write_text(json.dumps(report, indent=2) + '\n')
    print(path)


if __name__ == '__main__':
    main()
