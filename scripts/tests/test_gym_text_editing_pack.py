"""text-editing pack 계약 — 여정·좌표 지목·occurrence·insert-text.

[#5233 / PR #5242] 본문 편집 pack 이 기존 CLI 와 samples 만으로
탐색→치환→재검증을 가르치는지, deep_contains 와 T07(fill-fields) 를
끌어오지 않는지, occurrence 가 0 기준인지, insert-text 좌표가
search 주소와 같은지 CI 가 항상 확인한다. 바이너리 없이 순수 파일 검사다.
"""

from __future__ import annotations

import json
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACK = GYM / "packs" / "text-editing"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_text_editing.md"
EXCEPTIONS = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_text_editing_exceptions.md"

EXPECTED_IDS = [f"TE{i:02d}" for i in range(1, 91)]

ALLOWED_COMMANDS = {
    "digest",
    "edit",
    "explain",
    "export-structure",
    "info",
    "search",
}

ALLOWED_OPS = {
    "answer_eq",
    "value_eq",
    "value_ge",
    "differs_from_input",
    "file_exists",
    "len_answer_eq",
    "len_ge",
    "value_in",
}

BANNED_OPS = {"deep_contains", "not_contains"}
BANNED_TOKENS = {"fill-fields", "T07", "set-cell", "insert-text-in-cell"}

OCC_FAMILY = {
    "TE11", "TE15", "TE16", "TE17", "TE18", "TE19", "TE20", "TE21",
    "TE56", "TE57", "TE58",
}
ALL_FAMILY = {
    "TE01", "TE02", "TE10", "TE12", "TE14",
    "TE22", "TE23", "TE24", "TE51", "TE52", "TE53", "TE54", "TE55",
    "TE59", "TE60",
}
INSERT_FAMILY = {
    "TE13",
    "TE27", "TE28", "TE29", "TE30", "TE31", "TE32", "TE33", "TE34",
    "TE35", "TE36", "TE37", "TE38", "TE39", "TE40", "TE41", "TE42",
    "TE43", "TE44", "TE45", "TE46", "TE47", "TE48", "TE50",
}
DRY_FAMILY = {"TE08", "TE25", "TE26", "TE49", "TE83", "TE84", "TE90"}
INVEST_FAMILY = {
    "TE04", "TE05", "TE06", "TE07", "TE09",
    "TE61", "TE62", "TE63", "TE64", "TE65", "TE66", "TE67", "TE68",
    "TE69", "TE70", "TE71", "TE72", "TE73", "TE74", "TE75", "TE76",
    "TE85", "TE86", "TE87", "TE88", "TE89",
}


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def load_core():
    sys.path.insert(0, str(REPO_ROOT))
    from gym.core import schema  # noqa: WPS433

    return schema


def all_tasks():
    return [load_json(p) for p in sorted(TASKS.glob("TE*.json"))]


def all_refs():
    return [load_json(p) for p in sorted(REFS.glob("TE*.json"))]


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


class PackManifestContractTests(unittest.TestCase):
    def test_pack_json_keeps_identity_and_existing_commands(self):
        manifest = load_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "text-editing")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("편집", manifest["axis"])
        self.assertEqual(set(manifest["requires"]["commands"]), ALLOWED_COMMANDS)
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)
        self.assertTrue(runner["rhwpVersion"])

    def test_schema_accepts_the_live_pack(self):
        schema = load_core()
        errors = []
        schema.validate_pack(load_json(PACK / "pack.json"), str(PACK), errors)
        self.assertEqual(errors, [], errors)


