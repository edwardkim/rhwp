"""[competitive_bench] 경쟁 벤치 하네스 순수 로직 계약 — 바이너리·외부 도구 불요.

핵심 불변식(이 하네스의 존재 이유):
1. 집계는 정직하다 — medianMs 는 성공 실행만, byExt 는 형식별 성공을 남긴다.
2. 못 돌린 도구는 'n/a: 이유'로 렌더되고 **숫자를 지어내지 않는다**.
3. 충실도는 두 도구가 모두 성공한 파일에서만 계산한다(겹침 없으면 None).
4. 능력 매트릭스는 모든 행이 모든 컬럼 키를 갖고, rhwp 만 전 능력을 채운다.

gym 툴-테스트 패턴(importlib 로 모듈 적재 후 순수 함수만 시험)을 그대로 따른다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "competitive_bench.py"


def load():
    spec = importlib.util.spec_from_file_location("competitive_bench", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class MedianTests(unittest.TestCase):
    def test_median_ignores_none_and_handles_empty(self):
        m = load()
        self.assertEqual(m.median([3, 1, 2]), 2)
        self.assertEqual(m.median([1, None, 3]), 2)
        self.assertIsNone(m.median([]))
        self.assertIsNone(m.median([None, None]))


class SummarizeTests(unittest.TestCase):
    def _runs(self):
        return [
            {"file": "a.hwp", "ext": ".hwp", "ok": True, "ms": 10.0, "chars": 100},
            {"file": "b.hwp", "ext": ".hwp", "ok": True, "ms": 30.0, "chars": 300},
            {"file": "c.hwpx", "ext": ".hwpx", "ok": False, "ms": 5.0, "chars": None},
        ]

    def test_success_rate_and_median_use_ok_only(self):
        m = load()
        s = m.summarize_runs(self._runs())
        self.assertEqual(s["attempted"], 3)
        self.assertEqual(s["ok"], 2)
        self.assertEqual(s["successRate"], round(2 / 3, 3))
        # 실패의 5ms 는 중앙값에 끼면 안 된다 → 성공 10·30 의 중앙값 20.
        self.assertEqual(s["medianMs"], 20.0)
        self.assertEqual(s["medianChars"], 200)

    def test_by_ext_breakdown_records_format_support(self):
        m = load()
        s = m.summarize_runs(self._runs())
        self.assertEqual(s["byExt"][".hwp"], {"attempted": 2, "ok": 2, "fail": 0})
        # HWPX 는 시도했으나 실패 — pyhwp 형식 한계가 데이터로 남는다.
        self.assertEqual(s["byExt"][".hwpx"], {"attempted": 1, "ok": 0, "fail": 1})
        self.assertEqual(s["fail"], 1)

    def test_empty_runs_safe(self):
        m = load()
        s = m.summarize_runs([])
        self.assertEqual(s["attempted"], 0)
        self.assertEqual(s["fail"], 0)
        self.assertIsNone(s["successRate"])
        self.assertIsNone(s["medianMs"])
        self.assertEqual(s["fail"], 0)
        self.assertEqual(s["byExt"], {})

    def test_failed_run_chars_do_not_enter_median(self):
        m = load()
        s = m.summarize_runs([
            {"file": "a.hwp", "ok": True, "ms": 10, "chars": 10},
            {"file": "b.hwp", "ok": False, "ms": 1, "chars": 99999},
        ])
        self.assertEqual(s["medianChars"], 10)
        self.assertEqual(s["medianMs"], 10.0)

    def test_missing_ext_inferred_from_file(self):
        m = load()
        s = m.summarize_runs([
            {"file": "samples/doc.HWPX", "ok": True, "ms": 4, "chars": 1},
        ])
        self.assertIn(".hwpx", s["byExt"])
        self.assertEqual(s["byExt"][".hwpx"]["ok"], 1)

    def test_zero_chars_is_a_real_measurement(self):
        m = load()
        s = m.summarize_runs([
            {"file": "empty.hwp", "ext": ".hwp", "ok": True, "ms": 3, "chars": 0},
        ])
        self.assertEqual(s["medianChars"], 0)

    def test_bool_is_not_a_timing(self):
        m = load()
        s = m.summarize_runs([
            {"file": "a.hwp", "ext": ".hwp", "ok": True, "ms": True, "chars": False},
        ])
        self.assertIsNone(s["medianMs"])
        self.assertIsNone(s["medianChars"])


class FidelityTests(unittest.TestCase):
    def test_ratio_over_overlap_only(self):
        m = load()
        ref = [
            {"file": "a", "ok": True, "chars": 100},
            {"file": "b", "ok": True, "chars": 200},
        ]
        tool = [
            {"file": "a", "ok": True, "chars": 70},   # 0.70
            {"file": "b", "ok": True, "chars": 140},  # 0.70
        ]
        self.assertEqual(m.fidelity_vs_ref(tool, ref), 0.7)

    def test_none_when_no_overlap(self):
        m = load()
        ref = [{"file": "a", "ok": True, "chars": 100}]
        tool = [{"file": "b", "ok": True, "chars": 90}]  # 다른 파일
        self.assertIsNone(m.fidelity_vs_ref(tool, ref))

    def test_failed_or_missing_ref_excluded(self):
        m = load()
        ref = [
            {"file": "a", "ok": True, "chars": 100},
            {"file": "b", "ok": False, "chars": None},  # 기준 실패 → 제외
        ]
        tool = [
            {"file": "a", "ok": True, "chars": 50},   # 0.50
            {"file": "b", "ok": True, "chars": 999},  # 기준 없음 → 제외
        ]
        self.assertEqual(m.fidelity_vs_ref(tool, ref), 0.5)

    def test_zero_chars_is_not_treated_as_missing(self):
        m = load()
        ref = [{"file": "a", "ok": True, "chars": 100}]
        tool = [{"file": "a", "ok": True, "chars": 0}]
        self.assertEqual(m.fidelity_vs_ref(tool, ref), 0.0)
        self.assertEqual(m.fidelity_stats(tool, ref)["n"], 1)

    def test_zero_base_does_not_invent_ratio(self):
        m = load()
        ref = [{"file": "a", "ok": True, "chars": 0}]
        tool = [{"file": "a", "ok": True, "chars": 10}]
        self.assertIsNone(m.fidelity_vs_ref(tool, ref))
        self.assertEqual(m.fidelity_stats(tool, ref), {"n": 0, "median": None})

    def test_bool_chars_are_not_counts(self):
        m = load()
        ref = [{"file": "a", "ok": True, "chars": True}]
        tool = [{"file": "a", "ok": True, "chars": True}]
        self.assertIsNone(m.fidelity_vs_ref(tool, ref))


class OverlapMedianTests(unittest.TestCase):
    def test_overlap_median_uses_shared_ok_files_only(self):
        m = load()
        ref = [
            {"file": "a.hwp", "ok": True, "ms": 100.0},
            {"file": "b.hwp", "ok": True, "ms": 200.0},
            {"file": "c.hwpx", "ok": True, "ms": 900.0},  # tool 이 실패할 파일
        ]
        tool = [
            {"file": "a.hwp", "ok": True, "ms": 300.0},
            {"file": "b.hwp", "ok": True, "ms": 500.0},
            {"file": "c.hwpx", "ok": False, "ms": None},  # 겹침 아님
        ]
        t_ms, r_ms = m.overlap_median_ms(tool, ref)
        # 공정 비교: a·b 만. tool median=400, ref median=150 (900 제외).
        self.assertEqual(t_ms, 400.0)
        self.assertEqual(r_ms, 150.0)

    def test_no_overlap_returns_none_pair(self):
        m = load()
        self.assertEqual(
            m.overlap_median_ms(
                [{"file": "x", "ok": True, "ms": 1.0}],
                [{"file": "y", "ok": True, "ms": 2.0}],
            ),
            (None, None),
        )

    def test_zero_ms_is_a_real_measurement(self):
        m = load()
        t_ms, r_ms = m.overlap_median_ms(
            [{"file": "a", "ok": True, "ms": 0.0}],
            [{"file": "a", "ok": True, "ms": 0.0}],
        )
        self.assertEqual((t_ms, r_ms), (0.0, 0.0))

    def test_overlap_timing_reports_n(self):
        m = load()
        stats = m.overlap_timing(
            [
                {"file": "a", "ok": True, "ms": 10},
                {"file": "b", "ok": True, "ms": 20},
                {"file": "c", "ok": False, "ms": 1},
            ],
            [
                {"file": "a", "ok": True, "ms": 30},
                {"file": "b", "ok": True, "ms": 40},
                {"file": "c", "ok": True, "ms": 50},
            ],
        )
        self.assertEqual(stats["n"], 2)
        self.assertEqual(stats["tool"], 15.0)
        self.assertEqual(stats["ref"], 35.0)

    def test_missing_ref_ms_is_not_overlap(self):
        m = load()
        self.assertEqual(
            m.overlap_median_ms(
                [{"file": "a", "ok": True, "ms": 10}],
                [{"file": "a", "ok": True, "ms": None}],
            ),
            (None, None),
        )


class RhwpParseTests(unittest.TestCase):
    def test_sums_page_texts(self):
        m = load()
        env = json.dumps({"pages": [{"text": "가나다"}, {"text": "라마"}]})
        self.assertEqual(m.parse_rhwp_text_chars(env), 5)

    def test_bad_json_returns_none(self):
        m = load()
        self.assertIsNone(m.parse_rhwp_text_chars("not json"))
        self.assertIsNone(m.parse_rhwp_text_chars(json.dumps({"nope": 1})))
        self.assertIsNone(m.parse_rhwp_text_chars(None))
        self.assertIsNone(m.parse_rhwp_text_chars(json.dumps([1, 2, 3])))

    def test_unwraps_data_envelope(self):
        m = load()
        env = json.dumps({"ok": True, "data": {"pages": [{"text": "가나"}, {"text": "다"}]}})
        self.assertEqual(m.parse_rhwp_text_chars(env), 3)

    def test_batch_text_field(self):
        m = load()
        env = json.dumps({"schemaVersion": "1.0", "source": "a.hwp", "text": "한글본문"})
        self.assertEqual(m.parse_rhwp_text_chars(env), 4)

    def test_non_string_page_text_is_skipped_not_crashed(self):
        m = load()
        env = json.dumps({"pages": [{"text": None}, {"text": 12}, {"text": "가"}, "x"]})
        self.assertEqual(m.parse_rhwp_text_chars(env), 1)

    def test_empty_pages_is_zero_not_none(self):
        m = load()
        self.assertEqual(m.parse_rhwp_text_chars(json.dumps({"pages": []})), 0)

    def test_bytes_stdout(self):
        m = load()
        raw = json.dumps({"pages": [{"text": "ab"}]}).encode("utf-8")
        self.assertEqual(m.parse_rhwp_text_chars(raw), 2)

    def test_info_envelope_and_wrapper(self):
        m = load()
        raw = json.dumps({
            "schemaVersion": "1.0", "format": "hwp5",
            "pageCount": 6, "sections": 1, "paraCount": 40,
        })
        self.assertEqual(m.parse_rhwp_info(raw)["pageCount"], 6)
        wrapped = json.dumps({"data": {"format": "hwpx", "pageCount": 2.0}})
        self.assertEqual(m.parse_rhwp_info(wrapped)["format"], "hwpx")
        self.assertIsNone(m.parse_rhwp_info(json.dumps({"pages": [{"text": "x"}]})))

    def test_structure_envelope(self):
        m = load()
        raw = json.dumps({"mode": "outline", "nodeCount": 3, "structure": {"children": []}})
        parsed = m.parse_rhwp_structure(raw)
        self.assertEqual(parsed["nodeCount"], 3)
        self.assertTrue(parsed["hasStructure"])
        self.assertIsNone(m.parse_rhwp_structure(json.dumps({"ok": True})))


class CapabilityMatrixTests(unittest.TestCase):
    def test_every_row_has_every_column_key(self):
        m = load()
        matrix = m.capability_matrix()
        keys = [c["key"] for c in matrix["columns"]]
        for row in matrix["rows"]:
            for k in keys:
                self.assertIn(k, row, f"{row['tool']}: 컬럼 {k} 누락")
                self.assertIn(row[k], ("yes", "partial", "no"))

    def test_rhwp_fills_all_capabilities(self):
        m = load()
        matrix = m.capability_matrix()
        rhwp = next(r for r in matrix["rows"] if r["tool"] == "rhwp")
        keys = [c["key"] for c in matrix["columns"]]
        self.assertTrue(all(rhwp[k] == "yes" for k in keys), "rhwp 는 전 능력 yes 여야 한다")

    def test_hancom_is_windows_only(self):
        m = load()
        matrix = m.capability_matrix()
        hancom = next(r for r in matrix["rows"] if r["tool"] == "Hancom SDK")
        self.assertEqual(hancom["crossPlatform"], "no")

    def test_live_matrix_has_no_validation_issues(self):
        m = load()
        self.assertEqual(m.validate_capability_matrix(), [])
        self.assertEqual(m.validate_capability_matrix(m.capability_matrix()), [])

    def test_validation_flags_missing_and_bad_values(self):
        m = load()
        broken = {
            "columns": [{"key": "mcp", "label": "MCP"}],
            "rows": [
                {"tool": "rhwp", "mcp": "yes"},
                {"tool": "rhwp", "mcp": "maybe"},
                {"tool": "alt"},
            ],
        }
        issues = m.validate_capability_matrix(broken)
        self.assertTrue(any("중복" in i for i in issues))
        self.assertTrue(any("maybe" in i for i in issues))
        self.assertTrue(any("alt" in i and "누락" in i for i in issues))

    def test_exclusive_yes_is_only_rhwp_for_agent_surface(self):
        m = load()
        matrix = m.capability_matrix()
        excl = m.exclusive_yes(matrix, "rhwp")
        for key in ("mcp", "verifiable", "singleBinary", "memSafe", "agentCli"):
            self.assertIn(key, excl)
        self.assertNotIn("edit", excl)
        self.assertNotIn("render", excl)
        self.assertNotIn("crossPlatform", excl)
        self.assertEqual(m.exclusive_yes(matrix, "pyhwp (hwp5txt)"), [])

    def test_returned_rows_are_copies(self):
        m = load()
        a = m.capability_matrix()
        a["rows"][0]["mcp"] = "no"
        b = m.capability_matrix()
        self.assertEqual(b["rows"][0]["mcp"], "yes")


class HonestDegradationTests(unittest.TestCase):
    def test_unavailable_renders_na_with_reason_not_numbers(self):
        m = load()
        cell = m._fmt_cell(False, None, None, "미설치(이 머신)")
        self.assertTrue(cell.startswith("n/a:"))
        self.assertIn("미설치", cell)
        # 숫자 흔적이 없어야 한다.
        self.assertNotIn("ms", cell)
        self.assertNotIn("%", cell)

    def test_available_cell_shows_metrics(self):
        m = load()
        summary = {"attempted": 5, "ok": 5, "successRate": 1.0, "medianMs": 12.0,
                   "medianChars": 100, "byExt": {}}
        cell = m._fmt_cell(True, summary, 1.0, None)
        self.assertIn("100%", cell)
        self.assertIn("5/5", cell)
        self.assertIn("ms", cell)


class RenderReportTests(unittest.TestCase):
    def _payload(self):
        m = load()
        return {
            "generatedAt": "2026-01-01T00:00:00",
            "toolOrder": ["rhwp", "pyhwp", "soffice", "hwplib"],
            "env": {
                "os": "TestOS", "python": "3.11.0", "rhwpVersion": "rhwp v0.0.0",
                "rhwpProfile": "debug",
                "corpus": {"dir": "samples", "total": 2, "hwp": 1, "hwpx": 1},
                "tools": {
                    "rhwp": {"available": True, "detail": "v0.0.0"},
                    "pyhwp": {"available": True, "detail": "hwp5txt"},
                    "soffice": {"available": False, "detail": "미설치"},
                },
            },
            "tasks": [
                {"task": "export-text", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 2, "ok": 2, "successRate": 1.0,
                                 "medianMs": 10.0, "medianChars": 100,
                                 "byExt": {".hwp": {"attempted": 1, "ok": 1},
                                           ".hwpx": {"attempted": 1, "ok": 1}}},
                     "fidelityVsRhwp": 1.0},
                    {"tool": "pyhwp", "available": True,
                     "summary": {"attempted": 2, "ok": 1, "successRate": 0.5,
                                 "medianMs": 8.0, "medianChars": 70,
                                 "byExt": {".hwp": {"attempted": 1, "ok": 1},
                                           ".hwpx": {"attempted": 1, "ok": 0}}},
                     "fidelityVsRhwp": 0.7},
                    {"tool": "soffice", "available": False, "reason": "미설치(이 머신)"},
                    {"tool": "hwplib", "available": False, "reason": "Java 라이브러리, CLI 아님"},
                ]},
                {"task": "info", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 2, "ok": 2, "successRate": 1.0,
                                 "medianMs": 9.0, "medianChars": None, "byExt": {}},
                     "fidelityVsRhwp": None},
                    {"tool": "pyhwp", "available": False, "reason": "메타 봉투 없음"},
                    {"tool": "soffice", "available": False, "reason": "미설치"},
                    {"tool": "hwplib", "available": False, "reason": "CLI 아님"},
                ]},
            ],
            "capabilityMatrix": load().capability_matrix(),
            "verdict": ["rhwp 는 HWPX 까지 처리했다.", "pyhwp 는 HWP5 만."],
        }

    def test_report_has_required_sections(self):
        m = load()
        md = m.render_report(self._payload())
        self.assertIn("# 경쟁 벤치마크", md)
        self.assertIn("## 실행 환경", md)
        self.assertIn("## 능력 매트릭스", md)
        self.assertIn("## 정직한 평결", md)
        self.assertIn("## 재현", md)
        # 명제 문장이 있어야 한다.
        self.assertIn("에이전트", md)

    def test_unavailable_tool_shows_na_reason_in_table(self):
        m = load()
        md = m.render_report(self._payload())
        self.assertIn("n/a: 미설치(이 머신)", md)
        self.assertIn("n/a: Java 라이브러리, CLI 아님", md)

    def test_reproduction_command_present(self):
        m = load()
        md = m.render_report(self._payload())
        self.assertIn("competitive_bench.py", md)
        self.assertIn("cargo build --bin rhwp", md)

    def test_verdict_lines_rendered(self):
        m = load()
        md = m.render_report(self._payload())
        self.assertIn("HWPX 까지 처리했다", md)


class VerdictDerivationTests(unittest.TestCase):
    def test_verdict_derived_from_measured_numbers(self):
        m = load()
        payload = {
            "tasks": [
                {"task": "export-text", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 4, "ok": 4, "medianMs": 12.0, "byExt": {}}},
                    {"tool": "pyhwp", "available": True,
                     "summary": {"attempted": 4, "ok": 2, "medianMs": 8.0,
                                 "byExt": {".hwp": {"attempted": 2, "ok": 2},
                                           ".hwpx": {"attempted": 2, "ok": 0}}},
                     "overlapMs": {"tool": 8.0, "ref": 12.0},
                     "fidelityVsRhwp": 0.7},
                ]},
            ],
        }
        lines = m.verdict_lines(payload)
        text = " ".join(lines)
        # pyhwp 가 더 빠른 사실(8<12)을 정직하게 진술해야 한다.
        self.assertIn("pyhwp", text)
        self.assertTrue("더 빨" in text or "빠른" in text)
        # HWPX 0/2 한계도 진술.
        self.assertIn("HWPX", text)

    def test_empty_payload_does_not_raise(self):
        m = load()
        lines = m.verdict_lines({})
        self.assertTrue(any("능력" in line for line in lines))

    def test_tie_speed_is_not_a_win(self):
        m = load()
        payload = {
            "tasks": [
                {"task": "export-text", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 1, "ok": 1, "medianMs": 8.0}},
                    {"tool": "pyhwp", "available": True,
                     "summary": {"attempted": 1, "ok": 1, "medianMs": 8.0,
                                 "byExt": {".hwp": {"attempted": 1, "ok": 1}}},
                     "overlapMs": {"tool": 8.0, "ref": 8.0}},
                ]},
            ],
        }
        text = " ".join(m.verdict_lines(payload))
        self.assertIn("같", text)
        self.assertNotIn("더 빨", text)

    def test_unavailable_pyhwp_is_stated(self):
        m = load()
        payload = {
            "tasks": [
                {"task": "export-text", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 1, "ok": 1, "medianMs": 10}},
                    {"tool": "pyhwp", "available": False, "reason": "미설치"},
                ]},
            ],
        }
        text = " ".join(m.verdict_lines(payload))
        self.assertIn("실행하지 않았다", text)
        self.assertIn("미설치", text)

    def test_capability_verdict_comes_from_matrix(self):
        m = load()
        text = " ".join(m.verdict_lines({"capabilityMatrix": m.capability_matrix()}))
        self.assertIn("MCP", text)
        self.assertIn("rhwp 만", text)

    def test_refresh_verdict_overwrites_handwritten_claim(self):
        m = load()
        payload = {
            "tasks": [
                {"task": "export-text", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 2, "ok": 2, "medianMs": 10}},
                ]},
            ],
            "verdict": ["손으로 쓴 승패"],
        }
        refreshed = m.refresh_verdict(payload)
        self.assertNotIn("손으로 쓴 승패", refreshed["verdict"])
        self.assertTrue(any("2/2" in line for line in refreshed["verdict"]))


class CorpusSelectTests(unittest.TestCase):
    def test_per_ext_limit_and_sort(self):
        m = load()
        paths = [
            "samples/b.hwp",
            "samples/a.hwpx",
            "samples/z.txt",
            "samples/a.hwp",
            "samples/b.hwpx",
            "samples/c.hwp",
        ]
        picked = m.select_corpus_paths(paths, 2)
        self.assertEqual(picked, [
            "samples/a.hwp", "samples/b.hwp",
            "samples/a.hwpx", "samples/b.hwpx",
        ])

    def test_limit_zero_means_all_hwp_hwpx(self):
        m = load()
        paths = ["z.hwp", "a.hwpx", "skip.doc"]
        self.assertEqual(m.select_corpus_paths(paths, 0), ["z.hwp", "a.hwpx"])

    def test_backslash_and_duplicates_collapse(self):
        m = load()
        paths = [r"samples\a.hwp", "samples/a.hwp", "samples/a.HWP"]
        # POSIX 정규화 후 같은 문자열만 중복. 대소문자가 다른 경로는 별개다.
        picked = m.select_corpus_paths(paths, 0)
        self.assertEqual(picked.count("samples/a.hwp"), 1)
        self.assertIn("samples/a.HWP", picked)
        self.assertEqual(len(picked), 2)

    def test_negative_limit_is_all(self):
        m = load()
        self.assertEqual(
            m.select_corpus_paths(["b.hwp", "a.hwp"], -1),
            ["a.hwp", "b.hwp"],
        )


class ResolveToolTests(unittest.TestCase):
    def test_explicit_path_wins_when_exists(self):
        m = load()
        found = m.resolve_tool(
            "target/debug/rhwp", ["rhwp"],
            exists=lambda p: p == "target/debug/rhwp",
            which=lambda _n: None,
        )
        self.assertEqual(found, "target/debug/rhwp")

    def test_explicit_falls_back_to_which(self):
        m = load()
        found = m.resolve_tool(
            "rhwp", ["rhwp"],
            exists=lambda _p: False,
            which=lambda n: "/bin/rhwp" if n == "rhwp" else None,
        )
        self.assertEqual(found, "/bin/rhwp")

    def test_missing_is_none(self):
        m = load()
        self.assertIsNone(m.resolve_tool(
            None, ["hwp5txt", "soffice"],
            exists=lambda _p: False,
            which=lambda _n: None,
        ))

    def test_profile_from_path_uses_segment(self):
        m = load()
        self.assertEqual(m.rhwp_profile_from_path(r"C:\x\target\release\rhwp.exe"), "release")
        self.assertEqual(m.rhwp_profile_from_path("target/debug/rhwp"), "debug")
        self.assertEqual(m.rhwp_profile_from_path("target/prerelease/rhwp"), "debug")


class AssemblePayloadTests(unittest.TestCase):
    def test_kind_schema_and_derived_verdict(self):
        m = load()
        rhwp_runs = [
            {"file": "a.hwp", "ext": ".hwp", "ok": True, "ms": 10, "chars": 100},
            {"file": "b.hwpx", "ext": ".hwpx", "ok": True, "ms": 20, "chars": 80},
        ]
        py_runs = [
            {"file": "a.hwp", "ext": ".hwp", "ok": True, "ms": 4, "chars": 70},
            {"file": "b.hwpx", "ext": ".hwpx", "ok": False, "ms": 1, "chars": None},
        ]
        env = m.assemble_env(
            os_name="TestOS", python="3.11", rhwp_version="rhwp v0",
            rhwp_profile="debug", files=["a.hwp", "b.hwpx"],
            tools={"rhwp": {"available": True, "detail": "v0"}},
        )
        payload = m.assemble_payload(
            env=env,
            tasks=[{
                "task": "export-text",
                "results": [
                    m.available_result("rhwp", rhwp_runs, fidelity=1.0),
                    m.available_result(
                        "pyhwp", py_runs,
                        fidelity=m.fidelity_vs_ref(py_runs, rhwp_runs),
                        overlap=m.overlap_timing(py_runs, rhwp_runs),
                    ),
                    m.unavailable_result("soffice", "미설치"),
                    m.unavailable_result("hwplib", "CLI 아님"),
                ],
            }],
            generated_at="2026-01-01T00:00:00",
        )
        self.assertEqual(payload["kind"], m.REPORT_KIND)
        self.assertEqual(payload["schemaVersion"], m.SCHEMA_VERSION)
        self.assertEqual(payload["env"]["corpus"], {"dir": "samples", "total": 2, "hwp": 1, "hwpx": 1})
        self.assertEqual(payload["tasks"][0]["results"][1]["overlapMs"]["n"], 1)
        self.assertFalse(m.invented_metrics(payload["tasks"][0]["results"][2]))
        self.assertEqual(m.payload_shape_issues(payload), [])
        text = " ".join(payload["verdict"])
        self.assertIn("pyhwp", text)
        self.assertIn("더 빨", text)

    def test_unavailable_must_not_carry_numbers(self):
        m = load()
        clean = m.unavailable_result("hwplib", "CLI 아님")
        self.assertFalse(m.invented_metrics(clean))
        dirty = dict(clean)
        dirty["summary"] = {"ok": 3, "attempted": 3, "successRate": 1.0}
        self.assertTrue(m.invented_metrics(dirty))

    def test_shape_rejects_non_object_and_wrong_kind(self):
        m = load()
        self.assertTrue(m.payload_shape_issues("nope"))
        self.assertTrue(m.payload_shape_issues({"kind": "other", "tasks": []}))
        self.assertEqual(m.payload_shape_issues({"tasks": []}), [])


class SpeedCmpAndCellTests(unittest.TestCase):
    def test_speed_cmp_matrix(self):
        m = load()
        self.assertEqual(m.speed_cmp(8, 12), "tool_faster")
        self.assertEqual(m.speed_cmp(12, 8), "ref_faster")
        self.assertEqual(m.speed_cmp(8, 8), "tie")
        self.assertIsNone(m.speed_cmp(None, 8))
        self.assertIsNone(m.speed_cmp(True, 1))

    def test_fmt_cell_handles_missing_rate(self):
        m = load()
        cell = m._fmt_cell(True, {"attempted": 2, "ok": 2, "medianMs": 9}, None, None)
        self.assertIn("2/2", cell)
        self.assertIn("-", cell)
        self.assertIn("충실도", cell)

    def test_escape_md_cell_protects_table(self):
        m = load()
        self.assertEqual(m.escape_md_cell("a|b\nc"), "a\\|b c")


class MedianNumberTests(unittest.TestCase):
    def test_even_count_and_bool_filter(self):
        m = load()
        self.assertEqual(m.median([10, 30]), 20)
        self.assertIsNone(m.median([True, False, None]))
        self.assertEqual(m.median([True, 4, 6]), 5)


class WriteLfTests(unittest.TestCase):
    def test_dump_and_write_are_utf8_lf_no_bom(self):
        m = load()
        payload = m.assemble_payload(
            env={"os": "t"}, tasks=[], generated_at="2026-01-01T00:00:00",
        )
        text = m.dump_payload_json(payload)
        self.assertTrue(text.endswith("\n"))
        self.assertNotIn("\r", text)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "bench.json"
            m.write_text_lf(path, text)
            raw = path.read_bytes()
        self.assertFalse(raw.startswith(b"\xef\xbb\xbf"))
        self.assertNotIn(b"\r\n", raw)
        loaded = json.loads(raw.decode("utf-8"))
        self.assertEqual(loaded["kind"], "gymCompetitiveBench")
        self.assertEqual(loaded["verdict"], payload["verdict"])


class RenderHonestyTests(unittest.TestCase):
    def test_pipe_in_reason_does_not_break_table(self):
        m = load()
        payload = {
            "toolOrder": ["rhwp", "alt"],
            "env": {"tools": {}},
            "tasks": [
                {"task": "export-text", "results": [
                    {"tool": "rhwp", "available": False, "reason": "막힘|원인"},
                    {"tool": "alt", "available": False, "reason": "없음"},
                ]},
            ],
            "capabilityMatrix": m.capability_matrix(),
            "verdict": [],
        }
        md = m.render_report(payload)
        self.assertIn("n/a: 막힘\\|원인", md)
        self.assertIn("`gymCompetitiveBench`", md)

    def test_missing_tool_order_falls_back(self):
        m = load()
        md = m.render_report({
            "env": {},
            "tasks": [{"task": "info", "results": []}],
            "capabilityMatrix": m.capability_matrix(),
            "verdict": ["측정 없음"],
        })
        self.assertIn("| 과제 |", md)
        self.assertIn("rhwp", md)


class SummarizeEdgeTests(unittest.TestCase):
    def test_empty_ext_bucket_and_all_fail_median_none(self):
        m = load()
        s = m.summarize_runs([
            {"file": "no-suffix", "ok": False, "ms": 40, "chars": 9},
            {"file": "also-plain", "ok": False, "ms": 80, "chars": 3},
        ])
        self.assertEqual(s["attempted"], 2)
        self.assertEqual(s["ok"], 0)
        self.assertEqual(s["fail"], 2)
        self.assertEqual(s["successRate"], 0.0)
        self.assertIsNone(s["medianMs"])
        self.assertIsNone(s["medianChars"])
        self.assertEqual(s["byExt"][""], {"attempted": 2, "ok": 0, "fail": 2})

    def test_blank_ext_key_falls_back_to_file(self):
        m = load()
        s = m.summarize_runs([
            {"file": "doc.hwpx", "ext": "", "ok": True, "ms": 5, "chars": 2},
        ])
        self.assertEqual(s["byExt"][".hwpx"]["ok"], 1)


class DiscoverCorpusTests(unittest.TestCase):
    def test_temp_dir_limit_per_ext_ignores_txt(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "b.hwp").write_bytes(b"hwp-b")
            (root / "a.hwp").write_bytes(b"hwp-a")
            (root / "c.hwp").write_bytes(b"hwp-c")
            (root / "z.hwpx").write_bytes(b"hwpx-z")
            (root / "m.hwpx").write_bytes(b"hwpx-m")
            (root / "readme.txt").write_text("ignore", encoding="utf-8")
            (root / "notes.md").write_text("ignore", encoding="utf-8")
            (root / "nested").mkdir()
            (root / "nested" / "hidden.hwp").write_bytes(b"nope")
            picked = m.discover_corpus(str(root), 2)
        names = [Path(p).name for p in picked]
        self.assertEqual(names, ["a.hwp", "b.hwp", "m.hwpx", "z.hwpx"])
        self.assertTrue(all(n.endswith((".hwp", ".hwpx")) for n in names))
        self.assertNotIn("readme.txt", names)
        self.assertNotIn("hidden.hwp", names)

    def test_empty_dir_is_empty_list(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(m.discover_corpus(tmp, 5), [])

    def test_limit_zero_takes_all_matching(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "only.hwp").write_bytes(b"x")
            (root / "only.hwpx").write_bytes(b"y")
            picked = m.discover_corpus(str(root), 0)
        self.assertEqual([Path(p).name for p in picked], ["only.hwp", "only.hwpx"])


class ParseEnvelopeEdgeTests(unittest.TestCase):
    def test_parse_json_object_empty_and_malformed(self):
        m = load()
        self.assertIsNone(m.parse_json_object(""))
        self.assertIsNone(m.parse_json_object("   "))
        self.assertIsNone(m.parse_json_object("{"))
        self.assertIsNone(m.parse_json_object("null"))
        self.assertIsNone(m.parse_json_object("[]"))
        self.assertIsNone(m.parse_json_object(b"\xff"))
        self.assertIsNone(m.parse_json_body(""))
        self.assertIsNone(m.parse_json_body("not-json"))
        self.assertEqual(m.parse_json_object('{"a": 1}'), {"a": 1})

    def test_text_chars_empty_pages_and_non_dict(self):
        m = load()
        self.assertEqual(m.parse_rhwp_text_chars(json.dumps({"pages": []})), 0)
        mixed = {"pages": [
            "not-a-page",
            3,
            None,
            {"text": "가"},
            {"text": ["x"]},
            {"nope": "나"},
        ]}
        self.assertEqual(m.parse_rhwp_text_chars(json.dumps(mixed)), 1)
        wrapped = json.dumps({"result": {"pages": [{"text": "ab"}, {"text": "cd"}]}})
        self.assertEqual(m.parse_rhwp_text_chars(wrapped), 4)
        payload = json.dumps({"payload": {"text": "xyz"}})
        self.assertEqual(m.parse_rhwp_text_chars(payload), 3)

    def test_info_and_structure_malformed(self):
        m = load()
        self.assertIsNone(m.parse_rhwp_info(""))
        self.assertIsNone(m.parse_rhwp_info("{"))
        self.assertIsNone(m.parse_rhwp_info(json.dumps({"ok": True})))
        self.assertIsNone(m.parse_rhwp_structure(None))
        self.assertIsNone(m.parse_rhwp_structure("[]"))
        info = m.parse_rhwp_info(json.dumps({
            "result": {"format": "hwp3", "pageCount": True, "sections": 2},
        }))
        self.assertEqual(info["format"], "hwp3")
        self.assertIsNone(info["pageCount"])
        self.assertEqual(info["sections"], 2)
        struct = m.parse_rhwp_structure(json.dumps({
            "payload": {"mode": "clause", "nodeCount": 0, "structure": {"k": 1}},
        }))
        self.assertEqual(struct["nodeCount"], 0)
        self.assertTrue(struct["hasStructure"])


class VerdictBranchTests(unittest.TestCase):
    def _export(self, **kwargs):
        rhwp = kwargs.get("rhwp", {
            "tool": "rhwp", "available": True,
            "summary": {"attempted": 4, "ok": 4, "medianMs": 10.0, "byExt": {}},
        })
        pyhwp = kwargs.get("pyhwp")
        soffice = kwargs.get("soffice")
        results = [rhwp]
        if pyhwp is not None:
            results.append(pyhwp)
        if soffice is not None:
            results.append(soffice)
        extra = list(kwargs.get("extra_tasks") or [])
        return {"tasks": [{"task": "export-text", "results": results}, *extra]}

    def test_rhwp_faster_than_pyhwp(self):
        m = load()
        payload = self._export(pyhwp={
            "tool": "pyhwp", "available": True,
            "summary": {
                "attempted": 2, "ok": 2, "medianMs": 40.0,
                "byExt": {".hwp": {"attempted": 2, "ok": 2},
                          ".hwpx": {"attempted": 0, "ok": 0}},
            },
            "overlapMs": {"tool": 40.0, "ref": 9.0, "n": 2},
        })
        text = " ".join(m.verdict_lines(payload))
        self.assertIn("rhwp 가 더 빨랐다", text)
        self.assertIn("디버그", text)

    def test_soffice_unavailable_is_stated(self):
        m = load()
        payload = self._export(
            pyhwp={"tool": "pyhwp", "available": False, "reason": "six 없음"},
            soffice={"tool": "soffice", "available": False,
                     "reason": "미설치(이 머신); HWP5 필터 없음"},
        )
        text = " ".join(m.verdict_lines(payload))
        self.assertIn("LibreOffice(soffice)는 이 머신에서 실행하지 않았다", text)
        self.assertIn("미설치", text)

    def test_no_overlap_speed_is_honest(self):
        m = load()
        payload = self._export(pyhwp={
            "tool": "pyhwp", "available": True,
            "summary": {
                "attempted": 2, "ok": 1,
                "byExt": {".hwp": {"attempted": 1, "ok": 1},
                          ".hwpx": {"attempted": 1, "ok": 0}},
            },
            "overlapMs": {"n": 0, "tool": None, "ref": None},
        })
        text = " ".join(m.verdict_lines(payload))
        self.assertIn("겹친 파일이 없어", text)
        self.assertIn("동일-집합 비교를 만들지 않았다", text)

    def test_hwpx_zero_of_n_is_stated(self):
        m = load()
        payload = self._export(pyhwp={
            "tool": "pyhwp", "available": True,
            "summary": {
                "attempted": 5, "ok": 2,
                "byExt": {".hwp": {"attempted": 2, "ok": 2},
                          ".hwpx": {"attempted": 3, "ok": 0}},
            },
            "overlapMs": {"tool": 5.0, "ref": 5.0, "n": 2},
        })
        text = " ".join(m.verdict_lines(payload))
        self.assertIn("HWPX 0/3", text)

    def test_width_lists_rhwp_only_tasks(self):
        m = load()
        payload = self._export(
            pyhwp={"tool": "pyhwp", "available": False, "reason": "n/a"},
            extra_tasks=[
                {"task": "info", "results": [
                    {"tool": "rhwp", "available": True, "summary": {"ok": 1, "attempted": 1}},
                    {"tool": "pyhwp", "available": False},
                ]},
                {"task": "structure", "results": [
                    {"tool": "rhwp", "available": True, "summary": {"ok": 1, "attempted": 1}},
                    {"tool": "soffice", "available": False},
                ]},
                {"task": "convert", "results": [
                    {"tool": "rhwp", "available": True, "summary": {"ok": 1, "attempted": 1}},
                ]},
            ],
        )
        text = " ".join(m.verdict_lines(payload))
        self.assertIn("info, structure, convert", text)


class FmtCellEdgeTests(unittest.TestCase):
    def test_attempted_zero_is_na(self):
        m = load()
        self.assertEqual(m._fmt_cell(True, {"attempted": 0, "ok": 0}, 1.0, None), "n/a: 시도 없음")
        self.assertEqual(m._fmt_cell(True, None, None, None), "n/a: 시도 없음")
        self.assertEqual(m._fmt_cell(True, "bad", None, None), "n/a: 시도 없음")

    def test_fidelity_none_prints_dash(self):
        m = load()
        cell = m._fmt_cell(True, {
            "attempted": 3, "ok": 3, "successRate": 1.0, "medianMs": 11.0,
        }, None, None)
        self.assertIn("충실도 -", cell)
        self.assertIn("11ms", cell)
        self.assertNotIn("None", cell)


class ReportContractTests(unittest.TestCase):
    def test_constants(self):
        m = load()
        self.assertEqual(m.REPORT_KIND, "gymCompetitiveBench")
        self.assertEqual(m.SCHEMA_VERSION, "1.0")
        self.assertEqual(set(m.VALID_CAP), {"yes", "partial", "no"})
        self.assertEqual(tuple(m.DEFAULT_TOOL_ORDER), ("rhwp", "pyhwp", "soffice", "hwplib"))

    def test_stamp_fills_missing_kind(self):
        m = load()
        stamped = m.stamp_report_contract({"tasks": []})
        self.assertEqual(stamped["kind"], "gymCompetitiveBench")
        self.assertEqual(stamped["schemaVersion"], "1.0")
        kept = m.stamp_report_contract({
            "kind": "gymCompetitiveBench", "schemaVersion": "1.0", "tasks": [],
        })
        self.assertEqual(kept["kind"], m.REPORT_KIND)

    def test_load_report_payload_empty_and_bad(self):
        m = load()
        self.assertIsNone(m.load_report_payload("")[0])
        self.assertIsNone(m.load_report_payload("{")[0])
        self.assertIsNone(m.load_report_payload(json.dumps({"kind": "nope"}))[0])
        payload, issues = m.load_report_payload(json.dumps({"tasks": []}))
        self.assertEqual(issues, [])
        self.assertEqual(payload["kind"], "gymCompetitiveBench")
        self.assertIn("verdict", payload)


class FromJsonCliTests(unittest.TestCase):
    def _tiny_payload(self, m):
        return {
            "tasks": [
                {"task": "export-text", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 1, "ok": 1, "successRate": 1.0,
                                 "medianMs": 7.0, "medianChars": 4,
                                 "byExt": {".hwp": {"attempted": 1, "ok": 1, "fail": 0}}},
                     "fidelityVsRhwp": 1.0},
                    {"tool": "pyhwp", "available": False, "reason": "미설치"},
                    {"tool": "soffice", "available": False, "reason": "미설치(이 머신)"},
                ]},
            ],
            "env": {
                "os": "TestOS", "python": "3.11", "rhwpVersion": "rhwp v0",
                "rhwpProfile": "debug",
                "corpus": {"dir": "tmp", "total": 1, "hwp": 1, "hwpx": 0},
                "tools": {"rhwp": {"available": True, "detail": "v0"}},
            },
            "capabilityMatrix": m.capability_matrix(),
        }

    def test_from_json_prints_markdown(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp) / "bench.json"
            src.write_text(json.dumps(self._tiny_payload(m), ensure_ascii=False), encoding="utf-8")
            out, err = io.StringIO(), io.StringIO()
            with redirect_stdout(out), redirect_stderr(err):
                rc = m.main(["--from-json", str(src)])
        self.assertEqual(rc, 0)
        md = out.getvalue()
        self.assertIn("# 경쟁 벤치마크", md)
        self.assertIn("`gymCompetitiveBench`", md)
        self.assertIn("n/a: 미설치", md)

    def test_from_json_writes_stamped_json_and_md(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp) / "in.json"
            out_json = Path(tmp) / "out.json"
            out_md = Path(tmp) / "out.md"
            raw = dict(self._tiny_payload(m))
            # 옛 산출물처럼 kind 가 없어도 재렌더 JSON 에 계약이 찍혀야 한다.
            src.write_text(json.dumps(raw, ensure_ascii=False), encoding="utf-8")
            err = io.StringIO()
            with redirect_stdout(io.StringIO()), redirect_stderr(err):
                rc = m.main([
                    "--from-json", str(src),
                    "--out-json", str(out_json),
                    "--out-md", str(out_md),
                ])
            self.assertEqual(rc, 0)
            raw_bytes = out_json.read_bytes()
            stamped = json.loads(raw_bytes.decode("utf-8"))
            md = out_md.read_text(encoding="utf-8")
        self.assertFalse(raw_bytes.startswith(b"\xef\xbb\xbf"))
        self.assertEqual(stamped["kind"], "gymCompetitiveBench")
        self.assertEqual(stamped["schemaVersion"], "1.0")
        self.assertTrue(any("실행하지 않았다" in line for line in stamped["verdict"]))
        self.assertIn("gymCompetitiveBench", md)

    def test_from_json_rejects_malformed(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp) / "bad.json"
            src.write_text("{", encoding="utf-8")
            err = io.StringIO()
            with redirect_stdout(io.StringIO()), redirect_stderr(err):
                rc = m.main(["--from-json", str(src)])
        self.assertEqual(rc, 2)
        self.assertIn("깨졌다", err.getvalue())


class ValidateMatrixExtraTests(unittest.TestCase):
    def test_non_object_and_empty_sections(self):
        m = load()
        self.assertEqual(m.validate_capability_matrix("x"), ["matrix 가 객체가 아니다"])
        self.assertIn("columns 가 비었다", m.validate_capability_matrix({"columns": [], "rows": []})[0])
        self.assertIn("rows 가 비었다", m.validate_capability_matrix({
            "columns": [{"key": "mcp", "label": "MCP"}], "rows": [],
        })[0])

    def test_rhwp_must_be_all_yes(self):
        m = load()
        matrix = {
            "columns": [{"key": "mcp", "label": "MCP"}, {"key": "edit", "label": "편집"}],
            "rows": [{"tool": "rhwp", "mcp": "yes", "edit": "partial"}],
        }
        issues = m.validate_capability_matrix(matrix)
        self.assertTrue(any("rhwp.edit" in i for i in issues))

    def test_column_without_key_and_non_dict_row(self):
        m = load()
        issues = m.validate_capability_matrix({
            "columns": [{"label": "없음"}, {"key": "mcp", "label": "MCP"}],
            "rows": ["bad", {"tool": "alt", "mcp": "no"}],
        })
        self.assertTrue(any("key 가 없다" in i for i in issues))
        self.assertTrue(any("객체가 아니다" in i for i in issues))


if __name__ == "__main__":
    unittest.main()
