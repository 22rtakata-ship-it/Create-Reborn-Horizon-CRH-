"""Bind the port's model surfaces to its independent fallback artwork."""

from pathlib import PurePosixPath


FAMILIES = {
    'adjustable_chain_gearshift': 'chain_drive',
    'andesite_alloy_block': 'alloy_block',
    'andesite_casing': 'andesite_casing',
    'andesite_encased_shaft': 'andesite_casing',
    'basin': 'basin',
    'belt': 'belt_rubber',
    'belt_pulley': 'steel_axle',
    'brass_block': 'brass_block',
    'brass_casing': 'brass_casing',
    'brass_encased_shaft': 'brass_casing',
    'brass_funnel': 'brass_funnel',
    'chute': 'chute',
    'clockwork_bearing': 'clockwork',
    'clutch': 'clutch',
    'copper_casing': 'copper_casing',
    'creative_motor': 'creative_motor',
    'crushing_wheel': 'crusher_wheel',
    'deepslate_zinc_ore': 'deepslate_zinc_ore',
    'deployer': 'deployer',
    'depot': 'depot_rim',
    'encased_chain_drive': 'chain_drive',
    'fluid_pipe': 'pipe_copper',
    'fluid_tank': 'fluid_tank',
    'fluid_valve': 'valve',
    'flywheel': 'flywheel',
    'funnel': 'andesite_funnel',
    'gantry_carriage': 'gantry',
    'gantry_shaft': 'gantry',
    'gearbox': 'gearbox',
    'gearshift': 'gearshift',
    'goggles': 'cranial_goggles',
    'hand_crank': 'hand_crank',
    'industrial_iron_block': 'industrial_iron',
    'mechanical_bearing': 'bearing',
    'mechanical_harvester': 'harvester',
    'mechanical_plough': 'plough',
    'mechanical_pump': 'pump',
    'piston_extension_pole': 'piston',
    'portable_storage_interface': 'cargo_dock',
    'raw_zinc_block': 'raw_zinc_block',
    'shaft': 'steel_axle',
    'shaft_half': 'steel_axle',
    'smart_chute': 'smart_chute',
    'speedometer': 'speedometer',
    'sticker': 'sticker',
    'stressometer': 'stressometer',
    'weathered_iron_block': 'weathered_iron',
    'windmill_bearing': 'windmill',
    'zinc_block': 'zinc_block',
    'zinc_ore': 'zinc_ore',
}

PART_MATERIALS = {
    'adjustable_chain_gearshift/false': 'clutch',
    'adjustable_chain_gearshift/true': 'gearshift',
    'bearing/top': 'bearing',
    'bearing/top_wooden': 'windmill',
    'deployer/pole': 'steel_axle',
    'encased_fan/block': 'fan_case',
    'encased_fan/propeller': 'fan_blade',
    'fluid_valve/pointer': 'gauge_needle',
    'gauge/dial': 'gauge_needle',
    'gauge/speedometer/head': 'speedometer',
    'gauge/stressometer/head': 'stressometer',
    'hand_crank/handle': 'oak_paddles',
    'mechanical_crafter/arrow': 'brass_cog',
    'mechanical_crafter/belt': 'belt_rubber',
    'mechanical_crafter/belt_animated': 'belt_rubber',
    'mechanical_drill/block': 'drill_case',
    'mechanical_drill/head': 'cutting_steel',
    'mechanical_mixer/block': 'mixer_case',
    'mechanical_mixer/head': 'cutting_steel',
    'mechanical_mixer/pole': 'steel_axle',
    'mechanical_press/block': 'press_case',
    'mechanical_press/head': 'cutting_steel',
    'mechanical_pump/cog': 'brass_cog',
    'portable_storage_interface/block_middle_powered': 'cargo_dock_powered',
    'rope_pulley/rope': 'rope',
    'rope_pulley/rope_half': 'rope',
    'rope_pulley/rope_coil': 'rope',
    'sail/sail_frame_fallback': 'chassis',
    'smart_chute/block_powered': 'smart_chute_powered',
    'sticker/block_powered': 'sticker_powered',
}


def block_material(name, slot, texture):
    family, _, part = name.partition('/')
    if texture == 'minecraft:block/fire_0':
        return None  # Use Minecraft's existing animated effect.
    if texture == 'minecraft:block/glass':
        return 'tank_glass'
    if name in PART_MATERIALS:
        return PART_MATERIALS[name]
    if family in ('water_wheel', 'large_water_wheel'):
        wood = texture.removeprefix('minecraft:block/').removesuffix('_planks')
        return f'{wood}_paddles' if '_material_' in part else 'water_wheel'
    if family == 'sail':
        return part.removesuffix('_sail_fallback') + '_canvas'
    if family.startswith(('cut_', 'polished_cut_', 'small_')) or family == 'layered_andesite':
        return family
    if family.startswith(('large_cogwheel', 'cogwheel')):
        return 'oak_cog'
    if family.startswith(('andesite_encased_cogwheel', 'andesite_encased_large_cogwheel')):
        return 'andesite_casing'
    if family.startswith(('brass_encased_cogwheel', 'brass_encased_large_cogwheel')):
        return 'brass_casing'
    if family in ('belt_casing', 'belt_cover'):
        return 'andesite_casing' if part.startswith('andesite_') else 'brass_casing'
    if family in ('funnel', 'brass_funnel') and part.endswith('_on'):
        return FAMILIES[family] + '_active'
    if family == 'blaze_burner':
        if 'super' in part:
            return 'superheated'
        if 'rods' in part:
            return 'cold_flame'
        if part.startswith('blaze/'):
            return 'burner_hot'
        if part.startswith('goggles'):
            return 'cranial_goggles'
        return 'burner_case'
    if family == 'chassis_fallback':
        if part.startswith('sticky_'):
            return 'sticker'
        return 'chassis_end' if texture.endswith('_log') else 'chassis'
    if family == 'item_drain':
        return 'drain' if slot == 'grate' else 'pipe_copper'
    if family == 'mechanical_crafter':
        return {'frame': 'brass_casing', 'slot': 'chassis'}.get(slot, 'crafter')
    if family == 'mechanical_pump' and slot == 'outlet':
        return 'steel_axle'
    if family == 'mechanical_piston':
        return 'sticky_piston' if part.startswith('sticky/') else 'piston'
    if family == 'mechanical_saw':
        return 'cutting_steel' if part.startswith('blade_') else 'saw'
    if family == 'millstone':
        return 'millstone'
    if family == 'rope_pulley':
        return 'pulley'
    return FAMILIES[family]


def retarget(path, model):
    """Preserve geometry and aliases. Only replace concrete texture IDs."""
    prefix = 'assets/create/models/'
    if not path.startswith(prefix):
        return
    category, _, name = path.removeprefix(prefix).removesuffix('.json').partition('/')
    texture_category = 'block' if category == 'entity' else category
    for slot, texture in model.get('textures', {}).items():
        if not texture.startswith('minecraft:'):
            continue
        if category == 'item':
            material = name
        elif category == 'entity':
            material = 'sail_canvas' if name == 'logistics_hat' else 'belt_rubber'
        else:
            material = block_material(name, slot, texture)
        if material is not None:
            model['textures'][slot] = f'create:{texture_category}/fallback/{material}'


def texture_path(texture_id):
    return PurePosixPath('assets/create/textures') / (texture_id.removeprefix('create:') + '.png')