class TaskInventoryTests(unittest.TestCase):
    def test_expected_ids_are_complete_and_paired(self):
        task_ids = [p.stem for p in sorted(TASKS.glob("TE*.json"))]
        ref_ids = [p.stem for p in sorted(REFS.glob("TE*.json"))]
        self.assertEqual(task_ids, EXPECTED_IDS)
        self.assertEqual(ref_ids, EXPECTED_IDS)
        for tid in EXPECTED_IDS:
            task = load_json(TASKS / f"{tid}.json")
            ref = load_json(REFS / f"{tid}.json")
            self.assertEqual(task["id"], tid)
            self.assertEqual(ref["id"], tid)

    def test_every_task_passes_schema(self):
        schema = load_core()
        manifest = load_json(PACK / "pack.json")
        errors = []
        for task in all_tasks():
            schema.validate_task(task, manifest, None, errors)
        self.assertEqual(errors, [], errors)

    def test_every_check_is_named(self):
        missing = []
        for task in all_tasks():
            for check in task["checks"]:
                if not check.get("name"):
                    missing.append(task["id"])
        self.assertEqual(missing, [])

    def test_tiers_stay_in_range(self):
        for task in all_tasks():
            self.assertIsInstance(task["tier"], int, task["id"])
            self.assertGreaterEqual(task["tier"], 1, task["id"])
            self.assertLessEqual(task["tier"], 5, task["id"])


class SampleAndOracleTests(unittest.TestCase):
    def test_every_input_exists_in_the_repo(self):
        missing = []
        for task in all_tasks():
            rel = task["input"]
            if not (REPO_ROOT / rel).is_file():
                missing.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"표본 없음: {missing}")

    def test_secondary_sample_paths_in_cmds_exist(self):
        missing = []
        for task in all_tasks():
            for cmd in walk_cmds(task):
                for tok in cmd:
                    if isinstance(tok, str) and tok.startswith("samples/"):
                        if not (REPO_ROOT / tok).is_file():
                            missing.append(f"{task['id']}:{tok}")
        self.assertEqual(missing, [], f"보조 표본 없음: {missing}")

    def test_artifact_tasks_reject_unedited_copies(self):
        missing = []
        for task in all_tasks():
            files = task.get("submit", {}).get("files") or []
            produced = [f for f in files if f != "answer.json"]
            if not produced:
                continue
            ops = {c["op"] for c in task["checks"]}
            if "differs_from_input" not in ops:
                missing.append(f"{task['id']} differs_from_input 없음")
        self.assertEqual(missing, [], missing)

    def test_no_golden_hashes_or_frozen_match_counts(self):
        frozen = []
        for task in all_tasks():
            raw = json.dumps(task, ensure_ascii=False)
            if re.search(r'"sha256"|[0-9a-f]{64}', raw, re.I):
                frozen.append(f"{task['id']} 해시 박제 의심")
            for check in task["checks"]:
                if check["op"] == "answer_eq" and "value" in check:
                    frozen.append(f"{task['id']} answer_eq 값 박제")
                if (
                    check["op"] == "value_eq"
                    and check.get("path") in {"pageCount", "matchCount", "paragraphCount", "paraCount", "nodeCount"}
                    and check.get("value") not in {0, 1}
                ):
                    frozen.append(f"{task['id']} {check['path']}={check.get('value')} 박제")
        self.assertEqual(frozen, [], frozen)

    def test_live_answer_eq_does_not_embed_expected_literals(self):
        for task in all_tasks():
            for check in task["checks"]:
                if check.get("op") != "answer_eq":
                    continue
                self.assertIn("cmd", check, task["id"])
                self.assertIn("path", check, task["id"])
                self.assertNotIn("value", check, task["id"])


