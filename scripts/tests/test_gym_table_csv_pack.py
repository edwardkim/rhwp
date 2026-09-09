"""table-csv pack 계약 — CSV 왕복 축·연산자 화이트리스트·자산 실재.

이 가드는 #5212 의 table-csv 확장 불변식을 CI 가 다시 본다.
전 pack 정합은 test_gym_packs 가 보고, 여기는 table-csv 만 본다.

고정하는 것:
- 과제 id TC01+ 전부 기준 풀이 1:1
- 명령은 table-to-csv · csv-to-table · export-tables 만
- 전역 훑기 연산자 부재
- fill-fields / 새 CLI 부재
- --csv / 자산 경로가 실재
- 편집 산출물은 differs_from_input
- pack README 와 working 문서가 있다
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK = REPO_ROOT / "gym" / "packs" / "table-csv"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
ASSETS = PACK / "assets"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_coverage_and_extract.md"
PACK_JSON = PACK / "pack.json"

ALLOWED_CMDS = {"table-to-csv", "csv-to-table", "export-tables"}
ALLOWED_OPS = {
    "file_exists",
    "differs_from_input",
    "csv_cell_eq",
    "cell_text_eq",
    "utf8_bom",
    "value_eq",
    "answer_eq",
    "json_value_eq",
}
FORBIDDEN_OPS = {"deep_contains", "not_contains"}
FORBIDDEN_TOKENS = ("fill-fields", "deep_contains", "T07.json")
ALLOWED_SAMPLES = {
    "samples/143E433F503322BD33.hwp",
    "samples/hwpx/143E433F503322BD33.hwpx",
    "samples/hwpx/basic-table-01.hwpx",
    "samples/table-001.hwp",
    "samples/multi-table-001.hwp",
    "samples/table-004.hwp",
    "samples/hwpx/table-text.hwpx",
    "samples/hwp_table_test.hwp",
}


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("TC*.json"))


def load_tasks():
    return [read_json(p) for p in task_paths()]


def task_id_num(tid: str) -> int:
    if not tid.startswith("TC") or not tid[2:].isdigit():
        raise AssertionError(f"과제 ID 가 TCnn 이 아니다: {tid}")
    return int(tid[2:])


def iter_cmds(doc):
    for check in doc.get("checks", []):
        cmd = check.get("cmd") or []
        if cmd:
            yield cmd
    for step in doc.get("steps", []):
        run = step.get("run") or []
        if run:
            yield run
        answer = step.get("answer") or {}
        if isinstance(answer, dict):
            for spec in answer.values():
                if isinstance(spec, dict) and spec.get("cmd"):
                    yield spec["cmd"]


class PackSurfaceTests(unittest.TestCase):
    def test_pack_manifest_keeps_table_csv_identity(self):
        manifest = read_json(PACK_JSON)
        self.assertEqual(manifest["id"], "table-csv")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertTrue(manifest["axis"].startswith("편집"))
        for cmd in ("table-to-csv", "csv-to-table", "export-tables"):
            self.assertIn(cmd, manifest["requires"]["commands"])
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)

    def test_runner_identity_is_not_silently_rewritten(self):
        runner = read_json(PACK_JSON)["runner"]
        self.assertEqual(runner["rhwpVersion"], "0.8.4")
        self.assertEqual(
            runner["rhwpCommit"], "4324eb0e4cf1a65f7efb305993a79ac44859a7ca")
        self.assertEqual(
            runner["capabilitiesSha256"],
            "4767e61c3af751bb2f97af9d0b3e5ffa5cbb5dc70a89cf3ae85987132fa5473d")

    def test_pack_readme_and_working_doc_exist(self):
        self.assertTrue(README.is_file())
        self.assertTrue(WORKING.is_file())
        readme = README.read_text(encoding="utf-8")
        self.assertIn("csv-to-table", readme)
        self.assertIn("--bom", readme)
        self.assertIn("TC19", readme)
        self.assertIn("deep_contains", readme)
        self.assertGreater(len(readme.splitlines()), 200)


class TaskInventoryTests(unittest.TestCase):
    def test_existing_tc01_tc03_remain(self):
        ids = {read_json(p)["id"] for p in task_paths()}
        for n in range(1, 4):
            self.assertIn(f"TC{n:02d}", ids)

    def test_tc04_and_later_exist(self):
        nums = [task_id_num(read_json(p)["id"]) for p in task_paths()]
        self.assertGreaterEqual(max(nums), 20)
        self.assertGreaterEqual(len(nums), 20)

    def test_every_task_has_matching_reference(self):
        for path in task_paths():
            tid = read_json(path)["id"]
            ref_path = REFS / f"{tid}.json"
            self.assertTrue(ref_path.is_file(), f"기준풀이 없음: {tid}")
            self.assertEqual(read_json(ref_path)["id"], tid)
            self.assertTrue(read_json(ref_path).get("steps"))

    def test_no_orphan_reference(self):
        task_names = {p.name for p in TASKS.glob("*.json")}
        for path in REFS.glob("*.json"):
            self.assertIn(path.name, task_names, path.name)


class OperatorAndCommandTests(unittest.TestCase):
    def test_only_allowed_ops(self):
        for task in load_tasks():
            for check in task["checks"]:
                self.assertIn(check["op"], ALLOWED_OPS, f"{task['id']} {check['op']}")
                self.assertNotIn(check["op"], FORBIDDEN_OPS, task["id"])
                self.assertNotIn("allowGlobalScan", check, task["id"])
                self.assertTrue(check.get("name"), task["id"])

    def test_only_allowed_commands(self):
        for task in load_tasks():
            for cmd in iter_cmds(task):
                self.assertIn(cmd[0], ALLOWED_CMDS, f"{task['id']} 새 CLI? {cmd[0]}")
        for path in REFS.glob("*.json"):
            for cmd in iter_cmds(read_json(path)):
                self.assertIn(cmd[0], ALLOWED_CMDS, f"{path.name} 새 CLI? {cmd[0]}")

    def test_no_forbidden_tokens(self):
        for path in list(TASKS.glob("*.json")) + list(REFS.glob("*.json")):
            raw = path.read_text(encoding="utf-8")
            for token in FORBIDDEN_TOKENS:
                self.assertNotIn(token, raw, f"{path.name} 에 {token}")

    def test_samples_stay_on_whitelist(self):
        for task in load_tasks():
            self.assertIn(task["input"], ALLOWED_SAMPLES, task["id"])

    def test_csv_assets_exist(self):
        import re

        asset_re = re.compile(r"gym/packs/table-csv/assets/[A-Za-z0-9_.-]+")
        for task in load_tasks():
            blob = json.dumps(task, ensure_ascii=False)
            for rel in asset_re.findall(blob):
                self.assertTrue(
                    (REPO_ROOT / rel).is_file(),
                    f"{task['id']} 자산 없음: {rel}",
                )
        for path in REFS.glob("*.json"):
            for cmd in iter_cmds(read_json(path)):
                for token in cmd:
                    if "gym/packs/table-csv/assets/" in token:
                        self.assertTrue(
                            (REPO_ROOT / token).is_file(),
                            f"{path.name} 자산 없음: {token}",
                        )

    def test_artifact_edits_reject_passthrough(self):
        for task in load_tasks():
            if task["submit"]["kind"] != "artifact":
                continue
            ops = [c["op"] for c in task["checks"]]
            self.assertIn("file_exists", ops, task["id"])
            self.assertIn("differs_from_input", ops, task["id"])

    def test_bom_tasks_use_utf8_bom(self):
        for task in load_tasks():
            blob = json.dumps(task, ensure_ascii=False)
            if "--bom" not in blob and "BOM" not in task["title"]:
                continue
            if task["id"] in {"TC05", "TC06", "TC23"}:
                ops = [c["op"] for c in task["checks"]]
                self.assertIn("utf8_bom", ops, task["id"])


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
            self.assertGreaterEqual(task["tier"], 1)
            self.assertLessEqual(task["tier"], 5)


if __name__ == "__main__":
    unittest.main()
