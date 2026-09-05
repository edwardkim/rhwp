"""[#5300] rhwp-form-fill 스킬·픽스처 계약.

실 에이전트가 누름틀을 채우고 메일머지할 때 쓰는 규약이
기존 CLI 표면(fields / fill-fields / 이름[N] / batch fill / dry-run /
verify / sanitize)을 벗어나지 않는지, gym 과 새 edit 로직을 끌어들이지
않았는지를 바이너리 없이 커밋된 파일만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import re
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "rhwp-form-fill"
REF = SKILL / "references"
FIXT = REF / "fixtures"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_form_fill.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
]

REQUIRED_REFS = [
    "00_tree.md",
    "01_fields_survey.md",
    "02_fill_fields.md",
    "03_repeat_occurrence.md",
    "04_batch_fill.md",
    "05_dry_run_verify.md",
    "06_sanitize.md",
    "07_envelopes.md",
    "08_pitfalls.md",
    "09_journeys.md",
    "10_handoff.md",
    "11_failure_signals.md",
    "12_data_formats.md",
    "13_name_field.md",
    "14_insert_image.md",
    "15_axis_choice.md",
    "16_worked_traces.md",
    "17_intent_matrix.md",
    "18_field_catalog.md",
    "19_gate_recipes.md",
    "20_exit_codes.md",
    "README.md",
]

INVENTED_COMMANDS = [
    "edit mail-merge",
    "edit fill-all",
    "edit fill-nth",
    "edit stamp",
    "hwp_doc_mail_merge",
    "batch merge",
]


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_form_fill_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentFormFillSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.tree = load_json(FIXT, "tree.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.occ = load_json(FIXT, "occurrence_catalog.json")
        cls.batch = load_json(FIXT, "batch_rows.json")
        cls.env = load_json(FIXT, "envelope_keys.json")
        cls.fail_signals = load_json(FIXT, "failure_signals.json")
        cls.ladder = load_json(FIXT, "command_ladder.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-form-fill", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 edit 로직", self.skill)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "fields",
            "fill-fields",
            "이름[N]",
            "batch fill",
            "--dry-run",
            "--verify",
            "sanitize",
            "notFound",
            "ambiguous",
            "references/01_fields_survey.md",
            "references/03_repeat_occurrence.md",
            "references/04_batch_fill.md",
            "references/05_dry_run_verify.md",
            "references/06_sanitize.md",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_reference_docs_exist_and_long_enough(self):
        for name in REQUIRED_REFS:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertGreater(len(body), 400, f"{name} 가 너무 짧다")

    def test_index_lists_same_references(self):
        listed = self.idx["references"]
        for name in REQUIRED_REFS:
            self.assertIn(name, listed, name)

    def test_not_gym_and_no_new_edit_logic_in_fixtures(self):
        self.assertTrue(self.idx["notGym"])
        self.assertTrue(self.idx["noNewEditLogic"])
        self.assertTrue(self.tree["notGym"])
        self.assertTrue(self.tree["noNewCli"])
        self.assertTrue(self.tree["noNewEditLogic"])
        self.assertEqual(self.idx["issue"], 5300)
        self.assertEqual(self.tree["issue"], 5300)

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            peer = REPO / ".agents" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_skill_does_not_rewrite_peer_skill_bodies(self):
        # 이 테스트는 경로를 만지지 말라는 계약. 내용 인용이 아니라
        # 작업 범위 선언이 SKILL/index 에 있는지만 본다.
        text = self.skill + read(REF / "10_handoff.md")
        self.assertIn("재작성", text)
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, text)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        joined = "\n".join(blobs)
        for bad in INVENTED_COMMANDS:
            self.assertNotIn(bad, joined, f"발명된 명령: {bad}")

    def test_stop_rule_ids_in_skill_or_failure_chapter(self):
        fail = read(REF / "11_failure_signals.md")
        for rule in self.stops["rules"]:
            rid = rule["id"]
            self.assertTrue(
                rid in self.skill or rid in fail,
                f"정지 {rid} 문서 누락",
            )

    def test_ladder_order_documented(self):
        tree_md = read(REF / "00_tree.md")
        skill = self.skill
        for cmd in ("fields", "fill-fields", "batch fill", "sanitize"):
            self.assertIn(cmd, tree_md, cmd)
            self.assertIn(cmd, skill, cmd)
        box = tree_md.find("살아 있는 동사는")
        self.assertGreaterEqual(box, 0)
        prev = box
        for cmd in ("`fields`", "fill-fields", "batch fill", "sanitize"):
            pos = tree_md.find(cmd, box)
            self.assertGreaterEqual(pos, prev, f"명령 상자 순서 {cmd}")
            prev = pos

    def test_intent_matrix_size_and_schema(self):
        rows = self.intents["intents"]
        self.assertGreaterEqual(len(rows), 120)
        self.assertEqual(self.intents["count"], len(rows))
        ids = set()
        for row in rows:
            self.assertRegex(row["id"], r"^I\d{3}$")
            self.assertTrue(row["utterance"])
            self.assertTrue(row["command"])
            self.assertTrue(row["reference"].endswith(".md"))
            self.assertRegex(row["stop"], r"^F\d{2}$")
            self.assertNotIn(row["id"], ids)
            ids.add(row["id"])
            for bad in INVENTED_COMMANDS:
                self.assertNotIn(bad, row["command"])

    def test_journeys_point_at_known_stops(self):
        known = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 80)
        for j in items:
            self.assertIn(j["stop"], known, j["id"])
            self.assertTrue(j["steps"])
            self.assertTrue(j["notGym"])

    def test_occurrence_zero_based_and_oob(self):
        self.assertTrue(self.occ["zeroBased"])
        self.assertTrue(self.occ["bareKeyMeansFirstMatch"])
        self.assertTrue(self.occ["outOfRangeGoesToNotFound"])
        keys = [c["key"] for c in self.occ["cases"] if "key" in c]
        self.assertIn("목차1[0]", keys)
        self.assertIn("목차1[4]", keys)
        self.assertIn("목차1[5]", keys)
        oob = [c for c in self.occ["cases"] if c.get("outOfRange")]
        self.assertGreaterEqual(len(oob), 2)
        for c in oob:
            self.assertEqual(c.get("landsIn"), "notFound")

    def test_batch_rows_utf8_and_empty_rejected(self):
        self.assertEqual(self.batch["encoding"], "utf-8")
        self.assertEqual(self.batch["rowCount"], 40)
        self.assertTrue(self.batch["stdinIgnored"])
        self.assertEqual(self.batch["emptyCsvRejected"]["exit"], 2)
        self.assertEqual(len(self.batch["rows"]), 40)

    def test_data_files_exist_utf8(self):
        for name in (
            "mailmerge_12.jsonl",
            "mailmerge_12.csv",
            "empty_header_only.csv",
            "row_form01.json",
            "row_field01.json",
            "row_repeat_14.json",
        ):
            path = FIXT / "data" / name
            raw = path.read_bytes()
            raw.decode("utf-8")
            self.assertFalse(raw.startswith(b"\xff\xfe"), name)
            if name.endswith(".json"):
                json.loads(raw.decode("utf-8"))
            if name == "empty_header_only.csv":
                self.assertEqual(raw.decode("utf-8").strip(), "성명,myMsg01")

    def test_envelope_keys_cover_core_commands(self):
        cmds = self.env["commands"]
        for name in ("fields", "fill-fields", "batch-fill", "sanitize"):
            self.assertIn(name, cmds)
        self.assertIn("schemaVersion", cmds["fields"]["required"])
        self.assertIn("notFound", cmds["fill-fields"]["required"])
        self.assertIn("ambiguous", cmds["fill-fields"]["required"])
        self.assertIn("row", cmds["batch-fill"]["extra"])
        self.assertEqual(self.env["exitCodes"]["3"].find("verify") >= 0, True)

    def test_failure_signals_table_aligned(self):
        self.assertGreaterEqual(len(self.fail_signals["signals"]), 10)
        stops = {r["id"] for r in self.stops["rules"]}
        for row in self.fail_signals["signals"]:
            self.assertIn(row["stop"], stops)

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_form_fill"
        self.assertFalse(
            shadow.exists(),
            "픽스처는 references/fixtures 한 곳만. 복제 금지",
        )

    def test_working_doc_and_capability(self):
        text = read(WORKING)
        self.assertIn("#5300", text)
        self.assertIn("rhwp-form-fill", text)
        self.assertIn("gym", text)
        reg = read(REGISTRY)
        self.assertIn("CAP-5300", reg)
        self.assertIn("rhwp-form-fill", reg)

    def test_generator_roundtrip_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5300)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        idx = self.gen.skill_index()
        self.assertEqual(idx["skill"], "rhwp-form-fill")
        self.assertGreaterEqual(len(idx["references"]), 16)

    def test_core_reuse_is_existing_functions(self):
        reuse = self.tree["coreReuse"]
        self.assertIn("set_field_value_by_name", reuse)
        self.assertIn("collect_all_fields", reuse)

    def test_traces_exist(self):
        ids = load_json(FIXT, "traces_index.json")["ids"]
        self.assertGreaterEqual(len(ids), 30)
        for tid in ids:
            path = FIXT / "traces" / f"{tid}.json"
            self.assertTrue(path.is_file(), tid)
            obj = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(obj["id"], tid)

    def test_no_gym_tree_in_skill_dir(self):
        for path in SKILL.rglob("*"):
            rel = path.relative_to(SKILL).as_posix()
            self.assertFalse(rel.startswith("gym"), rel)
            self.assertNotIn("/gym/", f"/{rel}")


if __name__ == "__main__":
    unittest.main()