class PinpointAndBanTests(unittest.TestCase):
    def test_no_deep_contains_or_global_scan(self):
        hits = []
        for task in all_tasks():
            for check in task["checks"]:
                if check.get("op") in BANNED_OPS:
                    hits.append(f"{task['id']}:{check['op']}")
        self.assertEqual(hits, [], hits)

    def test_ops_are_pinpoint_only(self):
        unknown = []
        for task in all_tasks():
            for check in task["checks"]:
                if check.get("op") not in ALLOWED_OPS:
                    unknown.append(f"{task['id']}:{check.get('op')}")
        self.assertEqual(unknown, [], unknown)

    def test_no_t07_fill_fields_clone(self):
        hits = []
        for task in all_tasks():
            raw = json.dumps(task, ensure_ascii=False)
            for tok in BANNED_TOKENS:
                if tok in raw and not (tok == "T07" and "TE07" in raw):
                    # 힌트에서 T07 을 금지한다고 언급하는 것은 허용.
                    if "fill-fields" in raw and "T07" in raw and "아니다" in raw:
                        continue
                    if tok == "T07":
                        continue
                    hits.append(f"{task['id']}:{tok}")
            for cmd in walk_cmds(task):
                if "fill-fields" in cmd or "set-cell" in cmd:
                    hits.append(f"{task['id']}:cmd {cmd}")
        for ref in all_refs():
            for cmd in walk_cmds(ref):
                if "fill-fields" in cmd or "set-cell" in cmd:
                    hits.append(f"{ref['id']}:ref {cmd}")
        self.assertEqual(hits, [], hits)

    def test_commands_are_existing_surface_only(self):
        unknown = []
        for task in all_tasks():
            for cmd in walk_cmds(task):
                if cmd and cmd[0] not in ALLOWED_COMMANDS:
                    unknown.append(f"{task['id']}:{cmd[0]}")
        for ref in all_refs():
            for cmd in walk_cmds(ref):
                if cmd and cmd[0] not in ALLOWED_COMMANDS:
                    unknown.append(f"{ref['id']}:ref {cmd[0]}")
        self.assertEqual(unknown, [], unknown)


