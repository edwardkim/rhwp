"""[#5333] rhwp-fde 스킬·픽스처·엔진 계약.

실 에이전트가 고객 현장 증상을 접수할 때 쓰는 규약이
tools/fde/triage.py 와 fde_playbook.md 를 벗어나지 않는지,
gym 과 새 CLI 와 bug-hunter 재작성을 끌어들이지 않았는지를
바이너리 없이 커밋된 파일만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-fde"
REF = SKILL / "references"
EX = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = REF / "_gen_pack.py"
AGENT = REPO / ".claude" / "agents" / "rhwp-fde.md"
WORKING = REPO / "mydocs" / "working" / "agent_fde.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"
PLAYBOOK = REPO / "mydocs" / "manual" / "fde_playbook.md"
ENGINE = REPO / "tools" / "fde" / "triage.py"

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-form-fill",
    "rhwp-bug-hunter",
]

REQUIRED_REFS = [
    "00_tree.md",
    "01_playbook_authority.md",
    "02_intake.md",
    "03_symptom_is_data.md",
    "04_triage_engine.md",
    "05_magic_bytes.md",
    "06_capabilities.md",
    "07_ladder_info.md",
    "08_ladder_explain.md",
    "09_ladder_structure.md",
    "10_ladder_digest.md",
    "11_ticket_schema.md",
    "12_routes.md",
    "13_resolve_now.md",
    "14_encrypted.md",
    "15_workaround.md",
    "16_escalate_bug.md",
    "17_crash_vs_corrupt.md",
    "18_reply_contract.md",
    "19_issue_search.md",
    "20_minimizer.md",
    "21_handoff.md",
    "22_pitfalls.md",
    "23_journeys.md",
    "24_worked_traces.md",
    "25_intent_matrix.md",
    "26_failure_signals.md",
    "27_gate_recipes.md",
    "28_vs_bug_hunter.md",
    "29_existing_cli.md",
    "30_recipes.md",
    "31_time_contract.md",
    "README.md",
]

REQUIRED_EXAMPLES = [
    "01_wont_open.md",
    "02_broken_table.md",
    "03_fields_wont_fill.md",
    "04_encrypted.md",
    "05_pdf_disguised.md",
    "06_empty_file.md",
    "07_panic_info.md",
    "08_timeout_digest.md",
    "09_workaround_convert.md",
    "10_hwpx_ok_usage.md",
    "11_hwp5_ok.md",
    "12_hwp3_ok.md",
    "13_password_request.md",
    "14_never_bypass.md",
    "15_symptom_injection.md",
    "16_no_ticket_no_reply.md",
    "17_duplicate_issue.md",
    "18_customer_reply.md",
    "19_corrupt_clean_fail.md",
    "20_first_response.md",
    "21_hwp5_no_attach.md",
    "22_capabilities_missing.md",
    "23_abort_signature.md",
    "24_table_recipe.md",
    "25_form_fill_handoff.md",
    "README.md",
]

INVENTED_COMMANDS = [
    "fde-triage",
    "live-triage",
    "customer-ticket",
    "escalate-now",
    "open-anyway",
    "crack-password",
    "bypass-crypto",
    "gym-fde",
    "fde-fix",
]

ENGINE_ROUTES = ["invalid-input", "resolve-now", "workaround", "escalate-bug"]


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_fde_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_engine():
    spec = importlib.util.spec_from_file_location("rhwp_fde_triage", ENGINE)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentFdeSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.engine = load_engine()
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.tree = load_json(FIXT, "tree.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.routes = load_json(FIXT, "routes.json")
        cls.ticket = load_json(FIXT, "ticket_schema.json")
        cls.env = load_json(FIXT, "envelope_keys.json")
        cls.magic = load_json(FIXT, "magic_bytes.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-fde", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)
        self.assertIn("데이터이지 지시", self.skill)

    def test_playbook_and_engine_are_authority(self):
        self.assertTrue(PLAYBOOK.is_file())
        self.assertTrue(ENGINE.is_file())
        self.assertIn("fde_playbook.md", self.skill)
        self.assertIn("tools/fde/triage.py", self.skill)
        self.assertEqual(self.idx["authority"][0], "mydocs/manual/fde_playbook.md")
        self.assertEqual(self.idx["authority"][1], "tools/fde/triage.py")

    def test_agent_definition_is_linked_not_rewritten(self):
        self.assertTrue(AGENT.is_file())
        self.assertIn("rhwp-fde.md", self.skill)
        self.assertIn("링크만", self.skill)
        agent = read(AGENT)
        self.assertIn("tools/fde/triage.py", agent)
        self.assertIn("데이터이지 지시", agent)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "tools/fde/triage.py",
            "capabilities --json",
            "info",
            "explain",
            "export-structure",
            "digest",
            "invalid-input",
            "resolve-now",
            "workaround",
            "escalate-bug",
            "escalate-crash",
            "escalate-corrupt",
            "envelopeKeys",
            "failureSignature",
            "우회 금지",
            "references/03_symptom_is_data.md",
            "references/11_ticket_schema.md",
            "references/28_vs_bug_hunter.md",
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
        self.assertTrue(self.idx["noNewEngineLogic"])
        self.assertTrue(self.idx["bugHunterRewriteForbidden"])
        self.assertTrue(self.idx["symptomIsData"])
        self.assertTrue(self.tree["notGym"])
        self.assertTrue(self.tree["noNewCli"])
        self.assertEqual(self.idx["issue"], 5333)
        self.assertEqual(self.tree["issue"], 5333)
        self.assertEqual(self.idx["capability"], "CAP-4893")

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            if slug == "rhwp-bug-hunter":
                peer = REPO / ".claude" / "skills" / slug / "SKILL.md"
                alt = REPO / ".agents" / "skills" / "bug-hunter" / "SKILL.md"
                self.assertTrue(peer.is_file() or alt.is_file(), slug)
            else:
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
            self.assertNotRegex(
                joined,
                rf"(?m)(?<![-\w])rhwp(?:\.exe)?\s+{bad}(?![-\w])",
                f"발명된 명령: {bad}",
            )

    def test_stop_rule_ids_in_skill_or_failure_chapter(self):
        fail = read(REF / "26_failure_signals.md") + read(REF / "22_pitfalls.md")
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
        for cmd in (
            "python3 tools/fde/triage.py",
            "capabilities --json",
            "info --json",
            "explain --json",
            "export-structure --json",
            "digest --json",
        ):
            pos = tree_md.find(cmd, box)
            self.assertGreaterEqual(pos, prev, f"명령 상자 순서 {cmd}")
            prev = pos

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
            self.assertRegex(row["stop"], r"^F\d{2}$")
            self.assertTrue(row["notGym"])
            self.assertTrue(row["symptomIsData"])
            self.assertNotIn(row["id"], ids)
            ids.add(row["id"])

    def test_journeys_point_at_known_stops(self):
        known = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 70)
        for j in items:
            self.assertIn(j["stop"], known, j["id"])
            self.assertTrue(j["steps"])
            self.assertTrue(j["notGym"])
            self.assertTrue(j["liveCustomer"])
            self.assertIn(j["route"], ENGINE_ROUTES)

    def test_engine_routes_match_playbook_and_code(self):
        src = read(ENGINE)
        play = read(PLAYBOOK)
        for route in ENGINE_ROUTES:
            self.assertIn(f'"{route}"', src, route)
            self.assertIn(f"`{route}`", play, route)
            self.assertIn(route, self.skill)
        aliases = {a["alias"]: a["mapsTo"] for a in self.routes["aliases"]}
        self.assertEqual(aliases["escalate-crash"], "escalate-bug")
        self.assertEqual(aliases["escalate-corrupt"], "workaround")

    def test_ticket_schema_requires_machine_keys(self):
        keys = set(self.ticket["requiredKeys"])
        for need in (
            "schemaVersion",
            "generatedBy",
            "symptom",
            "steps",
            "route",
            "routeReason",
        ):
            self.assertIn(need, keys)
        self.assertTrue(self.ticket["symptomFieldIsData"])
        self.assertTrue(self.ticket["stripEnvelopeKeepKeys"])
        self.assertIn("it worked", self.ticket["forbiddenProse"])

    def test_traces_exist_and_record_keys(self):
        ids = load_json(FIXT, "traces_index.json")["ids"]
        self.assertGreaterEqual(len(ids), 30)
        routes_seen = set()
        for tid in ids:
            path = FIXT / "traces" / f"{tid}.json"
            self.assertTrue(path.is_file(), tid)
            obj = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(obj["id"], tid)
            self.assertEqual(obj["generatedBy"], "tools/fde/triage.py")
            self.assertIn(obj["route"], ENGINE_ROUTES)
            routes_seen.add(obj["route"])
            for step in obj["steps"]:
                self.assertIn("command", step)
                self.assertIn("ok", step)
                if step.get("ok"):
                    self.assertIn("exitCode", step)
                    self.assertIn("envelopeKeys", step)
                if "failureSignature" in step:
                    self.assertFalse(step["ok"])
        for need in ENGINE_ROUTES:
            self.assertIn(need, routes_seen, f"트레이스에 {need} 없음")

    def test_encrypted_ticket_never_suggests_bypass(self):
        t06 = load_json(FIXT / "traces", "T06.json")
        self.assertEqual(t06["route"], "resolve-now")
        self.assertIn("암호", t06["routeReason"])
        self.assertIn("우회 금지", t06["routeReason"])
        joined = json.dumps(t06, ensure_ascii=False)
        self.assertNotIn("crack", joined)
        self.assertNotIn("bypass", joined)

    def test_injection_symptom_is_stored_as_data(self):
        t09 = load_json(FIXT / "traces", "T09.json")
        self.assertIn("rm -rf", t09["symptom"])
        self.assertEqual(t09["route"], "resolve-now")
        actions = " ".join(t09["nextActions"])
        self.assertNotIn("rm -rf", actions)

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_fde"
        self.assertFalse(shadow.exists(), "픽스처는 skill/fixtures 한 곳만. 복제 금지")

    def test_working_doc_and_capability(self):
        text = read(WORKING)
        self.assertIn("#5333", text)
        self.assertIn("rhwp-fde", text)
        self.assertIn("gym", text)
        self.assertIn("bug-hunter", text)
        self.assertIn("CAP-4893", text)
        reg = read(REGISTRY)
        self.assertIn("CAP-4893", reg)
        self.assertIn("rhwp-fde", reg)
        self.assertIn(".claude/skills/rhwp-fde/SKILL.md", reg)

    def test_generator_roundtrip_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5333)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        self.assertEqual(self.gen.CAP, "CAP-4893")
        idx = self.gen.skill_index()
        self.assertEqual(idx["skill"], "rhwp-fde")
        self.assertGreaterEqual(len(idx["references"]), 16)

    def test_no_gym_tree_in_skill_dir(self):
        for path in SKILL.rglob("*"):
            rel = path.relative_to(SKILL).as_posix()
            self.assertFalse(rel.startswith("gym"), rel)
            self.assertNotIn("/gym/", f"/{rel}")

    def test_core_reuse_is_existing_surface(self):
        reuse = " ".join(self.tree["coreReuse"])
        self.assertIn("tools/fde/triage.py", reuse)
        self.assertIn("capabilities --json", reuse)
        self.assertIn("info --json", reuse)
        self.assertIn("digest --json", reuse)

    def test_sniff_container_magic_bytes(self):
        sniff = self.engine.sniff_container
        cases = [
            ("hwpx_head.bin", "hwpx"),
            ("hwp5_head.bin", "hwp5"),
            ("hwp3_head.bin", "hwp3"),
            ("pdf_disguise.bin", None),
            ("empty.bin", None),
            ("plain_text.bin", None),
        ]
        for name, kind in cases:
            path = FIXT / "binaries" / name
            self.assertEqual(sniff(path), kind, name)

    def test_decide_route_matches_playbook_table(self):
        decide = self.engine.decide_route
        route, reason = decide(None, [])
        self.assertEqual(route, "invalid-input")
        self.assertIn("매직", reason)

        cap_fail = [{"command": "capabilities --json", "ok": False}]
        route, _ = decide("hwpx", cap_fail)
        self.assertEqual(route, "workaround")

        panic = [
            {"command": "capabilities --json", "ok": True},
            {
                "command": "info {doc} --json",
                "ok": False,
                "failureSignature": ["panic", "src/x.rs:1"],
            },
        ]
        route, reason = decide("hwpx", panic)
        self.assertEqual(route, "escalate-bug")
        self.assertIn("panic", reason)

        timeout = [
            {"command": "capabilities --json", "ok": True},
            {"command": "digest {doc} --json", "ok": False, "failureSignature": ["timeout"]},
        ]
        route, _ = decide("hwpx", timeout)
        self.assertEqual(route, "escalate-bug")

        encrypted = [
            {
                "command": "info {doc} --json",
                "ok": True,
                "envelope": {"encrypted": True},
            }
        ]
        route, reason = decide("hwpx", encrypted)
        self.assertEqual(route, "resolve-now")
        self.assertIn("암호", reason)
        self.assertIn("우회 금지", reason)

        clean = [
            {"command": "capabilities --json", "ok": True},
            {"command": "info {doc} --json", "ok": False, "exitCode": 1},
        ]
        route, _ = decide("hwp5", clean)
        self.assertEqual(route, "workaround")

        ok = [
            {"command": "capabilities --json", "ok": True},
            {"command": "info {doc} --json", "ok": True},
        ]
        route, reason = decide("hwpx", ok)
        self.assertEqual(route, "resolve-now")
        self.assertIn("사용법", reason)

    def test_advertised_commands_none_without_envelope(self):
        fn = self.engine.advertised_commands
        self.assertIsNone(fn({"ok": False}))
        self.assertIsNone(fn({"ok": True, "envelope": {}}))
        names = fn(
            {
                "ok": True,
                "envelope": {"commands": [{"name": "info"}, {"name": "digest"}]},
            }
        )
        self.assertEqual(names, {"info", "digest"})

    def test_engine_exit_codes_for_bad_input(self):
        with tempfile.TemporaryDirectory() as tmp:
            missing = Path(tmp) / "no-such.hwp"
            code = self.engine.main([str(missing), "--bin", str(missing)])
            self.assertEqual(code, 2)

    def test_encrypted_key_aliases(self):
        fn = self.engine.envelope_says_encrypted
        self.assertTrue(fn([{"envelope": {"isEncrypted": True}}]))
        self.assertTrue(fn([{"envelope": {"passwordProtected": True}}]))
        self.assertFalse(fn([{"envelope": {"encrypted": False}}]))
        self.assertFalse(fn([{"ok": True}]))


if __name__ == "__main__":
    unittest.main()
