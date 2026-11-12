#!/usr/bin/env python3
"""Portable entry point for the modular workspace; never touches player worlds."""
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def run(*args):
    subprocess.run(args, cwd=ROOT, check=True)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['verify', 'build', 'test', 'bench', 'compatibility'])
    parser.add_argument('--horizon', type=Path)
    args = parser.parse_args()
    if args.action == 'compatibility':
        if args.horizon is None:
            parser.error('--horizon is required')
        actual = subprocess.check_output(['git', '-C', str(args.horizon), 'rev-parse', 'HEAD'], text=True).strip()
        expected = json.loads((ROOT/'compatibility.json').read_text())['horizon_revision']
        if actual != expected:
            raise SystemExit(f'Horizon mismatch: expected {expected}, got {actual}')
        print('Exact Horizon revision matches; runtime compatibility still requires native checks.')
    elif args.action == 'verify':
        run('cargo', 'fmt', '--all', '--check')
        run('cargo', 'test', '--locked', '--workspace')
        for package in ['create-core', 'create-mechanics', 'create-contraptions', 'create-horizon-adapter']:
            run('cargo', 'check', '--locked', '-p', package, '--no-default-features')
        for feature in ['mechanics', 'contraptions']:
            run('cargo', 'check', '--locked', '-p', 'create-horizon-adapter', '--no-default-features', '--features', feature)
        run('cargo', 'clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings')
        run('python3', 'tools/check_distribution.py')
    elif args.action == 'bench':
        for example in ['network_baseline', 'network_cache_baseline']:
            run('cargo', 'run', '--locked', '--release', '-p', 'create-testkit', '--example', example)
    else:
        run('cargo', args.action, '--locked', '--workspace')

if __name__ == '__main__':
    main()
