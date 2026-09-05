"""[#5342] rhwp-knowledge-map 스킬·픽스처 계약.

실 에이전트가 llms.txt → agent_knowledge_map.md → canonical 하나
순으로 문서에 들어갈 때, 지도 행을 재서술하지 않는지, 필드 이름을
§2 에서만 가져오는지, gym·새 CLI·대전/표면 재작성을 끌어들이지
않았는지를 바이너리 없이 커밋된 파일만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import re
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "rhwp-knowledge-map"
REF = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_knowledge_map_skill.md"
MAP = REPO / "mydocs" / "manual" / "agent_knowledge_map.md"
LLMS = REPO / "llms.txt"

FORBIDDEN_REWRITE = [
    "rhwp-codex",
    "rhwp-agent-surface",
]

PEER_ON_DEVEL = [
    "rhwp-cli",
    "rhwp-codex",
    "rhwp-contributor",
    "rhwp-doc-triage",
    "rhwp-exam-ingest",
    "rhwp-form-fill",
    "rhwp-mcp-session",
    "rhwp-onboarding",
    "rhwp-provenance",
    "rhwp-safe-edit",
    "rhwp-security-sweep",
    "rhwp-table-exchange",
    "rhwp-visual-regression",
    "rhwp-work-receipt",
    "rhwp-bulk-pipeline",
]

REQUIRED_REFS = [
    "00_first_read.md",
    "01_remeasure.md",
    "02_tree.md",
    "03_request_map.md",
    "04_boundary.md",
    "05_envelope_dict.md",
    "06_canonicals.md",
    "07_section_index.md",
    "08_jump_to_skill.md",
    "09_exceptions.md",
    "10_stale_last_verified.md",
    "11_version_mismatch.md",
    "12_map_vs_canonical.md",
    "13_stop_conditions.md",
    "14_handoff.md",
    "15_pitfalls.md",
    "16_journeys.md",
    "17_intent_matrix.md",
    "18_field_lookup.md",
    "19_three_questions.md",
    "20_samples_index.md",
    "21_contract_tests_index.md",
    "22_mcp_remeasure.md",
    "23_transcripts.md",
    "24_decision_table.md",
    "25_sibling_boundary.md",
    "README.md",
]

INVENTED_COMMANDS = [
    "rhwp knowledge-map",
    "rhwp knowledge_map",
    "rhwp map",
    "rhwp docs-index",
    "rhwp agent-map",
    "rhwp field-dict",
    "rhwp remap",
    "rhwp lookup-field",
    "rhwp open-map",
    "rhwp first-read",
]

INVENTED_FIELDS = [
    "schema_version",
    "untrusted_content",
    "page_count",
    "replaced_count",
    "is_error",
]


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_kmap_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentKnowledgeMapSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.skill = read(SKILL / "SKILL.md")
        cls.map = read(MAP)
        cls.llms = read(LLMS)
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.tree = load_json(FIXT, "tree.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.exceptions = load_json(FIXT, "exceptions.json")
        cls.verified = load_json(FIXT, "last_verified.json")
        cls.req = load_json(FIXT, "request_map.json")
        cls.honesty = load_json(FIXT, "honesty.json")
        cls.first = load_json(FIXT, "first_read.json")
        cls.remeasure = load_json(FIXT, "remeasure.json")
        cls.fields = load_json(FIXT, "envelope_fields.json")
        cls.mismatch = load_json(FIXT, "version_mismatch.json")
        cls.jumps = load_json(FIXT, "jump_skills.json")
        cls.trans = load_json(FIXT, "transcripts.json")
        cls.cards = load_json(FIXT, "section_cards.json")
        cls.working = read(WORKING)

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-knowledge-map", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)
        self.assertIn("진입점", self.skill)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "llms.txt",
            "agent_knowledge_map.md",
            "§2",
            "last_verified",
            "rhwp capabilities",
            "rhwp capabilities --mcp",
            "tools/list",
            "rhwp-codex",
            "rhwp-agent-surface",
            "R04",
            "R05",
            "R06",
            "R08",
            "R09",
            "canonical",
            "재서술",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_reference_docs_exist_and_long_enough(self):
        short = []
        for name in REQUIRED_REFS:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            if len(body) <= 200:
                short.append(f"{name}:{len(body)}")
        self.assertEqual(short, [], f"짧은 장: {short}")

    def test_examples_exist_and_point_at_canonical(self):
        for name in self.idx["examples"]:
            path = EXAMPLES / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertGreater(len(body), 80, name)
            self.assertIn("llms.txt", body + self.skill)

    def test_index_lists_same_references(self):
        listed = self.idx["references"]
        for name in REQUIRED_REFS:
            self.assertIn(name, listed, name)

    def test_not_gym_and_no_new_cli(self):
        self.assertTrue(self.idx["notGym"])
        self.assertTrue(self.idx["noNewCli"])
        self.assertTrue(self.idx["noNewEditLogic"])
        self.assertTrue(self.idx["routerOnly"])
        self.assertTrue(self.tree["notGym"])
        self.assertTrue(self.tree["noNewCli"])
        self.assertEqual(self.idx["issue"], 5342)
        self.assertEqual(self.tree["issue"], 5342)
        self.assertTrue(self.idx["doNotRenarrateMapRows"])
        self.assertTrue(self.idx["canonicalWins"])

    def test_first_read_order(self):
        order = self.first["order"]
        self.assertEqual(order[0]["path"], "llms.txt")
        self.assertEqual(order[1]["path"], "mydocs/manual/agent_knowledge_map.md")
        self.assertEqual(len(order), 3)
        self.assertIn("에이전트 지식 지도", self.llms)
        self.assertIn("agent_knowledge_map.md", self.llms)

    def test_remeasure_commands(self):
        cmds = {c["id"]: c for c in self.remeasure["commands"]}
        self.assertEqual(cmds["RM01"]["argv"], ["rhwp", "capabilities"])
        self.assertEqual(cmds["RM02"]["argv"], ["rhwp", "capabilities", "--mcp"])
        self.assertEqual(cmds["RM03"]["argv"], ["rhwp", "mcp-serve"])
        self.assertEqual(cmds["RM03"]["method"], "tools/list")
        blob = self.skill + read(REF / "01_remeasure.md")
        self.assertIn("rhwp capabilities --mcp", blob)
        self.assertIn("tools/list", blob)

    def test_peer_skills_exist_but_codex_not_rewritten(self):
        for slug in PEER_ON_DEVEL:
            peer = REPO / ".agents" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)
        self.assertIn("rhwp-codex", self.idx["forbiddenSkillsTouch"])
        self.assertIn("rhwp-agent-surface", self.idx["forbiddenSkillsTouch"])
        # 표면 스킬은 이 나무에 없을 수 있다. 만들지 않았는지만 본다.
        surface = REPO / ".agents" / "skills" / "rhwp-agent-surface" / "SKILL.md"
        self.assertFalse(surface.is_file())

    def test_skill_does_not_rewrite_codex_or_surface(self):
        text = self.skill + read(REF / "04_boundary.md") + read(REF / "25_sibling_boundary.md")
        self.assertIn("재작성", text)
        for slug in FORBIDDEN_REWRITE:
            self.assertIn(slug, text)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        for name in self.idx["examples"]:
            blobs.append(read(EXAMPLES / name))
        joined = "\n".join(blobs)
        for bad in INVENTED_COMMANDS:
            # 금지 목록에 백틱으로 인용하는 것은 허용. 호출 문장은 금지.
            call = re.search(rf"(?m)^(?:\$ )?{re.escape(bad)}\b", joined)
            self.assertIsNone(call, f"발명된 명령 호출: {bad}")

    def test_stop_rule_ids_in_skill_or_stop_chapter(self):
        stops_md = read(REF / "13_stop_conditions.md")
        blob = self.skill + stops_md
        for rule in self.stops["rules"]:
            self.assertIn(rule["id"], blob, f"정지 {rule['id']} 문서 누락")
        ids = {r["id"] for r in self.stops["rules"]}
        for need in ("R01", "R02", "R03", "R04", "R05", "R06", "R07", "R08", "R09"):
            self.assertIn(need, ids)

    def test_exception_paths_are_four(self):
        kinds = {p["kind"] for p in self.exceptions["paths"]}
        self.assertEqual(
            kinds,
            {
                "stale-last-verified",
                "binary-version-mismatch",
                "map-vs-canonical",
                "invented-field-name",
            },
        )

    def test_last_verified_fresh_and_simulated_stale(self):
        self.assertEqual(self.verified["staleDays"], 30)
        self.assertEqual(self.verified["asOf"], "2026-08-18")
        self.assertEqual(self.verified["lastVerified"], "2026-08-11")
        self.assertFalse(self.verified["stale"])
        sim = self.verified["simulatedStale"]
        self.assertTrue(sim["stale"])
        self.assertTrue(sim["doNotFillFromMemory"])
        self.assertGreater(sim["daysSince"], 30)

    def test_version_mismatch_binary_wins(self):
        self.assertEqual(self.mismatch["winner"], "binary")
        self.assertTrue(self.mismatch["mismatch"])
        self.assertIn("0.8.3", self.mismatch["mapBinary"])
        self.assertTrue(self.mismatch["packageVersion"])
        self.assertIn("rhwp capabilities", self.mismatch["remeasure"])

    def test_envelope_fields_extracted_from_map_only(self):
        names = self.fields["names"]
        self.assertGreaterEqual(len(names), 100)
        self.assertFalse(self.fields["invented"])
        self.assertFalse(self.fields["definitionsCopied"])
        self.assertEqual(self.fields["section"], "§2")
        for name in names:
            self.assertIn(f"`{name}`", self.map, f"지도에 없는 이름: {name}")
        for bad in INVENTED_FIELDS:
            self.assertNotIn(bad, names)

    def test_common_fields_present(self):
        names = set(self.fields["names"])
        for need in (
            "schemaVersion",
            "source",
            "untrustedContent",
            "untrustedFields",
            "filledCount",
            "replacedCount",
            "identical",
            "changedPages",
        ):
            self.assertIn(need, names, need)

    def test_intent_matrix_size_and_schema(self):
        rows = self.intents["intents"]
        self.assertGreaterEqual(len(rows), 80)
        self.assertEqual(self.intents["count"], len(rows))
        ids = set()
        for row in rows:
            self.assertRegex(row["id"], r"^I\d{3}$")
            self.assertTrue(row["utterance"])
            self.assertTrue(row["command"])
            self.assertTrue(row["reference"].endswith(".md"))
            self.assertRegex(row["stop"], r"^R\d{2}$")
            self.assertNotIn(row["id"], ids)
            ids.add(row["id"])
            for bad in INVENTED_COMMANDS:
                self.assertNotIn(bad, row["command"])

    def test_journeys_point_at_known_stops(self):
        known = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 40)
        for j in items:
            self.assertIn(j["stop"], known, j["id"])
            self.assertTrue(j["steps"])
            self.assertEqual(j["steps"][0], "llms.txt")

    def test_transcripts_are_excerpted_not_live(self):
        self.assertTrue(self.trans["excerptedFromCanonical"])
        self.assertFalse(self.trans["fabricatedLive"])
        self.assertGreaterEqual(self.trans["count"], 20)
        for item in self.trans["items"]:
            src = REPO / item["sourceFile"]
            self.assertTrue(src.is_file(), item["sourceFile"])
            self.assertTrue(item["excerpted"])
            self.assertFalse(item["fabricatedLive"])
            raw = item["raw"][:40]
            self.assertIn(raw, read(src), item["id"])

    def test_request_map_covers_entry_and_exceptions(self):
        stops = {row["stop"] for row in self.req["rows"]}
        for need in ("R01", "R02", "R04", "R05", "R06", "R08", "R09"):
            self.assertIn(need, stops, need)
        for row in self.req["rows"]:
            self.assertFalse(row["renarrate"])
            self.assertFalse(row["readWholeMap"])

    def test_jumps_stop_reading_the_map(self):
        skills = {j["skill"] for j in self.jumps["jumps"]}
        self.assertIn("rhwp-form-fill", skills)
        self.assertIn("rhwp-codex", skills)
        self.assertIn("rhwp-agent-surface", skills)
        for j in self.jumps["jumps"]:
            self.assertIn(j["stop"], {"R08", "R09"})

    def test_section_cards_do_not_copy_definitions(self):
        self.assertGreaterEqual(self.cards["count"], 20)
        self.assertTrue(self.cards["doNotRenarrate"])
        for card in self.cards["cards"]:
            self.assertTrue(card["id"])
            self.assertTrue(card["title"])
            self.assertTrue(card["canonical"])
            self.assertNotIn("한 줄 정의", json.dumps(card, ensure_ascii=False))

    def test_honesty_flags(self):
        self.assertTrue(self.honesty["doNotRenarrateMapRows"])
        self.assertTrue(self.honesty["doNotInventFieldNames"])
        self.assertTrue(self.honesty["canonicalWins"])
        self.assertTrue(self.honesty["binaryWins"])
        self.assertTrue(self.honesty["notGym"])
        self.assertTrue(self.honesty["noNewCli"])
        self.assertTrue(self.honesty["doNotRewriteCodex"])
        self.assertTrue(self.honesty["doNotRewriteSurface"])

    def test_generator_roundtrip_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5342)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        self.assertEqual(self.gen.STALE_DAYS, 30)
        self.assertEqual(self.gen.SKILL_NAME, "rhwp-knowledge-map")

    def test_core_reuse_is_llms_and_map(self):
        reuse = self.tree["coreReuse"]
        self.assertIn("llms.txt", reuse)
        self.assertIn("mydocs/manual/agent_knowledge_map.md", reuse)
        self.assertTrue(MAP.is_file())
        self.assertTrue(LLMS.is_file())
        self.assertIn("단일 진입점", self.map)

    def test_working_doc(self):
        self.assertIn("#5342", self.working)
        self.assertIn("rhwp-knowledge-map", self.working)
        self.assertIn("gym", self.working)
        self.assertIn("llms.txt", self.working)
        self.assertIn("canonical", self.working)

    def test_no_gym_tree_in_skill_dir(self):
        for path in SKILL.rglob("*"):
            rel = path.relative_to(SKILL).as_posix()
            self.assertFalse(rel.startswith("gym"), rel)
            self.assertNotIn("/gym/", f"/{rel}")

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_knowledge_map"
        self.assertFalse(shadow.exists(), "픽스처는 skill/fixtures 한 곳만")

    def test_map_not_rewritten(self):
        # 이 PR 은 정본 지도를 고치지 않는다.
        self.assertIn("kind: canonical", self.map[:200])
        self.assertIn("last_verified: 2026-08-11", self.map[:250])


if __name__ == "__main__":
    unittest.main()
