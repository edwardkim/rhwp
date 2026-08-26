#!/usr/bin/env python3
"""Contract tests for the W8-R7-Q2 metric projector."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from font_rank7_metric_hypothesis import (  # noqa: E402
    Rank7MetricError,
    bold_fallback_audit,
    q1_adapter,
    style_domain_audit,
)


def style_q0(*, bold_characters: int = 3) -> dict:
    return {
        "cohort": {
            "totalCharacters": 5,
            "styleDomain": {
                "axes": [
                    {
                        "ratio": 95,
                        "spacing": -9,
                        "bold": False,
                        "italic": False,
                        "characters": 2,
                    },
                    {
                        "ratio": 100,
                        "spacing": -5,
                        "bold": True,
                        "italic": False,
                        "characters": 3,
                    },
                ],
                "boldCharacters": bold_characters,
                "italicCharacters": 0,
            },
        }
    }


class Rank7MetricHypothesisTests(unittest.TestCase):
    def test_style_domain_preserves_actual_ratio_spacing_and_bold_axes(self):
        audit = style_domain_audit(style_q0())
        self.assertEqual(audit["characters"], 5)
        self.assertEqual(audit["ratioValues"], [95, 100])
        self.assertEqual(audit["spacingValues"], [-9, -5])
        self.assertEqual(audit["boldCharacters"], 3)
        self.assertFalse(audit["weightedAdvanceDeltaAvailableFromAggregate"])

    def test_style_domain_rejects_inconsistent_bold_total(self):
        with self.assertRaises(Rank7MetricError):
            style_domain_audit(style_q0(bold_characters=2))

    def test_q1_adapter_requires_both_formats_and_projection_equality(self):
        q1 = {
            "canonicalSha256": "a" * 64,
            "formats": [
                {
                    "format": "hwpx",
                    "boundary": {"canonicalTraceSha256": "b" * 64},
                    "fixedGeometry": [],
                },
                {
                    "format": "hwp5",
                    "boundary": {"canonicalTraceSha256": "c" * 64},
                    "fixedGeometry": [],
                },
            ],
            "formatComparison": {
                "layoutMetricProjectionEqual": True,
                "layoutRunProjectionEqual": True,
                "fixedGeometryEqual": True,
                "layoutMetricProjectionSha256": "d" * 64,
            },
        }
        adapted = q1_adapter(q1)
        self.assertEqual(adapted["trace"]["canonicalTraceSha256"], "b" * 64)
        q1["formatComparison"]["fixedGeometryEqual"] = False
        with self.assertRaises(Rank7MetricError):
            q1_adapter(q1)

    def test_bold_audit_requires_both_source_contracts(self):
        q0 = style_q0(bold_characters=4468)
        q0["cohort"]["styleDomain"]["axes"][1]["characters"] = 4468
        q0["cohort"]["styleDomain"]["axes"][0]["characters"] = 0
        q0["cohort"]["totalCharacters"] = 4468
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            lookup = root / "lookup.rs"
            decision = root / "decision.rs"
            lookup.write_text("bold_fallback: bold", encoding="utf-8")
            decision.write_text("fauxBoldDoesNotChangeLayoutAdvance", encoding="utf-8")
            audit = bold_fallback_audit(q0, lookup, decision)
            self.assertEqual(audit["q0BoldCharacters"], 4468)
            self.assertFalse(audit["layoutAdvanceChangedByBoldRequest"])
            decision.write_text("drift", encoding="utf-8")
            with self.assertRaises(Rank7MetricError):
                bold_fallback_audit(q0, lookup, decision)


if __name__ == "__main__":
    unittest.main()
