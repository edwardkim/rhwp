"""[#4586] gym 판정 종료 코드와 T12 HWPX 형식 계약 회귀 테스트."""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[2]
SCORE_PATH = REPO_ROOT / "gym" / "score.py"
T12_PATH = REPO_ROOT / "gym" / "tasks" / "T12.json"
T12_BASELINE = REPO_ROOT / "gym" / "baselines" / "claude-fable-5" / "T12"


def load_score_module():
    spec = importlib.util.spec_from_file_location("gym_score_issue_4586_test", SCORE_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ExitVerdictContractTests(unittest.TestCase):
    def setUp(self):
        self.score = load_score_module()
        self.task = {"input": "samples/field-01.hwp"}
        self.check = {
            "name": "변환물 IR 대조",
            "op": "answer_eq",
            "answer": "identical",
            "cmd": ["ir-diff", "{input}", "{file:conv.hwpx}", "--json"],
            "path": "identical",
            "expect_exits": [0, 3],
        }

    def test_exit_3_false_verdict_is_compared_instead_of_discarded(self):
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score,
            "run_cli",
            return_value=(3, {"identical": False, "diffCount": 6}, ""),
        ):
            detail = self.score.eval_check(
                self.check,
                self.task,
                sub_dir,
                {"identical": False},
                "rhwp",
            )

        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["expected"], False)
        self.assertEqual(detail["actual"], False)

    def test_exit_outside_allowed_set_is_rejected_with_allowed_values(self):
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score,
            "run_cli",
            return_value=(1, None, ""),
        ):
            detail = self.score.eval_check(
                self.check,
                self.task,
                sub_dir,
                {"identical": False},
                "rhwp",
            )

        self.assertFalse(detail["ok"], detail)
        self.assertIn("0", detail["error"])
        self.assertIn("3", detail["error"])

    def test_legacy_expect_exit_contract_remains_compatible(self):
        legacy = dict(self.check)
        legacy.pop("expect_exits")
        legacy["expect_exit"] = 0
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score,
            "run_cli",
            return_value=(0, {"identical": True}, ""),
        ):
            detail = self.score.eval_check(
                legacy,
                self.task,
                sub_dir,
                {"identical": True},
                "rhwp",
            )

        self.assertTrue(detail["ok"], detail)


class T12TaskContractTests(unittest.TestCase):
    def test_t12_requires_real_hwpx_and_accepts_ir_verdict_exit(self):
        task = json.loads(T12_PATH.read_text(encoding="utf-8"))
        self.assertIn("export-hwpx", task["instructions"])
        self.assertNotIn("rhwp convert", task["instructions"])

        checks = {check["name"]: check for check in task["checks"]}
        format_check = checks["HWPX 형식 확인"]
        self.assertEqual(format_check["cmd"][0], "info")
        self.assertEqual(format_check["path"], "format")
        self.assertEqual(format_check["value"], "hwpx")

        diff_check = checks["변환물 IR 대조"]
        self.assertEqual(diff_check["expect_exits"], [0, 3])

    def test_t12_baseline_records_false_verdict_and_runner_identity(self):
        answer = json.loads((T12_BASELINE / "answer.json").read_text(encoding="utf-8"))
        verification = json.loads(
            (T12_BASELINE / "verification.json").read_text(encoding="utf-8")
        )

        self.assertEqual(answer, {"identical": False})
        self.assertEqual(verification["artifactFormat"], "hwpx")
        self.assertEqual(verification["answer"], answer)
        self.assertTrue(verification["result"]["pass"])
        self.assertEqual(len(verification["runner"]["rhwpCommit"]), 40)
        self.assertEqual(len(verification["runner"]["capabilitiesSha256"]), 64)


if __name__ == "__main__":
    unittest.main()
