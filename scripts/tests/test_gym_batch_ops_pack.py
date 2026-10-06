"""batch-ops pack 계약 — 메일머지 축·자산 실재·fill 전용.

이 가드는 #5212 의 batch-ops 확장 불변식을 CI 가 다시 본다.
전 pack 정합은 test_gym_packs 가 보고, 여기는 batch-ops 만 본다.

고정하는 것:
- 과제 id BO01+ 전부 기준 풀이 1:1
- 명령은 batch · search 만. batch 의 둘째 토큰은 fill
- dry-run 은 answer + json_value_eq, 산출 없음
- 산출 과제는 file_exists + differs_from_input + search
- --data 자산이 실재
- fill-fields / 새 CLI 부재
- pack README 와 working 문서가 있다
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK = REPO_ROOT / "gym" / "packs" / "batch-ops"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
ASSETS = PACK / "assets"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_coverage_and_extract.md"
PACK_JSON = PACK / "pack.json"

ALLOWED_CMDS = {"batch", "search"}
ALLOWED_OPS = {
    "file_exists",
    "differs_from_input",
    "value_ge",
    "json_value_eq",
}
FORBIDDEN_TOKENS = ("fill-fields", "deep_contains", "csv-to-table")
ALLOWED_SAMPLES = {
    "samples/hwpx/form-01.hwpx",
    "samples/form-01.hwp",
    "samples/hwpx/form-02.hwpx",
    "samples/form-02.hwp",
}
DRY_RUN_IDS = {"BO04", "BO11", "BO19"}


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("BO*.json"))


def load_tasks():
    return [read_json(p) for p in task_paths()]


def task_id_num(tid: str) -> int:
    if not tid.startswith("BO") or not tid[2:].isdigit():
        raise AssertionError(f"과제 ID 가 BOnn 이 아니다: {tid}")
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
    def test_pack_manifest_keeps_batch_ops_identity(self):
        manifest = read_json(PACK_JSON)
        self.assertEqual(manifest["id"], "batch-ops")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertTrue(manifest["axis"].startswith("자동화"))
        for cmd in ("batch", "search"):
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
        self.assertIn("batch fill", readme)
        self.assertIn("--name-field", readme)
        self.assertIn("BO04", readme)
        self.assertIn("fill-fields", readme)
        self.assertGreater(len(readme.splitlines()), 200)


class TaskInventoryTests(unittest.TestCase):
    def test_existing_bo01_bo03_remain(self):
        ids = {read_json(p)["id"] for p in task_paths()}
        for n in range(1, 4):
            self.assertIn(f"BO{n:02d}", ids)

    def test_bo04_and_later_exist(self):
        nums = [task_id_num(read_json(p)["id"]) for p in task_paths()]
        self.assertGreaterEqual(max(nums), 16)
        self.assertGreaterEqual(len(nums), 16)

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
                self.assertTrue(check.get("name"), task["id"])

    def test_only_allowed_commands_and_fill_subcmd(self):
        for task in load_tasks():
            for cmd in iter_cmds(task):
                self.assertIn(cmd[0], ALLOWED_CMDS, f"{task['id']} 새 CLI? {cmd[0]}")
                if cmd[0] == "batch":
                    self.assertGreaterEqual(len(cmd), 2, task["id"])
                    self.assertEqual(cmd[1], "fill", f"{task['id']} batch 축이 fill 이 아니다")
        for path in REFS.glob("*.json"):
            for cmd in iter_cmds(read_json(path)):
                self.assertIn(cmd[0], ALLOWED_CMDS, path.name)
                if cmd[0] == "batch":
                    self.assertEqual(cmd[1], "fill", path.name)

    def test_no_forbidden_tokens(self):
        for path in list(TASKS.glob("*.json")) + list(REFS.glob("*.json")):
            raw = path.read_text(encoding="utf-8")
            for token in FORBIDDEN_TOKENS:
                self.assertNotIn(token, raw, f"{path.name} 에 {token}")

    def test_samples_stay_on_whitelist(self):
        for task in load_tasks():
            self.assertIn(task["input"], ALLOWED_SAMPLES, task["id"])

    def test_data_assets_exist(self):
        mentioned = set()
        for task in load_tasks():
            blob = json.dumps(task, ensure_ascii=False)
            if "gym/packs/batch-ops/assets/" in blob:
                for token in blob.replace("\\", "/").split():
                    token = token.strip('",')
                    if token.startswith("gym/packs/batch-ops/assets/"):
                        mentioned.add(token)
        for path in REFS.glob("*.json"):
            for cmd in iter_cmds(read_json(path)):
                for token in cmd:
                    if token.startswith("gym/packs/batch-ops/assets/"):
                        mentioned.add(token)
        self.assertTrue(mentioned, "자산 경로가 하나도 없다")
        for rel in mentioned:
            self.assertTrue((REPO_ROOT / rel).is_file(), f"자산 없음: {rel}")

    def test_dry_run_tasks_are_answer_only(self):
        for tid in DRY_RUN_IDS:
            task = read_json(TASKS / f"{tid}.json")
            self.assertEqual(task["submit"]["kind"], "answer", tid)
            ops = [c["op"] for c in task["checks"]]
            self.assertEqual(ops, ["json_value_eq"], tid)
            self.assertNotIn("file_exists", ops, tid)
            ref = read_json(REFS / f"{tid}.json")
            answer = ref["steps"][0]["answer"]
            self.assertIn("planned", answer)
            self.assertIn("const", answer["planned"])

    def test_artifact_tasks_search_needles(self):
        for task in load_tasks():
            if task["id"] in DRY_RUN_IDS:
                continue
            self.assertEqual(task["submit"]["kind"], "artifact", task["id"])
            ops = [c["op"] for c in task["checks"]]
            self.assertIn("file_exists", ops, task["id"])
            self.assertIn("differs_from_input", ops, task["id"])
            self.assertIn("value_ge", ops, task["id"])
            searches = [c for c in task["checks"] if c["op"] == "value_ge"]
            self.assertTrue(searches, task["id"])
            for check in searches:
                self.assertEqual(check["cmd"][0], "search", task["id"])
                self.assertIn("--json", check["cmd"])
                self.assertEqual(check["path"], "matchCount")
                self.assertGreaterEqual(check["value"], 1)


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
