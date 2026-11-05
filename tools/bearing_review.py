#!/usr/bin/env python3
# Copyright (c) 2026 create-horizon contributors
# SPDX-License-Identifier: MIT
"""Build and verify a private Horizon Mechanical Bearing review world."""
import argparse
import json
from pathlib import Path
import shlex

from bearing_native_checks import Batch
from burner_heat_java_checks import number
from chain_drive_checks import xyz
import contraption_review_stations as stations
from native_check_session import NativeCheckSession, wait_loaded


NAME = 'BearingReview'
WORLD = 'CreateBearingReview'
BEARING = (904, 84, 56)
CONTROL = (905, 83, 56)
WINDMILL = (904, 84, 72)
SAIL_PANEL = [
    ((905, 84, 72), 'sail_frame'), ((905, 83, 72), 'sail_frame'),
    ((905, 85, 72), 'white_sail'), ((905, 84, 71), 'yellow_sail'),
    ((905, 84, 73), 'white_sail'), ((905, 85, 73), 'cyan_sail'),
    ((905, 83, 71), 'blue_sail'), ((905, 85, 71), 'black_sail'),
]
AXIAL_SAIL = (906, 84, 72)


def prepare(session, windmill, farms, clockwork, linear, cargo, sails):
    batch = Batch()
    batch.load((908, 82, 60))
    batch.command('fill 897 81 49 925 94 78 minecraft:air')
    batch.command('fill 897 80 49 925 80 78 minecraft:smooth_stone')
    batch.command('time set 7250')
    batch.command('weather clear')
    batch.put((904, 82, 56), 'create:creative_motor[facing=up]{ScrollValue:8}')
    batch.put((904, 83, 56), 'create:clutch[axis=y]')
    batch.put(CONTROL, 'minecraft:lever[face=wall,facing=east]')
    batch.put(BEARING, 'create:mechanical_bearing[facing=up]')
    batch.put((904, 85, 56), 'minecraft:slime_block')
    batch.put((905, 85, 56), 'minecraft:oak_log[axis=x]')
    batch.put((903, 85, 56), 'minecraft:oak_planks')
    batch.put((904, 85, 55), 'minecraft:oak_planks')
    batch.put((916, 85, 54), 'create:creative_motor[facing=south]{ScrollValue:8}')
    batch.put((916, 85, 55), 'create:mechanical_bearing[facing=south]')
    batch.put((916, 85, 56), 'minecraft:slime_block')
    batch.put((917, 85, 56), 'minecraft:oak_log[axis=x]')
    batch.put((915, 85, 56), 'minecraft:oak_planks')
    if windmill and sails:
        batch.put(WINDMILL, 'create:windmill_bearing[facing=east]')
        batch.put((903, 84, 72), 'create:shaft[axis=x]')
        for pos, block in SAIL_PANEL:
            batch.put(pos, f'create:{block}[facing=east]')
        batch.put(AXIAL_SAIL, 'create:orange_sail[facing=east]')
    elif windmill:
        batch.put(WINDMILL, 'create:windmill_bearing[facing=up]')
        batch.put((904, 83, 72), 'create:shaft[axis=y]')
        for x in range(902, 907):
            batch.put((x, 85, 72), 'minecraft:slime_block')
            batch.put((x, 85, 71), 'minecraft:white_wool')
            batch.put((x, 85, 73), 'minecraft:light_blue_wool')
    stations.sign(batch, (908, 81, 56), ['MECHANICAL', 'Flip the lever.', 'Ride the rotor.', 'Stand on top.'])
    if windmill:
        stations.sign(batch, (907, 81, 74), ['WINDMILL', 'Empty hand:', 'start / stop.', 'Rear shaft: 1 RPM.'])
    if farms:
        stations.setup(batch, clockwork, linear, cargo)
    batch.tick(30 if farms else 12)
    batch.data(BEARING)
    rows = batch.run(session, 'prepare-review')
    assert number({'nbt': rows[0]}, 'Running') == 1, rows


