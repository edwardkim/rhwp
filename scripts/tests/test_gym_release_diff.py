"""[#4661/#5234] 릴리스 간 차등 회귀 도구 계약 — 오검출 관문·분류·관측 대조.

바이너리 두 개를 실제로 빌드하지 않고도 도구의 판정 논리를 고정한다. 분류·
관측 동일성·보고 조립은 순수 함수를 직접 부르고, 관측 함수(run_cli)는 목으로
갈아끼워 "구/신이 다른 답을 냈을 때" 를 합성한다.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
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
        self.assertEqual(obs["kind"], "resolve-error")
        self.assertEqual(obs["error"], "FileNotFoundError")

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


class ExceptionKindTests(unittest.TestCase):
    """예외 → 관측 kind 접기. 치명 예외는 접지 않는다."""

    def setUp(self):
        self.rd = load_rd()

    def test_timeout_kinds(self):
        self.assertEqual(self.rd.exception_kind(TimeoutError("x")), "timeout")
        expired = subprocess.TimeoutExpired(cmd=["rhwp"], timeout=1)
        self.assertEqual(self.rd.exception_kind(expired, context="digest"), "timeout")
        self.assertEqual(self.rd.exception_kind(expired, context="cli"), "timeout")

    def test_file_not_found_depends_on_context(self):
        err = FileNotFoundError("missing")
        self.assertEqual(self.rd.exception_kind(err, context="resolve"), "resolve-error")
        self.assertEqual(self.rd.exception_kind(err, context="digest"), "missing-bin")
        self.assertEqual(self.rd.exception_kind(err, context="cli"), "missing-bin")
        self.assertEqual(self.rd.exception_kind(err, context="observe"), "missing-bin")

    def test_permission_and_oserror(self):
        self.assertEqual(self.rd.exception_kind(PermissionError("no")), "permission")
        self.assertEqual(self.rd.exception_kind(OSError("io")), "os-error")

    def test_unicode_and_json(self):
        self.assertEqual(self.rd.exception_kind(UnicodeDecodeError("utf-8", b"\xff", 0, 1, "x")),
                         "decode-error")
        self.assertEqual(self.rd.exception_kind(UnicodeError("u")), "decode-error")
        self.assertEqual(self.rd.exception_kind(json.JSONDecodeError("e", "x", 0)), "value-error")

    def test_path_eval_kinds(self):
        self.assertEqual(self.rd.exception_kind(KeyError("k")), "digfail")
        self.assertEqual(self.rd.exception_kind(IndexError("i")), "digfail")
        self.assertEqual(self.rd.exception_kind(AttributeError("a")), "digfail")
        self.assertEqual(self.rd.exception_kind(TypeError("t")), "type-error")
        self.assertEqual(self.rd.exception_kind(ValueError("v")), "value-error")

    def test_runtime_and_unknown(self):
        self.assertEqual(self.rd.exception_kind(RuntimeError("r")), "cli-error")
        self.assertEqual(self.rd.exception_kind(Exception("e")), "unexpected")
        self.assertEqual(self.rd.exception_kind(None), "unexpected")

    def test_fatal_exceptions_are_flagged(self):
        self.assertTrue(self.rd.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(self.rd.is_fatal_exception(SystemExit(2)))
        self.assertTrue(self.rd.is_fatal_exception(MemoryError()))
        self.assertTrue(self.rd.is_fatal_exception(GeneratorExit()))
        self.assertFalse(self.rd.is_fatal_exception(OSError("x")))
        self.assertFalse(self.rd.is_fatal_exception(ValueError("x")))

    def test_exception_observation_shape(self):
        obs = self.rd.exception_observation(FileNotFoundError("o1.hwp"), context="resolve")
        self.assertEqual(obs["kind"], "resolve-error")
        self.assertEqual(obs["error"], "FileNotFoundError")
        self.assertIn("o1.hwp", obs["head"])

    def test_exception_probe_shape(self):
        row = self.rd.exception_probe(FileNotFoundError("rhwp"), "/opt/old/rhwp", "old")
        self.assertEqual(row["role"], "old")
        self.assertEqual(row["bin"], "rhwp")
        self.assertEqual(row["kind"], "missing-bin")
        self.assertEqual(row["error"], "FileNotFoundError")


class TruncateAndDigestHelperTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_truncate_head_none_and_limit(self):
        self.assertEqual(self.rd.truncate_head(None), "")
        self.assertEqual(self.rd.truncate_head("abcdef", 3), "abc")
        self.assertEqual(self.rd.truncate_head("abcdef", 0), "")
        self.assertEqual(self.rd.truncate_head("abcdef", -1), "")
        self.assertEqual(self.rd.truncate_head(1234, 2), "12")
        self.assertEqual(self.rd.truncate_head("abcdef", "nope"), "abcdef"[:self.rd.HEAD_LIMIT])

    def test_normalize_timeout(self):
        self.assertEqual(self.rd.normalize_timeout(30), 30)
        self.assertEqual(self.rd.normalize_timeout("15"), 15)
        self.assertEqual(self.rd.normalize_timeout(0), 0)
        self.assertEqual(self.rd.normalize_timeout(-3), 0)
        self.assertEqual(self.rd.normalize_timeout(None), 0)
        self.assertEqual(self.rd.normalize_timeout("x"), 0)
        self.assertEqual(self.rd.normalize_timeout(1.9), 1)

    def test_sha256_bytes_rejects_non_bytes(self):
        self.assertEqual(self.rd.sha256_bytes(b""), hashlib.sha256(b"").hexdigest())
        self.assertEqual(self.rd.sha256_bytes(bytearray(b"ab")), hashlib.sha256(b"ab").hexdigest())
        self.assertEqual(self.rd.sha256_bytes(memoryview(b"ab")), hashlib.sha256(b"ab").hexdigest())
        with self.assertRaises(TypeError):
            self.rd.sha256_bytes("ab")
        with self.assertRaises(TypeError):
            self.rd.sha256_bytes(None)

    def test_is_sha256_hex(self):
        good = "a" * 64
        self.assertTrue(self.rd.is_sha256_hex(good))
        self.assertFalse(self.rd.is_sha256_hex("A" * 64))
        self.assertFalse(self.rd.is_sha256_hex("a" * 63))
        self.assertFalse(self.rd.is_sha256_hex("g" * 64))
        self.assertFalse(self.rd.is_sha256_hex(None))
        self.assertFalse(self.rd.is_sha256_hex(1))

    def test_normalize_digest(self):
        self.assertEqual(self.rd.normalize_digest("B" * 64), "b" * 64)
        self.assertIsNone(self.rd.normalize_digest("short"))
        self.assertIsNone(self.rd.normalize_digest(None))
        self.assertIsNone(self.rd.normalize_digest(12))

    def test_observation_kind_helpers(self):
        self.assertEqual(self.rd.observation_kind_of(_value(1)), "value")
        self.assertEqual(self.rd.observation_kind_of({"kind": "exit", "code": 2}), "exit")
        self.assertIsNone(self.rd.observation_kind_of({}))
        self.assertIsNone(self.rd.observation_kind_of("x"))
        self.assertTrue(self.rd.is_known_observation_kind("value"))
        self.assertTrue(self.rd.is_known_observation_kind("timeout"))
        self.assertFalse(self.rd.is_known_observation_kind("skipped"))
        self.assertFalse(self.rd.is_error_observation(_value(1)))
        self.assertTrue(self.rd.is_error_observation({"kind": "digfail", "error": "KeyError"}))


class ProbeCapabilitiesTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_probe_success_matches_capabilities_digest(self):
        stdout = b'{"commands":["info"]}'
        proc = mock.Mock(stdout=stdout, returncode=0)
        with mock.patch.object(self.rd.subprocess, "run", return_value=proc) as run:
            digest = self.rd.capabilities_digest("rhwp")
            probed = self.rd.probe_capabilities("rhwp")
        self.assertEqual(digest, hashlib.sha256(stdout).hexdigest())
        self.assertTrue(probed["ok"])
        self.assertEqual(probed["digest"], digest)
        self.assertEqual(probed["kind"], "digest")
        self.assertEqual(probed["exit"], 0)
        self.assertGreaterEqual(run.call_count, 2)

    def test_probe_empty_path_is_missing_bin(self):
        probed = self.rd.probe_capabilities("")
        self.assertFalse(probed["ok"])
        self.assertEqual(probed["kind"], "missing-bin")
        self.assertIsNone(probed["digest"])

    def test_probe_file_not_found(self):
        with mock.patch.object(self.rd.subprocess, "run", side_effect=FileNotFoundError("rhwp")):
            probed = self.rd.probe_capabilities("missing")
        self.assertFalse(probed["ok"])
        self.assertEqual(probed["kind"], "missing-bin")
        self.assertEqual(probed["error"], "FileNotFoundError")
        self.assertIsNone(probed["digest"])

    def test_probe_permission(self):
        with mock.patch.object(self.rd.subprocess, "run", side_effect=PermissionError("x")):
            probed = self.rd.probe_capabilities("rhwp")
        self.assertEqual(probed["kind"], "permission")
        self.assertFalse(probed["ok"])

    def test_probe_timeout(self):
        with mock.patch.object(
            self.rd.subprocess, "run",
            side_effect=subprocess.TimeoutExpired(cmd=["rhwp"], timeout=1),
        ):
            probed = self.rd.probe_capabilities("rhwp", timeout=1)
        self.assertEqual(probed["kind"], "timeout")
        self.assertFalse(probed["ok"])
        self.assertIsNone(probed["digest"])

    def test_probe_oserror(self):
        with mock.patch.object(self.rd.subprocess, "run", side_effect=OSError(22, "bad")):
            probed = self.rd.probe_capabilities("rhwp")
        self.assertEqual(probed["kind"], "os-error")
        self.assertFalse(probed["ok"])

    def test_probe_does_not_swallow_keyboardinterrupt(self):
        with mock.patch.object(self.rd.subprocess, "run", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                self.rd.probe_capabilities("rhwp")

    def test_probe_does_not_swallow_systemexit(self):
        with mock.patch.object(self.rd.subprocess, "run", side_effect=SystemExit(2)):
            with self.assertRaises(SystemExit):
                self.rd.probe_capabilities("rhwp")

    def test_capabilities_digest_timeout_kwarg(self):
        proc = mock.Mock(stdout=b"x", returncode=0)
        with mock.patch.object(self.rd.subprocess, "run", return_value=proc) as run:
            digest = self.rd.capabilities_digest("rhwp", timeout=5)
        self.assertEqual(digest, hashlib.sha256(b"x").hexdigest())
        kwargs = run.call_args.kwargs
        self.assertEqual(kwargs.get("timeout"), 5)

    def test_capabilities_digest_str_stdout(self):
        proc = mock.Mock(stdout="abc", returncode=0)
        with mock.patch.object(self.rd.subprocess, "run", return_value=proc):
            digest = self.rd.capabilities_digest("rhwp")
        self.assertEqual(digest, hashlib.sha256(b"abc").hexdigest())

    def test_can_classify_surface_requires_two_strings(self):
        self.assertTrue(self.rd.can_classify_surface("aaa", "bbb"))
        self.assertFalse(self.rd.can_classify_surface(None, "bbb"))
        self.assertFalse(self.rd.can_classify_surface("aaa", None))
        self.assertFalse(self.rd.can_classify_surface(None, None))
        self.assertFalse(self.rd.can_classify_surface(1, 1))


class ClassifyHonestyTests(unittest.TestCase):
    """classify 는 삼원만 낸다. 표면 모름을 삼원으로 위장하지 않는다."""

    def setUp(self):
        self.rd = load_rd()

    def test_classify_never_returns_probe_failed(self):
        for surface in (True, False, 1, 0, "", "x"):
            for divergences in (None, 0, 1, [], [{}], False, True, "", "row"):
                got = self.rd.classify(surface, divergences)
                self.assertIn(got, self.rd.CLASSIFICATIONS)
                self.assertNotEqual(got, self.rd.STATUS_PROBE_FAILED)

    def test_classify_matrix_extended_shapes(self):
        cases = [
            (False, None, "stable"),
            (False, 0, "stable"),
            (False, 0.0, "stable"),
            (False, "", "stable"),
            (False, [], "stable"),
            (False, {}, "stable"),
            (False, False, "stable"),
            (False, 1, "regression"),
            (False, -1, "regression"),
            (False, "x", "regression"),
            (False, [1], "regression"),
            (False, {"a": 1}, "regression"),
            (False, True, "regression"),
            (True, None, "surface-changed"),
            (True, 0, "surface-changed"),
            (True, [], "surface-changed"),
            (True, [1], "surface-changed"),
            (True, True, "surface-changed"),
            ("yes", [], "surface-changed"),
            (1, 0, "surface-changed"),
        ]
        for surface, divergences, expected in cases:
            self.assertEqual(
                self.rd.classify(surface, divergences), expected,
                (surface, divergences),
            )

    def test_surface_wins_over_every_divergence_shape(self):
        for divergences in (None, 0, 1, [], [1], {}, "x", True, False):
            self.assertEqual(
                self.rd.classify(True, divergences), "surface-changed", divergences)

    def test_same_surface_empty_is_never_regression(self):
        for empty in (None, 0, 0.0, "", [], {}, False):
            self.assertEqual(self.rd.classify(False, empty), "stable", empty)

    def test_classify_or_probe_failed_separates_unknown_surface(self):
        self.assertEqual(
            self.rd.classify_or_probe_failed("a", "a", []), "stable")
        self.assertEqual(
            self.rd.classify_or_probe_failed("a", "a", [1]), "regression")
        self.assertEqual(
            self.rd.classify_or_probe_failed("a", "b", []), "surface-changed")
        self.assertEqual(
            self.rd.classify_or_probe_failed("a", "b", [1]), "surface-changed")
        self.assertEqual(
            self.rd.classify_or_probe_failed(None, "b", [1]), "probe-failed")
        self.assertEqual(
            self.rd.classify_or_probe_failed("a", None, []), "probe-failed")
        self.assertEqual(
            self.rd.classify_or_probe_failed(None, None, []), "probe-failed")

    def test_status_exit_and_reason(self):
        self.assertEqual(self.rd.status_exit("stable"), 0)
        self.assertEqual(self.rd.status_exit("surface-changed"), 2)
        self.assertEqual(self.rd.status_exit("regression"), 3)
        self.assertEqual(self.rd.status_exit("probe-failed"), 1)
        self.assertIn("위장하지 않는다", self.rd.reason_for("probe-failed"))
        self.assertEqual(self.rd.reason_for("stable"),
                         self.rd.CLASSIFICATION_REASON["stable"])
        with self.assertRaises(KeyError):
            self.rd.status_exit("skipped")

    def test_classifications_tuple_is_exactly_three(self):
        self.assertEqual(self.rd.CLASSIFICATIONS,
                         ("stable", "regression", "surface-changed"))
        self.assertNotIn(self.rd.STATUS_PROBE_FAILED, self.rd.CLASSIFICATIONS)
        self.assertEqual(set(self.rd.EXIT_BY_CLASS), set(self.rd.CLASSIFICATIONS))


class ObservationEqualityEdgeTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_none_equals_none(self):
        self.assertTrue(self.rd.observations_equal(None, None))
        self.assertFalse(self.rd.observations_equal(None, _value(None)))

    def test_nan_equals_nan_but_not_number(self):
        nan = float("nan")
        self.assertTrue(self.rd.observations_equal(_value(nan), _value(nan)))
        self.assertFalse(self.rd.observations_equal(_value(nan), _value(0)))
        self.assertFalse(self.rd.observations_equal(_value(nan), _value(1)))

    def test_inf_and_negative_inf(self):
        inf = float("inf")
        self.assertTrue(self.rd.observations_equal(_value(inf), _value(inf)))
        self.assertFalse(self.rd.observations_equal(_value(inf), _value(-inf)))
        self.assertFalse(self.rd.observations_equal(_value(inf), _value(1e308)))

    def test_nested_list_and_tuple(self):
        self.assertTrue(self.rd.observations_equal(
            _value([1, [2, 3.0]]), _value([1.0, [2, 3]])))
        self.assertFalse(self.rd.observations_equal(
            _value([1, 2]), _value([1, 2, 3])))
        self.assertTrue(self.rd._values_equal((1, 2.0), (1.0, 2)))
        self.assertFalse(self.rd._values_equal((1, 2), [1, 2]))

    def test_dict_key_mismatch_and_nested(self):
        self.assertFalse(self.rd.observations_equal(
            _value({"a": 1}), _value({"a": 1, "b": 2})))
        self.assertTrue(self.rd.observations_equal(
            _value({"a": {"b": 1}}), _value({"a": {"b": 1.0}})))
        self.assertFalse(self.rd.observations_equal(
            _value({"a": True}), _value({"a": 1})))

    def test_unicode_and_bytes_are_not_collapsed(self):
        self.assertTrue(self.rd.observations_equal(_value("한글"), _value("한글")))
        self.assertFalse(self.rd.observations_equal(_value("한글"), _value("한글 ")))
        self.assertFalse(self.rd._values_equal(b"ab", "ab"))

    def test_false_is_not_zero(self):
        self.assertFalse(self.rd.observations_equal(_value(False), _value(0)))
        self.assertFalse(self.rd.observations_equal(_value(False), _value(0.0)))
        self.assertTrue(self.rd.observations_equal(_value(False), _value(False)))

    def test_error_kinds_equal_only_with_same_payload(self):
        a = {"kind": "resolve-error", "error": "FileNotFoundError", "head": "x"}
        b = {"kind": "resolve-error", "error": "FileNotFoundError", "head": "x"}
        c = {"kind": "resolve-error", "error": "FileNotFoundError", "head": "y"}
        d = {"kind": "missing-bin", "error": "FileNotFoundError", "head": "x"}
        self.assertTrue(self.rd.observations_equal(a, b))
        self.assertFalse(self.rd.observations_equal(a, c))
        self.assertFalse(self.rd.observations_equal(a, d))

    def test_display_new_kinds(self):
        self.assertEqual(self.rd.observation_display(
            {"kind": "timeout", "error": "TimeoutError"}), "timeout")
        self.assertEqual(self.rd.observation_display(
            {"kind": "missing-bin", "error": "FileNotFoundError"}), "missing-bin")
        self.assertEqual(self.rd.observation_display(
            {"kind": "permission", "error": "PermissionError"}), "permission")
        self.assertEqual(self.rd.observation_display(
            {"kind": "resolve-error", "error": "OSError"}), "resolve-error")
        self.assertEqual(self.rd.observation_display("plain"), "plain")


class ObserveExceptionTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()
        self.task = {"input": "samples/x.hwp"}
        self.check = {"name": "쪽수", "op": "value_eq", "value": 6,
                      "cmd": ["info", "{input}", "--json"], "path": "pageCount"}

    def test_run_cli_timeout_is_observation(self):
        with mock.patch.object(
            self.rd.runner, "run_cli",
            side_effect=subprocess.TimeoutExpired(cmd=["rhwp"], timeout=1),
        ):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "timeout")
        self.assertEqual(obs["error"], "TimeoutExpired")

    def test_run_cli_missing_bin_is_observation(self):
        with mock.patch.object(self.rd.runner, "run_cli",
                               side_effect=FileNotFoundError("rhwp")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "missing-bin")
        self.assertEqual(obs["error"], "FileNotFoundError")

    def test_run_cli_permission_is_observation(self):
        with mock.patch.object(self.rd.runner, "run_cli",
                               side_effect=PermissionError("x")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "permission")

    def test_run_cli_oserror_is_observation(self):
        with mock.patch.object(self.rd.runner, "run_cli",
                               side_effect=OSError("broken pipe")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "os-error")

    def test_resolve_oserror_stays_resolve_error(self):
        with mock.patch.object(self.rd.runner, "resolve_args",
                               side_effect=OSError("noent")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "os-error")

    def test_resolve_keyerror_is_observation(self):
        with mock.patch.object(self.rd.runner, "resolve_args",
                               side_effect=KeyError("input")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "digfail")
        self.assertEqual(obs["error"], "KeyError")

    def test_resolve_typeerror_is_observation(self):
        with mock.patch.object(self.rd.runner, "resolve_args",
                               side_effect=TypeError("cmd")):
            obs = self.rd.observe("rhwp", self.check, self.task, ".")
        self.assertEqual(obs["kind"], "type-error")

    def test_non_dict_check_is_type_error(self):
        obs = self.rd.observe("rhwp", "not-a-check", self.task, ".")
        self.assertEqual(obs["kind"], "type-error")

    def test_observe_does_not_swallow_keyboardinterrupt_on_cli(self):
        with mock.patch.object(self.rd.runner, "run_cli", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                self.rd.observe("rhwp", self.check, self.task, ".")

    def test_observe_does_not_swallow_keyboardinterrupt_on_resolve(self):
        with mock.patch.object(self.rd.runner, "resolve_args", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                self.rd.observe("rhwp", self.check, self.task, ".")

    def test_dig_valueerror_is_digfail(self):
        check = dict(self.check, path="a[nope]")
        obs = self.rd.observation_from_result(0, {"a": [1]}, "", check)
        self.assertEqual(obs["kind"], "digfail")
        self.assertEqual(obs["error"], "ValueError")

    def test_dig_typeerror_is_digfail(self):
        check = dict(self.check, path="n.x")
        obs = self.rd.observation_from_result(0, {"n": 3}, "", check)
        self.assertEqual(obs["kind"], "digfail")

    def test_injected_dig_fn_exception_is_digfail(self):
        def boom(_env, _path):
            raise RuntimeError("dig")
        obs = self.rd.observation_from_result(0, {"pageCount": 1}, "", self.check, dig_fn=boom)
        self.assertEqual(obs["kind"], "digfail")
        self.assertEqual(obs["error"], "RuntimeError")

    def test_injected_find_cell_exception_is_digfail(self):
        check = {"op": "cell_text_eq", "table": 0, "row": 0, "col": 0, "path": "tables"}
        env = {"tables": [{"cells": []}]}

        def boom(*_a):
            raise TypeError("cell")
        obs = self.rd.observation_from_result(0, env, "", check, find_cell_fn=boom)
        self.assertEqual(obs["kind"], "digfail")
        self.assertEqual(obs["error"], "TypeError")

    def test_head_is_truncated_to_limit(self):
        long_head = "x" * 200
        obs = self.rd.observation_from_result(2, None, long_head, self.check)
        self.assertEqual(obs["kind"], "exit")
        self.assertEqual(len(obs["head"]), self.rd.HEAD_LIMIT)


class DiffTaskExceptionTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def _task(self, extra=None):
        checks = [{"name": "쪽수", "op": "value_eq",
                   "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]
        if extra:
            checks.extend(extra)
        return {"id": "T", "input": "x.hwp", "checks": checks}

    def test_cli_error_on_one_side_is_divergence(self):
        def fake(bin_path, args):
            if bin_path == "old":
                return (0, {"pageCount": 6}, "")
            raise FileNotFoundError("new")
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli", side_effect=fake):
            rows = self.rd.diff_task("old", "new", self._task(), "/sub", "pack")
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["old"], _value(6))
        self.assertEqual(rows[0]["new"]["kind"], "missing-bin")

    def test_same_cli_error_is_not_divergence(self):
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli",
                                  side_effect=FileNotFoundError("rhwp")):
            rows = self.rd.diff_task("old", "new", self._task(), "/sub", "pack")
        self.assertEqual(rows, [])

    def test_bad_task_shape_yields_empty(self):
        self.assertEqual(self.rd.diff_task("o", "n", None, "/s", "p"), [])
        self.assertEqual(self.rd.diff_task("o", "n", {"id": "T", "checks": "x"}, "/s", "p"), [])

    def test_file_op_mixed_with_value_only_compares_value(self):
        task = self._task(extra=[{"name": "산출", "op": "file_exists", "file": "o.hwp"}])
        called = []

        def fake(bin_path, args):
            called.append((bin_path, args))
            return (0, {"pageCount": 6}, "")
        with mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli", side_effect=fake):
            rows = self.rd.diff_task("old", "new", task, "/sub", "pack")
        self.assertEqual(rows, [])
        self.assertEqual(len(called), 2)

    def test_should_observe_rejects_non_dict(self):
        self.assertFalse(self.rd.should_observe(None))
        self.assertFalse(self.rd.should_observe("value_eq"))
        self.assertTrue(self.rd.should_observe({"op": "value_eq"}))

    def test_count_observable_checks(self):
        task = {"checks": [
            {"op": "value_eq"},
            {"op": "file_exists"},
            {"op": "cell_text_eq"},
            {"op": "same_hash"},
        ]}
        seen, skipped = self.rd.count_observable_checks(task)
        self.assertEqual(seen, 2)
        self.assertEqual(skipped, 2)
        self.assertEqual(self.rd.count_observable_checks(None), (0, 0))
        self.assertEqual(self.rd.count_observable_checks({"checks": None}), (0, 0))

    def test_resolve_submission_dir_pack_then_flat(self):
        with tempfile.TemporaryDirectory() as d:
            packed = os.path.join(d, "core-cli", "T01")
            os.makedirs(packed)
            got = self.rd.resolve_submission_dir(d, "core-cli", "T01")
            self.assertEqual(got, packed)
            flat_root = os.path.join(d, "flat")
            os.makedirs(os.path.join(flat_root, "T02"))
            got = self.rd.resolve_submission_dir(flat_root, "core-cli", "T02")
            self.assertEqual(got, os.path.join(flat_root, "T02"))
        self.assertEqual(self.rd.resolve_submission_dir("", "p", "T"), "")

    def test_resolve_submission_dir_oserror_is_not_fatal(self):
        with mock.patch("os.path.isdir", side_effect=OSError("x")):
            got = self.rd.resolve_submission_dir("/sub", "p", "T")
        self.assertTrue(got.endswith("T"))


class ReportHonestyTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def _row(self):
        return {
            "pack": "core-cli", "task": "T01", "check": "쪽수",
            "op": "value_eq", "path": "pageCount",
            "old": _value(6), "new": _value(7),
        }

    def test_build_report_without_string_digest_is_probe_failed(self):
        report = self.rd.build_report("old", None, "new", "aaa", 1, 1, [])
        self.assertEqual(report["classification"], "probe-failed")
        self.assertFalse(report["ok"])
        self.assertFalse(report["reviewRequired"])
        self.assertFalse(report["surfaceChanged"])
        self.assertEqual(report["exit"], 1)
        self.assertTrue(report["probeFailed"])
        self.assertEqual(self.rd.validate_report(report), [])

    def test_both_none_digests_are_not_stable(self):
        report = self.rd.build_report("old", None, "new", None, 0, 0, [])
        self.assertEqual(report["classification"], "probe-failed")
        self.assertNotEqual(report["classification"], "stable")
        self.assertFalse(report["ok"])

    def test_probe_failed_report_does_not_claim_surface_change(self):
        old_p = {"ok": False, "kind": "missing-bin", "error": "FileNotFoundError",
                 "digest": None, "head": "old"}
        new_p = {"ok": True, "kind": "digest", "digest": "a" * 64, "head": ""}
        report = self.rd.build_probe_failed_report("old/rhwp", old_p, "new/rhwp", new_p)
        self.assertEqual(report["classification"], "probe-failed")
        self.assertFalse(report["surfaceChanged"])
        self.assertEqual(len(report["probeErrors"]), 1)
        self.assertEqual(report["probeErrors"][0]["role"], "old")
        self.assertEqual(self.rd.validate_report(report), [])

    def test_empty_report_is_probe_failed(self):
        report = self.rd.empty_report("o", "n")
        self.assertEqual(report["classification"], "probe-failed")
        self.assertEqual(self.rd.validate_report(report), [])

    def test_validate_report_catches_ok_lie(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [self._row()])
        self.assertEqual(report["classification"], "regression")
        report["ok"] = True
        issues = self.rd.validate_report(report)
        self.assertTrue(any("ok" in i for i in issues))

    def test_validate_report_catches_review_lie(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [])
        report["reviewRequired"] = True
        issues = self.rd.validate_report(report)
        self.assertTrue(any("reviewRequired" in i for i in issues))

    def test_validate_report_catches_surface_lie(self):
        report = self.rd.build_report("o", "aaa", "n", "aaa", 1, 1, [])
        report["surfaceChanged"] = True
        issues = self.rd.validate_report(report)
        self.assertTrue(any("surfaceChanged" in i for i in issues))

    def test_validate_report_catches_probe_flag_on_stable(self):
        report = self.rd.build_report("o", "aaa", "n", "aaa", 1, 1, [])
        report["probeFailed"] = True
        issues = self.rd.validate_report(report)
        self.assertTrue(any("probeFailed" in i for i in issues))

    def test_validate_report_catches_divergence_count(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [self._row()])
        report["divergences"] = 0
        issues = self.rd.validate_report(report)
        self.assertTrue(any("divergences" in i for i in issues))

    def test_validate_report_rejects_unknown_class(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [])
        report["classification"] = "skipped"
        issues = self.rd.validate_report(report)
        self.assertTrue(any("알 수 없는" in i for i in issues))

    def test_validate_report_rejects_non_dict(self):
        self.assertEqual(self.rd.validate_report(None), ["report 가 dict 가 아니다"])

    def test_validate_report_accepts_honest_triple(self):
        for old_d, new_d, diffs, expected in (
            ("aaa", "aaa", [], "stable"),
            ("aaa", "aaa", [self._row()], "regression"),
            ("aaa", "bbb", [], "surface-changed"),
            ("aaa", "bbb", [self._row()], "surface-changed"),
        ):
            report = self.rd.build_report("o", old_d, "n", new_d, 2, 3, diffs)
            self.assertEqual(report["classification"], expected)
            self.assertEqual(self.rd.validate_report(report), [], expected)

    def test_stable_cannot_carry_diffs(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [])
        report["diffs"] = [self._row()]
        report["divergences"] = 1
        issues = self.rd.validate_report(report)
        self.assertTrue(any("stable" in i for i in issues))

    def test_regression_cannot_have_zero_diffs(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [self._row()])
        report["diffs"] = []
        report["divergences"] = 0
        issues = self.rd.validate_report(report)
        self.assertTrue(any("regression" in i for i in issues))


class WriteAndSummaryExceptionTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_write_report_safe_oserror(self):
        report = self.rd.build_report("o", "x", "n", "x", 0, 0, [])
        err = self.rd.write_report_safe(report, os.path.join("no-such-dir", "x", "out.json"))
        self.assertIsNotNone(err)
        self.assertIn("write_report", err)

    def test_write_report_safe_success(self):
        report = self.rd.build_report("o", "x", "n", "x", 0, 0, [])
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "out.json")
            self.assertIsNone(self.rd.write_report_safe(report, path))
            loaded = json.loads(Path(path).read_text(encoding="utf-8"))
        self.assertEqual(loaded["classification"], "stable")

    def test_render_summary_probe_failed(self):
        report = self.rd.empty_report("/opt/old/rhwp", "/opt/new/rhwp")
        lines = self.rd.render_summary(report, "out.json")
        text = "\n".join(lines)
        self.assertIn("프로브 실패", text)
        self.assertIn("probe-failed", text)
        self.assertNotIn("분류 [stable]", text)
        self.assertNotIn("분류 [regression]", text)
        self.assertEqual(lines[-1], "→ out.json")

    def test_render_summary_includes_pack_errors(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [])
        report["packErrors"] = ["pack core-cli: FileNotFoundError: x"]
        report["writeError"] = "write_report: OSError: disk"
        lines = self.rd.render_summary(report, "out.json")
        text = "\n".join(lines)
        self.assertIn("pack 오류", text)
        self.assertIn("쓰기 오류", text)

    def test_render_summary_surface_changed(self):
        report = self.rd.build_report("o", "aaa", "n", "bbb", 2, 2, [])
        lines = self.rd.render_summary(report, "p.json")
        text = "\n".join(lines)
        self.assertIn("surface-changed", text)
        self.assertIn("다름", text)


class PackLoadSafeTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_load_pack_safe_oserror(self):
        with mock.patch.object(self.rd.runner, "load_pack", side_effect=FileNotFoundError("p")):
            manifest, tasks, err = self.rd.load_pack_safe("core-cli")
        self.assertIsNone(manifest)
        self.assertEqual(tasks, [])
        self.assertIn("core-cli", err)
        self.assertIn("FileNotFoundError", err)

    def test_load_pack_safe_json_error(self):
        with mock.patch.object(self.rd.runner, "load_pack",
                               side_effect=json.JSONDecodeError("e", "x", 0)):
            _m, tasks, err = self.rd.load_pack_safe("broken")
        self.assertEqual(tasks, [])
        self.assertIn("JSONDecodeError", err)

    def test_load_pack_safe_success(self):
        with mock.patch.object(self.rd.runner, "load_pack",
                               return_value=({"id": "p"}, [{"id": "T"}])):
            manifest, tasks, err = self.rd.load_pack_safe("p")
        self.assertEqual(manifest["id"], "p")
        self.assertEqual(len(tasks), 1)
        self.assertIsNone(err)

    def test_load_pack_safe_non_list_tasks(self):
        with mock.patch.object(self.rd.runner, "load_pack",
                               return_value=({"id": "p"}, None)):
            _m, tasks, err = self.rd.load_pack_safe("p")
        self.assertEqual(tasks, [])
        self.assertIn("list", err)

    def test_discover_packs_safe_explicit(self):
        packs, err = self.rd.discover_packs_safe(["core-cli", "security"])
        self.assertEqual(packs, ["core-cli", "security"])
        self.assertIsNone(err)

    def test_discover_packs_safe_oserror(self):
        with mock.patch.object(self.rd.runner, "discover_packs", side_effect=OSError("x")):
            packs, err = self.rd.discover_packs_safe(None)
        self.assertEqual(packs, [])
        self.assertIn("discover_packs", err)

    def test_compare_packs_skips_broken_pack(self):
        def loader(pack_id):
            if pack_id == "bad":
                raise OSError("nope")
            return {"id": pack_id}, [{"id": "T", "checks": [
                {"name": "쪽수", "op": "value_eq",
                 "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}]
        with mock.patch.object(self.rd.runner, "load_pack", side_effect=loader), \
                mock.patch("os.path.isdir", return_value=True), \
                mock.patch.object(self.rd.runner, "run_cli",
                                  return_value=(0, {"pageCount": 6}, "")):
            coll = self.rd.compare_packs("old", "new", ["bad", "good"], "/sub")
        self.assertEqual(coll["tasksCompared"], 1)
        self.assertEqual(len(coll["packErrors"]), 1)
        self.assertIn("bad", coll["packErrors"][0])
        self.assertEqual(coll["diffs"], [])

    def test_compare_packs_records_task_exception(self):
        with mock.patch.object(self.rd.runner, "load_pack",
                               return_value=({"id": "p"}, [{"id": "T", "checks": [
                                   {"op": "value_eq", "cmd": ["info"], "path": "x"}]}])), \
                mock.patch.object(self.rd, "diff_task", side_effect=RuntimeError("boom")):
            coll = self.rd.compare_packs("o", "n", ["p"], "/sub")
        self.assertEqual(len(coll["taskErrors"]), 1)
        self.assertIn("RuntimeError", coll["taskErrors"][0])

    def test_attach_collection_errors_does_not_change_class(self):
        report = self.rd.build_report("o", "x", "n", "x", 1, 1, [])
        self.rd.attach_collection_errors(report, {
            "packErrors": ["pack z"],
            "taskErrors": ["t"],
        })
        self.assertEqual(report["classification"], "stable")
        self.assertEqual(report["packErrors"], ["pack z"])
        self.assertEqual(report["taskErrors"], ["t"])


class MainEntryExceptionTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_parse_args_digest_timeout(self):
        ns = self.rd.parse_args(["--old", "o", "--new", "n", "--digest-timeout", "7"])
        self.assertEqual(ns.old, "o")
        self.assertEqual(ns.new, "n")
        self.assertEqual(ns.digest_timeout, 7)

    def test_main_probe_failed_does_not_claim_stable(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "r.json")
            with mock.patch.object(self.rd, "find_bin_safe", side_effect=[("old", None), ("new", None)]), \
                    mock.patch.object(self.rd, "probe_capabilities",
                                      return_value={"ok": False, "kind": "missing-bin",
                                                    "digest": None, "error": "FileNotFoundError",
                                                    "head": "x", "exit": None}), \
                    mock.patch.object(self.rd, "discover_packs_safe", return_value=([], None)), \
                    mock.patch.object(self.rd, "compare_packs", return_value={
                        "diffs": [], "tasksCompared": 0, "observationsCompared": 0,
                        "observationsSkipped": 0, "packErrors": [], "taskErrors": [],
                    }):
                code = self.rd.main(["--old", "o", "--new", "n", "-o", out])
            self.assertEqual(code, 1)
            report = json.loads(Path(out).read_text(encoding="utf-8"))
        self.assertEqual(report["classification"], "probe-failed")
        self.assertFalse(report["ok"])
        self.assertNotEqual(report["classification"], "stable")

    def test_main_stable_when_probes_and_obs_match(self):
        digest = "a" * 64
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "r.json")
            with mock.patch.object(self.rd, "find_bin_safe", side_effect=[("old", None), ("new", None)]), \
                    mock.patch.object(self.rd, "probe_capabilities",
                                      return_value={"ok": True, "kind": "digest",
                                                    "digest": digest, "error": None,
                                                    "head": "", "exit": 0}), \
                    mock.patch.object(self.rd, "discover_packs_safe",
                                      return_value=(["p"], None)), \
                    mock.patch.object(self.rd, "compare_packs", return_value={
                        "diffs": [], "tasksCompared": 2, "observationsCompared": 4,
                        "observationsSkipped": 1, "packErrors": [], "taskErrors": [],
                    }):
                code = self.rd.main(["--old", "o", "--new", "n", "-o", out])
            self.assertEqual(code, 0)
            report = json.loads(Path(out).read_text(encoding="utf-8"))
        self.assertEqual(report["classification"], "stable")
        self.assertTrue(report["ok"])
        self.assertEqual(self.rd.validate_report(report), [])

    def test_main_regression_exit_three(self):
        digest = "b" * 64
        row = {"pack": "p", "task": "T", "check": "쪽수", "op": "value_eq",
               "path": "pageCount", "old": _value(6), "new": _value(7)}
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "r.json")
            with mock.patch.object(self.rd, "find_bin_safe", side_effect=[("old", None), ("new", None)]), \
                    mock.patch.object(self.rd, "probe_capabilities",
                                      return_value={"ok": True, "kind": "digest",
                                                    "digest": digest, "error": None,
                                                    "head": "", "exit": 0}), \
                    mock.patch.object(self.rd, "discover_packs_safe",
                                      return_value=(["p"], None)), \
                    mock.patch.object(self.rd, "compare_packs", return_value={
                        "diffs": [row], "tasksCompared": 1, "observationsCompared": 1,
                        "observationsSkipped": 0, "packErrors": [], "taskErrors": [],
                    }):
                code = self.rd.main(["--old", "o", "--new", "n", "-o", out])
            report = json.loads(Path(out).read_text(encoding="utf-8"))
        self.assertEqual(code, 3)
        self.assertEqual(report["classification"], "regression")
        self.assertFalse(report["surfaceChanged"])

    def test_main_surface_changed_exit_two(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "r.json")
            probes = [
                {"ok": True, "kind": "digest", "digest": "a" * 64, "error": None, "head": "", "exit": 0},
                {"ok": True, "kind": "digest", "digest": "b" * 64, "error": None, "head": "", "exit": 0},
            ]
            with mock.patch.object(self.rd, "find_bin_safe", side_effect=[("old", None), ("new", None)]), \
                    mock.patch.object(self.rd, "probe_capabilities", side_effect=probes), \
                    mock.patch.object(self.rd, "discover_packs_safe",
                                      return_value=(["p"], None)), \
                    mock.patch.object(self.rd, "compare_packs", return_value={
                        "diffs": [], "tasksCompared": 1, "observationsCompared": 1,
                        "observationsSkipped": 0, "packErrors": [], "taskErrors": [],
                    }):
                code = self.rd.main(["--old", "o", "--new", "n", "-o", out])
            report = json.loads(Path(out).read_text(encoding="utf-8"))
        self.assertEqual(code, 2)
        self.assertEqual(report["classification"], "surface-changed")
        self.assertTrue(report["reviewRequired"])

    def test_main_write_error_is_recorded_not_raised(self):
        digest = "c" * 64
        with mock.patch.object(self.rd, "find_bin_safe", side_effect=[("old", None), ("new", None)]), \
                mock.patch.object(self.rd, "probe_capabilities",
                                  return_value={"ok": True, "kind": "digest",
                                                "digest": digest, "error": None,
                                                "head": "", "exit": 0}), \
                mock.patch.object(self.rd, "discover_packs_safe", return_value=([], None)), \
                mock.patch.object(self.rd, "compare_packs", return_value={
                    "diffs": [], "tasksCompared": 0, "observationsCompared": 0,
                    "observationsSkipped": 0, "packErrors": [], "taskErrors": [],
                }), \
                mock.patch.object(self.rd, "write_report_safe", return_value="write_report: OSError"):
            code = self.rd.main(["--old", "o", "--new", "n", "-o", "unused.json"])
        self.assertEqual(code, 0)

    def test_find_bin_safe_oserror(self):
        with mock.patch.object(self.rd.runner, "find_bin", side_effect=OSError("x")):
            found, err = self.rd.find_bin_safe("rhwp")
        self.assertEqual(found, "rhwp")
        self.assertIn("find_bin", err)

    def test_find_bin_safe_success(self):
        with mock.patch.object(self.rd.runner, "find_bin", return_value="/abs/rhwp"):
            found, err = self.rd.find_bin_safe("rhwp")
        self.assertEqual(found, "/abs/rhwp")
        self.assertIsNone(err)


class CatalogContractTests(unittest.TestCase):
    """카탈로그·키 집합이 문서/시험과 같은 표를 보는지."""

    def setUp(self):
        self.rd = load_rd()

    def test_observation_kinds_cover_exception_table(self):
        for kind in (
            "value", "exit", "nojson", "digfail", "no-cmd", "resolve-error",
            "cli-error", "timeout", "missing-bin", "permission", "os-error",
            "type-error", "value-error", "decode-error", "unexpected",
        ):
            self.assertIn(kind, self.rd.OBSERVATION_KINDS)

    def test_file_ops_are_exactly_the_four(self):
        self.assertEqual(self.rd.FILE_OPS, {
            "file_exists", "same_hash", "differs_from_input", "files_differ",
        })

    def test_report_keys_include_honesty_fields(self):
        for key in (
            "classification", "classificationReason", "exit", "ok",
            "reviewRequired", "surfaceChanged", "divergences", "diffs",
        ):
            self.assertIn(key, self.rd.REPORT_KEYS)

    def test_report_kind_and_schema(self):
        self.assertEqual(self.rd.REPORT_KIND, "gymReleaseDiff")
        self.assertEqual(self.rd.SCHEMA_VERSION, "1.0")
        self.assertEqual(self.rd.DIGEST_TIMEOUT, 30)
        self.assertEqual(self.rd.HEAD_LIMIT, 80)
        self.assertEqual(self.rd.EXIT_PROBE_FAILED, 1)

    def test_classification_reasons_do_not_name_winner(self):
        for reason in self.rd.CLASSIFICATION_REASON.values():
            self.assertNotIn("옳", reason)
            self.assertNotIn("정답", reason)

    def test_probe_failed_reason_forbids_disguise(self):
        self.assertIn("위장하지 않는다", self.rd.PROBE_FAILED_REASON)
        self.assertIn("stable", self.rd.PROBE_FAILED_REASON)
        self.assertIn("regression", self.rd.PROBE_FAILED_REASON)
        self.assertIn("surface-changed", self.rd.PROBE_FAILED_REASON)


class CellAndExitEdgeTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_cell_missing_table_key_is_digfail(self):
        check = {"op": "cell_text_eq", "table": 0, "row": 0, "col": 0, "path": "tables"}
        env = {"tables": {"not": "a-list"}}
        obs = self.rd.observation_from_result(0, env, "", check)
        self.assertEqual(obs["kind"], "digfail")

    def test_cell_none_tables_is_digfail(self):
        check = {"op": "cell_text_eq", "table": 0, "row": 0, "col": 0, "path": "tables"}
        obs = self.rd.observation_from_result(0, {"tables": None}, "", check)
        self.assertEqual(obs["kind"], "digfail")

    def test_expected_exits_empty_list_falls_back(self):
        self.assertEqual(self.rd.expected_exits({"expect_exits": []}), [0])
        self.assertEqual(self.rd.expected_exits({"expect_exits": None}), [0])
        self.assertEqual(self.rd.expected_exits({"expect_exit": None}), [None])

    def test_exit_zero_allowed_by_default(self):
        check = {"op": "value_eq", "path": "n"}
        self.assertEqual(
            self.rd.observation_from_result(0, {"n": 1}, "", check), _value(1))

    def test_exit_three_not_allowed_without_expect(self):
        check = {"op": "value_eq", "path": "n"}
        obs = self.rd.observation_from_result(3, {"n": 1}, "judge", check)
        self.assertEqual(obs["kind"], "exit")
        self.assertEqual(obs["code"], 3)

    def test_make_diff_row_without_name_uses_op(self):
        row = self.rd.make_diff_row("T", {"op": "value_eq", "path": "x"}, _value(1), _value(2))
        self.assertEqual(row["check"], "value_eq")
        self.assertEqual(row["task"], "T")

    def test_make_diff_row_non_dict_check(self):
        row = self.rd.make_diff_row("T", None, _value(1), _value(2))
        self.assertEqual(row["op"], "")
        self.assertEqual(row["check"], "")


class ObservationKindMatrixTests(unittest.TestCase):
    """관측 kind 가 구/신에서 갈릴 때 회귀로만 잡히고 표면은 건드리지 않는다."""

    def setUp(self):
        self.rd = load_rd()

    def test_kind_pairs_do_not_change_classify(self):
        pairs = [
            (_value(1), _value(2)),
            (_value(1), {"kind": "exit", "code": 2, "head": ""}),
            ({"kind": "nojson", "head": "x"}, _value("x")),
            ({"kind": "digfail", "error": "KeyError"}, _value(None)),
            ({"kind": "timeout", "error": "TimeoutExpired"},
             {"kind": "timeout", "error": "TimeoutError"}),
            ({"kind": "missing-bin", "error": "FileNotFoundError"},
             _value(1)),
        ]
        for old, new in pairs:
            equal = self.rd.observations_equal(old, new)
            if equal:
                self.assertEqual(self.rd.classify(False, []), "stable")
            else:
                self.assertEqual(self.rd.classify(False, [1]), "regression")
            self.assertEqual(self.rd.classify(True, [1] if not equal else []),
                             "surface-changed")

    def test_identical_error_kinds_are_stable_input(self):
        obs = {"kind": "timeout", "error": "TimeoutExpired", "head": ""}
        self.assertTrue(self.rd.observations_equal(obs, dict(obs)))
        self.assertEqual(self.rd.classify(False, []), "stable")


class HonestyInvariantScanTests(unittest.TestCase):
    """모듈 상수·함수가 정직 조항을 깨지 않는지 훑는다."""

    def setUp(self):
        self.rd = load_rd()

    def test_exit_by_class_values_are_gate_contract(self):
        self.assertEqual(self.rd.EXIT_BY_CLASS["stable"], 0)
        self.assertEqual(self.rd.EXIT_BY_CLASS["surface-changed"], 2)
        self.assertEqual(self.rd.EXIT_BY_CLASS["regression"], 3)
        self.assertNotIn("probe-failed", self.rd.EXIT_BY_CLASS)
        self.assertNotIn("skipped", self.rd.EXIT_BY_CLASS)

    def test_ok_review_surface_mutually_consistent(self):
        cases = [
            ("aaa", "aaa", [], True, False, False, "stable"),
            ("aaa", "aaa", [1], False, False, False, "regression"),
            ("aaa", "bbb", [], False, True, True, "surface-changed"),
            ("aaa", "bbb", [1], False, True, True, "surface-changed"),
        ]
        for old_d, new_d, diffs, ok, review, surface, label in cases:
            report = self.rd.build_report("o", old_d, "n", new_d, 1, 1, diffs)
            self.assertEqual(report["ok"], ok, label)
            self.assertEqual(report["reviewRequired"], review, label)
            self.assertEqual(report["surfaceChanged"], surface, label)
            self.assertEqual(report["classification"], label)

    def test_probe_failed_never_sets_ok_or_review(self):
        for old_d, new_d in ((None, None), (None, "aaa"), ("aaa", None)):
            report = self.rd.build_report("o", old_d, "n", new_d, 0, 0, [])
            self.assertFalse(report["ok"], (old_d, new_d))
            self.assertFalse(report["reviewRequired"], (old_d, new_d))
            self.assertFalse(report["surfaceChanged"], (old_d, new_d))
            self.assertEqual(report["classification"], "probe-failed")


# 관측 동등성 표 — 한 행이 한 계약. 분류를 바꾸지 않는다.
EQUALITY_CASES = (
    ("int-float", _value(0), _value(0.0), True),
    ("int-int", _value(-3), _value(-3), True),
    ("bool-true", _value(True), _value(True), True),
    ("bool-false", _value(False), _value(False), True),
    ("bool-not-one", _value(True), _value(1), False),
    ("bool-not-one-float", _value(True), _value(1.0), False),
    ("str-space", _value("a"), _value("a "), False),
    ("empty-str-none", _value(""), _value(None), False),
    ("list-empty", _value([]), _value([]), True),
    ("list-vs-tuple-inner", _value([1, 2]), _value([1.0, 2.0]), True),
    ("dict-empty", _value({}), _value({}), True),
    ("exit-vs-value", {"kind": "exit", "code": 0, "head": ""}, _value(0), False),
    ("nojson-vs-str", {"kind": "nojson", "head": "nojson"}, _value("nojson"), False),
    ("digfail-vs-none", {"kind": "digfail", "error": "KeyError"}, _value(None), False),
    ("two-exits-same", {"kind": "exit", "code": 2, "head": "a"},
     {"kind": "exit", "code": 2, "head": "a"}, True),
    ("two-exits-diff-head", {"kind": "exit", "code": 2, "head": "a"},
     {"kind": "exit", "code": 2, "head": "b"}, False),
    ("nested-bool", _value({"ok": True}), _value({"ok": 1}), False),
    ("nested-num", _value({"n": [1, 2]}), _value({"n": [1.0, 2.0]}), True),
)


class GeneratedEqualityTableTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_equality_table(self):
        for name, left, right, expected in EQUALITY_CASES:
            self.assertEqual(
                self.rd.observations_equal(left, right), expected, name)

    def test_equality_table_is_symmetric(self):
        for name, left, right, expected in EQUALITY_CASES:
            self.assertEqual(
                self.rd.observations_equal(right, left), expected, name + "/sym")


# 분류 표 — 표면이 회귀보다 앞선다. probe-failed 행은 이 표에 없다.
CLASSIFY_CASES = (
    ("same-empty-list", False, [], "stable"),
    ("same-zero", False, 0, "stable"),
    ("same-none", False, None, "stable"),
    ("same-false", False, False, "stable"),
    ("same-one", False, 1, "regression"),
    ("same-list", False, [{"task": "T"}], "regression"),
    ("same-true", False, True, "regression"),
    ("surf-empty", True, [], "surface-changed"),
    ("surf-zero", True, 0, "surface-changed"),
    ("surf-list", True, [{"task": "T"}], "surface-changed"),
    ("surf-true", True, True, "surface-changed"),
    ("surf-none", True, None, "surface-changed"),
)


class GeneratedClassifyTableTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_classify_table(self):
        for name, surface, divergences, expected in CLASSIFY_CASES:
            self.assertEqual(self.rd.classify(surface, divergences), expected, name)

    def test_table_has_no_probe_failed(self):
        for name, _s, _d, expected in CLASSIFY_CASES:
            self.assertIn(expected, ("stable", "regression", "surface-changed"), name)


class ProbeFailedDisguiseTests(unittest.TestCase):
    """프로브 실패를 삼원 중 아무것으로도 부르지 않는지 고정한다."""

    def setUp(self):
        self.rd = load_rd()

    def test_missing_old_digest_not_surface_changed(self):
        report = self.rd.build_report("/o/rhwp", None, "/n/rhwp", "abc", 4, 8, [])
        self.assertNotEqual(report["classification"], "surface-changed")
        self.assertNotEqual(report["classification"], "stable")
        self.assertNotEqual(report["classification"], "regression")

    def test_missing_new_digest_not_regression_even_with_diffs(self):
        row = {"task": "T", "check": "x", "op": "value_eq", "path": "",
               "old": _value(1), "new": _value(2)}
        report = self.rd.build_report("/o/rhwp", "abc", "/n/rhwp", None, 1, 1, [row])
        self.assertEqual(report["classification"], "probe-failed")
        self.assertFalse(report["ok"])
        self.assertEqual(report["divergences"], 1)

    def test_summary_does_not_say_surface_same_when_probe_failed(self):
        report = self.rd.build_report("o", None, "n", None, 0, 0, [])
        text = "\n".join(self.rd.render_summary(report, "x.json"))
        self.assertNotIn("명령 표면(capabilities): 같음", text)
        self.assertIn("프로브 실패", text)


class CatchableExceptionCatalogTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_catchable_includes_timeout_and_os(self):
        names = {cls.__name__ for cls in self.rd.CATCHABLE_EXCEPTIONS}
        for needed in (
            "FileNotFoundError", "PermissionError", "TimeoutError",
            "TimeoutExpired", "OSError", "ValueError", "TypeError",
            "KeyError", "IndexError", "AttributeError", "JSONDecodeError",
            "RuntimeError", "UnicodeError",
        ):
            self.assertIn(needed, names)

    def test_fatal_not_in_catchable_as_base(self):
        for fatal in (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit):
            self.assertFalse(issubclass(fatal, OSError))
            self.assertTrue(self.rd.is_fatal_exception(fatal()))


class ParseArgsAndSummaryLimitTests(unittest.TestCase):
    def setUp(self):
        self.rd = load_rd()

    def test_parse_args_requires_old_and_new(self):
        with self.assertRaises(SystemExit):
            self.rd.parse_args([])
        with self.assertRaises(SystemExit):
            self.rd.parse_args(["--old", "o"])
        with self.assertRaises(SystemExit):
            self.rd.parse_args(["--new", "n"])

    def test_parse_args_pack_repeat(self):
        ns = self.rd.parse_args(["--old", "o", "--new", "n", "--pack", "a", "--pack", "b"])
        self.assertEqual(ns.pack, ["a", "b"])
        self.assertEqual(ns.agent, "claude-fable-5")

    def test_summary_caps_diff_rows_at_thirty(self):
        rows = []
        for i in range(40):
            rows.append({
                "pack": "p", "task": f"T{i:02d}", "check": "쪽수",
                "op": "value_eq", "path": "pageCount",
                "old": _value(i), "new": _value(i + 1),
            })
        report = self.rd.build_report("o", "x", "n", "x", 40, 40, rows)
        lines = self.rd.render_summary(report, "out.json")
        detail = [ln for ln in lines if ln.startswith("  p/T")]
        self.assertEqual(len(detail), 30)
        self.assertEqual(report["divergences"], 40)
        self.assertEqual(report["classification"], "regression")

    def test_write_report_rejects_bom_and_crlf(self):
        report = self.rd.build_report("o", "aaa", "n", "bbb", 1, 1, [])
        self.assertEqual(report["classification"], "surface-changed")
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "r.json")
            self.rd.write_report(report, path)
            raw = Path(path).read_bytes()
        self.assertFalse(raw.startswith(b"\xef\xbb\xbf"))
        self.assertNotIn(b"\r\n", raw)
        self.assertTrue(raw.endswith(b"\n"))
        self.assertEqual(json.loads(raw.decode("utf-8"))["exit"], 2)

    def test_validate_report_probe_failed_exit_must_be_one(self):
        report = self.rd.empty_report("o", "n")
        report["exit"] = 0
        issues = self.rd.validate_report(report)
        self.assertTrue(any("exit" in i for i in issues))
        report["exit"] = 2
        issues = self.rd.validate_report(report)
        self.assertTrue(any("exit" in i for i in issues))

    def test_validate_report_probe_failed_flags(self):
        report = self.rd.empty_report("o", "n")
        report["ok"] = True
        self.assertTrue(any("ok" in i for i in self.rd.validate_report(report)))
        report = self.rd.empty_report("o", "n")
        report["reviewRequired"] = True
        self.assertTrue(any("reviewRequired" in i for i in self.rd.validate_report(report)))
        report = self.rd.empty_report("o", "n")
        report["surfaceChanged"] = True
        self.assertTrue(any("surfaceChanged" in i for i in self.rd.validate_report(report)))
        report = self.rd.empty_report("o", "n")
        report["probeFailed"] = False
        self.assertTrue(any("probeFailed" in i for i in self.rd.validate_report(report)))


if __name__ == "__main__":
    unittest.main()
