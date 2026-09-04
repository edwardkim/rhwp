"""expert-challenges pack 확장 계약 — XC06+ · README · 기존 명령/연산자/표본만.

이 가드는 이슈 #5261 의 확장 규칙을 파일만으로 고정한다. 바이너리·네트워크가
없어도 돈다. 새 연산자·새 표본·T07/AU14+/WR/XC01-05 복제·다른 pack 편집이
들어오면 red.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACK = GYM / "packs" / "expert-challenges"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_expert_challenges.md"

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

PACK_COMMANDS = {
    "keygen",
    "replay",
    "run",
    "anchor",
    "settle",
    "conformance",
    "recall-scope",
    "lineage",
    "audit-report",
    "export-tables",
}

# devel 에 이미 있던 runner 신원. 이 확장은 복사만 한다.
RUNNER_PIN = {
    "rhwpVersion": "0.8.4",
    "rhwpCommit": "34ba36fa3c48f37867fbedd16614fdb7e8f44709",
    "capabilitiesSha256": "6e2b231fad617b618fda19663752fc1a57d56086ade31abade84d8e8e000de4c",
}

SCORE_FORBIDDEN = {"fill-fields", "fields", "inspect", "scan", "bundle", "disclose", "gate", "verify-signature"}
ALLOWED_SAMPLES = {
    "samples/table-001.hwp",
    "samples/table-004.hwp",
    "samples/multi-table-001.hwp",
    "samples/multi-table-002.hwp",
    "samples/inner-table-01.hwp",
    "samples/hwp_table_test.hwp",
    "samples/table-complex.hwp",
    "samples/hwpx/basic-table-01.hwpx",
}
MIN_TASKS = 55


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("XC*.json"))


def ref_paths():
    return sorted(REFS.glob("XC*.json"))


def xc_num(path: Path) -> int:
    return int(path.stem[2:])


def load_tasks():
    return [read_json(p) for p in task_paths()]


class ExpertChallengesPackLayoutTests(unittest.TestCase):
    def test_pack_manifest_identity_and_runner_copied(self):
        manifest = read_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "expert-challenges")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("자동화", manifest["axis"])
        cmds = set(manifest["requires"]["commands"])
        self.assertEqual(cmds, PACK_COMMANDS)
        self.assertEqual(manifest["runner"], RUNNER_PIN)

    def test_readme_exists_and_is_korean(self):
        self.assertTrue(README.is_file(), "gym/packs/expert-challenges/README.md 가 없다")
        text = README.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in (
            "보스",
            "사다리",
            "XC06",
            "XC55",
            "기존 연산자",
            "기존 명령",
            "T07",
            "복제하지",
            "fields[0]",
            "홍길동",
            "AU14",
            "WR",
            "라이브 오라클",
        ):
            self.assertIn(needle, text, f"README 에 '{needle}' 가 없다")
        self.assertNotIn("fill-fields", text)

    def test_working_notes_exist_and_are_korean(self):
        self.assertTrue(WORKING.is_file(), "mydocs/working/archives/gym_expert_challenges.md 가 없다")
        text = WORKING.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("XC06", "XC55", "기존 표본", "T07", "audit", "test_gym_packs"):
            self.assertIn(needle, text, f"작업 노트에 '{needle}' 가 없다")

    def test_readme_and_working_name_every_new_task(self):
        readme = README.read_text(encoding="utf-8")
        working = WORKING.read_text(encoding="utf-8")
        missing = []
        for n in range(6, MIN_TASKS + 1):
            tid = f"XC{n:02d}"
            if tid not in readme:
                missing.append(f"README:{tid}")
            if tid not in working:
                missing.append(f"working:{tid}")
        self.assertEqual(missing, [], f"문서에 빠진 과제: {missing}")


class ExpertChallengesTaskContractTests(unittest.TestCase):
    def test_xc06_plus_ship_with_paired_reference(self):
        plus = [p for p in task_paths() if xc_num(p) >= 6]
        self.assertGreaterEqual(len(plus), 50, "XC06+ 과제가 50건 미만이다")
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

    def test_task_ids_are_contiguous_xc_prefix(self):
        nums = sorted(xc_num(p) for p in task_paths())
        self.assertEqual(nums[0], 1)
        self.assertGreaterEqual(nums[-1], MIN_TASKS)
        self.assertEqual(nums, list(range(1, nums[-1] + 1)), "XC 번호에 구멍이 있다")

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
            if xc_num(path) >= 6:
                self.assertGreater(len(inst), 200, tid)
                self.assertGreaterEqual(task["tier"], 4, f"{tid} 보스 티어가 아니다")
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

    def test_score_commands_stay_inside_pack_requires(self):
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
                if cmd[0] in SCORE_FORBIDDEN:
                    bad.append(f"{task['id']}:금지 {cmd[0]}")
        self.assertEqual(bad, [], f"pack requires 밖 채점 명령: {bad}")
        for needed in ("lineage", "recall-scope", "conformance", "settle", "audit-report", "export-tables", "anchor"):
            self.assertIn(needed, used, f"XC06+ 가 {needed} 를 채점에 쓰지 않았다")

    def test_inputs_are_existing_samples(self):
        missing = []
        extra = []
        for task in load_tasks():
            rel = task["input"]
            self.assertTrue(rel.startswith("samples/"), task["id"])
            if not (REPO_ROOT / rel).exists():
                missing.append(f"{task['id']}:{rel}")
            if xc_num(Path(f"{task['id']}.json")) >= 6 and rel not in ALLOWED_SAMPLES:
                extra.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"없는 표본: {missing}")
        self.assertEqual(extra, [], f"허용 밖 표본: {extra}")

    def test_no_t07_clone(self):
        hits = []
        for task in load_tasks():
            if task["id"].startswith("T0"):
                hits.append(f"{task['id']}:core-cli id")
            if task.get("title") == "서식 채움":
                hits.append(f"{task['id']}:title")
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] in ("fields", "fill-fields"):
                    hits.append(f"{task['id']}:{cmd[0]}")
                if "fill-fields" in cmd:
                    hits.append(f"{task['id']}:fill-fields")
                if check.get("value") == "홍길동":
                    hits.append(f"{task['id']}:홍길동")
                if check.get("path") == "fields[0].value":
                    hits.append(f"{task['id']}:fields[0].value")
        self.assertEqual(hits, [], f"T07 복제 흔적: {hits}")

    def test_no_wr_clone(self):
        hits = []
        for path in task_paths():
            if xc_num(path) < 6:
                continue
            task = read_json(path)
            if task["id"].startswith("WR"):
                hits.append(f"{task['id']}:wr-id")
            if task.get("title") == "일일 영수증 3해시 발급":
                hits.append(f"{task['id']}:wr-title")
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] == "replay":
                    hits.append(f"{task['id']}:replay-score")
                if check.get("op") == "answer_eq" and check.get("answer") in {
                    "inputSha256",
                    "planSha256",
                    "outputSha256",
                }:
                    hits.append(f"{task['id']}:wr-3hash")
        self.assertEqual(hits, [], f"WR 복제 흔적: {hits}")

    def test_no_au14_clone(self):
        hits = []
        au_titles = {
            "계획서 옆칸 원자 실행",
            "둘째 행 계획 실행",
            "table-004 첫 칸 계획",
            "table-004 적합성 L1",
            "table-004 청구 검증",
            "내부표 2링크 리콜",
        }
        for path in task_paths():
            task = read_json(path)
            if task["id"].startswith("AU"):
                hits.append(f"{task['id']}:au-id")
            if task.get("title") in au_titles:
                hits.append(f"{task['id']}:au-title")
            if xc_num(path) >= 6 and task["tier"] <= 3:
                hits.append(f"{task['id']}:au-tier")
        self.assertEqual(hits, [], f"AU14+ 복제 흔적: {hits}")

    def test_no_xc01_05_clone(self):
        """XC01 L5 · XC02 unaffected+affected · XC03 4관문 · XC04 depth 3 --deep · XC05 L3+audit."""
        hits = []
        for path in task_paths():
            if xc_num(path) < 6:
                continue
            task = read_json(path)
            blob = json.dumps(task, ensure_ascii=False)
            if task.get("title", "").startswith("사다리 완주"):
                hits.append(f"{task['id']}:xc01-title")
            paths = {c.get("path") for c in task["checks"]}
            cmds = [c.get("cmd") or [] for c in task["checks"]]
            for cmd in cmds:
                if "conformance" in cmd and "--level" in cmd:
                    idx = cmd.index("--level")
                    level = cmd[idx + 1] if idx + 1 < len(cmd) else ""
                    if level == "L5":
                        hits.append(f"{task['id']}:xc01-l5")
                    if level == "L3" and "--deep" in cmd:
                        if any("audit-report" in (c.get("cmd") or []) for c in task["checks"]):
                            hits.append(f"{task['id']}:xc05-l3-audit")
                if "recall-scope" in cmd:
                    if "unaffected" in paths and "affected" in paths:
                        hits.append(f"{task['id']}:xc02-combo")
                if "lineage" in cmd and "--deep" in cmd:
                    for check in task["checks"]:
                        if check.get("path") == "depth" and check.get("value") == 3:
                            hits.append(f"{task['id']}:xc04-depth3")
                if "settle" in cmd and "--ledger" in cmd:
                    gate_paths = {"capsuleOk", "gateOk", "ledgerOk", "workorderOk"}
                    if gate_paths <= paths:
                        hits.append(f"{task['id']}:xc03-4gate")
            if "unaffected" in blob and "affected" in blob:
                if "unaffected" in paths and "affected" in paths:
                    hits.append(f"{task['id']}:xc02-paths")
        self.assertEqual(hits, [], f"XC01-05 복제 흔적: {hits}")

    def test_xc01_05_files_unchanged(self):
        for n in range(1, 6):
            tid = f"XC{n:02d}"
            task = read_json(TASKS / f"{tid}.json")
            self.assertEqual(task["id"], tid)
            self.assertIn(task["title"], {
                "사다리 완주 — 적합성 L5",
                "오염 리콜 드릴 — 계보 전파",
                "정산 완주 — 원장 4관문",
                "계보 완주 — 3세대 사슬",
                "감사 표준 발급 — 서명 귀속 L3",
            })

    def test_no_hardcoded_golden_hashes(self):
        bad = []
        for path in task_paths():
            if xc_num(path) < 6:
                continue
            blob = path.read_text(encoding="utf-8").lower()
            tokens = blob.replace('"', " ").replace(":", " ").split()
            if any(len(tok) == 64 and all(c in "0123456789abcdef" for c in tok) for tok in tokens):
                bad.append(path.stem)
        self.assertEqual(bad, [], f"박제 해시: {bad}")

    def test_schema_module_accepts_every_task(self):
        sys.path.insert(0, str(REPO_ROOT))
        from gym.core import schema as gym_schema  # noqa: WPS433

        manifest = read_json(PACK / "pack.json")
        errors = []
        gym_schema.validate_pack(manifest, str(PACK), errors)
        for task in load_tasks():
            gym_schema.validate_task(task, manifest, None, errors)
        self.assertEqual(errors, [], "\n".join(errors))


class ExpertChallengesReferenceTests(unittest.TestCase):
    def test_every_task_has_matching_reference(self):
        refs = {p.stem: read_json(p) for p in ref_paths()}
        for task in load_tasks():
            tid = task["id"]
            self.assertIn(tid, refs, f"{tid} 기준풀이 없음")
            self.assertEqual(refs[tid]["id"], tid)

    def test_no_orphan_references(self):
        task_ids = {t["id"] for t in load_tasks()}
        for path in ref_paths():
            self.assertIn(path.stem, task_ids, f"고아 기준풀이 {path.stem}")

    def test_reference_steps_use_input_or_sub_placeholders(self):
        for path in ref_paths():
            if xc_num(path) < 6:
                continue
            ref = read_json(path)
            self.assertTrue(ref.get("steps"), path.name)
            blob = json.dumps(ref, ensure_ascii=False)
            self.assertTrue(
                "{input}" in blob or "{sub:" in blob or "samples/" in blob,
                f"{path.name} 자리표/표본 경로가 없다",
            )

    def test_artifact_reference_writes_submit_files(self):
        refs = {p.stem: read_json(p) for p in ref_paths()}
        for task in load_tasks():
            if xc_num(Path(f"{task['id']}.json")) < 6:
                continue
            if task["submit"]["kind"] != "artifact":
                continue
            blob = json.dumps(refs[task["id"]], ensure_ascii=False)
            for fname in task["submit"]["files"]:
                self.assertIn(fname, blob, f"{task['id']}: 기준풀이가 {fname} 을 쓰지 않는다")


class ExpertChallengesExpansionScopeTests(unittest.TestCase):
    def test_xc06_plus_cover_command_families(self):
        families = {
            "lineage": 0,
            "recall-scope": 0,
            "conformance": 0,
            "settle": 0,
            "audit-report": 0,
            "export-tables": 0,
            "anchor": 0,
        }
        for path in task_paths():
            if xc_num(path) < 6:
                continue
            task = read_json(path)
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] in families:
                    families[cmd[0]] += 1
        for key, count in families.items():
            self.assertGreaterEqual(count, 2, f"XC06+ 에 {key} 축이 부족하다: {count}")

    def test_other_packs_untouched_by_new_ids(self):
        leaked = []
        packs = GYM / "packs"
        for pack_dir in sorted(p for p in packs.iterdir() if p.is_dir()):
            if pack_dir.name == "expert-challenges":
                continue
            for path in (pack_dir / "tasks").glob("XC*.json"):
                leaked.append(f"{pack_dir.name}/{path.name}")
        self.assertEqual(leaked, [], f"다른 pack 에 XC 과제: {leaked}")

    def test_audit_still_passes_for_expert_challenges_pack(self):
        spec_name = "gym_audit_expert_challenges_pack"
        import importlib.util

        spec = importlib.util.spec_from_file_location(spec_name, GYM / "tools" / "audit.py")
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], f"audit 실패: {report}")
        xc = [p for p in report.get("packs", []) if p["id"] == "expert-challenges" and p.get("issues")]
        self.assertEqual(xc, [], f"expert-challenges pack 위반: {xc}")


if __name__ == "__main__":
    unittest.main()
