"""serialization pack 계약 — 여정·필드 지목·exit·표본 실재.

[#5223 / PR #5232] 저장·변환 pack 이 기존 CLI 와 samples 만으로 왕복을
가르는지, 골든 바이트를 박제하지 않는지, extract-pages 가 1 기준인지,
ir-diff/--verify 가 차이(exit 3/4)를 실패로 위장하지 않는지를 CI 가
항상 확인한다. 바이너리 없이 순수 파일 검사다.
"""

from __future__ import annotations

import json
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACK = GYM / "packs" / "serialization"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_serialization_pack.md"
EXCEPTIONS = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_serialization_exceptions.md"

EXPECTED_IDS = [f"SR{i:02d}" for i in range(1, 57)]

NEW_REQUIRES = {
    "convert",
    "export-doclang",
    "export-hwpx",
    "export-markdown",
    "export-pdf",
    "extract-pages",
    "info",
    "ir-diff",
}

LIVE_ANSWER_PATHS = {
    "pageCount",
    "renderedCount",
    "pagesAfter",
    "pagesBefore",
    "paragraphsKept",
    "paragraphsRemoved",
    "backend",
    "wasDistribution",
    "lossCount",
    "diffCount",
    "doclangVersion",
    "identical",
    "verify.identical",
    "verifyPages.identical",
}

CONVERT_FAMILY = {"SR09", "SR13", "SR17", "SR21", "SR25", "SR26", "SR27", "SR56"}
EXTRACT_FAMILY = {"SR10", "SR14", "SR18", "SR22", "SR28", "SR29", "SR30", "SR31", "SR54"}
PDF_FAMILY = {
    "SR11", "SR15", "SR19", "SR23", "SR32", "SR33", "SR34", "SR35", "SR49", "SR50",
}
IR_FAMILY = {
    "SR01", "SR06", "SR12", "SR16", "SR20", "SR24", "SR36", "SR37", "SR38", "SR39", "SR53",
}
HWPX_FAMILY = {"SR01", "SR06", "SR40", "SR41", "SR42", "SR51"}
DOCLANG_FAMILY = {"SR03", "SR43", "SR44", "SR45", "SR52"}


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def load_core():
    sys.path.insert(0, str(REPO_ROOT))
    from gym.core import schema  # noqa: WPS433

    return schema


def all_tasks():
    return [load_json(p) for p in sorted(TASKS.glob("SR*.json"))]


def all_refs():
    return [load_json(p) for p in sorted(REFS.glob("SR*.json"))]


def walk_cmds(obj):
    """task checks / reference steps 안의 cmd·run 리스트를 모은다."""
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
    def test_pack_json_keeps_identity_and_declares_new_commands(self):
        manifest = load_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "serialization")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("변환", manifest["axis"])
        self.assertEqual(set(manifest["requires"]["commands"]), NEW_REQUIRES)
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
        task_ids = [p.stem for p in sorted(TASKS.glob("SR*.json"))]
        ref_ids = [p.stem for p in sorted(REFS.glob("SR*.json"))]
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
            # SR01/SR06 는 devel 부터 value_eq(format) 로 형식만 본다.
            if task["id"] in {"SR01", "SR06"}:
                continue
            files = task.get("submit", {}).get("files") or []
            produced = [f for f in files if f != "answer.json"]
            if not produced:
                continue
            ops = {c["op"] for c in task["checks"]}
            if "file_exists" not in ops:
                missing.append(f"{task['id']} file_exists 없음")
            if "differs_from_input" not in ops:
                missing.append(f"{task['id']} differs_from_input 없음")
        self.assertEqual(missing, [], missing)

    def test_no_golden_hashes_or_frozen_page_counts(self):
        """라이브 오라클 — 흔들리는 값을 JSON 에 숫자로 박제하지 않는다."""
        frozen = []
        for task in all_tasks():
            raw = json.dumps(task, ensure_ascii=False)
            if re.search(r'"sha256"|[0-9a-f]{64}', raw, re.I):
                if task["id"] != "SR05":
                    frozen.append(f"{task['id']} 해시 박제 의심")
            for check in task["checks"]:
                if check["op"] == "answer_eq" and check.get("path") in LIVE_ANSWER_PATHS:
                    if "value" in check:
                        frozen.append(f"{task['id']} {check['path']} 값 박제")
                if check["op"] == "value_eq" and check.get("path") in {
                    "pageCount", "renderedCount", "pagesAfter", "pagesBefore",
                    "paragraphsKept", "paragraphsRemoved", "lossCount", "diffCount",
                }:
                    frozen.append(f"{task['id']} {check['path']} 를 value_eq 로 박제")
        self.assertEqual(frozen, [], frozen)

    def test_live_answer_eq_does_not_embed_expected_literals(self):
        for task in all_tasks():
            for check in task["checks"]:
                if check.get("op") != "answer_eq":
                    continue
                self.assertIn("cmd", check, task["id"])
                self.assertIn("path", check, task["id"])
                self.assertNotIn("value", check, task["id"])


