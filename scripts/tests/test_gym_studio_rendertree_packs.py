"""studio-e2e · render-tree pack 계약 — 여정·자리 지목·표본 실재.

[#5262] 두 pack 이 기존 CLI 와 samples 만으로 차트 왕복·렌더 트리 추출을
가르는지, 골든 바이트를 박제하지 않는지, 편집에 deep_contains 를 쓰지
않는지, pack.json runner 신원을 바꾸지 않는지 CI 가 항상 확인한다.
바이너리 없이 순수 파일 검사다.
"""

from __future__ import annotations

import json
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
ST = GYM / "packs" / "studio-e2e"
RT = GYM / "packs" / "render-tree"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_studio_rendertree.md"

ST_IDS = [f"ST{i:02d}" for i in range(1, 41)]
RT_IDS = [f"RT{i:02d}" for i in range(1, 41)]

ST_EDIT = {"ST01", "ST02", "ST03", "ST04", "ST05", "ST06", "ST07"}
ST_INQUIRE = {
    "ST08", "ST09", "ST10", "ST11", "ST12", "ST13", "ST14", "ST15", "ST16",
    "ST17", "ST18", "ST19", "ST20", "ST21", "ST31", "ST32", "ST33", "ST34",
    "ST35", "ST36",
}
ST_EXTRACT = {
    "ST22", "ST23", "ST24", "ST25", "ST26", "ST27", "ST28", "ST29", "ST30",
    "ST37", "ST38",
}
ST_FLOOR = {"ST39", "ST40"}

RT_LATER = {
    "RT23": ("1", "002"),
    "RT24": ("1", "002"),
    "RT25": ("2", "003"),
    "RT26": ("1", "002"),
    "RT27": ("2", "003"),
    "RT28": ("3", "004"),
}
RT_FLAG = {
    "RT30": "--show-para-marks",
    "RT31": "--show-control-codes",
    "RT32": "--respect-vpos-reset",
    "RT33": "--show-para-marks",
    "RT34": "--show-control-codes",
    "RT40": "--show-para-marks",
}
RT_STRUCTURE = {"RT35", "RT36", "RT37"}

ALLOWED_ST = {"chart-to-csv", "csv-to-chart"}
ALLOWED_RT = {"export-render-tree", "info"}

ST_RUNNER = {
    "rhwpVersion": "0.8.4",
    "rhwpCommit": "fbca0aa6c22db9a30e6c417190ae4ddfe924773e",
    "capabilitiesSha256": "4767e61c3af751bb2f97af9d0b3e5ffa5cbb5dc70a89cf3ae85987132fa5473d",
}
RT_RUNNER = {
    "rhwpVersion": "0.8.4",
    "rhwpCommit": "4324eb0e4cf1a65f7efb305993a79ac44859a7ca",
    "capabilitiesSha256": "4767e61c3af751bb2f97af9d0b3e5ffa5cbb5dc70a89cf3ae85987132fa5473d",
}

