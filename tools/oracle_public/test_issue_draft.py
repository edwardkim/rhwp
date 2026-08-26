#!/usr/bin/env python3
"""issue_draft.py 계약 테스트 — 바이너리·네트워크 불요.

실행:
    python -m unittest tools/oracle_public/test_issue_draft.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import issue_draft as draft  # noqa: E402

FIXTURES = HERE / "fixtures"
SCRIPT = HERE / "issue_draft.py"
FORBIDDEN_HELP_SNIPPET = "이 도구는 초안 markdown 만 디스크에 씁니다."


def _run(args: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        text=True,
        encoding="utf-8",
        capture_output=True,
        **kwargs,
    )


class SchemaTests(unittest.TestCase):
    def test_mixed_fixture_parses(self) -> None:
        report = draft.load_report(FIXTURES / "report_mixed.json")
        self.assertEqual(report.threshold.metric, "worst_pixel_match_percent")
        self.assertEqual(len(report.documents), 3)

    def test_schema_file_exists_and_names_contract(self) -> None:
        schema = json.loads((HERE / "schema" / "failure_report.v1.json").read_text(encoding="utf-8"))
        self.assertEqual(schema["$id"], draft.SCHEMA_ID)
        self.assertIn("schema", schema["required"])
        self.assertIn("threshold", schema["required"])
        self.assertIn("documents", schema["required"])

    def test_wrong_schema_id_rejected(self) -> None:
        with self.assertRaises(draft.ReportError):
            draft.parse_report({"schema": "nope", "threshold": {}, "documents": []})

    def test_invalid_fixture_rejected(self) -> None:
        with self.assertRaises(draft.ReportError):
            draft.load_report(FIXTURES / "report_invalid.json")

    def test_missing_metrics_rejected(self) -> None:
        with self.assertRaises(draft.ReportError):
            draft.parse_document({"id": "x", "hwp": "a.hwp"}, 0)

    def test_bad_threshold_op_rejected(self) -> None:
        with self.assertRaises(draft.ReportError):
            draft.parse_threshold({"metric": "x", "op": "==", "value": 1})

    def test_broken_json_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.json"
            path.write_text("{", encoding="utf-8")
            with self.assertRaises(draft.ReportError):
                draft.load_report(path)


class GateTests(unittest.TestCase):
    def test_mixed_fixture_selects_two(self) -> None:
        report = draft.load_report(FIXTURES / "report_mixed.json")
        batch = draft.select_drafts(report)
        ids = [item.document.id for item in batch.drafts]
        self.assertEqual(ids, ["exam_math", "plan_2022"])
        self.assertEqual([item["id"] for item in batch.skipped], ["exam_eng"])

    def test_all_pass_writes_zero_drafts(self) -> None:
        report = draft.load_report(FIXTURES / "report_all_pass.json")
        self.assertEqual(draft.select_drafts(report).drafts, [])

    def test_explicit_exceeds_overrides_metric(self) -> None:
        report = draft.load_report(FIXTURES / "report_explicit.json")
        batch = draft.select_drafts(report)
        self.assertEqual([item.document.id for item in batch.drafts], ["forced_flag"])
        self.assertEqual(batch.skipped[0]["id"], "suppressed")

    def test_diff_ratio_greater_than(self) -> None:
        report = draft.load_report(FIXTURES / "report_diff_ratio.json")
        batch = draft.select_drafts(report)
        self.assertEqual([item.document.id for item in batch.drafts], ["bunjang"])

    def test_missing_metric_is_skipped(self) -> None:
        report = draft.parse_report(
            {
                "schema": draft.SCHEMA_ID,
                "threshold": {"metric": "missing", "op": "<", "value": 1},
                "documents": [{"id": "a", "hwp": "a.hwp", "metrics": {"other": 0}}],
            }
        )
        batch = draft.select_drafts(report)
        self.assertEqual(batch.drafts, [])
        self.assertIn("지표 없음", batch.skipped[0]["reason"])


class RenderTests(unittest.TestCase):
    def test_draft_contains_repro_and_numbers(self) -> None:
        report = draft.load_report(FIXTURES / "report_mixed.json")
        document = report.documents[0]
        body = draft.render_template(
            (HERE / "templates" / "issue.md").read_text(encoding="utf-8"),
            draft.draft_context(document, report, 87.5),
        )
        self.assertIn("python scripts/visual_sweep.py --hwp samples/exam_math.hwp", body)
        self.assertIn("87.5", body)
        self.assertIn("worst_pixel_match_percent", body)
        self.assertIn("3, 7", body)
        self.assertIn("submit: never", body)
        self.assertIn("초안 — 제출하지 않음", body)
        self.assertIn("수식 줄간격", body)

    def test_repro_is_synthesized_when_missing(self) -> None:
        report = draft.load_report(FIXTURES / "report_mixed.json")
        command = draft.synthesize_repro(report.documents[1], report)
        self.assertEqual(
            command,
            "python scripts/visual_sweep.py --hwp samples/exam_eng.hwp "
            "--pdf pdf/exam_eng-2022.pdf --key exam_eng --dpi 96 "
            "--pixel-diff-threshold 12",
        )

    def test_slug_collision_gets_suffix(self) -> None:
        used: set[str] = set()
        first = draft.unique_draft_path(Path("out"), "exam_math", used)
        second = draft.unique_draft_path(Path("out"), "exam_math", used)
        self.assertEqual(first.name, "exam_math.md")
        self.assertEqual(second.name, "exam_math-2.md")

    def test_slugify_strips_unsafe(self) -> None:
        self.assertEqual(draft.slugify("2022 plan.hwp"), "2022-plan.hwp")
        self.assertEqual(draft.slugify("???"), "untitled")


class WriteTests(unittest.TestCase):
    def test_writes_markdown_and_manifest(self) -> None:
        report = draft.load_report(FIXTURES / "report_mixed.json")
        template = (HERE / "templates" / "issue.md").read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            manifest = draft.write_drafts(
                report,
                out,
                template,
                force=False,
                dry_run=False,
                report_path=FIXTURES / "report_mixed.json",
            )
            self.assertEqual(manifest["drafted"], 2)
            self.assertEqual(manifest["skipped"], 1)
            self.assertFalse(manifest["submitted"])
            self.assertEqual(manifest["submit"], "manual")
            math = out / "exam_math.md"
            plan = out / "plan_2022.md"
            self.assertTrue(math.is_file())
            self.assertTrue(plan.is_file())
            self.assertFalse((out / "exam_eng.md").exists())
            text = math.read_text(encoding="utf-8")
            self.assertIn("87.5", text)
            self.assertIn("--pixel-diff-threshold 12", text)
            saved = json.loads((out / "manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(saved["drafted"], 2)

    def test_dry_run_writes_nothing(self) -> None:
        report = draft.load_report(FIXTURES / "report_mixed.json")
        template = (HERE / "templates" / "issue.md").read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory) / "nested"
            manifest = draft.write_drafts(
                report,
                out,
                template,
                force=False,
                dry_run=True,
                report_path=FIXTURES / "report_mixed.json",
            )
            self.assertEqual(manifest["drafted"], 2)
            self.assertFalse(out.exists())

    def test_refuse_overwrite_without_force(self) -> None:
        report = draft.load_report(FIXTURES / "report_mixed.json")
        template = (HERE / "templates" / "issue.md").read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            (out / "exam_math.md").write_text("keep", encoding="utf-8")
            with self.assertRaises(draft.ReportError):
                draft.write_drafts(
                    report,
                    out,
                    template,
                    force=False,
                    dry_run=False,
                    report_path=FIXTURES / "report_mixed.json",
                )
            self.assertEqual((out / "exam_math.md").read_text(encoding="utf-8"), "keep")


class CliTests(unittest.TestCase):
    def test_cli_mixed_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = _run(
                [
                    "--report",
                    str(FIXTURES / "report_mixed.json"),
                    "--out",
                    directory,
                    "--json",
                ]
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["drafted"], 2)
        self.assertFalse(payload["submitted"])
        self.assertEqual(payload["submit"], "manual")

    def test_cli_all_pass_exit_zero(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = _run(
                [
                    "--report",
                    str(FIXTURES / "report_all_pass.json"),
                    "--out",
                    directory,
                    "--json",
                ]
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["drafted"], 0)

    def test_cli_invalid_exit_two(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = _run(
                [
                    "--report",
                    str(FIXTURES / "report_invalid.json"),
                    "--out",
                    directory,
                ]
            )
        self.assertEqual(result.returncode, draft.EXIT_USAGE)
        self.assertIn("오류", result.stderr)

    def test_submit_flag_rejected(self) -> None:
        result = _run(
            [
                "--submit",
                "--report",
                str(FIXTURES / "report_mixed.json"),
                "--out",
                "unused",
            ]
        )
        self.assertEqual(result.returncode, draft.EXIT_SUBMIT_FORBIDDEN)
        self.assertIn("수동", result.stderr)

    def test_create_issue_flag_rejected(self) -> None:
        result = _run(["--create-issue"])
        self.assertEqual(result.returncode, draft.EXIT_SUBMIT_FORBIDDEN)

    def test_module_never_invokes_github(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertNotIn("import subprocess", source)
        self.assertNotIn("os.system", source)
        self.assertNotIn("shutil.which", source)
        self.assertIn(FORBIDDEN_HELP_SNIPPET, source)


class SourceIsolationTests(unittest.TestCase):
    def test_only_new_oracle_public_tree(self) -> None:
        self.assertTrue((HERE / "issue_draft.py").is_file())
        self.assertTrue((HERE / "templates" / "issue.md").is_file())
        self.assertTrue((HERE / "fixtures" / "report_mixed.json").is_file())
        sweep = HERE.parents[1] / "scripts" / "visual_sweep.py"
        self.assertTrue(sweep.is_file())


if __name__ == "__main__":
    unittest.main()
