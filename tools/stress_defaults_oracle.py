#!/usr/bin/env python3
"""Compare shared Rust impacts against pinned Create block registrations.

This checks default numerical config, not native world dispatch, custom server
config overrides, rendering or Goggles equipment interaction.
"""
import hashlib
import json
import re
import subprocess
from common import CREATE, PINS, ROOT, cargo


def main():
    path='src/main/java/com/simibubi/create/AllBlocks.java'
    raw=subprocess.check_output(['git','show',PINS['create_revision']+':'+path],cwd=CREATE)
    source=raw.decode()
    api_path='src/main/java/com/simibubi/create/api/stress/BlockStressValues.java'
    api=subprocess.check_output(['git','show',PINS['create_revision']+':'+api_path],cwd=CREATE).decode()
    assert 'return supplier == null ? 0 : supplier.getAsDouble();' in api
    cargo('build','--locked','--example','stress_defaults_probe','--target-dir',ROOT/'target/core')
    actual=json.loads(subprocess.check_output([str(ROOT/'target/core/debug/examples/stress_defaults_probe')],text=True))
    expected={}
    for block in actual:
        declaration=re.search(r'REGISTRATE\s*\.\s*block\("'+re.escape(block)+r'"\s*,',source)
        if not declaration:
            raise AssertionError(('missing source registration',block))
        registration=source[declaration.start():source.index('.register();',declaration.end())]
        impact=re.search(r'CStress\.setImpact\(([0-9.]+)\)',registration)
        expected[block]=float(impact[1]) if impact else 0.0
    assert actual==expected,{'java_defaults':expected,'rust':actual}
    report={'passed':len(actual),'reference':PINS['create_revision'],
            'source_sha256':hashlib.sha256(raw).hexdigest(),
            'scope':__doc__,'impacts_su_per_rpm':actual}
    folder=ROOT/'.local/stress-defaults-oracle';folder.mkdir(parents=True,exist_ok=True)
    (folder/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))


if __name__=='__main__':
    main()
