#!/usr/bin/env python3
"""Check contracts required for installing this source as one native mod."""
import json
import re
from common import ROOT, NATIVE, PINS
from catalog import BLOCKS, EXTRA_MODELS, INTERNAL_BLOCKS, ITEMS, PALETTE_BLOCKS, PARTS, rust_catalog


def main():
    manifest = json.loads((ROOT / "horizon-native.json").read_text())
    assert manifest["horizon"] == PINS["horizon_revision"], "Horizon pin differs"
    assert manifest["mixins"] == PINS["horizon_mixins_revision"], "Mixins pin differs"
    assert manifest["id"] == "create", "Native namespace differs"
    source = (NATIVE / "horizon-mixin.toml").read_text()
    assert re.search(r'^horizon_revision = "' + manifest["horizon"] + r'"$', source, re.M)
    assert re.search(r'^version = "' + re.escape(manifest["version"]) + r'"$', source, re.M)
    content = ROOT / "content"
    expected = {p.relative_to(content).as_posix(): p.read_text()
                for p in sorted(content.rglob("*")) if p.is_file() and p.suffix in {".json", ".toml"}
                and "target" not in p.relative_to(content).parts}
    actual = json.loads((NATIVE / "data/content.json").read_text())
    assert actual == expected, "Bundled public content is stale: run tools/content.py"
    model_prefix = "assets/create/models/"
    required = {model_prefix + "block/" + part + ".json" for part in PARTS}
    required.update(model_prefix + name + ".json" for name in EXTRA_MODELS)
    missing_models = required - actual.keys()
    assert not missing_models, "Missing built-in models: " + ", ".join(sorted(missing_models))
    for name, text in actual.items():
        if name.startswith(model_prefix) and name.endswith(".json"):
            for texture in json.loads(text).get("textures", {}).values():
                assert texture.startswith(("minecraft:", "#")), (name, texture)
    reference=json.loads((ROOT / "tools/data/recipes.json").read_text())
    assert reference["reference"] == PINS["create_revision"]
    assert (NATIVE / "assets/src/catalog.rs").read_text() == rust_catalog(), "Importer catalog is stale"
    declared_blocks={p.stem for p in (content / "data/create/horizon/block").glob("*.json")}
    declared_items={p.stem for p in (content / "data/create/horizon/item").glob("*.json")}
    assert declared_blocks == set(BLOCKS+INTERNAL_BLOCKS) and declared_items == set(ITEMS)
    tabs = {path.stem: json.loads(path.read_text())
            for path in (content / "data/create/horizon/creative_tab").glob("*.json")}
    assert set(tabs) == {"kinetics", "palettes"}
    assert tabs["kinetics"]["title"] == "itemGroup.create.base"
    assert tabs["kinetics"]["items"] == ["create:" + name for name in BLOCKS + ITEMS if name not in PALETTE_BLOCKS]
    assert tabs["palettes"]["title"] == "itemGroup.create.palettes"
    assert tabs["palettes"]["items"] == ["create:" + name for name in PALETTE_BLOCKS]
    assert len(list((content / "data/create/recipe").rglob("*.json"))) == len(reference["recipes"])
    for path, value in reference.get('loot_tables', {}).items():
        assert json.loads((content / 'data' / path).read_text()) == value, 'Source loot policy changed: ' + path
    for path in [*NATIVE.rglob("*"), *content.rglob("*")]:
        assert not path.is_symlink(), "Source bundles must not contain links: " + str(path)
        assert path.suffix.lower() not in {".jar", ".class", ".png", ".ogg"}, str(path)
    assert (NATIVE / "LICENSE").read_bytes() == (ROOT / "LICENSE").read_bytes()
    assert (NATIVE / "NOTICE.md").read_bytes() == (ROOT / "THIRD_PARTY_NOTICES.md").read_bytes()
    print("Source-install contracts passed: pins, bundled content, notices and no upstream asset files.")


if __name__ == "__main__":
    main()
