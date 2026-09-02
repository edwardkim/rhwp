"""Merge occupancy — anchors only, covered = grid minus anchors."""

from __future__ import annotations

import unittest

from table_exchange.occupancy import (
    OccupancyError,
    anchors_from_spans,
    build_occupancy,
    csv_roundtrip_for,
)
from table_exchange.tables import recipe02_table, table001


class OccupancyTests(unittest.TestCase):
    def test_recipe02_is_full_anchors(self) -> None:
        table = recipe02_table()
        self.assertEqual(table.occupancy.cell_count, 12)
        self.assertEqual(table.occupancy.covered_count, 0)
        self.assertEqual(table.csv_roundtrip, "allowed")

    def test_table001_header_covers_documented_cells(self) -> None:
        table = table001()
        occ = table.occupancy
        self.assertTrue(occ.is_covered(0, 2))
        self.assertTrue(occ.is_covered(0, 3))
        self.assertTrue(occ.is_covered(0, 5))
        self.assertTrue(occ.is_covered(0, 6))
        self.assertTrue(occ.is_covered(1, 7))
        self.assertTrue(occ.is_covered(2, 7))
        self.assertFalse(occ.is_covered(0, 1))
        self.assertEqual(occ.get(0, 2).anchor_col, 1)
        self.assertEqual(occ.get(1, 7).anchor_row, 0)
        self.assertLess(occ.cell_count, 19 * 9)
        self.assertLessEqual(occ.area_sum, 19 * 9)

    def test_overlap_is_rejected(self) -> None:
        spans = ((0, 0, 1, 2), (0, 1, 1, 2))
        with self.assertRaises(OccupancyError):
            build_occupancy(2, 3, anchors_from_spans(2, 3, spans))

    def test_csv_fills_covered_with_empty(self) -> None:
        occ = build_occupancy(2, 3, anchors_from_spans(2, 3, ((0, 0, 1, 2),), texts={(0, 0): "AB"}))
        grid = occ.grid_texts()
        self.assertEqual(grid[0], ["AB", "", ""])
        self.assertEqual(len(grid[0]), 3)

    def test_roundtrip_decision(self) -> None:
        plain = build_occupancy(2, 2, anchors_from_spans(2, 2, ()))
        merged = build_occupancy(2, 2, anchors_from_spans(2, 2, ((0, 0, 1, 2),)))
        self.assertEqual(csv_roundtrip_for(plain, None, False), "allowed")
        self.assertEqual(csv_roundtrip_for(merged, None, False), "extract-only")
        self.assertEqual(csv_roundtrip_for(plain, "header", False), "forbidden")
        self.assertEqual(csv_roundtrip_for(plain, None, True), "outer-only")
        wrapper = build_occupancy(1, 1, anchors_from_spans(1, 1, ()))
        self.assertEqual(csv_roundtrip_for(wrapper, None, False), "skip")
