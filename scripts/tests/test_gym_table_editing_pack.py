"""table-editing pack 계약 — 좌표 축·금지 목록·TB13+ 지목 연산자.

이 가드는 #5230/#5240 의 pack 내부 불변식을 CI 가 다시 본다.
전 pack 정합은 test_gym_packs · audit.py 가 보고, 여기는 표 좌표 pack 만 본다.

고정하는 것:
- T07.json 부재 (core-cli 누름틀 과제를 이 pack 에 복제하지 않는다)
- fill-fields 문자열 부재
- deep_contains / not_contains 부재
- TB13+ 모든 과제가 cell_text_eq 를 본판정으로 가진다
- 편집 산출물은 differs_from_input 으로 원본 복사를 거부한다
- 과제↔기준풀이 1:1, id 일치
- 표본은 기존 세 경로만
- pack README 와 working 문서가 있다
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK = REPO_ROOT / "gym" / "packs" / "table-editing"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_table_editing.md"
PACK_JSON = PACK / "pack.json"

ALLOWED_SAMPLES = {
    "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
    "samples/table-001.hwp",
    "samples/143E433F503322BD33.hwp",
}

FORBIDDEN_OPS = {"deep_contains", "not_contains"}
FORBIDDEN_TOKENS = ("fill-fields", "T07.json", "deep_contains")
NEW_TASK_MIN = 13  # TB13 이상
NEW_TASK_MAX = 80


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("TB*.json"))


def load_tasks():
    return [read_json(p) for p in task_paths()]


def task_id_num(tid: str) -> int:
    if not tid.startswith("TB") or not tid[2:].isdigit():
        raise AssertionError(f"과제 ID 가 TBnn 이 아니다: {tid}")
    return int(tid[2:])


class PackSurfaceTests(unittest.TestCase):
    def test_pack_manifest_keeps_table_editing_identity(self):
        manifest = read_json(PACK_JSON)
        self.assertEqual(manifest["id"], "table-editing")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertTrue(manifest["axis"].startswith("편집"))
        self.assertIn("edit", manifest["requires"]["commands"])
        self.assertIn("export-tables", manifest["requires"]["commands"])
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)
        self.assertTrue(runner["rhwpVersion"])

    def test_runner_identity_is_not_silently_rewritten(self):
        """과제만 늘리면서 기준 실행 신원을 갈아끼우지 않는다."""
        runner = read_json(PACK_JSON)["runner"]
        self.assertEqual(runner["rhwpVersion"], "0.8.2")
        self.assertEqual(
            runner["rhwpCommit"], "1e8667aa86aeb979119aa9152112b42e4f16a76c")
        self.assertEqual(
            runner["capabilitiesSha256"],
            "2c7c41bc8952b63c4502ec0685b76990e4ece5b178f6dc28a1a495b12880af75")

    def test_pack_readme_and_working_doc_exist(self):
        self.assertTrue(README.is_file(), "pack README 가 없다")
        self.assertTrue(WORKING.is_file(), "working 문서가 없다")
        readme = README.read_text(encoding="utf-8")
        working = WORKING.read_text(encoding="utf-8")
        self.assertIn("cell_text_eq", readme)
        self.assertIn("deep_contains", readme)
        self.assertIn("T07", readme)
        self.assertIn("TB13", readme)
        self.assertIn("cell_text_eq", working)
        self.assertIn("#5230", working)
        self.assertIn("#5240", working)
        self.assertGreater(len(readme.splitlines()), 80)
        self.assertGreater(len(working.splitlines()), 80)

    def test_readme_declares_non_goals(self):
        text = README.read_text(encoding="utf-8")
        self.assertIn("fill-fields", text)
        self.assertIn("새 CLI", text)
        self.assertIn("profiles", text)


class TaskInventoryTests(unittest.TestCase):
    def test_existing_tb01_tb12_remain(self):
        ids = {read_json(p)["id"] for p in task_paths()}
        for n in range(1, 13):
            self.assertIn(f"TB{n:02d}", ids, f"기존 TB{n:02d} 가 사라졌다")

    def test_tb13_and_later_exist(self):
        nums = [task_id_num(read_json(p)["id"]) for p in task_paths()]
        self.assertGreaterEqual(max(nums), NEW_TASK_MIN)
        new_ids = [n for n in nums if n >= NEW_TASK_MIN]
        self.assertGreaterEqual(len(new_ids), 8, "TB13+ 과제가 너무 적다")
        self.assertLessEqual(max(nums), NEW_TASK_MAX)

    def test_no_t07_file_in_this_pack(self):
        self.assertFalse((TASKS / "T07.json").exists())
        self.assertFalse((REFS / "T07.json").exists())
        for path in list(TASKS.glob("*")) + list(REFS.glob("*")):
            self.assertNotEqual(path.name, "T07.json")
            self.assertFalse(path.name.startswith("T07"))

    def test_task_ids_are_tb_prefixed_and_unique(self):
        ids = [read_json(p)["id"] for p in task_paths()]
        self.assertEqual(len(ids), len(set(ids)))
        for tid in ids:
            self.assertTrue(tid.startswith("TB"), tid)
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


class OperatorContractTests(unittest.TestCase):
    def test_no_task_uses_global_scan_ops(self):
        for task in load_tasks():
            for check in task["checks"]:
                self.assertNotIn(
                    check["op"], FORBIDDEN_OPS,
                    f"{task['id']} 가 전역 훑기 {check['op']} 를 쓴다")
                self.assertNotIn("allowGlobalScan", check, task["id"])

    def test_tb13_plus_all_use_cell_text_eq(self):
        found = 0
        for task in load_tasks():
            if task_id_num(task["id"]) < NEW_TASK_MIN:
                continue
            found += 1
            ops = [c["op"] for c in task["checks"]]
            self.assertIn("cell_text_eq", ops, f"{task['id']} 에 cell_text_eq 가 없다")
            pinpoint = [c for c in task["checks"] if c["op"] == "cell_text_eq"]
            self.assertTrue(pinpoint, task["id"])
            for check in pinpoint:
                self.assertEqual(check.get("table"), 0, task["id"])
                self.assertIsInstance(check.get("row"), int)
                self.assertIsInstance(check.get("col"), int)
                self.assertTrue(check.get("value"), f"{task['id']} 기대 문자열이 비었다")
                self.assertEqual(check.get("path"), "tables")
                self.assertEqual(check["cmd"][0], "export-tables")
                self.assertIn("--json", check["cmd"])
            self.assertIn("differs_from_input", ops, f"{task['id']} 복사 거부 없음")

        self.assertGreaterEqual(found, 8)

    def test_tb13_plus_are_artifact_edits(self):
        for task in load_tasks():
            if task_id_num(task["id"]) < NEW_TASK_MIN:
                continue
            self.assertEqual(task["submit"]["kind"], "artifact")
            files = task["submit"]["files"]
            self.assertEqual(len(files), 1)
            self.assertTrue(files[0].endswith(".hwp"), task["id"])
            self.assertIn("set-cell", task["instructions"])

    def test_tb13_plus_references_are_set_cell_chains(self):
        for task in load_tasks():
            if task_id_num(task["id"]) < NEW_TASK_MIN:
                continue
            ref = read_json(REFS / f"{task['id']}.json")
            runs = [s["run"] for s in ref["steps"] if "run" in s]
            self.assertTrue(runs, task["id"])
            for run in runs:
                self.assertEqual(run[:2], ["edit", "set-cell"], task["id"])
                self.assertIn("--table", run)
                self.assertIn("--row", run)
                self.assertIn("--col", run)
                self.assertIn("--text", run)
                self.assertIn("-o", run)
            self.assertNotIn("fill-fields", json.dumps(ref, ensure_ascii=False))

    def test_no_fill_fields_or_t07_token_in_pack_json(self):
        for path in list(TASKS.glob("*.json")) + list(REFS.glob("*.json")):
            raw = path.read_text(encoding="utf-8")
            lower = raw.lower()
            self.assertNotIn("fill-fields", lower, path.name)
            self.assertNotIn("deep_contains", lower, path.name)
            self.assertNotIn("\"t07\"", lower, path.name)

    def test_samples_stay_on_the_known_whitelist(self):
        for task in load_tasks():
            self.assertIn(task["input"], ALLOWED_SAMPLES, task["id"])


class SchemaSmokeTests(unittest.TestCase):
    def test_new_tasks_pass_schema_validate_task(self):
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

    def test_every_check_is_named(self):
        for task in load_tasks():
            for check in task["checks"]:
                self.assertTrue(check.get("name"), task["id"])

    def test_tiers_are_in_range(self):
        for task in load_tasks():
            self.assertIsInstance(task["tier"], int)
            self.assertGreaterEqual(task["tier"], 1)
            self.assertLessEqual(task["tier"], 5)


class ExistingEditTasksStayPinpoint(unittest.TestCase):
    def test_tb03_tb11_tb12_still_use_cell_text_eq(self):
        for tid in ("TB03", "TB11", "TB12"):
            task = read_json(TASKS / f"{tid}.json")
            ops = [c["op"] for c in task["checks"]]
            self.assertIn("cell_text_eq", ops, tid)
            self.assertNotIn("deep_contains", ops, tid)


if __name__ == "__main__":
    unittest.main()