LIVE_ANSWER_PATHS = {
    "chartCount",
    "charts[0].rowCount",
    "charts[0].colCount",
    "charts[0].chart",
    "pageCount",
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
        studio = load_json(ST / "pack.json")
        render = load_json(RT / "pack.json")
        self.assertEqual(studio["id"], "studio-e2e")
        self.assertEqual(render["id"], "render-tree")
        self.assertEqual(studio["kind"], "gymPack")
        self.assertEqual(render["kind"], "gymPack")
        self.assertEqual(studio["schemaVersion"], "1.0")
        self.assertEqual(render["schemaVersion"], "1.0")
        self.assertTrue(studio["axis"].startswith("편집"), studio["axis"])
        self.assertTrue(render["axis"].startswith("조회"), render["axis"])
        self.assertEqual(set(studio["requires"]["commands"]), ALLOWED_ST)
        self.assertEqual(set(render["requires"]["commands"]), ALLOWED_RT)
        self.assertEqual(studio["runner"], ST_RUNNER)
        self.assertEqual(render["runner"], RT_RUNNER)

    def test_schema_accepts_both_packs(self):
        schema = load_core()
        errors = []
        schema.validate_pack(load_json(ST / "pack.json"), str(ST), errors)
        schema.validate_pack(load_json(RT / "pack.json"), str(RT), errors)
        self.assertEqual(errors, [], errors)


class TaskInventoryTests(unittest.TestCase):
    def test_expected_ids_are_complete_and_paired(self):
        for pack, prefix, expected in ((ST, "ST", ST_IDS), (RT, "RT", RT_IDS)):
            task_ids = [p.stem for p in sorted((pack / "tasks").glob(f"{prefix}*.json"))]
            ref_ids = [p.stem for p in sorted((pack / "reference").glob(f"{prefix}*.json"))]
            self.assertEqual(task_ids, expected, prefix)
            self.assertEqual(ref_ids, expected, prefix)
            for tid in expected:
                task = load_json(pack / "tasks" / f"{tid}.json")
                ref = load_json(pack / "reference" / f"{tid}.json")
                self.assertEqual(task["id"], tid)
                self.assertEqual(ref["id"], tid)

    def test_families_partition_studio(self):
        union = ST_EDIT | ST_INQUIRE | ST_EXTRACT | ST_FLOOR
        self.assertEqual(union, set(ST_IDS))
        self.assertEqual(len(ST_EDIT) + len(ST_INQUIRE) + len(ST_EXTRACT) + len(ST_FLOOR), 40)

    def test_every_task_passes_schema(self):
        schema = load_core()
        errors = []
        for pack, prefix in ((ST, "ST"), (RT, "RT")):
            manifest = load_json(pack / "pack.json")
            for task in pack_tasks(pack, prefix):
                schema.validate_task(task, manifest, None, errors)
        self.assertEqual(errors, [], errors)

    def test_every_check_is_named_and_tiers_stay_in_range(self):
        for pack, prefix in ((ST, "ST"), (RT, "RT")):
            for task in pack_tasks(pack, prefix):
                self.assertIsInstance(task["tier"], int, task["id"])
                self.assertGreaterEqual(task["tier"], 1, task["id"])
                self.assertLessEqual(task["tier"], 5, task["id"])
                for check in task["checks"]:
                    self.assertTrue(check.get("name"), task["id"])

    def test_st01_and_rt01_keep_original_contracts(self):
        st01 = load_json(ST / "tasks" / "ST01.json")
        self.assertEqual(st01["input"], "samples/chart/세로막대형/묶은세로막대형.hwp")
        self.assertEqual(st01["submit"]["files"], ["out.hwp"])
        self.assertIn("91.7", st01["instructions"])
        rt01 = load_json(RT / "tasks" / "RT01.json")
        self.assertEqual(rt01["input"], "samples/2010-01-06.hwp")
        self.assertEqual(rt01["submit"]["files"], ["_rt/render_tree_001.json"])
        self.assertEqual(rt01["checks"][0]["minBytes"], 10000)


class SampleAndOracleTests(unittest.TestCase):
    def test_every_input_exists_in_the_repo(self):
        missing = []
        for pack, prefix in ((ST, "ST"), (RT, "RT")):
            for task in pack_tasks(pack, prefix):
                rel = task["input"]
                if not (REPO_ROOT / rel).is_file():
                    missing.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"표본 없음: {missing}")

    def test_asset_csvs_exist_for_edit_tasks(self):
        for tid in sorted(ST_EDIT):
            task = load_json(ST / "tasks" / f"{tid}.json")
            raw = json.dumps(task, ensure_ascii=False)
            match = re.search(r"gym/packs/studio-e2e/assets/(ST\d+-edit\.csv)", raw)
            self.assertIsNotNone(match, tid)
            self.assertTrue((REPO_ROOT / "gym/packs/studio-e2e/assets" / match.group(1)).is_file(), tid)

    def test_no_golden_hashes_or_frozen_live_fields(self):
        frozen = []
        for pack, prefix in ((ST, "ST"), (RT, "RT")):
            for task in pack_tasks(pack, prefix):
                raw = json.dumps(task, ensure_ascii=False)
                if re.search(r'"sha256"|[0-9a-f]{64}', raw, re.I):
                    frozen.append(f"{task['id']} 해시 박제 의심")
                for check in task["checks"]:
                    if check["op"] == "answer_eq" and check.get("path") in LIVE_ANSWER_PATHS:
                        if "value" in check:
                            frozen.append(f"{task['id']} {check['path']} 값 박제")
                    if check["op"] == "value_eq" and check.get("path") in LIVE_ANSWER_PATHS:
                        frozen.append(f"{task['id']} {check['path']} 를 value_eq 로 박제")
        self.assertEqual(frozen, [], frozen)

    def test_live_answer_eq_does_not_embed_expected_literals(self):
        for pack, prefix in ((ST, "ST"), (RT, "RT")):
            for task in pack_tasks(pack, prefix):
                for check in task["checks"]:
                    if check.get("op") not in {"answer_eq", "len_answer_eq"}:
                        continue
                    self.assertIn("cmd", check, task["id"])
                    self.assertIn("path", check, task["id"])
                    self.assertNotIn("value", check, task["id"])


