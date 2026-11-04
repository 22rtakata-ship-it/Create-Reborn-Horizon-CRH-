#!/usr/bin/env python3
"""Build an unpublished local Horizon extension with the official mixins tool.

The checkout is read-only input. All mod snapshots, transformed source, Cargo
outputs and artifacts stay under this repository's .local directory. This is a
developer pipeline, not an official Launcher installation or cache mutation.
"""
import argparse
import fcntl
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import posixpath
import re
import shutil
import subprocess
import tarfile
import tempfile

from common import ROOT, NATIVE, environment


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024*1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def clean_revision(source):
    revision = subprocess.check_output(['git','rev-parse','HEAD'],cwd=source,text=True).strip()
    dirty = subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=source,text=True)
    if dirty.strip():
        raise ValueError('Commit the local Horizon changes before building an immutable source snapshot')
    return revision


def snapshot_source(source, revision, destination):
    """Use only committed files; materialize tracked internal documentation links.

    Official mixins intentionally refuses symlink inputs. The Git archive is a
    private immutable input, with links resolved only to regular archive files;
    neither working-tree extras nor external link targets enter the build.
    """
    archive = subprocess.check_output(['git', 'archive', '--format=tar', revision], cwd=source)
    destination.mkdir()
    links = {}
    with tarfile.open(fileobj=io.BytesIO(archive)) as package:
        members = {entry.name: entry for entry in package.getmembers()}
        for name, entry in members.items():
            path = PurePosixPath(name)
            if path.is_absolute() or '..' in path.parts:
                raise ValueError('Invalid Git archive path: ' + name)
            output = destination.joinpath(*path.parts)
            if entry.isdir():
                output.mkdir(parents=True, exist_ok=True)
                continue
            current, visited = entry, set()
            while current.issym():
                if current.name in visited or posixpath.isabs(current.linkname):
                    raise ValueError('Invalid Git archive link: ' + name)
                visited.add(current.name)
                target = posixpath.normpath(posixpath.join(posixpath.dirname(current.name), current.linkname))
                if target.startswith('../') or target == '..' or target not in members:
                    raise ValueError('Git archive link escapes tracked files: ' + name)
                current = members[target]
            if not current.isfile():
                raise ValueError('Expected a regular tracked file: ' + name)
            if visited:
                links[name] = current.name
            output.parent.mkdir(parents=True, exist_ok=True)
            with package.extractfile(current) as stream:
                output.write_bytes(stream.read())
            output.chmod(current.mode & 0o777)
    (destination / '.horizon-revision').write_text(revision + '\n')
    return {'git_archive_sha256': hashlib.sha256(archive).hexdigest(), 'materialized_links': links}


