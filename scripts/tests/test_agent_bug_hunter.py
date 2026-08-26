"""[#5324] bug-hunter 스킬·픽스처 계약.

실 에이전트가 playbook 여정을 정답지와 대조할 때 쓰는 규약이
기존 CLI 표면과 tools/fidelity_compare 를 벗어나지 않는지,
gym 과 새 CLI 와 두 번째 루브릭을 끌어들이지 않았는지를
바이너리 없이 커밋된 파일만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "bug-hunter"
REF = SKILL / "references"
EX = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = REF / "_gen_pack.py"
POINTER = REPO / ".claude" / "skills" / "rhwp-bug-hunter" / "SKILL.md"
WORKING = REPO / "mydocs" / "working" / "agent_bug_hunter.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"
PLAYBOOK = REPO / "mydocs" / "manual" / "bug_hunting_playbook.md"

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-form-fill",
    "rhwp-visual-regression",
]

REQUIRED_REFS = [
    "00_tree.md",
    "01_playbook_authority.md",
    "02_judgment_traps.md",
    "03_journey_selection.md",
    "04_ground_truth.md",
    "05_hangul_pdf_provenance.md",
    "06_self_consistency_limit.md",
    "07_run_to_final.md",
    "08_pixel_visual.md",
    "09_text_multiset.md",
    "10_reread_values.md",
    "11_exit_json_contract.md",
    "12_fidelity_compare.md",
    "13_issue_template.md",
    "14_no_filing.md",
    "15_utf8_console.md",
    "16_pitfalls.md",
    "17_journeys.md",
    "18_worked_traces.md",
    "19_intent_matrix.md",
    "20_classification.md",
    "21_handoff.md",
    "22_failure_signals.md",
    "23_gate_recipes.md",
    "24_existing_cli.md",
    "README.md",
]

REQUIRED_EXAMPLES = [
    "01_kstartup_form.md",
    "02_hangul_pdf_compare.md",
    "03_gianmun_legal.md",
    "04_roundtrip_ir.md",
    "05_cli_contract.md",
    "06_float_margin_leet.md",
    "07_seoul_hwpx_zip.md",
    "08_full_table_fill.md",
    "09_rag_citation.md",
    "10_batch_archive.md",
    "11_official_notice.md",
    "12_pii_mask.md",
    "13_edit_format_preserve.md",
    "14_no_baseline.md",
    "15_console_encoding.md",
    "16_oracle_pass_not_lossless.md",
    "17_check_devel_first.md",
    "18_dont_generalize.md",
    "19_reject_hypothesis.md",
    "20_issue_from_finding.md",
    "README.md",
]

INVENTED_COMMANDS = [
    "bug-hunt",
    "oracle-check",
    "fidelity-diff",
    "ground-truth",
    "hunt-bugs",
    "compare-oracle",
    "text-multiset",
    "gym-hunt",
]


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_bug_hunter_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentBugHunterSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.skill = read(SKILL / "SKILL.md")
        cls.pointer = read(POINTER)
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.tree = load_json(FIXT, "tree.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.klass = load_json(FIXT, "classification.json")
        cls.issues = load_json(FIXT, "issue_templates.json")
        cls.prov = load_json(FIXT, "provenance_keys.json")
        cls.env = load_json(FIXT, "envelope_keys.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: bug-hunter", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)
        self.assertIn("버그픽스", self.skill)

    def test_playbook_is_only_rubric(self):
        self.assertTrue(PLAYBOOK.is_file())
        self.assertIn("bug_hunting_playbook.md", self.skill)
        self.assertIn("별도의 판정 기준", self.skill)
        self.assertIn("두 번째 루브릭", self.skill)
        self.assertTrue(self.idx["secondRubricForbidden"])
        self.assertTrue(self.tree["secondRubricForbidden"])
        self.assertEqual(self.idx["authority"][0], "mydocs/manual/bug_hunting_playbook.md")
        auth = read(REF / "01_playbook_authority.md")
        self.assertIn("유일한 권위", auth)
        self.assertIn("두 번째 루브릭", auth)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "fidelity_compare",
            "render-diff",
            "export-svg",
            "소실",
            "과잉",
            "치환",
            "reference_only",
            "svg_only",
            "파일:라인",
            "UTF-8",
            "실명인증",
            "references/04_ground_truth.md",
            "references/09_text_multiset.md",
            "references/12_fidelity_compare.md",
            "references/13_issue_template.md",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_claude_pointer_is_thin(self):
        self.assertTrue(POINTER.is_file())
        self.assertIn("name: rhwp-bug-hunter", self.pointer)
        self.assertIn(".agents/skills/bug-hunter", self.pointer)
        self.assertIn("얇은 포인터", self.pointer)
        self.assertLess(len(self.pointer), 2500)
        self.assertNotIn("gym/", self.pointer)

    def test_reference_docs_exist_and_long_enough(self):
        for name in REQUIRED_REFS:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertGreater(len(body), 400, f"{name} 가 너무 짧다")

    def test_examples_exist_and_long_enough(self):
        for name in REQUIRED_EXAMPLES:
            path = EX / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertGreater(len(body), 200, f"{name} 가 너무 짧다")

    def test_index_lists_same_references(self):
        listed = self.idx["references"]
        for name in REQUIRED_REFS:
            self.assertIn(name, listed, name)

    def test_not_gym_and_no_new_cli(self):
        self.assertTrue(self.idx["notGym"])
        self.assertTrue(self.idx["noNewCli"])
        self.assertTrue(self.tree["notGym"])
        self.assertTrue(self.tree["noNewCli"])
        self.assertTrue(self.idx["huntingNotFix"])
        self.assertEqual(self.idx["issue"], 5324)
        self.assertEqual(self.tree["issue"], 5324)

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            peer = REPO / ".claude" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill, self.pointer]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        for name in REQUIRED_EXAMPLES:
            blobs.append(read(EX / name))
        joined = "\n".join(blobs)
        for bad in INVENTED_COMMANDS:
            # 금지 목록 인용은 허용. `rhwp <발명>` 호출만 금지.
            # bug-hunter 슬러그와 겹치지 않게 단어 경계를 본다.
            self.assertNotRegex(
                joined,
                rf"(?m)(?<![-\w])rhwp(?:\.exe)?\s+{bad}(?![-\w])",
                f"발명된 명령: {bad}",
            )

    def test_stop_rule_ids_in_skill_or_failure_chapter(self):
        fail = read(REF / "22_failure_signals.md")
        for rule in self.stops["rules"]:
            rid = rule["id"]
            self.assertTrue(
                rid in self.skill or rid in fail,
                f"정지 {rid} 문서 누락",
            )

    def test_ladder_order_documented(self):
        tree_md = read(REF / "00_tree.md")
        box = tree_md.find("살아 있는 동사는")
        self.assertGreaterEqual(box, 0)
        prev = box
        for cmd in ("`rhwp`", "fidelity_compare"):
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
            self.assertTrue(row["notGym"])
            self.assertNotIn(row["id"], ids)
            ids.add(row["id"])
            for bad in INVENTED_COMMANDS:
                self.assertNotRegex(
                    row["command"],
                    rf"(?<![-\w]){bad}(?![-\w])",
                    f"{row['id']} command has {bad}",
                )

    def test_journeys_point_at_known_stops_and_playbook(self):
        known = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 80)
        playbook_ids = {j["id"] for j in items if j.get("playbookExample")}
        self.assertGreaterEqual(len(playbook_ids), 7)
        for j in items:
            self.assertIn(j["stop"], known, j["id"])
            self.assertTrue(j["steps"])
            self.assertTrue(j["notGym"])

    def test_classification_missing_extra_both(self):
        self.assertEqual(self.klass["missing"], "loss")
        self.assertEqual(self.klass["extra"], "excess")
        self.assertEqual(self.klass["both"], "substitution")
        by_id = {c["id"]: c for c in self.klass["rules"]}
        self.assertEqual(by_id["C01"]["labelEn"], "loss")
        self.assertEqual(by_id["C02"]["labelEn"], "excess")
        self.assertEqual(by_id["C03"]["labelEn"], "substitution")
        self.assertTrue(by_id["C04"]["issueReady"])
        self.assertTrue(by_id["C05"]["issueReady"])
        self.assertFalse(by_id["C01"]["issueReady"])
        tsv = (FIXT / "tsv" / "classification.tsv").read_text(encoding="utf-8")
        self.assertIn("loss", tsv)
        self.assertIn("excess", tsv)
        self.assertIn("substitution", tsv)
        chap = read(REF / "20_classification.md")
        self.assertIn("소실", chap)
        self.assertIn("과잉", chap)
        self.assertIn("치환", chap)

    def test_issue_template_requires_three_fields(self):
        self.assertEqual(
            self.issues["requiredFields"],
            ["repro", "codePath", "groundTruth"],
        )
        ids = {f["id"] for f in self.issues["fields"]}
        for need in ("repro", "codePath", "groundTruth"):
            self.assertIn(need, ids)
        tmpl = read(FIXT / "issue_template.md")
        self.assertIn("재현 명령", tmpl)
        self.assertIn("파일:라인", tmpl)
        self.assertIn("정답지", tmpl)
        body = read(REF / "13_issue_template.md")
        self.assertIn("재현 명령", body)
        self.assertIn("파일:라인", body)

    def test_provenance_keys_cover_hangul_pdf(self):
        keys = set(self.prov["keys"])
        for need in (
            "tool",
            "version",
            "outputPath",
            "fonts",
            "sourcePath",
            "referencePdfPath",
            "creator",
            "producer",
        ):
            self.assertIn(need, keys)
        prov_md = read(REF / "05_hangul_pdf_provenance.md")
        self.assertIn("Hwp 2022", prov_md)
        self.assertIn("fonts", prov_md)

    def test_no_filing_and_utf8(self):
        no_file = read(REF / "14_no_filing.md")
        self.assertIn("실명인증", no_file)
        self.assertIn("가상", no_file)
        utf = read(REF / "15_utf8_console.md")
        self.assertIn("UTF-8", utf)
        self.assertIn("결함", utf)
        self.assertIn("F13", self.skill)
        self.assertIn("F10", self.skill)

    def test_self_consistency_is_not_hangul_fidelity(self):
        self.assertTrue(self.tree["aaIsNotHangulFidelity"])
        self.assertTrue(self.env["commands"]["render-diff"]["selfIsNotHangulFidelity"])
        self.assertTrue(self.env["commands"]["export-hwpx"]["verifyDoesNotProveZip"])
        limit = read(REF / "06_self_consistency_limit.md")
        self.assertIn("한계", limit)
        self.assertIn("render-diff", limit)

    def test_fidelity_compare_is_tool_not_cli(self):
        fid = read(REF / "12_fidelity_compare.md")
        self.assertIn("tools/fidelity_compare", fid)
        self.assertIn("--out-dir", fid)
        self.assertIn("--text-only", fid)
        self.assertIn("하위명령", fid)
        self.assertIn("fidelity_compare", " ".join(self.tree["coreReuse"]))

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_bug_hunter"
        self.assertFalse(shadow.exists(), "픽스처는 skill/fixtures 한 곳만. 복제 금지")

    def test_working_doc_and_capability(self):
        text = read(WORKING)
        self.assertIn("#5324", text)
        self.assertIn("bug-hunter", text)
        self.assertIn("gym", text)
        self.assertIn("playbook", text)
        reg = read(REGISTRY)
        self.assertIn("CAP-5324", reg)
        self.assertIn("rhwp-bug-hunter", reg)
        self.assertIn("CAP-3398", reg)

    def test_generator_roundtrip_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5324)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        idx = self.gen.skill_index()
        self.assertEqual(idx["skill"], "bug-hunter")
        self.assertGreaterEqual(len(idx["references"]), 16)
        self.assertEqual(self.gen.classification_label_contract() if hasattr(self.gen, "classification_label_contract") else self.klass["missing"], "loss")

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

    def test_core_reuse_is_existing_functions(self):
        reuse = self.tree["coreReuse"]
        joined = " ".join(reuse)
        self.assertIn("fidelity_compare", joined)
        self.assertIn("export-svg", joined)
        self.assertIn("render-diff", joined)

    def test_playbook_examples_are_named(self):
        titles = {j["id"]: j["title"] for j in self.journeys["journeys"]}
        self.assertIn("K-Startup", titles["J01"])
        self.assertIn("한컴", titles["J02"])
        self.assertIn("기안", titles["J03"])
        self.assertIn("서울", titles["J07"])


if __name__ == "__main__":
    unittest.main()
