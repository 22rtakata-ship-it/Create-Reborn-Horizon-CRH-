"""Phase-1 scenarios for a real isolated Horizon client, never a mock world.

Coordinates straddle X chunks 128/129, far from spawn's retained chunks.
Control operations are confirmed in the pinned Horizon agent/control source.
"""
import math
import re

BASE = 2062
WHEEL = (BASE, 67, 0)
SHAFT = (BASE + 1, 67, 0)
PRESS = (BASE + 3, 68, 1)
DEPOT = (BASE + 3, 66, 1)


def number(call, pos, key):
    text = ' '.join(call('command', 'data get block ' + ' '.join(map(str, pos)))['chat'])
    found = re.search(r'\b' + re.escape(key) + r'"?\s*:\s*(-?[\d.]+(?:[eE][+-]?\d+)?)', text)
    if not found:
        raise AssertionError((pos, key, text))
    return float(found[1])


def counts(call):
    result = {}
    for stack in call('container', *DEPOT)['slots']:
        if stack:
            result[stack['item']] = result.get(stack['item'], 0) + stack['count']
    return result


def wait_output(call, item, count):
    for _ in range(120):
        actual = counts(call)
        if actual == {item: count}:
            return actual
        call('wait', 20)
    raise AssertionError(('output timeout', item, count, actual))


