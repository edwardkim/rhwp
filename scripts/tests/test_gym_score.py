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


class WrongTargetRegressionTests(unittest.TestCase):
    """[#4600] 잘못된 대상을 고친 제출이 통과하던 오검출의 음성 회귀.

    통과 제출만 검사하면 채점기는 "무엇이든 통과시키는" 방향으로 조용히
    썩는다. 여기서는 **반드시 실패해야 하는 제출**을 고정한다.
    """

    def setUp(self):
        self.score = load_score_module()

    # --- T07 서식 채움 — 첫 필드가 아니라 두 번째 필드를 채운 제출 ---

    T07_CHECK = {
        "name": "첫 필드 값이 정확히 홍길동",
        "op": "value_eq",
        "value": "홍길동",
        "cmd": ["fields", "{file:filled.hwp}", "--json"],
        "path": "fields[0].value",
    }

    def _fields_envelope(self, first_value, second_value):
        return {
            "fieldCount": 2,
            "fields": [
                {"name": "회사명", "value": first_value},
                {"name": "작성자", "value": second_value},
            ],
        }

    def test_t07_rejects_value_written_to_the_wrong_field(self):
        envelope = self._fields_envelope("", "홍길동")
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score, "run_cli", return_value=(0, envelope, "")
        ):
            detail = self.score.eval_check(self.T07_CHECK, {}, sub_dir, {}, "rhwp")

        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], "")

    def test_t07_accepts_value_written_to_the_first_field(self):
        envelope = self._fields_envelope("홍길동", "")
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score, "run_cli", return_value=(0, envelope, "")
        ):
            detail = self.score.eval_check(self.T07_CHECK, {}, sub_dir, {}, "rhwp")

        self.assertTrue(detail["ok"], detail)

    # --- T08 표 셀 교정 — (0,0) 이 아니라 (1,0) 을 고친 제출 ---

    T08_CHECK = {
        "name": "첫 표 (0,0) 셀이 정확히 짐검증",
        "op": "cell_text_eq",
        "table": 0,
        "row": 0,
        "col": 0,
        "value": "짐검증",
        "cmd": ["export-tables", "{file:cell.hwp}", "--json"],
        "path": "tables",
    }

    def _tables_envelope(self, first_cell, second_cell):
        return {
            "tableCount": 1,
            "tables": [
                {
                    "index": 0,
                    "cells": [
                        {"row": 0, "col": 0, "text": first_cell},
                        {"row": 1, "col": 0, "text": second_cell},
                    ],
                }
            ],
        }

    def test_t08_rejects_edit_applied_to_the_wrong_cell(self):
        envelope = self._tables_envelope("Ⅰ. 규제 심사(안) 개요", "짐검증")
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score, "run_cli", return_value=(0, envelope, "")
        ):
            detail = self.score.eval_check(self.T08_CHECK, {}, sub_dir, {}, "rhwp")

        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], "Ⅰ. 규제 심사(안) 개요")

    def test_t08_accepts_edit_applied_to_the_named_cell(self):
        envelope = self._tables_envelope("짐검증", "□ 요  약")
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score, "run_cli", return_value=(0, envelope, "")
        ):
            detail = self.score.eval_check(self.T08_CHECK, {}, sub_dir, {}, "rhwp")

        self.assertTrue(detail["ok"], detail)

    def test_t08_reports_missing_coordinate_instead_of_passing(self):
        """좌표가 없으면 조용히 통과하지 않고 actual=None 으로 실패한다."""
        envelope = {"tables": [{"index": 0, "cells": [{"row": 9, "col": 9, "text": "짐검증"}]}]}
        with tempfile.TemporaryDirectory() as sub_dir, mock.patch.object(
            self.score, "run_cli", return_value=(0, envelope, "")
        ):
            detail = self.score.eval_check(self.T08_CHECK, {}, sub_dir, {}, "rhwp")

        self.assertFalse(detail["ok"], detail)
        self.assertIsNone(detail["actual"])

    # --- T10 결정론 실증 — 원본을 복사만 한 제출 ---

    T10_CHECK = {"name": "원본 무편집 복사가 아님", "op": "differs_from_input", "file": "o1.hwp"}
    T10_TASK = {"input": "samples/field-01.hwp"}

    def test_t10_rejects_untouched_copy_of_the_input(self):
        source = REPO_ROOT / self.T10_TASK["input"]
        with tempfile.TemporaryDirectory() as sub_dir:
            (Path(sub_dir) / "o1.hwp").write_bytes(source.read_bytes())
            detail = self.score.eval_check(self.T10_CHECK, self.T10_TASK, sub_dir, {}, "rhwp")

        self.assertFalse(detail["ok"], detail)

    def test_t10_accepts_an_artifact_that_actually_changed(self):
        source = REPO_ROOT / self.T10_TASK["input"]
        with tempfile.TemporaryDirectory() as sub_dir:
            (Path(sub_dir) / "o1.hwp").write_bytes(source.read_bytes() + b"\x00")
            detail = self.score.eval_check(self.T10_CHECK, self.T10_TASK, sub_dir, {}, "rhwp")

        self.assertTrue(detail["ok"], detail)

    def test_submitted_hash_placeholder_feeds_the_live_oracle(self):
        """`{sha256:o1.hwp}` 는 채점 시점 해시로 풀려 replay 재현 판정에 넘어간다."""
        with tempfile.TemporaryDirectory() as sub_dir:
            artifact = Path(sub_dir) / "o1.hwp"
            artifact.write_bytes(b"gym")
            args = self.score.resolve_args(
                ["replay", "{file:plan.json}", "--expect-output-sha256", "{sha256:o1.hwp}"],
                {},
                sub_dir,
            )
            expected = self.score.sha256_of(str(artifact))

        self.assertEqual(args[-1], expected)
        self.assertEqual(len(args[-1]), 64)


