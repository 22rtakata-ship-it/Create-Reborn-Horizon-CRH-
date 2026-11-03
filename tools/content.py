#!/usr/bin/env python3
"""Generate the small playable content set with original, provisional models.

Models reference vanilla textures by ID and contain no upstream texture bytes.
The full horizon-port-pack carry-over remains in .local for subsequent milestones.
"""
import json
from common import ROOT

CONTENT = ROOT / "content"
BLOCKS = ["creative_motor", "shaft", "cogwheel", "large_cogwheel", "mechanical_press", "depot"]
SHEETS = ["iron", "golden", "copper"]


def write(path, value):
    target = CONTENT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def model(name, boxes, texture="minecraft:block/iron_block"):
    faces = {face: {"texture": "#all"} for face in ["down", "up", "north", "south", "west", "east"]}
    write("assets/create/models/block/" + name + ".json", {
        "textures": {"all": texture, "particle": texture},
        "elements": [{"from": a, "to": b, "faces": faces} for a, b in boxes],
    })


def main():
    CONTENT.mkdir(exist_ok=True)
    (CONTENT / "horizon-mod.toml").write_text('''id = "create"
name = "Create for Horizon — prototype"
version = "0.1.0"
description = "First milestone: motor, shafts, cogs, mechanical press and depot."
authors = ["create-horizon contributors"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''')
    for block in BLOCKS:
        declaration = {"template": "minecraft:glass", "item": True,
                       "properties": {"hardness": 1.5, "render_shape": "ENTITYBLOCK_ANIMATED"}}
        if block in ["shaft", "cogwheel", "large_cogwheel"]:
            declaration["state_properties"] = {"axis": {"type": "enum", "values": ["x", "y", "z"]}}
            declaration["default_state"] = {"axis": "y"}
            shapes = {"x": [[0, .375, .375, 1, .625, .625]],
                      "y": [[.375, 0, .375, .625, 1, .625]],
                      "z": [[.375, .375, 0, .625, .625, 1]]}
            if "cogwheel" in block:
                shapes = {"x": [[.375, 0, 0, .625, 1, 1]],
                          "y": [[0, .375, 0, 1, .625, 1]],
                          "z": [[0, 0, .375, 1, 1, .625]]}
            declaration["shapes"] = {"property": "axis", "values": shapes}
        elif block in ["creative_motor", "mechanical_press"]:
            values = ["north", "south", "east", "west"]
            if block == "creative_motor": values += ["up", "down"]
            declaration["state_properties"] = {"facing": {"type": "enum", "values": values}}
            declaration["default_state"] = {"facing": "east"}
            if block == "creative_motor":
                declaration["state_properties"]["speed"] = {"type": "enum", "values": ["16", "32", "64", "128", "256"]}
                declaration["default_state"]["speed"] = "16"
        else:
            # The barrel template supplies the ordinary container-open interaction.
            declaration["template"] = "minecraft:barrel"
            declaration["shapes"] = {"default": [[0, 0, 0, 1, .875, 1]]}
        write("data/create/horizon/block/" + block + ".json", declaration)
        # A blockstate model is still needed for the block item and model baking.
        name = {"creative_motor": "creative_motor/block", "mechanical_press": "mechanical_press/block", "depot": "depot/block"}.get(block, block)
        write("assets/create/blockstates/" + block + ".json", {"variants": {"": {"model": "create:block/" + name}}})
        write("assets/create/models/item/" + block + ".json", {"parent": "create:block/" + name})
        write("assets/create/items/" + block + ".json", {"model": {"type": "minecraft:model", "model": "create:item/" + block}})
        write("data/create/loot_table/blocks/" + block + ".json", {
            "type": "minecraft:block", "pools": [{"rolls": 1, "entries": [{"type": "minecraft:item", "name": "create:" + block}]}]})
    shaft = [([6, 0, 6], [10, 16, 10])]
    model("shaft", shaft)
    for name, low, high in [("cogwheel", 2, 14), ("large_cogwheel", -1, 17)]:
        boxes = shaft + [([low, 6, low], [high, 10, high])]
        for x, z in [(0, 6), (6, 0), (12, 6), (6, 12)]:
            boxes.append(([x, 5, z], [x + 4, 11, z + 4]))
        model(name, boxes, "minecraft:block/oak_planks")
    model("creative_motor/block", [([1, 0, 1], [15, 14, 15])], "minecraft:block/copper_block")
    model("mechanical_press/block", [([0, 8, 0], [16, 16, 16])])
    model("mechanical_press/head", [([6, -12, 6], [10, 9, 10]), ([2, -14, 2], [14, -10, 14])])
    model("depot/block", [([1, 0, 1], [15, 12, 15]), ([0, 12, 0], [16, 14, 16])], "minecraft:block/smooth_stone")
    write("data/create/horizon/menu/depot.json", {"slots": 1})
    for name in SHEETS:
        write("data/create/horizon/item/" + name + "_sheet.json", {"template": "minecraft:iron_ingot", "max_stack_size": 64})
        vanilla = "gold" if name == "golden" else name
        write("assets/create/models/item/" + name + "_sheet.json", {"parent": "minecraft:item/generated", "textures": {"layer0": "minecraft:item/" + vanilla + "_ingot"}})
        write("assets/create/items/" + name + "_sheet.json", {"model": {"type": "minecraft:model", "model": "create:item/" + name + "_sheet"}})
        # Adapted from the three MIT Create pressing recipes. Convention tags are
        # narrowed to vanilla ingots for this initial, self-contained content set.
        write("data/create/recipe/pressing/" + vanilla + "_ingot.json", {
            "type": "create:pressing", "ingredients": ["minecraft:" + vanilla + "_ingot"],
            "results": [{"id": "create:" + name + "_sheet"}],
        })
    ids = BLOCKS + [name + "_sheet" for name in SHEETS]
    write("data/create/horizon/creative_tab/kinetics.json", {"title": "itemGroup.create_horizon", "icon": "create:cogwheel", "items": ["create:" + name for name in ids]})
    english = ["Creative Motor", "Shaft", "Cogwheel", "Large Cogwheel", "Mechanical Press", "Depot"]
    russian = ["Творческий мотор", "Вал", "Шестерня", "Большая шестерня", "Механический пресс", "Депо"]
    for language, names, title in [("en_us", english, "Create for Horizon"), ("ru_ru", russian, "Create для Horizon")]:
        lang = {"itemGroup.create_horizon": title}
        lang.update({"block.create." + block: name for block, name in zip(BLOCKS, names)})
        lang.update({"item.create." + sheet + "_sheet": name for sheet, name in zip(SHEETS,
                    ["Iron Sheet", "Golden Sheet", "Copper Sheet"] if language == "en_us" else ["Железная пластина", "Золотая пластина", "Медная пластина"])})
        write("assets/create/lang/" + language + ".json", lang)
    print("Generated six blocks, three sheets, a depot menu and three pressing recipes.")


if __name__ == "__main__":
    main()