class JourneyFamilyTests(unittest.TestCase):
    def test_convert_family_writes_hwp_and_reads_envelope(self):
        for tid in sorted(CONVERT_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            cmds = walk_cmds(task)
            self.assertTrue(any(c and c[0] == "convert" for c in cmds), tid)
            produced = (task.get("submit", {}).get("files") or [])
            self.assertIn("conv.hwp", produced, tid)

    def test_extract_family_is_one_based(self):
        for tid in sorted(EXTRACT_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            found = False
            for cmd in walk_cmds(task):
                if not cmd or cmd[0] != "extract-pages":
                    continue
                found = True
                self.assertIn("--from", cmd, tid)
                self.assertIn("--to", cmd, tid)
                frm = int(cmd[cmd.index("--from") + 1])
                to = int(cmd[cmd.index("--to") + 1])
                self.assertGreaterEqual(frm, 1, f"{tid} --from 는 1 기준")
                self.assertGreaterEqual(to, frm, tid)
            self.assertTrue(found, tid)
            hint = task["instructions"] + " " + " ".join(
                tok for cmd in walk_cmds(task) for tok in cmd)
            self.assertIn("--from", hint, tid)

    def test_pdf_family_checks_format_or_live_field(self):
        for tid in sorted(PDF_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            cmds = walk_cmds(task)
            self.assertTrue(any(c and c[0] == "export-pdf" for c in cmds), tid)
            paths = {c.get("path") for c in task["checks"]}
            self.assertTrue(
                paths & {"format", "backend", "pageCount", "renderedCount"},
                tid,
            )

    def test_ir_family_allows_diff_exit(self):
        for tid in sorted(IR_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            ir_checks = [
                c for c in task["checks"]
                if c.get("cmd") and c["cmd"][0] == "ir-diff"
            ]
            self.assertTrue(ir_checks, tid)
            for check in ir_checks:
                self.assertEqual(check.get("expect_exits"), [0, 3], tid)

    def test_verify_and_verify_pages_exits(self):
        verify = []
        pages = []
        for task in all_tasks():
            for cmd in walk_cmds(task):
                if "--verify-pages" in cmd:
                    pages.append(task["id"])
                elif "--verify" in cmd:
                    verify.append(task["id"])
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if "--verify-pages" in cmd:
                    self.assertEqual(check.get("expect_exits"), [0, 4], task["id"])
                    self.assertEqual(check.get("path"), "verifyPages.identical")
                elif "--verify" in cmd:
                    self.assertEqual(check.get("expect_exits"), [0, 3], task["id"])
                    self.assertEqual(check.get("path"), "verify.identical")
        self.assertGreaterEqual(len(set(verify)), 2)
        self.assertGreaterEqual(len(set(pages)), 2)

    def test_hwpx_family_does_not_use_convert_for_hwpx_output(self):
        for tid in sorted(HWPX_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            ref = load_json(REFS / f"{tid}.json")
            files = task.get("submit", {}).get("files") or []
            self.assertIn("conv.hwpx", files, tid)
            heads = {cmd[0] for cmd in walk_cmds(task) + walk_cmds(ref) if cmd}
            self.assertIn("export-hwpx", heads, tid)
            self.assertNotIn("convert", heads, tid)

    def test_doclang_family_reads_envelope_not_xml_bytes(self):
        for tid in sorted(DOCLANG_FAMILY):
            task = load_json(TASKS / f"{tid}.json")
            cmds = walk_cmds(task)
            self.assertTrue(any(c and c[0] == "export-doclang" for c in cmds), tid)
            paths = {c.get("path") for c in task["checks"] if c.get("op") == "answer_eq"}
            self.assertTrue(paths & {"lossCount", "format", "doclangVersion"}, tid)


class ReferencePairTests(unittest.TestCase):
    def test_reference_steps_exist(self):
        for ref in all_refs():
            self.assertTrue(ref.get("steps"), ref["id"])

    def test_reference_reuses_the_task_command(self):
        """기준풀이가 과제가 지목한 명령을 다시 돌린다 — 다른 오라클을 몰래 쓰지 않는다."""
        mismatches = []
        for tid in EXPECTED_IDS:
            task = load_json(TASKS / f"{tid}.json")
            ref = load_json(REFS / f"{tid}.json")
            task_heads = {cmd[0] for cmd in walk_cmds(task) if cmd}
            ref_heads = {cmd[0] for cmd in walk_cmds(ref) if cmd}
            if tid in {"SR02", "SR05"}:
                continue
            if tid == "SR48":
                self.assertIn("convert", ref_heads)
                self.assertIn("info", ref_heads)
                continue
            if not (task_heads & ref_heads):
                mismatches.append(f"{tid}: task={sorted(task_heads)} ref={sorted(ref_heads)}")
        self.assertEqual(mismatches, [], mismatches)

    def test_reference_uses_sub_placeholders_not_file(self):
        leaked = []
        for ref in all_refs():
            raw = json.dumps(ref, ensure_ascii=False)
            if "{file:" in raw:
                leaked.append(ref["id"])
        self.assertEqual(leaked, [], leaked)


class DocumentationTests(unittest.TestCase):
    def test_readme_is_korean_journeys_and_failure_modes(self):
        text = README.read_text(encoding="utf-8")
        self.assertIn("여정", text)
        self.assertIn("실패 모드", text)
        for marker in ("J1", "J2", "J3", "J4", "J5", "J6", "F1", "F3", "F4"):
            self.assertIn(marker, text, marker)
        self.assertIn("1 기준", text)
        self.assertIn("라이브 오라클", text)
        self.assertNotRegex(text, r"^[A-Za-z0-9 ,.\-']+$")

    def test_working_notes_and_exceptions_exist(self):
        working = WORKING.read_text(encoding="utf-8")
        notes = EXCEPTIONS.read_text(encoding="utf-8")
        self.assertIn("SR13", working)
        self.assertIn("SR24", working)
        self.assertIn("라이브 오라클", working)
        self.assertIn("E1", notes)
        self.assertIn("E3", notes)
        self.assertIn("재조판", notes)
        self.assertIn("wasDistribution", notes)

    def test_readme_lists_rescued_and_followup_ids(self):
        text = README.read_text(encoding="utf-8")
        for tid in ("SR09", "SR13", "SR24", "SR25", "SR56"):
            self.assertIn(tid, text, tid)


class NoNewCliTests(unittest.TestCase):
    def test_commands_are_existing_surface_only(self):
        allowed = NEW_REQUIRES | {"export-hml", "export-ir-schema"}
        unknown = []
        for task in all_tasks():
            for cmd in walk_cmds(task):
                if cmd and cmd[0] not in allowed:
                    unknown.append(f"{task['id']}:{cmd[0]}")
        self.assertEqual(unknown, [], unknown)

    def test_no_new_pack_directory(self):
        self.assertTrue((PACK / "pack.json").is_file())
        self.assertFalse((GYM / "packs" / "serialization-expand").exists())


if __name__ == "__main__":
    unittest.main()
