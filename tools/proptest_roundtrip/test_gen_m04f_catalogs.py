"""M04-f 카탈로그 생성기 계약."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from gen_m04f_catalogs import (
    ACTIONS,
    FIXTURES,
    INVALID_FAMILIES,
    SKIP_REASONS,
    generate,
)


class GenM04fCatalogsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.out = Path(cls.tmp.name)
        cls.summary = generate(cls.out)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_minimum_catalog_sizes(self) -> None:
        counts = self.summary["counts"]
        self.assertGreaterEqual(counts["skipCatalog"], 200)
        self.assertGreaterEqual(counts["validPlans"], 800)
        self.assertGreaterEqual(counts["invalidPlans"], 80)
        self.assertGreaterEqual(counts["exceptions"], 200)
        self.assertGreaterEqual(counts["mutations"], 40)
        self.assertGreaterEqual(counts["conditions"], 40)
        self.assertEqual(counts["fixtures"], len(FIXTURES))

    def test_only_existing_run_actions(self) -> None:
        self.assertEqual(tuple(self.summary["actions"]), ACTIONS)
        for row in json.loads((self.out / "catalogs" / "fixtures.json").read_text(encoding="utf-8")):
            self.assertEqual(set(row["applyable"]), set(ACTIONS))

    def test_no_invented_document_core_actions(self) -> None:
        forbidden = {
            "insert_text",
            "delete_text",
            "merge_cells",
            "split_cell",
            "insert_table",
            "mutate_core",
        }
        for line in (self.out / "catalogs" / "valid_plans.jsonl").read_text(encoding="utf-8").splitlines():
            plan = json.loads(line)["plan"]
            for step in plan["steps"]:
                self.assertIn(step["action"], ACTIONS)
                self.assertNotIn(step["action"], forbidden)

    def test_invalid_catalog_covers_schema_families(self) -> None:
        families = {
            json.loads(line)["family"]
            for line in (self.out / "catalogs" / "invalid_plans.jsonl").read_text(encoding="utf-8").splitlines()
        }
        for family in INVALID_FAMILIES:
            self.assertIn(family, families, family)

    def test_unknown_action_is_rejected_not_applied(self) -> None:
        found = False
        for line in (self.out / "catalogs" / "invalid_plans.jsonl").read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            if row["family"] != "unknown_action":
                continue
            found = True
            self.assertEqual(row["expected"], "schema_reject")
            self.assertTrue(
                "발명" in row["why"] or "4종 밖" in row["why"],
                row["why"],
            )
        self.assertTrue(found)

    def test_skip_reasons_match_engine_taxonomy(self) -> None:
        reasons = {
            json.loads(line)["reason"]
            for line in (self.out / "catalogs" / "skip_catalog.jsonl").read_text(encoding="utf-8").splitlines()
        }
        self.assertTrue(set(SKIP_REASONS).issubset(reasons) or reasons <= set(SKIP_REASONS))
        self.assertIn("field_missing", reasons)
        self.assertIn("no_hits", reasons)
        self.assertIn("checkbox_missing", reasons)
        self.assertIn("unclaimed_capability", reasons)

    def test_unclaimed_mixed_fixture_never_apply(self) -> None:
        for line in (self.out / "catalogs" / "skip_catalog.jsonl").read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            if row["fixture"] != "ref_mixed_hwpx":
                continue
            self.assertEqual(row["reason"], "unclaimed_capability")
            self.assertEqual(row["expected"], "skip")

    def test_claimed_text_fixture_skips_tables_and_fields(self) -> None:
        saw_table = False
        saw_field = False
        for line in (self.out / "catalogs" / "skip_catalog.jsonl").read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            if row["fixture"] != "ref_text_hwpx":
                continue
            if row["reason"] == "table_missing":
                saw_table = True
            if row["reason"] == "field_missing":
                saw_field = True
        self.assertTrue(saw_table)
        self.assertTrue(saw_field)

    def test_matrices_cover_every_fixture_action(self) -> None:
        lines = (self.out / "matrices" / "fixture_x_step.tsv").read_text(encoding="utf-8").splitlines()
        self.assertGreater(len(lines), 1)
        pairs = {tuple(line.split("\t")[:4:3]) for line in lines[1:]}
        # fixture, action are cols 0 and 3
        pairs = {(line.split("\t")[0], line.split("\t")[3]) for line in lines[1:]}
        for fx in FIXTURES:
            for action in ACTIONS:
                self.assertIn((fx["id"], action), pairs)

    def test_reports_and_schema_exist(self) -> None:
        for rel in (
            "README.md",
            "schema/catalog.v1.json",
            "reports/fatten_summary.json",
            "reports/fatten_summary.md",
            "reports/skip_honesty.md",
            "reports/ci.md",
            "cases/fill_fields_variants.jsonl",
            "cases/replace_text_variants.jsonl",
            "cases/set_cell_variants.jsonl",
            "cases/set_checkbox_variants.jsonl",
        ):
            self.assertTrue((self.out / rel).is_file(), rel)


if __name__ == "__main__":
    unittest.main()
