"""[report] gym 능력 리포트 계약 — 축 프로파일 집계 + 정확도/커버리지 분리 불변식.

핵심 불변식: 정확도(측정된 것 통과율)와 커버리지(측정 폭)는 **다른 것**이라 뭉뚱
그리지 않는다. 축별 프로파일은 pack 의 axis 라벨(괄호 앞 차원)로 점수를 합산한다.
바이너리 없이 순수 합성만 시험한다.
"""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / "gym" / "report.py"


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


if __name__ == "__main__":
    unittest.main()