def restore_unchanged_mtimes(previous, stage):
    """Retain Cargo freshness after a fresh official stage at the same path."""
    for old in previous.rglob('*'):
        if not old.is_file() or old.is_symlink():
            continue
        new = stage / old.relative_to(previous)
        metadata = old.stat()
        if new.is_file() and not new.is_symlink() and new.stat().st_size == metadata.st_size:
            if sha(old) == sha(new):
                os.utime(new, ns=(metadata.st_atime_ns, metadata.st_mtime_ns))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--horizon',type=Path,required=True,help='Clean local Horizon checkout, with the required extensions committed')
    parser.add_argument('--mixins',type=Path,required=True,help='Pinned official horizon-mixins executable')
    parser.add_argument('--target-dir',type=Path,help='Reuse an owned .local Cargo target; never an official Launcher cache')
    parser.add_argument('--stage-dir',type=Path,help='Reuse an owned transformed source cache; inputs and copied binaries remain separate snapshots')
    parser.add_argument('--profile',choices=['dev','performance'],default='dev')
    parser.add_argument('--check-only',action='store_true')
    parser.add_argument('--jobs',type=int,default=2)
    args=parser.parse_args()
    if not 1<=args.jobs<=32: parser.error('--jobs must be1..32')
    source=args.horizon.resolve()
    mixins=args.mixins.resolve()
    if not source.is_dir() or not mixins.is_file(): parser.error('Existing source checkout and mixins executable required')
    local=(ROOT/'.local').resolve();local.mkdir(exist_ok=True)
    target=(args.target_dir or local/'dev-native-target').resolve()
    stage=(args.stage_dir or local/'dev-native-stage').resolve()
    try: target.relative_to(local)
    except ValueError: parser.error('--target-dir must be owned by this repository under .local')
    try: stage.relative_to(local)
    except ValueError: parser.error('--stage-dir must be owned by this repository under .local')
    if stage == local or stage == target or stage in target.parents or target in stage.parents:
        parser.error('Source staging and Cargo target must be separate owned directories')
    # Reuse development Cargo outputs only. The Launcher owns its build/source
    # caches; its standard pipeline remains tools/launcher.py.
    if 'launcher' in target.parts and 'native' in target.parts:
        parser.error('Do not use an official Launcher native cache as a development target')
    revision=clean_revision(source)
    work=Path(tempfile.mkdtemp(prefix='dev-native-',dir=local))
    bundle=work/'mods/create'
    shutil.copytree(NATIVE,bundle,ignore=shutil.ignore_patterns('target','Cargo.lock','__pycache__','*.pyc'))
    manifest=bundle/'horizon-mixin.toml'
    text,count=re.subn(r'(?m)^horizon_revision = "[0-9a-f]+"$',
                       'horizon_revision = "'+revision+'"',manifest.read_text())
    if count!=1: raise ValueError('Expected one pinned native source revision')
    manifest.write_text(text)
    native={p.relative_to(bundle).as_posix():sha(p) for p in sorted(bundle.rglob('*')) if p.is_file()}
    result={'passed':False,'pipeline':'local developer source + official horizon-mixins; not Launcher native-build',
            'horizon':revision,'horizon_source':str(source),'mixins':str(mixins),'mixins_sha256':sha(mixins),
            'native_files':native,'profile':args.profile,'check_only':args.check_only,
            'target':str(target),'work':str(work)}
    result['stage_cache'] = str(stage)
    # The official tool atomically updates its marked stage and preserves
    # identical files' mtimes. A stable path also preserves Cargo's package
    # identity across mod edits. Hold this cache lock through compilation.
    stage.parent.mkdir(parents=True, exist_ok=True)
    stage_lock = stage.with_suffix('.build.lock').open('a')
    fcntl.flock(stage_lock, fcntl.LOCK_EX)
    def save(): (work/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    save();print(json.dumps({'work':str(work),'source':revision}),flush=True)
    def run(command,name,cwd):
        result.setdefault('commands',[]).append({'command':list(map(str,command)),'cwd':str(cwd),'log':name})
        save()
        with (work/name).open('wb') as log:
            status=subprocess.run(list(map(str,command)),cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT)
        if status.returncode:
            raise RuntimeError(name+' failed; inspect '+str(work/name))
    env=environment();env['CARGO_TARGET_DIR']=str(target);env['CARGO_BUILD_JOBS']=str(args.jobs)
    try:
        source_snapshot = work/'source'
        result['source_snapshot'] = snapshot_source(source, revision, source_snapshot)
        save()
        previous = None
        if os.uname().sysname == 'Darwin' and stage.exists():
            # The pinned official tool atomically exchanges existing stages on
            # Linux only. Preserve our marked developer cache as a private
            # backup, then ask the unchanged tool for a fresh output at the
            # same path. This does not touch Launcher caches or input source.
            if stage.is_symlink() or not (stage/'.horizon-mixins-stage.json').is_file():
                raise ValueError('Refusing to move a directory not owned by official Horizon Mixins')
            previous = work/'previous-stage'
            stage.rename(previous)
            result['previous_stage'] = str(previous)
            save()
        try:
            run([mixins,'apply','--workspace',source_snapshot,'--mods',work/'mods','--out',stage],'mixins.log',ROOT)
        except Exception:
            if previous is not None and not stage.exists():
                previous.rename(stage)
                result['previous_stage_restored'] = True
            raise
        if previous is not None:
            restore_unchanged_mtimes(previous, stage)
        if clean_revision(source)!=revision: raise ValueError('Horizon source changed during staging')
        cargo=ROOT/'.tools/rust/bin/cargo'
        command=[cargo,'check' if args.check_only else 'build','--locked','--offline','-p','horizon-client']
        if args.check_only: command+=['--lib','--bins']
        else:
            command+=['--profile','fast']
            if args.profile=='dev': command+=['--config','profile.fast.package.horizon-client.opt-level=0']
        run(command,'cargo.log',stage)
        if not args.check_only:
            binary=target/'fast/horizon-client'
            artifacts=work/'artifacts';artifacts.mkdir()
            shutil.copy2(binary,artifacts/'horizon-client')
            result['game']=str(artifacts/'horizon-client');result['game_sha256']=sha(artifacts/'horizon-client')
        result['passed']=True
    except Exception as error:
        result['error']=str(error)
        raise
    finally:
        save()
        stage_lock.close()
    print(json.dumps({'passed':result['passed'],'result':str(work/'result.json'),'game':result.get('game')}))


if __name__=='__main__': main()
