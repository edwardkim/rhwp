#!/usr/bin/env python3
"""Portable process/log contracts, without simulating a Windows/Metal renderer."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1] / 'tool'
sys.path.insert(0, str(TOOLS))
spec = importlib.util.spec_from_file_location('windows_sdk_runner', TOOLS / 'check_windows_desktop_sdk.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class RunnerContracts(unittest.TestCase):
    def run_owned(self, source, timeout=5):
        with tempfile.TemporaryDirectory(prefix='rhwp-sdk-step-') as tmp:
            output = Path(tmp)
            receipt = {'steps': []}
            path = output / 'receipt.json'
            record = lambda: path.write_text(json.dumps(receipt))
            error = None
            try:
                module.run_step([sys.executable, '-c', source], cwd=output, env=dict(os.environ),
                                output=output, receipt=receipt, record=record, timeout=timeout)
            except BaseException as caught:
                error = caught
            saved = json.loads(path.read_text())
            log = (output / 'step-0.log').read_text()
            mode = (output / 'step-0.log').stat().st_mode & 0o777
            return saved['steps'][0], log, mode, error

    def test_success_is_recorded_after_owned_child_exits(self):
        step, log, mode, error = self.run_owned("print('actual-child-output')")
        self.assertIsNone(error)
        self.assertEqual(step['exitCode'], 0)
        self.assertIn('actual-child-output', log)
        self.assertGreaterEqual(step['finished'], step['started'])
        if os.name == 'posix':
            self.assertEqual(mode, 0o600)

    def test_failure_preserves_exit_and_completed_bounded_excerpt(self):
        step, log, _, error = self.run_owned("print('failed-child'); raise SystemExit(7)")
        self.assertEqual(step['exitCode'], 7)
        self.assertIsInstance(error, RuntimeError)
        self.assertIn('failed-child', str(error))
        self.assertIn('failed-child', log)

    def test_timeout_waits_for_child_termination_before_receipt(self):
        step, _, _, error = self.run_owned('import time; time.sleep(30)', timeout=0.1)
        self.assertIsInstance(error, subprocess.TimeoutExpired)
        self.assertTrue(step['interrupted'])
        self.assertIsNotNone(step['exitCode'])
        self.assertIn('finished', step)

    @unittest.skipIf(sys.platform == 'win32', 'real Windows is not a rejection case')
    def test_non_windows_execution_rejected_before_output_or_build(self):
        with tempfile.TemporaryDirectory(prefix='rhwp-windows-reject-') as tmp:
            output = Path(tmp) / 'not-created'
            result = subprocess.run([sys.executable, '-B', str(TOOLS / 'check_windows_desktop_sdk.py'),
                                     '--flutter-sdk', tmp, '--output', str(output)],
                                    capture_output=True, text=True, timeout=5)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('actual Windows', result.stderr)
            self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
