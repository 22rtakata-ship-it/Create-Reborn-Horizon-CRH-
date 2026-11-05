# Copyright (c) 2026 create-horizon contributors
# SPDX-License-Identifier: MIT
"""Build the actuator and farm stations in the disposable contraption world."""
import json

from chain_drive_checks import xyz

PISTON = (919, 83, 68)
PISTON_LEVER = (918, 83, 67)
STICKY_PISTON = (919, 83, 75)
STICKY_LEVER = (918, 83, 74)
HARVESTER = (904, 82, 88)
HARVESTER_LEVER = (905, 81, 88)
SAW = (919, 82, 88)
SAW_LEVER = (920, 81, 88)
CLOCK = (919, 85, 70)
PULLEY = (904, 85, 103)
PULLEY_CLUTCH = (903, 85, 104)
GANTRY = (918, 83, 103)
GANTRY_LEVER = (920, 82, 104)
STATIONARY_SAW = (919, 82, 110)
STATIONARY_SAW_LEVER = (918, 82, 111)
SAW_TREE = ((920, 83, 87), (920, 84, 87), (920, 85, 87))


def sign(batch, pos, lines, rotation=0):
    batch.put(pos, f'minecraft:oak_sign[rotation={rotation}]')
    messages = json.dumps(lines, ensure_ascii=False)
    batch.command(f'data merge block {xyz(pos)} {{front_text:{{messages:{messages},color:"black"}},is_waxed:1b}}')


def piston(batch, pos, sticky):
    x, y, z = pos
    kind = 'sticky_mechanical_piston' if sticky else 'mechanical_piston'
    batch.put(pos, f'create:{kind}[facing=east,axis_along_first=false,state=retracted]')
    for i in (-3, -2, -1):
        batch.put((x + i, y, z), 'create:piston_extension_pole[facing=east]')
    batch.put((x + 1, y, z), 'minecraft:oak_planks')
    batch.put((x, y, z - 1), 'create:gearshift[axis=z]')
    batch.put((x - 1, y, z - 1), 'minecraft:lever[face=wall,facing=west]')
    batch.put((x, y, z - 2), 'create:creative_motor[facing=south]{ScrollValue:-64}')


def farm_drive(batch, pos):
    x, y, z = pos
    batch.put(pos, 'create:mechanical_bearing[facing=up]')
    batch.put((x, y - 1, z), 'create:clutch[axis=y]')
    batch.put((x + 1, y - 1, z), 'minecraft:lever[face=wall,facing=east,powered=true]')
    batch.put((x, y - 2, z), 'create:creative_motor[facing=up]{ScrollValue:32}')


def setup(batch, clockwork=False, linear=False):
    batch.load((911, 82, 82))
    batch.command('fill 897 81 79 925 90 94 minecraft:air')
    batch.command('fill 897 80 79 925 80 94 minecraft:smooth_stone')
    batch.command('gamerule minecraft:random_tick_speed 0')
    piston(batch, PISTON, False)
    piston(batch, STICKY_PISTON, True)
    sign(batch, (914, 81, 68), ['PISTON', 'Lever reverses', 'the stroke.', 'Pushes only.'], 4)
    sign(batch, (914, 81, 75), ['STICKY PISTON', 'Lever reverses', 'the stroke.', 'Pulls the block.'], 4)
    farm_drive(batch, HARVESTER)
    hx, hy, hz = HARVESTER
    for i in range(3):
        batch.put((hx + i, hy + 1, hz), 'minecraft:slime_block')
    batch.put((hx + 3, hy + 1, hz), 'create:mechanical_harvester[facing=north]')
    for dx, dz, crop, age in [(3, -1, 'wheat', 7), (2, -2, 'carrots', 7),
                             (1, -3, 'potatoes', 7), (0, -3, 'beetroots', 3),
                             (-1, -3, 'wheat', 3)]:
        batch.put((hx + dx, hy, hz + dz), 'minecraft:farmland[moisture=7]')
        batch.put((hx + dx, hy + 1, hz + dz), f'minecraft:{crop}[age={age}]')
    sign(batch, (906, 81, 92), ['HARVESTER', 'Release clutch.', 'Watch ripe crops', 'get replanted.'])
    farm_drive(batch, SAW)
    sx, sy, sz = SAW
    batch.put((sx, sy + 1, sz), 'minecraft:slime_block')
    batch.put((sx + 1, sy + 1, sz), 'create:mechanical_saw[facing=north,axis_along_first=false,flipped=false]')
    for pos in SAW_TREE:
        batch.put(pos, 'minecraft:oak_log[axis=y]')
    batch.put((921, 85, 87), 'minecraft:oak_leaves[distance=1,persistent=true]')
    batch.put((923, 84, 87), 'minecraft:oak_log[axis=y]')
    sign(batch, (921, 81, 92), ['MOVING SAW', 'Release clutch.', 'Cuts the tree', 'beside the saw.'])
    if clockwork:
        batch.put(CLOCK, 'create:clockwork_bearing[facing=east]')
        batch.put((920, 85, 70), 'minecraft:oak_log[axis=y]')
        batch.put((921, 85, 70), 'minecraft:stripped_oak_log[axis=z]')
        batch.put((918, 85, 70), 'create:creative_motor[facing=east]{ScrollValue:32}')
        sign(batch, (915, 81, 71), ['CLOCKWORK', 'Two moving hands.', 'Empty hand:', 'stop / start.'], 4)
    if linear:
        linear_stations(batch)
    batch.command('tp @s 909.5 81 61.5')

