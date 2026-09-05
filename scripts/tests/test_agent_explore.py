"""[#5313] rhwp-explore 스킬·픽스처 계약.

실 에이전트가 처음 보는 HWP/HWPX 에서 다음 명령을 고를 때
기존 CLI 표면(`explore --json` 과 메뉴가 가리키는 조회)을 벗어나지
않는지, gym 과 새 CLI / 편집 로직을 끌어들이지 않았는지를
바이너리 없이 커밋된 파일만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "rhwp-explore"
REF = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_explore.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-form-fill",
    "rhwp-security-sweep",
    "rhwp-doc-triage",
    "rhwp-table-exchange",
]

REQUIRED_REFS = [
    "00_three_axes.md",
    "01_first_move.md",
    "02_envelope.md",
    "03_menu_priority.md",
    "04_routing_table.md",
    "05_security_first.md",
    "06_honest_heuristic.md",
    "07_exceptions.md",
    "08_table_extract.md",
    "09_form_fill.md",
    "10_structure_outline.md",
    "11_chart_extract.md",
    "12_long_doc_digest.md",
    "13_note_structure.md",
    "14_triage_overview.md",
    "15_handoff.md",
    "16_pitfalls.md",
    "17_journeys.md",
    "18_worked_traces.md",
    "19_intent_matrix.md",
    "20_exit_codes.md",
    "21_command_templates.md",
    "22_confidence.md",
    "23_why_engine_counts.md",
    "README.md",
]

REQUIRED_EXAMPLES = [
    "01_first_unseen.md",
    "02_encrypted.md",
    "03_empty.md",
    "04_form_only.md",
    "05_table_report.md",
    "06_security_first.md",
    "07_long_law.md",
    "08_plain_memo.md",
    "09_chart_deck.md",
    "10_mixed_kitchen.md",
]

AFFORDANCES = [
    "table-extract",
    "form-fill",
    "structure-outline",
    "chart-extract",
    "security-sweep",
    "long-doc-digest",
    "note-structure",
    "triage-overview",
]

ENVELOPE_KEYS = [
    "schemaVersion",
    "source",
    "format",
    "pageCount",
    "encrypted",
    "affordanceCount",
    "menu",
    "note",
]

MENU_KEYS = ["affordance", "why", "command", "skill", "confidence"]

INVENTED_COMMANDS = [
    "rhwp suggest",
    "rhwp affordances",
    "rhwp next",
    "rhwp recommend",
    "rhwp what-can-i-do",
    "explore --rank",
    "explore --only",
    "explore --affordance",
    "explore --menu",
    "hwp_suggest",
    "edit explore",
]


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_explore_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentExploreSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.tree = load_json(FIXT, "tree.json")
        cls.env = load_json(FIXT, "envelope_keys.json")
        cls.routing = load_json(FIXT, "routing_table.json")
        cls.prior = load_json(FIXT, "priorities.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.exceptions = load_json(FIXT, "exceptions.json")
        cls.honesty = load_json(FIXT, "honesty.json")
        cls.scenarios = load_json(FIXT, "scenarios.json")
        cls.traces_idx = load_json(FIXT, "traces_index.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-explore", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "rhwp explore <파일> --json",
            "explain",
            "capabilities",
            "schemaVersion",
            "affordanceCount",
            "security-sweep",
            "table-extract",
            "form-fill",
            "structure-outline",
            "chart-extract",
            "long-doc-digest",
            "note-structure",
            "triage-overview",
            "untrustedContent:false",
            "references/00_three_axes.md",
            "references/05_security_first.md",
            "references/07_exceptions.md",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_always_starts_with_explore_json(self):
        self.assertIn("언제나 explore", self.skill)
        self.assertIn("rhwp explore <파일> --json", self.skill)
        self.assertEqual(self.tree["firstMove"], "rhwp explore <file> --json")
        self.assertEqual(self.idx["firstMove"], "rhwp explore <file> --json")
        first = read(REF / "01_first_move.md")
        self.assertIn("rhwp explore 문서.hwp --json", first)

    def test_three_axes_are_distinct(self):
        axes = read(REF / "00_three_axes.md")
        self.assertIn("`explain`", axes)
        self.assertIn("`capabilities`", axes)
        self.assertIn("`explore`", axes)
        self.assertEqual(self.tree["threeAxes"], ["explain", "capabilities", "explore"])
        self.assertIn("문서마다 다름", axes)

    def test_reference_docs_exist_and_long_enough(self):
        short = []
        for name in REQUIRED_REFS:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            if len(body) <= 400:
                short.append(f"{name}:{len(body)}")
        self.assertEqual(short, [], f"짧은 장: {short}")

    def test_examples_exist_and_point_at_explore(self):
        for name in REQUIRED_EXAMPLES:
            path = EXAMPLES / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertIn("rhwp explore", body)
            self.assertGreater(len(body), 400, name)

    def test_index_lists_same_references(self):
        listed = self.idx["references"]
        for name in REQUIRED_REFS:
            self.assertIn(name, listed, name)
        for name in REQUIRED_EXAMPLES:
            self.assertIn(name, self.idx["examples"], name)

    def test_not_gym_and_no_new_cli(self):
        self.assertTrue(self.idx["notGym"])
        self.assertTrue(self.idx["noNewCli"])
        self.assertTrue(self.idx["noNewEditLogic"])
        self.assertTrue(self.tree["notGym"])
        self.assertTrue(self.tree["noNewCli"])
        self.assertTrue(self.tree["noNewEditLogic"])
        self.assertEqual(self.idx["issue"], 5313)
        self.assertEqual(self.tree["issue"], 5313)

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            peer = REPO / ".agents" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_skill_does_not_rewrite_peer_skill_bodies(self):
        text = self.skill + read(REF / "15_handoff.md")
        self.assertIn("재작성", text)
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, text)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        for name in REQUIRED_EXAMPLES:
            blobs.append(read(EXAMPLES / name))
        joined = "\n".join(blobs)
        for bad in INVENTED_COMMANDS:
            self.assertNotIn(bad, joined, f"발명된 명령: {bad}")

    def test_stop_rule_ids_in_skill_or_exception_chapter(self):
        exc = read(REF / "07_exceptions.md")
        sec = read(REF / "05_security_first.md")
        first = read(REF / "01_first_move.md")
        blob = self.skill + exc + sec + first
        for rule in self.stops["rules"]:
            rid = rule["id"]
            self.assertIn(rid, blob, f"정지 {rid} 문서 누락")

    def test_envelope_keys_match_cli_contract(self):
        self.assertEqual(self.env["required"], ENVELOPE_KEYS)
        self.assertEqual(self.env["menuItem"], MENU_KEYS)
        self.assertFalse(self.env["untrustedContent"])
        self.assertIn("schemaVersion", self.skill)
        self.assertIn("affordanceCount", self.skill)
        note = self.honesty["note"]
        self.assertIn("정직한 휴리스틱", note)
        self.assertEqual(self.env["note"], note)

    def test_routing_table_has_eight_affordances(self):
        ids = [row["id"] for row in self.routing["affordances"]]
        for name in AFFORDANCES:
            self.assertIn(name, ids, name)
        self.assertEqual(len(ids), 8)
        by_id = {row["id"]: row for row in self.routing["affordances"]}
        self.assertEqual(by_id["form-fill"]["command"], "rhwp fields <file> --json")
        self.assertEqual(
            by_id["table-extract"]["command"], "rhwp export-tables <file> --json"
        )
        self.assertEqual(
            by_id["long-doc-digest"]["command"],
            "rhwp digest <file> --sections --json",
        )
        self.assertEqual(
            by_id["triage-overview"]["command"], "rhwp digest <file> --json"
        )

    def test_priority_order_is_security_first(self):
        order = self.prior["order"]
        self.assertEqual(order[0], "security-sweep")
        self.assertEqual(order[-1], "triage-overview")
        self.assertLess(
            self.prior["priority"]["table-extract"],
            self.prior["priority"]["form-fill"],
        )
        self.assertEqual(
            self.prior["contractSampleIds"],
            [
                "security-sweep",
                "form-fill",
                "table-extract",
                "chart-extract",
                "triage-overview",
            ],
        )

    def test_security_sweep_before_llm_in_docs(self):
        sec = read(REF / "05_security_first.md")
        self.assertIn("본문", sec)
        self.assertIn("LLM", sec)
        self.assertIn("X03", sec)
        self.assertIn("inspect injection", sec)
        self.assertIn("inspect hidden-text", self.skill)

    def test_honest_heuristic_flags(self):
        self.assertTrue(self.honesty["suggestionNotCompleteness"])
        self.assertTrue(self.honesty["whyIsEngineCounts"])
        self.assertFalse(self.honesty["untrustedContent"])
        heur = read(REF / "06_honest_heuristic.md")
        self.assertIn("제안이지 완전성", heur)
        self.assertIn("untrustedContent", heur)
        why = read(REF / "23_why_engine_counts.md")
        self.assertIn("엔진이 센", why)

    def test_exception_paths_cover_three_kinds(self):
        kinds = {p["kind"] for p in self.exceptions["paths"]}
        self.assertIn("encrypted", kinds)
        self.assertIn("empty", kinds)
        self.assertIn("no-special", kinds)
        for path in self.exceptions["paths"]:
            self.assertFalse(path["inventMenu"], path["id"])
        need_pw = next(p for p in self.exceptions["paths"] if p["id"] == "E01")
        self.assertEqual(need_pw["exit"], 2)
        self.assertEqual(need_pw["stdout"], "")
        missing = next(p for p in self.exceptions["paths"] if p["id"] == "E04")
        self.assertEqual(missing["exit"], 1)

    def test_intent_matrix_size_and_schema(self):
        rows = self.intents["intents"]
        self.assertGreaterEqual(len(rows), 60)
        self.assertEqual(self.intents["count"], len(rows))
        ids = set()
        for row in rows:
            self.assertRegex(row["id"], r"^I\d{3}$")
            self.assertTrue(row["utterance"])
            self.assertTrue(row["command"])
            self.assertTrue(row["reference"].endswith(".md"))
            self.assertRegex(row["stop"], r"^X\d{2}$")
            self.assertNotIn(row["id"], ids)
            ids.add(row["id"])
            for bad in INVENTED_COMMANDS:
                self.assertNotIn(bad, row["command"])

    def test_first_intents_start_at_explore(self):
        first = self.intents["intents"][0]
        self.assertIn("explore", first["command"])
        starters = [
            row
            for row in self.intents["intents"]
            if row["id"] in {"I001", "I002", "I003", "I004", "I005"}
        ]
        self.assertEqual(len(starters), 5)
        for row in starters:
            self.assertIn("explore", row["command"])

    def test_journeys_point_at_known_stops(self):
        known = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 40)
        for j in items:
            self.assertIn(j["stop"], known, j["id"])
            self.assertTrue(j["steps"])
            self.assertTrue(j["notGym"])
            self.assertTrue(j["noNewCli"])

    def test_scenarios_rebuild_from_build_menu(self):
        rows = self.scenarios["scenarios"]
        self.assertGreaterEqual(len(rows), 40)
        for row in rows:
            menu = self.gen.build_menu(row["facts"])
            ids = [m["affordance"] for m in menu]
            self.assertEqual(ids, row["menuIds"], row["id"])
            self.assertEqual(menu[0]["affordance"], row["first"], row["id"])
            self.assertEqual("triage-overview", ids[-1], row["id"])
            if "security-sweep" in ids:
                self.assertEqual(ids[0], "security-sweep", row["id"])

    def test_envelope_transcripts_match_generator(self):
        for row in self.scenarios["scenarios"]:
            path = FIXT / "envelopes" / f"{row['id']}.json"
            self.assertTrue(path.is_file(), row["id"])
            env = json.loads(path.read_text(encoding="utf-8"))
            for key in ENVELOPE_KEYS:
                self.assertIn(key, env, f"{row['id']} {key}")
            self.assertEqual(env["schemaVersion"], "1.0")
            self.assertFalse(env["untrustedContent"])
            self.assertEqual(env["untrustedFields"], [])
            self.assertEqual(env["affordanceCount"], len(env["menu"]))
            self.assertEqual(
                [m["affordance"] for m in env["menu"]], row["menuIds"]
            )
            rebuilt = self.gen.envelope(row["source"], row["facts"])
            self.assertEqual(env["menu"], rebuilt["menu"], row["id"])
            self.assertEqual(env["note"], self.gen.HONESTY_NOTE)
            for item in env["menu"]:
                for key in MENU_KEYS:
                    self.assertIn(key, item)
                self.assertIn("<file>", item["command"])

    def test_priority_contract_sample_s40(self):
        env = load_json(FIXT / "envelopes", "S40.json")
        ids = [m["affordance"] for m in env["menu"]]
        self.assertEqual(
            ids,
            [
                "security-sweep",
                "form-fill",
                "table-extract",
                "chart-extract",
                "triage-overview",
            ],
        )

    def test_plain_document_is_overview_only(self):
        env = load_json(FIXT / "envelopes", "S01.json")
        self.assertEqual(len(env["menu"]), 1)
        self.assertEqual(env["menu"][0]["affordance"], "triage-overview")
        self.assertEqual(env["menu"][0]["command"], "rhwp digest <file> --json")

    def test_hidden_text_uses_medium_and_hidden_command(self):
        env = load_json(FIXT / "envelopes", "S14.json")
        first = env["menu"][0]
        self.assertEqual(first["affordance"], "security-sweep")
        self.assertEqual(first["confidence"], "medium")
        self.assertIn("hidden-text", first["command"])

    def test_injection_beats_hidden_for_command(self):
        env = load_json(FIXT / "envelopes", "S15.json")
        first = env["menu"][0]
        self.assertIn("inspect injection", first["command"])
        self.assertEqual(first["confidence"], "high")

    def test_long_doc_threshold(self):
        nine = load_json(FIXT / "envelopes", "S23.json")
        ten = load_json(FIXT / "envelopes", "S11.json")
        forty = load_json(FIXT / "envelopes", "S12.json")
        self.assertNotIn(
            "long-doc-digest", [m["affordance"] for m in nine["menu"]]
        )
        long10 = next(m for m in ten["menu"] if m["affordance"] == "long-doc-digest")
        self.assertEqual(long10["confidence"], "medium")
        self.assertIn("--sections", long10["command"])
        long40 = next(m for m in forty["menu"] if m["affordance"] == "long-doc-digest")
        self.assertEqual(long40["confidence"], "high")

    def test_encrypted_overview_mentions_password(self):
        env = load_json(FIXT / "envelopes", "S17.json")
        self.assertTrue(env["encrypted"])
        overview = next(
            m for m in env["menu"] if m["affordance"] == "triage-overview"
        )
        self.assertIn("--password", overview["why"])

    def test_no_special_and_empty_loaded_are_not_failures(self):
        kinds = {row["kind"] for row in self.scenarios["scenarios"]}
        self.assertIn("no-special", kinds)
        self.assertIn("empty-loaded", kinds)
        self.assertIn("encrypted", kinds)
        empty = next(
            r for r in self.scenarios["scenarios"] if r["id"] == "S22"
        )
        self.assertEqual(empty["menuIds"], ["triage-overview"])
        self.assertEqual(empty["stop"], "X05")

    def test_traces_exist_and_point_at_envelopes(self):
        ids = self.traces_idx["ids"]
        self.assertGreaterEqual(len(ids), 30)
        for tid in ids:
            path = FIXT / "traces" / f"{tid}.json"
            self.assertTrue(path.is_file(), tid)
            obj = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(obj["id"], tid)
            self.assertTrue(obj["usesExistingCommand"])
            self.assertTrue(obj["notGym"])
            self.assertEqual(obj["argv"][0], "explore")
            self.assertIn("--json", obj["argv"])
            env_name = Path(obj["envelope"]).name
            self.assertTrue((FIXT / "envelopes" / env_name).is_file(), tid)

    def test_core_reuse_is_existing_functions(self):
        reuse = " ".join(self.tree["coreReuse"])
        self.assertIn("build_menu", reuse)
        self.assertIn("DocFacts", reuse)
        self.assertIn("extract_tables", reuse)
        self.assertIn("collect_all_fields", reuse)

    def test_generator_roundtrip_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5313)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        self.assertEqual(self.gen.LONG_DOC_PAGES, 10)
        idx = self.gen.skill_index()
        self.assertEqual(idx["skill"], "rhwp-explore")
        self.assertGreaterEqual(len(idx["references"]), 16)

    def test_build_menu_matches_rust_contract_cases(self):
        plain = self.gen.build_menu(self.gen.default_facts())
        self.assertEqual([m["affordance"] for m in plain], ["triage-overview"])
        mixed = self.gen.build_menu(
            self.gen.default_facts(
                field_count=1,
                table_count=1,
                chart_count=1,
                injection_signal_count=1,
            )
        )
        self.assertEqual(
            [m["affordance"] for m in mixed],
            [
                "security-sweep",
                "form-fill",
                "table-extract",
                "chart-extract",
                "triage-overview",
            ],
        )

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_explore"
        self.assertFalse(
            shadow.exists(),
            "픽스처는 스킬 fixtures/ 한 곳만. 복제 금지",
        )
        nested = REF / "fixtures"
        self.assertFalse(nested.exists(), "fixtures 는 references 아래가 아니다")

    def test_working_doc_and_capability(self):
        text = read(WORKING)
        self.assertIn("#5313", text)
        self.assertIn("rhwp-explore", text)
        self.assertIn("gym", text)
        self.assertIn("explore --json", text)
        reg = read(REGISTRY)
        self.assertIn("CAP-5313", reg)
        self.assertIn("rhwp-explore", reg)

    def test_no_gym_tree_in_skill_dir(self):
        for path in SKILL.rglob("*"):
            rel = path.relative_to(SKILL).as_posix()
            self.assertFalse(rel.startswith("gym"), rel)
            self.assertNotIn("/gym/", f"/{rel}")

    def test_allowed_commands_are_existing_surface(self):
        allowed = set(self.tree["allowedCommands"])
        self.assertIn("rhwp explore <file> --json", allowed)
        self.assertIn("rhwp inspect injection <file> --json", allowed)
        self.assertIn("rhwp fields <file> --json", allowed)
        for bad in INVENTED_COMMANDS:
            self.assertNotIn(bad, allowed)


if __name__ == "__main__":
    unittest.main()
