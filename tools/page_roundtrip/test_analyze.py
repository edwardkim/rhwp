#!/usr/bin/env python3
"""analyze / catalog_ops 단위 시험."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import analyze as az  # noqa: E402
import catalog_ops as cop  # noqa: E402
from harness import CatalogEntry  # noqa: E402


class AnalyzeTests(unittest.TestCase):
    def test_5128_axis(self) -> None:
        r = az.analyze(
            "samples/한글문서파일형식_5.0_revision1.3.hwp",
            69,
            68,
            first_split_para=84,
            whole_tables=(174, 193),
        )
        self.assertEqual(r.delta, -1)
        self.assertEqual(r.primary, "hwp5_origin_stored_pagination")
        self.assertTrue(any(a.issue == 5128 and a.in_scope for a in r.axes))

    def test_foreign_seats(self) -> None:
        r = az.analyze("samples/issue-505-equations.hwp", 4, 1, issue=4056)
        self.assertFalse(any(a.in_scope for a in r.axes if a.issue == 4056))
        r2 = az.analyze("samples/정책.hwp", 215, 223, issue=4882)
        self.assertFalse(any(a.in_scope for a in r2.axes if a.issue == 4882))

    def test_reason(self) -> None:
        self.assertIn("69→68", az.expected_fail_reason(5128, 69, 68))
        self.assertIn("5253", az.expected_fail_reason(4056, 4, 1))


class CatalogOpsTests(unittest.TestCase):
    def test_drop_5128(self) -> None:
        entries = [
            CatalogEntry("samples/a.hwp", "hwpx", 5128, "x"),
            CatalogEntry("samples/issue-505-equations.hwp", "hwpx", 4056, "y"),
            CatalogEntry("samples/정책연구용역사업 중간진도보고서.hwp", "hwpx", 4882, "z"),
        ]
        kept = cop.drop_resolved(entries)
        self.assertEqual({e.issue for e in kept}, {4056, 4882})

    def test_m05_7_scope(self) -> None:
        good = [
            CatalogEntry("samples/issue-505-equations.hwp", "hwpx", 4056, "eq"),
            CatalogEntry(
                "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp",
                "hwpx",
                4882,
                "policy",
            ),
            CatalogEntry("samples/hwp3-sample16.hwp", "hwpx", 3518, "hwp3"),
        ]
        self.assertEqual(cop.assert_m05_7_scope(good), [])
        bad = good + [CatalogEntry("samples/한글문서파일형식_5.0_revision1.3.hwp", "hwpx", 5128, "spec")]
        errs = cop.assert_m05_7_scope(bad)
        self.assertTrue(any("5128" in e for e in errs))


if __name__ == "__main__":
    unittest.main()
