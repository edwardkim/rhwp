import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

WRAPPER = Path(__file__).resolve().parents[2] / 'scripts/wasm-pack-locked.sh'

class WrapperContract(unittest.TestCase):
    def run_case(self, mode='0', build=None, fail=False, custom=False, target_env=''):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'rhwp-studio/public').mkdir(parents=True)
            (root/'bin').mkdir()
            cargo=root/'bin/real-cargo'
            cargo.write_text('#!/usr/bin/env python3\nimport json,os,sys\nwith open(os.environ["CALLS"],"a") as f: f.write(json.dumps(sys.argv[1:])+"\\n")\nif sys.argv[1] in ("build","rustc") and os.environ["FAIL"]=="1": sys.exit(17)\n')
            pack=root/'bin/wasm-pack'
            pack.write_text('#!/usr/bin/env python3\nimport json,os,pathlib,subprocess\nsubprocess.run(["cargo","metadata","--format-version","1"],check=True)\nr=subprocess.run(["cargo",*json.loads(os.environ["BUILD"])])\nif r.returncode: raise SystemExit(r.returncode)\np=pathlib.Path(os.environ["OUT"]); p.mkdir()\nfor name in ("rhwp.js","rhwp_bg.wasm"): (p/name).write_text("fresh "+name)\n')
            for file in [cargo,pack]: file.chmod(0o755)
            env=dict(os.environ,CARGO_BUILD_TARGET=target_env,PATH=str(root/'bin')+os.pathsep+os.environ['PATH'],CARGO=str(cargo),CALLS=str(root/'calls'),FAIL=str(int(fail)),OUT='custom' if custom else 'pkg',RHWP_WASM_CDYLIB_ONLY=mode,BUILD=json.dumps(build or ['build','--lib','--release','--target','wasm32-unknown-unknown','--locked']))
            result=subprocess.run(['sh',str(WRAPPER),'--target','web','--release','--out-dir',env['OUT']],cwd=root,env=env,capture_output=True,text=True)
            calls=[json.loads(x) for x in (root/'calls').read_text().splitlines()] if (root/'calls').exists() else []
            synced=(root/'rhwp-studio/public/rhwp_bg.wasm').exists()
            return result,calls,synced

    def test_default_build_and_metadata_lock(self):
        result,calls,synced=self.run_case()
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(calls[0],['metadata','--format-version','1','--locked'])
        self.assertEqual(calls[1][0],'build'); self.assertTrue(synced)
        self.assertNotIn('--crate-type',calls[1])

    def test_opt_in_preserves_arguments_and_sync(self):
        result,calls,synced=self.run_case('1')
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(calls[1],['rustc','--lib','--release','--target','wasm32-unknown-unknown','--locked','--package','rhwp','--crate-type','cdylib','--config','profile.release.lto=false'])
        self.assertTrue(synced)

    def test_real_wasm_pack_target_environment(self):
        result,calls,synced=self.run_case('1',build=['build','--lib','--release','--locked','--message-format=json'],target_env='wasm32-unknown-unknown')
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertIn('--message-format=json',calls[1]); self.assertIn('rhwp',calls[1]); self.assertTrue(synced)

    def test_explicit_native_target_overrides_environment(self):
        result,_,synced=self.run_case('1',build=['build','--lib','--release','--target','x86_64-unknown-linux-gnu'],target_env='wasm32-unknown-unknown')
        self.assertEqual(result.returncode,2); self.assertFalse(synced)

    def test_failure_does_not_sync(self):
        result,_,synced=self.run_case('1',fail=True)
        self.assertEqual(result.returncode,17); self.assertFalse(synced)

    def test_other_output_does_not_sync(self):
        result,_,synced=self.run_case('1',custom=True)
        self.assertEqual(result.returncode,0); self.assertFalse(synced)

    def test_unsupported_invocations_are_rejected(self):
        for build in [
            ['build','--lib','--target','wasm32-unknown-unknown'],
            ['build','--lib','--release','--target','x86_64-unknown-linux-gnu'],
            ['build','--release','--target','wasm32-unknown-unknown'],
            ['build','--lib','--release','--target','wasm32-unknown-unknown','--config','profile.release.lto=true'],
        ]:
            with self.subTest(build=build):
                result,calls,synced=self.run_case('1',build=build)
                self.assertEqual(result.returncode,2); self.assertEqual(len(calls),1); self.assertFalse(synced)

    def test_invalid_mode_is_rejected(self):
        result,calls,synced=self.run_case('yes')
        self.assertEqual(result.returncode,2); self.assertEqual(calls,[]); self.assertFalse(synced)

if __name__=='__main__': unittest.main()
