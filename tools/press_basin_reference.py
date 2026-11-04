#!/usr/bin/env python3
"""Observe ordinary original Create press/Basin cycles in the private reference.

The caller exclusively leases the already-running disposable Java instance.
Canonical inventory seeds and a real motor replacement provide the inputs and
speed-change event. Samples are normal ServerTick.Post observations; no press
method, forced machine counter, schematic import or render comparison is used.
Creation order is recorded separately from internal block-entity tick order.
"""
import hashlib
import json
from pathlib import Path
import tempfile

from common import ROOT, PINS
from burner_heat_java_checks import items, number
import reference_java as java

BASIN = [580, 64, 30]
PRESS = [580, 66, 30]
MOTOR = [578, 66, 30]


def run():
    base = java.BASE.resolve()
    manifest = json.loads((base / 'manifest.json').read_text())
    assert manifest['data_version'] == 3955 and manifest['create_sha256'] == java.CREATE_SHA256
    work = Path(tempfile.mkdtemp(prefix='press-basin-java-', dir=ROOT / '.local'))
    report = {'passed': False, 'scope': __doc__, 'create': PINS['create_revision'],
              'fixture_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'cases': []}
    transcript = []

    def save():
        (work / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        (work / 'transcript.json').write_text(json.dumps(transcript, indent=2) + '\n')

    def call(method, body):
        reply = java.call(base, method, body)
        transcript.append({'method': method, 'body': body, 'reply': reply})
        save()
        return reply

    def command(value):
        return call('command', {'command': value})

    def observe(ticks):
        before = call('state', {})['server']['game_time']
        result = call('observe_ticks', {'ticks': ticks, 'positions': [BASIN, PRESS]})
        assert [s['game_time'] for s in result['samples']] == list(range(before + 1, before + ticks + 1))
        assert call('state', {})['server']['frozen']
        return result['samples']

    print(json.dumps({'work': str(work)}), flush=True)
    initial = call('state', {})
    if initial.get('screen') == 'PauseScreen':
        call('input', {'kind': 'escape'})
        initial = call('state', {})
    assert initial['ready'] and initial['data_version'] == 3955
    try:
        call('tick', {'frozen': True})
        command('gamemode creative')
        command('forceload add 576 16 591 47')
        command('fill 577 63 28 582 63 32 minecraft:smooth_stone')
        for item, count, output, amount in [
            ('minecraft:snowball', 8, 'minecraft:snow_block', 2),
            ('minecraft:snow_block', 9, 'minecraft:ice', 1),
        ]:
            for basin_first in [True, False]:
                row = {'input': {item: count}, 'expected': {output: amount},
                       'basin_created_first': basin_first}
                report['active_case'] = row
                command('fill 577 64 28 582 69 32 minecraft:air')
                actors = [('580 64 30', 'create:basin[facing=down]'),
                          # MechanicalPressBlock.getRotationAxis uses facing's
                          # axis; the original east/west ports face this shaft.
                          ('580 66 30', 'create:mechanical_press[facing=east]')]
                if not basin_first:
                    actors.reverse()
                for pos, block in actors:
                    command('setblock ' + pos + ' ' + block)
                command('setblock 579 66 30 create:shaft[axis=x]')
                command('setblock 578 66 30 create:creative_motor[facing=east]')
                command('data merge block 578 66 30 {ScrollValue:0}')
                row['settled'] = observe(30)[-1]
                assert number(row['settled']['blocks'][1], 'Speed') == 0
                command('data merge block 580 64 30 {InputItems:{Size:9,Items:[{Slot:0,id:"' +
                        item + '",count:' + str(count) + '}]},OutputItems:{Size:9,Items:[]}}')
                command('setblock 578 66 30 minecraft:air')
                command('setblock 578 66 30 create:creative_motor[facing=east]')
                command('data merge block 578 66 30 {ScrollValue:32}')
                row['samples'] = observe(180)
                for sample in row['samples']:
                    press = sample['blocks'][1]
                    assert abs(number(press, 'Speed')) in (0, 32)
                last = row['samples'][-1]
                assert abs(number(last['blocks'][1], 'Speed')) == 32
                assert items(last['blocks'][0]['nbt'], 'InputItems') == {}
                assert items(last['blocks'][0]['nbt'], 'OutputItems') == {output: amount}
                impacts = []
                previous = 0
                for sample in row['samples']:
                    current = items(sample['blocks'][0]['nbt'], 'OutputItems').get(output, 0)
                    if current != previous:
                        impacts.append({'tick': sample['tick'], 'time': sample['game_time'],
                                        'amount': current, 'press': sample['blocks'][1]})
                    previous = current
                assert len(impacts) == amount, impacts
                row['impacts'] = impacts
                report['cases'].append(row)
                report.pop('active_case', None)
                save()
                print(json.dumps({'input': item, 'basin_first': basin_first, 'impacts':
                                  [s['tick'] for s in impacts]}), flush=True)
        report['passed'] = True
        return report
    except BaseException as error:
        report['error'] = repr(error)
        raise
    finally:
        call('tick', {'frozen': True})
        call('camera', {'position': initial['position'], 'yaw': initial['yaw'], 'pitch': initial['pitch']})
        call('tick', {'frozen': initial['server']['frozen']})
        save()
        print(json.dumps({'passed': report['passed'], 'result': str(work / 'result.json')}), flush=True)


if __name__ == '__main__':
    run()