class JourneyFamilyTests(unittest.TestCase):
    def test_occurrence_family_is_zero_based_and_keeps_residue(self):
        for tid in sorted(OCC_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            ref = load_json(REFS / f"{tid}.json")
            found = False
            for cmd in walk_cmds(ref):
                if "--occurrence" not in cmd:
                    continue
                found = True
                occ = int(cmd[cmd.index("--occurrence") + 1])
                self.assertGreaterEqual(occ, 0, tid)
                self.assertLess(occ, 8, tid)
            self.assertTrue(found, tid)
            residue = [
                c for c in task["checks"]
                if c.get("op") == "value_ge" and c.get("value") == 1
            ]
            self.assertGreaterEqual(len(residue), 2, f"{tid} 새 문구+잔여 검사")
            self.assertIn("differs_from_input", {c["op"] for c in task["checks"]}, tid)
            self.assertNotIn("--occurrence", " ".join(
                tok for cmd in walk_cmds(task) for tok in cmd
            ), f"{tid} 과제 검사에 occurrence 를 박지 않는다 — 산출을 재검색한다")

    def test_all_family_requires_zero_old_phrase(self):
        for tid in sorted(ALL_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            ref = load_json(REFS / f"{tid}.json")
            zero = [
                c for c in task["checks"]
                if c.get("op") == "value_eq" and c.get("path") == "matchCount" and c.get("value") == 0
            ]
            # TE02 는 쪽수만 본다 — 예외.
            if tid == "TE02":
                self.assertIn("answer_eq", {c["op"] for c in task["checks"]}, tid)
                continue
            self.assertTrue(zero, tid)
            occ_in_ref = any("--occurrence" in cmd for cmd in walk_cmds(ref))
            self.assertFalse(occ_in_ref, f"{tid} 전건인데 occurrence 가 있다")

    def test_insert_family_uses_zero_based_coords(self):
        for tid in sorted(INSERT_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            ref = load_json(REFS / f"{tid}.json")
            found = False
            for cmd in walk_cmds(ref):
                if cmd[:2] != ["edit", "insert-text"]:
                    continue
                found = True
                self.assertIn("--section", cmd, tid)
                self.assertIn("--para", cmd, tid)
                self.assertIn("--offset", cmd, tid)
                sec = int(cmd[cmd.index("--section") + 1])
                para = int(cmd[cmd.index("--para") + 1])
                off = int(cmd[cmd.index("--offset") + 1])
                self.assertGreaterEqual(sec, 0, tid)
                self.assertGreaterEqual(para, 0, tid)
                self.assertGreaterEqual(off, 0, tid)
            self.assertTrue(found, tid)
            ones = [
                c for c in task["checks"]
                if c.get("op") == "value_eq" and c.get("path") == "matchCount" and c.get("value") == 1
            ]
            self.assertTrue(ones, f"{tid} 표지 1건 검사")

    def test_dry_run_family_does_not_submit_files(self):
        for tid in sorted(DRY_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            files = task.get("submit", {}).get("files") or []
            produced = [f for f in files if f != "answer.json"]
            self.assertEqual(produced, [], tid)
            cmds = walk_cmds(task) + walk_cmds(load_json(REFS / f"{tid}.json"))
            self.assertTrue(any("--dry-run" in cmd for cmd in cmds), tid)

    def test_investigation_family_is_answer_only(self):
        for tid in sorted(INVEST_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            self.assertEqual(task.get("axis"), "조사", tid)
            files = task.get("submit", {}).get("files") or []
            self.assertTrue(not files or files == ["answer.json"], tid)
            ops = {c["op"] for c in task["checks"]}
            self.assertTrue(ops & {"answer_eq", "len_answer_eq"}, tid)


class ReferencePairTests(unittest.TestCase):
    def test_reference_steps_exist(self):
        for ref in all_refs():
            self.assertTrue(ref.get("steps"), ref["id"])

    def test_reference_uses_sub_placeholders_not_file(self):
        leaked = []
        for ref in all_refs():
            raw = json.dumps(ref, ensure_ascii=False)
            if "{file:" in raw:
                leaked.append(ref["id"])
        self.assertEqual(leaked, [], leaked)

    def test_reference_reuses_the_task_command_family(self):
        """조사 과제는 같은 조회 명령을 다시 돌린다.

        편집 산출 과제는 기준풀이가 `edit` 을 돌리고 채점은 `search`/`info`/
        `explain` 으로 재검증한다. 그 짝이 이 pack 의 왕복이다.
        """
        mismatches = []
        for tid in EXPECTED_IDS:
            task = load_json(TASKS / f"{tid}.json")
            ref = load_json(REFS / f"{tid}.json")
            task_heads = {cmd[0] for cmd in walk_cmds(task) if cmd}
            ref_heads = {cmd[0] for cmd in walk_cmds(ref) if cmd}
            if task_heads & ref_heads:
                continue
            if "edit" in ref_heads and task_heads <= {"search", "info", "explain", "digest", "export-structure"}:
                continue
            mismatches.append(f"{tid}: task={sorted(task_heads)} ref={sorted(ref_heads)}")
        self.assertEqual(mismatches, [], mismatches)


class DocumentationTests(unittest.TestCase):
    def test_readme_is_korean_journeys_and_failure_modes(self):
        text = README.read_text(encoding="utf-8")
        self.assertIn("여정", text)
        self.assertIn("실패 모드", text)
        for marker in ("J1", "J2", "J3", "J4", "J5", "J6", "J7", "F1", "F3", "F6"):
            self.assertIn(marker, text, marker)
        self.assertIn("occurrence", text)
        self.assertIn("insert-text", text)
        self.assertIn("0 기준", text)
        self.assertIn("라이브 오라클", text)
        self.assertIn("deep_contains", text)
        self.assertIn("T07", text)
        self.assertNotRegex(text, r"^[A-Za-z0-9 ,.\-']+$")

    def test_working_notes_and_exceptions_exist(self):
        working = WORKING.read_text(encoding="utf-8")
        notes = EXCEPTIONS.read_text(encoding="utf-8")
        self.assertIn("TE15", working)
        self.assertIn("TE90", working)
        self.assertIn("라이브 오라클", working)
        self.assertIn("E1", notes)
        self.assertIn("E3", notes)
        self.assertIn("occurrence", notes)
        self.assertIn("insert-text", notes)
        self.assertIn("replacedCount", notes)

    def test_readme_lists_rescued_and_followup_ids(self):
        text = README.read_text(encoding="utf-8")
        for tid in ("TE11", "TE13", "TE15", "TE27", "TE61", "TE90"):
            self.assertIn(tid, text, tid)


class NoNewCliTests(unittest.TestCase):
    def test_no_new_pack_directory(self):
        self.assertTrue((PACK / "pack.json").is_file())
        self.assertFalse((GYM / "packs" / "text-editing-expand").exists())

    def test_runner_identity_unchanged_from_devel_shape(self):
        manifest = load_json(PACK / "pack.json")
        self.assertEqual(
            manifest["runner"]["rhwpCommit"],
            "1e8667aa86aeb979119aa9152112b42e4f16a76c",
        )


if __name__ == "__main__":
    unittest.main()
