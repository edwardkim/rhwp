#!/usr/bin/env python3
"""Contract tests for the W8-R7-Q3 private qualification wrapper."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from font_rank8_private_qualification import (  # noqa: E402
    Rank8PrivateQualificationError,
    build_public,
    parse_document_style_dump,
)


class Rank7PrivateQualificationTests(unittest.TestCase):
    def test_style_dump_extracts_ratio_spacing_bold_and_italic_without_text(self):
        payload = (
            b'  [CS] pos=0 id=7 bold=true spacing=-9% ratio=95% base=1000 '
            b'attr=0x00000003 text=#000000 char="private"\n'
        )
        self.assertEqual(
            parse_document_style_dump(payload),
            {
                7: {
                    "ratio": 95,
                    "spacing": -9,
                    "bold": True,
                    "italic": True,
                    "baseSizeHwpunit": 1000,
                }
            },
        )

    def test_style_dump_rejects_missing_or_conflicting_shape_evidence(self):
        with self.assertRaises(Rank8PrivateQualificationError):
            parse_document_style_dump(b"no char shape rows")
        payload = (
            b"[CS] id=7 bold=false spacing=-9% ratio=95% base=1000 attr=0x0\n"
            b"[CS] id=7 bold=true spacing=-9% ratio=95% base=1000 attr=0x2\n"
        )
        with self.assertRaises(Rank8PrivateQualificationError):
            parse_document_style_dump(payload)

    def test_public_projection_records_modelled_regression_signature(self):
        line = {
            "context": "table-cell",
            "storedRowDisposition": "admitted",
            "disposition": "overflow-introduced",
            "boldTargetCharacters": 0,
            "currentOverflowPx": 0.0,
            "candidateOverflowPx": 0.707,
            "advanceDeltaHwpunit": 162,
        }
        document = {
            "classification": "worsened",
            "format": "hwpx",
            "styleMeasured": True,
            "sourceUsageCharacters": 1,
            "renderObservedCharacters": 1,
            "currentTransformReplayMismatches": 0,
            "unframedTargetCharacters": 0,
            "invalidFrameTargetCharacters": 0,
            "invalidFrameTargetLines": 0,
            "cacheUnmodelledTargetCharacters": 0,
            "styleUnmodelledTargetCharacters": 0,
            "targetLines": 1,
            "lineDispositions": {"overflow-introduced": 1},
            "contextLines": {"table-cell": 1},
            "recordDeltaCounts": {"wider": 1},
            "candidateCoverage": {"exact-metric-applied": 1},
            "cacheTargetCharacters": {"admitted": 1},
            "cacheTargetLines": {"admitted": 1},
            "boldLineDispositions": {},
            "italicLineDispositions": {},
            "styleAxes": [
                {
                    "ratio": 100,
                    "spacing": 0,
                    "bold": False,
                    "italic": False,
                    "characters": 1,
                    "advanceDeltaHwpunit": 162,
                    "wider": 1,
                }
            ],
            "firstMetricDivergence": {"page": 0},
            "firstCapacityDivergence": {"page": 0},
            "lines": [line],
        }
        q0 = {
            "canonicalSha256": "a" * 64,
            "cohort": {
                "storedRiskCharacters": 1,
                "freshRiskCharacters": 0,
                "styleDomain": {"boldCharacters": 0, "italicCharacters": 0},
            },
        }
        q2 = {"canonicalSha256": "b" * 64}
        public = build_public({"documents": [document]}, q0, q2)
        audit = public["projection"]["regressionAudit"]
        self.assertEqual(audit["modelledObservations"], 1)
        self.assertEqual(audit["modelledDistinctBoundarySignatures"], 1)
        self.assertEqual(public["decision"]["status"], "no-change")


if __name__ == "__main__":
    unittest.main()
