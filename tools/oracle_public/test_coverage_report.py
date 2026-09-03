#!/usr/bin/env python3
"""coverage_report.py 가드 테스트 — 실 코퍼스 불요.

실행:
    python -m unittest tools/oracle_public/test_coverage_report.py
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import coverage_report as cov  # noqa: E402

MINI_REPO = HERE / "fixtures" / "mini_repo"
SCRIPT = HERE / "coverage_report.py"


def _touch(path: Path, body: str = "placeholder\n") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8", newline="\n")


def write_mini_tree(root: Path) -> None:
    """테스트용 최소 트리. 실 코퍼스와 같은 이름 규칙을 쓴다."""
    files = [
        "samples/exam_kor.hwp",
        "samples/exam_kor.hwpx",
        "samples/lonely.hwp",
        "samples/편람.hwp",
        "samples/편람.hwpx",
        "samples/hwp3-sample.hwp",
        "samples/hwp3-sample-hwpx.hwpx",
        "samples/3-09월_교육_통합_2022.hwp",
        "samples/basic/calendar_year.hwp",
        "samples/multi.hwp",
        "samples/readme.txt",
        "pdf/exam_kor-2022.pdf",
        "pdf/exam_kor-2020.pdf",
        "pdf/편람-hwp-2020.pdf",
        "pdf/편람-hwpx-2020.pdf",
        "pdf/hwp3-sample-hwpx-2022.pdf",
        "pdf/3-09월_교육_통합_2022.pdf",
        "pdf/basic/calendar_year-2022.pdf",
        "pdf/multi-2018.pdf",
        "pdf/multi-2024.pdf",
        "pdf/unused-2022.pdf",
        "pdf/hwpx/lonely.pdf",
    ]
    for rel in files:
        _touch(root / rel)


class ParseSuffixTests(unittest.TestCase):
    def test_plain_year(self) -> None:
        info = cov.parse_oracle_suffix("exam_kor", "exam_kor-2022.pdf")
        self.assertEqual(info, {"year": "2022", "variant": "2022", "fmt": None})

    def test_hwp_variant_does_not_steal_hwpx_stem(self) -> None:
        stolen = cov.parse_oracle_suffix("hwp3-sample", "hwp3-sample-hwpx-2022.pdf")
        self.assertIsNotNone(stolen)
        assert stolen is not None
        self.assertEqual(stolen["fmt"], "hwpx")
        own = cov.parse_oracle_suffix(
            "hwp3-sample-hwpx", "hwp3-sample-hwpx-2022.pdf"
        )
        self.assertEqual(own, {"year": "2022", "variant": "2022", "fmt": None})

    def test_hwp_2020_variant(self) -> None:
        info = cov.parse_oracle_suffix("편람", "편람-hwp-2020.pdf")
        self.assertEqual(info, {"year": "2020", "variant": "hwp-2020", "fmt": "hwp"})

    def test_exact_stem_with_year(self) -> None:
        info = cov.parse_oracle_suffix(
            "3-09월_교육_통합_2022", "3-09월_교육_통합_2022.pdf"
        )
        self.assertEqual(info, {"year": "2022", "variant": "exact", "fmt": None})

    def test_exact_stem_without_year_rejected(self) -> None:
        self.assertIsNone(cov.parse_oracle_suffix("lonely", "lonely.pdf"))

    def test_unrelated_name_rejected(self) -> None:
        self.assertIsNone(cov.parse_oracle_suffix("exam_kor", "other-2022.pdf"))

    def test_year_2010_is_out_of_scope(self) -> None:
        self.assertIsNone(cov.parse_oracle_suffix("doc", "doc-2010.pdf"))

    def test_hancom_alt_suffix(self) -> None:
        info = cov.parse_oracle_suffix("doc", "doc-hancom2020.pdf")
        self.assertIsNotNone(info)
        assert info is not None
        self.assertEqual(info["year"], "2020")
        self.assertEqual(info["variant"], "hancom2020")


class MiniRepoTests(unittest.TestCase):
    def test_checked_in_fixture_tree_exists(self) -> None:
        self.assertTrue((MINI_REPO / "samples" / "exam_kor.hwp").is_file())
        self.assertTrue((MINI_REPO / "pdf" / "exam_kor-2022.pdf").is_file())
        self.assertTrue((MINI_REPO / "samples" / "lonely.hwp").is_file())

    def test_walk_ignores_non_hwp(self) -> None:
        docs = cov.walk_samples(MINI_REPO / "samples", MINI_REPO)
        names = [doc.sample for doc in docs]
        self.assertNotIn("samples/readme.txt", names)
        self.assertIn("samples/exam_kor.hwp", names)
        self.assertIn("samples/exam_kor.hwpx", names)

    def test_mini_unmatched_and_pairs(self) -> None:
        report = cov.build_report(MINI_REPO)
        errors = cov.validate_report(report)
        self.assertEqual(errors, [])
        samples = {item["sample"] for item in report["pairs"]}
        self.assertIn("samples/exam_kor.hwp", samples)
        self.assertIn("samples/편람.hwp", samples)
        self.assertIn("samples/hwp3-sample-hwpx.hwpx", samples)
        self.assertIn("samples/3-09월_교육_통합_2022.hwp", samples)
        unmatched = {item["sample"] for item in report["unmatched"]}
        self.assertIn("samples/lonely.hwp", unmatched)
        self.assertIn("samples/hwp3-sample.hwp", unmatched)
        self.assertNotIn("samples/readme.txt", unmatched)
        self.assertEqual(report["unmatchedCount"], 2)
        self.assertEqual(report["sampleCount"], 10)
        self.assertEqual(report["matchedSampleCount"], 8)
        self.assertEqual(report["pairCount"], 11)

    def test_format_tag_does_not_cross_hwp_hwpx(self) -> None:
        report = cov.build_report(MINI_REPO)
        hwp_pdfs = [
            item["pdf"]
            for item in report["pairs"]
            if item["sample"] == "samples/편람.hwp"
        ]
        hwpx_pdfs = [
            item["pdf"]
            for item in report["pairs"]
            if item["sample"] == "samples/편람.hwpx"
        ]
        self.assertEqual(hwp_pdfs, ["pdf/편람-hwp-2020.pdf"])
        self.assertEqual(hwpx_pdfs, ["pdf/편람-hwpx-2020.pdf"])

    def test_hwpx_stem_keeps_full_name(self) -> None:
        report = cov.build_report(MINI_REPO)
        hits = [
            item
            for item in report["pairs"]
            if item["sample"] == "samples/hwp3-sample-hwpx.hwpx"
        ]
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pdf"], "pdf/hwp3-sample-hwpx-2022.pdf")
        self.assertEqual(hits[0]["hancomVersion"], "2022")
        self.assertEqual(hits[0]["stem"], "hwp3-sample-hwpx")

    def test_exact_year_in_stem(self) -> None:
        report = cov.build_report(MINI_REPO)
        hits = [
            item
            for item in report["pairs"]
            if item["sample"] == "samples/3-09월_교육_통합_2022.hwp"
        ]
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pdf"], "pdf/3-09월_교육_통합_2022.pdf")
        self.assertEqual(hits[0]["variant"], "exact")
        self.assertEqual(hits[0]["hancomVersion"], "2022")

    def test_nested_dir_and_extra_root(self) -> None:
        report = cov.build_report(MINI_REPO)
        calendar = [
            item["pdf"]
            for item in report["pairs"]
            if item["sample"] == "samples/basic/calendar_year.hwp"
        ]
        self.assertEqual(calendar, ["pdf/basic/calendar_year-2022.pdf"])
        exam_roots = {
            item["oracleRoot"]
            for item in report["pairs"]
            if item["sample"] == "samples/exam_kor.hwp"
        }
        self.assertEqual(exam_roots, {"pdf"})

    def test_version_table_counts(self) -> None:
        report = cov.build_report(MINI_REPO)
        by_ver = report["byHancomVersion"]
        self.assertEqual(by_ver["2018"]["pairCount"], 1)
        self.assertEqual(by_ver["2018"]["sampleCount"], 1)
        self.assertEqual(by_ver["2020"]["pairCount"], 4)
        self.assertEqual(by_ver["2020"]["sampleCount"], 4)
        self.assertEqual(by_ver["2022"]["pairCount"], 5)
        self.assertEqual(by_ver["2022"]["sampleCount"], 5)
        self.assertEqual(by_ver["2024"]["pairCount"], 1)
        self.assertEqual(by_ver["2024"]["sampleCount"], 1)

    def test_unmatched_count_is_measured_not_hardcoded(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertNotRegex(source, r"unmatchedCount\s*=\s*276")
        self.assertNotRegex(source, r"UNMATCHED\s*=\s*276")
        report = cov.build_report(MINI_REPO)
        self.assertEqual(report["unmatchedCount"], len(report["unmatched"]))
        self.assertNotEqual(report["unmatchedCount"], 276)

    def test_does_not_import_forbidden_siblings(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        # docstring 에 형제 도구 금지 안내는 있어도 import 는 없어야 한다.
        self.assertNotIn("import visual_sweep", source)
        self.assertNotIn("import issue_draft", source)
        self.assertNotIn("import oracle_resolver", source)
        self.assertNotIn("from oracle_resolver", source)


class TempTreeAndCliTests(unittest.TestCase):
    def test_temp_tree_matches_checked_in_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_mini_tree(root)
            built = cov.build_report(root)
        checked = cov.build_report(MINI_REPO)
        self.assertEqual(built["unmatchedCount"], checked["unmatchedCount"])
        self.assertEqual(built["pairCount"], checked["pairCount"])
        self.assertEqual(
            [item["sample"] for item in built["unmatched"]],
            [item["sample"] for item in checked["unmatched"]],
        )

    def test_validate_rejects_bad_year(self) -> None:
        data = cov.build_report(MINI_REPO)
        data["pairs"][0]["hancomVersion"] = "2010"
        errors = cov.validate_report(data)
        self.assertTrue(any("hancomVersion" in err for err in errors))

    def test_validate_rejects_count_mismatch(self) -> None:
        data = cov.build_report(MINI_REPO)
        data["unmatchedCount"] = 276
        errors = cov.validate_report(data)
        self.assertTrue(any("unmatchedCount" in err for err in errors))

    def test_markdown_contains_unmatched_and_years(self) -> None:
        report = cov.build_report(MINI_REPO)
        md = cov.render_markdown(report)
        self.assertIn("짝 없는 샘플 (2건)", md)
        self.assertIn("`samples/lonely.hwp`", md)
        self.assertIn("| 2018 |", md)
        self.assertIn("| 2020 |", md)
        self.assertIn("| 2022 |", md)
        self.assertIn("| 2024 |", md)
        self.assertNotIn("276", md)

    def test_cli_writes_json_and_markdown(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out_json = Path(tmp) / "coverage.json"
            out_md = Path(tmp) / "coverage.md"
            code = cov.main(
                [
                    "--repo-root",
                    str(MINI_REPO),
                    "--pretty",
                    "--validate",
                    "--json-out",
                    str(out_json),
                    "--md-out",
                    str(out_md),
                ]
            )
            self.assertEqual(code, 0)
            payload = json.loads(out_json.read_text(encoding="utf-8"))
            self.assertEqual(payload["claim"], "M01-3")
            self.assertEqual(cov.validate_report(payload), [])
            md = out_md.read_text(encoding="utf-8")
            self.assertIn("M01-3", md)
            self.assertIn("짝 없는 샘플", md)

    def test_cli_missing_samples_is_usage_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            code = cov.main(["--repo-root", tmp, "--json-out", "-"])
            self.assertEqual(code, 2)

    def test_does_not_read_visual_sweep(self) -> None:
        self.assertFalse(hasattr(cov, "visual_sweep"))
        self.assertNotIn("scripts/visual_sweep.py", os.listdir(str(HERE)))


if __name__ == "__main__":
    unittest.main()
