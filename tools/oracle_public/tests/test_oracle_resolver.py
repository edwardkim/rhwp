#!/usr/bin/env python3
"""oracle_resolver 단위·픽스처 시험."""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent
TOOL_DIR = TESTS_DIR.parent
MODULE_PATH = TOOL_DIR / "oracle_resolver.py"
MINI_REPO = TOOL_DIR / "fixtures" / "mini_repo"
SCHEMA_PATH = TOOL_DIR / "schema" / "oracle_pair_manifest.schema.json"
EXPECTED_PATH = TOOL_DIR / "fixtures" / "expected_mini_manifest.json"

SPEC = importlib.util.spec_from_file_location("oracle_resolver", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
oracle_resolver = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = oracle_resolver
SPEC.loader.exec_module(oracle_resolver)


def load_mini() -> dict:
    return oracle_resolver.build_manifest(MINI_REPO)


class ParseSuffixTests(unittest.TestCase):
    def test_plain_year(self) -> None:
        info = oracle_resolver.parse_oracle_suffix("exam_kor", "exam_kor-2022.pdf")
        self.assertEqual(info, {"year": "2022", "variant": "2022", "fmt": None})

    def test_hwp_variant_does_not_steal_hwpx_stem(self) -> None:
        stolen = oracle_resolver.parse_oracle_suffix(
            "hwp3-sample", "hwp3-sample-hwpx-2022.pdf"
        )
        self.assertIsNotNone(stolen)
        assert stolen is not None
        self.assertEqual(stolen["fmt"], "hwpx")
        own = oracle_resolver.parse_oracle_suffix(
            "hwp3-sample-hwpx", "hwp3-sample-hwpx-2022.pdf"
        )
        self.assertEqual(own, {"year": "2022", "variant": "2022", "fmt": None})

    def test_hwp_2020_variant(self) -> None:
        info = oracle_resolver.parse_oracle_suffix("편람", "편람-hwp-2020.pdf")
        self.assertEqual(info, {"year": "2020", "variant": "hwp-2020", "fmt": "hwp"})

    def test_exact_stem_with_year(self) -> None:
        info = oracle_resolver.parse_oracle_suffix(
            "3-09월_교육_통합_2022", "3-09월_교육_통합_2022.pdf"
        )
        self.assertEqual(info, {"year": "2022", "variant": "exact", "fmt": None})

    def test_exact_stem_without_year_rejected(self) -> None:
        self.assertIsNone(oracle_resolver.parse_oracle_suffix("lonely", "lonely.pdf"))

    def test_unrelated_name_rejected(self) -> None:
        self.assertIsNone(oracle_resolver.parse_oracle_suffix("exam_kor", "other-2022.pdf"))

    def test_hancom_alt_suffix(self) -> None:
        info = oracle_resolver.parse_oracle_suffix("doc", "doc-hancom2020.pdf")
        self.assertEqual(info["year"], "2020")
        self.assertEqual(info["variant"], "hancom2020")


class MiniRepoTests(unittest.TestCase):
    def test_fixture_tree_exists(self) -> None:
        self.assertTrue((MINI_REPO / "samples" / "exam_kor.hwp").is_file())
        self.assertTrue((MINI_REPO / "pdf" / "exam_kor-2022.pdf").is_file())
        self.assertTrue(SCHEMA_PATH.is_file())

    def test_walk_ignores_non_hwp(self) -> None:
        docs = oracle_resolver.walk_samples(MINI_REPO / "samples", MINI_REPO)
        names = [doc.sample for doc in docs]
        self.assertNotIn("samples/readme.txt", names)
        self.assertIn("samples/exam_kor.hwp", names)
        self.assertIn("samples/exam_kor.hwpx", names)

    def test_mini_pair_count_and_unmatched(self) -> None:
        manifest = load_mini()
        errors = oracle_resolver.validate_manifest(manifest)
        self.assertEqual(errors, [])
        samples = {item["sample"] for item in manifest["pairs"]}
        self.assertIn("samples/exam_kor.hwp", samples)
        self.assertIn("samples/편람.hwp", samples)
        self.assertIn("samples/hwp3-sample-hwpx.hwpx", samples)
        self.assertIn("samples/3-09월_교육_통합_2022.hwp", samples)
        unmatched = {item["sample"] for item in manifest["unmatched"]}
        self.assertIn("samples/lonely.hwp", unmatched)
        self.assertIn("samples/hwp3-sample.hwp", unmatched)
        self.assertNotIn("samples/readme.txt", unmatched)

    def test_format_tag_does_not_cross_hwp_hwpx(self) -> None:
        manifest = load_mini()
        hwp_pdfs = [
            item["pdf"]
            for item in manifest["pairs"]
            if item["sample"] == "samples/편람.hwp"
        ]
        hwpx_pdfs = [
            item["pdf"]
            for item in manifest["pairs"]
            if item["sample"] == "samples/편람.hwpx"
        ]
        self.assertEqual(hwp_pdfs, ["pdf/편람-hwp-2020.pdf"])
        self.assertEqual(hwpx_pdfs, ["pdf/편람-hwpx-2020.pdf"])

    def test_hwpx_stem_keeps_full_name(self) -> None:
        manifest = load_mini()
        hits = [
            item
            for item in manifest["pairs"]
            if item["sample"] == "samples/hwp3-sample-hwpx.hwpx"
        ]
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pdf"], "pdf/hwp3-sample-hwpx-2022.pdf")
        self.assertEqual(hits[0]["hancomVersion"], "2022")
        self.assertEqual(hits[0]["stem"], "hwp3-sample-hwpx")

    def test_exact_year_in_stem(self) -> None:
        manifest = load_mini()
        hits = [
            item
            for item in manifest["pairs"]
            if item["sample"] == "samples/3-09월_교육_통합_2022.hwp"
        ]
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pdf"], "pdf/3-09월_교육_통합_2022.pdf")
        self.assertEqual(hits[0]["variant"], "exact")
        self.assertEqual(hits[0]["hancomVersion"], "2022")

    def test_nested_dir_and_extra_root(self) -> None:
        manifest = load_mini()
        calendar = [
            item["pdf"]
            for item in manifest["pairs"]
            if item["sample"] == "samples/basic/calendar_year.hwp"
        ]
        self.assertEqual(calendar, ["pdf/basic/calendar_year-2022.pdf"])
        exam_roots = {
            item["oracleRoot"]
            for item in manifest["pairs"]
            if item["sample"] == "samples/exam_kor.hwp"
        }
        self.assertEqual(exam_roots, {"pdf"})

    def test_counts_are_consistent(self) -> None:
        manifest = load_mini()
        self.assertEqual(manifest["pairCount"], len(manifest["pairs"]))
        self.assertEqual(manifest["oracleLinkCount"], len(manifest["pairs"]))
        self.assertEqual(manifest["unmatchedCount"], len(manifest["unmatched"]))
        self.assertEqual(
            manifest["matchedSampleCount"],
            len({item["sample"] for item in manifest["pairs"]}),
        )
        self.assertEqual(manifest["targetPairCount"], 269)
        self.assertEqual(manifest["schemaVersion"], "1.0")

    def test_expected_fixture_snapshot(self) -> None:
        manifest = load_mini()
        expected = json.loads(EXPECTED_PATH.read_text(encoding="utf-8"))
        self.assertEqual(manifest["pairs"], expected["pairs"])
        self.assertEqual(manifest["unmatched"], expected["unmatched"])
        self.assertEqual(manifest["pairCount"], expected["pairCount"])


class SchemaAndCliTests(unittest.TestCase):
    def test_schema_file_is_draft07(self) -> None:
        schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        self.assertIn("draft-07", schema["$schema"])
        self.assertEqual(schema["properties"]["schemaVersion"]["const"], "1.0")
        self.assertIn("pairs", schema["required"])
        self.assertIn("unmatched", schema["required"])

    def test_validate_rejects_bad_year(self) -> None:
        data = load_mini()
        data["pairs"][0]["hancomVersion"] = "2010"
        errors = oracle_resolver.validate_manifest(data)
        self.assertTrue(any("hancomVersion" in err for err in errors))

    def test_cli_writes_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "oracle_pairs.json"
            code = oracle_resolver.main(
                [
                    "--repo-root",
                    str(MINI_REPO),
                    "--pretty",
                    "--validate",
                    "-o",
                    str(out),
                ]
            )
            self.assertEqual(code, 0)
            payload = json.loads(out.read_text(encoding="utf-8"))
            self.assertGreaterEqual(payload["pairCount"], 1)
            self.assertEqual(oracle_resolver.validate_manifest(payload), [])

    def test_cli_missing_samples_is_usage_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            code = oracle_resolver.main(["--repo-root", tmp, "-o", "-"])
            self.assertEqual(code, 2)


if __name__ == "__main__":
    unittest.main()
