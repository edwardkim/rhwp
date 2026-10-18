"""raw 미달을 색상 경계 보정값으로 통과시키지 않는 검토 계약."""

import csv
import tempfile
import unittest
from pathlib import Path

from PIL import Image
from test_visual_sweep import SWEEP


class RawGateTests(unittest.TestCase):
    def test_adjusted_score_cannot_rescue_raw_failure(self):
        gate = SWEEP.pr_review_gate([{
            "page": 130, "silhouette_raw_match_percent": 79.94718,
            "tolerant_content_match_percent": 99.10104,
        }])
        self.assertEqual(gate["status"], "re_review_required")
        self.assertEqual(gate["metric"], "silhouette_raw_match_percent")
        self.assertIn("79.94718", SWEEP.review_gate_reason(gate))

    def test_exactly_ninety_raw_passes(self):
        gate = SWEEP.pr_review_gate([{
            "page": 1, "silhouette_raw_match_percent": 90.0,
            "tolerant_content_match_percent": 99.0,
        }])
        self.assertEqual(gate["status"], "passed")

    def test_adjusted_only_is_unmeasured(self):
        gate = SWEEP.pr_review_gate([{
            "page": 1, "tolerant_content_match_percent": 100.0,
        }])
        self.assertEqual(gate["status"], "re_review_required")
        self.assertEqual(gate["unavailable_metric_pages"], [1])

    def test_threshold_boundary_is_diagnostic_in_tsv_and_labels(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            native, pdf = root / "rhwp_001.png", root / "pdf-001.png"
            Image.new("RGB", (8, 8), (231, 231, 231)).save(native)
            Image.new("RGB", (8, 8), (233, 233, 233)).save(pdf)
            result = SWEEP.write_silhouette_tsv([(1, native, pdf)], root, "boundary")
            with (root / "silhouette.tsv").open(encoding="utf-8") as stream:
                row = next(csv.DictReader(stream, delimiter="\t"))
            self.assertEqual(float(row["tolerant_content_match_percent"]), 100.0)
            self.assertEqual(float(row["silhouette_raw_match_percent"]), 0.0)
            self.assertEqual(row["below_90"], "1")
            self.assertEqual(result["pr_review_gate"]["status"], "re_review_required")
            label = SWEEP.review_comment_line(result["metrics"][0])
            self.assertIn("판정값 raw 실루엣(2px 이웃) = 0.00%", label)
            self.assertIn("진단용", label)


if __name__ == "__main__":
    unittest.main()
