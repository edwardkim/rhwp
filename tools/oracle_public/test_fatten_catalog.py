#!/usr/bin/env python3
"""M01-f fatten_catalog 단위·픽스처 시험."""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

TOOL_DIR = Path(__file__).resolve().parent
MODULE_PATH = TOOL_DIR / "fatten_catalog.py"
MINI_REPO = TOOL_DIR / "fixtures" / "mini_repo"

SPEC = importlib.util.spec_from_file_location("fatten_catalog", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
fatten = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = fatten
SPEC.loader.exec_module(fatten)


class FamilyTests(unittest.TestCase):
    def test_exam_and_basic(self) -> None:
        self.assertEqual(fatten.classify_family("samples/exam_kor.hwp", "exam_kor"), "exam")
        self.assertEqual(
            fatten.classify_family("samples/basic/calendar_year.hwp", "calendar_year"),
            "basic",
        )
        self.assertEqual(
            fatten.classify_family("samples/hwpx/lonely.hwp", "lonely"),
            "hwpx_tree",
        )

    def test_directory_of_root_sample(self) -> None:
        self.assertEqual(fatten.sample_directory("samples/exam_kor.hwp"), "")
        self.assertEqual(fatten.sample_directory("samples/basic/calendar_year.hwp"), "basic")


class MiniRepoFattenTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.out = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_run_writes_expected_artifacts(self) -> None:
        summary = fatten.run(MINI_REPO, self.out, ("pdf", "pdf-2020", "pdf-large"))
        self.assertGreaterEqual(summary["pairCount"], 1)
        self.assertGreaterEqual(summary["unmatchedCount"], 1)
        self.assertFalse(summary["constraints"]["visualSweepTouched"])
        self.assertFalse(summary["constraints"]["engineTouched"])
        self.assertFalse(summary["constraints"]["gymTouched"])

        year_2022 = self.out / "fixtures" / "pairs" / "by_year" / "2022.json"
        self.assertTrue(year_2022.is_file())
        year_data = json.loads(year_2022.read_text(encoding="utf-8"))
        self.assertEqual(year_data["pairCount"], len(year_data["pairs"]))
        self.assertTrue(all(item["hancomVersion"] == "2022" for item in year_data["pairs"]))

        unmatched = json.loads(
            (self.out / "catalogs" / "unmatched.json").read_text(encoding="utf-8")
        )
        self.assertEqual(unmatched["unmatchedCount"], len(unmatched["samples"]))
        self.assertEqual(unmatched["unmatchedCount"], summary["unmatchedCount"])
        for row in unmatched["samples"]:
            self.assertIn("family", row)
            self.assertIn("suggestedPdfs", row)

        sweep = json.loads(
            (self.out / "transcripts" / "cheap_sweep.json").read_text(encoding="utf-8")
        )
        self.assertEqual(sweep["pairCount"], summary["pairCount"])
        self.assertEqual(sweep["rowsPath"], "transcripts/cheap_sweep.ndjson")
        self.assertIn("largestPages", sweep)
        ndjson = (self.out / "transcripts" / "cheap_sweep.ndjson").read_text(encoding="utf-8")
        nd_rows = [json.loads(line) for line in ndjson.splitlines() if line.strip()]
        self.assertEqual(len(nd_rows), sweep["pairCount"])

        self.assertTrue((self.out / "reports" / "coverage_matrix.md").is_file())
        self.assertTrue((self.out / "reports" / "coverage_by_directory.md").is_file())
        self.assertTrue((self.out / "reports" / "coverage_by_family.md").is_file())
        self.assertTrue((self.out / "reports" / "pair_index.md").is_file())
        self.assertTrue((self.out / "catalogs" / "unused_oracle_pdfs.json").is_file())
        self.assertTrue((self.out / "drafts" / "examples" / "report_cheap_anomalies.json").is_file())
        self.assertTrue((self.out / "drafts" / "examples" / "manifest.json").is_file())

        report = json.loads(
            (self.out / "drafts" / "examples" / "report_cheap_anomalies.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(report["schema"], "oracle_public.failure_report/v1")
        self.assertIn("documents", report)

        matrix = (self.out / "reports" / "coverage_matrix.md").read_text(encoding="utf-8")
        self.assertIn("한컴", matrix)
        self.assertIn("`hwp`", matrix)

    def test_pair_index_tsv_columns(self) -> None:
        fatten.run(MINI_REPO, self.out, ("pdf", "pdf-2020", "pdf-large"))
        tsv = (self.out / "fixtures" / "pairs" / "index.tsv").read_text(encoding="utf-8")
        header = tsv.splitlines()[0].split("\t")
        self.assertEqual(
            header,
            [
                "id",
                "sample",
                "pdf",
                "stem",
                "hancomVersion",
                "variant",
                "sourceFormat",
                "oracleRoot",
                "family",
            ],
        )
        self.assertGreater(len(tsv.splitlines()), 1)

    def test_does_not_touch_visual_sweep(self) -> None:
        source = MODULE_PATH.read_text(encoding="utf-8")
        self.assertIn("visual_sweep.py", source)
        self.assertIn("읽거나", source)
        self.assertNotRegex(source, r"open\(.*visual_sweep")


class NearMissTests(unittest.TestCase):
    def test_prefix_hit(self) -> None:
        pdfs = [
            {
                "pdf": "pdf/exam_kor-2022.pdf",
                "oracleRoot": "pdf",
                "relParent": "",
                "filename": "exam_kor-2022.pdf",
                "stem": "exam_kor-2022",
            }
        ]
        hits = fatten.near_miss_pdfs("exam_kor", "", pdfs)
        self.assertTrue(hits)
        self.assertEqual(hits[0]["pdf"], "pdf/exam_kor-2022.pdf")


if __name__ == "__main__":
    unittest.main()