def review(session, windmill, farms, clockwork, linear, cargo, sails):
    call = session.call

    def command(value):
        return call('command', value)

    def snapshot(pos):
        return {'nbt': ' '.join(command('data get block ' + xyz(pos))['chat'])}

    def interact(pos, face):
        call('look-at', 'block', *pos, face)
        call('wait', 4)
        call('looking-at')
        call('mouse', 'right', 'click')
        call('wait', 6)

    def wait_running(pos, running):
        for _ in range(16):
            data = snapshot(pos)
            if number(data, 'Running', 0) == running:
                return data
            call('wait', 5)
        raise AssertionError((pos, running, data))

    def wait_block(pos, block):
        for _ in range(16):
            state = call('block', *pos)
            if state['block'] == block:
                return state
            call('wait', 5)
        raise AssertionError((pos, block, state))

    def screenshot(name):
        path = session.scratch / name
        call('screenshot', path)
        assert path.stat().st_size > 1024, path
        return path

    with session:
        wait_loaded(call, [BEARING, CONTROL, (916, 85, 56)])
        assert number(snapshot(BEARING), 'Running') == 1
        command('gamemode survival')
        command('tp @s 904.5 88 56.5')
        call('wait', 35)
        supported = call('state')['player']['position']
        assert abs(supported[1] - 86) < 0.06, supported
        command('tp @s 904.5 88 60.5')
        call('wait', 35)
        control = call('state')['player']['position']
        assert abs(control[1] - 81) < 0.06, control
        command('gamemode creative')
        command('tp @s 908.5 81 56.5')
        call('wait', 6)
        interact(CONTROL, 'east')
        assert number(snapshot(BEARING), 'Running') == 0
        interact(CONTROL, 'east')
        assert number(snapshot(BEARING), 'Running') == 1

        if windmill:
            command('tp @s 904.5 81 75.5')
            call('wait', 6)
            wait_loaded(call, [WINDMILL])
            command('clear @s')
            call('select', 9)
            call('inventory')
            interact(WINDMILL, 'south')
            windmill_nbt = wait_running(WINDMILL, 1)
            assert number(windmill_nbt, 'LastGenerated') == 1, windmill_nbt
            shaft = snapshot((903, 84, 72) if sails else (904, 83, 72))
            assert number(shaft, 'Speed') == 1, shaft
            interact(WINDMILL, 'south')
            wait_running(WINDMILL, 0)
            if sails:
                for pos, block in SAIL_PANEL:
                    state = call('block', *pos)
                    assert state['block'] == 'create:' + block and 'facing=east' in state['state'], state
                assert call('block', *AXIAL_SAIL)['block'] == 'create:orange_sail'
            else:
                for x in range(902, 907):
                    state = call('block', x, 85, 71)
                    assert 'white_wool' in str(state), state
            interact(WINDMILL, 'south')
            wait_running(WINDMILL, 1)

        if farms:
            wait_loaded(call, [stations.PISTON, stations.STICKY_PISTON, stations.HARVESTER, stations.SAW])
            for base, sign, block in ((stations.PISTON, (914, 81, 68), 'create:mechanical_piston'),
                                      (stations.STICKY_PISTON, (914, 81, 75), 'create:sticky_mechanical_piston')):
                state = call('block', *base)
                assert state['block'] == block and 'state=extended' in state['state'], state
                assert call('block', base[0] + 4, base[1], base[2])['block'] == 'minecraft:oak_planks'
                assert call('block', *sign)['block'] == 'minecraft:oak_sign'
            for base, lever in ((stations.HARVESTER, stations.HARVESTER_LEVER), (stations.SAW, stations.SAW_LEVER)):
                assert number(snapshot(base), 'Running', 0) == 0, base
                assert 'powered=true' in call('block', *lever)['state']
            if cargo:
                assert call('block', *stations.FARM_CHEST)['block'] == 'minecraft:chest'
                assert number(snapshot(stations.CARGO), 'Running', 0) == 1
            if clockwork:
                wait_loaded(call, [stations.CLOCK])
                assert call('block', *stations.CLOCK)['block'] == 'create:clockwork_bearing'
                assert number(snapshot(stations.CLOCK), 'Running', 0) == 1
            for pos in stations.SAW_TREE:
                assert call('block', *pos)['block'] == 'minecraft:oak_log'
        if linear:
            command('tp @s 911.5 83 102.5')
            wait_loaded(call, [stations.PULLEY, stations.GANTRY, stations.STATIONARY_SAW])
            assert call('block', *stations.PULLEY)['block'] == 'create:rope_pulley'
            assert call('block', 904, 82, 103)['block'] == 'minecraft:oak_log'
            assert call('block', *stations.GANTRY)['block'] == 'create:gantry_carriage'
            assert call('block', 918, 84, 103)['block'] == 'minecraft:oak_planks'
            assert call('block', *stations.STATIONARY_SAW)['block'] == 'create:mechanical_saw'
            assert call('block', 920, 82, 110)['block'] == 'minecraft:oak_log'
            command('tp @s 920.5 81 107.5')
            interact(stations.GANTRY_LEVER, 'south')
            wait_block((920, 83, 103), 'create:gantry_carriage')
            wait_block((920, 84, 103), 'minecraft:oak_planks')
            command('tp @s 917.5 81 107.5')
            interact((917, 82, 104), 'south')
            wait_block(stations.GANTRY, 'create:gantry_carriage')
            wait_block((918, 84, 103), 'minecraft:oak_planks')
            command('tp @s 920.5 81 107.5')
            interact(stations.GANTRY_LEVER, 'south')
            assert 'powered=true' in call('block', *stations.GANTRY_LEVER)['state']
            command('tp @s 917.5 81 107.5')
            interact((917, 82, 104), 'south')
            assert 'powered=false' in call('block', 917, 82, 104)['state']
        command('gamemode spectator')
        command('tp @s 910.5 88 62.5')
        call('look-at', 904.5, 85.1, 56.5)
        call('key', 'f1', 'tap')
        call('wait', 8)
        first = screenshot('bearing-first.png')
        call('wait', 15)
        second = screenshot('bearing-second.png')
        assert first.read_bytes() != second.read_bytes()
        extra = []
        if cargo:
            command('tp @s 908.5 87.2 65.5')
            call('look-at', 904.5, 85.5, 62.5)
            call('wait', 12)
            extra.append(str(screenshot('mounted-chest.png')))
        if sails:
            command('tp @s 910.5 86.2 75.5')
            call('look-at', 905.5, 84.5, 72.5)
            call('wait', 12)
            extra.append(str(screenshot('colored-sails.png')))
        call('key', 'f1', 'tap')
        command('gamemode creative')
        command('tp @s 909.5 81 61.5')
        call('look-at', 904.5, 85.1, 56.5)
        command('save-all flush')

    return dict(supported=supported, control=control, screenshots=[str(first), str(second), *extra])


