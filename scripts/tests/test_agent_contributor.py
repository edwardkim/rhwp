"""[#5322] rhwp-contributor 스킬 고도화 계약.

실사용 에이전트가 이슈→분석→브랜치→구현→게이트→영수증→문서→한국어 PR
순서를 공식 규약대로 닫는지 파일만으로 고정한다.

새 CLI 를 시험하지 않는다. gym/ 을 열지 않는다.
다른 에이전트 스킬 본문을 요구하거나 바꾸지 않는다. 바이너리·네트워크를 부르지 않는다.

정본: .agents/skills/rhwp-contributor/
작업 기록: mydocs/working/archives/agent_contributor.md
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "rhwp-contributor"
SKILL_MD = SKILL / "SKILL.md"
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
ENVS = FIXTURES / "envelopes"
CATALOG = FIXTURES / "catalog.json"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_contributor.md"

HARD_GATE = "cargo fmt --all -- --check"
STALE_FMT = "cargo fmt --check"

REFERENCE_NAMES = (
    "README.md",
    "procedure-order.md",
    "issue-first.md",
    "analyze-canonical.md",
    "branch-isolation.md",
    "isolation-worktree.md",
    "implement-scope.md",
    "staging-named-files.md",
    "fmt-hard-gate.md",
    "rustfmt-unix.md",
    "clippy-and-tests.md",
    "visual-evidence.md",
    "work-receipt-pointers.md",
    "working-doc.md",
    "korean-pr.md",
    "pr-template-checkboxes.md",
    "exceptions.md",
    "pitfalls.md",
    "decision-tree.md",
    "recipe-index.md",
    "command-field-catalog.md",
)

SIBLING_SKILLS = (
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-safe-edit",
    "rhwp-work-receipt",
)

REF_TOKENS = {
    "procedure-order.md": ("이슈", "분석", "브랜치", HARD_GATE, "closes #"),
    "issue-first.md": ("gh issue", "gh pr list", "DoD"),
    "analyze-canonical.md": ("AGENTS.md", "local_validation.md", "DocumentCore"),
    "branch-isolation.md": ("upstream/devel", "fetch", "devel"),
    "isolation-worktree.md": ("worktree", "rhwp-desk", "훔치"),
    "implement-scope.md": ("DocumentCore", "gym", "새"),
    "staging-named-files.md": ("git add -A", "git add --"),
    "fmt-hard-gate.md": (HARD_GATE, STALE_FMT, "낡은"),
    "rustfmt-unix.md": ("newline_style", "Unix", "autocrlf"),
    "clippy-and-tests.md": ("cargo clippy -- -D warnings", "cargo test"),
    "visual-evidence.md": ("SVG", "레이아웃", "한컴"),
    "work-receipt-pointers.md": ("replay --capsule", "audit", "lineage"),
    "working-doc.md": ("mydocs/working", "agent_contributor.md"),
    "korean-pr.md": ("--body-file", "closes #", "한국어"),
    "pr-template-checkboxes.md": ("첫", HARD_GATE),
    "exceptions.md": ("crates/", "autocrlf", "noci", "FAILURE"),
    "pitfalls.md": ("git add -A", STALE_FMT, "gym"),
    "decision-tree.md": ("PR 올려", HARD_GATE, "gym"),
    "recipe-index.md": ("01_issue_first.md", "09_fmt_all_check.md"),
    "command-field-catalog.md": ("hardGate", "noci", "replay"),
}

SKILL_TOKENS = (
    HARD_GATE,
    "cargo clippy -- -D warnings",
    "upstream/devel",
    "git add -A",
    "DocumentCore",
    "replay --capsule",
    "audit",
    "lineage",
    "mydocs/working",
    "closes #",
    "--body-file",
    "첫 체크박스",
    "newline_style",
    "Unix",
    "sparse",
    "autocrlf",
    "noci",
    "FAILURE",
    "gym",
    "새 CLI",
    "isolation",
    "references/fmt-hard-gate.md",
    "fixtures/catalog.json",
)

WORKING_TOKENS = (
    "#5322",
    "rhwp-contributor",
    HARD_GATE,
    "gym",
    "DocumentCore",
    "git add -A",
    "5000",
)

INVENTED = (
    "rhwp contribute",
    "rhwp pr-gate",
    "rhwp receipt",
)

ALLOWED_CMD = {
    "git",
    "gh",
    "cargo",
    "python",
    "node",
    "rhwp",
    "replay",
    "audit",
    "lineage",
    "read",
}


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path):
    return json.loads(read(path))


class SkillLayoutTests(unittest.TestCase):
    def test_skill_root_exists(self):
        self.assertTrue(SKILL.is_dir(), SKILL)
        self.assertTrue(SKILL_MD.is_file(), SKILL_MD)
        self.assertTrue(REFS.is_dir(), REFS)
        self.assertTrue(EXAMPLES.is_dir(), EXAMPLES)
        self.assertTrue(FIXTURES.is_dir(), FIXTURES)
        self.assertTrue(CATALOG.is_file(), CATALOG)
        self.assertTrue(WORKING.is_file(), WORKING)

    def test_references_present(self):
        names = sorted(p.name for p in REFS.glob("*.md"))
        self.assertEqual(sorted(REFERENCE_NAMES), names)

    def test_sibling_skills_exist_untouched_by_this_suite(self):
        for name in SIBLING_SKILLS:
            path = REPO / ".agents" / "skills" / name / "SKILL.md"
            self.assertTrue(path.is_file(), path)

    def test_does_not_live_under_gym(self):
        self.assertNotIn("gym", SKILL.parts)
        self.assertFalse((REPO / "gym" / "docs" / "agent_contributor.md").exists())


class FrontmatterTests(unittest.TestCase):
    def test_skill_frontmatter(self):
        body = read(SKILL_MD)
        self.assertTrue(body.startswith("---\n"), "frontmatter 시작")
        end = body.find("\n---\n", 4)
        self.assertGreater(end, 0, "frontmatter 종료")
        fm = body[4:end]
        self.assertIn("name: rhwp-contributor", fm)
        self.assertIn("description:", fm)

    def test_skill_tokens(self):
        body = read(SKILL_MD)
        for tok in SKILL_TOKENS:
            self.assertIn(tok, body, tok)

    def test_hard_gate_not_stale(self):
        body = read(SKILL_MD)
        self.assertIn(HARD_GATE, body)
        self.assertIn(STALE_FMT, body)
        self.assertRegex(body, r"낡은|아님|부족")

    def test_skill_does_not_invent_cli(self):
        body = read(SKILL_MD)
        self.assertIn("새 CLI", body)
        for bad in INVENTED:
            if bad in body:
                self.assertRegex(
                    body,
                    rf"(만들지 않|발명하지 않|없다|금지).{{0,80}}{re.escape(bad)}|{re.escape(bad)}.{{0,80}}(만들지 않|발명하지 않|없다|금지)",
                    f"{bad} 가 금지 안내 없이 등장",
                )


class ReferenceTokenTests(unittest.TestCase):
    def test_each_reference_has_contract_tokens(self):
        for name, tokens in REF_TOKENS.items():
            body = read(REFS / name)
            for tok in tokens:
                self.assertIn(tok, body, f"{name} 에 {tok!r} 없음")

    def test_fmt_doc_rejects_stale(self):
        body = read(REFS / "fmt-hard-gate.md")
        self.assertIn(HARD_GATE, body)
        self.assertIn("낡은", body)

    def test_work_receipt_is_pointer(self):
        body = read(REFS / "work-receipt-pointers.md")
        self.assertIn("replay --capsule", body)
        self.assertIn("audit", body)
        self.assertIn("lineage", body)
        self.assertRegex(body, r"다시 쓰지|복제하지")


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(CATALOG)

    def test_catalog_header(self):
        self.assertEqual(self.cat["catalogVersion"], "1.0")
        self.assertEqual(self.cat["skill"], "rhwp-contributor")
        self.assertEqual(self.cat["issue"], 5322)
        self.assertEqual(self.cat["hardGate"], HARD_GATE)
        self.assertEqual(self.cat["staleFmt"], STALE_FMT)
        self.assertTrue(self.cat["neverGitAddA"])
        self.assertTrue(self.cat["neverStealNamedWorktrees"])
        self.assertTrue(self.cat["neverInventDocumentCore"])
        self.assertTrue(self.cat["noNewCli"])
        self.assertEqual(self.cat["base"], "devel")
        self.assertEqual(self.cat["firstPrCheckbox"], HARD_GATE)

    def test_catalog_lists_match_files(self):
        for rel in self.cat["envelopes"]:
            self.assertTrue((ENVS / rel).is_file(), rel)
        for rel in self.cat["examples"]:
            self.assertTrue((EXAMPLES / rel).is_file(), rel)
        for rel in self.cat["references"]:
            self.assertTrue((REFS / rel).is_file(), rel)
        for rel in self.cat["transcripts"]:
            self.assertTrue((FIXTURES / "transcripts" / rel).is_file(), rel)
        for rel in self.cat["checklists"]:
            self.assertTrue((FIXTURES / "checklists" / rel).is_file(), rel)

    def test_no_stray_envelope_files(self):
        listed = set(self.cat["envelopes"])
        actual = {p.name for p in ENVS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_example_readme_lists_same_files(self):
        readme = read(EXAMPLES / "README.md")
        for rel in self.cat["examples"]:
            self.assertIn(rel, readme, rel)

    def test_forbidden_worktrees(self):
        listed = self.cat["forbiddenWorktrees"]
        blob = "\n".join(listed)
        self.assertIn(r"C:\Users\swsz9\rhwp", blob)
        self.assertIn("rhwp-desk", blob)
        self.assertIn("rhwp-handoff", blob)
        self.assertIn("rhwp-scaffold-final", blob)
        self.assertIn("rhwp-doc-repro", blob)


class EnvelopeTests(unittest.TestCase):
    def test_every_envelope_has_exit_meta(self):
        for path in ENVS.glob("*.json"):
            env = load_json(path)
            meta = env["_skillMeta"]
            self.assertIn(meta["exit"], (0, 1, 2, 3), path.name)
            self.assertIn(meta["command"], ALLOWED_CMD, path.name)
            self.assertEqual(meta["hardGate"], HARD_GATE, path.name)
            self.assertTrue(meta["staleFmtRejected"], path.name)

    def test_stale_fmt_rejected(self):
        env = load_json(ENVS / "fmt_stale_check_only.json")
        self.assertEqual(env["command"], STALE_FMT)
        self.assertFalse(env["acceptedAsGate"])
        self.assertEqual(env["mustUse"], HARD_GATE)
        self.assertEqual(env["_skillMeta"]["exit"], 2)

    def test_fmt_pass_uses_hard_gate(self):
        env = load_json(ENVS / "fmt_pass.json")
        self.assertEqual(env["command"], HARD_GATE)
        self.assertEqual(env["_skillMeta"]["exit"], 0)
        self.assertTrue(env["cratesPresent"])

    def test_noci_vs_failure(self):
        noci = load_json(ENVS / "ci_noci.json")
        self.assertEqual(noci["classification"], "noci")
        self.assertFalse(noci["isFailure"])
        fail = load_json(ENVS / "ci_failure.json")
        self.assertEqual(fail["classification"], "FAILURE")
        self.assertFalse(fail["isNoci"])
        self.assertEqual(fail["_skillMeta"]["exit"], 3)

    def test_git_add_a_rejected(self):
        env = load_json(ENVS / "git_add_a_rejected.json")
        self.assertEqual(env["command"], "git add -A")
        self.assertTrue(env["rejected"])


class ScenarioTests(unittest.TestCase):
    def test_count_and_commands(self):
        cat = load_json(FIXTURES / "scenario_catalog.json")
        self.assertGreaterEqual(cat["count"], 80)
        self.assertEqual(cat["count"], len(cat["scenarios"]))
        for sc in cat["scenarios"]:
            self.assertIn(sc["command"], ALLOWED_CMD, sc["id"])
            self.assertEqual(sc["hardGate"], HARD_GATE)

    def test_cards_match_catalog(self):
        cat = load_json(FIXTURES / "scenario_catalog.json")
        cards = FIXTURES / "scenario-cards"
        index = load_json(cards / "index.json")
        self.assertEqual(index["count"], cat["count"])
        for sc in cat["scenarios"]:
            path = cards / f"{sc['id']}.json"
            self.assertTrue(path.is_file(), path)
            card = load_json(path)
            self.assertEqual(card["hardGate"], HARD_GATE)
            self.assertIn("git add -A", card["never"])


class WorkingDocTests(unittest.TestCase):
    def test_tokens(self):
        text = read(WORKING)
        for tok in WORKING_TOKENS:
            self.assertIn(tok, text, tok)


class PrBodyTests(unittest.TestCase):
    def test_first_checkbox_is_fmt_gate(self):
        body = read(FIXTURES / "pr-bodies" / "closes_5322.md")
        self.assertIn("closes #5322", body)
        idx = body.find("- [")
        self.assertGreaterEqual(idx, 0)
        window = body[idx : idx + 80]
        self.assertIn(HARD_GATE, window)


class RustfmtPolicyTests(unittest.TestCase):
    def test_unix_newlines(self):
        text = read(REPO / "rustfmt.toml")
        self.assertIn('newline_style = "Unix"', text)


if __name__ == "__main__":
    unittest.main()
