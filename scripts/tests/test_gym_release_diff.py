"""[#4661][#5237] 릴리스 간 차등 회귀 도구 계약 — 오검출 관문·분류·관측 대조.

바이너리 두 개를 실제로 빌드하지 않고도 도구의 판정 논리를 고정한다. 관측
함수(run_cli)를 목으로 갈아끼워 "구/신이 다른 답을 냈을 때" 를 합성한다.
분류·종료 코드·보고 조립은 release_diff 의 순수 함수를 직접 부른다 — 시험이
같은 삼항을 복붙하면 가짜다(#5237).
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


def _tables_env(text="짐검증", row=0, col=0):
    return {"tables": [{"cells": [{"row": row, "col": col, "text": text}]}]}


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

    def test_keyerror_resolve_is_an_observation_not_a_crash(self):
        with mock.patch.object(self.rd.runner, "resolve_args",
                               side_effect=KeyError("input")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs, {"kind": "resolve-error", "error": "KeyError"})

    def test_typeerror_resolve_is_an_observation_not_a_crash(self):
        with mock.patch.object(self.rd.runner, "resolve_args",
                               side_effect=TypeError("cmd is None")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs, {"kind": "resolve-error", "error": "TypeError"})

    def test_indexerror_and_oserror_resolve_stay_observations(self):
        for exc in (IndexError("cmd[0]"), OSError("broken pipe")):
            with self.subTest(exc=type(exc).__name__), \
                    mock.patch.object(self.rd.runner, "resolve_args", side_effect=exc):
                obs = self.rd.observe("rhwp", self.check, self.task, ".")
            self.assertEqual(obs["kind"], "resolve-error")
            self.assertEqual(obs["error"], type(exc).__name__)

    def test_no_cmd_is_reported_without_calling_cli(self):
        called = []
        check = {"name": "빈검사", "op": "value_eq"}
        with mock.patch.object(self.rd.runner, "run_cli",
                               side_effect=lambda *a: called.append(a)):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "no-cmd"})
        self.assertEqual(called, [])

    def test_env_none_is_nojson_not_exit(self):
        """허용 종료인데 봉투가 없으면 파싱 실패(nojson) 다 — 종료 코드 문제가 아니다."""
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, None, "not json")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "nojson")
        self.assertEqual(obs["head"], "not json")

    def test_nojson_head_is_truncated_to_80(self):
        long_head = "x" * 200
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, None, long_head)):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "nojson")
        self.assertEqual(len(obs["head"]), 80)

    def test_empty_env_and_pagecount_path_is_digfail(self):
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, {}, "ok")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs, {"kind": "digfail", "error": "KeyError"})

    def test_dig_indexerror_is_digfail(self):
        check = dict(self.check, path="tables[0]")
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, {"tables": []}, "")):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "digfail", "error": "IndexError"})

    def test_dig_typeerror_is_digfail(self):
        check = dict(self.check, path="pageCount")
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, [1, 2, 3], "")):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "digfail", "error": "TypeError"})

    def test_empty_path_returns_whole_envelope(self):
        env = {"pageCount": 6, "extra": True}
        check = dict(self.check, path="")
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, env, "")):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "value", "value": env})

    def test_cell_text_eq_observes_named_cell_not_whole_table(self):
        check = {"name": "셀", "op": "cell_text_eq", "table": 0, "row": 0,
                 "col": 0, "value": "짐검증",
                 "cmd": ["export-tables", "{file:cell.hwp}", "--json"],
                 "path": "tables"}
        env = _tables_env("짐검증")
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, env, "")):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "value", "value": "짐검증"})

    def test_cell_text_eq_missing_cell_is_none_value(self):
        check = {"name": "셀", "op": "cell_text_eq", "table": 0, "row": 9,
                 "col": 9, "value": "없음",
                 "cmd": ["export-tables", "{file:cell.hwp}", "--json"],
                 "path": "tables"}
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, _tables_env("짐검증"), "")):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "value", "value": None})

    def test_cell_text_eq_cell_without_text_key_is_none(self):
        check = {"name": "셀", "op": "cell_text_eq", "table": 0, "row": 0,
                 "col": 0, "value": "x",
                 "cmd": ["export-tables", "{file:cell.hwp}", "--json"],
                 "path": "tables"}
        env = {"tables": [{"cells": [{"row": 0, "col": 0}]}]}
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(0, env, "")):
            obs = self.rd.observe("rhwp", check, self.task, ".")
        self.assertEqual(obs, {"kind": "value", "value": None})

    def test_file_ops_are_excluded_from_observation(self):
        """파일 존재/동일성은 관측이 아니라 상태라 raw 대조에서 빠진다."""
        self.assertIn("file_exists", self.rd.FILE_OPS)
        self.assertIn("same_hash", self.rd.FILE_OPS)
        self.assertIn("differs_from_input", self.rd.FILE_OPS)
        self.assertIn("files_differ", self.rd.FILE_OPS)

    def test_files_differ_is_a_file_op(self):
        self.assertIn("files_differ", self.rd.FILE_OPS)
        self.assertTrue(self.rd.FILE_OPS <= {
            "file_exists", "same_hash", "differs_from_input", "files_differ",
            "xml_root_eq", "json_value_eq", "csv_cell_eq", "utf8_bom",
        })


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

    def test_files_differ_check_is_skipped(self):
        task = {"id": "T", "input": "x.hwp",
                "checks": [{"name": "두 산출", "op": "files_differ",
                            "files": ["a.hwp", "b.hwp"]}]}
        called = []
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli",
                                  side_effect=lambda *a: called.append(a) or (0, {}, "")):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(rows, [])
        self.assertEqual(called, [], "files_differ 는 CLI 를 부르지 않아야 한다")

    def test_mixed_file_and_value_checks_only_observe_value(self):
        task = {"id": "T", "input": "x.hwp",
                "checks": [
                    {"name": "산출", "op": "files_differ", "files": ["a.hwp", "b.hwp"]},
                    {"name": "쪽수", "op": "value_eq", "value": 6,
                     "cmd": ["info", "{input}", "--json"], "path": "pageCount"},
                ]}
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli",
                                  return_value=(0, {"pageCount": 1}, "")):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(rows, [])

    def test_cell_text_eq_divergence_is_captured(self):
        check = {"name": "셀", "op": "cell_text_eq", "table": 0, "row": 0,
                 "col": 0, "value": "짐검증",
                 "cmd": ["export-tables", "{file:cell.hwp}", "--json"],
                 "path": "tables"}
        task = {"id": "T08", "input": "x.hwp", "checks": [check]}

        def fake(bin_path, args):
            return (0, _tables_env("구값" if bin_path == "old" else "신값"), "")

        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli", side_effect=fake):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["old"], {"kind": "value", "value": "구값"})
        self.assertEqual(rows[0]["new"], {"kind": "value", "value": "신값"})
        self.assertEqual(rows[0]["op"], "cell_text_eq")


class ClassificationTests(unittest.TestCase):
    """분류 규칙 — surface-changed 관문이 순수 회귀와 표면 변경을 가른다.

    여기의 삼항을 시험이 다시 적으면 #5237 가짜 시험이다. 반드시
    `rd.classify` 를 부른다.
    """

    def setUp(self):
        self.rd = load_rd()

    def test_classification_matrix(self):
        # (surfaceChanged, diffs) → 분류. diffs 는 main 이 넘기는 목록.
        cases = [
            (False, [], "stable"),
            (False, [{"task": "T"}], "regression"),
            (True, [], "surface-changed"),
            (True, [{"task": "T"}], "surface-changed"),  # 표면이 바뀌면 diff 여부와 무관
        ]
        for surface, diffs, expected in cases:
            self.assertEqual(
                self.rd.classify(surface, diffs), expected, (surface, diffs),
            )

    def test_surface_changed_wins_even_if_diffs_exist(self):
        self.assertEqual(
            self.rd.classify(True, [{"old": 1, "new": 2}]),
            "surface-changed",
        )

    def test_empty_diffs_without_surface_change_is_stable(self):
        self.assertEqual(self.rd.classify(False, []), "stable")
        self.assertEqual(self.rd.classify(False, None), "stable")
        self.assertEqual(self.rd.classify(False, False), "stable")

    def test_truthy_diffs_without_surface_change_is_regression(self):
        self.assertEqual(self.rd.classify(False, True), "regression")
        self.assertEqual(self.rd.classify(False, [0]), "regression")


class ExitCodeTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_exit_code_mapping(self):
        self.assertEqual(self.rd.exit_code("stable"), 0)
        self.assertEqual(self.rd.exit_code("regression"), 3)
        self.assertEqual(self.rd.exit_code("surface-changed"), 2)

    def test_unknown_classification_raises(self):
        with self.assertRaises(KeyError):
            self.rd.exit_code("unknown")

    def test_exit_code_follows_classify(self):
        pairs = [
            (False, [], 0),
            (False, [{"x": 1}], 3),
            (True, [], 2),
            (True, [{"x": 1}], 2),
        ]
        for surface, diffs, expected in pairs:
            cls = self.rd.classify(surface, diffs)
            self.assertEqual(self.rd.exit_code(cls), expected, (surface, diffs, cls))


class ReportContractTests(unittest.TestCase):
    """build_report 가 main 없이 봉투 키를 고정한다."""

    def setUp(self):
        self.rd = load_rd()

    def test_report_kind_and_schema(self):
        report = self.rd.build_report(
            "/opt/old/rhwp", "/opt/new/rhwp",
            "aaa", "bbb", False, 4, 12, [],
        )
        self.assertEqual(report["kind"], "gymReleaseDiff")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertEqual(report["old"], {"bin": "rhwp", "capabilitiesSha256": "aaa"})
        self.assertEqual(report["new"], {"bin": "rhwp", "capabilitiesSha256": "bbb"})
        self.assertFalse(report["surfaceChanged"])
        self.assertEqual(report["tasksCompared"], 4)
        self.assertEqual(report["observationsCompared"], 12)
        self.assertEqual(report["divergences"], 0)
        self.assertEqual(report["classification"], "stable")
        self.assertEqual(report["diffs"], [])

    def test_report_classification_uses_classify(self):
        diffs = [{"task": "T", "old": 1, "new": 2}]
        stable = self.rd.build_report("old", "new", "h1", "h1", False, 1, 1, [])
        regression = self.rd.build_report("old", "new", "h1", "h1", False, 1, 1, diffs)
        surface = self.rd.build_report("old", "new", "h1", "h2", True, 1, 1, diffs)
        self.assertEqual(stable["classification"], self.rd.classify(False, []))
        self.assertEqual(regression["classification"], self.rd.classify(False, diffs))
        self.assertEqual(surface["classification"], self.rd.classify(True, diffs))
        self.assertEqual(regression["divergences"], 1)
        self.assertTrue(surface["surfaceChanged"])

    def test_report_required_keys(self):
        report = self.rd.build_report("o", "n", "a", "b", False, 0, 0, [])
        self.assertEqual(set(report), {
            "kind", "schemaVersion", "old", "new", "surfaceChanged",
            "tasksCompared", "observationsCompared", "divergences",
            "classification", "diffs",
        })


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
        self.assertEqual(report["kind"], "gymReleaseDiff")
        self.assertEqual(self.rd.exit_code(report["classification"]), 0)


if __name__ == "__main__":
    unittest.main()