class StudioJourneyTests(unittest.TestCase):
    def test_edit_tasks_use_changed_count_sentinel(self):
        for tid in sorted(ST_EDIT):
            task = load_json(ST / "tasks" / f"{tid}.json")
            ops = {c["op"] for c in task["checks"]}
            self.assertIn("file_exists", ops, tid)
            self.assertIn("differs_from_input", ops, tid)
            self.assertIn("value_eq", ops, tid)
            self.assertNotIn("deep_contains", ops, tid)
            self.assertNotIn("not_contains", ops, tid)
            files = task["submit"]["files"]
            self.assertEqual(len(files), 1, tid)
            self.assertTrue(files[0].startswith("out."), tid)
            if task["input"].endswith(".hwpx"):
                self.assertEqual(files[0], "out.hwpx", tid)
            else:
                self.assertEqual(files[0], "out.hwp", tid)
            for check in task["checks"]:
                if check["op"] == "value_eq":
                    self.assertEqual(check["path"], "changedCount", tid)
                    self.assertEqual(check["value"], 0, tid)
                    self.assertEqual(check["cmd"][0], "csv-to-chart", tid)
                    self.assertIn("--dry-run", check["cmd"], tid)

    def test_edit_references_call_csv_to_chart(self):
        for tid in sorted(ST_EDIT):
            ref = load_json(ST / "reference" / f"{tid}.json")
            runs = [cmd for cmd in walk_cmds(ref) if cmd and cmd[0] == "csv-to-chart"]
            self.assertTrue(runs, tid)
            self.assertIn("-o", runs[0], tid)
            self.assertTrue(any(tok.startswith("{sub:out.") for tok in runs[0]), tid)

    def test_inquire_tasks_are_live_chart_to_csv(self):
        for tid in sorted(ST_INQUIRE):
            task = load_json(ST / "tasks" / f"{tid}.json")
            self.assertEqual(task["submit"]["kind"], "answer", tid)
            found = False
            for check in task["checks"]:
                if check["op"] != "answer_eq":
                    continue
                found = True
                self.assertEqual(check["cmd"][0], "chart-to-csv", tid)
                self.assertIn("--json", check["cmd"], tid)
                self.assertIn(check["path"], LIVE_ANSWER_PATHS, tid)
            self.assertTrue(found, tid)

    def test_chart_numbers_are_one_based(self):
        for task in pack_tasks(ST, "ST"):
            for cmd in walk_cmds(task) + walk_cmds(load_json(ST / "reference" / f"{task['id']}.json")):
                if "--chart" not in cmd:
                    continue
                value = cmd[cmd.index("--chart") + 1]
                self.assertGreaterEqual(int(value), 1, f"{task['id']} --chart 는 1 기준")

    def test_extract_family_writes_csv(self):
        for tid in sorted(ST_EXTRACT):
            task = load_json(ST / "tasks" / f"{tid}.json")
            self.assertIn("chart.csv", task["submit"]["files"], tid)
            ops = {c["op"] for c in task["checks"]}
            self.assertIn("file_exists", ops, tid)
            ref = load_json(ST / "reference" / f"{tid}.json")
            runs = [cmd for cmd in walk_cmds(ref) if cmd and cmd[0] == "chart-to-csv" and "-o" in cmd]
            self.assertTrue(runs, tid)

    def test_scatter_marks_x_header(self):
        task = load_json(ST / "tasks" / "ST25.json")
        cells = [c for c in task["checks"] if c["op"] == "csv_cell_eq"]
        self.assertTrue(cells)
        self.assertEqual(cells[0]["value"], "X")
        self.assertEqual(cells[0]["row"], 0)
        self.assertEqual(cells[0]["col"], 0)

    def test_bom_task_sets_utf8_bom(self):
        task = load_json(ST / "tasks" / "ST26.json")
        ops = {c["op"] for c in task["checks"]}
        self.assertIn("utf8_bom", ops)
        ref = load_json(ST / "reference" / "ST26.json")
        self.assertTrue(any("--bom" in cmd for cmd in walk_cmds(ref)))

    def test_floor_tasks_use_ge_operators(self):
        st39 = load_json(ST / "tasks" / "ST39.json")
        st40 = load_json(ST / "tasks" / "ST40.json")
        self.assertEqual(st39["checks"][0]["op"], "value_ge")
        self.assertEqual(st39["checks"][0]["path"], "chartCount")
        self.assertEqual(st40["checks"][0]["op"], "len_ge")
        self.assertEqual(st40["checks"][0]["path"], "charts")

    def test_report_second_chart_is_chart_two(self):
        task = load_json(ST / "tasks" / "ST19.json")
        cmds = walk_cmds(task)
        self.assertTrue(any("--chart" in c and "2" in c for c in cmds), task)


