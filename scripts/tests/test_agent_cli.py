"""[#5316] rhwp-cli 스킬 고도화 계약.

실사용 에이전트가 기존 export/dump/ir-diff/hwp5-* 만으로
HWP/HWPX 를 분석·디버깅하도록 문서·픽스처가 같은 단어를 쓰는지
파일만으로 고정한다.

새 CLI 를 시험하지 않는다. gym/ 을 열지 않는다.
다른 에이전트 스킬 본문을 요구하거나 바꾸지 않는다. 바이너리·네트워크를 부르지 않는다.

정본: .agents/skills/rhwp-cli/
작업 기록: mydocs/working/archives/agent_cli.md
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "rhwp-cli"
SKILL_MD = SKILL / "SKILL.md"
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_cli.md"

DEBUG_ORDER = (
    "export-svg",
    "dump-pages",
    "dump",
    "ir-diff",
    "export-render-tree",
    "hwp5-inventory-diff",
)

CORE = (
    "export-svg",
    "export-png",
    "export-pdf",
    "export-text",
    "export-markdown",
    "dump-pages",
    "dump",
    "dump-records",
    "diag",
    "info",
    "export-render-tree",
    "ir-diff",
    "thumbnail",
    "convert",
    "hwp5-inventory-diff",
)

SIBLING_SKILLS = (
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-safe-edit",
    "rhwp-security-sweep",
    "rhwp-work-receipt",
    "rhwp-form-fill",
    "rhwp-table-exchange",
    "rhwp-visual-regression",
)


def load_json(rel: str):
    path = SKILL / rel
    return json.loads(path.read_text(encoding="utf-8"))


class AgentCliSkillTests(unittest.TestCase):
    def test_frontmatter_and_tokens(self):
        text = SKILL_MD.read_text(encoding="utf-8")
        self.assertTrue(text.startswith("---\n"))
        self.assertIn("name: rhwp-cli", text)
        for token in CORE + ("--debug-overlay", "HWPUNIT", "oracle", "generated", "gym", "새 CLI"):
            self.assertIn(token, text, token)

    def test_references_exist_and_are_real(self):
        idx = load_json("fixtures/skill_index.json")
        self.assertEqual(idx["issue"], 5316)
        self.assertFalse(idx["gym"])
        self.assertFalse(idx["newCli"])
        refs = idx["references"]
        self.assertGreaterEqual(len(refs), 29)
        skill = SKILL_MD.read_text(encoding="utf-8")
        for name in refs:
            path = REFS / name
            self.assertTrue(path.is_file(), name)
            body = path.read_text(encoding="utf-8")
            self.assertGreater(len(body), 400, name)
            self.assertIn(name, skill)

    def test_examples_contain_rhwp(self):
        idx = load_json("fixtures/skill_index.json")
        for name in idx["examples"]:
            body = (EXAMPLES / name).read_text(encoding="utf-8")
            self.assertIn("rhwp", body)

    def test_debug_order(self):
        doc = load_json("fixtures/debug_order.json")
        names = [s["command"] for s in doc["order"]]
        self.assertEqual(tuple(names), DEBUG_ORDER)

    def test_page_units(self):
        units = load_json("fixtures/page_units.json")
        self.assertEqual(units["inch_hwpunit"], 7200)
        self.assertEqual(units["px_hwpunit"], 75)
        self.assertTrue(units["pageZeroBased"])

    def test_exception_envelopes(self):
        missing = load_json("fixtures/envelopes/missing_file.json")
        self.assertEqual(missing["exitCode"], 1)
        page = load_json("fixtures/envelopes/bad_page_index.json")
        self.assertEqual(page["exitCode"], 2)
        skia = load_json("fixtures/envelopes/native_skia_missing.json")
        self.assertEqual(skia["exitCode"], 2)
        load = load_json("fixtures/envelopes/load_fail.json")
        self.assertEqual(load["exitCode"], 1)

    def test_oracle_generated_order(self):
        fam = load_json("fixtures/hwp5_family.json")
        self.assertEqual(fam["argumentOrder"], ["oracle", "generated"])

    def test_scenarios_are_existing_cli(self):
        cat = load_json("fixtures/scenario_catalog.json")
        self.assertGreaterEqual(cat["count"], 100)
        for s in cat["scenarios"]:
            self.assertFalse(s["newCli"])
            self.assertFalse(s["gym"])
            self.assertFalse(s["selfRoundTripIsHangul"])

    def test_working_doc(self):
        text = WORKING.read_text(encoding="utf-8")
        self.assertIn("5316", text)
        self.assertIn("gym", text)
        self.assertIn("새 CLI", text)

    def test_siblings_still_present(self):
        for slug in SIBLING_SKILLS:
            peer = REPO / ".agents" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_no_new_bin(self):
        cargo = (REPO / "Cargo.toml").read_text(encoding="utf-8")
        self.assertEqual(cargo.count("[[bin]]"), 2)

    def test_roundtrip_not_hangul(self):
        idx = load_json("fixtures/skill_index.json")
        self.assertTrue(idx["selfRoundTripIsNotHangul"])
        text = (REFS / "19_roundtrip_vs_hangul.md").read_text(encoding="utf-8")
        self.assertIn("한컴", text)


if __name__ == "__main__":
    unittest.main()
