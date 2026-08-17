"""[coverage] gym 커버리지 측정기 계약 — 분모 정직성 불변식.

핵심 불변식: 커버리지 분모는 **에이전트-대면 카테고리만**이다. diagnostic(hwp5-*·
dump-* 개발 probe)·internal·serve(인프라)를 분모에 넣으면 커버리지가 실제보다 낮게
나와, 진단 도구를 '빈 곳'으로 오인하게 만든다. 바이너리 없이 순수 로직만 시험한다.

#5208 이 더한 것: pack×명령 격자와 REGISTRY 미사용 연산자. 기존 키의 의미는
그대로 두고, 같은 봉투에 packs·unusedOperators 를 붙인다.
"""

from __future__ import annotations

import importlib.util
import json
import tempfile
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


def _write(path: Path, doc: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(doc, ensure_ascii=False), encoding="utf-8")


def _mini_gym(root: Path) -> None:
    """두 pack — alpha 는 명령·연산자를 쓰고, beta 는 빈 격자 행."""
    _write(root / "packs" / "alpha" / "pack.json", {"id": "alpha"})
    _write(
        root / "packs" / "alpha" / "tasks" / "A01.json",
        {
            "id": "A01",
            "checks": [
                {"op": "value_eq", "cmd": ["search", "{input}", "--json"]},
                {"op": "file_exists", "file": "out.hwp"},
            ],
        },
    )
    _write(
        root / "packs" / "alpha" / "reference" / "A01.json",
        {
            "id": "A01",
            "steps": [
                {"run": ["batch", "fill", "--form", "{input}"]},
                {
                    "answer": {
                        "hits": {
                            "cmd": ["export-text", "{input}", "--json"],
                            "path": "pageCount",
                        }
                    }
                },
            ],
        },
    )
    _write(root / "packs" / "beta" / "pack.json", {"id": "beta"})
    _write(
        root / "packs" / "beta" / "tasks" / "B01.json",
        {"id": "B01", "checks": [{"op": "differs_from_input", "file": "out.hwp"}]},
    )


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

    def test_measure_keeps_schema_and_adds_grid_keys(self):
        r = load().measure(FIXTURE, used={"info"})
        self.assertEqual(r["kind"], "gymCoverage")
        self.assertEqual(r["schemaVersion"], "1.0")
        # 새 키는 항상 있고, 스캔을 넘기지 않으면 빈 값이다(순수 함수).
        self.assertEqual(r["packs"], {})
        self.assertEqual(r["unusedOperators"], [])

    def test_measure_embeds_pack_grid_and_unused_operators(self):
        r = load().measure(
            FIXTURE,
            used={"info"},
            packs={"table-csv": ["csv-to-table", "table-to-csv"],
                   "extraction": ["export-text"]},
            unused_operators=["value_in", "deep_contains"],
        )
        self.assertEqual(r["kind"], "gymCoverage")
        self.assertEqual(r["schemaVersion"], "1.0")
        # 기존 키는 그대로.
        self.assertEqual(r["agentFacingTotal"], 2)
        self.assertEqual(r["covered"], 1)
        # 격자 행·명령은 정렬된다.
        self.assertEqual(list(r["packs"]), ["extraction", "table-csv"])
        self.assertEqual(r["packs"]["table-csv"], ["csv-to-table", "table-to-csv"])
        self.assertEqual(r["unusedOperators"], ["deep_contains", "value_in"])


