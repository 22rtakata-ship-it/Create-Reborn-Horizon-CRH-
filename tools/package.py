#!/usr/bin/env python3
"""Build and validate the documented native/content archive formats."""
import argparse
import json
import shutil
import subprocess
import zipfile
from common import ROOT, PINS, check_horizon


def main():
    argparse.ArgumentParser(description=__doc__).parse_args()
    check_horizon()
    cli = ROOT / "target/debug/cargo-horizon-mod"
    subprocess.run([str(cli), "build"], cwd=ROOT / "content", check=True)
    destination = ROOT / "dist"
    destination.mkdir(parents=True, exist_ok=True)
    manifest = {"schema": 1, "id": "create", "version": "0.1.0", "game": PINS["minecraft_target_version"],
                "horizon": PINS["horizon_revision"], "mixins": PINS["horizon_mixins_revision"]}
    native_files = {"horizon-native.json": (json.dumps(manifest, indent=2) + "\n").encode()}
    for path in sorted((ROOT / "native").rglob("*")):
        if path.is_file() and path.suffix in [".rs", ".toml"]:
            native_files["mixins/create/" + path.relative_to(ROOT / "native").as_posix()] = path.read_bytes()
    for name in ["LICENSE", "THIRD_PARTY_NOTICES.md"]:
        # NOTICE* is an allowed native bundle text file.
        native_files["mixins/create/" + ("NOTICE.md" if name.endswith(".md") else name)] = (ROOT / name).read_bytes()
    content_archive = ROOT / "content/target/horizon-mod/create.hmod"
    with zipfile.ZipFile(content_archive) as package:
        content_files = {name: package.read(name) for name in package.namelist() if not name.endswith("/")}
    outputs = [("create-0.1.0.hnmod", native_files), ("create-0.1.0.hmod", {**native_files, **content_files})]
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
    content_target = destination / "create-content-0.1.0.hmod"
    shutil.copy2(content_archive, content_target)
    subprocess.run([str(cli), "check", str(content_target), "--json"], check=True)
    print("Native archives require the local documented Mixins compatibility fix on this Horizon pin.")
    print("Original assets are never packaged. Place your own JAR in the instance's horizon-mods/create-resources/.")


if __name__ == "__main__":
    main()
