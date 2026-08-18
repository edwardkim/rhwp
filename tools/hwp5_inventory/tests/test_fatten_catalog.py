"""Generator writes catalogs, cases, reports."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from hwp5_inventory.cases import CASES
from hwp5_inventory.catalog import CONTROLS, FAILURE_CLASSES, TAGS
from hwp5_inventory.fatten_catalog import SHOWCASE_IDS, run


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
        self.assertEqual(self.coverage["kind"], "hwp5InventoryFattenCatalog")
        self.assertEqual(self.coverage["claimId"], "M-hwp5")
        self.assertEqual(self.coverage["issue"], 5469)
        self.assertEqual(self.coverage["caseCount"], len(CASES))
        self.assertEqual(self.coverage["tagCount"], len(TAGS))
        self.assertEqual(self.coverage["controlCount"], len(CONTROLS))
        self.assertEqual(self.coverage["pageCountLogic"], "out of scope — owned by #4882")

    def test_catalog_jsonl(self) -> None:
        tags = (self.root / "fixtures" / "tags.jsonl").read_text(encoding="utf-8").splitlines()
        controls = (self.root / "fixtures" / "controls.jsonl").read_text(encoding="utf-8").splitlines()
        fields = (self.root / "fixtures" / "fields.jsonl").read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(tags), len(TAGS))
        self.assertEqual(len(controls), len(CONTROLS))
        self.assertGreaterEqual(len(fields), 20)
        first = json.loads(tags[0])
        self.assertEqual(first["tag_name"], "DOCUMENT_PROPERTIES")

    def test_failure_class_fixture(self) -> None:
        data = json.loads((self.root / "fixtures" / "failure_classes.json").read_text(encoding="utf-8"))
        codes = [item["code"] for item in data["classes"]]
        self.assertEqual(codes, [item.code for item in FAILURE_CLASSES])

    def test_every_case_emits_core_files(self) -> None:
        for case in CASES:
            required = (
                f"fixtures/cases/{case.case_id}.json",
                f"fixtures/inventories/{case.case_id}.oracle.jsonl",
                f"fixtures/inventories/{case.case_id}.generated.jsonl",
                f"fixtures/diffs/{case.case_id}.index.jsonl",
                f"fixtures/diffs/{case.case_id}.lcs.jsonl",
            )
            for rel in required:
                path = self.root / rel
                self.assertTrue(path.is_file(), rel)
            self.assertGreater(
                (self.root / f"fixtures/inventories/{case.case_id}.oracle.jsonl").stat().st_size,
                80,
                case.case_id,
            )
            if case.case_id in SHOWCASE_IDS or case.family == "table":
                self.assertGreater(
                    (self.root / f"transcripts/inventory_diff/{case.case_id}.diff.md").stat().st_size,
                    80,
                    case.case_id,
                )

    def test_table_cases_emit_probe_transcripts(self) -> None:
        table_cases = [case for case in CASES if case.family == "table"]
        self.assertGreaterEqual(len(table_cases), 8)
        for case in table_cases:
            for rel in (
                f"transcripts/inventory_diff/{case.case_id}.table-fields.md",
                f"transcripts/inventory_diff/{case.case_id}.table-probe-plan.md",
                f"transcripts/table_probe/{case.case_id}.generation.md",
            ):
                self.assertTrue((self.root / rel).is_file(), rel)

    def test_reports_exist(self) -> None:
        for name in (
            "coverage.json",
            "coverage.md",
            "failure_class_matrix.md",
            "probe_axis_matrix.md",
            "pair_index.md",
            "fatten_summary.json",
            "fatten_summary.md",
            "incorporation_manifest.json",
        ):
            self.assertTrue((self.root / "reports" / name).is_file(), name)

    def test_forbidden_paths_listed(self) -> None:
        forbidden = self.coverage["forbiddenPaths"]
        self.assertIn("gym", forbidden)
        self.assertIn("oracle_public", forbidden)
        self.assertIn("src/serializer page-count export", forbidden)

    def test_cli_contract_exit_codes(self) -> None:
        data = json.loads((self.root / "fixtures" / "cli_contract.json").read_text(encoding="utf-8"))
        self.assertEqual(data["page_count_owned_by"], 4882)
        codes = {(row["command"], tuple(row["args"])): row["exit"] for row in data["exit_codes"]}
        self.assertEqual(codes[("hwp5-inventory", ())], 2)
        self.assertEqual(codes[("hwp5-inventory", ("--help",))], 0)
        self.assertEqual(codes[("hwp5-inventory-diff", ())], 2)
        self.assertEqual(codes[("hwp5-table-probe", ("--help",))], 0)


if __name__ == "__main__":
    unittest.main()
