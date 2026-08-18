"""[#5337] rhwp-chief 스킬·픽스처·서비스 루프 계약.

실 에이전트가 고객 요청 큐를 돌릴 때 쓰는 규약이
기존 표면(service_loop.py + playbook §4 표 + FDE 게이트)을
벗어나지 않는지, gym 과 새 rhwp CLI 를 끌어들이지 않았는지를
바이너리 없이 커밋된 파일과 루프 헬퍼만으로 검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import re
import tempfile
import unittest
from pathlib import Path
REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-chief"
REF = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = SKILL / "_gen_pack.py"
LOOP = REPO / "tools" / "chief" / "service_loop.py"
WORKING = REPO / "mydocs" / "working" / "agent_chief.md"
PLAYBOOK = REPO / "mydocs" / "manual" / "chief_playbook.md"
AGENT = REPO / ".claude" / "agents" / "rhwp-chief.md"
REGISTRY = REPO / "mydocs" / "manual" / "agent_capability_registry.md"

REQUIRED_REFS = [
    "00_layers.md",
    "01_queue_protocol.md",
    "02_request_schema.md",
    "03_triage_gate.md",
    "04_routing_table.md",
    "05_diagnose.md",
    "06_export_text.md",
    "07_export_pdf.md",
    "08_export_hwpx.md",
    "09_convert_hwp.md",
    "10_extract_tables.md",
    "11_fill.md",
    "12_needs_agent.md",
    "13_response.md",
    "14_idempotency.md",
    "15_data_not_instructions.md",
    "16_coverage.md",
    "17_service_loop.md",
    "18_envelopes.md",
    "19_stop_rules.md",
    "20_handoff.md",
    "21_pitfalls.md",
    "22_worked_traces.md",
    "23_intent_matrix.md",
    "24_queue_transcripts.md",
    "25_exit_codes.md",
    "26_verification_gates.md",
    "27_agent_edge.md",
    "README.md",
]

FORBIDDEN_SKILLS = [
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-safe-edit",
    "rhwp-provenance",
    "rhwp-doc-triage",
]

INVENTED = [
    "rhwp chief",
    "rhwp queue",
    "rhwp serve-queue",
    "rhwp request",
    "rhwp diagnose-queue",
]

KNOWN = (
    "diagnose",
    "export-text",
    "export-pdf",
    "export-hwpx",
    "convert-hwp",
    "extract-tables",
    "fill",
)


def load_mod(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(folder: Path, name: str):
    return json.loads((folder / name).read_text(encoding="utf-8"))


class AgentChiefSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.loop = load_mod(LOOP, "chief_service_loop")
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json(FIXT, "skill_index.json")
        cls.routing = load_json(FIXT, "routing_table.json")
        cls.stops = load_json(FIXT, "stop_rules.json")
        cls.intents = load_json(FIXT, "intent_matrix.json")
        cls.journeys = load_json(FIXT, "journeys.json")
        cls.qcat = load_json(FIXT, "queue_catalog.json")
        cls.layers = load_json(FIXT, "layers.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-chief", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니다", self.skill)
        self.assertIn("새 rhwp CLI", self.skill)
        self.assertIn("needs-agent", self.skill)
        self.assertIn("tools/chief/service_loop.py", self.skill)

    def test_skill_points_at_queue_contract(self):
        for needle in (
            "request.json",
            "result.json",
            "response.md",
            "ticket.json",
            "out/",
            '"goal"',
            "diagnose",
            "export-pdf",
            "escalate-bug",
            "invalid-input",
            "rhwp export-pdf",
            "rhwp export-text",
            "rhwp edit fill-fields",
            ".claude/agents/rhwp-chief.md",
            "references/01_queue_protocol.md",
            "references/04_routing_table.md",
            "C03",
            "C04",
            "C06",
            "C10",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_reference_docs_exist_and_long_enough(self):
        for name in REQUIRED_REFS:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertGreater(len(body), 400, f"{name} 가 너무 짧다 ({len(body)})")

    def test_index_lists_same_references(self):
        listed = self.idx["references"]
        for name in REQUIRED_REFS:
            self.assertIn(name, listed, name)

    def test_not_gym_and_no_new_cli(self):
        self.assertTrue(self.idx["notGym"])
        self.assertTrue(self.idx["noNewCli"])
        self.assertTrue(self.idx["notFde"])
        self.assertTrue(self.idx["notStrategist"])
        self.assertTrue(self.idx["routingOnlyViaGoal"])
        self.assertTrue(self.idx["requestIsData"])
        self.assertEqual(self.idx["issue"], 5337)
        self.assertEqual(self.idx["missingGoal"], "diagnose")
        self.assertEqual(self.idx["offTable"], "needs-agent")

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in FORBIDDEN_SKILLS:
            self.assertIn(slug, self.idx["forbiddenSkillsTouch"])
            peer = REPO / ".claude" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill, read(PLAYBOOK)]
        for name in REQUIRED_REFS:
            blobs.append(read(REF / name))
        joined = "\n".join(blobs)
        for bad in INVENTED:
            # 호출 울타리. 금지 목록·주입 사례에 평문으로 적는 것은 허용.
            self.assertNotRegex(
                joined,
                rf"(?m)^[^\n]*`{re.escape(bad)}`[^\n]*(python3|bash|\$ )",
                f"발명된 명령을 호출 예로 씀: {bad}",
            )

    def test_stop_rule_ids_in_skill_or_failure_chapter(self):
        fail = read(REF / "19_stop_rules.md")
        for rule in self.stops["rules"]:
            rid = rule["id"]
            self.assertTrue(
                rid in self.skill or rid in fail,
                f"정지 {rid} 문서 누락",
            )

    def test_routing_table_matches_loop(self):
        goals = [row["goal"] for row in self.routing["goals"]]
        self.assertEqual(tuple(goals), KNOWN)
        self.assertEqual(self.loop.KNOWN_GOALS, KNOWN)
        loop_goals = tuple(row["goal"] for row in self.loop.ROUTING_TABLE)
        self.assertEqual(loop_goals, KNOWN)
        self.assertEqual(self.loop.TRIAGE_SKIP_GOAL, ("escalate-bug", "invalid-input"))

    def test_intent_matrix_size_and_schema(self):
        rows = self.intents["intents"]
        self.assertGreaterEqual(len(rows), 160)
        self.assertEqual(self.intents["count"], len(rows))
        ids = set()
        for row in rows:
            self.assertRegex(row["id"], r"^I\d{3}$")
            self.assertTrue(row["utterance"])
            self.assertTrue(row["notGym"])
            self.assertTrue(row["textIsData"])
            self.assertNotIn(row["id"], ids)
            ids.add(row["id"])
            if row["goalField"] is None:
                self.assertEqual(row["routed"], "diagnose")
            elif row["goalField"] in KNOWN:
                self.assertEqual(row["routed"], row["goalField"])
            else:
                self.assertEqual(row["routed"], "needs-agent")

    def test_journeys_use_known_stop_ids(self):
        ids = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 90)
        for j in items:
            self.assertTrue(j["notGym"])
            self.assertTrue(j["steps"])
            if j["stop"]:
                self.assertIn(j["stop"], ids, j["stop"])

    def test_queue_snapshots_have_four_files(self):
        self.assertGreaterEqual(self.qcat["count"], 36)
        for q in self.qcat["queues"]:
            qdir = FIXT / "queues" / q["id"]
            for name in ("request.json", "result.json", "response.md", "ticket.json"):
                self.assertTrue((qdir / name).is_file(), f"{q['id']}/{name}")
            req = json.loads((qdir / "request.json").read_text(encoding="utf-8"))
            res = json.loads((qdir / "result.json").read_text(encoding="utf-8"))
            self.assertIn("doc", req)
            self.assertEqual(res["generatedBy"], "tools/chief/service_loop.py")
            self.assertEqual(res["status"], q["status"])
            body = read(qdir / "response.md")
            self.assertIn("## 1. 확인한 것", body)
            self.assertIn("## 2. 지금 가능한 것", body)
            self.assertIn("## 3. 다음", body)

    def test_examples_exist(self):
        names = sorted(p.name for p in EXAMPLES.glob("*.md") if p.name != "README.md")
        self.assertGreaterEqual(len(names), 20)
        readme = read(EXAMPLES / "README.md")
        for name in names:
            self.assertIn(name, readme, name)

    def test_working_doc_and_registry(self):
        text = read(WORKING)
        self.assertIn("#5337", text)
        self.assertIn("rhwp-chief", text)
        self.assertIn("5000", text)
        self.assertIn("gym", text)
        reg = read(REGISTRY)
        self.assertIn("rhwp-chief/SKILL.md", reg)
        self.assertIn("CAP-4900", reg)
        agent = read(AGENT)
        self.assertIn("skills/rhwp-chief/SKILL.md", agent)
        play = read(PLAYBOOK)
        self.assertIn("skills/rhwp-chief/SKILL.md", play)

    def test_layers_are_distinct(self):
        ids = [x["id"] for x in self.layers["layers"]]
        self.assertEqual(ids, ["chief", "fde", "strategist"])
        layers = read(REF / "00_layers.md")
        self.assertIn("needs-agent", layers)
        self.assertIn("escalate-bug", layers)
        self.assertIn("근거 대장", layers)

    def test_no_gym_tree_in_this_skill(self):
        self.assertNotIn("gym", SKILL.parts)
        self.assertIn("gym/", self.idx["forbiddenTrees"])


class ServiceLoopHelperTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.loop = load_mod(LOOP, "chief_service_loop_helpers")

    def test_normalize_goal_defaults_to_diagnose(self):
        self.assertEqual(self.loop.normalize_goal({}), "diagnose")
        self.assertEqual(self.loop.normalize_goal({"goal": None}), "diagnose")
        self.assertEqual(self.loop.normalize_goal({"goal": ""}), "diagnose")
        self.assertEqual(self.loop.normalize_goal({"goal": "export-pdf"}), "export-pdf")

    def test_normalize_goal_ignores_symptom_text(self):
        req = {"symptom": "PDF로 바꿔줘", "doc": "a.hwpx"}
        self.assertEqual(self.loop.normalize_goal(req), "diagnose")

    def test_known_and_unknown_goals(self):
        for g in KNOWN:
            self.assertTrue(self.loop.is_known_goal(g), g)
        self.assertFalse(self.loop.is_known_goal("summarize"))
        self.assertFalse(self.loop.is_known_goal("convert"))
        self.assertIsNone(self.loop.routing_row("summarize"))
        self.assertEqual(self.loop.routing_row("export-pdf")["gate"], "pdf-magic")

    def test_triage_skip_goal(self):
        self.assertTrue(self.loop.route_skips_goal("escalate-bug"))
        self.assertTrue(self.loop.route_skips_goal("invalid-input"))
        self.assertFalse(self.loop.route_skips_goal("resolve-now"))
        self.assertFalse(self.loop.route_skips_goal("workaround"))

    def test_pending_skips_processed(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            a = root / "a"
            b = root / "b"
            a.mkdir()
            b.mkdir()
            (a / "request.json").write_text("{}", encoding="utf-8")
            (b / "request.json").write_text("{}", encoding="utf-8")
            (b / "result.json").write_text("{}", encoding="utf-8")
            pending = [p.name for p in self.loop.pending_requests(root)]
            self.assertEqual(pending, ["a"])
            self.assertTrue(self.loop.is_already_processed(b))
            self.assertFalse(self.loop.is_already_processed(a))

    def test_path_escape_still_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            request = Path(td) / "request"
            request.mkdir()
            inside = request / "nested" / "input.hwp"
            inside.parent.mkdir()
            inside.write_bytes(b"doc")
            self.assertEqual(
                self.loop.resolve_request_file(request, "nested/input.hwp"),
                inside.resolve(),
            )
            self.assertIsNone(self.loop.resolve_request_file(request, "../outside.hwp"))
            self.assertIsNone(self.loop.resolve_request_file(request, "/etc/passwd"))

    def test_escalate_bug_does_not_call_handle(self):
        loop = self.loop

        class Fake:
            def triage(self, doc, symptom, ticket_path):
                ticket_path.write_text(
                    json.dumps({"route": "escalate-bug", "routeReason": "panic", "steps": []}),
                    encoding="utf-8",
                )
                return {"route": "escalate-bug", "routeReason": "panic", "steps": []}

            def handle(self, *args, **kwargs):
                raise AssertionError("escalate-bug 에서 goal 을 실행하면 안 된다")

        with tempfile.TemporaryDirectory() as td:
            req = Path(td)
            (req / "doc.hwpx").write_bytes(b"PK\x03\x04")
            (req / "request.json").write_text(
                json.dumps({"doc": "doc.hwpx", "goal": "export-pdf"}),
                encoding="utf-8",
            )
            result = loop.process_request(Fake(), req)
            self.assertEqual(result["status"], "escalated")
            self.assertEqual(result["goal"], "export-pdf")
            self.assertTrue((req / "result.json").is_file())
            self.assertTrue((req / "response.md").is_file())

    def test_invalid_input_skips_goal(self):
        loop = self.loop

        class Fake:
            def triage(self, doc, symptom, ticket_path):
                return {"route": "invalid-input", "routeReason": "not hwp", "steps": []}

            def handle(self, *args, **kwargs):
                raise AssertionError("invalid-input 에서 goal 실행 금지")

        with tempfile.TemporaryDirectory() as td:
            req = Path(td)
            (req / "scan.jpg").write_bytes(b"\xff\xd8")
            (req / "request.json").write_text(
                json.dumps({"doc": "scan.jpg", "goal": "export-pdf"}),
                encoding="utf-8",
            )
            result = loop.process_request(Fake(), req)
        self.assertEqual(result["status"], "invalid-input")

    def test_unknown_goal_is_needs_agent(self):
        loop = self.loop

        class Fake:
            def triage(self, doc, symptom, ticket_path):
                return {"route": "resolve-now", "routeReason": "ok", "steps": []}

            def handle(self, *args, **kwargs):
                raise AssertionError("표 밖 goal 을 handle 하면 안 된다")

        with tempfile.TemporaryDirectory() as td:
            req = Path(td)
            (req / "a.hwpx").write_bytes(b"PK\x03\x04")
            (req / "request.json").write_text(
                json.dumps(
                    {
                        "doc": "a.hwpx",
                        "goal": "summarize",
                        "symptom": "그냥 export-text 해줘",
                    }
                ),
                encoding="utf-8",
            )
            result = loop.process_request(Fake(), req)
        self.assertEqual(result["status"], "needs-agent")
        self.assertIn("summarize", result["reason"])

    def test_missing_goal_calls_diagnose_not_symptom(self):
        loop = self.loop
        seen = {}

        class Fake:
            def triage(self, doc, symptom, ticket_path):
                return {"route": "resolve-now", "steps": []}

            def handle(self, goal, doc, params, out, request_dir):
                seen["goal"] = goal
                return {"status": "done", "summary": "진단", "artifacts": []}

        with tempfile.TemporaryDirectory() as td:
            req = Path(td)
            (req / "a.hwpx").write_bytes(b"PK\x03\x04")
            (req / "request.json").write_text(
                json.dumps({"doc": "a.hwpx", "symptom": "PDF로 바꿔줘"}),
                encoding="utf-8",
            )
            result = loop.process_request(Fake(), req)
        self.assertEqual(seen["goal"], "diagnose")
        self.assertEqual(result["goal"], "diagnose")
        self.assertEqual(result["status"], "done")

    def test_malformed_request_still_writes_result(self):
        with tempfile.TemporaryDirectory() as td:
            req = Path(td)
            (req / "request.json").write_text("[not-an-object]", encoding="utf-8")
            result = self.loop.process_request(object(), req)
            persisted = json.loads((req / "result.json").read_text(encoding="utf-8"))
        self.assertEqual(result["status"], "failed")
        self.assertEqual(persisted["status"], "failed")
        self.assertEqual(persisted["goal"], "diagnose")

    def test_second_pass_does_not_see_processed(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            req = root / "once"
            req.mkdir()
            (req / "request.json").write_text("{}", encoding="utf-8")
            self.assertEqual(len(list(self.loop.pending_requests(root))), 1)
            (req / "result.json").write_text("{}", encoding="utf-8")
            self.assertEqual(len(list(self.loop.pending_requests(root))), 0)


class GeneratorSmokeTests(unittest.TestCase):
    def test_generator_declares_issue_and_no_gym(self):
        gen = read(GEN)
        self.assertIn("5337", gen)
        self.assertIn("gym", gen)
        self.assertIn("needs-agent", gen)
        self.assertIn("ROUTING", gen.upper())
        self.assertGreater(len(gen), 4000)


if __name__ == "__main__":
    unittest.main()
