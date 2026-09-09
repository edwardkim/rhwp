"""extraction pack 계약 — 읽기 축·명령 화이트리스트·과제↔기준 1:1.

이 가드는 커버리지·추출 확장(#5212)의 pack 내부 불변식을 CI 가 다시 본다.
전 pack 정합은 test_gym_packs · audit.py 가 보고, 여기는 extraction 만 본다.

고정하는 것:
- 과제 id EX01+ 전부 기준 풀이 1:1
- 연산자는 answer_eq / len_answer_eq 만
- 명령은 extract-data · export-text · chart-to-csv 만
- 새 CLI 이름 부재
- fill-fields / csv-to-table / batch 문자열 부재
- 표본은 기존 samples/ 경로만
- pack README 와 working 문서가 있다
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK = REPO_ROOT / "gym" / "packs" / "extraction"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_coverage_and_extract.md"
PACK_JSON = PACK / "pack.json"

ALLOWED_CMDS = {"extract-data", "export-text", "chart-to-csv"}
ALLOWED_OPS = {"answer_eq", "len_answer_eq"}
FORBIDDEN_TOKENS = ("fill-fields", "csv-to-table", "batch fill", "deep_contains")
ALLOWED_SAMPLE_PREFIXES = (
    "samples/chart/",
    "samples/20250130-hongbo.hwp",
    "samples/exam-kor-",
    "samples/hwpx/exam-kor-",
    "samples/hwpx/blank_hwpx.hwpx",
    "samples/table-001.hwp",
    "samples/2010-01-06.hwp",
)


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("EX*.json"))


def load_tasks():
    return [read_json(p) for p in task_paths()]


def task_id_num(tid: str) -> int:
    if not tid.startswith("EX") or not tid[2:].isdigit():
        raise AssertionError(f"과제 ID 가 EXnn 이 아니다: {tid}")
    return int(tid[2:])


class PackSurfaceTests(unittest.TestCase):
    def test_pack_manifest_keeps_extraction_identity(self):
        manifest = read_json(PACK_JSON)
        self.assertEqual(manifest["id"], "extraction")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertTrue(manifest["axis"].startswith("조회"))
        for cmd in ("chart-to-csv", "export-text", "extract-data"):
            self.assertIn(cmd, manifest["requires"]["commands"])
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)
        self.assertTrue(runner["rhwpVersion"])

    def test_runner_identity_is_not_silently_rewritten(self):
        runner = read_json(PACK_JSON)["runner"]
        self.assertEqual(runner["rhwpVersion"], "0.8.4")
        self.assertEqual(
            runner["rhwpCommit"], "4324eb0e4cf1a65f7efb305993a79ac44859a7ca")
        self.assertEqual(
            runner["capabilitiesSha256"],
            "4767e61c3af751bb2f97af9d0b3e5ffa5cbb5dc70a89cf3ae85987132fa5473d")

    def test_pack_readme_and_working_doc_exist(self):
        self.assertTrue(README.is_file(), "pack README 가 없다")
        self.assertTrue(WORKING.is_file(), "working 문서가 없다")
        readme = README.read_text(encoding="utf-8")
        working = WORKING.read_text(encoding="utf-8")
        self.assertIn("extract-data", readme)
        self.assertIn("EX19", readme)
        self.assertIn("itemCount", readme)
        self.assertIn("extraction", working)
        self.assertGreater(len(readme.splitlines()), 200)
        self.assertGreater(len(working.splitlines()), 200)


class TaskInventoryTests(unittest.TestCase):
    def test_existing_ex01_ex04_remain(self):
        ids = {read_json(p)["id"] for p in task_paths()}
        for n in range(1, 5):
            self.assertIn(f"EX{n:02d}", ids, f"기존 EX{n:02d} 가 사라졌다")

    def test_ex05_and_later_exist(self):
        nums = [task_id_num(read_json(p)["id"]) for p in task_paths()]
        self.assertGreaterEqual(max(nums), 20)
        self.assertGreaterEqual(len(nums), 20)

    def test_task_ids_are_ex_prefixed_and_unique(self):
        ids = [read_json(p)["id"] for p in task_paths()]
        self.assertEqual(len(ids), len(set(ids)))
        for tid in ids:
            self.assertTrue(tid.startswith("EX"), tid)
            task_id_num(tid)

    def test_every_task_has_matching_reference(self):
        for path in task_paths():
            tid = read_json(path)["id"]
            ref_path = REFS / f"{tid}.json"
            self.assertTrue(ref_path.is_file(), f"기준풀이 없음: {tid}")
            ref = read_json(ref_path)
            self.assertEqual(ref["id"], tid)
            self.assertTrue(ref.get("steps"), f"{tid} 기준풀이 steps 가 비었다")

    def test_no_orphan_reference(self):
        task_names = {p.name for p in TASKS.glob("*.json")}
        for path in REFS.glob("*.json"):
            self.assertIn(path.name, task_names, f"고아 기준풀이: {path.name}")


class OperatorAndCommandTests(unittest.TestCase):
    def test_only_allowed_ops(self):
        for task in load_tasks():
            for check in task["checks"]:
                self.assertIn(check["op"], ALLOWED_OPS, task["id"])
                self.assertTrue(check.get("name"), task["id"])

    def test_only_allowed_commands(self):
        for task in load_tasks():
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                self.assertTrue(cmd, task["id"])
                self.assertIn(cmd[0], ALLOWED_CMDS, f"{task['id']} 새 CLI? {cmd[0]}")

    def test_references_use_same_whitelist(self):
        for path in REFS.glob("*.json"):
            ref = read_json(path)
            for step in ref["steps"]:
                if "run" in step:
                    self.assertIn(step["run"][0], ALLOWED_CMDS, path.name)
                answer = step.get("answer") or {}
                for spec in answer.values():
                    if isinstance(spec, dict) and spec.get("cmd"):
                        self.assertIn(spec["cmd"][0], ALLOWED_CMDS, path.name)

    def test_no_forbidden_tokens(self):
        for path in list(TASKS.glob("*.json")) + list(REFS.glob("*.json")):
            raw = path.read_text(encoding="utf-8")
            for token in FORBIDDEN_TOKENS:
                self.assertNotIn(token, raw, f"{path.name} 에 {token}")

    def test_samples_stay_on_known_prefixes(self):
        for task in load_tasks():
            inp = task["input"]
            self.assertTrue(
                any(inp.startswith(p) for p in ALLOWED_SAMPLE_PREFIXES),
                f"{task['id']} 미허용 표본: {inp}",
            )

    def test_all_tasks_are_answer_kind(self):
        for task in load_tasks():
            self.assertEqual(task["submit"]["kind"], "answer", task["id"])

    def test_kind_flags_are_known(self):
        known = {"date", "amount", "number", "all"}
        for task in load_tasks():
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] == "extract-data" and "--kind" in cmd:
                    kind = cmd[cmd.index("--kind") + 1]
                    self.assertIn(kind, known, task["id"])

    def test_chart_index_is_one_based(self):
        for task in load_tasks():
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] == "chart-to-csv" and "--chart" in cmd:
                    n = cmd[cmd.index("--chart") + 1]
                    self.assertNotEqual(n, "0", f"{task['id']} 차트 0 기준")
                    self.assertTrue(n.isdigit() and int(n) >= 1, task["id"])


class SchemaSmokeTests(unittest.TestCase):
    def test_tasks_pass_schema_validate_task(self):
        import sys

        gym_root = str(REPO_ROOT / "gym")
        if gym_root not in sys.path:
            sys.path.insert(0, gym_root)
        from core import schema  # noqa: WPS433

        manifest = read_json(PACK_JSON)
        errors = []
        for task in load_tasks():
            schema.validate_task(task, manifest, None, errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_tiers_are_in_range(self):
        for task in load_tasks():
            self.assertIsInstance(task["tier"], int)
            self.assertGreaterEqual(task["tier"], 1)
            self.assertLessEqual(task["tier"], 5)


if __name__ == "__main__":
    unittest.main()
