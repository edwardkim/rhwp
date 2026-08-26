"""core-cli T15+ · casual-rides CR05+ pack 계약.

[#5258] 입문존과 코어 CLI 온램프를 기존 연산자·기존 표본·기존 명령만으로
두껍게 했는지, T07(fields[0]==홍길동) 을 복제하지 않았는지, runner 신원을
복사만 했는지, 새 pack/CLI 가 없는지 CI 가 파일만으로 확인한다.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
CORE = GYM / "packs" / "core-cli"
CASUAL = GYM / "packs" / "casual-rides"
WORKING = REPO_ROOT / "mydocs" / "working" / "gym_core_casual.md"
MAINTAINER = GYM / "profiles" / "maintainer.json"

EXISTING_OPS = {
    "same_hash",
    "differs_from_input",
    "file_exists",
    "files_differ",
    "xml_root_eq",
    "json_value_eq",
    "csv_cell_eq",
    "utf8_bom",
    "answer_eq",
    "len_answer_eq",
    "len_ge",
    "value_eq",
    "value_ge",
    "value_in",
    "deep_contains",
    "not_contains",
    "cell_text_eq",
}

CORE_COMMANDS = {
    "export-tables",
    "extract-data",
    "fields",
    "gate",
    "harness",
    "harness-status",
    "info",
    "inspect",
    "ir-diff",
    "replay",
    "search",
}
CASUAL_COMMANDS = {"info", "explain", "export-tables", "search"}
ANSWER_OPS = frozenset({"answer_eq", "len_answer_eq"})
BANNED_OPS = {"deep_contains", "not_contains"}
CORE_REF_CMDS = CORE_COMMANDS | {"run", "export-hwpx"}
MIN_CORE = 54
MIN_CR = 44

CORE_RUNNER = {
    "rhwpVersion": "0.8.2",
    "rhwpCommit": "94e4790e5a6bc766b75c3c9695b290f87e3793d4",
    "capabilitiesSha256": "2c7c41bc8952b63c4502ec0685b76990e4ece5b178f6dc28a1a495b12880af75",
}
CASUAL_RUNNER = {
    "rhwpVersion": "0.8.4",
    "rhwpCommit": "34ba36fa3c48f37867fbedd16614fdb7e8f44709",
    "capabilitiesSha256": "6e2b231fad617b618fda19663752fc1a57d56086ade31abade84d8e8e000de4c",
}


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def walk_cmds(obj):
    found = []
    if isinstance(obj, dict):
        if isinstance(obj.get("cmd"), list):
            found.append(obj["cmd"])
        if isinstance(obj.get("run"), list):
            found.append(obj["run"])
        for value in obj.values():
            found.extend(walk_cmds(value))
    elif isinstance(obj, list):
        for item in obj:
            found.extend(walk_cmds(item))
    return found


def t_num(path: Path) -> int:
    return int(path.stem[1:])


def cr_num(path: Path) -> int:
    return int(path.stem[2:])


def core_task_paths():
    return sorted((CORE / "tasks").glob("T*.json"), key=t_num)


def cr_task_paths():
    return sorted((CASUAL / "tasks").glob("CR*.json"), key=cr_num)


class ManifestTests(unittest.TestCase):
    def test_core_pack_keeps_identity_and_runner(self):
        manifest = read_json(CORE / "pack.json")
        self.assertEqual(manifest["id"], "core-cli")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("조사", manifest["axis"])
        self.assertEqual(set(manifest["requires"]["commands"]), CORE_COMMANDS)
        self.assertEqual(manifest["runner"], CORE_RUNNER)

    def test_casual_pack_keeps_identity_and_runner(self):
        manifest = read_json(CASUAL / "pack.json")
        self.assertEqual(manifest["id"], "casual-rides")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("입문", manifest["axis"])
        self.assertEqual(set(manifest["requires"]["commands"]), CASUAL_COMMANDS)
        self.assertEqual(manifest["runner"], CASUAL_RUNNER)

    def test_maintainer_still_lists_both_packs(self):
        maintainer = read_json(MAINTAINER)
        self.assertIn("core-cli", maintainer["packs"])
        self.assertIn("casual-rides", maintainer["packs"])

    def test_no_new_pack_directory(self):
        self.assertFalse((GYM / "packs" / "core-casual-expand").exists())
        self.assertFalse((GYM / "packs" / "core-cli-expand").exists())
        self.assertFalse((GYM / "packs" / "casual-rides-expand").exists())


class InventoryTests(unittest.TestCase):
    def test_core_ids_are_contiguous_and_paired(self):
        paths = core_task_paths()
        nums = [t_num(p) for p in paths]
        self.assertEqual(nums[0], 1)
        self.assertGreaterEqual(nums[-1], MIN_CORE)
        self.assertEqual(nums, list(range(1, nums[-1] + 1)))
        missing = [p.stem for p in paths if not (CORE / "reference" / p.name).is_file()]
        self.assertEqual(missing, [])
        for path in paths:
            task = read_json(path)
            ref = read_json(CORE / "reference" / path.name)
            self.assertEqual(task["id"], path.stem)
            self.assertEqual(ref["id"], path.stem)
            self.assertTrue(ref.get("steps"), path.stem)

    def test_casual_ids_are_contiguous_and_paired(self):
        paths = cr_task_paths()
        nums = [cr_num(p) for p in paths]
        self.assertEqual(nums[0], 1)
        self.assertGreaterEqual(nums[-1], MIN_CR)
        self.assertEqual(nums, list(range(1, nums[-1] + 1)))
        missing = [p.stem for p in paths if not (CASUAL / "reference" / p.name).is_file()]
        self.assertEqual(missing, [])
        for path in paths:
            task = read_json(path)
            ref = read_json(CASUAL / "reference" / path.name)
            self.assertEqual(task["id"], path.stem)
            self.assertEqual(ref["id"], path.stem)
            self.assertTrue(ref.get("steps"), path.stem)

    def test_schema_accepts_both_packs(self):
        sys.path.insert(0, str(REPO_ROOT))
        from gym.core import schema as gym_schema  # noqa: WPS433

        errors = []
        for pack in (CORE, CASUAL):
            manifest = read_json(pack / "pack.json")
            gym_schema.validate_pack(manifest, str(pack), errors)
            for path in sorted((pack / "tasks").glob("*.json")):
                gym_schema.validate_task(read_json(path), manifest, None, errors)
        self.assertEqual(errors, [], "\n".join(errors))


class TaskContractTests(unittest.TestCase):
    def test_new_core_instructions_are_unique_korean(self):
        seen_title = {}
        seen_inst = {}
        for path in core_task_paths():
            task = read_json(path)
            tid = task["id"]
            self.assertRegex(task["title"], r"[가-힣]", tid)
            self.assertRegex(task["instructions"], r"[가-힣]", tid)
            if t_num(path) >= 15:
                self.assertGreater(len(task["instructions"]), 200, tid)
            self.assertNotIn(task["title"], seen_title, tid)
            self.assertNotIn(task["instructions"], seen_inst, tid)
            seen_title[task["title"]] = tid
            seen_inst[task["instructions"]] = tid

    def test_new_casual_instructions_are_unique_korean(self):
        seen_title = {}
        seen_inst = {}
        for path in cr_task_paths():
            task = read_json(path)
            tid = task["id"]
            self.assertRegex(task["title"], r"[가-힣]", tid)
            self.assertRegex(task["instructions"], r"[가-힣]", tid)
            if cr_num(path) >= 5:
                self.assertGreater(len(task["instructions"]), 200, tid)
            self.assertNotIn(task["title"], seen_title, tid)
            self.assertNotIn(task["instructions"], seen_inst, tid)
            seen_title[task["title"]] = tid
            seen_inst[task["instructions"]] = tid

    def test_operators_are_existing_only(self):
        unknown = []
        banned = []
        for pack, paths in ((CORE, core_task_paths()), (CASUAL, cr_task_paths())):
            for path in paths:
                task = read_json(path)
                for check in task["checks"]:
                    op = check["op"]
                    if op not in EXISTING_OPS:
                        unknown.append(f"{pack.name}/{task['id']}:{op}")
                    if op in BANNED_OPS:
                        banned.append(f"{pack.name}/{task['id']}:{op}")
                    self.assertTrue(check.get("name"), task["id"])
        self.assertEqual(unknown, [])
        self.assertEqual(banned, [])

    def test_core_check_commands_stay_in_requires(self):
        bad = []
        for path in core_task_paths():
            task = read_json(path)
            for check in task["checks"]:
                cmd = check.get("cmd")
                if cmd and cmd[0] not in CORE_COMMANDS:
                    bad.append(f"{task['id']}:{cmd[0]}")
        self.assertEqual(bad, [])

    def test_casual_check_commands_stay_in_requires(self):
        used = set()
        bad = []
        for path in cr_task_paths():
            task = read_json(path)
            self.assertEqual(task["tier"], 1, task["id"])
            for check in task["checks"]:
                cmd = check.get("cmd")
                if not cmd:
                    continue
                used.add(cmd[0])
                if cmd[0] not in CASUAL_COMMANDS:
                    bad.append(f"{task['id']}:{cmd[0]}")
        self.assertEqual(bad, [])
        self.assertEqual(used, CASUAL_COMMANDS)

    def test_inputs_exist_in_repo(self):
        missing = []
        for pack, paths in ((CORE, core_task_paths()), (CASUAL, cr_task_paths())):
            for path in paths:
                rel = read_json(path)["input"]
                if not (REPO_ROOT / rel).is_file():
                    missing.append(f"{pack.name}/{path.stem}:{rel}")
        self.assertEqual(missing, [])

    def test_no_t07_clone(self):
        hits = []
        for path in list(core_task_paths()) + list(cr_task_paths()):
            if path.stem == "T07":
                continue
            task = read_json(path)
            tid = task["id"]
            if task.get("title") == "서식 채움":
                hits.append(f"{tid}:title")
            files = task.get("submit", {}).get("files") or []
            if "filled.hwp" in files:
                hits.append(f"{tid}:filled.hwp")
            for check in task["checks"]:
                if check.get("value") == "홍길동":
                    hits.append(f"{tid}:value 홍길동")
                if check.get("path") == "fields[0].value" and check.get("value") == "홍길동":
                    hits.append(f"{tid}:fields[0]==홍길동")
        self.assertEqual(hits, [])

    def test_no_hardcoded_golden_counts_in_new_answer_eq(self):
        bad = []
        for path in list(core_task_paths()) + list(cr_task_paths()):
            n = t_num(path) if path.stem.startswith("T") else cr_num(path)
            if path.stem.startswith("T") and n < 15:
                continue
            if path.stem.startswith("CR") and n < 5:
                continue
            task = read_json(path)
            for check in task["checks"]:
                if check["op"] in ANSWER_OPS and "value" in check:
                    bad.append(task["id"])
        self.assertEqual(bad, [])

    def test_answer_reference_mirrors_check_cmd_path(self):
        for pack, paths, min_n, num_fn in (
            (CORE, core_task_paths(), 15, t_num),
            (CASUAL, cr_task_paths(), 5, cr_num),
        ):
            for path in paths:
                if num_fn(path) < min_n:
                    continue
                task = read_json(path)
                if task["submit"]["kind"] != "answer":
                    continue
                answer_checks = [c for c in task["checks"] if c["op"] in ANSWER_OPS]
                if not answer_checks:
                    continue
                ref = read_json(pack / "reference" / path.name)
                answer = None
                for step in ref["steps"]:
                    if "answer" in step:
                        answer = step["answer"]
                        break
                self.assertIsNotNone(answer, task["id"])
                self.assertEqual(set(answer), {c["answer"] for c in answer_checks}, task["id"])
                by_key = {c["answer"]: c for c in answer_checks}
                for key, spec in answer.items():
                    check = by_key[key]
                    self.assertEqual(spec["cmd"], check["cmd"], f"{task['id']}/{key}")
                    self.assertEqual(spec["path"], check["path"], f"{task['id']}/{key}")


class ReferenceTests(unittest.TestCase):
    def test_reference_uses_sub_or_input_not_file_placeholder(self):
        leaked = []
        for pack, prefix, min_n, num_fn in (
            (CORE, "T", 15, t_num),
            (CASUAL, "CR", 5, cr_num),
        ):
            for path in sorted((pack / "reference").glob(f"{prefix}*.json")):
                if num_fn(path) < min_n:
                    continue
                raw = path.read_text(encoding="utf-8")
                if "{file:" in raw:
                    leaked.append(path.stem)
        self.assertEqual(leaked, [])

    def test_core_reference_commands_are_known(self):
        unknown = []
        for path in sorted((CORE / "reference").glob("T*.json")):
            if t_num(path) < 15:
                continue
            for cmd in walk_cmds(read_json(path)):
                if cmd and cmd[0] not in CORE_REF_CMDS:
                    unknown.append(f"{path.stem}:{cmd[0]}")
        self.assertEqual(unknown, [])

    def test_casual_reference_commands_stay_in_requires(self):
        unknown = []
        for path in sorted((CASUAL / "reference").glob("CR*.json")):
            for cmd in walk_cmds(read_json(path)):
                if cmd and cmd[0] not in CASUAL_COMMANDS:
                    unknown.append(f"{path.stem}:{cmd[0]}")
        self.assertEqual(unknown, [])

    def test_artifact_reference_writes_submit_files(self):
        for path in core_task_paths():
            if t_num(path) < 15:
                continue
            task = read_json(path)
            if task["submit"]["kind"] == "answer":
                continue
            blob = (CORE / "reference" / path.name).read_text(encoding="utf-8")
            for fname in task["submit"]["files"]:
                self.assertIn(fname, blob, f"{task['id']}:{fname}")


class DocumentationTests(unittest.TestCase):
    def test_core_readme_lists_new_ids(self):
        text = (CORE / "README.md").read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("T15", "T54", "라이브 오라클", "T07", "홍길동", "기존 연산자", "복제"):
            self.assertIn(needle, text, needle)
        missing = [f"T{n:02d}" for n in range(15, MIN_CORE + 1) if f"T{n:02d}" not in text]
        self.assertEqual(missing, [])

    def test_casual_readme_lists_new_ids(self):
        text = (CASUAL / "README.md").read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("CR05", "CR44", "라이브 오라클", "T07", "홍길동", "입문"):
            self.assertIn(needle, text, needle)
        missing = [f"CR{n:02d}" for n in range(5, MIN_CR + 1) if f"CR{n:02d}" not in text]
        self.assertEqual(missing, [])

    def test_working_notes_cover_both_packs(self):
        text = WORKING.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in (
            "T15", "T54", "CR05", "CR44", "T07", "audit", "test_gym_packs",
            "기존 표본", "runner", "홍길동",
        ):
            self.assertIn(needle, text, needle)
        missing = []
        for n in range(15, MIN_CORE + 1):
            if f"T{n:02d}" not in text:
                missing.append(f"T{n:02d}")
        for n in range(5, MIN_CR + 1):
            if f"CR{n:02d}" not in text:
                missing.append(f"CR{n:02d}")
        self.assertEqual(missing, [])


class AuditTests(unittest.TestCase):
    def test_audit_still_passes_for_both_packs(self):
        import importlib.util

        spec = importlib.util.spec_from_file_location("gym_audit_core_casual", GYM / "tools" / "audit.py")
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], f"audit 실패: {report}")
        bad = [
            p for p in report.get("packs", [])
            if p["id"] in {"core-cli", "casual-rides"} and p.get("issues")
        ]
        self.assertEqual(bad, [], bad)


if __name__ == "__main__":
    unittest.main()