class WeakCheckLockTests(unittest.TestCase):
    """[#4600] 경로 없는 전역 검사로 되돌아가는 것을 막는 과제 계약 잠금."""

    def _task(self, task_id):
        path = REPO_ROOT / "gym" / "tasks" / f"{task_id}.json"
        return json.loads(path.read_text(encoding="utf-8"))

    def test_t07_pins_the_first_field_instead_of_scanning_the_envelope(self):
        checks = self._task("T07")["checks"]
        self.assertTrue(all(c["op"] != "deep_contains" for c in checks), checks)
        self.assertEqual(checks[0]["path"], "fields[0].value")

    def test_t08_pins_the_named_cell_instead_of_scanning_the_envelope(self):
        checks = self._task("T08")["checks"]
        self.assertTrue(all(c["op"] != "deep_contains" for c in checks), checks)
        cell = checks[0]
        self.assertEqual(cell["op"], "cell_text_eq")
        self.assertEqual((cell["table"], cell["row"], cell["col"]), (0, 0, 0))

    def test_t10_proves_provenance_not_only_equality(self):
        task = self._task("T10")
        ops = [c["op"] for c in task["checks"]]
        self.assertIn("same_hash", ops)
        self.assertIn("differs_from_input", ops)
        self.assertIn("plan.json", task["submit"]["files"])

        replay = [c for c in task["checks"] if c.get("cmd", [""])[0] == "replay"]
        reproduce = [c for c in replay if c["path"] == "reproduced"]
        self.assertEqual(len(reproduce), 1, replay)
        self.assertIs(reproduce[0]["value"], True)
        self.assertIn("{sha256:o1.hwp}", reproduce[0]["cmd"])


class TaskCommandExistenceTests(unittest.TestCase):
    """[#4600 부수] 과제가 부르는 명령이 CLI 에 실재하는지 — T13 이 없는
    `harness-status` 를 불러 영구 실패하던 것을 잡은 가드."""

    def test_every_task_command_is_a_known_cli_command(self):
        tasks_dir = REPO_ROOT / "gym" / "tasks"
        called = set()
        for path in sorted(tasks_dir.glob("*.json")):
            task = json.loads(path.read_text(encoding="utf-8"))
            for check in task.get("checks", []):
                cmd = check.get("cmd")
                if cmd:
                    called.add((task["id"], cmd[0]))

        # 우산 명령의 하위는 cmd[1] 로 오므로 머리 토큰만 대조한다.
        head_tokens = {name for _, name in called}
        self.assertNotIn("harness-status", head_tokens)
        for task_id, name in sorted(called):
            self.assertRegex(name, r"^[a-z][a-z0-9-]*$", f"{task_id}: {name}")


if __name__ == "__main__":
    unittest.main()
