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


class CommandsInDocTests(unittest.TestCase):
    """과제·기준풀이 문서에서 명령 첫 토큰만 모은다."""

    def test_empty_doc_is_empty_set(self):
        self.assertEqual(load().commands_in_doc({}), set())

    def test_ignores_empty_cmd_and_missing_op(self):
        used = load().commands_in_doc({
            "checks": [
                {"op": "file_exists", "file": "out.hwp"},
                {"op": "value_eq", "cmd": []},
                {"cmd": ["search", "x"]},
            ]
        })
        self.assertEqual(used, {"search"})

    def test_steps_run_and_nested_answer_cmds(self):
        used = load().commands_in_doc({
            "steps": [
                {"run": ["batch", "fill"]},
                {"run": []},
                {
                    "answer": {
                        "n": {"cmd": ["export-text", "a"]},
                        "m": "not-a-spec",
                        "k": {"path": "x"},
                    }
                },
            ]
        })
        self.assertEqual(used, {"batch", "export-text"})

    def test_answer_non_dict_is_ignored(self):
        used = load().commands_in_doc({"steps": [{"answer": ["bad"]}]})
        self.assertEqual(used, set())


class OperatorsInDocTests(unittest.TestCase):
    def test_collects_only_named_ops(self):
        ops = load().operators_in_doc({
            "checks": [
                {"op": "value_eq"},
                {"op": "file_exists"},
                {"name": "이름만"},
                {},
            ]
        })
        self.assertEqual(ops, {"value_eq", "file_exists"})

    def test_reference_without_checks_is_empty(self):
        self.assertEqual(load().operators_in_doc({"id": "X", "steps": []}), set())


