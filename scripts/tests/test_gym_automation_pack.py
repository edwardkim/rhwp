"""automation pack 확장 계약 — AU14+ · README · 기존 명령/표본만.

이 가드는 이슈 #5257 의 확장 규칙을 파일만으로 고정한다. 바이너리·네트워크가
없어도 돈다. 새 연산자·새 표본·T07/XC/WR 복제·다른 pack 편집이 들어오면 red.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACK = GYM / "packs" / "automation"
TASKS = PACK / "tasks"
REFS = PACK / "reference"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "archives" / "gym_automation.md"

# gym/core/checks.py REGISTRY 와 같은 기존 연산자만 허용.
EXISTING_OPS = {
    "same_hash",
    "differs_from_input",
    "file_exists",
    "files_differ",
    "xml_root_eq",
    "json_value_eq",
    "json_keys_contain",
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

# pack.json requires.commands — 채점 cmd[0] 은 이 집합 안이어야 한다.
PACK_COMMANDS = {
    "anchor",
    "audit",
    "audit-report",
    "bundle",
    "conformance",
    "disclose",
    "export-tables",
    "gate",
    "lineage",
    "recall-scope",
    "settle",
    "verify-signature",
}

# devel 에 이미 있던 runner 신원. 이 확장은 복사만 한다.
RUNNER_PIN = {
    "rhwpVersion": "0.8.2",
    "rhwpCommit": "1e8667aa86aeb979119aa9152112b42e4f16a76c",
    "capabilitiesSha256": "2c7c41bc8952b63c4502ec0685b76990e4ece5b178f6dc28a1a495b12880af75",
}

FORBIDDEN_SCORE_CMDS = {"fill-fields", "fields", "inspect", "scan", "replay", "run", "keygen"}


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted(TASKS.glob("AU*.json"))


def au_num(path: Path) -> int:
    return int(path.stem[2:])


class AutomationPackLayoutTests(unittest.TestCase):
    def test_pack_manifest_identity_and_runner_copied(self):
        manifest = read_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "automation")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertIn("자동화", manifest["axis"])
        cmds = set(manifest["requires"]["commands"])
        self.assertEqual(cmds, PACK_COMMANDS)
        self.assertEqual(manifest["runner"], RUNNER_PIN)

    def test_readme_exists_and_is_korean(self):
        self.assertTrue(README.is_file(), "gym/packs/automation/README.md 가 없다")
        text = README.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("자동화", "AU14", "기존 명령", "T07", "XC01", "WR01"):
            self.assertIn(needle, text, f"README 에 '{needle}' 가 없다")
        self.assertNotIn("fill-fields", text)

    def test_working_notes_exist_and_are_korean(self):
        self.assertTrue(WORKING.is_file(), "mydocs/working/archives/gym_automation.md 가 없다")
        text = WORKING.read_text(encoding="utf-8")
        self.assertGreater(len(text), 2000)
        for needle in ("AU14", "기존 표본", "T07", "audit", "test_gym_packs"):
            self.assertIn(needle, text, f"작업 노트에 '{needle}' 가 없다")


class AutomationTaskContractTests(unittest.TestCase):
    def test_au14_plus_ship_with_paired_reference(self):
        plus = [p for p in task_paths() if au_num(p) >= 14]
        self.assertGreaterEqual(len(plus), 50, "AU14+ 과제가 50건 미만이다")
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
        self.assertEqual(missing, [], f"기준풀이 없음: {missing}")

    def test_task_ids_are_contiguous_au_prefix(self):
        nums = sorted(au_num(p) for p in task_paths())
        self.assertEqual(nums[0], 1)
        self.assertGreaterEqual(nums[-1], 70)
        self.assertEqual(nums, list(range(1, nums[-1] + 1)), "AU 번호에 구멍이 있다")

    def test_operators_are_existing_only(self):
        unknown = []
        for path in task_paths():
            task = read_json(path)
            for check in task["checks"]:
                op = check["op"]
                if op not in EXISTING_OPS:
                    unknown.append(f"{task['id']}:{op}")
        self.assertEqual(unknown, [], f"새 연산자: {unknown}")

    def test_score_commands_stay_inside_pack_requires(self):
        bad = []
        for path in task_paths():
            task = read_json(path)
            for check in task.get("checks", []):
                cmd = check.get("cmd")
                if cmd and cmd[0] not in PACK_COMMANDS:
                    bad.append(f"{task['id']}:{cmd[0]}")
        self.assertEqual(bad, [], f"pack requires 밖 채점 명령: {bad}")

    def test_inputs_are_existing_samples(self):
        missing = []
        for path in task_paths():
            task = read_json(path)
            rel = task["input"]
            self.assertTrue(rel.startswith("samples/"), task["id"])
            if not (REPO_ROOT / rel).exists():
                missing.append(f"{task['id']}:{rel}")
        self.assertEqual(missing, [], f"없는 표본: {missing}")

    def test_no_t07_clone(self):
        hits = []
        for path in task_paths():
            task = read_json(path)
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
        self.assertEqual(hits, [], f"T07 복제 흔적: {hits}")

    def test_no_xc_clone(self):
        """XC01 L5 · XC02 unaffected==0 · XC03 4관문 · XC04 depth 3 --deep · XC05 L3 --deep."""
        hits = []
        for path in task_paths():
            if au_num(path) < 14:
                continue
            task = read_json(path)
            blob = json.dumps(task, ensure_ascii=False)
            if '"L5"' in blob:
                hits.append(f"{task['id']}:L5")
            if task.get("title", "").startswith("사다리 완주"):
                hits.append(f"{task['id']}:xc-title")
            cmds = [c.get("cmd") or [] for c in task["checks"]]
            for cmd in cmds:
                if "recall-scope" in cmd:
                    # XC02 는 unaffected==0 과 affected>=2 를 한 과제에 묶는다.
                    paths = {c.get("path") for c in task["checks"]}
                    if "unaffected" in paths and "affected" in paths:
                        hits.append(f"{task['id']}:xc02-combo")
                if cmd[:2] == ["lineage", "{file:caps/c.capsule.json}"] or (
                    "lineage" in cmd and "--deep" in cmd
                ):
                    for check in task["checks"]:
                        if check.get("path") == "depth" and check.get("value") == 3:
                            hits.append(f"{task['id']}:xc04-depth3")
                if "settle" in cmd and "--ledger" in cmd:
                    hits.append(f"{task['id']}:xc03-ledger")
                if "conformance" in cmd and "--level" in cmd:
                    idx = cmd.index("--level")
                    level = cmd[idx + 1] if idx + 1 < len(cmd) else ""
                    if level in {"L3", "L4", "L5"} and "--deep" in cmd:
                        hits.append(f"{task['id']}:xc-conform-deep")
        self.assertEqual(hits, [], f"XC 복제 흔적: {hits}")

    def test_no_wr_clone(self):
        """WR 는 replay 를 채점 명령으로 쓰고 3해시를 answer.json 에 옮긴다."""
        hits = []
        for path in task_paths():
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

    def test_au14_plus_stay_off_boss_tier(self):
        for path in task_paths():
            if au_num(path) < 14:
                continue
            task = read_json(path)
            self.assertLessEqual(task["tier"], 3, f"{task['id']} 보스 티어는 XC 의 일")

    def test_reference_steps_use_input_or_sub_placeholders(self):
        for path in sorted(REFS.glob("AU*.json")):
            if au_num(path) < 14:
                continue
            ref = read_json(path)
            self.assertTrue(ref.get("steps"), path.name)
            blob = json.dumps(ref, ensure_ascii=False)
            self.assertTrue(
                "{input}" in blob or "{sub:" in blob or "samples/" in blob,
                f"{path.name} 자리표/표본 경로가 없다",
            )

    def test_no_hardcoded_golden_hashes(self):
        bad = []
        for path in task_paths():
            if au_num(path) < 14:
                continue
            task = read_json(path)
            blob = json.dumps(task, ensure_ascii=False)
            if "sha256:" in blob.lower() and "inputSha256" not in blob:
                # 박제 해시 금지. 필드 이름 언급은 허용.
                if any(len(tok) == 64 and all(c in "0123456789abcdef" for c in tok)
                       for tok in blob.lower().replace('"', " ").split()):
                    bad.append(task["id"])
        self.assertEqual(bad, [], f"박제 해시: {bad}")


class AutomationExpansionScopeTests(unittest.TestCase):
    def test_au14_plus_cover_command_families(self):
        families = {
            "export-tables": 0,
            "audit": 0,
            "verify-signature": 0,
            "anchor": 0,
            "conformance": 0,
            "disclose": 0,
            "gate": 0,
            "settle": 0,
            "audit-report": 0,
            "recall-scope": 0,
            "bundle": 0,
        }
        for path in task_paths():
            if au_num(path) < 14:
                continue
            task = read_json(path)
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if cmd and cmd[0] in families:
                    families[cmd[0]] += 1
        for key, count in families.items():
            self.assertGreaterEqual(count, 2, f"AU14+ 에 {key} 축이 부족하다: {count}")

    def test_instructions_are_korean_and_unique(self):
        seen = {}
        for path in task_paths():
            task = read_json(path)
            inst = task["instructions"]
            self.assertRegex(inst, r"[가-힣]", task["id"])
            if au_num(path) >= 14:
                self.assertGreater(len(inst), 200, task["id"])
            dup = seen.get(inst)
            self.assertIsNone(dup, f"{task['id']} 지시문이 {dup} 와 같다")
            seen[inst] = task["id"]

    def test_titles_are_unique(self):
        seen = {}
        for path in task_paths():
            title = read_json(path)["title"]
            dup = seen.get(title)
            self.assertIsNone(dup, f"{path.stem} 제목이 {dup} 와 같다")
            seen[title] = path.stem

    def test_other_packs_untouched_by_new_ids(self):
        """새 ID 는 AU 접두만. 다른 pack 폴더에 AU14+ 를 심지 않는다."""
        leaked = []
        packs = GYM / "packs"
        for pack_dir in sorted(p for p in packs.iterdir() if p.is_dir()):
            if pack_dir.name == "automation":
                continue
            for path in (pack_dir / "tasks").glob("AU*.json"):
                leaked.append(f"{pack_dir.name}/{path.name}")
        self.assertEqual(leaked, [], f"다른 pack 에 AU 과제: {leaked}")

    def test_audit_still_passes_for_automation_pack(self):
        spec_name = "gym_audit_automation_pack"
        import importlib.util

        spec = importlib.util.spec_from_file_location(spec_name, GYM / "tools" / "audit.py")
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], f"audit 실패: {report}")
        auto = [p for p in report.get("packs", []) if p["id"] == "automation"]
        self.assertEqual(auto, [], f"automation pack 위반: {auto}")


if __name__ == "__main__":
    unittest.main()