class RenderTreeJourneyTests(unittest.TestCase):
    def test_every_extract_has_page_root(self):
        for task in pack_tasks(RT, "RT"):
            if task["id"] == "RT01":
                continue
            types = [
                c for c in task["checks"]
                if c["op"] == "json_value_eq" and c.get("path") == "type"
            ]
            self.assertTrue(types, task["id"])
            self.assertEqual(types[0]["value"], "Page", task["id"])

    def test_later_pages_align_p_and_filename(self):
        for tid, (page, suffix) in RT_LATER.items():
            task = load_json(RT / "tasks" / f"{tid}.json")
            ref = load_json(RT / "reference" / f"{tid}.json")
            files = task["submit"]["files"]
            self.assertEqual(files, [f"_rt/render_tree_{suffix}.json"], tid)
            found = False
            for cmd in walk_cmds(ref):
                if cmd and cmd[0] == "export-render-tree":
                    found = True
                    self.assertIn("-p", cmd, tid)
                    self.assertEqual(cmd[cmd.index("-p") + 1], page, tid)
            self.assertTrue(found, tid)

    def test_flag_family_passes_the_flag(self):
        for tid, flag in RT_FLAG.items():
            ref = load_json(RT / "reference" / f"{tid}.json")
            raw = json.dumps(ref, ensure_ascii=False)
            self.assertIn(flag, raw, tid)
            task = load_json(RT / "tasks" / f"{tid}.json")
            self.assertIn(flag, task["instructions"], tid)

    def test_structure_only_skips_pagecount(self):
        for tid in sorted(RT_STRUCTURE):
            task = load_json(RT / "tasks" / f"{tid}.json")
            ops = {c["op"] for c in task["checks"]}
            self.assertIn("file_exists", ops, tid)
            self.assertIn("json_value_eq", ops, tid)
            self.assertNotIn("answer_eq", ops, tid)
            ref = load_json(RT / "reference" / f"{tid}.json")
            self.assertFalse(any("answer" in step for step in ref["steps"]), tid)

    def test_page_index_is_zero_based(self):
        for ref in pack_refs(RT, "RT"):
            for cmd in walk_cmds(ref):
                if not (cmd and cmd[0] == "export-render-tree" and "-p" in cmd):
                    continue
                page = int(cmd[cmd.index("-p") + 1])
                self.assertGreaterEqual(page, 0, ref["id"])
                self.assertLessEqual(page, 3, ref["id"])

    def test_rt01_keeps_large_min_bytes(self):
        other = []
        for task in pack_tasks(RT, "RT"):
            for check in task["checks"]:
                if check["op"] == "file_exists" and task["id"] != "RT01":
                    if check.get("minBytes", 0) >= 10000:
                        other.append(task["id"])
        self.assertEqual(other, [], other)

    def test_export_render_tree_is_in_every_reference(self):
        for ref in pack_refs(RT, "RT"):
            heads = {cmd[0] for cmd in walk_cmds(ref) if cmd}
            self.assertIn("export-render-tree", heads, ref["id"])