class ListPackIdsTests(unittest.TestCase):
    def test_requires_pack_json_and_sorts(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "packs" / "zeta").mkdir(parents=True)
            (root / "packs" / "alpha").mkdir(parents=True)
            (root / "packs" / "alpha" / "pack.json").write_text("{}", encoding="utf-8")
            (root / "packs" / "zeta" / "pack.json").write_text("{}", encoding="utf-8")
            (root / "packs" / "no-manifest").mkdir()
            (root / "packs" / "file-not-dir.txt").write_text("x", encoding="utf-8")
            ids = cov.list_pack_ids(str(root))
        self.assertEqual(ids, ["alpha", "zeta"])

    def test_missing_packs_dir_is_empty(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(cov.list_pack_ids(tmp), [])


class MeasureEdgeTests(unittest.TestCase):
    def test_coverage_percent_truncates_toward_zero(self):
        cmds = [
            {"name": "a", "category": "query"},
            {"name": "b", "category": "query"},
            {"name": "c", "category": "edit"},
        ]
        r = load().measure(cmds, used={"a"})
        self.assertEqual(r["covered"], 1)
        self.assertEqual(r["uncovered"], 2)
        self.assertEqual(r["coveragePercent"], 33)

    def test_full_coverage_is_one_hundred(self):
        cmds = [
            {"name": "info", "category": "query"},
            {"name": "search", "category": "query"},
        ]
        r = load().measure(cmds, used={"info", "search", "extra-not-in-denom"})
        self.assertEqual(r["coveragePercent"], 100)
        self.assertEqual(r["uncovered"], 0)
        self.assertEqual(r["coveredCommands"], ["info", "search"])

    def test_uncovered_by_category_is_sorted(self):
        cmds = [
            {"name": "zeta", "category": "edit"},
            {"name": "alpha", "category": "edit"},
            {"name": "mid", "category": "query"},
        ]
        r = load().measure(cmds, used=set())
        self.assertEqual(r["uncoveredByCategory"]["edit"], ["alpha", "zeta"])
        self.assertEqual(r["uncoveredByCategory"]["query"], ["mid"])

    def test_internal_category_is_excluded(self):
        cmds = [
            {"name": "info", "category": "query"},
            {"name": "secret", "category": "internal"},
        ]
        r = load().measure(cmds, used=set())
        self.assertEqual(r["agentFacingTotal"], 1)
        self.assertIn("secret", r["excludedNonAgent"])
        self.assertNotIn("secret", r["uncoveredByCategory"].get("internal", []))

    def test_duplicate_command_names_count_once(self):
        cmds = [
            {"name": "info", "category": "query"},
            {"name": "info", "category": "query"},
        ]
        r = load().measure(cmds, used={"info"})
        self.assertEqual(r["agentFacingTotal"], 1)
        self.assertEqual(r["covered"], 1)

    def test_sorted_pack_grid_empty_and_none(self):
        cov = load()
        self.assertEqual(cov._sorted_pack_grid(None), {})
        self.assertEqual(cov._sorted_pack_grid({}), {})
        self.assertEqual(
            cov._sorted_pack_grid({"b": ["z", "a"], "a": ["m"]}),
            {"a": ["m"], "b": ["a", "z"]},
        )

    def test_schema_keys_are_stable(self):
        r = load().measure(FIXTURE, used={"info"})
        self.assertEqual(
            set(r),
            {
                "kind",
                "schemaVersion",
                "agentFacingTotal",
                "covered",
                "uncovered",
                "coveragePercent",
                "uncoveredByCategory",
                "coveredCommands",
                "excludedNonAgent",
                "packs",
                "unusedOperators",
            },
        )


class FormatHumanTests(unittest.TestCase):
    def test_full_coverage_message(self):
        text = load().format_human(
            load().measure(
                [{"name": "info", "category": "query"}],
                used={"info"},
                packs={"core-cli": ["info"]},
                unused_operators=[],
            )
        )
        self.assertIn("전부 노출됨", text)
        self.assertIn("[core-cli] info", text)
        self.assertIn("미사용 연산자 없음", text)

    def test_empty_pack_row_prints_none(self):
        text = load().format_human(
            load().measure(
                FIXTURE,
                used={"info"},
                packs={"beta": []},
                unused_operators=["value_in"],
            )
        )
        self.assertIn("[beta] (없음)", text)
        self.assertIn("미사용 연산자 (1): value_in", text)

    def test_no_pack_scan_message(self):
        text = load().format_human(load().measure(FIXTURE, used={"info"}))
        self.assertIn("pack 스캔 없음", text)

    def test_ends_with_newline(self):
        text = load().format_human(load().measure(FIXTURE, used={"info"}))
        self.assertTrue(text.endswith("\n"))


class UnusedOperatorEdgeTests(unittest.TestCase):
    def test_injected_registry_does_not_touch_live_checks(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _mini_gym(root)
            unused = cov.unused_operators(str(root), registry={"only_me"})
        self.assertEqual(unused, ["only_me"])

    def test_empty_registry_means_nothing_unused(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _mini_gym(root)
            unused = cov.unused_operators(str(root), registry=set())
        self.assertEqual(unused, [])

    def test_reference_ops_are_not_counted_as_used(self):
        """기준풀이에는 checks 가 없다 — 과제 op 만 사용으로 친다."""
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(root / "packs" / "solo" / "pack.json", {"id": "solo"})
            _write(
                root / "packs" / "solo" / "tasks" / "S01.json",
                {"id": "S01", "checks": [{"op": "file_exists", "file": "x"}]},
            )
            _write(
                root / "packs" / "solo" / "reference" / "S01.json",
                {"id": "S01", "steps": [{"run": ["info"]}]},
            )
            used = cov.used_operators(str(root))
            unused = cov.unused_operators(
                str(root), registry={"file_exists", "answer_eq"}
            )
        self.assertEqual(used, {"file_exists"})
        self.assertEqual(unused, ["answer_eq"])


class RealGymGridContractTests(unittest.TestCase):
    """실제 gym 격자가 이번 PR 의 얇은 pack 확장을 반영한다."""

    def test_extraction_commands_include_new_kinds(self):
        grid = load().used_commands_by_pack(str(REPO_ROOT / "gym"))
        for cmd in ("chart-to-csv", "export-text", "extract-data"):
            self.assertIn(cmd, grid["extraction"], cmd)

    def test_table_csv_commands_include_roundtrip(self):
        grid = load().used_commands_by_pack(str(REPO_ROOT / "gym"))
        for cmd in ("table-to-csv", "csv-to-table", "export-tables"):
            self.assertIn(cmd, grid["table-csv"], cmd)

    def test_batch_ops_commands_include_fill_and_search(self):
        grid = load().used_commands_by_pack(str(REPO_ROOT / "gym"))
        self.assertIn("batch", grid["batch-ops"])
        self.assertIn("search", grid["batch-ops"])

    def test_grid_rows_match_pack_json_folders(self):
        cov = load()
        gym = str(REPO_ROOT / "gym")
        self.assertEqual(sorted(cov.used_commands_by_pack(gym)), cov.list_pack_ids(gym))

    def test_report_on_real_gym_keeps_legacy_and_grid(self):
        cov = load()
        r = cov.report(FIXTURE, str(REPO_ROOT / "gym"))
        self.assertEqual(r["kind"], "gymCoverage")
        self.assertGreaterEqual(len(r["packs"]), 12)
        self.assertIn("extraction", r["packs"])
        self.assertTrue(r["unusedOperators"] == sorted(r["unusedOperators"]))


class CliTests(unittest.TestCase):
    def _cap(self, path: Path) -> None:
        path.write_text(
            json.dumps({"commands": FIXTURE}, ensure_ascii=False),
            encoding="utf-8",
        )

    def test_json_flag_emits_schema(self):
        import subprocess

        with tempfile.TemporaryDirectory() as tmp:
            cap = Path(tmp) / "cap.json"
            self._cap(cap)
            proc = subprocess.run(
                ["python", str(TOOL), "--capabilities", str(cap), "--json"],
                capture_output=True,
                text=True,
                encoding="utf-8",
                cwd=str(REPO_ROOT),
            )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["kind"], "gymCoverage")
        self.assertEqual(payload["schemaVersion"], "1.0")
        self.assertIn("packs", payload)
        self.assertIn("unusedOperators", payload)

    def test_human_output_mentions_grid(self):
        import subprocess

        with tempfile.TemporaryDirectory() as tmp:
            cap = Path(tmp) / "cap.json"
            self._cap(cap)
            proc = subprocess.run(
                ["python", str(TOOL), "--capabilities", str(cap)],
                capture_output=True,
                text=True,
                encoding="utf-8",
                cwd=str(REPO_ROOT),
            )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("pack×명령 격자", proc.stdout)
        self.assertIn("미사용 연산자", proc.stdout)

    def test_missing_source_exits_two(self):
        import subprocess

        proc = subprocess.run(
            ["python", str(TOOL)],
            capture_output=True,
            text=True,
            encoding="utf-8",
            cwd=str(REPO_ROOT),
        )
        self.assertEqual(proc.returncode, 2)
        self.assertIn("--bin", proc.stderr)
        self.assertIn("--capabilities", proc.stderr)


class IterPackDocsTests(unittest.TestCase):
    def test_skips_non_json_and_missing_subdir(self):
        cov = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(root / "packs" / "alpha" / "pack.json", {"id": "alpha"})
            _write(root / "packs" / "alpha" / "tasks" / "A01.json", {"id": "A01"})
            (root / "packs" / "alpha" / "tasks" / "notes.txt").write_text("x", encoding="utf-8")
            (root / "packs" / "beta").mkdir(parents=True)
            docs = list(cov.iter_pack_docs(str(root), "tasks"))
        self.assertEqual([p for p, _, _ in docs], ["alpha"])
        self.assertEqual(docs[0][2]["id"], "A01")


class UnknownCategoryAndOverlapTests(unittest.TestCase):
    """분모 정직성 — 모르는 카테고리·빈 used·이름 겹침."""

    def test_unknown_category_is_neither_agent_nor_excluded(self):
        cmds = [
            {"name": "mystery", "category": "unknown-cat"},
            {"name": "info", "category": "query"},
            {"name": "hwp5-inventory", "category": "diagnostic"},
        ]
        r = load().measure(cmds, used={"info", "mystery"})
        self.assertEqual(r["agentFacingTotal"], 1)
        self.assertEqual(r["coveredCommands"], ["info"])
        self.assertNotIn("mystery", r["coveredCommands"])
        self.assertNotIn("mystery", r["excludedNonAgent"])
        flat = [n for names in r["uncoveredByCategory"].values() for n in names]
        self.assertNotIn("mystery", flat)
        self.assertIn("hwp5-inventory", r["excludedNonAgent"])

    def test_empty_used_set_covers_nothing_of_agent_facing(self):
        r = load().measure(FIXTURE, used=set())
        self.assertEqual(r["covered"], 0)
        self.assertEqual(r["uncovered"], 2)
        self.assertEqual(r["coveragePercent"], 0)
        self.assertEqual(r["coveredCommands"], [])
        flat = [n for names in r["uncoveredByCategory"].values() for n in names]
        self.assertEqual(set(flat), {"info", "csv-to-table"})

    def test_empty_used_and_empty_denominator_is_full(self):
        r = load().measure([], used=set())
        self.assertEqual(r["agentFacingTotal"], 0)
        self.assertEqual(r["coveragePercent"], 100)
        self.assertEqual(r["uncovered"], 0)

    def test_overlapping_names_across_categories_count_once(self):
        cmds = [
            {"name": "info", "category": "query"},
            {"name": "info", "category": "query"},
            {"name": "search", "category": "query"},
            {"name": "search", "category": "export"},
        ]
        r = load().measure(cmds, used={"info", "search"})
        self.assertEqual(r["agentFacingTotal"], 2)
        self.assertEqual(r["covered"], 2)
        self.assertEqual(r["uncovered"], 0)
        self.assertEqual(r["coveredCommands"], ["info", "search"])

    def test_used_outside_agent_facing_does_not_inflate_covered(self):
        r = load().measure(FIXTURE, used={"info", "hwp5-inventory", "mcp-serve"})
        self.assertEqual(r["covered"], 1)
        self.assertEqual(r["coveredCommands"], ["info"])
        self.assertNotIn("hwp5-inventory", r["coveredCommands"])

    def test_real_gym_expanded_packs_expose_expected_ops_and_cmds(self):
        cov = load()
        gym = str(REPO_ROOT / "gym")
        used = cov.used_commands(gym)
        ops = cov.used_operators(gym)
        grid = cov.used_commands_by_pack(gym)
        for cmd in (
            "extract-data",
            "export-text",
            "chart-to-csv",
            "table-to-csv",
            "csv-to-table",
            "export-tables",
            "batch",
            "search",
        ):
            self.assertIn(cmd, used, cmd)
        for op in (
            "answer_eq",
            "len_answer_eq",
            "csv_cell_eq",
            "utf8_bom",
            "cell_text_eq",
            "json_value_eq",
            "value_eq",
            "value_ge",
            "file_exists",
            "differs_from_input",
        ):
            self.assertIn(op, ops, op)
        self.assertEqual(len(grid["extraction"]), len(set(grid["extraction"])))
        self.assertEqual(len(grid["table-csv"]), len(set(grid["table-csv"])))
        self.assertEqual(len(grid["batch-ops"]), len(set(grid["batch-ops"])))


if __name__ == "__main__":
    unittest.main()

