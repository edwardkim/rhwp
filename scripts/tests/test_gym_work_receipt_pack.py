"""work-receipt pack 확장 계약 — WR05+ · README · 기존 명령/연산자/표본만.

이 가드는 PR #5238 의 확장 규칙을 파일만으로 고정한다. 바이너리·네트워크가
없어도 돈다. 새 연산자·새 표본·T07 복제·XC01–XC05 복제·AU02/AU07/AU12
복제가 들어오면 red.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACK = GYM / "packs" / "work-receipt"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "gym_work_receipt.md"
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

PACK_COMMANDS = {"audit", "lineage", "replay"}
ANSWER_OPS = frozenset({"answer_eq", "len_answer_eq"})
FORBIDDEN_CMDS = {
    "conformance",
    "settle",
    "recall-scope",
    "keygen",
    "gate",
    "fill-fields",
    "fields",
    "verify-signature",
    "audit-report",
    "anchor",
}
FORBIDDEN_FLAGS = {"--deep", "--sign-key", "--keyring", "--anchor-log"}
MIN_TASKS = 56


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("WR*.json"))


def ref_paths():
    return sorted(REFS.glob("WR*.json"))


def load_tasks():
    return [read_json(p) for p in task_paths()]


def load_refs():
    return {p.stem: read_json(p) for p in ref_paths()}


def wr_num(path: Path) -> int:
    return int(path.stem[2:])


class WorkReceiptPackLayoutTests(unittest.TestCase):
    def test_pack_manifest_identity(self):
        manifest = read_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "work-receipt")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("영수증", manifest["axis"])
        cmds = set(manifest["requires"]["commands"])
        self.assertEqual(cmds, PACK_COMMANDS)
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)
        self.assertTrue(runner["rhwpVersion"])

    def test_maintainer_still_lists_the_pack(self):
        maintainer = read_json(MAINTAINER)
        self.assertIn("work-receipt", maintainer["packs"])
        self.assertEqual(maintainer["packs"], sorted(maintainer["packs"]))

    def test_readme_exists_and_is_korean(self):
        self.assertTrue(README.is_file(), "gym/packs/work-receipt/README.md 가 없다")
        text = README.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in (
            "영수증",
            "라이브 오라클",
            "WR05",
            "WR56",
            "기존 연산자",
            "XC01",
            "T07",
            "AU02",
        ):
            self.assertIn(needle, text, f"README 에 '{needle}' 가 없다")
        self.assertIn("복제하지", text)

    def test_working_notes_exist_and_are_korean(self):
        self.assertTrue(WORKING.is_file(), "mydocs/working/gym_work_receipt.md 가 없다")
        text = WORKING.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("WR05", "WR56", "기존 표본", "T07", "audit", "test_gym_packs"):
            self.assertIn(needle, text, f"작업 노트에 '{needle}' 가 없다")

    def test_readme_and_working_name_every_new_task(self):
        readme = README.read_text(encoding="utf-8")
        working = WORKING.read_text(encoding="utf-8")
        missing = []
        for n in range(5, MIN_TASKS + 1):
            tid = f"WR{n:02d}"
            if tid not in readme:
                missing.append(f"README:{tid}")
            if tid not in working:
                missing.append(f"working:{tid}")
        self.assertEqual(missing, [], f"문서에 빠진 과제: {missing}")


class WorkReceiptTaskContractTests(unittest.TestCase):
    def test_wr05_plus_ship_with_paired_reference(self):
        plus = [p for p in task_paths() if wr_num(p) >= 5]
        self.assertGreaterEqual(len(plus), 50, "WR05+ 과제가 50건 미만이다")
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

    def test_task_ids_are_contiguous_wr_prefix(self):
        nums = sorted(wr_num(p) for p in task_paths())
        self.assertEqual(nums[0], 1)
        self.assertGreaterEqual(nums[-1], MIN_TASKS)
        self.assertEqual(nums, list(range(1, nums[-1] + 1)), "WR 번호에 구멍이 있다")

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
            if wr_num(path) >= 5:
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
        self.assertEqual(bad, [], f"pack requires 밖 명령: {bad}")
        self.assertEqual(used, PACK_COMMANDS, f"세 명령을 다 쓰지 않았다: {used}")

    def test_inputs_are_existing_samples(self):
        missing = []
        for task in load_tasks():
            rel = task["input"]
            self.assertTrue(rel.startswith("samples/"), task["id"])
            if not (REPO_ROOT / rel).exists():
                missing.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"없는 표본: {missing}")

    def test_no_t07_clone(self):
        hits = []
        for task in load_tasks():
            if task["id"].startswith("T0"):
                hits.append(f"{task['id']}:core-cli id")
            if task.get("title") == "서식 채움":
                hits.append(f"{task['id']}:title")
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] == "fields":
                    hits.append(f"{task['id']}:fields 명령")
                if "fill-fields" in cmd:
                    hits.append(f"{task['id']}:fill-fields")
                if check.get("value") == "홍길동":
                    hits.append(f"{task['id']}:홍길동")
        self.assertEqual(hits, [], f"T07 복제 흔적: {hits}")

    def test_no_xc_or_ladder_clone(self):
        hits = []
        for task in load_tasks():
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] in FORBIDDEN_CMDS:
                    hits.append(f"{task['id']}:금지 명령 {cmd[0]}")
                for flag in FORBIDDEN_FLAGS:
                    if flag in cmd:
                        hits.append(f"{task['id']}:금지 플래그 {flag}")
                if check.get("path") == "depth" and check.get("value") in (2, 3):
                    hits.append(f"{task['id']}:depth=={check.get('value')} 복제")
                if check.get("path") == "verdict" and check.get("value") in (
                    "conformant",
                    "allow",
                ):
                    hits.append(f"{task['id']}:사다리 판정 복제")
        self.assertEqual(hits, [], f"XC/사다리 복제 흔적: {hits}")

    def test_no_au02_rate_plus_floor_clone(self):
        hits = []
        for task in load_tasks():
            has_rate = False
            has_floor = False
            for check in task["checks"]:
                if check.get("path") == "reproducedRate" and check.get("value") == 1.0:
                    has_rate = True
                if check.get("path") == "total" and check.get("op") == "value_ge":
                    has_floor = True
            if has_rate and has_floor:
                hits.append(task["id"])
        self.assertEqual(hits, [], f"AU02 복제: {hits}")

    def test_no_hardcoded_golden_counts_in_answer_eq(self):
        bad = []
        for task in load_tasks():
            for check in task["checks"]:
                if check["op"] == "answer_eq" and "value" in check:
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
        for task in load_tasks():
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


class WorkReceiptReferenceTests(unittest.TestCase):
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

    def test_artifact_reference_writes_capsule(self):
        refs = load_refs()
        for task in load_tasks():
            files = task["submit"].get("files") or []
            capsules = [f for f in files if f.endswith(".capsule.json")]
            if not capsules:
                continue
            blob = json.dumps(refs[task["id"]], ensure_ascii=False)
            for cap in capsules:
                self.assertIn(
                    cap,
                    blob,
                    f"{task['id']}: 기준풀이가 {cap} 을 쓰지 않는다",
                )
            self.assertIn("--capsule", blob, task["id"])

    def test_reference_steps_use_input_or_sub_placeholders(self):
        for path in ref_paths():
            ref = read_json(path)
            self.assertTrue(ref.get("steps"), path.name)
            blob = json.dumps(ref, ensure_ascii=False)
            self.assertTrue(
                "{input}" in blob or "{sub:" in blob or "samples/" in blob,
                f"{path.name} 자리표/표본 경로가 없다",
            )


class WorkReceiptExpansionScopeTests(unittest.TestCase):
    def test_wr05_plus_cover_receipt_axes(self):
        axes = {
            "attest": 0,
            "verify": 0,
            "lineage": 0,
            "audit": 0,
            "parentOk": 0,
            "lineageOk": 0,
            "inputSha256": 0,
        }
        for path in task_paths():
            if wr_num(path) < 5:
                continue
            blob = path.read_text(encoding="utf-8")
            for key in axes:
                if key in blob:
                    axes[key] += 1
        for key, count in axes.items():
            self.assertGreaterEqual(count, 2, f"WR05+ 에 {key} 축이 부족하다: {count}")

    def test_audit_still_passes_for_work_receipt_pack(self):
        spec_name = "gym_audit_work_receipt_pack"
        import importlib.util

        spec = importlib.util.spec_from_file_location(spec_name, GYM / "tools" / "audit.py")
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], f"audit 실패: {report}")
        wr = [p for p in report.get("packs", []) if p["id"] == "work-receipt"]
        self.assertEqual(wr, [], f"work-receipt pack 위반: {wr}")


if __name__ == "__main__":
    unittest.main()
