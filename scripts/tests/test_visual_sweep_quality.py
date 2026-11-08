"""명시적으로 호출하는 과거 #8064 비교 도구의 계약 검사다.

새 렌더러 golden 대신 기존 TSV를 사용하는 독립 검사다.
비교 도구는 Visual Sweep의 기본 검토 판정에 자동으로 연결되지 않는다.
"""
import copy
import csv
import hashlib
import json
import tempfile
import unittest
from decimal import Decimal
from pathlib import Path

from scripts import visual_sweep_quality as quality
from test_visual_sweep import SWEEP

ROOT = Path(__file__).resolve().parents[2]
SUMMARY = ROOT / "mydocs/pr/assets/issue7642/final_sweep_20261019/summary.tsv"


class AcceptedQualityTests(unittest.TestCase):
    def setUp(self):
        with SUMMARY.open(encoding="utf-8", newline="") as stream:
            self.entries = list(csv.DictReader(stream, delimiter="\t"))
        self.entry = next(x for x in self.entries if x["key"] == "all-12")
        self.metrics = self.read_scores(self.entry["native_tsv"])
        self.provenance = {
            "hwp": {"path": self.entry["executed_input"], "sha256": self.entry["input_sha256"]},
            "pdf": {"path": self.entry["executed_pdf"], "sha256": self.entry["pdf_sha256"]},
            "raster_backend": "native-skia", "layout_generation": "2022",
            "comparison_profile": "print",
            "pdf_rasterizer": {"policy": "pdfium_smooth_rgb_v1", "version": "153.0.7999.0"},
        }

    def read_scores(self, relative):
        with (SUMMARY.parent / relative).open(encoding="utf-8", newline="") as stream:
            return list(csv.DictReader(stream, delimiter="\t"))

    def check(self, metrics=None, provenance=None, dpi=96, counts=None, root=ROOT):
        return quality.check_quality(root, provenance or self.provenance,
                                     self.metrics if metrics is None else metrics, dpi, 2,
                                     {"rhwp": 18, "pdf": 18} if counts is None else counts,
                                     SWEEP.SILHOUETTE_METHOD)

    def test_complete_accepted_scope_and_original_tsv_hashes(self):
        self.assertEqual((len(self.entries), sum(int(x["pages"]) for x in self.entries)), (179, 2082))
        for entry in self.entries:
            for side in ["native"]:
                path = SUMMARY.parent / entry[f"{side}_tsv"]
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), entry[f"{side}_tsv_sha256"])
                scores = quality._scores(self.read_scores(entry[f"{side}_tsv"]))
                self.assertEqual(set(scores), set(range(1, int(entry["pages"]) + 1)))
                self.assertGreaterEqual(min(scores.values()), Decimal("90"))

    def test_accepted_pages_pass_without_downgrading_other_review_requirements(self):
        self.assertEqual(self.check()["status"], "passed")
        manifest = {"provenance": self.provenance, "pr_review_gate": {"status": "not_evaluated"}}
        quality.enforce_quality(ROOT, manifest, self.metrics, 96, {"rhwp": 18, "pdf": 18}, SWEEP.SILHOUETTE_METHOD)
        self.assertEqual(manifest["pr_review_gate"]["status"], "not_evaluated")

    def test_inclusive_decimal_drop_and_subthreshold_decline(self):
        ref = {1: Decimal("98.07832")}
        # 0.1%p 하락은 새 2%p 재검토 경계 안이며, 정확히 2부터 재검토한다.
        for candidate, status in [("97.97832", "passed"), ("96.07833", "passed"),
                                  ("96.07832", "re_review_required"),
                                  ("96.07831", "re_review_required")]:
            with self.subTest(candidate=candidate):
                result = quality.compare_pages(ref, {1: Decimal(candidate)}, 1)
                self.assertEqual(result["status"], status)
                self.assertEqual(bool(result["regressions"]), status == "re_review_required")

    def test_another_pages_improvement_does_not_offset_a_two_point_drop(self):
        result = quality.compare_pages({1: Decimal("99"), 2: Decimal("96")},
                                       {1: Decimal("97"), 2: Decimal("99")}, 2)
        self.assertEqual(result["status"], "re_review_required")
        self.assertEqual([row["page"] for row in result["regressions"]], [1])

    def test_below_90_still_requires_review_with_less_than_two_point_drop(self):
        result = quality.compare_pages({1: Decimal("90.5")}, {1: Decimal("89.5")}, 1)
        self.assertEqual(result["status"], "re_review_required")
        self.assertEqual(result["regressions"], [])
        self.assertEqual(result["below_90_pages"], [1])

    def test_raw_loss_is_not_rescued_by_adjusted_score_or_other_page_improvements(self):
        rows = copy.deepcopy(self.metrics)
        rows[15][quality.METRIC] = "88.19667"
        rows[15]["tolerant_content_match_percent"] = "100"
        manifest = {"provenance": self.provenance, "pr_review_gate": {"status": "font_mismatch_exception"}}
        quality.enforce_quality(ROOT, manifest, rows, 96, {"rhwp": 18, "pdf": 18}, SWEEP.SILHOUETTE_METHOD)
        self.assertEqual(manifest["pr_review_gate"]["status"], "re_review_required")
        self.assertIn("p16", SWEEP.review_gate_reason(manifest["pr_review_gate"]))

    def test_dpi_mismatch_is_not_a_numeric_regression_comparison(self):
        result = self.check(dpi=192)
        self.assertEqual(result["checks"][0]["reason"], "measurement_conditions_differ")
        self.assertNotIn("regressions", result["checks"][0])

    def test_other_generation_and_pdf_policy_are_not_reused(self):
        for key, value in [("layout_generation", "2024"),
                           ("comparison_profile", "screen"), ("pdf_rasterizer", {"policy": "pdftoppm"})]:
            with self.subTest(key=key):
                provenance = copy.deepcopy(self.provenance)
                provenance[key] = value
                self.assertEqual(self.check(provenance=provenance)["status"], "re_review_required")

    def test_native_only_contract_does_not_require_wasm_or_svg_validation(self):
        for backend in ["wasm-canvas2d", "svg-webfont"]:
            with self.subTest(backend=backend):
                provenance = {**self.provenance, "raster_backend": backend}
                manifest = {"provenance": provenance, "pr_review_gate": {"status": "not_evaluated"}}
                result = quality.enforce_quality(ROOT, manifest, [], 96, None, SWEEP.SILHOUETTE_METHOD)
                self.assertEqual(result["status"], "not_registered")
                self.assertEqual(manifest["pr_review_gate"]["status"], "not_evaluated")

    def test_missing_duplicate_nonfinite_and_absent_raw_fail_closed(self):
        for rows in [self.metrics[:1], self.metrics + self.metrics[:1],
                     [{**x, quality.METRIC: "NaN"} for x in self.metrics],
                     [{k: v for k, v in x.items() if k != quality.METRIC} for x in self.metrics]]:
            self.assertEqual(self.check(rows)["status"], "re_review_required")
        self.assertEqual(self.check(counts={"rhwp": 19, "pdf": 18})["status"], "re_review_required")

    def test_input_and_reference_byte_changes_do_not_remove_protection(self):
        for side in ["hwp", "pdf"]:
            provenance = copy.deepcopy(self.provenance)
            provenance[side]["sha256"] = "0" * 64
            self.assertEqual(self.check(provenance=provenance)["checks"][0]["reason"], "protected_input_or_reference_changed")

    def test_changed_summary_fails_closed(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            registry = root / "scripts/visual_sweep_quality_contracts.json"
            registry.parent.mkdir()
            registry.write_bytes((ROOT / registry.relative_to(root)).read_bytes())
            summary = root / SUMMARY.relative_to(ROOT)
            summary.parent.mkdir(parents=True)
            summary.write_bytes(SUMMARY.read_bytes() + b"\n")
            self.assertEqual(self.check(root=root)["reason"], "invalid_quality_evidence")


if __name__ == "__main__":
    unittest.main()
