#!/usr/bin/env python3
"""Check contracts required for installing this source as one native mod."""
import json
import re
from common import ROOT, NATIVE, PINS


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
                for p in sorted(content.rglob("*")) if p.is_file()
                and "target" not in p.relative_to(content).parts}
    actual = json.loads((NATIVE / "data/content.json").read_text())
    assert actual == expected, "Bundled public content is stale: run tools/content.py"
    for path in NATIVE.rglob("*"):
        assert not path.is_symlink(), "Source bundles must not contain links: " + str(path)
        assert path.suffix.lower() not in {".jar", ".class", ".png", ".ogg"}, str(path)
    assert (NATIVE / "LICENSE").read_bytes() == (ROOT / "LICENSE").read_bytes()
    assert (NATIVE / "NOTICE.md").read_bytes() == (ROOT / "THIRD_PARTY_NOTICES.md").read_bytes()
    print("Source-install contracts passed: pins, bundled content, notices and no upstream asset files.")


if __name__ == "__main__":
    main()
