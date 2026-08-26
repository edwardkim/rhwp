#!/usr/bin/env python3
"""Live contract tests for tools/form_fill/form_fill.py.

Ports already-shipped CLI rules. Does not call DocumentCore and does not
invent a new fill writer.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import catalogs
import form_fill as ff


class ParseFieldKeyTests(unittest.TestCase):
    def test_plain_name_is_occurrence_zero(self) -> None:
        self.assertEqual(ff.parse_field_key("제목명"), ("제목명", 0))

    def test_indexed_name(self) -> None:
        self.assertEqual(ff.parse_field_key("피규제집단명[3]"), ("피규제집단명", 3))

    def test_non_numeric_brackets_stay_in_name(self) -> None:
        self.assertEqual(ff.parse_field_key("항목[갑]"), ("항목[갑]", 0))

    def test_missing_close_stays_in_name(self) -> None:
        self.assertEqual(ff.parse_field_key("항목[3"), ("항목[3", 0))

    def test_inner_zero(self) -> None:
        self.assertEqual(ff.parse_field_key("성명[0]"), ("성명", 0))


class SurveyTests(unittest.TestCase):
    def test_field_count_matches_list(self) -> None:
        form = catalogs.form_by_id("field-01")
        env = ff.survey_fields(form)
        self.assertEqual(env["schemaVersion"], "1.0")
        self.assertEqual(env["fieldCount"], len(env["fields"]))
        self.assertEqual(env["fieldCount"], 11)
        self.assertEqual(env["fields"][0]["name"], "회사명")
        self.assertEqual(env["textSecurity"]["status"], "clean")

    def test_empty_document_is_zero_not_error(self) -> None:
        form = catalogs.form_by_id("empty-none")
        env = ff.survey_fields(form)
        self.assertEqual(env["fieldCount"], 0)
        self.assertEqual(env["fields"], [])

    def test_repeated_name_count_regulatory(self) -> None:
        form = catalogs.form_by_id("reg-80168")
        self.assertGreaterEqual(form.name_counts()["피규제집단명"], 3)
        self.assertEqual(form.name_counts()["피규제집단명"], 14)


class FillPlanTests(unittest.TestCase):
    def test_plain_unique_name_fills_one(self) -> None:
        form = catalogs.form_by_id("field-01")
        plan = ff.plan_fill(form, {"회사명": "주식회사 검증"})
        self.assertEqual(plan.filled_count, 1)
        self.assertEqual(plan.filled[0].name, "회사명")
        self.assertEqual(plan.filled[0].occurrence, 0)
        self.assertEqual(plan.not_found, [])
        self.assertEqual(plan.ambiguous, [])

    def test_unknown_name_is_not_found(self) -> None:
        form = catalogs.form_by_id("field-01")
        plan = ff.plan_fill(form, {"회사명": "A", "존재하지않는필드": "B"})
        self.assertEqual(plan.filled_count, 1)
        self.assertIn("존재하지않는필드", plan.not_found)

    def test_occurrence_targets_nth(self) -> None:
        form = catalogs.form_by_id("reg-80168")
        plan = ff.plan_fill(
            form,
            {"피규제집단명[0]": "가상협회 회원사", "피규제집단명[2]": "가상조합 조합원"},
        )
        self.assertEqual(plan.filled_count, 2)
        after = ff.apply_values(form, plan)
        values = ff.values_by_name(after)["피규제집단명"]
        self.assertEqual(values[0], "가상협회 회원사")
        self.assertEqual(values[2], "가상조합 조합원")
        self.assertEqual(values[1], form.values_of("피규제집단명")[1])

    def test_plain_repeated_name_is_ambiguous_and_fills_first(self) -> None:
        form = catalogs.form_by_id("reg-80168")
        plan = ff.plan_fill(form, {"피규제집단명": "가상협회 회원사"})
        self.assertEqual(plan.filled_count, 1)
        self.assertEqual(len(plan.ambiguous), 1)
        self.assertEqual(plan.ambiguous[0].name, "피규제집단명")
        self.assertEqual(plan.ambiguous[0].matched, 1)
        self.assertEqual(plan.ambiguous[0].total, 14)

    def test_out_of_range_index_not_found(self) -> None:
        form = catalogs.form_by_id("reg-80168")
        key = "피규제집단명[114]"
        plan = ff.plan_fill(form, {key: "값"})
        self.assertEqual(plan.filled_count, 0)
        self.assertEqual(plan.not_found, [key])


class DryRunVerifyTests(unittest.TestCase):
    def test_dry_run_omits_output(self) -> None:
        form = catalogs.form_by_id("field-01")
        env = ff.fill_envelope(form, {"회사명": "주식회사 A"}, dry_run=True, output="out/x.hwp")
        self.assertTrue(env["dryRun"])
        self.assertNotIn("output", env)
        self.assertIsNone(env["verify"])
        self.assertEqual(ff.exit_for_envelope(env), 0)

    def test_write_includes_output_format(self) -> None:
        form = catalogs.form_by_id("gian-1")
        env = ff.fill_envelope(form, {"제목": "협조 요청"}, output="out/gian.hwpx")
        self.assertFalse(env["dryRun"])
        self.assertEqual(env["outputFormat"], "hwpx")
        self.assertTrue(env["output"].endswith(".hwpx"))

    def test_hwpx_to_hwp_flips_format(self) -> None:
        form = catalogs.form_by_id("gian-1")
        self.assertEqual(ff.output_format_label(form.fmt, "out/gian.hwp"), "hwp5")

    def test_verify_identical_exit_zero(self) -> None:
        form = catalogs.form_by_id("field-01")
        env = ff.fill_envelope(form, {"회사명": "검증사"}, verify=True, output="out/v.hwp")
        self.assertTrue(env["verify"]["identical"])
        self.assertEqual(env["verify"]["diffCount"], 0)
        self.assertEqual(ff.exit_for_envelope(env), 0)

    def test_without_verify_field_is_null(self) -> None:
        form = catalogs.form_by_id("field-01")
        env = ff.fill_envelope(form, {"회사명": "A"}, output="out/n.hwp")
        self.assertIsNone(env["verify"])


class Honggildong4781Tests(unittest.TestCase):
    def test_first_field_only_is_pass(self) -> None:
        form = catalogs.form_by_id("field-01")
        data = ff.first_field_honggildong_request(form)
        self.assertEqual(data, {"회사명": "홍길동"})
        case = ff.honggildong_case(form, data, intended=["회사명"])
        self.assertTrue(case["detect"]["firstFieldOk"])
        self.assertEqual(case["detect"]["cloneCount"], 0)
        self.assertEqual(case["detect"]["verdict"], "pass")
        self.assertEqual(case["afterValues"][0], "홍길동")
        self.assertTrue(all(value != "홍길동" for value in case["afterValues"][1:]))

    def test_clone_all_is_forbidden(self) -> None:
        form = catalogs.form_by_id("field-01")
        clone = ff.clone_honggildong_request(form)
        self.assertGreater(len(clone), 1)
        case = ff.honggildong_case(form, clone, intended=["회사명"])
        self.assertEqual(case["detect"]["verdict"], "clone_forbidden")
        self.assertGreater(case["detect"]["cloneCount"], 0)

    def test_form01_single_field_cannot_clone(self) -> None:
        form = catalogs.form_by_id("form-01")
        case = ff.honggildong_case(
            form, {"myMsg01": "홍길동"}, intended=["myMsg01"]
        )
        self.assertEqual(case["detect"]["verdict"], "pass")
        self.assertEqual(case["detect"]["cloneCount"], 0)


class BatchTests(unittest.TestCase):
    def test_jsonl_rows_keep_order(self) -> None:
        form = catalogs.form_by_id("field-01")
        text = '{"회사명":"가나다 주식회사"}\n{"회사명":"라마바 주식회사"}\n'
        rows = ff.parse_jsonl_rows(text)
        recs = ff.batch_fill(form, rows, out_dir="out")
        self.assertEqual(len(recs), 2)
        self.assertEqual(recs[0]["row"], 0)
        self.assertEqual(recs[1]["row"], 1)
        self.assertEqual(recs[0]["filledCount"], 1)
        self.assertEqual(ff.batch_exit(recs), 0)

    def test_csv_strips_bom(self) -> None:
        form = catalogs.form_by_id("field-01")
        text = "\ufeff회사명,작성자\r\n한빛,홍길동\r\n"
        rows = ff.parse_csv_rows(text)
        self.assertEqual(rows[0]["data"]["회사명"], "한빛")
        recs = ff.batch_fill(form, rows, out_dir="out")
        self.assertEqual(recs[0]["notFound"], [])

    def test_csv_column_mismatch_is_row_error(self) -> None:
        text = "회사명,작성자\n정상A,정상B\n칸이,너무,많다\n"
        rows = ff.parse_csv_rows(text)
        self.assertIsNone(rows[0].get("error"))
        self.assertIn("error", rows[1])
        form = catalogs.form_by_id("field-01")
        recs = ff.batch_fill(form, rows, out_dir="out")
        self.assertEqual(ff.batch_exit(recs), 1)
        self.assertNotIn("output", recs[1])

    def test_dry_run_batch_has_no_output(self) -> None:
        form = catalogs.form_by_id("field-01")
        rows = [{"row": 0, "data": {"회사명": "하나"}}]
        recs = ff.batch_fill(form, rows, dry_run=True, out_dir="out")
        self.assertTrue(recs[0]["dryRun"])
        self.assertNotIn("output", recs[0])

    def test_name_field_suffix_does_not_overwrite(self) -> None:
        form = catalogs.form_by_id("field-01")
        rows = [
            {"row": 0, "data": {"회사명": "홍길동", "작성자": "일차"}},
            {"row": 1, "data": {"회사명": "홍길동", "작성자": "이차"}},
        ]
        recs = ff.batch_fill(form, rows, name_field="회사명", out_dir="out")
        names = [Path(rec["output"]).name for rec in recs]
        self.assertEqual(len(set(name.lower() for name in names)), 2)

    def test_name_field_cannot_escape(self) -> None:
        form = catalogs.form_by_id("field-01")
        rows = [{"row": 0, "data": {"회사명": "../../탈출", "작성자": "값"}}]
        recs = ff.batch_fill(form, rows, name_field="회사명", out_dir="out")
        out = recs[0]["output"]
        self.assertTrue(out.startswith("out/"))
        self.assertNotIn("..", out)

    def test_broken_jsonl_stays_in_stream(self) -> None:
        text = '{"회사명":"정상 앞"}\n이건 JSON 이 아니다\n["배열은 객체가 아니다"]\n{"회사명":"정상 뒤"}\n'
        rows = ff.parse_jsonl_rows(text)
        self.assertEqual(len(rows), 4)
        form = catalogs.form_by_id("field-01")
        recs = ff.batch_fill(form, rows, out_dir="out")
        self.assertEqual(ff.batch_exit(recs), 1)
        self.assertIsNone(recs[0].get("error"))
        self.assertIsNotNone(recs[1].get("error"))
        self.assertIsNotNone(recs[2].get("error"))
        self.assertIsNone(recs[3].get("error"))


class GateTests(unittest.TestCase):
    def test_gate_rejects_ambiguous(self) -> None:
        form = catalogs.form_by_id("reg-80168")
        env = ff.fill_envelope(form, {"피규제집단명": "가상협회 회원사"}, dry_run=True)
        self.assertFalse(ff.gate_single(env))

    def test_gate_accepts_indexed_repeat(self) -> None:
        form = catalogs.form_by_id("reg-80168")
        data = {f"피규제집단명[{i}]": f"집단{i}" for i in range(14)}
        env = ff.fill_envelope(form, data, verify=True, output="out/r.hwp")
        self.assertTrue(ff.gate_single(env))


class ArgvTests(unittest.TestCase):
    def test_argv_uses_existing_cli_only(self) -> None:
        args = ff.argv_fill("samples/field-01.hwp", '{"회사명":"A"}', dry_run=True)
        self.assertEqual(args[0], "edit")
        self.assertEqual(args[1], "fill-fields")
        self.assertIn("--dry-run", args)
        self.assertIn("--json", args)
        batch = ff.argv_batch("samples/field-01.hwp", "rows.jsonl", "out", name_field="작성자")
        self.assertEqual(batch[:2], ["batch", "fill"])
        self.assertNotIn("mail-merge", batch)


if __name__ == "__main__":
    unittest.main()
