"""[#5318] rhwp-codex 스킬·픽스처 계약.

실 에이전트가 대전으로 전 명령 표면을 항해할 때
기존 표면(mydocs/manual/agent_codex, tools/gen_agent_codex.py,
rhwp capabilities --search)을 벗어나지 않는지, gym 과 새 CLI 와
생성 장 수기 수정을 끌어들이지 않았는지를 바이너리 없이 검사한다.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-codex"
FIX = SKILL / "fixtures"
REF = SKILL / "references"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_codex_skill.md"
GEN = REPO / "tools" / "gen_agent_codex.py"

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
]

REQUIRED_REFS = [
    "00_covenants.md",
    "01_request_tree.md",
    "02_how_to_read.md",
    "03_regen_freshness.md",
    "04_capabilities_search.md",
    "05_boundary_knowledge_map.md",
    "06_chapter_85.md",
    "20_intent_matrix.md",
    "21_journeys.md",
]

INVENTED = [
    "codex-search",
    "agent-help",
    "do-what-i-mean",
    "edit mail-merge",
    "handbook-regen",
]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(name: str):
    return json.loads((FIX / name).read_text(encoding="utf-8"))


class AgentCodexSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json("skill_index.json")
        cls.tree = load_json("request_tree.json")
        cls.cov = load_json("covenants.json")
        cls.stops = load_json("stop_rules.json")
        cls.intents = load_json("intent_matrix.json")
        cls.journeys = load_json("journeys.json")
        cls.regen = load_json("regen.json")
        cls.boundary = load_json("boundary.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-codex", self.skill)
        self.assertIn("gym", self.skill)
        self.assertIn("새 CLI", self.skill)
        self.assertIn("generated:", self.skill)

    def test_four_covenants(self):
        names = [c["name"] for c in self.cov["covenants"]]
        self.assertEqual(names, ["판정=데이터", "결정론", "출처 표지", "원본 무훼손"])
        for token in names:
            self.assertIn(token, self.skill)

    def test_seven_branches(self):
        ids = [b["id"] for b in self.tree["branches"]]
        self.assertEqual(ids, ["파악", "수확", "편집", "변환", "검증", "보안", "대량"])
        for name in ids:
            self.assertIn(name, self.skill)

    def test_regen_exit_3_is_data(self):
        self.assertEqual(self.regen["staleExit"], 3)
        self.assertTrue(self.regen["staleIsData"])
        self.assertIn("--check", self.skill)
        text = read(GEN)
        self.assertIn("--check", text)

    def test_search_fallback(self):
        self.assertIn("capabilities --search", self.skill)
        qs = load_json("search_fallback.json")["queries"]
        self.assertGreaterEqual(len(qs), 40)
        for q in qs:
            self.assertEqual(q["argv"][:2], ["capabilities", "--search"])

    def test_boundary_section_22(self):
        self.assertIn("§2-2", self.skill)
        self.assertIn("2-2", self.boundary["envelopeFieldDictionary"])
        km = read(REPO / "mydocs" / "manual" / "agent_knowledge_map.md")
        self.assertIn("2-2", km)

    def test_chapter_85_developer_only(self):
        self.assertIn("개발자", self.skill)
        self.assertIn("개발자", read(REF / "06_chapter_85.md"))

    def test_references_exist(self):
        for name in REQUIRED_REFS:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            self.assertGreater(len(read(path)), 200, name)

    def test_not_gym_flags(self):
        self.assertTrue(self.idx["notGym"])
        self.assertTrue(self.idx["noNewCli"])
        self.assertTrue(self.idx["doNotHandEditGenerated"])
        self.assertEqual(self.idx["issue"], 5318)
        self.assertIn("gym/", self.idx["forbiddenTrees"])

    def test_forbidden_peers_exist(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            self.assertTrue((REPO / ".claude" / "skills" / slug / "SKILL.md").is_file(), slug)

    def test_no_invented_commands_in_skill(self):
        blobs = [self.skill]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        joined = "\n".join(blobs)
        for bad in INVENTED:
            self.assertNotIn(bad, joined, bad)

    def test_intents_and_journeys(self):
        rows = self.intents["intents"]
        self.assertGreaterEqual(len(rows), 80)
        known = {r["id"] for r in self.stops["rules"]}
        for j in self.journeys["journeys"]:
            self.assertIn(j["stop"], known)
            self.assertTrue(j["notGym"])

    def test_working_doc(self):
        self.assertTrue(WORKING.is_file())
        text = read(WORKING)
        self.assertIn("5318", text)
        self.assertIn("rhwp-codex", text)

    def test_generated_chapters_untouched_marker(self):
        never = self.regen["neverHandEdit"]
        self.assertIn("10_조회.md", never)
        self.assertIn("85_진단_프로브.md", never)
        self.assertIn("README.md", self.regen["mayHandEdit"])


if __name__ == "__main__":
    unittest.main()