class PackGridTests(unittest.TestCase):
    def test_groups_commands_per_pack_and_sorts(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _mini_gym(root)
            grid = cov.used_commands_by_pack(str(root))
        self.assertEqual(set(grid), {"alpha", "beta"})
        # 과제 cmd + 기준풀이 run + answer.cmd 를 모으고 정렬한다.
        self.assertEqual(grid["alpha"], ["batch", "export-text", "search"])
        # 명령이 없는 pack 도 빈 행으로 남긴다 — 누락과 빈 곳을 구분한다.
        self.assertEqual(grid["beta"], [])

    def test_does_not_leak_commands_across_packs(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _mini_gym(root)
            grid = cov.used_commands_by_pack(str(root))
        self.assertNotIn("search", grid["beta"])
        self.assertNotIn("batch", grid["beta"])

    def test_used_commands_includes_reference_answer_cmds(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _mini_gym(root)
            used = cov.used_commands(str(root))
        self.assertEqual(used, {"search", "batch", "export-text"})

    def test_real_gym_pack_grid_covers_known_thin_packs(self):
        cov = load()
        grid = cov.used_commands_by_pack(str(REPO_ROOT / "gym"))
        self.assertIn("extraction", grid)
        self.assertIn("table-csv", grid)
        self.assertIn("batch-ops", grid)
        self.assertIn("chart-to-csv", grid["extraction"])
        self.assertIn("export-text", grid["extraction"])
        self.assertIn("csv-to-table", grid["table-csv"])
        self.assertIn("batch", grid["batch-ops"])
        for pid, cmds in grid.items():
            self.assertEqual(cmds, sorted(cmds), pid)
            self.assertEqual(len(cmds), len(set(cmds)), pid)

    def test_report_combines_scan_with_legacy_keys(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _mini_gym(root)
            r = cov.report(FIXTURE, str(root))
        self.assertEqual(r["kind"], "gymCoverage")
        self.assertEqual(r["schemaVersion"], "1.0")
        self.assertEqual(r["agentFacingTotal"], 2)
        self.assertEqual(r["packs"]["alpha"], ["batch", "export-text", "search"])
        self.assertEqual(r["packs"]["beta"], [])
        # 픽스처 분모의 노출 명령은 gym 스캔 결과와 교집합.
        self.assertEqual(r["coveredCommands"], [])
        self.assertIn("csv-to-table", r["uncoveredByCategory"]["edit"])


class UnusedOperatorTests(unittest.TestCase):
    def test_unused_operators_are_registry_minus_task_ops(self):
        cov = load()
        registry = {"value_eq", "file_exists", "value_in", "deep_contains"}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _mini_gym(root)
            unused = cov.unused_operators(str(root), registry=registry)
            used = cov.used_operators(str(root))
        # 과제 op 만 센다 — 기준풀이에는 checks 가 없다.
        self.assertEqual(used, {"value_eq", "file_exists", "differs_from_input"})
        self.assertEqual(unused, ["deep_contains", "value_in"])

    def test_registered_operators_match_checks_registry(self):
        cov = load()
        names = cov.registered_operators()
        self.assertIn("answer_eq", names)
        self.assertIn("value_eq", names)
        self.assertIn("utf8_bom", names)
        self.assertTrue(names)  # 빈 등록부는 도구 파손

    def test_real_gym_unused_operators_are_registered_and_sorted(self):
        cov = load()
        gym = str(REPO_ROOT / "gym")
        unused = cov.unused_operators(gym)
        registry = cov.registered_operators()
        used = cov.used_operators(gym)
        self.assertEqual(set(unused), set(registry) - used)
        self.assertEqual(unused, sorted(unused))
        self.assertTrue(set(unused).issubset(registry))
        # 라이브 오라클의 기본 연산자는 반드시 쓰인다.
        self.assertNotIn("answer_eq", unused)
        self.assertNotIn("file_exists", unused)

    def test_format_human_mentions_grid_and_unused_ops(self):
        cov = load()
        text = cov.format_human(
            cov.measure(
                FIXTURE,
                used={"info"},
                packs={"extraction": ["export-text"]},
                unused_operators=["value_in"],
            )
        )
        self.assertIn("pack×명령 격자", text)
        self.assertIn("[extraction] export-text", text)
        self.assertIn("미사용 연산자", text)
        self.assertIn("value_in", text)


if __name__ == "__main__":
    unittest.main()
