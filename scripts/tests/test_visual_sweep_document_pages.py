"""일부 쪽의 높은 점수가 전체 문서의 추가·누락 쪽을 면제하지 않는지 확인한다."""

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from PIL import Image

from scripts.tests.test_visual_sweep import SWEEP


class DocumentPageCountTests(unittest.TestCase):
    def test_missing_count_invalidates_a_previous_pass_before_rasterization(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ("source.hwp", "oracle.pdf"):
                (root / name).write_bytes(b"input")
            out = root / "out"
            base = out / "case"
            for name in ("svg", "render_tree"):
                (base / name).mkdir(parents=True)
            (base / "svg/page_001.svg").write_text("<svg/>")
            (base / "render_tree/page_001.json").write_text("{}")
            SWEEP.update_root_summary(out, {"key": "case", "pr_review_gate": {"status": "passed"}})
            with (
                patch.object(SWEEP, "sweep_provenance", return_value={}),
                patch.object(SWEEP, "run_manifest_for_target", return_value={}),
                patch.object(SWEEP, "load_note_shape", return_value={}),
                patch.object(SWEEP, "export_native_target"),
                patch.object(SWEEP, "check_sweep_embedded_fonts"),
                patch.object(SWEEP, "document_page_counts", side_effect=SystemExit("전체 쪽수 없음")),
                patch.object(SWEEP, "run") as run,
                self.assertRaisesRegex(SystemExit, "전체 쪽수"),
            ):
                SWEEP.render_target(
                    root, SWEEP.Target("case", Path("source.hwp"), Path("oracle.pdf")),
                    out, "rhwp", 96, 32, [1], resume=True, svg_rasterizer="webfont",
                )
            run.assert_not_called()
            saved = json.loads((out / "summary.json").read_text())[0]
            self.assertEqual(saved["run_state"], "failed")
            self.assertEqual(saved["pr_review_gate"]["status"], "re_review_required")
            self.assertEqual(saved["pr_review_gate"]["reason"], "unavailable_document_page_counts")

    def test_invalid_measurements_cannot_pass_the_threshold(self):
        for value in (None, True, float("nan"), float("inf"), -1.0, 101.0):
            with self.subTest(value=value):
                gate = SWEEP.pr_review_gate(
                    [{"page": 1, "tolerant_content_match_percent": value}],
                    expected_pages=[1],
                )
                self.assertEqual(gate["status"], "re_review_required")
                self.assertEqual(gate["unavailable_metric_pages"], [1])

    def test_invalid_native_document_counts_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            metadata = base / "native-export.json"
            for count in (None, True, 0, -1, "32", 1.5):
                with self.subTest(count=count):
                    metadata.write_text(json.dumps({"pageCount": count}), encoding="utf-8")
                    with patch.object(SWEEP, "run", return_value=subprocess.CompletedProcess(
                        ["pdfinfo"], 0, "Pages: 31\n", "",
                    )), self.assertRaisesRegex(SystemExit, "전체 쪽수"):
                        SWEEP.document_page_counts(base, base / "oracle.pdf", metadata)

    def test_high_score_cannot_waive_an_extra_document_page(self):
        gate = SWEEP.pr_review_gate(
            [{"page": 10, "tolerant_content_match_percent": 95.36398}],
            expected_pages=[10], document_page_counts={"rhwp": 32, "pdf": 31},
        )
        self.assertEqual(gate["status"], "re_review_required")
        self.assertTrue(gate["page_count_mismatch"])
        self.assertEqual(gate["below_threshold_pages"], [])
        self.assertEqual(gate["unavailable_metric_pages"], [])
        self.assertEqual(
            SWEEP.review_gate_reason(gate), "전체 쪽수 불일치: RHWP 32쪽 / PDF 31쪽",
        )

    def test_font_evidence_cannot_override_the_measured_document_page_counts(self):
        gate = SWEEP.pr_review_gate(
            [{"page": 10, "tolerant_content_match_percent": 85.0}],
            document_page_counts={"rhwp": 30, "pdf": 31},
            font_mismatch_evidence={
                "font_issue_unresolvable": True, "layout_geometry_matched": True,
                "page_count_matched": True, "affected_pages": [10],
            },
        )
        self.assertEqual(gate["status"], "re_review_required")

    def test_equal_document_page_counts_keep_the_existing_threshold(self):
        gate = SWEEP.pr_review_gate(
            [{"page": 10, "tolerant_content_match_percent": 90.0}],
            document_page_counts={"rhwp": 31, "pdf": 31},
        )
        self.assertEqual(gate["status"], "passed")
        self.assertFalse(gate["page_count_mismatch"])

    def test_native_and_wasm_metadata_supply_the_whole_document_page_count(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            for name in ("native-export.json", "wasm-export-complete.json"):
                metadata = base / name
                metadata.write_text(json.dumps({"pageCount": 32}), encoding="utf-8")
                with patch.object(SWEEP, "run", return_value=subprocess.CompletedProcess(
                    ["pdfinfo"], 0, "Pages:           31\n", "",
                )):
                    self.assertEqual(
                        SWEEP.document_page_counts(base, base / "oracle.pdf", metadata),
                        {"rhwp": 32, "pdf": 31},
                    )

    def test_silhouette_tsv_records_a_count_defect_even_with_identical_rasters(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            paths = [base / "rhwp_010.png", base / "pdf-10.png"]
            for path in paths:
                Image.new("RGB", (32, 32), "white").save(path)
            result = SWEEP.write_silhouette_tsv(
                [(10, *paths)], base / "out", "selected-page",
                document_counts={"rhwp": 32, "pdf": 31},
            )
            self.assertEqual(result["metrics"][0]["tolerant_content_match_percent"], 100.0)
            self.assertEqual(result["pr_review_gate"]["status"], "re_review_required")
            saved = json.loads((base / "out" / "silhouette_manifest.json").read_text())
            self.assertEqual(saved["pr_review_gate"]["document_page_counts"], {"rhwp": 32, "pdf": 31})

    def test_measurement_only_status_preserves_equal_document_counts(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            image = base / "page_001.png"
            Image.new("RGB", (32, 32), "white").save(image)
            result = SWEEP.write_silhouette_tsv(
                [(1, image, image)], base / "out", "measurement-only",
                document_counts={"rhwp": 1, "pdf": 1},
            )
            gate = result["pr_review_gate"]
            self.assertEqual(gate["status"], "not_evaluated")
            self.assertEqual(gate["document_page_counts"], {"rhwp": 1, "pdf": 1})
            self.assertFalse(gate["page_count_mismatch"])


if __name__ == "__main__":
    unittest.main()
