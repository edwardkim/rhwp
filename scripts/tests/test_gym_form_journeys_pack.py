"""form-journeys pack 확장 계약 — FJ06+ · README · 기존 명령/연산자/표본만.

이 가드는 PR #5213 의 확장 규칙을 파일만으로 고정한다. 바이너리·네트워크가
없어도 돈다. 새 연산자·새 표본·T07(fields[0]==홍길동) 복제·새 CLI 가
들어오면 red.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACK = GYM / "packs" / "form-journeys"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "gym_form_journeys.md"
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

PACK_COMMANDS = {"edit", "fields", "search"}
ANSWER_OPS = frozenset({"answer_eq", "len_answer_eq"})
BANNED_OPS = {"deep_contains", "not_contains"}
BANNED_CMDS = {
    "batch",
    "run",
    "gate",
    "conformance",
    "export-tables",
    "table-to-csv",
    "csv-to-table",
    "replay",
    "audit",
    "lineage",
}
ALLOWED_SAMPLES = {
    "samples/field-01.hwp",
    "samples/field-01-memo.hwp",
    "samples/form-01.hwp",
    "samples/form-02.hwp",
}
MIN_TASKS = 56


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("FJ*.json"))


def ref_paths():
    return sorted(REFS.glob("FJ*.json"))


def load_tasks():
    return [read_json(p) for p in task_paths()]


def load_refs():
    return {p.stem: read_json(p) for p in ref_paths()}


def fj_num(path: Path) -> int:
    return int(path.stem[2:])


class FormJourneysPackLayoutTests(unittest.TestCase):
    def test_pack_manifest_identity(self):
        manifest = read_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "form-journeys")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("서식", manifest["axis"])
        cmds = set(manifest["requires"]["commands"])
        self.assertEqual(cmds, PACK_COMMANDS)
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)
        self.assertTrue(runner["rhwpVersion"])

    def test_maintainer_still_lists_the_pack(self):
        maintainer = read_json(MAINTAINER)
        self.assertIn("form-journeys", maintainer["packs"])
        self.assertEqual(maintainer["packs"], sorted(maintainer["packs"]))

    def test_readme_exists_and_is_korean(self):
        self.assertTrue(README.is_file(), "gym/packs/form-journeys/README.md 가 없다")
        text = README.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in (
            "서식",
            "라이브 오라클",
            "FJ06",
            "FJ56",
            "기존 연산자",
            "T07",
            "복제하지",
            "fields[0]",
            "홍길동",
        ):
            self.assertIn(needle, text, f"README 에 '{needle}' 가 없다")

    def test_working_notes_exist_and_are_korean(self):
        self.assertTrue(WORKING.is_file(), "mydocs/working/gym_form_journeys.md 가 없다")
        text = WORKING.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("FJ06", "FJ56", "기존 표본", "T07", "audit", "test_gym_packs"):
            self.assertIn(needle, text, f"작업 노트에 '{needle}' 가 없다")

    def test_readme_and_working_name_every_new_task(self):
        readme = README.read_text(encoding="utf-8")
        working = WORKING.read_text(encoding="utf-8")
        missing = []
        for n in range(6, MIN_TASKS + 1):
            tid = f"FJ{n:02d}"
            if tid not in readme:
                missing.append(f"README:{tid}")
            if tid not in working:
                missing.append(f"working:{tid}")
        self.assertEqual(missing, [], f"문서에 빠진 과제: {missing}")


class FormJourneysTaskContractTests(unittest.TestCase):
    def test_fj06_plus_ship_with_paired_reference(self):
        plus = [p for p in task_paths() if fj_num(p) >= 6]
        self.assertGreaterEqual(len(plus), 50, "FJ06+ 과제가 50건 미만이다")
        missing = []
        for path in plus:
            tid = path.stem
            ref = REFS / f"{tid}.json"
            if not ref.is_file():
                missing.append(tid)
                continue
            task = read_json(path)
            ref_obj = read_json(ref)
            self.assertEqual(task["id"], tid)
            self.assertEqual(ref_obj["id"], tid)
            self.assertTrue(ref_obj.get("steps"), tid)
        self.assertEqual(missing, [], f"기준풀이 없음: {missing}")

    def test_task_ids_are_contiguous_fj_prefix(self):
        nums = sorted(fj_num(p) for p in task_paths())
        self.assertEqual(nums[0], 1)
        self.assertGreaterEqual(nums[-1], MIN_TASKS)
        self.assertEqual(nums, list(range(1, nums[-1] + 1)), "FJ 번호에 구멍이 있다")

    def test_every_task_has_required_keys_and_named_checks(self):
        for task in load_tasks():
            tid = task["id"]
            for key in ("id", "tier", "title", "input", "instructions", "submit", "checks"):
                self.assertIn(key, task, f"{tid}: 필수 키 {key}")
            self.assertIsInstance(task["tier"], int)
            self.assertGreaterEqual(task["tier"], 1, tid)
            self.assertLessEqual(task["tier"], 5, tid)
            self.assertTrue(task["title"].strip(), tid)
            self.assertTrue(task["instructions"].strip(), tid)
            self.assertTrue(task["checks"], tid)
            for check in task["checks"]:
                self.assertTrue(check.get("name"), f"{tid}: 이름 없는 검사")
                self.assertIn(check["op"], EXISTING_OPS, f"{tid}: 허용 밖 연산자 {check['op']}")
                self.assertNotIn(check["op"], BANNED_OPS, f"{tid}: 전역 훑기 {check['op']}")

    def test_titles_and_instructions_are_unique_korean(self):
        seen_title = {}
        seen_inst = {}
        for path in task_paths():
            task = read_json(path)
            tid = task["id"]
            title = task["title"]
            inst = task["instructions"]
            self.assertRegex(inst, r"[가-힣]", tid)
            self.assertRegex(title, r"[가-힣]", tid)
            if fj_num(path) >= 6:
                self.assertGreater(len(inst), 200, tid)
            self.assertNotIn(title, seen_title, f"{tid} 제목이 {seen_title.get(title)} 와 중복")
            self.assertNotIn(inst, seen_inst, f"{tid} 지시문이 {seen_inst.get(inst)} 와 같다")
            seen_title[title] = tid
            seen_inst[inst] = tid

    def test_operators_are_existing_only(self):
        unknown = []
        for task in load_tasks():
            for check in task["checks"]:
                if check["op"] not in EXISTING_OPS:
                    unknown.append(f"{task['id']}:{check['op']}")
        self.assertEqual(unknown, [], f"새 연산자: {unknown}")

    def test_commands_stay_inside_pack_requires(self):
        bad = []
        used = set()
        for task in load_tasks():
            for check in task.get("checks", []):
                cmd = check.get("cmd")
                if not cmd:
                    continue
                used.add(cmd[0])
                if cmd[0] not in PACK_COMMANDS:
                    bad.append(f"{task['id']}:{cmd[0]}")
                if cmd[0] in BANNED_CMDS:
                    bad.append(f"{task['id']}:금지 {cmd[0]}")
        self.assertEqual(bad, [], f"pack requires 밖 명령: {bad}")
        self.assertEqual(used, PACK_COMMANDS, f"세 명령을 다 쓰지 않았다: {used}")

    def test_inputs_are_existing_samples(self):
        missing = []
        extra = []
        for task in load_tasks():
            rel = task["input"]
            self.assertTrue(rel.startswith("samples/"), task["id"])
            if not (REPO_ROOT / rel).exists():
                missing.append(f"{task['id']}:{rel}")
            if rel not in ALLOWED_SAMPLES:
                extra.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"없는 표본: {missing}")
        self.assertEqual(extra, [], f"허용 밖 표본: {extra}")

    def test_no_t07_clone(self):
        hits = []
        for task in load_tasks():
            if task["id"] == "T07" or task["id"].startswith("T0"):
                hits.append(f"{task['id']}:core-cli id")
            if task.get("title") == "서식 채움":
                hits.append(f"{task['id']}:title")
            blob = json.dumps(task, ensure_ascii=False)
            for check in task["checks"]:
                path = check.get("path")
                value = check.get("value")
                if path == "fields[0].value" and value == "홍길동":
                    hits.append(f"{task['id']}:fields[0]==홍길동")
                if value == "홍길동":
                    hits.append(f"{task['id']}:value 홍길동")
            if "fields[0].value" in blob and "홍길동" in blob:
                # 거부 문구는 허용, 판정 값은 위에서 잡는다
                for check in task["checks"]:
                    if check.get("path") == "fields[0].value" and check.get("value") not in ("", None):
                        if check.get("value") == "홍길동":
                            hits.append(f"{task['id']}:비공란 홍길동")
        self.assertEqual(hits, [], f"T07 복제 흔적: {hits}")

    def test_fill_tasks_on_field01_leave_company_empty(self):
        """field-01 계열을 채우는 산출 과제는 회사명 공란을 같이 본다."""
        missing = []
        for task in load_tasks():
            if task["submit"]["kind"] != "artifact":
                continue
            if task["input"] not in ( "samples/field-01.hwp", "samples/field-01-memo.hwp"):
                continue
            blob = json.dumps(task, ensure_ascii=False)
            if "fill-fields" not in blob and not any(
                "fill-fields" in json.dumps(load_refs()[task["id"]], ensure_ascii=False)
                for _ in [0]
            ):
                continue
            ref_blob = json.dumps(load_refs()[task["id"]], ensure_ascii=False)
            if "fill-fields" not in ref_blob:
                continue
            has_empty = any(
                c.get("path") == "fields[0].value" and c.get("value") == ""
                for c in task["checks"]
            )
            if not has_empty:
                missing.append(task["id"])
        self.assertEqual(missing, [], f"회사명 공란 검사 없음: {missing}")

    def test_no_hardcoded_golden_counts_in_answer_eq(self):
        bad = []
        for task in load_tasks():
            for check in task["checks"]:
                if check["op"] in ANSWER_OPS and "value" in check:
                    bad.append(task["id"])
        self.assertEqual(bad, [], f"answer_eq 에 박제 value: {bad}")

    def test_answer_tasks_submit_answer_json(self):
        for task in load_tasks():
            kind = task["submit"]["kind"]
            self.assertIn(kind, ("answer", "artifact"), task["id"])
            if kind == "answer":
                self.assertIn("answer.json", task["submit"]["files"], task["id"])

    def test_schema_module_accepts_every_task(self):
        sys.path.insert(0, str(REPO_ROOT))
        from gym.core import schema as gym_schema  # noqa: WPS433

        manifest = read_json(PACK / "pack.json")
        errors = []
        gym_schema.validate_pack(manifest, str(PACK), errors)
        for task in load_tasks():
            gym_schema.validate_task(task, manifest, None, errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_answer_reference_mirrors_check_cmd_path(self):
        refs = load_refs()
        for path in task_paths():
            if fj_num(path) < 6:
                continue
            task = read_json(path)
            answer_checks = [c for c in task["checks"] if c["op"] in ANSWER_OPS]
            if not answer_checks:
                continue
            tid = task["id"]
            steps = refs[tid]["steps"]
            answer = None
            for step in steps:
                if "answer" in step:
                    answer = step["answer"]
                    break
            self.assertIsNotNone(answer, f"{tid}: 기준풀이에 answer 가 없다")
            check_answers = {c["answer"] for c in answer_checks}
            self.assertEqual(
                set(answer),
                check_answers,
                f"{tid}: 기준풀이 키 {sorted(answer)} != 검사 키 {sorted(check_answers)}",
            )
            by_key = {c["answer"]: c for c in answer_checks}
            for key, spec in answer.items():
                check = by_key[key]
                self.assertEqual(spec["cmd"], check["cmd"], f"{tid}/{key} cmd")
                self.assertEqual(spec["path"], check["path"], f"{tid}/{key} path")


class FormJourneysReferenceTests(unittest.TestCase):
    def test_every_task_has_matching_reference(self):
        refs = load_refs()
        for task in load_tasks():
            tid = task["id"]
            self.assertIn(tid, refs, f"{tid} 기준풀이 없음")
            self.assertEqual(refs[tid]["id"], tid)

    def test_no_orphan_references(self):
        task_ids = {t["id"] for t in load_tasks()}
        for stem in load_refs():
            self.assertIn(stem, task_ids, f"고아 기준풀이 {stem}")

    def test_reference_steps_use_input_or_sub_placeholders(self):
        for path in ref_paths():
            ref = read_json(path)
            self.assertTrue(ref.get("steps"), path.name)
            blob = json.dumps(ref, ensure_ascii=False)
            self.assertTrue(
                "{input}" in blob or "{sub:" in blob or "samples/" in blob,
                f"{path.name} 자리표/표본 경로가 없다",
            )

    def test_artifact_reference_writes_submit_files(self):
        refs = load_refs()
        for task in load_tasks():
            if task["submit"]["kind"] != "artifact":
                continue
            blob = json.dumps(refs[task["id"]], ensure_ascii=False)
            for fname in task["submit"]["files"]:
                self.assertIn(fname, blob, f"{task['id']}: 기준풀이가 {fname} 을 쓰지 않는다")


class FormJourneysExpansionScopeTests(unittest.TestCase):
    def test_fj06_plus_cover_form_axes(self):
        axes = {
            "작성자": 0,
            "부서명": 0,
            "이메일": 0,
            "목차1[": 0,
            "dry-run": 0,
            "notFound": 0,
            "ambiguous": 0,
            "sanitize": 0,
            "fieldCount": 0,
            "replace-text": 0,
        }
        refs = load_refs()
        for path in task_paths():
            if fj_num(path) < 6:
                continue
            blob = path.read_text(encoding="utf-8") + json.dumps(
                refs[path.stem], ensure_ascii=False
            )
            for key in axes:
                if key in blob:
                    axes[key] += 1
        for key, count in axes.items():
            self.assertGreaterEqual(count, 2, f"FJ06+ 에 {key} 축이 부족하다: {count}")

    def test_audit_still_passes_for_form_journeys_pack(self):
        spec_name = "gym_audit_form_journeys_pack"
        import importlib.util

        spec = importlib.util.spec_from_file_location(spec_name, GYM / "tools" / "audit.py")
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], f"audit 실패: {report}")
        fj = [p for p in report.get("packs", []) if p["id"] == "form-journeys" and p.get("issues")]
        self.assertEqual(fj, [], f"form-journeys pack 위반: {fj}")


if __name__ == "__main__":
    unittest.main()
