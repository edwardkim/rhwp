"""OM · LR · CD pack 계약 — 여정·필드 지목·exit·표본 실재.

[#5219 / PR #5222] objects-media / layout-rendering / corpus-diagnostics
세 pack 이 기존 CLI 와 samples 만으로 자리 지목·조판 판정·코퍼스 진단을
가르는지, 골든 바이트를 박제하지 않는지, 편집에 deep_contains 를 쓰지
않는지, T07 을 복제하지 않는지, extract-pages 가 1 기준인지를 CI 가
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
OM = GYM / "packs" / "objects-media"
LR = GYM / "packs" / "layout-rendering"
CD = GYM / "packs" / "corpus-diagnostics"
WORKING = REPO_ROOT / "mydocs" / "working" / "gym_om_lr_cd.md"

OM_IDS = [f"OM{i:02d}" for i in range(1, 46)]
LR_IDS = [f"LR{i:02d}" for i in range(1, 49)]
CD_IDS = [f"CD{i:02d}" for i in range(1, 49)]

OM_FILL = {"OM02", "OM06", "OM31", "OM32", "OM33", "OM34", "OM35", "OM45"}
LR_VERIFY = {
    "LR02", "LR06", "LR08", "LR10", "LR18", "LR19", "LR20", "LR21",
    "LR22", "LR23", "LR24", "LR25", "LR26", "LR27", "LR37", "LR40",
    "LR45", "LR46", "LR48",
}
LR_SVG = {"LR03", "LR07", "LR33", "LR34", "LR35", "LR42"}
CD_EXTRACT = {
    "CD06", "CD10", "CD26", "CD27", "CD28", "CD29", "CD30", "CD41", "CD46",
}
CD_CONVERT = {"CD07", "CD31", "CD32", "CD33", "CD34", "CD38", "CD42"}
CD_IR = {
    "CD04", "CD09", "CD20", "CD21", "CD22", "CD23", "CD36", "CD40", "CD45", "CD47",
}

ALLOWED_CMDS = {
    "fields",
    "thumbnail",
    "export-markdown",
    "edit",
    "info",
    "verify",
    "render-diff",
    "export-svg",
    "scan",
    "dump-pages",
    "ir-diff",
    "extract-pages",
    "convert",
}

LIVE_ANSWER_PATHS = {
    "fieldCount",
    "fields[0].name",
    "fields[1].name",
    "fields[2].name",
    "fields[0].fieldType",
    "fields[1].fieldType",
    "fields[2].fieldType",
    "width",
    "height",
    "mime",
    "imageCount",
    "pageCount",
    "paraCount",
    "verdict",
    "passCount",
    "status",
    "identical",
    "diffCount",
    "files",
    "files[0].extMismatch",
    "format",
}

RUNNER_PIN = {
    "rhwpVersion": "0.8.2",
    "rhwpCommit": "1e8667aa86aeb979119aa9152112b42e4f16a76c",
    "capabilitiesSha256": "2c7c41bc8952b63c4502ec0685b76990e4ece5b178f6dc28a1a495b12880af75",
}


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def load_core():
    sys.path.insert(0, str(REPO_ROOT))
    from gym.core import schema  # noqa: WPS433

    return schema


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


def pack_tasks(pack: Path, prefix: str):
    return [load_json(p) for p in sorted((pack / "tasks").glob(f"{prefix}*.json"))]


def pack_refs(pack: Path, prefix: str):
    return [load_json(p) for p in sorted((pack / "reference").glob(f"{prefix}*.json"))]


class ManifestContractTests(unittest.TestCase):
    def test_pack_json_keeps_identity(self):
        expected = {
            "objects-media": ("발견", {"export-markdown", "fields", "thumbnail"}),
            "layout-rendering": ("검증", {"info", "render-diff", "thumbnail", "verify"}),
            "corpus-diagnostics": (
                "진단",
                {"dump-pages", "info", "ir-diff", "render-diff", "scan"},
            ),
        }
        for pack_id, (axis_head, commands) in expected.items():
            manifest = load_json(GYM / "packs" / pack_id / "pack.json")
            self.assertEqual(manifest["id"], pack_id)
            self.assertEqual(manifest["kind"], "gymPack")
            self.assertEqual(manifest["schemaVersion"], "1.0")
            self.assertTrue(manifest["axis"].startswith(axis_head), pack_id)
            self.assertEqual(set(manifest["requires"]["commands"]), commands)
            self.assertEqual(manifest["runner"], RUNNER_PIN, pack_id)

    def test_schema_accepts_the_three_packs(self):
        schema = load_core()
        errors = []
        for pack_id in ("objects-media", "layout-rendering", "corpus-diagnostics"):
            pack = GYM / "packs" / pack_id
            schema.validate_pack(load_json(pack / "pack.json"), str(pack), errors)
        self.assertEqual(errors, [], errors)


class TaskInventoryTests(unittest.TestCase):
    def test_expected_ids_are_complete_and_paired(self):
        cases = (
            (OM, "OM", OM_IDS),
            (LR, "LR", LR_IDS),
            (CD, "CD", CD_IDS),
        )
        for pack, prefix, expected in cases:
            task_ids = [p.stem for p in sorted((pack / "tasks").glob(f"{prefix}*.json"))]
            ref_ids = [p.stem for p in sorted((pack / "reference").glob(f"{prefix}*.json"))]
            self.assertEqual(task_ids, expected, prefix)
            self.assertEqual(ref_ids, expected, prefix)
            for tid in expected:
                task = load_json(pack / "tasks" / f"{tid}.json")
                ref = load_json(pack / "reference" / f"{tid}.json")
                self.assertEqual(task["id"], tid)
                self.assertEqual(ref["id"], tid)

    def test_every_task_passes_schema(self):
        schema = load_core()
        errors = []
        for pack_id, prefix in (
            ("objects-media", "OM"),
            ("layout-rendering", "LR"),
            ("corpus-diagnostics", "CD"),
        ):
            pack = GYM / "packs" / pack_id
            manifest = load_json(pack / "pack.json")
            for task in pack_tasks(pack, prefix):
                schema.validate_task(task, manifest, None, errors)
        self.assertEqual(errors, [], errors)

    def test_every_check_is_named_and_tiers_stay_in_range(self):
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for task in pack_tasks(pack, prefix):
                self.assertIsInstance(task["tier"], int, task["id"])
                self.assertGreaterEqual(task["tier"], 1, task["id"])
                self.assertLessEqual(task["tier"], 5, task["id"])
                for check in task["checks"]:
                    self.assertTrue(check.get("name"), task["id"])


class SampleAndOracleTests(unittest.TestCase):
    def test_every_input_exists_in_the_repo(self):
        missing = []
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for task in pack_tasks(pack, prefix):
                rel = task["input"]
                if not (REPO_ROOT / rel).is_file():
                    missing.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"표본 없음: {missing}")

    def test_secondary_sample_paths_in_cmds_exist(self):
        missing = []
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for obj in pack_tasks(pack, prefix) + pack_refs(pack, prefix):
                tid = obj["id"]
                for cmd in walk_cmds(obj):
                    for tok in cmd:
                        if not (isinstance(tok, str) and tok.startswith("samples/")):
                            continue
                        path = REPO_ROOT / tok
                        if not (path.is_file() or path.is_dir()):
                            missing.append(f"{tid}:{tok}")
        self.assertEqual(missing, [], f"보조 표본 없음: {missing}")

    def test_no_golden_hashes_or_frozen_live_fields(self):
        frozen = []
        allowed_pagecount = CD_EXTRACT
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for task in pack_tasks(pack, prefix):
                raw = json.dumps(task, ensure_ascii=False)
                if re.search(r'"sha256"|[0-9a-f]{64}', raw, re.I):
                    frozen.append(f"{task['id']} 해시 박제 의심")
                for check in task["checks"]:
                    if check["op"] == "answer_eq" and check.get("path") in LIVE_ANSWER_PATHS:
                        if "value" in check:
                            frozen.append(f"{task['id']} {check['path']} 값 박제")
                    if check["op"] == "value_eq" and check.get("path") in {
                        "pageCount",
                        "fieldCount",
                        "imageCount",
                        "width",
                        "height",
                        "paraCount",
                        "diffCount",
                    }:
                        if not (task["id"] in allowed_pagecount and check.get("path") == "pageCount"):
                            frozen.append(f"{task['id']} {check['path']} 를 value_eq 로 박제")
        self.assertEqual(frozen, [], frozen)

    def test_live_answer_eq_does_not_embed_expected_literals(self):
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for task in pack_tasks(pack, prefix):
                for check in task["checks"]:
                    if check.get("op") not in {"answer_eq", "len_answer_eq"}:
                        continue
                    self.assertIn("cmd", check, task["id"])
                    self.assertIn("path", check, task["id"])
                    self.assertNotIn("value", check, task["id"])


class EditContractTests(unittest.TestCase):
    def test_fill_tasks_use_value_eq_not_global_scan(self):
        for tid in sorted(OM_FILL):
            task = load_json(OM / "tasks" / f"{tid}.json")
            ops = {c["op"] for c in task["checks"]}
            self.assertIn("value_eq", ops, tid)
            self.assertNotIn("deep_contains", ops, tid)
            self.assertNotIn("not_contains", ops, tid)
            for check in task["checks"]:
                if check["op"] == "value_eq":
                    self.assertRegex(check["path"], r"^fields\[\d+\]\.value$", tid)

    def test_no_t07_clone(self):
        banned = []
        for task in pack_tasks(OM, "OM"):
            raw = json.dumps(task, ensure_ascii=False)
            files = task.get("submit", {}).get("files") or []
            for check in task["checks"]:
                if check.get("value") == "홍길동":
                    banned.append(f"{task['id']} 홍길동 값")
            if "filled.hwp" in files:
                banned.append(f"{task['id']} filled.hwp")
            if task["id"] in OM_FILL - {"OM02", "OM06"}:
                first_only = (
                    len(task["checks"]) == 1
                    and task["checks"][0].get("path") == "fields[0].value"
                    and task["checks"][0].get("value") == "홍길동"
                )
                if first_only:
                    banned.append(f"{task['id']} T07 단일 검사")
        self.assertEqual(banned, [], banned)

    def test_new_fill_tasks_leave_unfilled_slots_empty(self):
        for tid in ("OM31", "OM32", "OM33", "OM35", "OM45"):
            task = load_json(OM / "tasks" / f"{tid}.json")
            empty = [
                c for c in task["checks"]
                if c.get("op") == "value_eq" and c.get("value") == ""
            ]
            self.assertTrue(empty, f"{tid} 공란 검사 없음")


class JourneyFamilyTests(unittest.TestCase):
    def test_verify_family_allows_expectation_exit(self):
        for tid in sorted(LR_VERIFY):
            task = load_json(LR / "tasks" / f"{tid}.json")
            found = False
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if not cmd or cmd[0] != "verify":
                    continue
                found = True
                self.assertEqual(check.get("expect_exits"), [0, 3], tid)
                self.assertIn(check.get("path"), {"verdict", "passCount"}, tid)
            self.assertTrue(found, tid)

    def test_hwpx_format_expectations_use_hwpx(self):
        for tid in ("LR20", "LR21"):
            task = load_json(LR / "tasks" / f"{tid}.json")
            cmds = walk_cmds(task)
            self.assertTrue(
                any(c and "--expect-format" in c and "hwpx" in c for c in cmds),
                tid,
            )

    def test_svg_family_rejects_unedited_copies(self):
        for tid in sorted(LR_SVG):
            task = load_json(LR / "tasks" / f"{tid}.json")
            ops = {c["op"] for c in task["checks"]}
            self.assertIn("file_exists", ops, tid)
            self.assertIn("differs_from_input", ops, tid)
            self.assertIn("xml_root_eq", ops, tid)
            ref = load_json(LR / "reference" / f"{tid}.json")
            heads = {cmd[0] for cmd in walk_cmds(ref) if cmd}
            self.assertIn("export-svg", heads, tid)
            for cmd in walk_cmds(ref):
                if cmd and cmd[0] == "export-svg":
                    self.assertIn("-p", cmd, tid)
                    self.assertEqual(cmd[cmd.index("-p") + 1], "0", tid)

    def test_extract_family_is_one_based(self):
        for tid in sorted(CD_EXTRACT):
            ref = load_json(CD / "reference" / f"{tid}.json")
            task = load_json(CD / "tasks" / f"{tid}.json")
            found = False
            for cmd in walk_cmds(ref):
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
            hint = task["instructions"]
            self.assertIn("--from", hint, tid)
            ops = {c["op"] for c in task["checks"]}
            self.assertIn("file_exists", ops, tid)
            self.assertIn("differs_from_input", ops, tid)

    def test_ir_family_allows_diff_exit(self):
        for tid in sorted(CD_IR):
            task = load_json(CD / "tasks" / f"{tid}.json")
            ir_checks = [
                c for c in task["checks"]
                if c.get("cmd") and c["cmd"][0] == "ir-diff"
            ]
            self.assertTrue(ir_checks, tid)
            for check in ir_checks:
                self.assertEqual(check.get("expect_exits"), [0, 3], tid)

    def test_convert_family_writes_hwp_and_reads_format(self):
        for tid in sorted(CD_CONVERT):
            task = load_json(CD / "tasks" / f"{tid}.json")
            files = task.get("submit", {}).get("files") or []
            self.assertIn("conv.hwp", files, tid)
            self.assertIn("answer.json", files, tid)
            paths = {c.get("path") for c in task["checks"]}
            self.assertIn("format", paths, tid)
            ref = load_json(CD / "reference" / f"{tid}.json")
            heads = {cmd[0] for cmd in walk_cmds(ref) if cmd}
            self.assertIn("convert", heads, tid)

    def test_scan_family_targets_existing_folders(self):
        folders = set()
        for task in pack_tasks(CD, "CD"):
            for cmd in walk_cmds(task):
                if cmd and cmd[0] == "scan":
                    folders.add(cmd[1])
        self.assertTrue(folders)
        for folder in folders:
            self.assertTrue((REPO_ROOT / folder).is_dir(), folder)
            self.assertTrue(folder.startswith("samples/"), folder)


class ReferencePairTests(unittest.TestCase):
    def test_reference_steps_exist(self):
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for ref in pack_refs(pack, prefix):
                self.assertTrue(ref.get("steps"), ref["id"])

    def test_reference_uses_sub_placeholders_not_file(self):
        leaked = []
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for ref in pack_refs(pack, prefix):
                raw = json.dumps(ref, ensure_ascii=False)
                if "{file:" in raw:
                    leaked.append(ref["id"])
        self.assertEqual(leaked, [], leaked)

    def test_fill_references_call_edit_fill_fields(self):
        for tid in sorted(OM_FILL):
            ref = load_json(OM / "reference" / f"{tid}.json")
            runs = [cmd for cmd in walk_cmds(ref) if cmd and cmd[0] == "edit"]
            self.assertTrue(runs, tid)
            self.assertEqual(runs[0][1], "fill-fields", tid)


class DocumentationTests(unittest.TestCase):
    def test_three_readmes_are_korean_journeys(self):
        for pack, markers in (
            (OM, ("J1", "J6", "실패 모드", "라이브 오라클", "OM31")),
            (LR, ("J1", "J7", "실패 모드", "1 기준", "LR20")),
            (CD, ("J1", "J7", "실패 모드", "max-depth", "CD26")),
        ):
            text = (pack / "README.md").read_text(encoding="utf-8")
            self.assertIn("여정", text)
            for marker in markers:
                self.assertIn(marker, text, f"{pack.name}:{marker}")
            self.assertNotRegex(text, r"^[A-Za-z0-9 ,.\-']+$")

    def test_working_notes_cover_all_three_packs(self):
        text = WORKING.read_text(encoding="utf-8")
        for marker in (
            "OM10", "OM45", "LR11", "LR48", "CD11", "CD48",
            "T07", "deep_contains", "라이브 오라클", "runner",
        ):
            self.assertIn(marker, text, marker)

    def test_readme_lists_new_ids(self):
        om = (OM / "README.md").read_text(encoding="utf-8")
        lr = (LR / "README.md").read_text(encoding="utf-8")
        cd = (CD / "README.md").read_text(encoding="utf-8")
        for tid in ("OM10", "OM34", "OM45"):
            self.assertIn(tid, om, tid)
        for tid in ("LR11", "LR27", "LR48"):
            self.assertIn(tid, lr, tid)
        for tid in ("CD11", "CD34", "CD48"):
            self.assertIn(tid, cd, tid)


class NoNewCliTests(unittest.TestCase):
    def test_commands_are_existing_surface_only(self):
        unknown = []
        for pack, prefix in ((OM, "OM"), (LR, "LR"), (CD, "CD")):
            for obj in pack_tasks(pack, prefix) + pack_refs(pack, prefix):
                for cmd in walk_cmds(obj):
                    if cmd and cmd[0] not in ALLOWED_CMDS:
                        unknown.append(f"{obj['id']}:{cmd[0]}")
        self.assertEqual(unknown, [], unknown)

    def test_no_new_pack_directory(self):
        self.assertTrue((OM / "pack.json").is_file())
        self.assertTrue((LR / "pack.json").is_file())
        self.assertTrue((CD / "pack.json").is_file())
        self.assertFalse((GYM / "packs" / "om-lr-cd-expand").exists())
        self.assertFalse((GYM / "packs" / "objects-media-expand").exists())


if __name__ == "__main__":
    unittest.main()
