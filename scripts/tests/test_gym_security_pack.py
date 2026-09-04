"""security pack 확장 계약 — SE14+ · README · 기존 연산자/표본만.

이 가드는 PR #5225 의 확장 규칙을 파일만으로 고정한다. 바이너리·네트워크가
없어도 돈다. 새 연산자·새 표본·T07 복제·core-cli 복제가 들어오면 red.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACK = GYM / "packs" / "security"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_security_pack.md"

# gym/core/checks.py REGISTRY 와 같은 기존 연산자만 허용.
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

# pack.json requires.commands — 이 집합 밖 CLI 는 확장 범위가 아니다.
PACK_COMMANDS = {"edit", "inspect", "scan"}

def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("SE*.json"))


def se_id(path: Path) -> str:
    return path.stem


def se_num(path: Path) -> int:
    return int(path.stem[2:])


class SecurityPackLayoutTests(unittest.TestCase):
    def test_pack_manifest_identity(self):
        manifest = read_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "security")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("보안", manifest["axis"])
        cmds = set(manifest["requires"]["commands"])
        self.assertEqual(cmds, PACK_COMMANDS)
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)

    def test_readme_exists_and_is_korean(self):
        self.assertTrue(README.is_file(), "gym/packs/security/README.md 가 없다")
        text = README.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("보안", "스윕", "SE14", "라이브 오라클", "기존 연산자"):
            self.assertIn(needle, text, f"README 에 '{needle}' 가 없다")
        self.assertNotIn("fill-fields", text)

    def test_working_notes_exist_and_are_korean(self):
        self.assertTrue(WORKING.is_file(), "mydocs/working/archives/gym_security_pack.md 가 없다")
        text = WORKING.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("SE14", "기존 표본", "T07", "audit", "test_gym_packs"):
            self.assertIn(needle, text, f"작업 노트에 '{needle}' 가 없다")


class SecurityTaskContractTests(unittest.TestCase):
    def test_se14_plus_ship_with_paired_reference(self):
        plus = [p for p in task_paths() if se_num(p) >= 14]
        self.assertGreaterEqual(len(plus), 50, "SE14+ 과제가 50건 미만이다")
        missing = []
        for path in plus:
            tid = se_id(path)
            ref = REFS / f"{tid}.json"
            if not ref.is_file():
                missing.append(tid)
                continue
            task = read_json(path)
            ref_obj = read_json(ref)
            self.assertEqual(task["id"], tid)
            self.assertEqual(ref_obj["id"], tid)
        self.assertEqual(missing, [], f"기준풀이 없음: {missing}")

    def test_task_ids_are_contiguous_se_prefix(self):
        nums = sorted(se_num(p) for p in task_paths())
        self.assertEqual(nums[0], 1)
        self.assertGreaterEqual(nums[-1], 80)
        self.assertEqual(nums, list(range(1, nums[-1] + 1)), "SE 번호에 구멍이 있다")

    def test_operators_are_existing_only(self):
        unknown = []
        for path in task_paths():
            task = read_json(path)
            for check in task["checks"]:
                op = check["op"]
                if op not in EXISTING_OPS:
                    unknown.append(f"{task['id']}:{op}")
        self.assertEqual(unknown, [], f"새 연산자: {unknown}")

    def test_commands_stay_inside_pack_requires(self):
        bad = []
        for path in task_paths():
            task = read_json(path)
            for check in task.get("checks", []):
                cmd = check.get("cmd")
                if cmd and cmd[0] not in PACK_COMMANDS:
                    bad.append(f"{task['id']}:{cmd[0]}")
        self.assertEqual(bad, [], f"pack requires 밖 명령: {bad}")

    def test_inputs_are_existing_samples(self):
        missing = []
        for path in task_paths():
            task = read_json(path)
            rel = task["input"]
            self.assertTrue(rel.startswith("samples/"), task["id"])
            abs_path = REPO_ROOT / rel
            if not abs_path.exists():
                missing.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"없는 표본: {missing}")

    def test_no_t07_clone(self):
        """T07 복제는 서식 채움 명령·첫 필드 값 대조다. 금지 문구 언급은 복제가 아니다."""
        hits = []
        for path in task_paths():
            task = read_json(path)
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

    def test_every_check_has_a_name(self):
        unnamed = []
        for path in task_paths():
            task = read_json(path)
            for check in task["checks"]:
                if not check.get("name"):
                    unnamed.append(task["id"])
        self.assertEqual(unnamed, [])

    def test_tiers_are_in_range(self):
        for path in task_paths():
            task = read_json(path)
            self.assertIsInstance(task["tier"], int)
            self.assertGreaterEqual(task["tier"], 1, task["id"])
            self.assertLessEqual(task["tier"], 5, task["id"])

    def test_answer_tasks_submit_answer_json(self):
        for path in task_paths():
            task = read_json(path)
            kind = task["submit"]["kind"]
            self.assertIn(kind, ("answer", "artifact"), task["id"])
            if kind == "answer":
                self.assertIn("answer.json", task["submit"]["files"], task["id"])

    def test_reference_steps_use_input_or_sub_placeholders(self):
        for path in sorted(REFS.glob("SE*.json")):
            ref = read_json(path)
            self.assertTrue(ref.get("steps"), path.name)
            blob = json.dumps(ref, ensure_ascii=False)
            self.assertTrue(
                "{input}" in blob or "{sub:" in blob or "samples/" in blob,
                f"{path.name} 자리표/표본 경로가 없다",
            )

    def test_no_hardcoded_golden_counts_in_answer_eq(self):
        """라이브 오라클 — answer_eq 검사에 박제 숫자를 value 로 두지 않는다."""
        bad = []
        for path in task_paths():
            task = read_json(path)
            for check in task["checks"]:
                if check["op"] == "answer_eq" and "value" in check:
                    bad.append(task["id"])
        self.assertEqual(bad, [], f"answer_eq 에 박제 value: {bad}")


class SecurityExpansionScopeTests(unittest.TestCase):
    def test_se14_plus_cover_all_four_axes(self):
        axes = {"unicode": 0, "injection": 0, "hidden-text": 0, "watermark": 0, "redact": 0, "sanitize": 0, "scan": 0}
        for path in task_paths():
            if se_num(path) < 14:
                continue
            blob = path.read_text(encoding="utf-8")
            for key in axes:
                if key in blob:
                    axes[key] += 1
        for key, count in axes.items():
            self.assertGreaterEqual(count, 3, f"SE14+ 에 {key} 축이 부족하다: {count}")

    def test_instructions_are_korean_and_unique(self):
        seen = {}
        for path in task_paths():
            task = read_json(path)
            inst = task["instructions"]
            self.assertRegex(inst, r"[가-힣]", task["id"])
            if se_num(path) >= 14:
                self.assertGreater(len(inst), 200, task["id"])
            dup = seen.get(inst)
            self.assertIsNone(dup, f"{task['id']} 지시문이 {dup} 와 같다")
            seen[inst] = task["id"]

    def test_audit_still_passes_for_security_pack(self):
        sys.path.insert(0, str(GYM / "tools"))
        spec_name = "gym_audit_security_pack"
        import importlib.util

        spec = importlib.util.spec_from_file_location(spec_name, GYM / "tools" / "audit.py")
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], f"audit 실패: {report}")
        sec = [p for p in report.get("packs", []) if p["id"] == "security"]
        self.assertEqual(sec, [], f"security pack 위반: {sec}")


if __name__ == "__main__":
    unittest.main()
