"""해결 불가능한 글꼴만 낮은 점수의 PR 제출 예외로 인정한다."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.tests.test_visual_sweep import SWEEP


class FontExceptionTests(unittest.TestCase):
    def evidence(self) -> dict[str, object]:
        return {
            "path": "font-evidence.json",
            "sha256": "a" * 64,
            "font_issue_unresolvable": True,
            "layout_geometry_matched": True,
            "page_count_matched": True,
            "affected_pages": [3],
        }

    def test_unresolvable_font_allows_low_score_submission(self) -> None:
        gate = SWEEP.pr_review_gate(
            [{"page": 3, "tolerant_content_match_percent": 28.4}],
            font_mismatch_evidence=self.evidence(),
        )
        self.assertEqual(gate["status"], "font_mismatch_exception")
        self.assertEqual(gate["below_threshold_pages"][0]["tolerant_content_match_percent"], 28.4)

    def test_exception_does_not_cover_unreviewed_low_page(self) -> None:
        gate = SWEEP.pr_review_gate(
            [{"page": 4, "tolerant_content_match_percent": 28.4}],
            font_mismatch_evidence=self.evidence(),
        )
        self.assertEqual(gate["status"], "re_review_required")

    def test_exception_does_not_waive_missing_page(self) -> None:
        gate = SWEEP.pr_review_gate(
            [{"page": 3, "tolerant_content_match_percent": 28.4}],
            expected_pages=[3, 4], font_mismatch_evidence=self.evidence(),
        )
        self.assertEqual(gate["status"], "re_review_required")
        self.assertEqual(gate["unavailable_metric_pages"], [4])

    def test_layout_or_page_count_defect_and_resolvable_font_block_exception(self) -> None:
        for field in ("font_issue_unresolvable", "layout_geometry_matched", "page_count_matched"):
            with self.subTest(field=field):
                evidence = self.evidence()
                evidence[field] = False
                gate = SWEEP.pr_review_gate(
                    [{"page": 3, "tolerant_content_match_percent": 28.4}],
                    font_mismatch_evidence=evidence,
                )
                self.assertEqual(gate["status"], "re_review_required")

    def test_evidence_is_hashed_and_bound_to_actual_input_and_head(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            hwp, pdf, evidence_file = root / "input.hwp", root / "reference.pdf", root / "font.json"
            hwp.write_bytes(b"input")
            pdf.write_bytes(b"pdf")
            details = {
                **self.evidence(), "source_sha": "b" * 40,
                "hwp_sha256": SWEEP.sha256_file(hwp), "pdf_sha256": SWEEP.sha256_file(pdf),
                "pdf_fonts": ["Independent PDF font"], "rhwp_fonts": ["Installed substitute"],
                "font_supply_attempts": ["Confirmed the unavailable source face and installed substitute"],
                "unresolvable_reason": "Source face cannot be supplied in the validation environment",
                "geometry_review_evidence": "Compared table rules, paragraph starts and image bounds",
            }
            evidence_file.write_text(json.dumps(details), encoding="utf-8")
            with patch.object(SWEEP, "git_head_identifier", return_value="b" * 40):
                record = SWEEP.font_mismatch_evidence_record(root, evidence_file, hwp=hwp, pdf=pdf)
                self.assertEqual(record["sha256"], SWEEP.sha256_file(evidence_file))
                for field in ("source_sha", "hwp_sha256", "pdf_sha256"):
                    with self.subTest(field=field):
                        wrong = {**details, field: "c" * 64}
                        evidence_file.write_text(json.dumps(wrong), encoding="utf-8")
                        with self.assertRaises(SystemExit):
                            SWEEP.font_mismatch_evidence_record(root, evidence_file, hwp=hwp, pdf=pdf)

    def test_nonempty_text_is_not_an_unresolvable_font_proof(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            evidence = root / "font.md"
            evidence.write_text("The fonts probably differ", encoding="utf-8")
            with self.assertRaises(SystemExit):
                SWEEP.font_mismatch_evidence_record(root, evidence)