def run(call, graphics, log_path, scratch):
    evidence = {}
    def put(x, y, z, state):
        return call('setblock', BASE+x, y, z, state)
    def speed(pos, expected):
        actual = number(call, pos, 'Speed')
        if not math.isclose(actual, expected, abs_tol=0.0001):
            raise AssertionError((pos, expected, actual))
    call('command', f'tp @s {BASE+3.5} 73 5.5')
    call('wait-ready', 120, timeout=150)
    call('fill', BASE-3, 64, -3, BASE+8, 73, 6, 'minecraft:air')
    call('fill', BASE-3, 63, -3, BASE+8, 63, 6, 'minecraft:smooth_stone')
    for x,y,z,state in [
        (0,67,0,'create:water_wheel[facing=east]'),
        (1,67,0,'create:shaft[axis=x]'),
        (2,67,0,'create:large_cogwheel[axis=x]'),
        (2,68,1,'create:cogwheel[axis=x]'),
        (3,68,1,'create:mechanical_press[facing=east]'),
        (3,66,1,'create:depot'),
    ]:
        put(x,y,z,state)
    # Same real bubble-current layout as inherited engineering_checks.py.
    for y in range(66,71):
        for x,z in [(-1,1),(1,1),(0,2),(0,0)]:
            if call('block',BASE+x,y,z)['block']=='minecraft:air':
                put(x,y,z,'minecraft:glass')
    put(0,65,1,'minecraft:soul_sand')
    for y in range(66,70):
        put(0,y,1,'minecraft:water')
    call('wait',60)
    rpm=number(call,WHEEL,'Speed')
    if abs(rpm)!=8:
        raise AssertionError(('wheel rpm',rpm))
    speed(SHAFT,rpm)
    speed(PRESS,-2*rpm)
    evidence['wheel_and_gears']={'wheel_rpm':rpm,'press_rpm':-2*rpm}
    put(0,65,1,'minecraft:magma_block');call('wait',50)
    speed(SHAFT,-rpm);speed(PRESS,2*rpm)
    put(0,65,1,'minecraft:soul_sand');call('wait',50)
    speed(PRESS,-2*rpm)
    evidence['flow_reversal']=True
    call('container',*DEPOT,'minecraft:iron_ingot',2)
    evidence['pressing']=wait_output(call,'create:iron_sheet',2)
    # Original water wheel: 32 SU/RPM * 8 RPM = 256 SU. Three presses
    # at 16 RPM consume 3 * 8 SU/RPM * 16 = 384 SU, exceeding capacity.
    put(4,68,1,'create:mechanical_press[facing=east]')
    put(5,68,1,'create:mechanical_press[facing=east]');call('wait',30)
    speed(PRESS,0)
    capacity=number(call,PRESS,'Capacity');stress=number(call,PRESS,'Stress')
    if capacity!=256 or stress!=384:
        raise AssertionError(('stress accounting',capacity,stress))
    put(5,68,1,'minecraft:air');put(4,68,1,'minecraft:air');call('wait',30)
    speed(PRESS,-2*rpm)
    evidence['overload_recovery']={'capacity':capacity,'stress':stress}
    put(1,67,0,'create:clutch[axis=x]');call('wait',20)
    put(1,67,-1,'minecraft:redstone_block');call('wait',20);speed(PRESS,0)
    put(1,67,-1,'minecraft:air');call('wait',30);speed(PRESS,-2*rpm)
    put(1,67,0,'create:shaft[axis=x]');call('wait',20)
    evidence['clutch']=True
    # Replace only the fixture depot to give each processing scenario a clean inventory.
    put(3,66,1,'minecraft:air');put(3,66,1,'create:depot')
    call('command','kill @e[type=minecraft:item]')
    put(1,67,0,'minecraft:air')
    call('container',*DEPOT,'minecraft:copper_ingot',2);call('wait',180)
    if counts(call)!={'minecraft:copper_ingot':2}:
        raise AssertionError(('unpowered inventory',counts(call)))
    put(1,67,0,'create:shaft[axis=x]');call('wait',20)
    evidence['power_loss']=wait_output(call,'create:copper_sheet',2)
    if graphics:
        call('command',f'tp @s {BASE+4.5} 70 5.5')
        call('look-at',WHEEL[0]+.5,WHEEL[1]+.5,WHEEL[2]+.5)
        call('wait',40)
        call('screenshot',scratch/'core-factory.png')
        # Submitted matrices must change with time, not merely show a static model.
        from engineering_checks import poses
        samples=poses(log_path,list(WHEEL))
        moving=[row for row in samples if row['model'].endswith('/water_wheel')]
        if len({tuple(row['matrix']) for row in moving})<2:
            raise AssertionError(('missing wheel animation samples',moving[-2:]))
        evidence['rotational_render_samples']=len(moving)
    # Verify actual resident-column eviction using the documented block.loaded flag.
    call('command',f'tp @s {BASE+4096.5} 80 4096.5')
    call('wait-ready',120,timeout=150)
    for _ in range(120):
        loaded=[call('block',*p)['loaded'] for p in [SHAFT,PRESS]]
        if not any(loaded):
            break
        call('wait',20)
    else:
        raise AssertionError(('factory columns did not unload',loaded))
    call('command',f'tp @s {BASE+3.5} 73 5.5')
    call('wait-ready',120,timeout=150);call('wait',60)
    if not all(call('block',*p)['loaded'] for p in [SHAFT,PRESS]):
        raise AssertionError('factory columns did not reload')
    speed(PRESS,-2*rpm)
    if counts(call)!={'create:copper_sheet':2}:
        raise AssertionError(('inventory after chunk reload',counts(call)))
    evidence['chunk_unload_reload']=True
    # Observe an actual running cycle before the harness closes/reopens this world.
    put(3,66,1,'minecraft:air');put(3,66,1,'create:depot')
    call('command','kill @e[type=minecraft:item]')
    call('container',*DEPOT,'minecraft:gold_ingot',2)
    for _ in range(60):
        ticks=number(call,PRESS,'Ticks')
        if number(call,PRESS,'Running')==1 and 0<ticks<120:
            break
        call('wait',1)
    else:
        raise AssertionError('could not observe a running press before restart')
    evidence['pre_restart_ticks']=ticks
    evidence['rpm']=rpm
    return evidence


def reloaded(call, fixture):
    call('command',f'tp @s {BASE+3.5} 73 5.5')
    call('wait-ready',120,timeout=150);call('wait',40)
    if number(call,PRESS,'Speed')!=-2*fixture['rpm']:
        raise AssertionError('network did not recover after restart')
    fixture['restart_output']=wait_output(call,'create:golden_sheet',2)
