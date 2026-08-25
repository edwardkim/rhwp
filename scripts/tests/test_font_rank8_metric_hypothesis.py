#!/usr/bin/env python3
"""Contract tests for the W8-Q2 rank-8 metric projector."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from font_rank8_metric_hypothesis import (  # noqa: E402
    Rank8MetricError,
    apply_metric_transform,
    compare_fonts,
    crossing_disposition,
    crossing_index,
    reject_absolute_paths,
)


def record(transforms: list[dict]) -> dict:
    return {"layoutMetric": {"transforms": transforms}}


class Rank8MetricHypothesisTests(unittest.TestCase):
    def test_transform_replays_ratio_spacing_and_justification_order(self):
        value = record(
            [
                {"kind": "ratio", "input": "13.333333333333334", "output": "0.9"},
                {"kind": "letterSpacing", "input": "-0.6666666666666667"},
                {"kind": "extraCharacterSpacing", "input": "-0.319770114942528"},
            ]
        )
        self.assertEqual(apply_metric_transform(value, 1000), 831)
        self.assertEqual(apply_metric_transform(value, 936), 776)

    def test_transform_rejects_unknown_or_invalid_inputs(self):
        with self.assertRaises(Rank8MetricError):
            apply_metric_transform(record([{"kind": "tabContextAdvance"}]), 500)
        with self.assertRaises(Rank8MetricError):
            apply_metric_transform(record([]), -1)

    def test_crossing_classification_is_non_monotonic_safe(self):
        self.assertEqual(crossing_index([400, 400, 400], 1000), 2)
        self.assertIsNone(crossing_index([300, 300, 300], 1000))
        self.assertEqual(crossing_disposition(4, 5), "crossing-delayed")
        self.assertEqual(crossing_disposition(4, None), "crossing-removed")
        self.assertEqual(crossing_disposition(None, 4), "crossing-introduced")
        self.assertEqual(crossing_disposition(5, 4), "crossing-earlier")

    def test_public_output_rejects_absolute_paths(self):
        for value in ["/home/private/font.ttf", r"C:\\Fonts\\font.ttf", r"\\server\share"]:
            with self.assertRaises(Rank8MetricError):
                reject_absolute_paths({"value": value})
        reject_absolute_paths({"artifact": "mydocs/report.json"})

    def test_font_comparison_separates_metric_from_identity(self):
        def font(sha256: str, advance: int, outline: str, technology: str) -> dict:
            return {
                "cmap": {65: "A"},
                "metrics": {"A": (advance, 0)},
                "fixtureMetrics": {65: advance},
                "fixtureOutlines": {65: outline},
                "public": {
                    "sha256": sha256,
                    "nameTable": {"family": [sha256]},
                    "technology": technology,
                },
            }

        comparison = compare_fonts(
            font("a" * 64, 668, "outline-a", "truetype-glyf"),
            font("b" * 64, 668, "outline-b", "opentype-cff"),
        )
        self.assertEqual(comparison["advanceMismatchCount"], 0)
        self.assertEqual(comparison["fixtureOutlineDigestMismatches"], 1)
        self.assertFalse(comparison["byteIdentity"])
        self.assertFalse(comparison["nameIdentity"])
        self.assertFalse(comparison["technologyIdentity"])


if __name__ == "__main__":
    unittest.main()