def launcher(session):
    folder = session.scratch
    command = [session.game, '--resources', session.resources, '--home', folder / 'home',
               '--world', WORLD, '--name', NAME, '--creative', '--width', '1440',
               '--height', '900', '--render-distance', '4']
    path = folder / 'Open Create Contraptions Review.command'
    path.write_text('#!/bin/zsh\ncd ' + shlex.quote(str(folder)) +
                    ' || exit 1\nexec ' + shlex.join(map(str, command)) + '\n')
    path.chmod(0o755)
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ('game', 'resources', 'ctl'):
        parser.add_argument('--' + flag, type=Path, required=True)
    parser.add_argument('--create-jar', type=Path)
    parser.add_argument('--windmill', action='store_true')
    parser.add_argument('--farms', action='store_true')
    parser.add_argument('--clockwork', action='store_true')
    parser.add_argument('--linear', action='store_true')
    parser.add_argument('--cargo', action='store_true')
    parser.add_argument('--sails', action='store_true')
    args = parser.parse_args()
    if args.sails and not args.windmill:
        parser.error('--sails requires --windmill')
    if (args.clockwork or args.linear or args.cargo) and not args.farms:
        parser.error('--clockwork, --linear and --cargo require --farms')
    session = NativeCheckSession(args.game, args.resources, args.ctl, args.create_jar,
                                 name='bearing-review', graphics=True, world=WORLD, username=NAME)
    print(session.scratch, flush=True)
    prepare(session, args.windmill, args.farms, args.clockwork, args.linear, args.cargo, args.sails)
    report = review(session, args.windmill, args.farms, args.clockwork, args.linear, args.cargo, args.sails)
    report['launcher'] = str(launcher(session))
    (session.scratch / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
