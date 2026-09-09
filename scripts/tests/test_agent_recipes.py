"""[#5331] rhwp-recipes 스킬·픽스처 계약.

실 에이전트가 요청을 mydocs/manual/recipes/ 한 장으로 고를 때
기존 CLI 표면만 가리키는지, 07·08 을 발명하지 않는지, gym 과
이웃 스킬 재작성·새 CLI 를 끌어들이지 않았는지를
바이너리 없이 커밋된 파일만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-recipes"
REF = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_recipes.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"
RECIPES = REPO / "mydocs" / "manual" / "recipes"

FORBIDDEN_SKILLS = [
    "rhwp-form-fill",
    "rhwp-table-exchange",
    "rhwp-security-sweep",
    "rhwp-bulk-pipeline",
    "rhwp-visual-regression",
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
]

REQUIRED_REFS = [
    "00_tree.md",
    "01_request_map.md",
    "02_card_01.md",
    "03_card_02.md",
    "04_card_03.md",
    "05_card_04.md",
    "06_card_05.md",
    "07_card_06.md",
    "08_card_09.md",
    "09_card_10.md",
    "10_gap_07_08.md",
    "11_exceptions.md",
    "12_untrusted.md",
    "13_first_commands.md",
    "14_next_skills.md",
    "15_stop_conditions.md",
    "16_handoff.md",
    "17_pitfalls.md",
    "18_journeys.md",
    "19_intent_matrix.md",
    "20_stale_last_verified.md",
    "21_missing_recipe.md",
    "22_two_recipe_match.md",
    "23_transcripts.md",
    "24_decision_table.md",
    "README.md",
]

EXISTING = ("01", "02", "03", "04", "05", "06", "09", "10")
MISSING = ("07", "08")

INVENTED_COMMANDS = [
    "rhwp recipe",
    "rhwp recipes",
    "rhwp route",
    "rhwp playbook",
    "rhwp recommend-recipe",
    "recipe --pick",
    "rhwp pick-recipe",
    "rhwp recipe-router",
]

FIRST_COMMANDS = {
    "01": "rhwp fields <file> --json",
    "02": "rhwp export-tables <file> --json",
    "03": "rhwp edit redact <file> --dry-run",
    "04": "rhwp info <file> --json",
    "05": "rhwp fields <file> --json",
    "06": "rhwp render-diff <file> --via hwpx",
    "09": "rhwp batch info --json",
    "10": "rhwp inspect hidden-text <file> --json",
}

NEXT_SKILLS = {
    "01": "rhwp-form-fill",
    "02": "rhwp-table-exchange",
    "03": "rhwp-security-sweep",
    "04": "rhwp-doc-triage",
    "05": "rhwp-form-fill",
    "06": "rhwp-visual-regression",
    "09": "rhwp-bulk-pipeline",
    "10": "rhwp-security-sweep",
}


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_recipes_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentRecipesSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.tree = load_json(FIXT, "tree.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.cards = load_json(FIXT, "recipe_cards.json")
        cls.gap = load_json(FIXT, "gap_07_08.json")
        cls.exceptions = load_json(FIXT, "exceptions.json")
        cls.two = load_json(FIXT, "two_recipe_cases.json")
        cls.verified = load_json(FIXT, "last_verified.json")
        cls.req = load_json(FIXT, "request_map.json")
        cls.honesty = load_json(FIXT, "honesty.json")
        cls.trans_idx = load_json(FIXT, "transcripts_index.json")
        cls.traces_idx = load_json(FIXT, "traces_index.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-recipes", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)
        self.assertIn("라우터", self.skill)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "01",
            "02",
            "03",
            "04",
            "05",
            "06",
            "09",
            "10",
            "07·08",
            "rhwp fields <file> --json",
            "rhwp export-tables <file> --json",
            "rhwp edit redact <file> --dry-run",
            "rhwp info <file> --json",
            "rhwp render-diff <file> --via hwpx",
            "rhwp batch info --json",
            "rhwp inspect hidden-text <file> --json",
            "last_verified",
            "R05",
            "untrustedContent",
            "references/10_gap_07_08.md",
            "references/22_two_recipe_match.md",
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
        self.assertTrue(self.tree["routerOnly"])
        self.assertEqual(self.idx["issue"], 5331)
        self.assertEqual(self.tree["issue"], 5331)

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            peer = REPO / ".claude" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_skill_does_not_rewrite_peer_skill_bodies(self):
        text = self.skill + read(REF / "16_handoff.md")
        self.assertIn("재작성", text)
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, text)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        for name in self.idx["examples"]:
            blobs.append(read(EXAMPLES / name))
        joined = "\n".join(blobs)
        for bad in INVENTED_COMMANDS:
            self.assertNotIn(bad, joined, f"발명된 명령: {bad}")

    def test_stop_rule_ids_in_skill_or_stop_chapter(self):
        stops_md = read(REF / "15_stop_conditions.md")
        blob = self.skill + stops_md
        for rule in self.stops["rules"]:
            self.assertIn(rule["id"], blob, f"정지 {rule['id']} 문서 누락")

    def test_existing_recipe_files_match_cards(self):
        ids = [c["id"] for c in self.cards["cards"]]
        self.assertEqual(ids, list(EXISTING))
        for card in self.cards["cards"]:
            path = REPO / card["path"]
            self.assertTrue(path.is_file(), card["path"])
            self.assertEqual(card["firstCommand"], FIRST_COMMANDS[card["id"]])
            self.assertEqual(card["nextSkill"], NEXT_SKILLS[card["id"]])
            self.assertTrue(card["triggers"])
            self.assertTrue(card["stopWhen"])
            self.assertTrue(card["untrustedNote"])
            self.assertFalse(card["stale"])
            self.assertTrue(card["lastVerified"])

    def test_gap_07_08_honest(self):
        self.assertTrue(self.gap["doNotInvent"])
        missing = {row["id"]: row for row in self.gap["missing"]}
        for rid in MISSING:
            self.assertIn(rid, missing)
            self.assertFalse(missing[rid]["exists"])
            self.assertFalse(missing[rid]["invent"])
        self.assertFalse((RECIPES / "07_handoff.md").exists())
        self.assertFalse((RECIPES / "08_collaboration.md").exists())
        listed = [p.name for p in RECIPES.glob("0[78]_*.md")]
        self.assertEqual(listed, [])
        gap_md = read(REF / "10_gap_07_08.md")
        self.assertIn("#3905", gap_md)
        self.assertIn("만들지 않는다", gap_md)
        self.assertIn("07", self.skill)
        self.assertIn("08", self.skill)

    def test_exception_paths_are_three(self):
        kinds = {p["kind"] for p in self.exceptions["paths"]}
        self.assertEqual(
            kinds, {"missing-recipe", "stale-last-verified", "two-recipe-match"}
        )
        for path in self.exceptions["paths"]:
            self.assertFalse(path["inventMenu"], path["id"])

    def test_two_recipe_cases_have_two_existing_candidates(self):
        self.assertGreaterEqual(len(self.two["cases"]), 5)
        for case in self.two["cases"]:
            self.assertEqual(len(case["candidates"]), 2, case["id"])
            for rid in case["candidates"]:
                self.assertIn(rid, EXISTING)
            self.assertEqual(case["stop"], "R05")
            self.assertTrue(case["ask"])

    def test_last_verified_fresh_and_simulated_stale(self):
        self.assertTrue(self.verified["allExistingFresh"])
        self.assertEqual(self.verified["staleDays"], 30)
        self.assertEqual(self.verified["asOf"], "2026-08-18")
        sim = self.verified["simulatedStale"]
        self.assertTrue(sim["stale"])
        self.assertTrue(sim["doNotTreatAsRecipe"])
        self.assertGreater(sim["daysSince"], 30)

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

    def test_gap_intents_do_not_invent_commands(self):
        gaps = [r for r in self.intents["intents"] if r["stop"] == "R02"]
        self.assertGreaterEqual(len(gaps), 4)
        for row in gaps:
            self.assertIn("없음", row["command"])

    def test_journeys_point_at_known_stops(self):
        known = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 40)
        for j in items:
            self.assertIn(j["stop"], known, j["id"])
            self.assertTrue(j["steps"])
            self.assertTrue(j["notGym"])
            self.assertTrue(j["noNewCli"])

    def test_transcripts_are_excerpted_not_live(self):
        self.assertTrue(self.trans_idx["excerptedFromCanonical"])
        self.assertFalse(self.trans_idx["fabricatedLive"])
        self.assertGreaterEqual(self.trans_idx["count"], 40)
        for tid in self.trans_idx["ids"]:
            path = FIXT / "transcripts" / f"{tid}.json"
            self.assertTrue(path.is_file(), tid)
            obj = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(obj["id"], tid)
            self.assertTrue(obj["excerpted"])
            self.assertFalse(obj["fabricatedLive"])
            self.assertTrue(obj["sourceFile"].startswith("mydocs/manual/recipes/"))
            src = REPO / obj["sourceFile"]
            self.assertTrue(src.is_file(), obj["sourceFile"])
            # 원문이 정본에 실제로 있다 (발췌 계약)
            self.assertIn(obj["raw"][:40], read(src), tid)

    def test_traces_point_at_transcripts(self):
        ids = self.traces_idx["ids"]
        self.assertGreaterEqual(len(ids), 30)
        for tid in ids:
            path = FIXT / "traces" / f"{tid}.json"
            self.assertTrue(path.is_file(), tid)
            obj = json.loads(path.read_text(encoding="utf-8"))
            self.assertTrue(obj["usesExistingCommand"])
            self.assertTrue(obj["notGym"])
            self.assertFalse(obj["fabricatedLive"])
            tname = obj["transcript"]
            self.assertTrue((FIXT / "transcripts" / f"{tname}.json").is_file(), tid)

    def test_request_map_covers_existing_and_gap(self):
        recipes = {row["recipe"] for row in self.req["rows"]}
        for rid in EXISTING:
            self.assertIn(rid, recipes, rid)
        self.assertIn("07", recipes)
        self.assertIn("08", recipes)
        for row in self.req["rows"]:
            if row["exists"]:
                self.assertEqual(row["firstCommand"], FIRST_COMMANDS[row["recipe"]])
            else:
                self.assertIsNone(row["firstCommand"])
                self.assertIn(row["stop"], {"R02", "R03"})

    def test_honesty_flags(self):
        self.assertTrue(self.honesty["doNotInvent07"])
        self.assertTrue(self.honesty["doNotInvent08"])
        self.assertTrue(self.honesty["routerOnly"])
        self.assertIn("07·08", self.honesty["note"])

    def test_generator_roundtrip_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5331)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        self.assertEqual(tuple(self.gen.EXISTING_IDS), EXISTING)
        self.assertEqual(tuple(self.gen.MISSING_IDS), MISSING)
        self.assertEqual(self.gen.STALE_DAYS, 30)

    def test_core_reuse_is_canonical_recipes(self):
        reuse = self.tree["coreReuse"]
        for rid, name in (
            ("01", "01_fill_form_and_submit.md"),
            ("02", "02_table_csv_roundtrip.md"),
            ("09", "09_bulk_extract_convert.md"),
            ("10", "10_security_sweep_before_share.md"),
        ):
            self.assertTrue(any(name in p for p in reuse), rid)

    def test_working_doc_and_capability(self):
        text = read(WORKING)
        self.assertIn("#5331", text)
        self.assertIn("rhwp-recipes", text)
        self.assertIn("gym", text)
        self.assertIn("07", text)
        self.assertIn("08", text)
        reg = read(REGISTRY)
        self.assertIn("CAP-5331", reg)
        self.assertIn("rhwp-recipes", reg)

    def test_no_gym_tree_in_skill_dir(self):
        for path in SKILL.rglob("*"):
            rel = path.relative_to(SKILL).as_posix()
            self.assertFalse(rel.startswith("gym"), rel)
            self.assertNotIn("/gym/", f"/{rel}")

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_recipes"
        self.assertFalse(shadow.exists(), "픽스처는 skill/fixtures 한 곳만")


if __name__ == "__main__":
    unittest.main()
