"""[report] gym 능력 리포트 계약 — 축 프로파일 집계 + 정확도/커버리지 분리 불변식.

핵심 불변식: 정확도(측정된 것 통과율)와 커버리지(측정 폭)는 **다른 것**이라 뭉뚱
그리지 않는다. 축별 프로파일은 pack 의 axis 라벨(괄호 앞 차원)로 점수를 합산한다.
바이너리 없이 순수 합성만 시험한다.

예외 칸(#5275): 없는 스코어카드, 깨진 JSON, 미가용 pack 은 스택이 아니라
kind 로 남긴다. 새 CLI 플래그는 없다. 예전 성공 칸의 숫자·문구는 그대로다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "report.py"
DOCS = REPO_ROOT / "gym" / "docs" / "certify_report.md"


def load():
    spec = importlib.util.spec_from_file_location("gym_report", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


SCORECARD = {
    "agent": "x",
    "runner": {"rhwpVersion": "0.8.4", "rhwpCommit": "abc123def456"},
    "total": {"score": 5, "max": 8, "packsScored": 3, "packsUnavailable": 1},
    "packs": [
        {"id": "a", "axis": "편집 (표 좌표)", "score": 3, "max": 3, "status": "scored"},
        {"id": "b", "axis": "편집 (치환)", "score": 1, "max": 3, "status": "scored"},
        {"id": "c", "axis": "조사 (읽기)", "score": 1, "max": 2, "status": "scored"},
        {"id": "d", "axis": "보안 (PII)", "score": 0, "max": 0, "status": "unavailable"},
    ],
}
COVERAGE = {
    "coveragePercent": 82, "covered": 42, "agentFacingTotal": 51,
    "uncoveredByCategory": {"export": ["export-pdf"]},
}


def _write(path, payload):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        if isinstance(payload, str):
            fh.write(payload)
        else:
            json.dump(payload, fh, ensure_ascii=False)


class ReportTests(unittest.TestCase):
    def test_axis_profile_aggregates_by_label(self):
        r = load().compile_report(SCORECARD, COVERAGE)
        by = {a["axis"]: a for a in r["axisProfile"]}
        # 편집 두 pack 합산: 3+1=4 / 3+3=6 = 66%.
        self.assertEqual((by["편집"]["score"], by["편집"]["max"], by["편집"]["percent"]), (4, 6, 66))
        self.assertEqual(by["조사"]["score"], 1)

    def test_accuracy_and_coverage_are_separate(self):
        r = load().compile_report(SCORECARD, COVERAGE)
        self.assertEqual(r["accuracy"]["percent"], 62)   # 5/8
        self.assertEqual(r["coverage"]["percent"], 82)
        self.assertNotEqual(r["accuracy"]["percent"], r["coverage"]["percent"])

    def test_unavailable_pack_excluded_from_axis_profile(self):
        r = load().compile_report(SCORECARD, COVERAGE)
        self.assertNotIn("보안", [a["axis"] for a in r["axisProfile"]])
        self.assertIn("d", r["packsUnavailable"])

    def test_coverage_is_optional(self):
        cov = load().compile_report(SCORECARD, {})
        card = load().render_card(cov)
        self.assertIn("정확도", card)
        self.assertNotIn("커버리지", card)  # coverage 없으면 그 줄을 뺀다


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_exception_kinds_are_unique_and_documented(self):
        kinds = self.m.EXCEPTION_KINDS
        self.assertEqual(len(kinds), len(set(kinds)))
        for kind in kinds:
            self.assertIn(kind, self.m.EXCEPTION_KIND_HELP)
            self.assertTrue(self.m.EXCEPTION_KIND_HELP[kind])
            self.assertTrue(self.m.is_known_exception_kind(kind))
            self.assertTrue(self.m.describe_exception_kind(kind))

    def test_docs_backtick_every_exception_kind(self):
        text = DOCS.read_text(encoding="utf-8")
        for kind in self.m.EXCEPTION_KINDS:
            self.assertIn(f"`{kind}`", text, msg=kind)

    def test_docs_backtick_every_pack_status(self):
        text = DOCS.read_text(encoding="utf-8")
        for status in self.m.PACK_STATUSES:
            self.assertIn(f"`{status}`", text, msg=status)

    def test_report_keys_cover_legacy_and_exception_slots(self):
        for key in ("kind", "accuracy", "coverage", "axisProfile",
                    "packsUnavailable", "exceptions", "trusted"):
            self.assertIn(key, self.m.REPORT_KEYS)

    def test_unknown_kind_is_not_known(self):
        self.assertFalse(self.m.is_known_exception_kind("not-a-kind"))
        self.assertFalse(self.m.is_known_exception_kind(None))
        self.assertFalse(self.m.is_known_exception_kind(1))

    def test_describe_unknown_falls_back_to_unexpected(self):
        self.assertEqual(
            self.m.describe_exception_kind("??"),
            self.m.EXCEPTION_KIND_HELP["unexpected"],
        )

    def test_cli_flags_unchanged(self):
        names = self.m.cli_flag_names()
        self.assertEqual(tuple(sorted(names)), tuple(sorted(self.m.REPORT_CLI_FLAGS)))

    def test_no_new_cli_flag_constants(self):
        self.assertNotIn("--limit", self.m.REPORT_CLI_FLAGS)
        self.assertNotIn("--task", self.m.REPORT_CLI_FLAGS)
        self.assertNotIn("--timeout", self.m.REPORT_CLI_FLAGS)
        self.assertNotIn("--strict", self.m.REPORT_CLI_FLAGS)


class JsonShapeTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_is_json_object_rejects_bool_list_none(self):
        self.assertTrue(self.m.is_json_object({"a": 1}))
        self.assertFalse(self.m.is_json_object([]))
        self.assertFalse(self.m.is_json_object("x"))
        self.assertFalse(self.m.is_json_object(None))
        self.assertFalse(self.m.is_json_object(True))

    def test_is_json_array(self):
        self.assertTrue(self.m.is_json_array([]))
        self.assertTrue(self.m.is_json_array([1]))
        self.assertFalse(self.m.is_json_array({}))

    def test_as_int_rejects_bool_and_text(self):
        self.assertEqual(self.m.as_int(3), 3)
        self.assertEqual(self.m.as_int(True, 0), 0)
        self.assertEqual(self.m.as_int("3", 0), 0)
        self.assertEqual(self.m.as_int(4.0), 4)
        self.assertEqual(self.m.as_int(4.5, 0), 0)
        self.assertEqual(self.m.as_int(None, 7), 7)

    def test_percent_of_zero_max_is_zero(self):
        self.assertEqual(self.m.percent_of(0, 0), 0)
        self.assertEqual(self.m.percent_of(5, 8), 62)
        self.assertEqual(self.m.percent_of(8, 8), 100)

    def test_axis_label_edges(self):
        self.assertEqual(self.m.axis_label("편집 (표)"), "편집")
        self.assertEqual(self.m.axis_label(""), "미분류")
        self.assertEqual(self.m.axis_label(None), "미분류")
        self.assertEqual(self.m.axis_label("   "), "미분류")
        self.assertEqual(self.m.axis_label("조사"), "조사")

    def test_pack_status_helpers(self):
        scored = {"id": "a", "status": "scored"}
        missing = {"id": "b", "status": "unavailable"}
        errored = {"id": "c", "status": "error"}
        self.assertTrue(self.m.is_scored_pack(scored))
        self.assertTrue(self.m.is_unavailable_pack(missing))
        self.assertTrue(self.m.is_error_pack(errored))
        self.assertFalse(self.m.is_scored_pack("nope"))
        self.assertFalse(self.m.is_unavailable_pack({"status": "other"}))
        self.assertEqual(self.m.pack_id_of({}), "?")
        self.assertEqual(self.m.pack_id_of({"id": "  "}), "?")
        self.assertEqual(self.m.pack_id_of({"id": "tb"}), "tb")


class ExceptionRecordTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_exception_record_omits_empty_slots(self):
        rec = self.m.exception_record("missing-scorecard", "없다")
        self.assertEqual(rec["kind"], "missing-scorecard")
        self.assertEqual(rec["message"], "없다")
        self.assertNotIn("path", rec)
        self.assertNotIn("pack", rec)

    def test_exception_record_keeps_pack_and_path(self):
        rec = self.m.exception_record(
            "unavailable-pack", "부재", pack="d", path="sc.json", where="packs",
        )
        self.assertEqual(rec["pack"], "d")
        self.assertEqual(rec["path"], "sc.json")
        self.assertEqual(rec["where"], "packs")

    def test_unknown_kind_coerced_to_unexpected(self):
        rec = self.m.exception_record("???", "x")
        self.assertEqual(rec["kind"], "unexpected")

    def test_report_error_as_record(self):
        err = self.m.ReportError("missing-scorecard", "없음", path="a.json")
        rec = err.as_record()
        self.assertEqual(rec["kind"], "missing-scorecard")
        self.assertEqual(rec["path"], "a.json")

    def test_report_error_unknown_kind_becomes_unexpected(self):
        err = self.m.ReportError("not-real", "x")
        self.assertEqual(err.kind, "unexpected")

    def test_informational_kinds_do_not_untrust(self):
        notes = [
            self.m.exception_record("unavailable-pack", "부재", pack="d"),
            self.m.exception_record("empty-total", "0"),
        ]
        self.assertEqual(self.m.structural_exceptions(notes), [])
        notes.append(self.m.exception_record("malformed-json", "깨짐"))
        self.assertEqual(len(self.m.structural_exceptions(notes)), 1)


class FatalCatchableTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_fatal_exceptions_are_not_catchable(self):
        for exc in (KeyboardInterrupt(), SystemExit(2), MemoryError(), GeneratorExit()):
            self.assertTrue(self.m.is_fatal_exception(exc))
            self.assertFalse(self.m.is_catchable_exception(exc))

    def test_os_and_json_are_catchable(self):
        self.assertTrue(self.m.is_catchable_exception(FileNotFoundError("x")))
        self.assertTrue(self.m.is_catchable_exception(json.JSONDecodeError("m", "d", 0)))
        self.assertTrue(self.m.is_catchable_exception(self.m.ReportError("os-error", "e")))

    def test_classify_os_error_by_role(self):
        self.assertEqual(
            self.m.classify_os_error(FileNotFoundError("x"), role="scorecard"),
            "missing-scorecard",
        )
        self.assertEqual(
            self.m.classify_os_error(FileNotFoundError("x"), role="coverage"),
            "missing-coverage",
        )
        self.assertEqual(
            self.m.classify_os_error(FileNotFoundError("x"), role="bin"),
            "missing-bin",
        )
        self.assertEqual(
            self.m.classify_os_error(PermissionError("x"), role="scorecard"),
            "permission",
        )
        self.assertEqual(
            self.m.classify_os_error(json.JSONDecodeError("m", "d", 0), role="scorecard"),
            "malformed-json",
        )

    def test_wrap_exception_reraises_fatal(self):
        with self.assertRaises(KeyboardInterrupt):
            self.m.wrap_exception(KeyboardInterrupt(), role="scorecard")

    def test_wrap_exception_preserves_report_error(self):
        err = self.m.ReportError("missing-scorecard", "없음")
        self.assertIs(self.m.wrap_exception(err), err)


class LoadJsonTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_missing_scorecard_file(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "no-such.json")
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(path, role="scorecard")
            self.assertEqual(ctx.exception.kind, "missing-scorecard")

    def test_missing_coverage_file(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "no-cov.json")
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(path, role="coverage")
            self.assertEqual(ctx.exception.kind, "missing-coverage")

    def test_empty_path_is_missing(self):
        with self.assertRaises(self.m.ReportError) as ctx:
            self.m.load_json_object("", role="scorecard")
        self.assertEqual(ctx.exception.kind, "missing-scorecard")
        with self.assertRaises(self.m.ReportError) as ctx:
            self.m.load_json_object("   ", role="coverage")
        self.assertEqual(ctx.exception.kind, "missing-coverage")

    def test_bad_json_scorecard(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "bad.json")
            _write(path, "{not json")
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(path, role="scorecard")
            self.assertEqual(ctx.exception.kind, "malformed-json")

    def test_empty_file_is_malformed_json(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "empty.json")
            _write(path, "")
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(path, role="scorecard")
            self.assertEqual(ctx.exception.kind, "malformed-json")

    def test_array_scorecard_is_malformed_scorecard(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "arr.json")
            _write(path, [1, 2, 3])
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(path, role="scorecard")
            self.assertEqual(ctx.exception.kind, "malformed-scorecard")

    def test_array_coverage_is_malformed_coverage(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "arr.json")
            _write(path, ["x"])
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(path, role="coverage")
            self.assertEqual(ctx.exception.kind, "malformed-coverage")

    def test_scalar_json_is_malformed(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "n.json")
            _write(path, "4")
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(path, role="scorecard")
            self.assertEqual(ctx.exception.kind, "malformed-scorecard")

    def test_directory_path_is_malformed_role(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(self.m.ReportError) as ctx:
                self.m.load_json_object(d, role="scorecard")
            self.assertEqual(ctx.exception.kind, "malformed-scorecard")

    def test_valid_object_roundtrip(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "ok.json")
            _write(path, {"agent": "x", "packs": []})
            data = self.m.load_json_object(path, role="scorecard")
            self.assertEqual(data["agent"], "x")

    def test_parse_json_text_none(self):
        with self.assertRaises(self.m.ReportError) as ctx:
            self.m.parse_json_text(None, role="scorecard")
        self.assertEqual(ctx.exception.kind, "malformed-json")

    def test_trailing_comma_is_malformed_json(self):
        with self.assertRaises(self.m.ReportError) as ctx:
            self.m.parse_json_text('{"a": 1,}', role="scorecard")
        self.assertEqual(ctx.exception.kind, "malformed-json")


class CompileReportExceptionTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_unavailable_pack_recorded_as_exception_and_list(self):
        r = self.m.compile_report(SCORECARD, COVERAGE)
        kinds = [e["kind"] for e in r["exceptions"]]
        self.assertIn("unavailable-pack", kinds)
        self.assertIn("d", r["packsUnavailable"])
        self.assertTrue(r["trusted"], r["exceptions"])
        pack_ids = [e.get("pack") for e in r["exceptions"] if e["kind"] == "unavailable-pack"]
        self.assertEqual(pack_ids, ["d"])

    def test_unavailable_does_not_change_accuracy_math(self):
        r = self.m.compile_report(SCORECARD, COVERAGE)
        self.assertEqual(r["accuracy"], {"score": 5, "max": 8, "percent": 62})
        self.assertEqual(r["packsScored"], 3)

    def test_missing_scorecard_object_does_not_raise(self):
        r = self.m.compile_report(None, {})  # type: ignore[arg-type]
        self.assertEqual(r["kind"], self.m.REPORT_KIND)
        kinds = [e["kind"] for e in r["exceptions"]]
        self.assertIn("malformed-scorecard", kinds)
        self.assertFalse(r["trusted"])
        self.assertEqual(r["accuracy"]["percent"], 0)
        self.assertEqual(r["axisProfile"], [])

    def test_array_scorecard_does_not_raise(self):
        r = self.m.compile_report([1, 2], {})  # type: ignore[arg-type]
        self.assertIn("malformed-scorecard", [e["kind"] for e in r["exceptions"]])
        self.assertEqual(r["packsUnavailable"], [])

    def test_empty_packs_note(self):
        card = {"agent": "x", "total": {"score": 0, "max": 0, "packsScored": 0}, "packs": []}
        r = self.m.compile_report(card, {})
        kinds = [e["kind"] for e in r["exceptions"]]
        self.assertIn("empty-packs", kinds)
        self.assertIn("empty-total", kinds)
        self.assertTrue(r["trusted"])

    def test_missing_packs_key(self):
        r = self.m.compile_report({"total": {"score": 1, "max": 1, "packsScored": 1}}, {})
        self.assertIn("empty-packs", [e["kind"] for e in r["exceptions"]])

    def test_packs_not_a_list(self):
        r = self.m.compile_report({"packs": {"id": "a"}, "total": {"score": 1, "max": 1}}, {})
        self.assertTrue(any(e["kind"] == "malformed-scorecard" for e in r["exceptions"]))
        self.assertEqual(r["axisProfile"], [])

    def test_malformed_pack_row_skipped(self):
        card = {
            "total": {"score": 3, "max": 3, "packsScored": 1},
            "packs": [
                "broken",
                {"id": "ok", "axis": "편집", "score": 3, "max": 3, "status": "scored"},
            ],
        }
        r = self.m.compile_report(card, {})
        self.assertIn("malformed-pack-row", [e["kind"] for e in r["exceptions"]])
        self.assertEqual(r["axisProfile"][0]["score"], 3)
        self.assertFalse(r["trusted"])

    def test_error_pack_listed_separately_from_unavailable(self):
        card = {
            "total": {"score": 1, "max": 1, "packsScored": 1},
            "packs": [
                {"id": "ok", "axis": "조사", "score": 1, "max": 1, "status": "scored"},
                {"id": "gone", "axis": "보안", "status": "unavailable"},
                {"id": "boom", "axis": "변환", "status": "error"},
            ],
        }
        r = self.m.compile_report(card, {})
        self.assertEqual(r["packsUnavailable"], ["gone"])
        self.assertEqual(r["packsErrored"], ["boom"])
        self.assertEqual([a["axis"] for a in r["axisProfile"]], ["조사"])

    def test_array_coverage_note(self):
        r = self.m.compile_report(SCORECARD, ["nope"])  # type: ignore[arg-type]
        self.assertIn("malformed-coverage", [e["kind"] for e in r["exceptions"]])
        self.assertIsNone(r["coverage"]["percent"])

    def test_bool_score_does_not_inflate_axis(self):
        card = {
            "total": {"score": 0, "max": 1, "packsScored": 1},
            "packs": [
                {"id": "t", "axis": "편집", "score": True, "max": True, "status": "scored"},
            ],
        }
        r = self.m.compile_report(card, {})
        self.assertEqual(r["axisProfile"][0]["score"], 0)
        self.assertEqual(r["axisProfile"][0]["max"], 0)

    def test_compile_report_has_all_report_keys(self):
        r = self.m.compile_report(SCORECARD, COVERAGE)
        for key in self.m.REPORT_KEYS:
            self.assertIn(key, r, msg=key)

    def test_multiple_unavailable_packs(self):
        card = {
            "total": {"score": 0, "max": 0, "packsScored": 0},
            "packs": [
                {"id": "u1", "status": "unavailable"},
                {"id": "u2", "status": "unavailable"},
            ],
        }
        r = self.m.compile_report(card, {})
        self.assertEqual(r["packsUnavailable"], ["u1", "u2"])
        unavail = [e for e in r["exceptions"] if e["kind"] == "unavailable-pack"]
        self.assertEqual(len(unavail), 2)


class RenderCardExceptionTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_card_lists_unavailable_and_exception_kind(self):
        r = self.m.compile_report(SCORECARD, COVERAGE)
        card = self.m.render_card(r)
        self.assertIn("미가용 pack", card)
        self.assertIn("`unavailable-pack`", card)
        self.assertIn("예외 경로", card)
        self.assertIn("정확도", card)
        self.assertIn("커버리지", card)

    def test_card_without_exceptions_has_no_exception_heading(self):
        card_in = {
            "agent": "x",
            "total": {"score": 1, "max": 1, "packsScored": 1},
            "packs": [{"id": "a", "axis": "조사", "score": 1, "max": 1, "status": "scored"}],
        }
        r = self.m.compile_report(card_in, {})
        text = self.m.render_card(r)
        self.assertNotIn("예외 경로", text)
        self.assertNotIn("커버리지", text)

    def test_render_card_accepts_non_object(self):
        text = self.m.render_card(None)  # type: ignore[arg-type]
        self.assertIn("정확도", text)
        self.assertTrue(text.endswith("\n"))

    def test_untrusted_card_mentions_trust(self):
        r = self.m.compile_report([1], {})  # type: ignore[arg-type]
        text = self.m.render_card(r)
        self.assertIn("신뢰", text)

    def test_error_pack_line_on_card(self):
        card = {
            "total": {"score": 0, "max": 0, "packsScored": 0},
            "packs": [{"id": "boom", "status": "error"}],
        }
        text = self.m.render_card(self.m.compile_report(card, {}))
        self.assertIn("오류 pack", text)
        self.assertIn("boom", text)


class MainCliTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_main_missing_args_exit_2(self):
        buf = io.StringIO()
        with mock.patch.object(sys, "stderr", buf):
            code = self.m.main([])
        self.assertEqual(code, 2)
        self.assertIn("필수", buf.getvalue())

    def test_main_missing_scorecard_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            cov = os.path.join(d, "cov.json")
            _write(cov, COVERAGE)
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--scorecard", os.path.join(d, "no.json"),
                                    "--coverage", cov])
            self.assertEqual(code, 2)
            self.assertIn("missing-scorecard", buf.getvalue())

    def test_main_missing_coverage_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            sc = os.path.join(d, "sc.json")
            _write(sc, SCORECARD)
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--scorecard", sc,
                                    "--coverage", os.path.join(d, "no.json")])
            self.assertEqual(code, 2)
            self.assertIn("missing-coverage", buf.getvalue())

    def test_main_bad_json_scorecard_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            sc = os.path.join(d, "sc.json")
            cov = os.path.join(d, "cov.json")
            _write(sc, "{")
            _write(cov, COVERAGE)
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--scorecard", sc, "--coverage", cov])
            self.assertEqual(code, 2)
            self.assertIn("malformed-json", buf.getvalue())

    def test_main_array_scorecard_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            sc = os.path.join(d, "sc.json")
            cov = os.path.join(d, "cov.json")
            _write(sc, [1])
            _write(cov, COVERAGE)
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--scorecard", sc, "--coverage", cov])
            self.assertEqual(code, 2)
            self.assertIn("malformed-scorecard", buf.getvalue())

    def test_main_success_json_stdout(self):
        with tempfile.TemporaryDirectory() as d:
            sc = os.path.join(d, "sc.json")
            cov = os.path.join(d, "cov.json")
            _write(sc, SCORECARD)
            _write(cov, COVERAGE)
            buf = io.StringIO()
            with mock.patch.object(sys, "stdout", buf):
                code = self.m.main(["--scorecard", sc, "--coverage", cov, "--json"])
            self.assertEqual(code, 0)
            payload = json.loads(buf.getvalue())
            self.assertEqual(payload["kind"], "gymCapabilityReport")
            self.assertEqual(payload["accuracy"]["percent"], 62)
            self.assertIn("d", payload["packsUnavailable"])
            self.assertTrue(any(e["kind"] == "unavailable-pack" for e in payload["exceptions"]))

    def test_main_success_out_file(self):
        with tempfile.TemporaryDirectory() as d:
            sc = os.path.join(d, "sc.json")
            cov = os.path.join(d, "cov.json")
            out = os.path.join(d, "card.md")
            _write(sc, SCORECARD)
            _write(cov, COVERAGE)
            code = self.m.main(["--scorecard", sc, "--coverage", cov, "--out", out])
            self.assertEqual(code, 0)
            text = Path(out).read_text(encoding="utf-8")
            self.assertIn("정확도", text)
            self.assertIn("`unavailable-pack`", text)

    def test_main_scorecard_without_coverage_still_usage(self):
        with tempfile.TemporaryDirectory() as d:
            sc = os.path.join(d, "sc.json")
            _write(sc, SCORECARD)
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--scorecard", sc])
            self.assertEqual(code, 2)
            self.assertIn("필수", buf.getvalue())

    def test_main_empty_bin_is_missing_bin(self):
        buf = io.StringIO()
        with mock.patch.object(sys, "stderr", buf):
            code = self.m.main(["--bin", "   "])
        self.assertEqual(code, 2)
        self.assertIn("missing-bin", buf.getvalue())

    def test_write_text_creates_parent(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "nested", "out.md")
            self.m.write_text(path, "hello\n")
            self.assertEqual(Path(path).read_text(encoding="utf-8"), "hello\n")

    def test_dump_report_json_trailing_newline(self):
        r = self.m.compile_report(SCORECARD, COVERAGE)
        text = self.m.dump_report_json(r)
        self.assertTrue(text.endswith("\n"))
        self.assertEqual(json.loads(text)["kind"], "gymCapabilityReport")


class FromBinExceptionTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_from_bin_missing_scorecard_after_score(self):
        def fake_run(_argv):
            return None

        with tempfile.TemporaryDirectory() as d:
            self.m.HERE = d
            with mock.patch.object(self.m, "_run", fake_run):
                with self.assertRaises(self.m.ReportError) as ctx:
                    self.m._from_bin("dummy-bin")
            self.assertEqual(ctx.exception.kind, "missing-scorecard")

    def test_from_bin_bad_scorecard_json(self):
        def fake_run(_argv):
            return None

        with tempfile.TemporaryDirectory() as d:
            sc_dir = os.path.join(d, "submissions", "_report")
            os.makedirs(sc_dir)
            _write(os.path.join(sc_dir, "scorecard.json"), "{")
            self.m.HERE = d
            with mock.patch.object(self.m, "_run", fake_run):
                with self.assertRaises(self.m.ReportError) as ctx:
                    self.m._from_bin("dummy-bin")
            self.assertEqual(ctx.exception.kind, "malformed-json")

    def test_run_nonzero_is_report_tool_failed(self):
        fake = mock.Mock()
        fake.returncode = 3
        fake.stdout = b"out"
        fake.stderr = b"err"
        with mock.patch.object(self.m.subprocess, "run", return_value=fake):
            with mock.patch.object(sys, "stderr", io.StringIO()):
                with self.assertRaises(self.m.ReportError) as ctx:
                    self.m._run([os.path.join("gym", "score.py")])
        self.assertEqual(ctx.exception.kind, "report-tool-failed")


class CollectPackTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_collect_helpers_ignore_non_objects(self):
        card = {"packs": [None, "x", {"id": "u", "status": "unavailable"}]}
        self.assertEqual(self.m.collect_unavailable_packs(card), ["u"])
        self.assertEqual(self.m.collect_error_packs(card), [])
        self.assertEqual(self.m.collect_scored_packs(card), [])

    def test_collect_on_non_scorecard(self):
        self.assertEqual(self.m.collect_unavailable_packs(None), [])
        self.assertEqual(self.m.collect_error_packs("x"), [])
        self.assertEqual(self.m.iter_pack_rows(None), [])

    def test_validate_scorecard_total_not_object(self):
        notes = self.m.validate_scorecard({"packs": [], "total": [1]})
        kinds = [n["kind"] for n in notes]
        self.assertIn("malformed-scorecard", kinds)

    def test_coverage_none_is_ok(self):
        self.assertEqual(self.m.validate_coverage(None), [])
        self.assertEqual(self.m.validate_coverage({}), [])


class RoleKindTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_role_tables(self):
        self.assertEqual(self.m.role_missing_kind("scorecard"), "missing-scorecard")
        self.assertEqual(self.m.role_missing_kind("coverage"), "missing-coverage")
        self.assertEqual(self.m.role_missing_kind("bin"), "missing-bin")
        self.assertEqual(self.m.role_malformed_kind("scorecard"), "malformed-scorecard")
        self.assertEqual(self.m.role_malformed_kind("coverage"), "malformed-coverage")

    def test_scorecard_path_for_agent(self):
        path = self.m.scorecard_path_for_agent("_report")
        self.assertTrue(path.endswith(os.path.join("submissions", "_report", "scorecard.json"))
                        or path.replace("\\", "/").endswith("submissions/_report/scorecard.json"))

    def test_pack_status_help_covers_all(self):
        for status in self.m.PACK_STATUSES:
            self.assertTrue(self.m.PACK_STATUS_HELP[status])


if __name__ == "__main__":
    unittest.main()
