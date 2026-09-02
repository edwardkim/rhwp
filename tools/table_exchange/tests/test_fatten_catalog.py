"""Generator writes catalogs, cases, reports."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from table_exchange.cases import CASES
from table_exchange.catalog import INVALID_REASONS
from table_exchange.fatten_catalog import SHOWCASE_IDS, run


class FattenCatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        cls.coverage = run(cls.root)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_coverage_shape(self) -> None:
        self.assertEqual(self.coverage["kind"], "tableCsvRoundtripFattenCatalog")
        self.assertEqual(self.coverage["claimId"], "M-tbl")
        self.assertEqual(self.coverage["issue"], 5485)
        self.assertEqual(self.coverage["caseCount"], len(CASES))
        self.assertEqual(self.coverage["editLogic"], "out of scope — existing CLI only")
        self.assertEqual(self.coverage["gym"], "out of scope")

    def test_cli_contract(self) -> None:
        data = json.loads((self.root / "fixtures" / "cli_contract.json").read_text(encoding="utf-8"))
        self.assertEqual(data["commands"], ["export-tables", "table-to-csv", "csv-to-table"])
        self.assertEqual(tuple(data["invalidReasons"]), INVALID_REASONS)
        self.assertNotIn("insert-row", json.dumps(data))

    def test_every_case_emits_core_files(self) -> None:
        lines = (self.root / "fixtures" / "cases.jsonl").read_text(encoding="utf-8").splitlines()
        ids = {json.loads(line)["caseId"] for line in lines}
        self.assertEqual(ids, {case.case_id for case in CASES})
        for family in {case.family for case in CASES}:
            path = self.root / "fixtures" / "ledgers" / f"{family}.json"
            self.assertTrue(path.is_file(), family)
            ledger = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(ledger["family"], family)
            self.assertEqual(ledger["count"], len(ledger["cases"]))
        for case_id in SHOWCASE_IDS:
            env = self.root / "fixtures" / "envelopes" / f"{case_id}.json"
            self.assertTrue(env.is_file(), case_id)
            self.assertTrue((self.root / "fixtures" / "cases" / f"{case_id}.json").is_file(), case_id)

    def test_index_matches(self) -> None:
        lines = (self.root / "fixtures" / "index.jsonl").read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), len(CASES))

    def test_showcase_transcripts(self) -> None:
        for case_id in SHOWCASE_IDS:
            matches = list((self.root / "transcripts").rglob(f"{case_id}.md"))
            self.assertTrue(matches, case_id)

    def test_recipe02_changed_nine(self) -> None:
        data = json.loads(
            (self.root / "fixtures" / "cases" / "R-recipe02-edited.json").read_text(encoding="utf-8")
        )
        self.assertEqual(data["envelope"]["changedCount"], 9)
        self.assertTrue(data["envelope"]["dryRun"])
        self.assertIsNone(data["envelope"]["changedPages"])

    def test_table001_both_reasons(self) -> None:
        data = json.loads(
            (self.root / "fixtures" / "cases" / "D-table_001-both-2x2.json").read_text(
                encoding="utf-8"
            )
        )
        reasons = {item["reason"] for item in data["invalid"]}
        self.assertEqual(reasons, {"rowCountMismatch", "colCountMismatch"})
        self.assertEqual(data["expectExit"], 2)
        self.assertFalse(data["writes"])

    def test_verify_exit3(self) -> None:
        data = json.loads(
            (self.root / "fixtures" / "envelopes" / "V-hwp_table_test_t0-exit3-diff2.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(data["_skillMeta"]["exit"], 3)
        self.assertTrue(data["_skillMeta"]["outputKept"])
        self.assertFalse(data["verify"]["identical"])
