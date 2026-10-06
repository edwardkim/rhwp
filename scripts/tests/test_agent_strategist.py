"""[#5335] rhwp-strategist 스킬·픽스처 계약.

실 에이전트가 목표+코퍼스를 근거 좌표로 받칠 때 쓰는 규약이
기존 표면(engagement.py / search / extract-data / info)을 벗어나지
않는지, gym 과 새 CLI 와 FDE/Chief 층을 끌어들이지 않았는지를
바이너리 없이 커밋된 파일만으로 검사한다.
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-strategist"
REF = SKILL / "references"
FIX = SKILL / "fixtures"
EX = SKILL / "examples"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_strategist.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"
AGENT = REPO / ".claude" / "agents" / "rhwp-strategist.md"
ENGINE = REPO / "tools" / "strategist" / "engagement.py"

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-form-fill",
    "rhwp-work-receipt",
]

REQUIRED_REFS = [
    "00_tree.md",
    "01_playbook_authority.md",
    "02_engagement_protocol.md",
    "03_corpus_map.md",
    "04_evidence_ledger.md",
    "05_claim_gate.md",
    "06_coordinate_rules.md",
    "07_search_extract_envelopes.md",
    "08_validate_exit.md",
    "09_out_of_scope.md",
    "10_fde_chief_boundary.md",
    "11_sws_audit.md",
    "12_pitfalls.md",
    "13_decision_tree.md",
    "14_recipe_index.md",
    "15_envelope_field_catalog.md",
    "16_journeys.md",
    "17_stop_rules.md",
    "18_handoff.md",
    "19_failed_document_ledger.md",
    "20_question_design.md",
    "README.md",
]

INVENTED_COMMANDS = [
    "rhwp strategy",
    "rhwp claim-check",
    "rhwp forecast",
    "rhwp evidence-ledger",
    "rhwp claim-gate",
    "edit strategy",
]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentStrategistContract(unittest.TestCase):
    def test_skill_frontmatter_and_engine(self):
        text = read(SKILL / "SKILL.md")
        self.assertTrue(text.startswith("---\n"))
        self.assertIn("name: rhwp-strategist", text)
        self.assertIn("tools/strategist/engagement.py", text)
        self.assertIn("engagement.json", text)
        self.assertIn("gym", text)
        self.assertTrue(
            "엔진은 전략을 만들지 않는다" in text or "엔진은 전략을 발명하지 않는다" in text
        )

    def test_required_references_exist_and_are_substantial(self):
        for name in REQUIRED_REFS:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertGreater(len(body), 200, f"{name} 가 너무 짧다")

    def test_no_invented_cli_in_skill_tree(self):
        paths = [SKILL / "SKILL.md", *REF.glob("*.md"), *EX.glob("*.md")]
        allow = ("금지", "발명", "만들지", "치지 않는다", "추측 실행하지")
        for path in paths:
            text = read(path)
            for cmd in INVENTED_COMMANDS:
                for match in re.finditer(re.escape(cmd), text):
                    ctx = text[max(0, match.start() - 160) : match.end() + 24]
                    if any(token in ctx for token in allow):
                        continue
                    self.fail(f"{path.name} 에 발명 명령 호출: {cmd} ({ctx!r})")

    def test_neighbor_skills_untouched_exist(self):
        for slug in FORBIDDEN_SKILLS:
            path = REPO / ".claude" / "skills" / slug / "SKILL.md"
            self.assertTrue(path.is_file(), slug)

    def test_engine_copy_coords_contract(self):
        src = read(ENGINE)
        self.assertIn("COORD_KEYS", src)
        self.assertIn("copy_coords", src)
        self.assertIn("if k in src", src)
        self.assertIn("status", src)
        self.assertIn("failed", src)
        self.assertIn("validate_spec", src)
        self.assertIn("unknown-evidence", src)
        self.assertIn("placeholder", src)
        self.assertIn("unlinked", src)

    def test_missing_page_fixture(self):
        env = load_json(FIX / "envelopes", "search_missing_page.json")
        self.assertNotIn("page", env["matches"][0])
        ledger = load_json(FIX / "ledgers", "gov_rfp_missing_page.json")
        ev2 = next(e for e in ledger["entries"] if e["id"] == "EV-2")
        self.assertNotIn("page", ev2)

    def test_failed_docs_not_dropped(self):
        cmap = load_json(FIX / "corpus_maps", "mixed_failed.json")
        self.assertEqual(cmap["documentCount"], len(cmap["documents"]))
        failed = [d for d in cmap["documents"] if d["status"] == "failed"]
        self.assertGreaterEqual(len(failed), 2)
        self.assertEqual(
            cmap["mappedCount"],
            sum(1 for d in cmap["documents"] if d["status"] == "ok"),
        )

    def test_gate_kinds(self):
        kinds = set()
        for name in (
            "placeholder.json",
            "unknown_evidence.json",
            "unlinked.json",
        ):
            v = load_json(FIX / "validate", name)
            self.assertEqual(v["verdict"], "fail")
            self.assertEqual(v["_skillMeta"]["exit"], 3)
            kinds.add(v["violations"][0]["kind"])
        self.assertEqual(kinds, {"placeholder", "unknown-evidence", "unlinked"})

    def test_working_and_registry_and_agent(self):
        working = read(WORKING)
        self.assertRegex(working, r"5335")
        self.assertIn("gym", working)
        self.assertIn("engagement.py", working)
        reg = read(REGISTRY)
        self.assertIn("CAP-4903", reg)
        self.assertIn("skills/rhwp-strategist/SKILL.md", reg)
        agent = read(AGENT)
        self.assertIn("skills/rhwp-strategist/SKILL.md", agent)

    def test_examples_cover_forecast_and_gate(self):
        names = {p.name for p in EX.glob("*.md")}
        self.assertIn("09_no_market_forecast.md", names)
        self.assertIn("05_section5_gate_pass.md", names)
        self.assertIn("03_failed_doc_stays.md", names)
        self.assertIn("04_missing_page_omitted.md", names)
        forecast = read(EX / "09_no_market_forecast.md")
        self.assertIn("ST-FORECAST", forecast)

    def test_tree_and_catalog_issue(self):
        tree = load_json(FIX, "tree.json")
        self.assertEqual(tree["issue"], 5335)
        self.assertTrue(tree["notGym"])
        self.assertTrue(tree["noNewCli"])
        self.assertTrue(tree["neverInventMissingPage"])
        cat = load_json(FIX, "catalog.json")
        self.assertEqual(cat["skill"], "rhwp-strategist")
        self.assertNotRegex(
            json.dumps(cat, ensure_ascii=False),
            r"rhwp (strategy|forecast|claim-check)",
        )

    def test_engagement_schema_tokens(self):
        eng = load_json(FIX / "engagements", "gov_rfp.json")
        self.assertIn("objective", eng)
        self.assertIn("corpus", eng)
        self.assertIn("questions", eng)
        self.assertTrue(eng["questions"])

    def test_no_gym_path_as_authority(self):
        skill = read(SKILL / "SKILL.md")
        self.assertNotRegex(skill, r"gym/packs")
        self.assertNotRegex(skill, r"gym/baselines")


if __name__ == "__main__":
    unittest.main()
