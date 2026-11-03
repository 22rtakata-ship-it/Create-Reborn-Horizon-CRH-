#!/usr/bin/env python3
"""Prepare the pinned Mixins tool with the documented compiler-attribute fix.

Horizon uses #![recursion_limit = "256"]. The pinned tool incorrectly treats
this built-in compiler limit as an unexpanded procedural attribute. The sole
local change allows that attribute; Rust source and hook validation stay intact.
"""
import io
import tarfile
from urllib.request import urlopen
from common import ROOT, PINS, cargo


def main():
    root = ROOT / ".tools" / "horizon-mixins"
    revision = PINS["horizon_mixins_revision"]
    if not root.exists():
        with urlopen("https://codeload.github.com/horizon-rust/horizon-mixins/tar.gz/" + revision, timeout=60) as response:
            data = response.read()
        root.mkdir(parents=True)
        with tarfile.open(fileobj=io.BytesIO(data)) as package:
            for member in package.getmembers():
                parts = member.name.split("/")[1:]
                if not parts or parts == [""]: continue
                if member.issym() or member.islnk() or ".." in parts:
                    raise SystemExit("Unexpected Mixins archive entry")
                target = root.joinpath(*parts)
                if member.isdir(): target.mkdir(parents=True, exist_ok=True)
                elif member.isfile():
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(package.extractfile(member).read())
        (root / ".revision").write_text(revision)
    if (root / ".revision").read_text().strip() != revision:
        raise SystemExit("Mixins source revision differs from compatibility.json")
    path = root / "crates/horizon-mixins-core/src/index.rs"
    source = path.read_text()
    changed = '                | "recursion_limit"' not in source
    if changed:
        anchor = '                | "track_caller"'
        if source.count(anchor) != 1:
            raise SystemExit("Mixins compiler-attribute patch no longer applies")
        path.write_text(source.replace(anchor, anchor + '\n                | "recursion_limit"'))
    if changed or not (ROOT / "target/debug/horizon-mixins").exists():
        cargo("build", "--locked", "--manifest-path", root / "Cargo.toml", "-p", "horizon-mixins-cli")


if __name__ == "__main__":
    main()
