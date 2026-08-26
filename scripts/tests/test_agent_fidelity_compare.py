"""[#5329] rhwp-fidelity-compare 스킬·픽스처 계약.

실 에이전트가 한컴 공식 PDF 와 rhwp export-svg 를 대조할 때 쓰는 규약이
기존 도구 표면(tools/fidelity_compare, export-svg)을 벗어나지 않는지,
gym 과 새 CLI 를 끌어들이지 않았는지를 바이너리 없이 커밋된 파일만으로
검사한다. Chrome·한컴 PDF 가 없어도 닫힌다.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-fidelity-compare"
REF = SKILL / "references"
EX = SKILL / "examples"
FIXT = SKILL / "fixtures"
WORKING = REPO / "mydocs" / "working" / "agent_fidelity_compare.md"
TOOL_README = REPO / "tools" / "fidelity_compare" / "README.md"
GOVERNANCE = (
    REPO / "mydocs" / "manual" / "verification" / "visual_verification_governance.md"
)

FORBIDDEN_SKILLS = [
    "rhwp-visual-regression",
    "rhwp-cli",
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-form-fill",
]

REQUIRED_REFS = [
    "00_tree.md",
    "01_when_to_use.md",
    "02_setup_venv.md",
    "03_windows.md",
    "04_page_sheets.md",
    "05_pixel_ranking.md",
    "06_text_report.md",
    "07_font_style.md",
    "08_local_face_aliases.md",
    "09_tofu.md",
    "10_font_path_dir.md",
    "11_provenance.md",
    "12_visual_verdict.md",
    "13_missing_chrome.md",
    "14_missing_venv.md",
    "15_page_count_mismatch.md",
    "16_encrypted_pdf.md",
    "17_tofu_harness.md",
    "18_registered_keys.md",
    "19_direct_pair.md",
    "20_outputs.md",
    "21_vs_visual_regression.md",
    "22_vs_bug_hunter.md",
    "23_journeys.md",
    "24_pitfalls.md",
    "25_worked_traces.md",
    "26_handoff.md",
    "27_exception_catalog.md",
    "README.md",
]

REQUIRED_EXAMPLES = [
    "01_when_independent_pdf.md",
    "02_when_only_render_diff.md",
    "03_venv_posix.md",
    "04_venv_windows.md",
    "05_text_only_plan.md",
    "06_pixel_rank_worst_first.md",
    "07_text_report_multiset.md",
    "08_font_style_aliases.md",
    "09_rhwp_font_path_dir.md",
    "10_provenance_record.md",
    "11_missing_chrome.md",
    "12_missing_venv.md",
    "13_page_count_mismatch.md",
    "14_encrypted_pdf.md",
    "15_tofu_harness.md",
    "16_direct_pair.md",
    "17_registered_korexam.md",
    "18_layout_ledger.md",
    "19_handoff_visual_regression.md",
    "20_handoff_bug_hunter.md",
    "21_break_system_packages.md",
    "22_maintainer_verdict.md",
    "README.md",
]

INVENTED_COMMANDS = [
    "fidelity-diff",
    "pdf-compare",
    "hangul-diff",
    "oracle-diff",
    "hancom-compare",
    "pixel-diff",
]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(name: str):
    return json.loads((FIXT / name).read_text(encoding="utf-8"))


class AgentFidelityCompareSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json("skill_index.json")
        cls.tree = load_json("tree.json")
        cls.stops = load_json("stop_rules.json")
        cls.journeys = load_json("journeys.json")
        cls.exc = load_json("exception_catalog.json")
        cls.prov = load_json("provenance_schema.json")
        cls.out = load_json("outputs.json")
        cls.fonts = load_json("font_aliases.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-fidelity-compare", self.skill)
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)
        self.assertIn("fidelity_compare", self.skill)
        self.assertIn("export-svg", self.skill)

    def test_windows_python_and_no_break_system_packages(self):
        self.assertIn(r"venv\Scripts\python.exe", self.skill)
        self.assertIn("--break-system-packages", self.skill)
        self.assertFalse(self.tree["breakSystemPackages"])
        self.assertEqual(self.tree["windowsPython"], r"venv\Scripts\python.exe")

    def test_required_references_exist(self):
        listed = set(self.idx["references"])
        for name in REQUIRED_REFS:
            self.assertIn(name, listed, name)
            path = REF / name
            self.assertTrue(path.is_file(), name)
            self.assertGreater(path.stat().st_size, 400, name)

    def test_required_examples_exist(self):
        listed = set(self.idx["examples"])
        for name in REQUIRED_EXAMPLES:
            self.assertIn(name, listed, name)
            self.assertTrue((EX / name).is_file(), name)

    def test_schema_issue_5329(self):
        for name in (
            "tree.json",
            "stop_rules.json",
            "journeys.json",
            "skill_index.json",
            "exception_catalog.json",
        ):
            data = load_json(name)
            self.assertEqual(data["schemaVersion"], "1.0", name)
            self.assertEqual(data["issue"], 5329, name)
            self.assertTrue(data["notGym"], name)
            self.assertTrue(data["noNewCli"], name)

    def test_journeys_use_stop_ids(self):
        ids = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 80)
        for j in items:
            self.assertIn(j["stop"], ids, j["id"])
            self.assertTrue(j["steps"])
            self.assertTrue(j["notGym"])

    def test_stop_ids_in_docs(self):
        catalog = read(REF / "27_exception_catalog.md")
        for rule in self.stops["rules"]:
            rid = rule["id"]
            self.assertTrue(
                rid in catalog or rid in self.skill,
                rid,
            )

    def test_forbidden_peers_exist_untouched_here(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            peer = REPO / ".claude" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_invented_commands_forbidden(self):
        forbidden = set(self.idx["inventedCommandsForbidden"])
        for cmd in INVENTED_COMMANDS:
            self.assertIn(cmd, forbidden)
            self.assertNotIn(f"rhwp {cmd}", self.skill)

    def test_forbidden_gym_tree(self):
        self.assertIn("gym/", self.idx["forbiddenTrees"])

    def test_text_report_header(self):
        header = (FIXT / "tsv" / "text_report_mixed.tsv").read_text(
            encoding="utf-8"
        ).splitlines()[0]
        self.assertEqual(
            header,
            "page\treference_only\tsvg_only\treference_only_chars\tsvg_only_chars\tnote",
        )
        self.assertEqual(
            self.out["textReportTsvHeader"],
            header,
        )

    def test_report_ranked_worst_first(self):
        lines = (FIXT / "tsv" / "report_ranked.tsv").read_text(
            encoding="utf-8"
        ).splitlines()
        self.assertEqual(lines[0], "page\tdiff%\tnote")
        scores = [float(line.split("\t")[1]) for line in lines[1:] if line]
        self.assertEqual(scores, sorted(scores, reverse=True))

    def test_exception_ids(self):
        ids = {e["id"] for e in self.exc["exceptions"]}
        for need in (
            "E-CHROME",
            "E-VENV",
            "E-PAGECOUNT",
            "E-ENCRYPT",
            "E-TOFU",
        ):
            self.assertIn(need, ids)

    def test_font_defaults(self):
        self.assertEqual(self.fonts["defaultExportFlag"], "--font-style")
        self.assertFalse(self.fonts["embedDefault"])
        self.assertEqual(self.fonts["envFontDir"], "RHWP_FONT_PATH_DIR")

    def test_provenance_required_fields(self):
        fields = set(self.prov["requiredFields"])
        for need in (
            "hangulTool",
            "hangulVersion",
            "exportPath",
            "fonts",
            "originalPath",
            "oraclePath",
        ):
            self.assertIn(need, fields)

    def test_working_doc(self):
        self.assertTrue(WORKING.is_file())
        text = read(WORKING)
        self.assertIn("#5329", text)
        self.assertIn("rhwp-fidelity-compare", text)

    def test_authorities_exist(self):
        self.assertTrue(TOOL_README.is_file())
        readme = read(TOOL_README)
        self.assertIn("--text-only", readme)
        self.assertIn("visual_verification_governance", readme)
        self.assertTrue(GOVERNANCE.is_file())
        gov = read(GOVERNANCE)
        self.assertIn("최종 시각 판정", gov)

    def test_core_reuse(self):
        reuse = " ".join(self.tree["coreReuse"])
        self.assertIn("fidelity_compare", reuse)
        self.assertIn("export-svg", reuse)

    def test_text_only_skips_chrome(self):
        self.assertTrue(self.tree["textOnlySkipsChrome"])
        self.assertTrue(self.tree["rankingIsCandidate"])
        self.assertTrue(self.tree["verdictIsMaintainer"])

    def test_no_gym_path_in_skill(self):
        self.assertNotRegex(self.skill, r"(?m)^gym/")


if __name__ == "__main__":
    unittest.main()
