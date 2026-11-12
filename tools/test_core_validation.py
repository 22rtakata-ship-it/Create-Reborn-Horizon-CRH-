"""Guard against reporting preflight/absent resources as gameplay success.

Synthetic files here check orchestration only; they are never game resources.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from core_validation import ROOT, preflight


class ValidationReporting(unittest.TestCase):
    def fixture(self, root):
        game, ctl = root/'horizon-client', root/'horizon-ctl'
        game.write_text('not executed by preflight')
        ctl.write_text('not executed by preflight')
        pins = json.loads((ROOT/'compatibility.json').read_text())
        build = {'horizon':pins['horizon_revision'],'horizon_mixins':pins['horizon_mixins_revision'],
                 'mods':[{'id':'create'}]}
        (root/'native-mods.json').write_text(json.dumps(build))
        return game, ctl

    def test_absent_resources_are_blocked_with_an_otherwise_valid_sidecar(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);game,ctl=self.fixture(root)
            self.assertTrue(any('resources' in reason for reason in preflight(game,ctl,None)))

    def test_wrong_engine_revision_is_rejected_before_launch(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);game,ctl=self.fixture(root)
            sidecar=root/'native-mods.json';build=json.loads(sidecar.read_text());build['horizon']='wrong'
            sidecar.write_text(json.dumps(build))
            (root/'facts.json').write_text('{}')
            (root/'horizon-resources.json').write_text('{}')
            self.assertTrue(any('Horizon revision' in reason for reason in preflight(game,ctl,root)))

    def test_malformed_sidecar_is_blocked_instead_of_crashing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);game,ctl=self.fixture(root)
            (root/'native-mods.json').write_text('[]')
            self.assertTrue(any('malformed' in reason for reason in preflight(game,ctl,None)))

    def test_preflight_success_does_not_launch_or_claim_gameplay(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);game,ctl=self.fixture(root)
            (root/'facts.json').write_text('{}')
            (root/'horizon-resources.json').write_text('{}')
            report=root/'report.json'
            run=subprocess.run([sys.executable,str(ROOT/'tools/core_validation.py'),'--game',str(game),
                                '--ctl',str(ctl),'--resources',str(root),'--preflight-only','--report',str(report)],
                               capture_output=True,text=True)
            self.assertEqual(run.returncode,0,run.stderr)
            data=json.loads(report.read_text())
            self.assertEqual(data['status'],'preflight_passed')
            self.assertFalse(data['gameplay_attempted'])
            self.assertFalse(data['gameplay_passed'])

    def test_blocked_cli_writes_report_and_nonzero_status(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);game,ctl=self.fixture(root);report=root/'report.json'
            run=subprocess.run([sys.executable,str(ROOT/'tools/core_validation.py'),'--game',str(game),
                                '--ctl',str(ctl),'--report',str(report)],capture_output=True,text=True)
            self.assertEqual(run.returncode,2,run.stderr)
            data=json.loads(report.read_text())
            self.assertEqual(data['status'],'blocked')
            self.assertFalse(data['gameplay_attempted'])
            self.assertFalse(data['gameplay_passed'])


if __name__=='__main__':
    unittest.main()
