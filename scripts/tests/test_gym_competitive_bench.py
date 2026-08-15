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
import json
import unittest
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
        self.assertEqual(s["byExt"][".hwp"], {"attempted": 2, "ok": 2})
        # HWPX 는 시도했으나 실패 — pyhwp 형식 한계가 데이터로 남는다.
        self.assertEqual(s["byExt"][".hwpx"], {"attempted": 1, "ok": 0})

    def test_empty_runs_safe(self):
        m = load()
        s = m.summarize_runs([])
        self.assertEqual(s["attempted"], 0)
        self.assertIsNone(s["successRate"])
        self.assertIsNone(s["medianMs"])


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


class RhwpParseTests(unittest.TestCase):
    def test_sums_page_texts(self):
        m = load()
        env = json.dumps({"pages": [{"text": "가나다"}, {"text": "라마"}]})
        self.assertEqual(m.parse_rhwp_text_chars(env), 5)

    def test_bad_json_returns_none(self):
        m = load()
        self.assertIsNone(m.parse_rhwp_text_chars("not json"))
        self.assertIsNone(m.parse_rhwp_text_chars(json.dumps({"nope": 1})))


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


if __name__ == "__main__":
    unittest.main()
