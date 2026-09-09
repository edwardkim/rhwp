"""[#5312] rhwp-visual-regression 스킬·픽스처 계약.

실 에이전트가 편집/변환 전후 레이아웃을 숫자로 판정할 때 쓰는 규약이
기존 CLI 표면(render-diff / ir-diff / thumbnail / export-png)을 벗어나지
않는지, gym 과 새 CLI 를 끌어들이지 않았는지를 바이너리 없이 커밋된
파일만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-visual-regression"
REF = SKILL / "references"
EX = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_visual_regression.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-form-fill",
]

REQUIRED_REFS = [
    "00_tree.md",
    "01_render_diff_self.md",
    "02_render_diff_two_file.md",
    "03_render_diff_batch.md",
    "04_struct_mismatch.md",
    "05_status_codes.md",
    "06_ir_diff.md",
    "07_thumbnail_vs_png.md",
    "08_determinism.md",
    "09_max_disp.md",
    "10_envelopes.md",
    "11_pitfalls.md",
    "12_journeys.md",
    "13_handoff.md",
    "14_failure_signals.md",
    "15_node_paths.md",
    "16_worked_traces.md",
    "17_intent_matrix.md",
    "18_tsv_schema.md",
    "19_gate_recipes.md",
    "20_exit_codes.md",
    "21_page_mismatch.md",
    "22_load_fail.md",
    "23_over_status.md",
    "24_export_render_tree.md",
    "README.md",
]

REQUIRED_EXAMPLES = [
    "01_self_roundtrip_form01.md",
    "02_self_roundtrip_via_hwp.md",
    "03_two_file_fill.md",
    "04_two_file_same_length.md",
    "05_aa_determinism.md",
    "06_batch_pass_folder.md",
    "07_batch_mixed_status.md",
    "08_struct_intended.md",
    "09_struct_unrelated.md",
    "10_page_mismatch.md",
    "11_load_fail.md",
    "12_over_threshold.md",
    "13_ir_diff_json.md",
    "14_thumbnail_stale.md",
    "15_export_png_rerender.md",
    "16_max_disp_struct_independent.md",
    "17_text_mode_exit1.md",
    "18_json_mode_exit3.md",
    "19_geom_inventory_gate.md",
    "20_warn_textrun.md",
    "README.md",
]

INVENTED_COMMANDS = [
    "visual-diff",
    "pixel-diff",
    "layout-diff",
    "screenshot-compare",
    "render-compare",
    "gym-render",
]

HARD_STATUSES = ["OVER", "STRUCT_MISMATCH", "PAGE_MISMATCH", "LOAD_FAIL"]
SOFT_STATUSES = ["PASS", "WARN_TEXTRUN"]


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_visual_regression_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentVisualRegressionSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.tree = load_json(FIXT, "tree.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.status = load_json(FIXT, "status_catalog.json")
        cls.env = load_json(FIXT, "envelope_keys.json")
        cls.det = load_json(FIXT, "determinism.json")
        cls.thumb = load_json(FIXT, "thumbnail_vs_png.json")
        cls.maxd = load_json(FIXT, "max_disp.json")
        cls.tsv = load_json(FIXT, "tsv_schema.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-visual-regression", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "render-diff",
            "ir-diff",
            "thumbnail",
            "export-png",
            "STRUCT_MISMATCH",
            "PAGE_MISMATCH",
            "LOAD_FAIL",
            "geom_inventory.tsv",
            "A==A",
            "1.0px",
            "references/01_render_diff_self.md",
            "references/04_struct_mismatch.md",
            "references/06_ir_diff.md",
            "references/07_thumbnail_vs_png.md",
            "references/08_determinism.md",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

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
        self.assertEqual(self.idx["issue"], 5312)
        self.assertEqual(self.tree["issue"], 5312)

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            peer = REPO / ".claude" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        for name in REQUIRED_EXAMPLES:
            blobs.append(read(EX / name))
        joined = "\n".join(blobs)
        for bad in INVENTED_COMMANDS:
            self.assertNotIn(bad, joined, f"발명된 명령: {bad}")

    def test_stop_rule_ids_in_skill_or_failure_chapter(self):
        fail = read(REF / "14_failure_signals.md")
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
        for cmd in ("`render-diff`", "`ir-diff`", "`thumbnail`", "`export-png`"):
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

    def test_status_catalog_exit_codes(self):
        self.assertEqual(self.status["defaultMaxDispPx"], 1.0)
        by_id = {s["id"]: s for s in self.status["statuses"]}
        for name in SOFT_STATUSES:
            self.assertFalse(by_id[name]["hard"])
            self.assertEqual(by_id[name]["textExit"], 0)
            self.assertEqual(by_id[name]["jsonExit"], 0)
        for name in ["OVER", "STRUCT_MISMATCH", "PAGE_MISMATCH"]:
            self.assertTrue(by_id[name]["hard"])
            self.assertEqual(by_id[name]["textExit"], 1)
            self.assertEqual(by_id[name]["jsonExit"], 3)
        self.assertEqual(by_id["LOAD_FAIL"]["jsonExit"], 1)

    def test_struct_mismatch_is_data_not_reflex_fail(self):
        struct = read(REF / "04_struct_mismatch.md")
        self.assertIn("반사", struct)
        self.assertIn("경로", struct)
        self.assertIn("임계", struct)
        self.assertIn("TextLine10", self.skill)

    def test_aa_must_pass(self):
        self.assertTrue(self.tree["aaMustPass"])
        self.assertEqual(self.det["expectedStatus"], "PASS")
        self.assertEqual(self.det["expectedMaxDisp"], 0.0)
        self.assertIn("A==A", self.skill)
        self.assertIn("A==A", read(REF / "08_determinism.md"))

    def test_thumbnail_is_not_rerender(self):
        self.assertFalse(self.thumb["thumbnail"]["rerender"])
        self.assertTrue(self.thumb["exportPng"]["rerender"])
        self.assertTrue(self.tree["thumbnailIsStoredPreview"])
        thumb_md = read(REF / "07_thumbnail_vs_png.md")
        self.assertIn("PrvImage", thumb_md)
        self.assertIn("재렌더", thumb_md)

    def test_max_disp_default_and_struct_independent(self):
        self.assertEqual(self.maxd["defaultPx"], 1.0)
        self.assertTrue(self.maxd["structMismatchIgnoresThreshold"])
        self.assertTrue(self.tree["structIgnoresThreshold"])
        cases = {c["id"]: c for c in self.maxd["cases"]}
        self.assertEqual(cases["D03"]["status"], "STRUCT_MISMATCH")
        self.assertEqual(cases["D03"]["threshold"], 100.0)

    def test_tsv_columns_match_cli(self):
        cols = self.tsv["columns"]
        self.assertEqual(
            cols,
            [
                "sample",
                "status",
                "pages_a",
                "pages_b",
                "max_disp",
                "worst_page",
                "struct_pages",
                "over_pages",
                "elapsed_ms",
                "error",
                "struct_delta",
            ],
        )
        raw = (FIXT / "tsv" / "geom_inventory_pass.tsv").read_text(encoding="utf-8")
        header = raw.splitlines()[0].split("\t")
        self.assertEqual(header, cols)
        self.assertIn("PASS", raw)

    def test_envelope_keys_cover_core_commands(self):
        cmds = self.env["commands"]
        self.assertIn("render-diff-single", cmds)
        self.assertIn("render-diff-batch", cmds)
        self.assertIn("ir-diff", cmds)
        self.assertEqual(cmds["ir-diff"]["diffExit"], 3)
        self.assertEqual(cmds["ir-diff"]["textDiffExit"], 0)
        self.assertEqual(cmds["render-diff-single"]["hardExit"], 3)
        self.assertEqual(cmds["render-diff-single"]["textHardExit"], 1)

    def test_ir_diff_envelopes_obey_invariant(self):
        ident = load_json(FIXT / "envelopes", "ir_diff_identical.json")
        diff = load_json(FIXT / "envelopes", "ir_diff_different.json")
        self.assertTrue(ident["identical"])
        self.assertEqual(ident["diffCount"], 0)
        self.assertEqual(ident["categories"], {})
        self.assertFalse(diff["identical"])
        self.assertGreater(diff["diffCount"], 0)
        self.assertTrue(diff["categories"])

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_visual_regression"
        self.assertFalse(
            shadow.exists(),
            "픽스처는 skill/fixtures 한 곳만. 복제 금지",
        )

    def test_working_doc_and_capability(self):
        text = read(WORKING)
        self.assertIn("#5312", text)
        self.assertIn("rhwp-visual-regression", text)
        self.assertIn("gym", text)
        reg = read(REGISTRY)
        self.assertIn("CAP-5312", reg)
        self.assertIn("rhwp-visual-regression", reg)

    def test_generator_roundtrip_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5312)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        idx = self.gen.skill_index()
        self.assertEqual(idx["skill"], "rhwp-visual-regression")
        self.assertGreaterEqual(len(idx["references"]), 16)

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
        self.assertIn("render_geom_diff", joined)
        self.assertIn("ir-diff", joined)


if __name__ == "__main__":
    unittest.main()
