from __future__ import annotations

import importlib.util
import io
import json
import subprocess
import tempfile
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from scripts import renderer_baseline as BASELINE


MODULE_PATH = Path(__file__).resolve().parents[1] / "canvaskit_parity_batches.py"
SPEC = importlib.util.spec_from_file_location("canvaskit_parity_batches", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"canvaskit_parity_batches 모듈을 불러올 수 없습니다: {MODULE_PATH}")
BATCHES = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = BATCHES
SPEC.loader.exec_module(BATCHES)


class BatchMapTests(unittest.TestCase):
    def test_supported_batches_match_parity_plan(self) -> None:
        self.assertEqual(
            BATCHES.BATCH_NAMES,
            {
                2: "Paint Family Parity",
                3: "Strict Text Variant Replay",
            },
        )

    def test_batch_two_covers_paint_family_harnesses(self) -> None:
        ids = [entry["id"] for entry in BATCHES.batch_jobs()[2]]
        self.assertIn("rust-canvaskit-policy", ids)
        self.assertIn("studio-renderer-contract", ids)
        self.assertIn("renderer-baseline-readiness", ids)

    def test_batch_three_covers_strict_text_variant_harnesses(self) -> None:
        ids = [entry["id"] for entry in BATCHES.batch_jobs()[3]]
        self.assertIn("rust-text-variants", ids)
        self.assertIn("studio-text-variant-selection", ids)
        self.assertIn("studio-renderer-contract", ids)

    def test_driver_never_points_at_manifest_as_output(self) -> None:
        for entries in BATCHES.batch_jobs().values():
            for entry in entries:
                joined = " ".join(entry["command"])
                self.assertNotIn("--write-manifest", joined)
                self.assertFalse(
                    any(
                        part.endswith("renderer_baseline_manifest.json")
                        and prev in {"-o", "--output"}
                        for prev, part in zip(entry["command"], entry["command"][1:])
                    )
                )

    def test_list_mode_prints_plan_and_no_manifest_rewrite(self) -> None:
        buffer = io.StringIO()
        with redirect_stdout(buffer):
            exit_code = BATCHES.main(["--list", "--batches", "2,3"])
        text = buffer.getvalue()
        self.assertEqual(exit_code, 0)
        self.assertIn("batch 2: Paint Family Parity", text)
        self.assertIn("batch 3: Strict Text Variant Replay", text)
        self.assertIn("manifest updates: never from this driver", text)

    def test_parse_batches_rejects_unknown(self) -> None:
        with self.assertRaises(SystemExit):
            BATCHES.parse_batches("1")


class RendererCliReuseTests(unittest.TestCase):
    def setUp(self):
        work_dir = Path(__file__).resolve().parents[2] / 'output' / 'renderer-baseline-contract'
        work_dir.mkdir(parents=True, exist_ok=True)
        self.original_root = BASELINE.ROOT
        self.tmp = tempfile.TemporaryDirectory(dir=work_dir)
        self.root = Path(self.tmp.name)
        BASELINE.ROOT = self.root
        BASELINE._cli_snapshots.clear()
        BASELINE._cli_snapshot_dir = None
        self.output = self.root / 'target/debug/rhwp'
        self.output.parent.mkdir(parents=True)
        self.builds = []
    def tearDown(self):
        if BASELINE._cli_snapshot_dir:
            BASELINE._cli_snapshot_dir.cleanup()
        BASELINE._cli_snapshots.clear()
        BASELINE._cli_snapshot_dir = None
        BASELINE.ROOT = self.original_root
        self.tmp.cleanup()
    def build(self, command, **kwargs):
        self.builds.append(command)
        self.output.write_text('native' if '--features' in command else 'default')
        self.output.chmod(0o755)
        record={'reason':'compiler-artifact','target':{'name':'rhwp','kind':['bin']},'executable':str(self.output)}
        return subprocess.CompletedProcess(command,0,json.dumps(record)+'\n')
    def cmd(self, native=False):
        return ['cargo','run',*(['--features','native-skia'] if native else []),'--bin','rhwp','--','export-png','with space.hwp','--page','0']
    def test_alternating_features_preserve_both_binaries_and_all_arguments(self):
        with patch.object(BASELINE.subprocess,'run',side_effect=self.build):
            results=[BASELINE.resolve_native_command(self.cmd(native),self.root,{}) for native in [False,True,False,True]]
        self.assertEqual(len(self.builds),2)
        self.assertEqual([Path(r[0]).read_text() for r in results],['default','native','default','native'])
        for r in results:
            self.assertEqual(r[1:],['export-png','with space.hwp','--page','0'])
            self.assertTrue(Path(r[0]).stat().st_mode & 0o100)
        self.assertNotIn('--release',self.builds[0])
        self.assertNotIn('--features',self.builds[0])
        self.assertIn('native-skia',self.builds[1])
    def test_failed_build_does_not_use_old_target_binary(self):
        self.output.write_text('stale')
        with patch.object(BASELINE.subprocess,'run',side_effect=subprocess.CalledProcessError(9,['cargo','build'])):
            with self.assertRaises(subprocess.CalledProcessError):
                BASELINE.resolve_native_command(self.cmd(),self.root,{})
        self.assertEqual(BASELINE._cli_snapshots,{})
    def test_missing_executable_is_fatal(self):
        with patch.object(BASELINE.subprocess,'run',return_value=subprocess.CompletedProcess([],0,'{}\n')):
            with self.assertRaises(RuntimeError):
                BASELINE.resolve_native_command(self.cmd(),self.root,{})
        self.assertEqual(BASELINE._cli_snapshots,{})
    def test_other_commands_and_environments_remain_separate(self):
        command=['node','capture.mjs','--profiles=print']
        with patch.object(BASELINE.subprocess,'run',side_effect=self.build):
            self.assertEqual(BASELINE.resolve_native_command(command,self.root,{}),command)
            BASELINE.resolve_native_command(self.cmd(),self.root,{'RUSTFLAGS':'-C opt-level=1'})
            BASELINE.resolve_native_command(self.cmd(),self.root,{'RUSTFLAGS':'-C opt-level=2'})
        self.assertEqual(len(self.builds),2)
    def test_runtime_failure_propagates(self):
        with patch.object(BASELINE,'resolve_native_command',return_value=['/saved/rhwp','export-svg']), patch.object(BASELINE.subprocess,'run',side_effect=subprocess.CalledProcessError(2,['/saved/rhwp'])):
            with self.assertRaises(subprocess.CalledProcessError):
                BASELINE.run_command(self.cmd(),self.root)


if __name__ == "__main__":
    unittest.main()
