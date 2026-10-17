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

    def test_all_pass_writes_zero_drafts(self) -> None:
        report = draft.load_report(FIXTURES / "report_all_pass.json")
        self.assertEqual(draft.select_drafts(report).drafts, [])


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



    def test_slugify_strips_unsafe(self) -> None:
        self.assertEqual(draft.slugify("2022 plan.hwp"), "2022-plan.hwp")
        self.assertEqual(draft.slugify("???"), "untitled")


class WriteTests(unittest.TestCase):
    pass




class CliTests(unittest.TestCase):

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
    pass


if __name__ == "__main__":
    unittest.main()
