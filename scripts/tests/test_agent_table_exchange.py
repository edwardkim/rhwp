"""[#5306] rhwp-table-exchange 스킬 고도화 계약.

실사용 에이전트가 표↔CSV 왕복을 기존 CLI 만으로 닫도록
문서·픽스처·워크스루가 같은 단어를 쓰는지 파일만으로 고정한다.

새 CLI 를 시험하지 않는다. DocumentCore 편집 로직을 시험하지 않는다.
gym/ 을 열지 않는다. 바이너리·네트워크를 부르지 않는다.

정본: .claude/skills/rhwp-table-exchange/
작업 기록: mydocs/working/archives/agent_table_exchange.md
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-table-exchange"
SKILL_MD = SKILL / "SKILL.md"
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
ENVS = FIXTURES / "envelopes"
LOOPS = FIXTURES / "loops"
MATS = FIXTURES / "matrices"
TRANS = FIXTURES / "transcripts"
CSV = FIXTURES / "csv"
DECS = FIXTURES / "decisions"
CATALOG = FIXTURES / "catalog.json"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_table_exchange.md"

REFERENCE_NAMES = (
    "export_tables_matrix.md",
    "table_to_csv_envelopes.md",
    "csv_to_table_contract.md",
    "dry_run_verify.md",
    "merged_table_fallback.md",
    "pitfalls.md",
    "failure_envelopes.md",
    "sample_transcripts.md",
    "coordinate_index.md",
)

SIBLING_SKILLS = (
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-safe-edit",
    "rhwp-form-fill",
)

KNOWN_HEADS = {
    "export-tables",
    "table-to-csv",
    "csv-to-table",
    "edit",
    "batch",
    "find",
    "info",
    "fields",
    "search",
    "ir-diff",
    "export-svg",
}

EDIT_SUBS = {"set-cell"}

INVALID_REASONS = (
    "rowCountMismatch",
    "colCountMismatch",
    "coveredCellNotEmpty",
    "controlCharacter",
    "csvParse",
)

REF_TOKENS = {
    "export_tables_matrix.md": (
        "export-tables",
        "tables[].index",
        "rowSpan",
        "colSpan",
        "containerPath",
        "cellCount",
        "samples/table-001.hwp",
        "samples/inner-table-01.hwp",
        "samples/basic/treatise sample.hwp",
        "untrustedFields",
    ),
    "table_to_csv_envelopes.md": (
        "table-to-csv",
        "--table",
        "--bom",
        "tables[].csv",
        "outputFormat",
        "U+FEFF",
        "RFC 4180",
        "untrustedContent",
    ),
    "csv_to_table_contract.md": (
        "csv-to-table",
        "rowCountMismatch",
        "colCountMismatch",
        "coveredCellNotEmpty",
        "controlCharacter",
        "csvParse",
        "changedCount",
        "invalid[]",
        "한 칸도",
    ),
    "dry_run_verify.md": (
        "--dry-run",
        "--verify",
        "changedPages",
        "identical",
        "exit 3",
        "exit 2",
        "판정",
    ),
    "merged_table_fallback.md": (
        "edit set-cell",
        "앵커",
        "덮인",
        "--keep-style",
        "overflow",
        "발명하지",
    ),
    "pitfalls.md": (
        "--bom",
        "0행",
        "중첩",
        "v1",
        "untrusted",
        "1×1",
    ),
    "failure_envelopes.md": (
        "exit 3",
        "exit 2",
        "stdout",
        "invalid[]",
        "0바이트",
        "DATA",
    ),
    "sample_transcripts.md": (
        "samples/hwp_table_test.hwp",
        "changedCount",
        "issue2007",
        "table-001",
        "제목,담당자,세부 내용",
    ),
    "coordinate_index.md": (
        "index",
        "containerPath",
        "0부터 시작하지",
        "--table",
        "hwp_export_tables",
    ),
}

SKILL_TOKENS = (
    "새 CLI",
    "export-tables",
    "table-to-csv",
    "--table",
    "--bom",
    "csv-to-table",
    "--dry-run",
    "--verify",
    "exit 3",
    "edit set-cell",
    "coveredCellNotEmpty",
    "controlCharacter",
    "rowCountMismatch",
    "gym",
    "references/export_tables_matrix.md",
    "fixtures/catalog.json",
)

WORKING_TOKENS = (
    "#5306",
    "rhwp-table-exchange",
    "export-tables",
    "table-to-csv",
    "csv-to-table",
    "dry-run",
    "--verify",
    "exit 3",
    "gym",
    "5000",
    "set-cell",
)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path):
    return json.loads(read(path))


def md_files(root: Path):
    return sorted(p for p in root.rglob("*.md") if p.is_file())


class SkillLayoutTests(unittest.TestCase):
    def test_skill_root_exists(self):
        self.assertTrue(SKILL.is_dir(), SKILL)
        self.assertTrue(SKILL_MD.is_file(), SKILL_MD)
        self.assertTrue(REFS.is_dir(), REFS)
        self.assertTrue(EXAMPLES.is_dir(), EXAMPLES)
        self.assertTrue(FIXTURES.is_dir(), FIXTURES)
        self.assertTrue(CATALOG.is_file(), CATALOG)
        self.assertTrue(WORKING.is_file(), WORKING)

    def test_nine_references_present(self):
        names = sorted(p.name for p in REFS.glob("*.md"))
        self.assertEqual(sorted(REFERENCE_NAMES), names)

    def test_sibling_skills_exist_untouched_by_this_suite(self):
        for name in SIBLING_SKILLS:
            path = REPO / ".claude" / "skills" / name / "SKILL.md"
            self.assertTrue(path.is_file(), path)

    def test_does_not_live_under_gym(self):
        self.assertNotIn("gym", SKILL.parts)
        self.assertFalse((REPO / "gym" / "docs" / "agent_table_exchange.md").exists())


class FrontmatterTests(unittest.TestCase):
    def test_skill_frontmatter(self):
        body = read(SKILL_MD)
        self.assertTrue(body.startswith("---\n"), "frontmatter 시작")
        end = body.find("\n---\n", 4)
        self.assertGreater(end, 0, "frontmatter 종료")
        fm = body[4:end]
        self.assertIn("name: rhwp-table-exchange", fm)
        desc = ""
        for line in fm.splitlines():
            if line.startswith("description:"):
                desc = line.split(":", 1)[1].strip()
        self.assertGreaterEqual(len(desc), 20)

    def test_skill_is_router(self):
        body = read(SKILL_MD)
        self.assertIn("rhwp ", body)
        self.assertIn("```bash", body)
        for tok in SKILL_TOKENS:
            self.assertIn(tok, body, tok)

    def test_skill_forbids_new_cli_and_gym(self):
        body = read(SKILL_MD)
        self.assertIn("새 CLI 를 만들지 않는다", body)
        self.assertIn("발명하지 않는다", body)
        self.assertRegex(body, r"gym")


class ReferenceTokenTests(unittest.TestCase):
    def test_each_reference_has_contract_tokens(self):
        for name, tokens in REF_TOKENS.items():
            body = read(REFS / name)
            for tok in tokens:
                self.assertIn(tok, body, f"{name} 에 {tok!r} 없음")

    def test_matrix_uses_real_samples(self):
        body = read(REFS / "export_tables_matrix.md")
        for sample in (
            "samples/hwp_table_test.hwp",
            "samples/table-001.hwp",
            "samples/inner-table-01.hwp",
            "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx",
        ):
            self.assertIn(sample, body, sample)

    def test_fallback_does_not_invent_merge_writer(self):
        body = read(REFS / "merged_table_fallback.md")
        self.assertIn("edit set-cell", body)
        self.assertIn("발명하지 않는다", body)
        self.assertNotRegex(body, r"rhwp merge-cells|rhwp split-cell|rhwp insert-row")

    def test_dry_run_treats_exit3_as_data(self):
        body = read(REFS / "dry_run_verify.md")
        self.assertIn("고장이 아니다", body)
        self.assertIn("changedPages: null", body)
        self.assertIn("산출물을 **남긴다**", body)


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(CATALOG)

    def test_catalog_header(self):
        self.assertEqual(self.cat["catalogVersion"], "1.0")
        self.assertEqual(self.cat["skill"], "rhwp-table-exchange")
        self.assertEqual(self.cat["issue"], 5306)
        self.assertEqual(
            self.cat["commands"],
            ["export-tables", "table-to-csv", "csv-to-table", "edit set-cell"],
        )
        self.assertEqual(tuple(self.cat["invalidReasons"]), INVALID_REASONS)

    def test_catalog_lists_match_files(self):
        for rel in self.cat["envelopes"]:
            self.assertTrue((ENVS / rel).is_file(), rel)
        for rel in self.cat["loops"]:
            self.assertTrue((LOOPS / rel).is_file(), rel)
        for rel in self.cat["matrices"]:
            self.assertTrue((MATS / rel).is_file(), rel)
        for rel in self.cat["transcripts"]:
            self.assertTrue((TRANS / rel).is_file(), rel)
        for rel in self.cat["csv"]:
            self.assertTrue((CSV / rel).is_file(), rel)
        for rel in self.cat["examples"]:
            self.assertTrue((EXAMPLES / rel).is_file(), rel)
        for rel in self.cat["references"]:
            self.assertTrue((REFS / rel).is_file(), rel)

    def test_no_stray_envelope_files(self):
        listed = set(self.cat["envelopes"])
        actual = {p.name for p in ENVS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_no_stray_loop_files(self):
        listed = set(self.cat["loops"])
        actual = {p.name for p in LOOPS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_no_stray_example_files(self):
        listed = set(self.cat["examples"]) | {"README.md"}
        actual = {p.name for p in EXAMPLES.glob("*.md")}
        self.assertEqual(listed, actual)

    def test_example_readme_lists_same_files(self):
        readme = read(EXAMPLES / "README.md")
        for rel in self.cat["examples"]:
            self.assertIn(rel, readme, rel)


class EnvelopeFixtureTests(unittest.TestCase):
    def test_every_envelope_declares_exit_meta(self):
        for path in ENVS.glob("*.json"):
            data = load_json(path)
            meta = data.get("_skillMeta")
            self.assertIsInstance(meta, dict, path.name)
            self.assertIn("exit", meta, path.name)
            self.assertIn(meta["exit"], (0, 1, 2, 3), path.name)
            self.assertIn("branch", meta, path.name)
            self.assertEqual(meta.get("skill"), "rhwp-table-exchange", path.name)
            self.assertEqual(meta.get("issue"), 5306, path.name)

    def test_recipe02_extract_csv(self):
        data = load_json(ENVS / "table_to_csv_hwp_table_test_t0.json")
        self.assertEqual(data["_skillMeta"]["exit"], 0)
        self.assertFalse(data["bom"])
        self.assertEqual(data["tables"][0]["index"], 0)
        self.assertEqual(data["tables"][0]["rowCount"], 4)
        self.assertEqual(data["tables"][0]["colCount"], 3)
        self.assertIn("제목,담당자,세부 내용", data["tables"][0]["csv"])
        self.assertEqual(data["untrustedFields"], ["tables[].csv"])

    def test_bom_file_not_envelope(self):
        data = load_json(ENVS / "table_to_csv_bom_file.json")
        self.assertTrue(data["bom"])
        self.assertFalse(data["_envelopeCsvStartsWithBom"])
        self.assertEqual(data["_filePrefix"], [0xEF, 0xBB, 0xBF])
        self.assertFalse(data["tables"][0]["csv"].startswith("\ufeff"))

    def test_roundtrip_ok_changed_nine(self):
        data = load_json(ENVS / "csv_to_table_ok_recipe02.json")
        self.assertEqual(data["changedCount"], 9)
        self.assertEqual(data["invalid"], [])
        self.assertTrue(data["verify"]["identical"])
        self.assertEqual(len(data["changed"]), 9)
        rows = {c["row"] for c in data["changed"]}
        self.assertNotIn(0, rows)

    def test_dry_run_null_pages_no_output(self):
        data = load_json(ENVS / "csv_to_table_dry_run.json")
        self.assertTrue(data["dryRun"])
        self.assertIsNone(data["changedPages"])
        self.assertIsNone(data["output"])
        self.assertFalse(data["_skillMeta"]["outputKept"])

    def test_row_mismatch_is_exit2_no_write(self):
        data = load_json(ENVS / "csv_to_table_row_mismatch.json")
        self.assertEqual(data["_skillMeta"]["exit"], 2)
        self.assertEqual(data["changedCount"], 0)
        self.assertFalse(data["_skillMeta"]["outputKept"])
        reasons = {item["reason"] for item in data["invalid"]}
        self.assertIn("rowCountMismatch", reasons)

    def test_col_mismatch_collects_rows(self):
        data = load_json(ENVS / "csv_to_table_col_mismatch.json")
        self.assertEqual(data["_skillMeta"]["exit"], 2)
        reasons = [item["reason"] for item in data["invalid"]]
        self.assertGreaterEqual(reasons.count("colCountMismatch"), 4)

    def test_table001_collects_both_dimensions(self):
        data = load_json(ENVS / "csv_to_table_table001_both_mismatch.json")
        self.assertEqual(data["rowCount"], 19)
        self.assertEqual(data["colCount"], 9)
        reasons = {item["reason"] for item in data["invalid"]}
        self.assertIn("rowCountMismatch", reasons)
        self.assertIn("colCountMismatch", reasons)

    def test_covered_cell_reason(self):
        data = load_json(ENVS / "csv_to_table_covered.json")
        self.assertEqual(data["_skillMeta"]["exit"], 2)
        item = data["invalid"][0]
        self.assertEqual(item["reason"], "coveredCellNotEmpty")
        self.assertEqual((item["row"], item["col"]), (0, 2))

    def test_control_character_lf_and_tab(self):
        for name in ("csv_to_table_control_lf.json", "csv_to_table_control_tab.json"):
            data = load_json(ENVS / name)
            self.assertEqual(data["_skillMeta"]["exit"], 2, name)
            self.assertEqual(data["invalid"][0]["reason"], "controlCharacter", name)

    def test_csv_parse(self):
        data = load_json(ENVS / "csv_to_table_csv_parse.json")
        self.assertEqual(data["invalid"][0]["reason"], "csvParse")
        self.assertEqual(data["changedCount"], 0)

    def test_verify_fail_is_exit3_keeps_output(self):
        data = load_json(ENVS / "csv_to_table_verify_fail.json")
        self.assertEqual(data["_skillMeta"]["exit"], 3)
        self.assertTrue(data["_skillMeta"]["outputKept"])
        self.assertFalse(data["verify"]["identical"])
        self.assertIn("output", data)

    def test_set_cell_covered_is_silent_stdout(self):
        data = load_json(ENVS / "set_cell_covered_exit2.json")
        self.assertEqual(data["_skillMeta"]["exit"], 2)
        self.assertEqual(data["_skillMeta"]["stdoutBytes"], 0)
        self.assertIn("앵커 (0,1)", data["stderrContains"])

    def test_unknown_table_exit1_silent(self):
        data = load_json(ENVS / "table_to_csv_unknown_table_exit1.json")
        self.assertEqual(data["_skillMeta"]["exit"], 1)
        self.assertEqual(data["_skillMeta"]["stdoutBytes"], 0)

    def test_jichi_index_zero_is_header(self):
        data = load_json(ENVS / "export_tables_jichi_index_not_zero.json")
        zero = data["tables"][0]
        self.assertEqual(zero["index"], 0)
        self.assertEqual(zero["containerPath"][0]["kind"], "header")
        body = next(t for t in data["tables"] if t["index"] == 12)
        self.assertNotIn("containerPath", body)

    def test_inner_table_nested_path(self):
        data = load_json(ENVS / "export_tables_inner_table.json")
        nested = None
        for cell in data["tables"][0]["cells"]:
            if cell.get("nested"):
                nested = cell["nested"][0]
        self.assertIsNotNone(nested)
        self.assertEqual(nested["containerPath"][0]["kind"], "tableCell")
        self.assertEqual(nested["cellCount"], 24)

    def test_treatise_wider_than_info(self):
        data = load_json(ENVS / "export_tables_treatise_container.json")
        self.assertEqual(data["tableCount"], 3)
        self.assertIn("info 표 열거는 최상위 1개", data["_infoContrast"])


class CsvFixtureTests(unittest.TestCase):
    def test_table0_original_is_4x3(self):
        text = (CSV / "table0_original.csv").read_bytes().decode("utf-8")
        lines = [ln for ln in text.splitlines() if ln != ""]
        self.assertEqual(len(lines), 4)
        self.assertEqual(lines[0], "제목,담당자,세부 내용")

    def test_table0_edited_keeps_header(self):
        text = (CSV / "table0_edited.csv").read_text(encoding="utf-8")
        self.assertTrue(text.startswith("제목,담당자,세부 내용"))
        self.assertIn("서버 이관", text)
        self.assertEqual(len([ln for ln in text.splitlines() if ln != ""]), 4)

    def test_bom_file_has_prefix(self):
        raw = (CSV / "table0_bom.csv").read_bytes()
        self.assertEqual(raw[:3], b"\xef\xbb\xbf")

    def test_control_lf_contains_newline_inside_quotes(self):
        raw = (CSV / "table0_control_lf.csv").read_bytes()
        self.assertIn(b"\n", raw)
        self.assertIn(b'"\xec\x84\x9c\xeb\xb2\x84', raw)

    def test_quoted_csv_escapes_quote(self):
        text = (CSV / "table0_quoted.csv").read_text(encoding="utf-8")
        self.assertIn('""', text)


class MatrixTests(unittest.TestCase):
    def test_merge_decision_has_documented_samples(self):
        data = load_json(MATS / "merge_decision.json")
        ids = {c["id"] for c in data["cases"]}
        for needed in (
            "hwp_table_test_t0",
            "table_001",
            "inner_table",
            "jichi_header_zero",
            "wrapper_1x1",
        ):
            self.assertIn(needed, ids, needed)
        allowed = next(c for c in data["cases"] if c["id"] == "hwp_table_test_t0")
        self.assertEqual(allowed["csvRoundtrip"], "allowed")
        merged = next(c for c in data["cases"] if c["id"] == "table_001")
        self.assertEqual(merged["csvRoundtrip"], "extract-only")

    def test_exit_codes_do_not_call_exit3_exception(self):
        data = load_json(MATS / "exit_codes.json")
        by_exit = {c["exit"]: c for c in data["codes"]}
        self.assertFalse(by_exit[3]["isException"])
        self.assertEqual(by_exit[3]["kind"], "verify-judgment")

    def test_invalid_reasons_complete(self):
        data = load_json(MATS / "invalid_reasons.json")
        reasons = {r["reason"] for r in data["reasons"]}
        self.assertEqual(reasons, set(INVALID_REASONS))
        self.assertTrue(data["collectAll"])

    def test_header_row_drop_is_not_ok(self):
        data = load_json(MATS / "header_row.json")
        drop = next(c for c in data["cases"] if c["id"] == "drop-header")
        self.assertFalse(drop["ok"])


class LoopTests(unittest.TestCase):
    def test_roundtrip_forbids_output_on_dry_run(self):
        data = load_json(LOOPS / "roundtrip_plain.json")
        dry = next(s for s in data["steps"] if s["id"] == "dry-run")
        self.assertIn("--dry-run", dry["command"])
        self.assertIn("output", dry["forbidFieldsPresent"])
        write = next(s for s in data["steps"] if s["id"] == "write-verify")
        self.assertIn("--verify", write["command"])
        self.assertEqual(write["expect"]["changedCount"], 9)

    def test_merge_fallback_forbids_csv_to_table(self):
        data = load_json(LOOPS / "merge_fallback.json")
        self.assertIn("csv-to-table", data["forbiddenNext"])
        self.assertEqual(data["steps"][1]["command"][1:3], ["edit", "set-cell"])

    def test_every_command_loop_starts_with_rhwp(self):
        for path in LOOPS.glob("*.json"):
            data = load_json(path)
            for step in data.get("steps", []):
                cmd = step.get("command")
                if isinstance(cmd, list):
                    self.assertEqual(cmd[0], "rhwp", f"{path.name}:{step.get('id')}")


class TranscriptTests(unittest.TestCase):
    def test_recipe02_transcript_closes(self):
        data = load_json(TRANS / "recipe02_roundtrip.json")
        self.assertEqual(data["sample"], "samples/hwp_table_test.hwp")
        last = data["turns"][-2]
        self.assertEqual(last["changedCount"], 9)
        self.assertEqual(last["verify"]["identical"], True)
        reread = data["turns"][-1]
        self.assertEqual(reread["row1"], ["서버 이관", "홍길동", "1차 완료"])

    def test_playbook_mismatch_exit2(self):
        data = load_json(TRANS / "playbook_table001_mismatch.json")
        turn = data["turns"][0]
        self.assertEqual(turn["exit"], 2)
        self.assertEqual(turn["rowCount"], 19)
        self.assertIn("rowCountMismatch", turn["invalidReasons"])

    def test_codex20_uses_table_one(self):
        data = load_json(TRANS / "codex20_issue2007.json")
        extract = data["turns"][1]
        self.assertIn("--table", extract["argv"])
        self.assertEqual(extract["index"], 1)


class CommandReferenceTests(unittest.TestCase):
    TOKEN_RE = re.compile(r"rhwp ([a-z][a-z0-9-]*)")
    SUB_RE = re.compile(r"rhwp edit ([a-z][a-z0-9-]*)")

    def test_heads_are_known(self):
        unknown = []
        for path in md_files(SKILL):
            for match in self.TOKEN_RE.finditer(read(path)):
                head = match.group(1)
                if head not in KNOWN_HEADS:
                    unknown.append(f"{path.name}: rhwp {head}")
        self.assertEqual(unknown, [], "스킬이 모르는 머리 명령을 안내한다")

    def test_edit_subs_are_set_cell_only(self):
        unknown = []
        for path in md_files(SKILL):
            for match in self.SUB_RE.finditer(read(path)):
                sub = match.group(1)
                if sub not in EDIT_SUBS:
                    unknown.append(f"{path.name}: rhwp edit {sub}")
        self.assertEqual(unknown, [])

    def test_no_invented_cli_flags_on_csv_to_table(self):
        joined = "\n".join(read(p) for p in md_files(SKILL))
        self.assertNotRegex(joined, r"csv-to-table[^\n]*--in-place")
        self.assertNotRegex(joined, r"csv-to-table[^\n]*--force")
        self.assertNotRegex(joined, r"csv-to-table[^\n]*--resize")


class WorkingDocTests(unittest.TestCase):
    def test_working_tokens(self):
        body = read(WORKING)
        for tok in WORKING_TOKENS:
            self.assertIn(tok, body, tok)

    def test_working_lists_out_of_scope(self):
        body = read(WORKING)
        self.assertIn("gym/", body)
        self.assertIn("DocumentCore", body)
        self.assertIn("새 CLI", body)


class DecisionTests(unittest.TestCase):
    def test_request_routes_exist(self):
        data = load_json(DECS / "request_to_command.json")
        self.assertGreaterEqual(len(data["rows"]), 10)
        routes = " ".join(r.get("route", "") for r in data["rows"])
        self.assertIn("export-tables", routes)
        self.assertIn("set-cell", routes)


if __name__ == "__main__":
    unittest.main()