def linear_stations(batch):
    batch.load((911, 82, 102))
    batch.command('fill 897 81 95 925 94 114 minecraft:air')
    batch.command('fill 897 80 95 925 80 114 minecraft:smooth_stone')
    batch.put(PULLEY, 'create:rope_pulley[axis=x]')
    batch.put((904, 82, 103), 'minecraft:oak_log[axis=y]')
    batch.put((903, 85, 103), 'create:clutch[axis=x]')
    batch.put(PULLEY_CLUTCH, 'minecraft:lever[face=wall,facing=south,powered=true]')
    batch.put((902, 85, 103), 'create:gearshift[axis=x]')
    batch.put((902, 85, 104), 'minecraft:lever[face=wall,facing=south]')
    batch.put((901, 85, 103), 'create:creative_motor[facing=east]{ScrollValue:32}')
    sign(batch, (905, 81, 105), ['ROPE PULLEY', 'Right: clutch.', 'Left: reverse.', 'Picks up the log.'])
    for x, part in ((918, 'start'), (919, 'middle'), (920, 'end')):
        batch.put((x, 82, 103), f'create:gantry_shaft[facing=east,part={part},powered=true]')
    batch.put(GANTRY_LEVER, 'minecraft:lever[face=wall,facing=south,powered=true]')
    batch.put(GANTRY, 'create:gantry_carriage[facing=up,axis_along_first=false]')
    batch.put((918, 84, 103), 'minecraft:oak_planks')
    batch.put((917, 82, 103), 'create:gearshift[axis=x]')
    batch.put((917, 82, 104), 'minecraft:lever[face=wall,facing=south]')
    batch.put((916, 82, 103), 'create:creative_motor[facing=east]{ScrollValue:-32}')
    sign(batch, (919, 81, 106), ['GANTRY', 'Right: rail stop.', 'Left: reverse.', 'Carries the plank.'])
    batch.put(STATIONARY_SAW, 'create:mechanical_saw[facing=east]')
    batch.put((918, 82, 110), 'create:clutch[axis=x]')
    batch.put(STATIONARY_SAW_LEVER, 'minecraft:lever[face=wall,facing=south,powered=true]')
    batch.put((917, 82, 110), 'create:creative_motor[facing=east]{ScrollValue:64}')
    for y in range(82, 85):
        batch.put((920, y, 110), 'minecraft:oak_log[axis=y]')
    batch.put((921, 84, 110), 'minecraft:oak_leaves[persistent=true,distance=1]')
    sign(batch, (921, 81, 113), ['STATIONARY SAW', 'Release clutch.', 'Cuts the trunk', 'and canopy.'])
