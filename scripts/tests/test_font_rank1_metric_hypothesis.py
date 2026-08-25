#!/usr/bin/env python3
"""Contract tests for the W8-R1-Q2 metric hypothesis projector."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from font_rank1_metric_hypothesis import (  # noqa: E402
    Rank1MetricError,
    exhaustive_equivalence,
    metric_width,
    parse_generated_metric,
    project_record,
    reject_absolute_paths,
)


SOURCE = """
static FONT_0_LATIN_0: [u16; 2] = [500, 0];
static FONT_0_LATIN_RANGES: [LatinRange; 1] = [LatinRange {
    start: 0x0020,
    end: 0x0021,
    widths: &FONT_0_LATIN_0,
}];
static FONT_0_HANGUL_CHO: [u8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static FONT_0_HANGUL_JUNG: [u8; 21] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static FONT_0_HANGUL_JONG: [u8; 28] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static FONT_0_HANGUL_WIDTHS: [u16; 1] = [1000];
static FONT_0_HANGUL: HangulMetric = HangulMetric {
    cho_groups: 1,
    jung_groups: 1,
    jong_groups: 1,
    cho_map: &FONT_0_HANGUL_CHO,
    jung_map: &FONT_0_HANGUL_JUNG,
    jong_map: &FONT_0_HANGUL_JONG,
    widths: &FONT_0_HANGUL_WIDTHS,
};
FontMetric {
    name: "MBatang",
    bold: false,
    italic: false,
    em_size: 1000,
    latin_ranges: &FONT_0_LATIN_RANGES,
    hangul: Some(&FONT_0_HANGUL),
}
"""


def record(character: str, base: int, final: int) -> dict:
    return {
        "source": {"character": character, "codePoint": ord(character)},
        "layoutMetric": {"baseAdvanceHwpunit": base, "finalAdvanceHwpunit": final, "widthSource": "heuristicHalfwidth", "transforms": []},
    }


class Rank1MetricHypothesisTests(unittest.TestCase):
    def test_generated_metric_parser_preserves_zero_as_miss(self):
        metric = parse_generated_metric(SOURCE)
        self.assertEqual(metric["index"], 0)
        self.assertEqual(metric_width(metric, 0x20), 500)
        self.assertIsNone(metric_width(metric, 0x21))
        self.assertEqual(metric_width(metric, 0xAC00), 1000)
        self.assertEqual(metric_width(metric, 0xD7A3), 1000)

    def test_generated_metric_parser_fails_on_duplicate_face(self):
        with self.assertRaises(Rank1MetricError):
            parse_generated_metric(SOURCE + SOURCE)

    def test_virtual_relation_preserves_metric_miss_fallback(self):
        metric = parse_generated_metric(SOURCE)
        value = project_record(record("A", 500, 500), metric, {}, {})
        self.assertEqual(value["virtualBase"], 500)
        self.assertEqual(value["exactBase"], 500)
        self.assertFalse(value["generatedMetricHit"])
        self.assertFalse(value["exactApplied"])

    def test_exact_covered_width_is_compared_after_virtual_relation(self):
        metric = parse_generated_metric(SOURCE)
        value = project_record(record("가", 1000, 1000), metric, {ord("가"): "ga"}, {"ga": (1000, 0)})
        self.assertEqual(value["currentFinal"], value["virtualFinal"])
        self.assertEqual(value["virtualFinal"], value["exactFinal"])
        self.assertTrue(value["generatedMetricHit"])
        self.assertTrue(value["exactApplied"])

    def test_exhaustive_domain_rejects_any_exact_advance_drift(self):
        metric = parse_generated_metric(SOURCE)
        cmap = {0x20: "space", 0xAC00: "ga"}
        compatible = exhaustive_equivalence(metric, cmap, {"space": (500, 0), "ga": (1000, 0)})
        self.assertEqual(compatible["virtualToExact"]["advanceMismatchCount"], 0)
        self.assertFalse(compatible["boundedCohortReparsePerformed"])
        with self.assertRaises(Rank1MetricError):
            exhaustive_equivalence(metric, cmap, {"space": (500, 0), "ga": (999, 0)})

    def test_public_output_rejects_absolute_paths(self):
        for value in ["/home/private/font.ttf", r"C:\\Fonts\\font.ttf", r"\\server\share"]:
            with self.assertRaises(Rank1MetricError):
                reject_absolute_paths({"value": value})
        reject_absolute_paths({"artifact": "mydocs/report.json"})


if __name__ == "__main__":
    unittest.main()
