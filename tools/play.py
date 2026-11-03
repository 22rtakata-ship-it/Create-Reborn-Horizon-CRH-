#!/usr/bin/env python3
"""Play the prototype in the project's own persistent development world."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from common import ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--resources", type=Path)
    parser.add_argument("--original-assets", action="store_true")
    args = parser.parse_args()
    config = ROOT / ".local/development.json"
    if args.resources:
        origin = args.resources.resolve()
        config.parent.mkdir(parents=True, exist_ok=True)
        config.write_text(json.dumps({"resources": str(origin)}, indent=2) + "\n")
    elif config.exists():
        origin = Path(json.loads(config.read_text())["resources"])
    else:
        parser.error("Pass --resources with a prepared Horizon 26.3 resource folder once")
    if not (origin / "facts.json").is_file(): raise SystemExit("Prepared resources not found")
    game = ROOT / "target/fast/horizon-client"
    if not game.is_file(): raise SystemExit("Run python3 tools/stage.py build first")
    resources = ROOT / ".local/play-resources"
    if not resources.exists():
        if os.uname().sysname == "Darwin": subprocess.run(["cp", "-cR", str(origin), str(resources)], check=True)
        else: shutil.copytree(origin, resources)
    env = os.environ.copy()
    for key, folder in [("HORIZON_HOME", "play-home"), ("HORIZON_LAUNCHER_HOME", "play-launcher"),
                        ("XDG_DATA_HOME", "play-data"), ("XDG_CONFIG_HOME", "play-config"),
                        ("HORIZON_LAUNCHER_TRASH", "play-trash"), ("HORIZON_LAUNCHER_CLIPBOARD", "play-clipboard"),
                        ("HORIZON_LAUNCHER_DESKTOP", "play-desktop")]: env[key] = str(ROOT / ".local" / folder)
    env.update({"HORIZON_NO_AUDIO": "1", "HORIZON_LAUNCHER_NO_SELF_UPDATE": "1"})
    content = ROOT / ".local/dist/create-content-0.1.0.hmod" if args.original_assets else ROOT / "content"
    if not content.exists(): raise SystemExit("Run tools/package.py --with-original-assets first")
    work = ROOT / ".local/play-home"
    work.mkdir(parents=True, exist_ok=True)
    subprocess.run([str(game), "--resources", str(resources), "--world", "CreateDevelopment", "--creative",
                    "--render-distance", "4", "--dev-mod", str(content)], cwd=work, env=env, check=True)


if __name__ == "__main__":
    main()
