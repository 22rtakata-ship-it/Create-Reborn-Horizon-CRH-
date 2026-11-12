#!/usr/bin/env python3
"""Preflight and run Phase-1 acceptance using the real isolated-client harness.

Exit 0: requested checks passed; 1: checks failed; 2: prerequisites blocked.
A successful preflight is never reported as a successful gameplay run.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def preflight(game, ctl, resources):
    pins = json.loads((ROOT / 'compatibility.json').read_text())
    blocked = []
    for label, path in [('client', game), ('control binary', ctl)]:
        if not path.is_file():
            blocked.append(f'Missing {label}: {path}')
    metadata = game.parent / 'native-mods.json'
    try:
        build = json.loads(metadata.read_text())
        if not isinstance(build, dict) or not isinstance(build.get('mods'), list) or not all(isinstance(mod, dict) for mod in build['mods']):
            raise ValueError('Invalid native build metadata')
        if build.get('horizon') != pins['horizon_revision']:
            blocked.append('Client Horizon revision differs from compatibility.json')
        if build.get('horizon_mixins') != pins['horizon_mixins_revision']:
            blocked.append('Client Mixins revision differs from compatibility.json')
        if not any(mod.get('id') == 'create' for mod in build.get('mods', [])):
            blocked.append('Client sidecar does not enable the create native mod')
    except (OSError, ValueError, TypeError):
        blocked.append(f'Missing or malformed native build sidecar: {metadata}')
    if resources is None or not all((resources / name).is_file() for name in ['facts.json', 'horizon-resources.json']):
        blocked.append('Missing prepared Minecraft 26.3 resources (facts.json and horizon-resources.json); supply --resources from your own Minecraft installation')
    return blocked


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', required=True, type=Path)
    parser.add_argument('--ctl', required=True, type=Path)
    parser.add_argument('--resources', type=Path)
    parser.add_argument('--preparer', type=Path, help='Official pinned horizon-prepare; checks completeness before launch')
    parser.add_argument('--horizon', type=Path, help='Pinned source folder containing research-lock.json; required with --preparer')
    parser.add_argument('--offscreen', action='store_true')
    parser.add_argument('--preflight-only', action='store_true')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.preparer and not args.horizon:
        parser.error('--preparer requires --horizon')
    game, ctl = args.game.resolve(), args.ctl.resolve()
    resources = args.resources.resolve() if args.resources else None
    result = {'status': 'blocked', 'gameplay_attempted': False, 'gameplay_passed': False, 'blocked': preflight(game, ctl, resources)}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    if not result['blocked'] and args.preparer:
        checked = subprocess.run([str(args.preparer.resolve()), '--check', '--root', str(args.horizon.resolve()),
                                  '--output', str(resources)], capture_output=True, text=True)
        result['resource_check'] = {'returncode': checked.returncode, 'stdout': checked.stdout, 'stderr': checked.stderr}
        if checked.returncode:
            result['blocked'].append('Official preparer reports incomplete or outdated resources')
    exit_code = 2
    if not result['blocked']:
        if args.preflight_only:
            result['status'] = 'preflight_passed'
            exit_code = 0
        else:
            (ROOT / '.local').mkdir(exist_ok=True)
            command = [sys.executable, str(ROOT / 'tools/smoke.py'), '--core-only', '--game', str(game),
                       '--ctl', str(ctl), '--resources', str(resources)]
            if args.offscreen:
                command.append('--offscreen')
            log = args.report.with_suffix('.log')
            result['gameplay_attempted'] = True
            with log.open('w') as output:
                run = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT)
            result.update(status='passed' if run.returncode == 0 else 'failed', gameplay_passed=run.returncode == 0,
                          returncode=run.returncode, log=str(log))
            exit_code = 0 if run.returncode == 0 else 1
    args.report.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    return exit_code


if __name__ == '__main__':
    raise SystemExit(main())
