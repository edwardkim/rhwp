"""[coverage] gym 커버리지 측정기 계약 — 분모 정직성 불변식.

핵심 불변식: 커버리지 분모는 **에이전트-대면 카테고리만**이다. diagnostic(hwp5-*·
dump-* 개발 probe)·internal·serve(인프라)를 분모에 넣으면 커버리지가 실제보다 낮게
나와, 진단 도구를 '빈 곳'으로 오인하게 만든다. 바이너리 없이 순수 로직만 시험한다.
"""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "coverage.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_coverage", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


FIXTURE = [
    {"name": "info", "category": "query"},                # 에이전트-대면, 노출
    {"name": "csv-to-table", "category": "edit"},          # 에이전트-대면, 미노출
    {"name": "hwp5-inventory", "category": "diagnostic"},  # 제외(개발 probe)
    {"name": "mcp-serve", "category": "serve"},            # 제외(인프라)
]


class CoverageTests(unittest.TestCase):
    def test_denominator_is_agent_facing_only(self):
        r = load().measure(FIXTURE, used={"info"})
        # 분모 = 에이전트-대면 2개(info·csv-to-table), 진단·serve 제외.
        self.assertEqual(r["agentFacingTotal"], 2)
        self.assertEqual(r["covered"], 1)
        self.assertEqual(r["uncovered"], 1)
        self.assertEqual(r["coveragePercent"], 50)

    def test_diagnostic_not_counted_as_gap(self):
        r = load().measure(FIXTURE, used={"info"})
        flat = [n for names in r["uncoveredByCategory"].values() for n in names]
        self.assertIn("csv-to-table", flat)
        self.assertNotIn("hwp5-inventory", flat)  # 진단은 빈 곳 아님
        self.assertNotIn("mcp-serve", flat)
        self.assertIn("hwp5-inventory", r["excludedNonAgent"])

    def test_empty_denominator_is_full_not_zero_division(self):
        r = load().measure([{"name": "x", "category": "diagnostic"}], used=set())
        # 잴 게 없으면 빈 곳도 없다 — 0 나누기 대신 100.
        self.assertEqual(r["agentFacingTotal"], 0)
        self.assertEqual(r["coveragePercent"], 100)

    def test_real_gym_used_commands_is_measurable(self):
        # 실제 gym 팩을 스캔해도 예외 없이 집합을 낸다(순수 로직 경로).
        cov = load()
        used = cov.used_commands(str(REPO_ROOT / "gym"))
        self.assertIsInstance(used, set)
        self.assertIn("info", used)  # core-cli 가 info 를 쓴다


if __name__ == "__main__":
    unittest.main()
