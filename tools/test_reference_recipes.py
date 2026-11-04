"""Native dry-mixing export must reject unsupported JSON instead of erasing it."""
import copy
import unittest

from reference_recipes import dry_mixing


class DryMixing(unittest.TestCase):
    def setUp(self):
        self.recipe = {'type': 'create:mixing', 'ingredients': [
            {'item': 'minecraft:andesite'}, {'tag': 'c:nuggets/iron'}],
            'results': [{'id': 'create:andesite_alloy'}]}

    def test_source_item_tag_and_alternative_forms(self):
        self.assertTrue(dry_mixing(self.recipe))
        self.recipe['ingredients'][0] = [{'item': 'minecraft:andesite'}, {'item': 'minecraft:stone'}]
        self.assertTrue(dry_mixing(self.recipe))
        self.recipe['results'][0].update(count=2, chance=.25, components={'minecraft:custom_data': {'marker': 1}})
        self.assertTrue(dry_mixing(self.recipe))

    def test_fluid_heat_and_unknown_ingredient_fields_fail_whole_recipe(self):
        for ingredient in [
            {'type': 'neoforge:single', 'fluid': 'minecraft:water', 'amount': 1000},
            {'item': 'minecraft:andesite', 'amount': 2},
            {'tag': 'c:nuggets/iron', 'unknown': 7},
            {'type': 'unported:ingredient', 'item': 'minecraft:andesite'},
            [], {}, 3,
        ]:
            with self.subTest(ingredient=ingredient):
                recipe = copy.deepcopy(self.recipe)
                recipe['ingredients'][0] = ingredient
                self.assertFalse(dry_mixing(recipe))
        for heat in ['heated', 'superheated', None, 1, 'unknown']:
            recipe = copy.deepcopy(self.recipe)
            recipe['heat_requirement'] = heat
            self.assertFalse(dry_mixing(recipe))
        for duration in [None, '100', .5, True, 1 << 31]:
            recipe = copy.deepcopy(self.recipe)
            recipe['processing_time'] = duration
            self.assertFalse(dry_mixing(recipe))

    def test_output_fluid_and_invalid_stack_numbers_are_not_flattened(self):
        for output in [
            {'fluid': 'minecraft:water', 'amount': 1000},
            {'id': 'create:andesite_alloy', 'amount': 1000},
            {'id': 'create:andesite_alloy', 'count': True},
            {'id': 'create:andesite_alloy', 'count': 0},
            {'id': 'create:andesite_alloy', 'count': 100},
            {'id': 'create:andesite_alloy', 'chance': float('nan')},
            {'id': 'create:andesite_alloy', 'chance': 1.01},
            {'id': 'create:andesite_alloy', 'components': []},
        ]:
            with self.subTest(output=output):
                recipe = copy.deepcopy(self.recipe)
                recipe['results'][0] = output
                self.assertFalse(dry_mixing(recipe))


if __name__ == '__main__':
    unittest.main()
