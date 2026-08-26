"""Case catalog coverage and uniqueness."""

from __future__ import annotations

import unittest

from table_exchange.cases import CASES, assert_catalog_coverage, cases_by_family
from table_exchange.catalog import INVALID_REASONS


class CaseCatalogTests(unittest.TestCase):
    def test_ids_unique(self) -> None:
        ids = [case.case_id for case in CASES]
        self.assertEqual(len(ids), len(set(ids)))

    def test_coverage_floors(self) -> None:
        counts = assert_catalog_coverage()
        self.assertGreaterEqual(sum(counts.values()), 150)

    def test_every_invalid_reason_appears(self) -> None:
        seen = {
            item.get("reason")
            for case in CASES
            for item in case.invalid
        }
        for reason in INVALID_REASONS:
            self.assertIn(reason, seen, reason)

    def test_dimension_never_writes(self) -> None:
        for case in cases_by_family()["dimension"]:
            if case.invalid:
                self.assertFalse(case.writes, case.case_id)
                self.assertEqual(case.expect_exit, 2, case.case_id)
                self.assertEqual(case.envelope.get("changedCount"), 0, case.case_id)

    def test_covered_rejects_nonempty(self) -> None:
        nonempty = [
            case
            for case in cases_by_family()["covered"]
            if any(item.get("reason") == "coveredCellNotEmpty" for item in case.invalid)
        ]
        self.assertGreaterEqual(len(nonempty), 15)
        for case in nonempty:
            self.assertFalse(case.writes, case.case_id)
            self.assertEqual(case.next_action, "edit set-cell", case.case_id)

    def test_dry_run_null_pages(self) -> None:
        for case in cases_by_family()["dry-run"]:
            if case.command != "csv-to-table":
                continue
            self.assertIsNone(case.envelope.get("changedPages"), case.case_id)
            self.assertTrue(case.envelope.get("dryRun"), case.case_id)
            self.assertFalse(case.writes, case.case_id)

    def test_verify_exit3_keeps_output(self) -> None:
        fails = [case for case in CASES if case.expect_exit == 3]
        self.assertGreaterEqual(len(fails), 4)
        for case in fails:
            self.assertTrue(case.envelope["_skillMeta"]["outputKept"], case.case_id)
            self.assertFalse(case.envelope["verify"]["identical"], case.case_id)
            self.assertEqual(case.envelope.get("invalid"), [], case.case_id)

    def test_no_new_commands(self) -> None:
        allowed = {"export-tables", "table-to-csv", "csv-to-table"}
        for case in CASES:
            self.assertIn(case.command, allowed, case.case_id)
            joined = " ".join(case.argv)
            self.assertNotRegex(joined, r"merge-cells|split-cell|insert-row|csv-to-chart")

    def test_argv_starts_with_rhwp(self) -> None:
        for case in CASES:
            self.assertEqual(case.argv[0], "rhwp", case.case_id)
