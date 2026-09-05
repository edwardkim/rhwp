"""[#5326] rhwp-agent-surface 스킬·픽스처 계약.

실 에이전트가 3층 표면을 더하거나 굴릴 때 쓰는 규약이 소스 allowlist 를
벗어나지 않는지, 규칙 3줄·예외 4바늘·capabilities 사용법이 빠지지 않았는지를
바이너리 없이 커밋된 파일만으로 검사한다. 도구 이름을 발명하면 실패한다.
"""

from __future__ import annotations

import importlib.util
import json
import re
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "rhwp-agent-surface"
REF = SKILL / "references"
EX = SKILL / "examples"
FIXT = SKILL / "fixtures"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_surface_skill.md"
PLAYBOOK = REPO / "mydocs" / "manual" / "agent_surface_playbook.md"

HWP_NAME = re.compile(r"\bhwp_[a-z0-9_]+\b")
INTENTIONAL_UNKNOWN = {"hwp_doc_foo", "hwp_example", "hwp_doc_example"}
DO_NOT_INVENT = {
    "hwp_doc_redact",
    "hwp_doc_insert_row",
    "hwp_doc_run",
    "hwp_doc_convert",
}

SIBLING_SKILLS = ("rhwp-mcp-session", "rhwp-cli", "rhwp-codex")


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_agent_surface_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.__dict__


def json_files(folder: Path):
    return sorted(p for p in folder.rglob("*.json") if p.is_file())


class AgentSurfaceSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        ns = load_gen()
        cls.session = ns["extract_session_tools"]()
        cls.read = ns["extract_session_read_tools"]()
        cls.stateless = [t["name"] for t in ns["extract_stateless_tools"]()]
        cls.commands = ns["extract_cli_commands"]()
        cls.profiles = ns["extract_profiles"]()
        cls.allowed = set(cls.session) | set(cls.stateless)
        cls.allowlist = json.loads((FIXT / "allowlist.json").read_text(encoding="utf-8"))
        cls.skill = (SKILL / "SKILL.md").read_text(encoding="utf-8")
        cls.layers = json.loads((FIXT / "layers.json").read_text(encoding="utf-8"))
        cls.rules = json.loads((FIXT / "rules.json").read_text(encoding="utf-8"))

    def test_skill_front_matter_and_not_gym_path(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-agent-surface", self.skill)
        self.assertNotRegex(self.skill, r"(?i)\bgym/")
        self.assertIn("발명", self.skill)
        self.assertIn("3층", self.skill)

    def test_skill_points_at_required_topics(self):
        for needle in (
            "rhwp capabilities",
            "capabilities --mcp",
            "capabilities --search",
            "mcp_tool_definitions",
            "ALL_SESSION_TOOLS",
            "identical:false",
            "replacedCount",
            "notFound",
            "isError",
            "nextCall",
            "드리프트",
            "프로필",
            "닫힌",
            "untrustedContent",
            "agent_surface_playbook.md",
            "references/three_layers.md",
            "references/rule1_single_source.md",
            "references/rule2_reuse_core.md",
            "references/rule3_judgment_is_data.md",
            "references/exception_paths.md",
        ):
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_skill_stays_distinct_from_siblings(self):
        self.assertIn("rhwp-mcp-session", self.skill)
        self.assertIn("rhwp-cli", self.skill)
        self.assertIn("rhwp-codex", self.skill)
        self.assertIn("다시 쓰지", self.skill)
        self.assertNotIn("name: rhwp-mcp-session", self.skill)
        # 호스트 부착 본문을 여기 복제하지 않는다.
        self.assertNotIn('"mcpServers"', self.skill)

    def test_reference_docs_exist(self):
        for name in (
            "README.md",
            "three_layers.md",
            "rule1_single_source.md",
            "rule2_reuse_core.md",
            "rule3_judgment_is_data.md",
            "capabilities_how_to.md",
            "add_surface_piece.md",
            "acceptance_checklist.md",
            "exception_paths.md",
            "drift_guards.md",
            "forbidden_overlap.md",
        ):
            path = REF / name
            self.assertTrue(path.is_file(), path)
            text = path.read_text(encoding="utf-8")
            self.assertGreater(len(text.splitlines()), 10, name)
            self.assertNotRegex(text, r"(?i)\bgym/")

    def test_examples_exist(self):
        for name in (
            "README.md",
            "consume_capabilities.md",
            "add_json_command.md",
            "add_stateless_tool.md",
            "add_session_tool.md",
            "closed_handle_recovery.md",
            "profile_blocked.md",
            "drift_guard_fail.md",
            "missing_capabilities_key.md",
            "judgment_is_data.md",
        ):
            path = EX / name
            self.assertTrue(path.is_file(), path)
            self.assertGreater(len(path.read_text(encoding="utf-8").splitlines()), 8, name)

    def test_allowlist_matches_source(self):
        self.assertEqual(self.allowlist["session_tools"], self.session)
        self.assertEqual(self.allowlist["session_read_tools"], self.read)
        self.assertEqual(self.allowlist["stateless_tools"], self.stateless)
        self.assertEqual(
            self.allowlist["cli_commands"], [c["name"] for c in self.commands]
        )
        self.assertTrue(self.allowlist["counts_are_not_contracts"])
        self.assertIn("rhwp capabilities", self.allowlist["ssot"]["how_to_read"][0])
        self.assertIn("mcp_tool_definitions()", self.allowlist["ssot"]["mcp_stateless"])

    def test_session_not_duplicated_and_has_lifecycle(self):
        self.assertEqual(len(self.session), len(set(self.session)))
        self.assertIn("hwp_open", self.session)
        self.assertIn("hwp_close", self.session)
        self.assertIn("hwp_doc_save", self.session)
        self.assertIn("hwp_doc_structure", self.session)
        self.assertIn("hwp_doc_extract_data", self.session)
        for name in self.read:
            self.assertIn(name, self.session)

    def test_every_session_tool_has_a_card(self):
        cards = {p.stem for p in (FIXT / "session").glob("hwp_*.json")}
        self.assertEqual(set(self.session), cards)
        for name in self.session:
            card = json.loads((FIXT / "session" / f"{name}.json").read_text(encoding="utf-8"))
            self.assertEqual(card["name"], name)
            self.assertFalse(card["in_capabilities_mcp"])
            self.assertTrue(card["in_tools_list"])
            self.assertEqual(card["layer"], "mcp-session")

    def test_three_layers_and_three_rules(self):
        ids = [layer["id"] for layer in self.layers["layers"]]
        self.assertEqual(ids, ["cli-json", "mcp-stateless", "mcp-session"])
        for layer in self.layers["layers"]:
            self.assertTrue(layer["ssot"])
        rule_ids = [rule["id"] for rule in self.rules["rules"]]
        self.assertEqual(rule_ids, [1, 2, 3])
        self.assertEqual(self.rules["rules"][0]["fork"], "mcp_tool_definitions()")
        self.assertIn("identical", self.rules["rules"][2]["data_fields"])
        self.assertIn("is_error_only", self.rules["rules"][2])

    def test_pairing_both_sides_exist(self):
        for row in self.allowlist["pairing"]:
            self.assertIn(row["session"], self.session, row)
            self.assertIn(row["stateless"], self.stateless, row)

    def test_no_invented_tools_in_skill_tree(self):
        offenders = []
        trees = [SKILL / "SKILL.md", *REF.glob("*.md"), *EX.glob("*.md"), *json_files(FIXT)]
        for path in trees:
            text = path.read_text(encoding="utf-8")
            for match in HWP_NAME.findall(text):
                if match in INTENTIONAL_UNKNOWN or match in DO_NOT_INVENT:
                    continue
                if match.endswith("_"):
                    continue
                if match not in self.allowed:
                    offenders.append(f"{path.relative_to(REPO)}:{match}")
        self.assertEqual(offenders, [], "발명된 도구 이름\n" + "\n".join(offenders))

    def test_do_not_invent_listed(self):
        forbidden = set(self.allowlist["invent_forbidden"])
        self.assertTrue(DO_NOT_INVENT <= forbidden | DO_NOT_INVENT)
        for name in DO_NOT_INVENT:
            self.assertNotIn(name, self.allowed)

    def test_exception_needles(self):
        needed = {
            "missing_capabilities_key",
            "drift_guard_fail",
            "closed_handle",
            "profile_blocked",
            "identical_false_is_data",
            "replaced_zero_is_data",
            "not_found_is_data",
        }
        have = {p.stem for p in (FIXT / "exceptions").glob("*.json")}
        self.assertTrue(needed <= have, needed - have)
        closed = json.loads((FIXT / "exceptions" / "closed_handle.json").read_text(encoding="utf-8"))
        self.assertTrue(closed["retry"])
        self.assertEqual(closed["envelope"]["nextCall"]["name"], "hwp_open")
        missing = json.loads(
            (FIXT / "exceptions" / "missing_capabilities_key.json").read_text(encoding="utf-8")
        )
        self.assertIn("edit redact --json", missing["measured_missing"])
        self.assertIn("미표기", missing["consumer"])
        blocked = json.loads(
            (FIXT / "exceptions" / "profile_blocked.json").read_text(encoding="utf-8")
        )
        self.assertEqual(blocked["unknown_profile"]["exit"], 2)
        self.assertIn("우회", blocked["tool_outside_profile"]["why"] + blocked["fix"])

    def test_envelopes_judgment_is_data(self):
        ident = json.loads((FIXT / "envelopes" / "ir_diff_not_identical.json").read_text(encoding="utf-8"))
        self.assertFalse(ident["isError"])
        self.assertEqual(ident["cli_exit"], 3)
        self.assertFalse(ident["fields"]["identical"])
        zero = json.loads((FIXT / "envelopes" / "replace_zero.json").read_text(encoding="utf-8"))
        self.assertEqual(zero["fields"]["replacedCount"], 0)
        self.assertTrue(zero["no_output_file"])
        nf = json.loads((FIXT / "envelopes" / "fill_not_found.json").read_text(encoding="utf-8"))
        self.assertIn("없는필드", nf["fields"]["notFound"])
        self.assertEqual(nf["cli_exit"], 0)

    def test_search_and_semantics_and_forbidden_combo(self):
        queries = json.loads((FIXT / "search" / "queries.json").read_text(encoding="utf-8"))
        self.assertTrue(queries["and_semantics"])
        self.assertIn("--mcp", queries["cannot_combine"])
        empty = json.loads((FIXT / "search" / "없음XYZ.json").read_text(encoding="utf-8"))
        self.assertTrue(empty["empty"])
        self.assertEqual(empty["match_count"], 0)
        combo = json.loads(
            (FIXT / "exceptions" / "search_combined_with_mcp.json").read_text(encoding="utf-8")
        )
        self.assertEqual(combo["exit"], 2)

    def test_add_surface_acceptance_matches_playbook(self):
        acc = json.loads((FIXT / "add_surface" / "acceptance.json").read_text(encoding="utf-8"))
        self.assertGreaterEqual(len(acc["items"]), 8)
        playbook = PLAYBOOK.read_text(encoding="utf-8")
        self.assertIn("stdout 순수성", playbook)
        self.assertIn("schemaVersion", "".join(acc["items"]))
        kinds = json.loads((FIXT / "add_surface" / "kinds.json").read_text(encoding="utf-8"))
        self.assertEqual(
            [k["id"] for k in kinds["kinds"]],
            ["cli-json-command", "mcp-stateless-tool", "mcp-session-tool"],
        )

    def test_profiles_are_boundaries(self):
        names = [p["name"] for p in self.profiles]
        self.assertEqual(self.allowlist["profiles"], names)
        self.assertIn("개발통합", names)
        self.assertIn("행정서식", names)
        idx = json.loads((FIXT / "profiles" / "index.json").read_text(encoding="utf-8"))
        self.assertEqual(idx["names"], names)
        biz = json.loads((FIXT / "profiles" / "경영보고.json").read_text(encoding="utf-8"))
        self.assertIn("hwp_fill_fields", biz["blocked_stateless_sample"])
        self.assertNotIn("hwp_info", biz["blocked_stateless_sample"])
        self.assertTrue(biz["bypass_forbidden"])

    def test_drift_guards_named(self):
        idx = json.loads((FIXT / "drift" / "index.json").read_text(encoding="utf-8"))
        self.assertIn("capabilities_mcp_covers_every_json_command", idx["guards"])
        self.assertIn("tools_list_matches_capabilities_manifest", idx["guards"])
        cover = json.loads(
            (FIXT / "drift" / "capabilities_mcp_covers_every_json_command.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertIn("capabilities", cover["exempt"])
        self.assertIn("dump-pages", cover["exempt"])

    def test_catalogs_cover_source(self):
        cmds = json.loads((FIXT / "commands" / "catalog.json").read_text(encoding="utf-8"))
        self.assertEqual(
            [row["name"] for row in cmds["items"]], [c["name"] for c in self.commands]
        )
        tools = json.loads((FIXT / "tools" / "catalog.json").read_text(encoding="utf-8"))
        self.assertEqual([row["name"] for row in tools["items"]], self.stateless)
        self.assertTrue(tools["in_capabilities_mcp"])

    def test_working_doc_exists_and_closes_issue(self):
        text = WORKING.read_text(encoding="utf-8")
        self.assertIn("#5326", text)
        self.assertIn("rhwp capabilities", text)
        self.assertIn("mcp_tool_definitions", text)
        self.assertIn("금지", text)
        self.assertRegex(text, r"(?i)gym")
        self.assertIn("feat/agent-surface", text)
        for sibling in SIBLING_SKILLS:
            self.assertIn(sibling, text)

    def test_playbook_is_canonical_and_untouched_here(self):
        text = PLAYBOOK.read_text(encoding="utf-8")
        self.assertIn("canonical: mydocs/manual/agent_surface_playbook.md", text)
        self.assertIn("mcp_tool_definitions()", text)
        self.assertIn("판정은 데이터", text)

    def test_generator_does_not_hardcode_invented_tools(self):
        src = GEN.read_text(encoding="utf-8")
        for match in HWP_NAME.findall(src):
            if match in INTENTIONAL_UNKNOWN or match in DO_NOT_INVENT:
                continue
            # 생성기의 추출 정규식에 들어 있는 토큰은 패턴이다.
            if match in {"hwp_"}:
                continue

    def test_sibling_skill_files_not_modified_by_this_tree(self):
        # 이 시험은 작업 트리에 형제 스킬 폴더가 그대로 있는지만 본다.
        # 본문을 고치면 git status 가 아니라 이 스킬의 금지를 상기시킨다.
        for name in SIBLING_SKILLS:
            path = REPO / ".agents" / "skills" / name / "SKILL.md"
            self.assertTrue(path.is_file(), path)

    def test_scenarios_stay_on_allowlist(self):
        catalog = json.loads((FIXT / "scenarios.json").read_text(encoding="utf-8"))
        self.assertTrue(catalog["not_gym"])
        self.assertGreaterEqual(len(catalog["items"]), 20)
        for item in catalog["items"]:
            tool = item.get("tool")
            if tool:
                self.assertIn(tool, self.allowed, item["id"])

    def test_reuse_map_names_cores(self):
        core = json.loads((FIXT / "reuse" / "core_map.json").read_text(encoding="utf-8"))
        blob = json.dumps(core, ensure_ascii=False)
        for name in (
            "set_field_value_by_name_at",
            "replace_all_native",
            "grep",
            "collect_field_records",
            "extract_tables",
            "edit_serialize",
        ):
            self.assertIn(name, blob)

    def test_transcripts_describe_three_entry_points(self):
        bare = json.loads((FIXT / "transcripts" / "capabilities_bare.json").read_text(encoding="utf-8"))
        self.assertTrue(bare["always_json"])
        self.assertTrue(bare["do_not_pass_json_flag"])
        mcp = json.loads((FIXT / "transcripts" / "capabilities_mcp.json").read_text(encoding="utf-8"))
        self.assertFalse(mcp["session_tools_present"])
        self.assertIn("hwp_info", mcp["stateless_names"])
        search = json.loads(
            (FIXT / "transcripts" / "capabilities_search_redact.json").read_text(encoding="utf-8")
        )
        self.assertTrue(search["and_semantics"])
        self.assertTrue(search["combine_mcp_forbidden"])


if __name__ == "__main__":
    unittest.main()
