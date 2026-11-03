#!/usr/bin/env python3
"""Build and validate the documented native/content archive formats."""
import argparse
import json
import shutil
import subprocess
import zipfile
from common import ROOT, NATIVE, PINS, check_horizon


def main():
    argparse.ArgumentParser(description=__doc__).parse_args()
    check_horizon()
    cli = ROOT / "target/debug/cargo-horizon-mod"
    subprocess.run([str(cli), "build"], cwd=ROOT / "content", check=True)
    destination = ROOT / "dist"
    destination.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((ROOT / "horizon-native.json").read_text())
    if (manifest["horizon"], manifest["mixins"]) != (PINS["horizon_revision"], PINS["horizon_mixins_revision"]):
        raise SystemExit("horizon-native.json differs from compatibility.json")
    native_files = {"horizon-native.json": (json.dumps(manifest, indent=2) + "\n").encode()}
    for path in sorted(NATIVE.rglob("*")):
        if path.is_file() and (path.suffix in [".rs", ".toml"] or "data" in path.relative_to(NATIVE).parts):
            native_files["mixins/create/" + path.relative_to(NATIVE).as_posix()] = path.read_bytes()
    for name in ["LICENSE", "THIRD_PARTY_NOTICES.md"]:
        # NOTICE* is an allowed native bundle text file.
        native_files["mixins/create/" + ("NOTICE.md" if name.endswith(".md") else name)] = (ROOT / name).read_bytes()
    content_archive = ROOT / "content/target/horizon-mod/create.hmod"
    with zipfile.ZipFile(content_archive) as package:
        content_files = {name: package.read(name) for name in package.namelist() if not name.endswith("/")}
    outputs = [(f"create-{manifest['version']}.hnmod", native_files), (f"create-{manifest['version']}.hmod", {**native_files, **content_files})]
    for name, files in outputs:
        target = destination / name
        with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as package:
            for path, data in sorted(files.items()):
                info = zipfile.ZipInfo(path, (2026, 11, 3, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                package.writestr(info, data)
        subprocess.run([str(cli), "check", str(target), "--kind", "native-mod", "--json"], check=True)
        print(target)
    content_target = destination / f"create-content-{manifest['version']}.hmod"
    shutil.copy2(content_archive, content_target)
    subprocess.run([str(cli), "check", str(content_target), "--json"], check=True)
    print("Native archives include all public content and pin the published Mixins PR #26 commit; no local tool patch.")
    print("Original assets are never packaged. Place your own JAR in the instance's horizon-mods/create-resources/.")


if __name__ == "__main__":
    main()
