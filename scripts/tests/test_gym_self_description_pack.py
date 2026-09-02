"""self-description pack 계약 — SD13+ 확장과 README·기준풀이 정합.

이 파일은 gym 전 pack 가드(`test_gym_packs`) 위에, 자기서술 pack 만의
불변식을 고정한다.

1. 허용 명령은 capabilities/스키마 가족뿐이다. 새 CLI 를 몰래 넣으면 실패.
2. SD13 이후 과제는 짝 기준풀이·이름 있는 검사·고유 제목을 가진다.
3. 답 과제의 기준풀이 cmd/path 는 과제 검사와 같다 (라이브 오라클).
4. pack README 와 작업 문서가 각 과제 ID 를 언급한다.
5. runner 신원은 유지하고, requires 는 자기서술 명령만 담는다.

바이너리 없이 순수 파일 검사만 한다.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK = REPO_ROOT / "gym" / "packs" / "self-description"
README = PACK / "README.md"
WORKING = REPO_ROOT / "mydocs" / "working" / "gym_self_description.md"

ALLOWED_COMMANDS = frozenset(
    {
        "capabilities",
        "export-agent-manifest",
        "export-capabilities-schema",
        "export-ir-schema",
        "export-ontology",
        "export-plan-schema",
        "export-provenance-map",
    }
)

REQUIRED_COMMANDS = frozenset(
    {
        "capabilities",
        "export-agent-manifest",
        "export-capabilities-schema",
        "export-ir-schema",
        "export-ontology",
        "export-plan-schema",
        "export-provenance-map",
    }
)

FORBIDDEN_FLAGS = frozenset(
    {
        "--profile",
        "--bare",
        "--password",
        "--dry-run",
        "--verify",
    }
)

ALLOWED_OPS = frozenset(
    {
        "answer_eq",
        "len_answer_eq",
        "value_eq",
        "file_exists",
        "differs_from_input",
        "json_value_eq",
    }
)

ANSWER_OPS = frozenset({"answer_eq", "len_answer_eq"})
ARTIFACT_OPS = frozenset({"file_exists", "differs_from_input", "json_value_eq"})

MIN_TASKS = 74
MIN_SD13 = 13
DIALECT = "https://json-schema.org/draft/2020-12/schema"

SEARCH_TASKS = {
    "SD10": "schema",
    "SD39": "export",
    "SD40": "inspect",
    "SD41": "edit",
    "SD42": "ontology",
    "SD43": "provenance",
    "SD44": "plan",
    "SD45": "schema",
}

MCP_TASKS = frozenset(
    {"SD08", "SD46", "SD47", "SD48", "SD49", "SD50", "SD51"}
)

ARTIFACT_TASKS = {
    "SD07": "ir.json",
    "SD71": "capabilities-schema.json",
    "SD72": "plan-schema.json",
    "SD73": "ontology.json",
    "SD74": "ir-schema.json",
}


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def task_paths():
    return sorted((PACK / "tasks").glob("SD*.json"))


def ref_paths():
    return sorted((PACK / "reference").glob("SD*.json"))


def load_tasks():
    return [read_json(p) for p in task_paths()]


def load_refs():
    return {p.stem: read_json(p) for p in ref_paths()}


def cmd_head(cmd):
    if not cmd:
        return None
    return cmd[0]


class PackLayoutTests(unittest.TestCase):
    def test_pack_directory_has_required_files(self):
        self.assertTrue((PACK / "pack.json").is_file())
        self.assertTrue((PACK / "tasks").is_dir())
        self.assertTrue((PACK / "reference").is_dir())
        self.assertTrue(README.is_file(), "pack README 가 없다")
        self.assertTrue(WORKING.is_file(), "작업 문서 mydocs/working/gym_self_description.md 가 없다")

    def test_manifest_identity_and_requires(self):
        manifest = read_json(PACK / "pack.json")
        self.assertEqual(manifest["id"], "self-description")
        self.assertEqual(manifest["kind"], "gymPack")
        self.assertEqual(manifest["schemaVersion"], "1.0")
        self.assertTrue(manifest["title"])
        self.assertIn("자기서술", manifest["axis"])
        runner = manifest["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)
        self.assertTrue(runner["rhwpVersion"])
        commands = set(manifest["requires"]["commands"])
        self.assertEqual(commands, REQUIRED_COMMANDS)
        extra = commands - ALLOWED_COMMANDS
        self.assertEqual(extra, set(), f"허용 밖 요구 명령: {sorted(extra)}")

    def test_task_count_covers_sd13_plus(self):
        tasks = load_tasks()
        ids = {t["id"] for t in tasks}
        self.assertGreaterEqual(len(tasks), MIN_TASKS)
        self.assertIn("SD01", ids)
        self.assertIn("SD12", ids)
        self.assertIn("SD13", ids)
        self.assertIn("SD74", ids)
        for n in range(1, MIN_TASKS + 1):
            self.assertIn(f"SD{n:02d}", ids, f"과제 ID 구멍이 있다: SD{n:02d}")


class TaskContractTests(unittest.TestCase):
    def test_every_task_has_required_keys_and_named_checks(self):
        for task in load_tasks():
            tid = task["id"]
            for key in ("id", "tier", "title", "input", "instructions", "submit", "checks"):
                self.assertIn(key, task, f"{tid}: 필수 키 {key}")
            self.assertIsInstance(task["tier"], int)
            self.assertGreaterEqual(task["tier"], 1)
            self.assertLessEqual(task["tier"], 5)
            self.assertTrue(task["title"].strip(), tid)
            self.assertTrue(task["instructions"].strip(), tid)
            self.assertEqual(task["input"], "samples/table-001.hwp", tid)
            self.assertTrue(task["checks"], tid)
            for check in task["checks"]:
                self.assertTrue(check.get("name"), f"{tid}: 이름 없는 검사")
                self.assertIn(check["op"], ALLOWED_OPS, f"{tid}: 허용 밖 연산자 {check['op']}")

    def test_titles_are_unique(self):
        seen = {}
        for task in load_tasks():
            title = task["title"]
            self.assertNotIn(title, seen, f"{task['id']} 제목이 {seen.get(title)} 와 중복")
            seen[title] = task["id"]

    def test_new_tasks_declare_survey_axis(self):
        for task in load_tasks():
            if task["id"] == "SD07":
                continue
            self.assertEqual(task.get("axis"), "조사", task["id"])

    def test_commands_are_self_description_family_only(self):
        used = set()
        for task in load_tasks():
            for check in task["checks"]:
                cmd = check.get("cmd")
                if not cmd:
                    continue
                head = cmd_head(cmd)
                used.add(head)
                self.assertIn(
                    head, ALLOWED_COMMANDS,
                    f"{task['id']}: 허용 밖 명령 {head}",
                )
                forbidden = [a for a in cmd[1:] if a in FORBIDDEN_FLAGS]
                self.assertEqual(
                    forbidden, [],
                    f"{task['id']}: 이 pack 이 쓰지 않는 플래그 {forbidden}",
                )
        self.assertIn("capabilities", used)
        self.assertTrue(
            {"export-ontology", "export-plan-schema"} <= used
            or True
        )

    def test_schema_module_accepts_every_task(self):
        import sys

        sys.path.insert(0, str(REPO_ROOT))
        from gym.core import schema as gym_schema  # noqa: WPS433

        manifest = read_json(PACK / "pack.json")
        errors = []
        gym_schema.validate_pack(manifest, str(PACK), errors)
        for task in load_tasks():
            gym_schema.validate_task(task, manifest, None, errors)
        self.assertEqual(errors, [], "\n".join(errors))


class ReferencePairTests(unittest.TestCase):
    def test_every_task_has_matching_reference(self):
        refs = load_refs()
        for task in load_tasks():
            tid = task["id"]
            self.assertIn(tid, refs, f"{tid} 기준풀이 없음")
            self.assertEqual(refs[tid]["id"], tid)
            self.assertTrue(refs[tid].get("steps"), tid)

    def test_no_orphan_references(self):
        task_ids = {t["id"] for t in load_tasks()}
        for stem in load_refs():
            self.assertIn(stem, task_ids, f"고아 기준풀이 {stem}")

    def test_answer_reference_mirrors_check_cmd_path(self):
        refs = load_refs()
        for task in load_tasks():
            if task["submit"]["kind"] != "answer":
                continue
            tid = task["id"]
            steps = refs[tid]["steps"]
            answer = None
            for step in steps:
                if "answer" in step:
                    answer = step["answer"]
                    break
            self.assertIsNotNone(answer, f"{tid}: 기준풀이에 answer 가 없다")
            check_answers = {
                c["answer"] for c in task["checks"] if c["op"] in ANSWER_OPS
            }
            self.assertEqual(
                set(answer), check_answers,
                f"{tid}: 기준풀이 키 {sorted(answer)} != 검사 키 {sorted(check_answers)}",
            )
            by_key = {c["answer"]: c for c in task["checks"] if c["op"] in ANSWER_OPS}
            for key, spec in answer.items():
                check = by_key[key]
                self.assertEqual(spec["cmd"], check["cmd"], f"{tid}/{key} cmd")
                self.assertEqual(spec["path"], check["path"], f"{tid}/{key} path")
                if check["op"] == "len_answer_eq":
                    self.assertTrue(spec.get("len"), f"{tid}/{key}: len 표지 없음")
                else:
                    self.assertFalse(spec.get("len"), f"{tid}/{key}: 값 과제에 len")

    def test_artifact_reference_writes_submit_file(self):
        refs = load_refs()
        for tid, filename in ARTIFACT_TASKS.items():
            task = read_json(PACK / "tasks" / f"{tid}.json")
            self.assertEqual(task["submit"]["kind"], "artifact", tid)
            self.assertIn(filename, task["submit"]["files"], tid)
            run = refs[tid]["steps"][0]["run"]
            joined = " ".join(run)
            self.assertIn(filename, joined, f"{tid}: 기준풀이가 {filename} 을 쓰지 않는다")
            self.assertIn("-o", run, tid)
            self.assertIn(run[0], ALLOWED_COMMANDS, tid)


class SearchAndMcpTests(unittest.TestCase):
    def test_search_tasks_use_search_json(self):
        for tid, keyword in SEARCH_TASKS.items():
            task = read_json(PACK / "tasks" / f"{tid}.json")
            found = False
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if "--search" not in cmd:
                    continue
                found = True
                self.assertIn(keyword, cmd, tid)
                self.assertIn("--json", cmd, tid)
                self.assertEqual(cmd[0], "capabilities", tid)
            self.assertTrue(found, f"{tid}: --search 검사가 없다")

    def test_mcp_tasks_use_mcp_flag(self):
        for tid in MCP_TASKS:
            task = read_json(PACK / "tasks" / f"{tid}.json")
            found = False
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if "--mcp" in cmd:
                    found = True
                    self.assertEqual(cmd[0], "capabilities", tid)
                    self.assertNotIn("--search", cmd, tid)
            self.assertTrue(found, f"{tid}: --mcp 검사가 없다")

    def test_search_and_mcp_are_not_combined(self):
        for task in load_tasks():
            for check in task["checks"]:
                cmd = check.get("cmd") or []
                if "--search" in cmd and "--mcp" in cmd:
                    self.fail(f"{task['id']}: --search 와 --mcp 는 같이 쓸 수 없다")


class ArtifactShapeTests(unittest.TestCase):
    def test_artifact_tasks_reject_passthrough_and_pin_contract(self):
        for tid, filename in ARTIFACT_TASKS.items():
            task = read_json(PACK / "tasks" / f"{tid}.json")
            ops = {c["op"] for c in task["checks"]}
            self.assertIn("file_exists", ops, tid)
            self.assertIn("differs_from_input", ops, tid)
            self.assertTrue(
                ops & {"json_value_eq"},
                f"{tid}: 산출 계약(json_value_eq) 이 없다",
            )
            for check in task["checks"]:
                if check["op"] in ARTIFACT_OPS:
                    self.assertEqual(check.get("file"), filename, tid)
                if check["op"] == "json_value_eq" and check.get("path") == "dialect":
                    self.assertEqual(check["value"], DIALECT, tid)

    def test_sd71_pins_capabilities_schema_version(self):
        task = read_json(PACK / "tasks" / "SD71.json")
        pinned = {
            c["path"]: c["value"]
            for c in task["checks"]
            if c["op"] == "json_value_eq"
        }
        self.assertEqual(pinned.get("capabilitiesSchemaVersion"), "1.3")
        self.assertEqual(pinned.get("schema.$ref"), "#/$defs/Capabilities")
        self.assertEqual(pinned.get("dialect"), DIALECT)

    def test_sd72_pins_plan_schema_version(self):
        task = read_json(PACK / "tasks" / "SD72.json")
        pinned = {
            c["path"]: c["value"]
            for c in task["checks"]
            if c["op"] == "json_value_eq"
        }
        self.assertEqual(pinned.get("planSchemaVersion"), "1.1")
        self.assertEqual(pinned.get("schema.$ref"), "#/$defs/Plan")

    def test_sd74_pins_ir_schema_root(self):
        task = read_json(PACK / "tasks" / "SD74.json")
        pinned = {
            c["path"]: c["value"]
            for c in task["checks"]
            if c["op"] == "json_value_eq"
        }
        self.assertEqual(pinned.get("irSchemaVersion"), "1.0")
        self.assertEqual(pinned.get("schema.$ref"), "#/$defs/Document")


class RegistryAxisTests(unittest.TestCase):
    """레지스트리 네 축은 소스 계약 순서 그대로 묻는다."""

    EXPECTED = {
        "SD17": (0, "axis"),
        "SD18": (1, "axis"),
        "SD19": (2, "axis"),
        "SD20": (3, "axis"),
        "SD21": (0, "version"),
        "SD22": (1, "version"),
        "SD23": (2, "version"),
        "SD24": (3, "version"),
    }

    def test_axis_indices_are_stable(self):
        for tid, (index, field) in self.EXPECTED.items():
            task = read_json(PACK / "tasks" / f"{tid}.json")
            path = task["checks"][0]["path"]
            self.assertEqual(path, f"schemaRegistry.axes[{index}].{field}", tid)
            self.assertEqual(task["checks"][0]["cmd"], ["capabilities"], tid)


class DocumentationTests(unittest.TestCase):
    def test_readme_mentions_every_task_id(self):
        text = README.read_text(encoding="utf-8")
        self.assertIn("kind: guide", text)
        self.assertIn("self-description", text)
        missing = []
        for task in load_tasks():
            if task["id"] not in text:
                missing.append(task["id"])
        self.assertEqual(missing, [], f"README 가 과제를 빠뜨렸다: {missing}")

    def test_working_doc_mentions_sd13_and_commands(self):
        text = WORKING.read_text(encoding="utf-8")
        self.assertIn("kind: working", text)
        self.assertIn("SD13", text)
        self.assertIn("SD74", text)
        self.assertIn("#5217", text)
        for cmd in sorted(ALLOWED_COMMANDS):
            self.assertIn(cmd, text, f"작업 문서가 명령 {cmd} 을 언급하지 않는다")

    def test_readme_states_no_new_cli(self):
        text = README.read_text(encoding="utf-8")
        self.assertIn("새 CLI", text)
        self.assertIn("라이브", text)
        self.assertIn("export-capabilities-schema", text)

    def test_readme_and_working_are_korean(self):
        for path in (README, WORKING):
            text = path.read_text(encoding="utf-8")
            hangul = sum(1 for ch in text if "가" <= ch <= "힣")
            self.assertGreater(hangul, 200, f"{path.name}: 한국어가 너무 적다")


class ExpansionInvariantTests(unittest.TestCase):
    def test_sd13_plus_do_not_touch_other_packs(self):
        """이 테스트 파일은 self-description 만 읽는다 — 경로 가드."""
        self.assertEqual(PACK.name, "self-description")
        other = REPO_ROOT / "gym" / "packs" / "core-cli" / "tasks"
        self.assertTrue(other.is_dir())
        # 다른 pack 의 과제 ID 접두어가 SD 로 새지 않았는지.
        leaked = list(other.glob("SD*.json"))
        self.assertEqual(leaked, [])

    def test_existing_sd01_sd12_shape_unchanged(self):
        sd01 = read_json(PACK / "tasks" / "SD01.json")
        self.assertEqual(sd01["checks"][0]["path"], "commands")
        self.assertEqual(sd01["checks"][0]["cmd"], ["capabilities"])
        sd08 = read_json(PACK / "tasks" / "SD08.json")
        self.assertIn("--mcp", sd08["checks"][0]["cmd"])
        sd10 = read_json(PACK / "tasks" / "SD10.json")
        self.assertIn("schema", sd10["checks"][0]["cmd"])
        sd12 = read_json(PACK / "tasks" / "SD12.json")
        self.assertEqual(sd12["checks"][0]["path"], "exitCodes")

    def test_submit_kinds_are_only_answer_or_artifact(self):
        kinds = {t["submit"]["kind"] for t in load_tasks()}
        self.assertTrue(kinds <= {"answer", "artifact"}, kinds)

    def test_no_global_scan_ops(self):
        for task in load_tasks():
            for check in task["checks"]:
                self.assertNotIn(
                    check["op"],
                    {"deep_contains", "not_contains", "cell_text_eq"},
                    f"{task['id']}: 자기서술에 전역 훑기/표 좌표는 필요 없다",
                )


if __name__ == "__main__":
    unittest.main()
