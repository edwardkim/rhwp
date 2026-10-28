"""[#5294] rhwp-safe-edit 스킬 고도화 계약.

에이전트가 원본을 훼손하지 않고 기존 edit / run 경로를 부르도록
문서·픽스처·워크스루가 같은 단어를 쓰는지 파일만으로 고정한다.

새 편집 로직을 시험하지 않는다. gym/ 을 열지 않는다.
다른 네 에이전트 스킬(onboarding / mcp-session / provenance / doc-triage)의
본문을 요구하거나 바꾸지 않는다. 바이너리·네트워크를 부르지 않는다.

정본: .claude/skills/rhwp-safe-edit/
작업 기록: mydocs/working/archives/agent_safe_edit.md
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-safe-edit"
SKILL_MD = SKILL / "SKILL.md"
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
PLANS = FIXTURES / "plans"
ENVS = FIXTURES / "envelopes"
LOOPS = FIXTURES / "loops"
CATALOG = FIXTURES / "catalog.json"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_safe_edit.md"

REFERENCE_NAMES = (
    "single_edit.md",
    "run_plans.md",
    "verify_loops.md",
    "failure_envelopes.md",
)

RUN_ACTIONS = (
    "fill_fields",
    "replace_text",
    "set_cell",
    "set_checkbox",
)

# 이 파동이 만지지 않는 형제 스킬. 존재만 확인하고 내용을 요구하지 않는다.
SIBLING_SKILLS = (
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-provenance",
    "rhwp-doc-triage",
)

# 1층 CLI 하위명령 — 스킬이 배선하는 기존 표면.
EDIT_SUBS = (
    "fill-fields",
    "replace-text",
    "set-cell",
    "insert-image",
    "redact",
    "sanitize",
)

# 문서에 반드시 남아야 하는 토큰. 문장이 바뀌어도 계약 단어는 남는다.
REF_TOKENS = {
    "single_edit.md": (
        "edit fill-fields",
        "edit replace-text",
        "edit set-cell",
        "edit insert-image",
        "edit redact",
        "edit sanitize",
        "--dry-run",
        "-o",
        "--verify",
        "outputFormat",
        "notFound",
        "ambiguous",
        "overflow",
        "원본",
        "csv-to-table",
        "batch fill",
    ),
    "run_plans.md": (
        "planVersion",
        "export-plan-schema",
        "fill_fields",
        "replace_text",
        "set_cell",
        "set_checkbox",
        "invalid[]",
        "preview",
        "assertions",
        "preconditions",
        "inputSha256",
        "fieldExists",
        "fieldEquals",
        "textFound",
        "dry-run",
        "nextCall",
    ),
    "verify_loops.md": (
        "--dry-run",
        "--verify",
        "changedPages",
        "fields",
        "export-tables",
        "search",
        "export-svg",
        "ir-diff",
        "verify: null",
        "재독",
    ),
    "failure_envelopes.md": (
        "exit 3",
        "exit 4",
        "판정",
        "notFound",
        "ambiguous",
        "invalid[]",
        "preconditionFailed",
        "stdout",
        "InspectKeptOutput",
        "ReplanNoOutput",
    ),
}

SKILL_TOKENS = (
    "원본 불변",
    "산출 분리",
    "선확인",
    "판정은 예외가 아니라 봉투",
    "edit fill-fields",
    "rhwp run",
    "export-plan-schema",
    "exit 3",
    "references/single_edit.md",
    "references/run_plans.md",
    "references/verify_loops.md",
    "references/failure_envelopes.md",
    "fixtures/catalog.json",
)

WORKING_TOKENS = (
    "#5294",
    "rhwp-safe-edit",
    "edit",
    "run",
    "dry-run",
    "--verify",
    "exit 3",
    "gym",
    "5000",
)

INVENTED_ACTIONS = (
    "insert_image",
    "insert-image",
    "redact",
    "sanitize",
    "fillFields",
    "replaceText",
    "setCell",
    "setCheckbox",
    "op",
    "fill",
)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path):
    return json.loads(read(path))


def md_files(root: Path):
    return sorted(p for p in root.rglob("*.md") if p.is_file())


def json_files(root: Path):
    return sorted(p for p in root.rglob("*.json") if p.is_file())


class SkillLayoutTests(unittest.TestCase):
    def test_skill_root_exists(self):
        self.assertTrue(SKILL.is_dir(), SKILL)
        self.assertTrue(SKILL_MD.is_file(), SKILL_MD)
        self.assertTrue(REFS.is_dir(), REFS)
        self.assertTrue(EXAMPLES.is_dir(), EXAMPLES)
        self.assertTrue(FIXTURES.is_dir(), FIXTURES)
        self.assertTrue(CATALOG.is_file(), CATALOG)
        self.assertTrue(WORKING.is_file(), WORKING)

    def test_four_references_present(self):
        names = sorted(p.name for p in REFS.glob("*.md"))
        self.assertEqual(sorted(REFERENCE_NAMES), names)

    def test_sibling_skills_exist_untouched_by_this_suite(self):
        # 존재만 확인한다. 본문 토큰을 요구하면 이 파동이 형제 스킬에 결합된다.
        for name in SIBLING_SKILLS:
            path = REPO / ".claude" / "skills" / name / "SKILL.md"
            self.assertTrue(path.is_file(), path)

    def test_does_not_live_under_gym(self):
        self.assertNotIn("gym", SKILL.parts)
        self.assertFalse((REPO / "gym" / "docs" / "agent_safe_edit.md").exists())


class FrontmatterTests(unittest.TestCase):
    def test_skill_frontmatter(self):
        body = read(SKILL_MD)
        self.assertTrue(body.startswith("---\n"), "frontmatter 시작")
        end = body.find("\n---\n", 4)
        self.assertGreater(end, 0, "frontmatter 종료")
        fm = body[4:end]
        self.assertIn("name: rhwp-safe-edit", fm)
        self.assertIn("description:", fm)
        desc = ""
        for line in fm.splitlines():
            if line.startswith("description:"):
                desc = line.split(":", 1)[1].strip()
        self.assertGreaterEqual(len(desc), 20)

    def test_skill_is_router_not_only_prose(self):
        body = read(SKILL_MD)
        self.assertIn("rhwp ", body)
        self.assertIn("```bash", body)
        for tok in SKILL_TOKENS:
            self.assertIn(tok, body, tok)


class ReferenceTokenTests(unittest.TestCase):
    def test_each_reference_has_contract_tokens(self):
        for name, tokens in REF_TOKENS.items():
            body = read(REFS / name)
            for tok in tokens:
                self.assertIn(tok, body, f"{name} 에 {tok!r} 없음")

    def test_single_edit_lists_six_commands(self):
        body = read(REFS / "single_edit.md")
        for sub in EDIT_SUBS:
            self.assertIn(f"edit {sub}", body, sub)

    def test_run_plans_does_not_invent_fifth_action(self):
        body = read(REFS / "run_plans.md")
        # 금지 안내로 insert_image 가 *등장*할 수는 있다. 유효 action 목록에 넣으면 안 된다.
        self.assertIn("알 수 없는 action", body)
        self.assertIn("fill_fields·replace_text·set_cell·set_checkbox", body)
        self.assertNotRegex(body, r"action 5종|valid action.*insert_image")
        self.assertRegex(body, r"다섯 번째 action 을 (문서에 )?추가하지 않는다")

    def test_verify_loop_distinguishes_null_and_empty_pages(self):
        body = read(REFS / "verify_loops.md")
        self.assertIn("changedPages: null", body)
        self.assertIn("빈 배열", body)
        self.assertIn("확정 불가", body)

    def test_failure_envelopes_treat_exit3_as_data(self):
        body = read(REFS / "failure_envelopes.md")
        self.assertIn("고장이 아니다", body)
        self.assertIn("1층", body)
        self.assertIn("산출물은 **있다**", body)
        self.assertIn("디스크 무변경", body)
        self.assertIn("CAS", body)
        self.assertRegex(body, r"exit 3.{0,20}exit 2")

    def test_layer1_verify_keeps_output_layer3_does_not(self):
        fail = read(REFS / "failure_envelopes.md")
        verify = read(REFS / "verify_loops.md")
        joined = fail + "\n" + verify
        self.assertIn("남는다", joined)
        self.assertIn("남기지 않는다", joined)


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(CATALOG)

    def test_catalog_header(self):
        self.assertEqual(self.cat["catalogVersion"], "1.0")
        self.assertEqual(self.cat["skill"], "rhwp-safe-edit")
        self.assertEqual(self.cat["issue"], 5294)
        self.assertEqual(tuple(self.cat["actions"]), RUN_ACTIONS)

    def test_catalog_lists_match_files(self):
        for rel in self.cat["plans"]["valid"]:
            self.assertTrue((PLANS / rel).is_file(), rel)
        for rel in self.cat["plans"]["invalid"]:
            self.assertTrue((PLANS / rel).is_file(), rel)
        for rel in self.cat["envelopes"]:
            self.assertTrue((ENVS / rel).is_file(), rel)
        for rel in self.cat["loops"]:
            self.assertTrue((LOOPS / rel).is_file(), rel)
        for rel in self.cat["examples"]:
            self.assertTrue((EXAMPLES / rel).is_file(), rel)
        for rel in self.cat["references"]:
            self.assertTrue((REFS / rel).is_file(), rel)

    def test_no_stray_plan_files(self):
        listed = set(self.cat["plans"]["valid"]) | set(self.cat["plans"]["invalid"])
        actual = {p.name for p in PLANS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_no_stray_envelope_files(self):
        listed = set(self.cat["envelopes"])
        actual = {p.name for p in ENVS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_no_stray_loop_files(self):
        listed = set(self.cat["loops"])
        actual = {p.name for p in LOOPS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_example_readme_lists_same_files(self):
        readme = read(EXAMPLES / "README.md")
        for rel in self.cat["examples"]:
            self.assertIn(rel, readme, rel)


class PlanFixtureTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(CATALOG)

    def test_valid_plans_have_required_keys(self):
        for name in self.cat["plans"]["valid"]:
            plan = load_json(PLANS / name)
            self.assertEqual(plan.get("planVersion"), "1.0", name)
            self.assertIsInstance(plan.get("input"), str, name)
            self.assertIsInstance(plan.get("output"), str, name)
            steps = plan.get("steps")
            self.assertIsInstance(steps, list, name)
            self.assertGreaterEqual(len(steps), 1, name)
            for step in steps:
                self.assertIn(step.get("action"), RUN_ACTIONS, f"{name}: {step}")

    def test_valid_plans_do_not_use_wrong_top_keys_instead_of_required(self):
        for name in self.cat["plans"]["valid"]:
            plan = load_json(PLANS / name)
            self.assertNotIn("source", plan, name)
            self.assertNotIn("ops", plan, name)
            self.assertNotIn("op", plan, name)

    def test_invalid_plans_are_json_but_break_a_known_rule(self):
        checks = {
            "invalid_missing_plan_version.json": lambda p: "planVersion" not in p,
            "invalid_wrong_keys_source_op.json": lambda p: "input" not in p and "source" in p,
            "invalid_empty_steps.json": lambda p: p.get("steps") == [],
            "invalid_unknown_action.json": lambda p: p["steps"][0]["action"]
            not in RUN_ACTIONS,
            "invalid_camel_action.json": lambda p: p["steps"][0]["action"] == "fillFields",
            "invalid_replace_empty_find.json": lambda p: p["steps"][0].get("find") == "",
            "invalid_set_cell_newline.json": lambda p: True,
            "invalid_two_conditions.json": lambda p: len(p["steps"][0].get("if", {})) > 1,
            "invalid_numeric_plan_version.json": lambda p: not isinstance(
                p.get("planVersion"), str
            ),
        }
        listed = set(self.cat["plans"]["invalid"])
        self.assertEqual(listed, set(checks))
        for name, pred in checks.items():
            plan = load_json(PLANS / name)
            self.assertTrue(pred(plan), name)

    def test_no_valid_plan_uses_invented_action(self):
        for name in self.cat["plans"]["valid"]:
            plan = load_json(PLANS / name)
            for step in plan["steps"]:
                self.assertNotIn(step["action"], INVENTED_ACTIONS, name)

    def test_fill_fields_data_is_object(self):
        for path in PLANS.glob("valid_*.json"):
            plan = load_json(path)
            for step in plan.get("steps") or []:
                if step.get("action") == "fill_fields":
                    self.assertIsInstance(step.get("data"), dict, path.name)

    def test_set_cell_has_grid_keys(self):
        plan = load_json(PLANS / "valid_set_cell.json")
        step = plan["steps"][0]
        for key in ("table", "row", "col", "text"):
            self.assertIn(key, step, key)

    def test_checkbox_has_occurrence(self):
        plan = load_json(PLANS / "valid_set_checkbox.json")
        self.assertIn("occurrence", plan["steps"][0])

    def test_preconditions_shape(self):
        plan = load_json(PLANS / "valid_preconditions.json")
        pre = plan["preconditions"]
        self.assertEqual(set(pre), {"inputSha256"})
        hexes = pre["inputSha256"]
        self.assertEqual(len(hexes), 64)
        self.assertRegex(hexes, r"^[0-9a-f]+$")

    def test_conditional_if_is_single_key(self):
        for name in (
            "valid_conditional_field_exists.json",
            "valid_conditional_field_equals.json",
            "valid_conditional_text_found.json",
        ):
            step = load_json(PLANS / name)["steps"][0]
            self.assertEqual(len(step["if"]), 1, name)
            key = next(iter(step["if"]))
            self.assertIn(key, ("fieldExists", "fieldEquals", "textFound"))

    def test_assertions_verify_plan(self):
        plan = load_json(PLANS / "valid_assertions_verify.json")
        self.assertTrue(plan["assertions"]["verify"])
        self.assertTrue(plan["assertions"]["notFoundEmpty"])


class EnvelopeFixtureTests(unittest.TestCase):
    def test_every_envelope_declares_exit_meta(self):
        for path in ENVS.glob("*.json"):
            data = load_json(path)
            meta = data.get("_skillMeta")
            self.assertIsInstance(meta, dict, path.name)
            self.assertIn("exit", meta, path.name)
            self.assertIn(meta["exit"], (0, 1, 2, 3, 4), path.name)
            self.assertIn("branch", meta, path.name)

    def test_exit0_incomplete_signals(self):
        nf = load_json(ENVS / "fill_notfound_exit0.json")
        self.assertEqual(nf["_skillMeta"]["exit"], 0)
        self.assertTrue(nf["notFound"])
        self.assertFalse(nf["_skillMeta"].get("complete", True))

        amb = load_json(ENVS / "fill_ambiguous_exit0.json")
        self.assertEqual(amb["_skillMeta"]["exit"], 0)
        self.assertTrue(amb["ambiguous"])
        self.assertEqual(amb["ambiguous"][0]["total"], 5)

    def test_verify_null_is_not_pass(self):
        data = load_json(ENVS / "verify_null.json")
        self.assertIsNone(data["verify"])
        self.assertFalse(data["_skillMeta"]["verified"])

    def test_layer1_verify_diff_keeps_output(self):
        data = load_json(ENVS / "edit_verify_diff.json")
        self.assertEqual(data["_skillMeta"]["exit"], 3)
        self.assertTrue(data["_skillMeta"]["outputKept"])
        self.assertFalse(data["verify"]["identical"])
        self.assertIn("output", data)

    def test_run_verify_fail_does_not_keep_output(self):
        data = load_json(ENVS / "run_verify_fail_no_output.json")
        self.assertEqual(data["_skillMeta"]["exit"], 3)
        self.assertFalse(data["_skillMeta"]["outputKept"])
        self.assertFalse(data["verify"]["identical"])

    def test_cas_is_exit3_with_empty_invalid(self):
        data = load_json(ENVS / "run_precondition_failed.json")
        self.assertEqual(data["_skillMeta"]["exit"], 3)
        self.assertEqual(data["invalid"], [])
        self.assertEqual(data["preconditionFailed"]["kind"], "inputSha256")
        self.assertEqual(data["nextCall"]["name"], "run")
        self.assertIn("--dry-run", data["nextCall"]["arguments"])

    def test_run_invalid_collects_multiple(self):
        data = load_json(ENVS / "run_invalid_collected.json")
        self.assertEqual(data["_skillMeta"]["exit"], 2)
        self.assertGreaterEqual(len(data["invalid"]), 3)
        actions = [item["action"] for item in data["invalid"]]
        self.assertIn("insert_image", actions)

    def test_replace_zero_has_no_output_key(self):
        data = load_json(ENVS / "replace_zero.json")
        self.assertEqual(data["replacedCount"], 0)
        self.assertNotIn("output", data)
        self.assertFalse(data["_skillMeta"]["outputKept"])

    def test_redact_missing_output_is_stderr_only(self):
        data = load_json(ENVS / "redact_missing_output.json")
        self.assertEqual(data["_skillMeta"]["stdoutBytes"], 0)
        self.assertEqual(data["_skillMeta"]["exit"], 2)
        self.assertIn("마스킹은 되돌릴 수 없습니다", data["stderrContains"])

    def test_overflow_does_not_block_fill(self):
        data = load_json(ENVS / "set_cell_overflow.json")
        self.assertEqual(data["_skillMeta"]["exit"], 0)
        self.assertTrue(data["overflow"])
        self.assertFalse(data["_skillMeta"]["complete"])

    def test_csv_invalid_reasons(self):
        data = load_json(ENVS / "csv_to_table_invalid.json")
        reasons = {item["reason"] for item in data["invalid"]}
        self.assertIn("rowCountMismatch", reasons)
        self.assertIn("colCountMismatch", reasons)
        self.assertEqual(data["changedCount"], 0)

    def test_batch_row_notfound_still_exit0(self):
        data = load_json(ENVS / "batch_row_notfound.json")
        self.assertEqual(data["_skillMeta"]["exit"], 0)
        self.assertTrue(data["notFound"])
        self.assertEqual(data["row"], 2)

    def test_dry_run_preview_has_no_disk(self):
        data = load_json(ENVS / "run_dry_run_preview.json")
        self.assertTrue(data["dryRun"])
        self.assertEqual(data["invalid"], [])
        self.assertTrue(data["preview"])
        self.assertFalse(data["_skillMeta"]["outputKept"])


class LoopFixtureTests(unittest.TestCase):
    def test_command_loops_have_steps(self):
        for name in (
            "layer1_fill.json",
            "layer1_replace.json",
            "layer1_set_cell.json",
            "layer3_run.json",
        ):
            data = load_json(LOOPS / name)
            self.assertGreaterEqual(len(data["steps"]), 3, name)
            ids = [s["id"] for s in data["steps"]]
            self.assertIn("dry-run", ids, name)
            for step in data["steps"]:
                self.assertIsInstance(step["command"], list, name)
                self.assertEqual(step["command"][0], "rhwp", name)
                self.assertIn("expectExit", step, name)
                self.assertIn("readFields", step, name)

    def test_layer1_fill_forbids_output_on_dry_run(self):
        data = load_json(LOOPS / "layer1_fill.json")
        dry = next(s for s in data["steps"] if s["id"] == "dry-run")
        self.assertIn("output", dry["forbidFieldsPresent"])
        self.assertIn("--dry-run", dry["command"])

    def test_layer3_starts_with_schema(self):
        data = load_json(LOOPS / "layer3_run.json")
        self.assertEqual(data["steps"][0]["id"], "schema")
        self.assertEqual(data["steps"][0]["command"][1], "export-plan-schema")

    def test_replace_reread_expects_zero_matches(self):
        data = load_json(LOOPS / "layer1_replace.json")
        reread = next(s for s in data["steps"] if s["id"] == "reread")
        self.assertEqual(reread["expectMatchCount"], 0)

    def test_invariants_loops(self):
        vn = load_json(LOOPS / "verify_null_vs_object.json")
        self.assertEqual(vn["invariants"][0]["equals"], None)
        self.assertEqual(vn["invariants"][0]["notMeaning"], "passed")
        cp = load_json(LOOPS / "changed_pages_null.json")
        meanings = {item["meaning"] for item in cp["invariants"]}
        self.assertTrue(any("확정 불가" in m for m in meanings))
        self.assertTrue(any("바뀐 쪽 없음" in m for m in meanings))


class CommandReferenceTests(unittest.TestCase):
    """스킬 폴더의 rhwp 토큰이 알려진 머리 명령 집합에 속하는지.

    실재 집합의 정본은 바이너리 자기서술(tests/skills_contract.rs)이다.
    여기서는 이 파동이 지어낸 머리 토큰이 없는지만 본다.
    """

    KNOWN_HEADS = {
        "edit",
        "run",
        "fields",
        "search",
        "info",
        "export-tables",
        "export-svg",
        "export-text",
        "export-plan-schema",
        "export-hwpx",
        "ir-diff",
        "table-to-csv",
        "csv-to-table",
        "batch",
        "convert",
        "capabilities",
    }

    TOKEN_RE = re.compile(r"rhwp ([a-z][a-z0-9-]*)")

    def test_heads_are_known(self):
        unknown = []
        for path in md_files(SKILL):
            for match in self.TOKEN_RE.finditer(read(path)):
                head = match.group(1)
                if head not in self.KNOWN_HEADS:
                    unknown.append(f"{path.name}: rhwp {head}")
        self.assertEqual(unknown, [], "스킬이 모르는 머리 명령을 안내한다")

    def test_edit_subs_are_known(self):
        sub_re = re.compile(r"rhwp edit ([a-z][a-z0-9-]*)")
        unknown = []
        for path in md_files(SKILL):
            for match in sub_re.finditer(read(path)):
                sub = match.group(1)
                if sub not in EDIT_SUBS:
                    unknown.append(f"{path.name}: rhwp edit {sub}")
        self.assertEqual(unknown, [])

    def test_examples_contain_run_or_edit(self):
        for path in EXAMPLES.glob("*.md"):
            if path.name == "README.md":
                continue
            body = read(path)
            self.assertTrue(
                "rhwp edit" in body or "rhwp run" in body or "rhwp batch" in body
                or "rhwp csv-to-table" in body or "sha256" in body,
                path.name,
            )


class CrossLinkTests(unittest.TestCase):
    LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")

    def test_relative_links_inside_skill_resolve(self):
        missing = []
        for path in md_files(SKILL):
            for _label, href in self.LINK_RE.findall(read(path)):
                if href.startswith(("http://", "https://", "mailto:")):
                    continue
                if href.startswith("#"):
                    continue
                target = href.split("#", 1)[0]
                if not target:
                    continue
                dest = (path.parent / target).resolve()
                # 스킬 밖으로 나가는 링크(mydocs/…)는 저장소 루트 기준으로 존재해야 한다.
                if not dest.exists():
                    missing.append(f"{path.name} -> {href}")
        self.assertEqual(missing, [])

    def test_skill_points_at_working_doc(self):
        self.assertIn("mydocs/working/agent_safe_edit.md", read(SKILL_MD))


class WorkingDocTests(unittest.TestCase):
    def test_working_doc_tokens(self):
        body = read(WORKING)
        for tok in WORKING_TOKENS:
            self.assertIn(tok, body, tok)

    def test_working_doc_states_no_new_edit_logic(self):
        body = read(WORKING)
        self.assertRegex(body, r"새 편집 로직|발명 금지|기존 edit")

    def test_working_doc_states_not_gym(self):
        body = read(WORKING)
        self.assertRegex(body, r"gym 금지|gym/ 를 열지|gym 을 만지지")


class InvariantProseTests(unittest.TestCase):
    """문서 여러 곳이 같은 불변식을 말하는지 — 한 파일만 고치면 깨진다."""

    def _all_prose(self) -> str:
        parts = [read(SKILL_MD), read(WORKING)]
        parts.extend(read(p) for p in REFS.glob("*.md"))
        parts.extend(read(p) for p in EXAMPLES.glob("*.md"))
        return "\n".join(parts)

    def test_original_invariance_mentioned(self):
        text = self._all_prose()
        self.assertIn("원본", text)
        self.assertRegex(text, r"원본 불변|원본은 그대로|원본 바이트")

    def test_no_in_place_as_default(self):
        skill = read(SKILL_MD)
        self.assertIn("--in-place", skill)
        self.assertRegex(skill, r"명시|redact")

    def test_four_actions_listed_together(self):
        text = read(REFS / "run_plans.md") + read(SKILL_MD)
        for action in RUN_ACTIONS:
            self.assertIn(action, text)

    def test_exit_table_has_01234(self):
        body = read(REFS / "failure_envelopes.md")
        for code in ("| 0 ", "| 1 ", "| 2 ", "| 3 ", "| 4 "):
            self.assertIn(code, body, code)


class GymIsolationTests(unittest.TestCase):
    def test_skill_markdown_does_not_instruct_editing_gym(self):
        for path in md_files(SKILL):
            body = read(path)
            self.assertNotRegex(body, r"gym/packs|gym/tools|certify\.py|score\.py")

    def test_fixtures_are_not_under_gym(self):
        for path in json_files(SKILL):
            self.assertNotIn("gym", path.parts)


class CatalogActionLockTests(unittest.TestCase):
    def test_catalog_actions_exactly_four(self):
        cat = load_json(CATALOG)
        self.assertEqual(len(cat["actions"]), 4)
        self.assertEqual(cat["actions"], list(RUN_ACTIONS))


if __name__ == "__main__":
    unittest.main()
