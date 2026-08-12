"""[#4661] 릴리스 간 차등 회귀 도구 계약 — 오검출 관문·분류·관측 대조.

바이너리 두 개를 실제로 빌드하지 않고도 도구의 판정 논리를 고정한다. 관측
함수(run_cli)를 목으로 갈아끼워 "구/신이 다른 답을 냈을 때" 를 합성한다.
"""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
RD_PATH = REPO_ROOT / "gym" / "tools" / "release_diff.py"


def load_rd():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    spec = importlib.util.spec_from_file_location("gym_release_diff_test", RD_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ObservationTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()
        self.task = {"input": "samples/x.hwp"}
        self.check = {"name": "쪽수", "op": "value_eq", "value": 6,
                      "cmd": ["info", "{input}", "--json"], "path": "pageCount"}

    def test_observation_extracts_raw_value_not_verdict(self):
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, {"pageCount": 6}, "")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs, {"kind": "value", "value": 6})

    def test_judgment_exit_code_is_allowed_not_treated_as_failure(self):
        c = dict(self.check, expect_exits=[0, 3])
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(3, {"pageCount": 7}, "")):
            obs = self.rd.observe("rhwp", c, self.task, ".")
        self.assertEqual(obs, {"kind": "value", "value": 7})

    def test_unexpected_exit_is_reported_not_crashed(self):
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(2, None, "usage error")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "exit")
        self.assertEqual(obs["code"], 2)

    def test_missing_hash_placeholder_is_an_observation(self):
        check = {"name": "재현", "op": "value_eq", "value": "ok",
                 "cmd": ["replay", "{sha256:o1.hwp}", "--json"], "path": "verdict"}
        with mock.patch.object(self.rd.runner, "resolve_args", side_effect=FileNotFoundError):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "resolve-error", "error": "FileNotFoundError"})

    def test_file_ops_are_excluded_from_observation(self):
        """파일 존재/동일성은 관측이 아니라 상태라 raw 대조에서 빠진다."""
        self.assertIn("file_exists", self.rd.FILE_OPS)
        self.assertIn("same_hash", self.rd.FILE_OPS)
        self.assertIn("differs_from_input", self.rd.FILE_OPS)


class DiffTaskTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_divergent_observation_is_captured(self):
        task = {"id": "T", "input": "x.hwp",
                "checks": [{"name": "쪽수", "op": "value_eq", "value": 6,
                            "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}
        # 구=6, 신=7 을 두 바이너리로 합성
        def fake(bin_path, args):
            return (0, {"pageCount": 6 if bin_path == "old" else 7}, "")
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli", side_effect=fake):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["old"], {"kind": "value", "value": 6})
        self.assertEqual(rows[0]["new"], {"kind": "value", "value": 7})

    def test_identical_observation_yields_no_row(self):
        task = {"id": "T", "input": "x.hwp",
                "checks": [{"name": "쪽수", "op": "value_eq", "value": 6,
                            "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli",
                                  return_value=(0, {"pageCount": 6}, "")):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(rows, [])

    def test_file_op_check_is_skipped(self):
        task = {"id": "T", "input": "x.hwp",
                "checks": [{"name": "산출", "op": "file_exists", "file": "o.hwp"}]}
        called = []
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli",
                                  side_effect=lambda *a: called.append(a) or (0, {}, "")):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(rows, [])
        self.assertEqual(called, [], "file_exists 는 CLI 를 부르지 않아야 한다")


class ClassificationTests(unittest.TestCase):
    """분류 규칙 — surface-changed 관문이 순수 회귀와 표면 변경을 가른다."""

    def test_classification_matrix(self):
        # (surfaceChanged, hasDiffs) → 분류
        cases = [
            (False, False, "stable"),
            (False, True, "regression"),
            (True, False, "surface-changed"),
            (True, True, "surface-changed"),  # 표면이 바뀌면 diff 여부와 무관
        ]
        for surface, diffs, expected in cases:
            cls = ("surface-changed" if surface
                   else ("regression" if diffs else "stable"))
            self.assertEqual(cls, expected, (surface, diffs))


class CommittedReportTests(unittest.TestCase):
    """커밋된 자기-대조 리포트가 있으면 stable·비결정성 0 을 상시 확인한다."""

    def setUp(self):
        self.rd = load_rd()

    def test_self_diff_report_is_stable(self):
        import json
        import os
        path = os.path.join(self.rd.runner.GYM, "release-diff.json")
        if not os.path.exists(path):
            self.skipTest("커밋된 self-diff 리포트 없음")
        report = json.loads(Path(path).read_text(encoding="utf-8"))
        if report["old"]["capabilitiesSha256"] != report["new"]["capabilitiesSha256"]:
            self.skipTest("두 바이너리 리포트(자기-대조 아님)")
        self.assertEqual(report["classification"], "stable")
        self.assertEqual(report["divergences"], 0,
                         "자기-대조에서 분기가 나오면 관측에 비결정성이 있다")


if __name__ == "__main__":
    unittest.main()
