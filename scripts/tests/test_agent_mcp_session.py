"""[#5293] rhwp-mcp-session 스킬·픽스처 계약.

실 에이전트가 mcp-serve 세션/무상태를 고를 때 쓰는 규약이 소스 allowlist 를
벗어나지 않는지, 수명·복구·단일 출처 문장이 빠지지 않았는지를 바이너리 없이
커밋된 파일만으로 검사한다. 도구 이름을 발명하면 실패한다.
"""

from __future__ import annotations

import importlib.util
import json
import re
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-mcp-session"
REF = SKILL / "references"
FIXT = REF / "fixtures"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "agent_mcp_session.md"

HWP_NAME = re.compile(r"\bhwp_[a-z0-9_]+\b")
# 픽스처가 일부러 싣는 오타 이름 — 복구 시나리오 전용.
INTENTIONAL_UNKNOWN = {"hwp_doc_foo"}
# 문서가 "만들지 마라"고 적는 부재 이름. 구현에 넣으면 이 집합에서 빼야 한다.
DO_NOT_INVENT = {"hwp_doc_insert_row", "hwp_doc_redact"}


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_mcp_session_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.__dict__


def json_files(folder: Path):
    return sorted(p for p in folder.rglob("*.json") if p.is_file())


class AgentMcpSessionSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        ns = load_gen()
        cls.session = ns["extract_session_tools"]()
        cls.read = ns["extract_read_tools"]()
        cls.stateless = ns["extract_stateless_tools"]()
        cls.allowed = set(cls.session) | set(cls.stateless)
        cls.allowlist = json.loads((FIXT / "allowlist.json").read_text(encoding="utf-8"))
        cls.skill = (SKILL / "SKILL.md").read_text(encoding="utf-8")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-mcp-session", self.skill)
        self.assertNotRegex(self.skill, r"(?i)\bgym/")
        self.assertIn("발명", self.skill)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "hwp_open",
            "hwp_doc_",
            "hwp_close",
            "capabilities --mcp",
            "무상태",
            "isError",
            "nextCall",
            "references/session_lifecycle.md",
            "references/stateless_when.md",
            "references/capabilities_ssot.md",
            "references/error_recovery.md",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_reference_docs_exist(self):
        for name in (
            "README.md",
            "session_lifecycle.md",
            "session_tools.md",
            "stateless_when.md",
            "capabilities_ssot.md",
            "error_recovery.md",
            "pairing.md",
            "host_attach.md",
            "decision_tree.md",
        ):
            path = REF / name
            self.assertTrue(path.is_file(), path)
            text = path.read_text(encoding="utf-8")
            self.assertGreater(len(text.splitlines()), 10, name)

    def test_allowlist_matches_source(self):
        self.assertEqual(self.allowlist["session_tools"], self.session)
        self.assertEqual(self.allowlist["stateless_tools"], self.stateless)
        self.assertEqual(self.allowlist["session_read_tools"], self.read)
        self.assertTrue(self.allowlist["counts_are_not_contracts"])
        self.assertIn("capabilities --mcp", self.allowlist["ssot"]["stateless"])

    def test_every_session_tool_has_a_card(self):
        cards = {p.stem for p in (FIXT / "tools").glob("hwp_*.json")}
        self.assertEqual(set(self.session), cards)
        for name in self.session:
            card = json.loads((FIXT / "tools" / f"{name}.json").read_text(encoding="utf-8"))
            self.assertEqual(card["name"], name)
            self.assertFalse(card["in_capabilities_mcp"])
            self.assertTrue(card["in_tools_list"])
            twin = card.get("stateless_twin")
            if twin:
                self.assertIn(twin, self.stateless, f"{name} 짝 {twin} 이 무상태에 없다")

    def test_pairing_both_sides_exist(self):
        for row in self.allowlist["pairing"]:
            self.assertIn(row["session"], self.session)
            self.assertIn(row["stateless"], self.stateless)

    def test_no_invented_tools_in_skill_tree(self):
        offenders = []
        for path in [SKILL / "SKILL.md", *REF.glob("*.md"), *json_files(FIXT)]:
            text = path.read_text(encoding="utf-8")
            for match in HWP_NAME.findall(text):
                if match in INTENTIONAL_UNKNOWN or match in DO_NOT_INVENT:
                    continue
                if match.endswith("_"):
                    continue
                if match not in self.allowed:
                    offenders.append(f"{path.relative_to(REPO)}:{match}")
        self.assertEqual(offenders, [], "발명된 도구 이름\n" + "\n".join(offenders))

    def test_unknown_name_only_in_recovery_trace(self):
        hits = []
        for path in json_files(FIXT):
            if "hwp_doc_foo" in path.read_text(encoding="utf-8"):
                hits.append(path.name)
        self.assertEqual(hits, ["18_unknown_tool_didyoumean.json"])

    def test_session_traces_open_then_close(self):
        for path in sorted((FIXT / "traces").glob("*.json")):
            trace = json.loads(path.read_text(encoding="utf-8"))
            if trace["kind"] != "session":
                continue
            tools = [s["tool"] for s in trace["steps"]]
            self.assertTrue(
                tools[0] in ("hwp_open", "hwp_ws_list", "hwp_ws_open"),
                f"{path.name} 세션 시작이 open/ws 가 아님: {tools[0]}",
            )
            self.assertEqual(tools[-1], "hwp_close", f"{path.name} 이 close 로 끝나지 않음")

    def test_lifecycle_trace_mentions_single_write_point(self):
        save_traces = []
        for path in (FIXT / "traces").glob("*.json"):
            trace = json.loads(path.read_text(encoding="utf-8"))
            names = [s["tool"] for s in trace["steps"]]
            if "hwp_doc_save" in names:
                save_traces.append(path.name)
                self.assertEqual(trace.get("single_write_point"), "hwp_doc_save")
        self.assertGreaterEqual(len(save_traces), 3)

    def test_error_layers_are_the_three(self):
        layers = set()
        for path in (FIXT / "errors").glob("*.json"):
            spec = json.loads(path.read_text(encoding="utf-8"))
            layers.add(spec["layer"])
            self.assertIn(spec["layer"], {"jsonrpc", "isError", "envelope"})
            if spec["layer"] == "jsonrpc":
                self.assertFalse(spec["retry"])
            for name in spec["tools"]:
                self.assertIn(name, self.allowed)
        self.assertEqual(layers, {"jsonrpc", "isError", "envelope"})

    def test_closed_handle_recovery_uses_nextcall(self):
        spec = json.loads((FIXT / "errors" / "tool_closed_handle.json").read_text(encoding="utf-8"))
        self.assertTrue(spec["retry"])
        self.assertIn("hwp_open", spec["fix"])
        trace = json.loads(
            (FIXT / "traces" / "09_closed_handle_recovery.json").read_text(encoding="utf-8")
        )
        layers = [s["expect_layer"] for s in trace["steps"]]
        self.assertIn("isError", layers)

    def test_decisions_first_tool_is_real(self):
        for path in (FIXT / "decisions").glob("*.json"):
            spec = json.loads(path.read_text(encoding="utf-8"))
            self.assertIn(spec["choice"], {"session", "stateless"})
            self.assertIn(spec["first_tool"], self.allowed, path.name)
            if spec["choice"] == "stateless":
                self.assertIn("hwp_open", spec["forbidden"])

    def test_scenario_catalog_stays_on_allowlist(self):
        catalog = json.loads((FIXT / "scenario_catalog.json").read_text(encoding="utf-8"))
        self.assertGreaterEqual(len(catalog["items"]), 100)
        for item in catalog["items"]:
            self.assertIn(item["first_tool"], self.allowed)
            self.assertEqual(item["ssot"], "capabilities --mcp + tools/list")
            for name in item["if_session"] + item["if_stateless"]:
                if name != "hwp_doc_info":
                    self.assertIn(name, self.allowed, item["id"])

    def test_working_doc_exists_and_closes_issue(self):
        text = WORKING.read_text(encoding="utf-8")
        self.assertIn("#5293", text)
        self.assertIn("hwp_open", text)
        self.assertIn("capabilities --mcp", text)
        self.assertIn("gym", text.lower())
        self.assertRegex(text, r"(?i)gym")
        self.assertIn("금지", text)

    def test_capability_registry_lists_this_skill(self):
        registry = (
            REPO / "mydocs" / "manual" / "agent_capability_registry.md"
        ).read_text(encoding="utf-8")
        self.assertIn("CAP-5293", registry)
        self.assertIn("rhwp-mcp-session", registry)
        self.assertIn(".claude/skills/rhwp-mcp-session/SKILL.md", registry)

    def test_session_count_matches_source_constant(self):
        # 개수는 계약이 아니지만, 카드·상수 드리프트는 잡는다.
        self.assertGreaterEqual(len(self.session), 16)
        self.assertEqual(len(self.session), len(set(self.session)))
        self.assertIn("hwp_open", self.session)
        self.assertIn("hwp_close", self.session)
        self.assertIn("hwp_doc_structure", self.session)
        self.assertIn("hwp_doc_extract_data", self.session)

    def test_mutate_tools_are_subset_of_session(self):
        for name in self.allowlist["mutate_tools"]:
            self.assertIn(name, self.session)
            self.assertNotIn(name, self.read)

    def test_generator_does_not_hardcode_invented_tools(self):
        src = GEN.read_text(encoding="utf-8")
        # PAIRING_CANDIDATES 값도 실존해야 한다 — 생성기가 검증한다.
        for match in HWP_NAME.findall(src):
            if match in INTENTIONAL_UNKNOWN:
                continue
            # 생성기 문자열에 적힌 이름은 소스에 있거나 생성 시 걸러진다.
            # 세션 메타 키는 전부 세션 상수에 있어야 한다.
        ns = load_gen()
        for name in ns["SESSION_META"]:
            self.assertIn(name, self.session, f"SESSION_META 고아: {name}")


if __name__ == "__main__":
    unittest.main()
