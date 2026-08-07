"""P0 하니스의 오라클 격리·재사용 계약을 빠르게 검증한다."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from compare import selected_oracle_paths
from extract_spec import verify
from oracle_version import matches_expected_version
from run_gate import hwp_pids, new_hwp_pids, stored_oracle_status, wait_for_hwp_exit
from runner_ocx import clear_previous_outputs


class HarnessContractTests(unittest.TestCase):
    def test_hancom_2022_version_spellings_use_the_same_major(self) -> None:
        self.assertTrue(matches_expected_version("12, 0, 0, 4547", "12"))
        self.assertTrue(matches_expected_version("12.0.0.4547", "12,"))
        self.assertFalse(matches_expected_version("13.0.0.1", "12"))
        self.assertFalse(matches_expected_version(None, "12"))

    def test_previous_oracle_outputs_are_removed_before_a_new_run(self) -> None:
        scenario = {"id": "field-read", "saveAs": "nested/result.hwp"}
        with tempfile.TemporaryDirectory() as directory:
            out_dir = Path(directory)
            for relative in ("field-read.returns.json", "field-read.rejected.json", "nested/result.hwp"):
                path = out_dir / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("old", encoding="utf-8")
            returns, rejected, saved = clear_previous_outputs(scenario, out_dir)
            self.assertFalse(returns.exists())
            self.assertFalse(rejected.exists())
            self.assertIsNotNone(saved)
            self.assertFalse(saved.exists())

    def test_oracle_output_cannot_escape_the_requested_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ValueError):
                clear_previous_outputs({"id": "escape", "saveAs": "../outside.hwp"}, Path(directory))

    def test_skip_ocx_rejects_wrong_or_invalid_oracle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "field-read.returns.json"
            path.write_text(json.dumps({"oracle": {"version": "12.0.0.4547"}}), encoding="utf-8")
            self.assertEqual(stored_oracle_status(path, "12"), "SKIPPED")
            path.write_text(json.dumps({"oracle": {"version": "13.0.0.1"}}), encoding="utf-8")
            self.assertEqual(stored_oracle_status(path, "12"), "STALE_ORACLE")
            path.write_text("not-json", encoding="utf-8")
            self.assertEqual(stored_oracle_status(path, "12"), "INVALID_ORACLE")

    def test_compare_only_reads_explicitly_eligible_oracles(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            out_dir = Path(directory)
            for name in ("fresh", "stale"):
                (out_dir / f"{name}.returns.json").write_text("{}", encoding="utf-8")
            self.assertEqual(
                [path.name for path in selected_oracle_paths(out_dir, ["fresh"])],
                ["fresh.returns.json"],
            )

    def test_tasklist_parser_selects_only_hancom_processes(self) -> None:
        tasklist = 'Hwp.exe,101,Console,1,10 K\r\nHwpFrame.exe,202,Console,1,10 K\r\nnode.exe,303,Console,1,10 K\r\n'
        self.assertEqual(hwp_pids(tasklist), {"101", "202"})

    def test_new_process_detection_does_not_terminate_anything(self) -> None:
        from unittest.mock import patch

        tasklist = 'Hwp.exe,101,Console,1,10 K\r\nHwpFrame.exe,202,Console,1,10 K\r\n'
        with patch("run_gate.hwp_pids", return_value=hwp_pids(tasklist)):
            self.assertEqual(new_hwp_pids({"101"}), {"202"})

    def test_quit_settle_waits_for_asynchronous_hancom_exit(self) -> None:
        from unittest.mock import patch

        with patch("run_gate.new_hwp_pids", side_effect=[{"202"}, set()]), patch("run_gate.time.sleep") as sleep:
            self.assertEqual(wait_for_hwp_exit({"101"}, 10.0), set())
        sleep.assert_called_once()

    def test_tracked_spec_keeps_all_declared_parameter_set_items(self) -> None:
        spec_dir = HERE.parents[1] / "npm" / "hwpctrl-ocx" / "spec"
        api = json.loads((spec_dir / "webhwpctrl_api.json").read_text(encoding="utf-8"))["entries"]
        sets = json.loads((spec_dir / "parameter_sets.json").read_text(encoding="utf-8"))["sets"]
        actions = json.loads((spec_dir / "actions.json").read_text(encoding="utf-8"))["actions"]
        self.assertEqual(verify(api, sets, actions), [])


if __name__ == "__main__":
    unittest.main()
