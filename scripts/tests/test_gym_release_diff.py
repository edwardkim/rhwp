"""[#4661/#5234] 릴리스 간 차등 회귀 도구 계약 — 오검출 관문·분류·관측 대조.

바이너리 두 개를 실제로 빌드하지 않고도 도구의 판정 논리를 고정한다. 분류·
관측 동일성·보고 조립은 순수 함수를 직접 부르고, 관측 함수(run_cli)는 목으로
갈아끼워 "구/신이 다른 답을 냈을 때" 를 합성한다.
"""

from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
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


def _value(v):
    return {"kind": "value", "value": v}


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
        self.assertEqual(obs, _value(6))

    def test_judgment_exit_code_is_allowed_not_treated_as_failure(self):
        c = dict(self.check, expect_exits=[0, 3])
        with mock.patch.object(self.rd.runner, "run_cli",
                               return_value=(3, {"pageCount": 7}, "")):
            obs = self.rd.observe("rhwp", c, self.task, ".")
        self.assertEqual(obs, _value(7))

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
        self.assertIn("files_differ", self.rd.FILE_OPS)
        for op in self.rd.FILE_OPS:
            self.assertFalse(self.rd.should_observe({"op": op}), op)
        self.assertTrue(self.rd.should_observe({"op": "value_eq"}))

    def test_no_cmd_is_an_observation_without_cli(self):
        called = []
        with mock.patch.object(self.rd.runner, "run_cli",
                               side_effect=lambda *a: called.append(a)):
            obs = self.rd.observe("rhwp", {"op": "value_eq"}, self.task, ".")
        self.assertEqual(obs, {"kind": "no-cmd"})
        self.assertEqual(called, [])

    def test_nojson_when_exit_ok_but_envelope_missing(self):
        obs = self.rd.observation_from_result(0, None, "not json", self.check)
        self.assertEqual(obs["kind"], "nojson")
        self.assertEqual(obs["head"], "not json")

    def test_digfail_when_path_missing(self):
        obs = self.rd.observation_from_result(0, {"other": 1}, "", self.check)
        self.assertEqual(obs, {"kind": "digfail", "error": "KeyError"})

    def test_cell_text_eq_observes_coordinate_not_whole_table(self):
        check = {"name": "칸", "op": "cell_text_eq", "table": 0, "row": 1, "col": 2,
                 "cmd": ["export-tables", "{input}", "--json"], "path": "tables"}
        env = {"tables": [{"cells": [
            {"row": 0, "col": 0, "text": "아님"},
            {"row": 1, "col": 2, "text": "대상"},
        ]}]}
        obs = self.rd.observation_from_result(0, env, "", check)
        self.assertEqual(obs, _value("대상"))

    def test_cell_text_eq_missing_cell_is_none_value(self):
        check = {"op": "cell_text_eq", "table": 0, "row": 9, "col": 9,
                 "path": "tables"}
        env = {"tables": [{"cells": [{"row": 0, "col": 0, "text": "가"}]}]}
        self.assertEqual(self.rd.observation_from_result(0, env, "", check),
                         _value(None))

    def test_cell_text_eq_bad_table_index_is_digfail(self):
        check = {"op": "cell_text_eq", "table": 3, "row": 0, "col": 0,
                 "path": "tables"}
        env = {"tables": [{"cells": [{"row": 0, "col": 0, "text": "가"}]}]}
        obs = self.rd.observation_from_result(0, env, "", check)
        self.assertEqual(obs["kind"], "digfail")
        self.assertEqual(obs["error"], "IndexError")

    def test_expected_exits_falls_back_to_zero(self):
        self.assertEqual(self.rd.expected_exits({}), [0])
        self.assertEqual(self.rd.expected_exits({"expect_exit": 3}), [3])
        self.assertEqual(self.rd.expected_exits({"expect_exits": [0, 3]}), [0, 3])


class ObservationEqualityTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_int_and_float_same_number_are_equal(self):
        self.assertTrue(self.rd.observations_equal(_value(6), _value(6.0)))
        self.assertTrue(self.rd.observations_equal(_value(6), _value(6)))

    def test_bool_does_not_collapse_to_int(self):
        self.assertFalse(self.rd.observations_equal(_value(True), _value(1)))
        self.assertTrue(self.rd.observations_equal(_value(True), _value(True)))

    def test_kind_mismatch_is_not_equal_even_if_display_collides(self):
        self.assertFalse(self.rd.observations_equal(
            {"kind": "nojson", "head": "x"}, _value("nojson")))
        self.assertFalse(self.rd.observations_equal(
            {"kind": "exit", "code": 1, "head": ""}, _value("exit1")))

    def test_dict_key_order_does_not_matter(self):
        self.assertTrue(self.rd.observations_equal(
            {"kind": "value", "value": {"b": 1, "a": 2}},
            {"value": {"a": 2.0, "b": 1}, "kind": "value"},
        ))

    def test_display_keeps_cli_shape(self):
        self.assertEqual(self.rd.observation_display(_value(6)), 6)
        self.assertEqual(self.rd.observation_display(
            {"kind": "exit", "code": 2, "head": "e"}), "exit2")
        self.assertEqual(self.rd.observation_display(
            {"kind": "nojson", "head": "x"}), "nojson")
        self.assertEqual(self.rd.observation_display(
            {"kind": "digfail", "error": "KeyError"}), "digfail")


class DiffTaskTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_divergent_observation_is_captured(self):
        task = {"id": "T", "input": "x.hwp",
                "checks": [{"name": "쪽수", "op": "value_eq", "value": 6,
                            "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}
        def fake(bin_path, args):
            return (0, {"pageCount": 6 if bin_path == "old" else 7}, "")
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli", side_effect=fake):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["old"], _value(6))
        self.assertEqual(rows[0]["new"], _value(7))
        self.assertEqual(rows[0]["task"], "T")
        self.assertEqual(rows[0]["check"], "쪽수")

    def test_identical_observation_yields_no_row(self):
        task = {"id": "T", "input": "x.hwp",
                "checks": [{"name": "쪽수", "op": "value_eq", "value": 6,
                            "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli",
                                  return_value=(0, {"pageCount": 6}, "")):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(rows, [])

    def test_numeric_int_float_is_not_a_divergence(self):
        """6 과 6.0 을 회귀로 오신고하지 않는다."""
        task = {"id": "T", "input": "x.hwp",
                "checks": [{"name": "쪽수", "op": "value_eq",
                            "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}
        def fake(bin_path, args):
            return (0, {"pageCount": 6 if bin_path == "old" else 6.0}, "")
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli", side_effect=fake):
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

    def setUp(self):
        self.rd = load_rd()

    def test_classification_matrix(self):
        cases = [
            (False, False, "stable"),
            (False, True, "regression"),
            (True, False, "surface-changed"),
            (True, True, "surface-changed"),
        ]
        for surface, diffs, expected in cases:
            self.assertEqual(
                self.rd.classify(surface, diffs), expected, (surface, diffs))

    def test_empty_diff_list_is_stable_when_surface_same(self):
        self.assertEqual(self.rd.classify(False, []), "stable")
        self.assertEqual(self.rd.classify(False, 0), "stable")

    def test_nonempty_diff_list_is_regression_when_surface_same(self):
        self.assertEqual(self.rd.classify(False, [{"task": "T"}]), "regression")
        self.assertEqual(self.rd.classify(False, 4), "regression")

    def test_surface_changed_wins_even_with_no_diffs(self):
        self.assertEqual(self.rd.classify(True, []), "surface-changed")
        self.assertEqual(self.rd.classify(True, 0), "surface-changed")

    def test_surface_changed_from_digest(self):
        self.assertFalse(self.rd.surface_changed("aaa", "aaa"))
        self.assertTrue(self.rd.surface_changed("aaa", "bbb"))

    def test_exit_codes_match_gate_contract(self):
        self.assertEqual(self.rd.exit_for("stable"), 0)
        self.assertEqual(self.rd.exit_for("surface-changed"), 2)
        self.assertEqual(self.rd.exit_for("regression"), 3)
        self.assertEqual(
            {c: self.rd.exit_for(c) for c in self.rd.CLASSIFICATIONS},
            self.rd.EXIT_BY_CLASS,
        )

    def test_unknown_classification_has_no_exit(self):
        with self.assertRaises(KeyError):
            self.rd.exit_for("skipped")


class ReportTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def _row(self):
        return {
            "pack": "core-cli", "task": "T01", "check": "쪽수",
            "op": "value_eq", "path": "pageCount",
            "old": _value(6), "new": _value(7),
        }

    def test_stable_report_fields(self):
        report = self.rd.build_report(
            "/opt/old/rhwp", "aaa", "/opt/new/rhwp", "aaa",
            10, 20, [], observations_skipped=3,
        )
        self.assertEqual(report["kind"], "gymReleaseDiff")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertEqual(report["kind"], self.rd.REPORT_KIND)
        self.assertEqual(report["schemaVersion"], self.rd.SCHEMA_VERSION)
        self.assertEqual(report["classification"], "stable")
        self.assertEqual(report["classificationReason"],
                         self.rd.CLASSIFICATION_REASON["stable"])
        self.assertFalse(report["surfaceChanged"])
        self.assertEqual(report["divergences"], 0)
        self.assertEqual(report["tasksCompared"], 10)
        self.assertEqual(report["observationsCompared"], 20)
        self.assertEqual(report["observationsSkipped"], 3)
        self.assertEqual(report["exit"], 0)
        self.assertTrue(report["ok"])
        self.assertFalse(report["reviewRequired"])
        self.assertEqual(report["old"], {"bin": "rhwp", "capabilitiesSha256": "aaa"})
        self.assertEqual(report["new"], {"bin": "rhwp", "capabilitiesSha256": "aaa"})
        self.assertEqual(report["diffs"], [])

    def test_regression_report_when_surface_same_and_diffs(self):
        row = self._row()
        report = self.rd.build_report(
            "old.exe", "same", "new.exe", "same", 2, 5, [row],
        )
        self.assertEqual(report["classification"], "regression")
        self.assertIn("관측이 갈렸다", report["classificationReason"])
        self.assertFalse(report["surfaceChanged"])
        self.assertEqual(report["divergences"], 1)
        self.assertEqual(report["exit"], 3)
        self.assertFalse(report["ok"])
        self.assertFalse(report["reviewRequired"])
        self.assertEqual(report["diffs"][0]["task"], "T01")

    def test_surface_changed_even_when_observations_match(self):
        report = self.rd.build_report(
            "old", "aaa", "new", "bbb", 8, 12, [],
        )
        self.assertEqual(report["classification"], "surface-changed")
        self.assertTrue(report["surfaceChanged"])
        self.assertIn("capabilities digest", report["classificationReason"])
        self.assertEqual(report["exit"], 2)
        self.assertFalse(report["ok"])
        self.assertTrue(report["reviewRequired"])
        self.assertEqual(report["divergences"], 0)

    def test_surface_changed_with_diffs_is_still_review(self):
        report = self.rd.build_report(
            "old", "aaa", "new", "bbb", 1, 1, [self._row()],
        )
        self.assertEqual(report["classification"], "surface-changed")
        self.assertEqual(report["divergences"], 1)
        self.assertTrue(report["reviewRequired"])
        self.assertEqual(report["exit"], 2)

    def test_report_does_not_mutate_caller_diffs(self):
        diffs = [self._row()]
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, diffs)
        diffs.append({"task": "injected"})
        self.assertEqual(len(report["diffs"]), 1)

    def test_summary_and_report_file_are_deterministic(self):
        report = self.rd.build_report(
            "old/rhwp", "aaa", "new/rhwp", "aaa", 3, 4, [self._row()],
        )
        lines = self.rd.render_summary(report, "out.json")
        text = "\n".join(lines)
        self.assertIn("과제 3 · 관측 대조 4건", lines[0])
        self.assertIn("분류 [regression]", text)
        self.assertIn("이유:", text)
        self.assertIn("core-cli/T01", text)
        self.assertIn("6", text)
        self.assertIn("7", text)
        self.assertEqual(lines[-1], "→ out.json")
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "release-diff.json")
            self.rd.write_report(report, path)
            raw = Path(path).read_bytes()
            self.assertFalse(raw.startswith(b"\xef\xbb\xbf"))
            self.assertNotIn(b"\r\n", raw)
            loaded = json.loads(raw.decode("utf-8"))
        self.assertEqual(loaded, report)
        self.assertEqual(loaded["classification"], "regression")


class CommittedReportTests(unittest.TestCase):
    """커밋된 자기-대조 리포트가 있으면 stable·비결정성 0 을 상시 확인한다."""

    def setUp(self):
        self.rd = load_rd()

    def test_self_diff_report_is_stable(self):
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