class ReferencePairTests(unittest.TestCase):
    def test_reference_steps_exist(self):
        for pack, prefix in ((ST, "ST"), (RT, "RT")):
            for ref in pack_refs(pack, prefix):
                self.assertTrue(ref.get("steps"), ref["id"])

    def test_reference_uses_sub_placeholders_not_file(self):
        leaked = []
        for pack, prefix in ((ST, "ST"), (RT, "RT")):
            for ref in pack_refs(pack, prefix):
                raw = json.dumps(ref, ensure_ascii=False)
                if "{file:" in raw:
                    leaked.append(ref["id"])
        self.assertEqual(leaked, [], leaked)


class DocumentationTests(unittest.TestCase):
    def test_two_readmes_are_korean_journeys(self):
        for pack, markers in (
            (ST, ("J1", "J4", "실패 모드", "라이브 오라클", "ST06", "1부터")),
            (RT, ("J1", "J5", "실패 모드", "0 기준", "RT28", "Page")),
        ):
            text = (pack / "README.md").read_text(encoding="utf-8")
            self.assertIn("여정", text)
            for marker in markers:
                self.assertIn(marker, text, f"{pack.name}:{marker}")
            self.assertNotRegex(text, r"^[A-Za-z0-9 ,.\-']+$")

    def test_working_notes_cover_both_packs(self):
        text = WORKING.read_text(encoding="utf-8")
        for marker in (
            "ST02", "ST40", "RT02", "RT40", "changedCount", "Page",
            "라이브 오라클", "runner", "deep_contains",
        ):
            self.assertIn(marker, text, marker)

    def test_readme_lists_new_ids(self):
        studio = (ST / "README.md").read_text(encoding="utf-8")
        render = (RT / "README.md").read_text(encoding="utf-8")
        for tid in ("ST02", "ST19", "ST25", "ST40"):
            self.assertIn(tid, studio, tid)
        for tid in ("RT02", "RT28", "RT40"):
            self.assertIn(tid, render, tid)


class NoNewCliTests(unittest.TestCase):
    def test_commands_are_existing_surface_only(self):
        unknown = []
        for obj in pack_tasks(ST, "ST") + pack_refs(ST, "ST"):
            for cmd in walk_cmds(obj):
                if cmd and cmd[0] not in ALLOWED_ST:
                    unknown.append(f"{obj['id']}:{cmd[0]}")
        for obj in pack_tasks(RT, "RT") + pack_refs(RT, "RT"):
            for cmd in walk_cmds(obj):
                if cmd and cmd[0] not in ALLOWED_RT:
                    unknown.append(f"{obj['id']}:{cmd[0]}")
        self.assertEqual(unknown, [], unknown)

    def test_no_new_pack_directory(self):
        self.assertTrue((ST / "pack.json").is_file())
        self.assertTrue((RT / "pack.json").is_file())
        self.assertFalse((GYM / "packs" / "studio-rendertree-expand").exists())
        self.assertFalse((GYM / "packs" / "studio-e2e-expand").exists())

    def test_does_not_touch_open_pr_packs(self):
        # 이 시험 파일 자체가 다른 pack JSON 을 고치지 않았음을 고정한다.
        forbidden = (
            "automation",
            "core-cli",
            "casual-rides",
            "expert-challenges",
        )
        here = Path(__file__).read_text(encoding="utf-8")
        for name in forbidden:
            self.assertNotIn(f"packs/{name}/tasks", here, name)


if __name__ == "__main__":
    unittest.main()
