"""Contract case uniqueness and coverage."""

from __future__ import annotations

import unittest

from hwp5_inventory.cases import CASES, assert_catalog_coverage
from hwp5_inventory.catalog import FAILURE_CLASSES, TAGS


class CaseCoverageTests(unittest.TestCase):
    def test_unique_ids_and_samples(self) -> None:
        ids = [case.case_id for case in CASES]
        samples = [case.sample for case in CASES]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertEqual(len(samples), len(set(samples)))
        self.assertGreaterEqual(len(CASES), 50)

    def test_failure_classes_a_to_f(self) -> None:
        seen = {case.failure_class for case in CASES}
        self.assertEqual(seen, {item.code for item in FAILURE_CLASSES})

    def test_families_cover_inventory_language(self) -> None:
        families = {case.family for case in CASES}
        for required in (
            "table",
            "shape",
            "para",
            "docinfo",
            "field",
            "note",
            "page",
            "equation",
            "form",
        ):
            self.assertIn(required, families)

    def test_align_and_focus_are_cli_tokens(self) -> None:
        for case in CASES:
            self.assertIn(case.align_preferred, {"index", "lcs"})
            self.assertIn(
                case.focus, {"all", "table", "shape", "ctrl", "missing", "docinfo"}
            )
            self.assertIn(
                case.hancom_judgment,
                {
                    "파일 읽기 오류",
                    "파일 손상",
                    "열림 + 조판 실패",
                    "성공",
                    "rhwp-studio 정상 + 한컴 실패",
                },
            )
            self.assertIn(case.contract_status, {"violated", "satisfied", "unknown"})

    def test_table_cases_name_probe_axes(self) -> None:
        table_cases = [case for case in CASES if case.family == "table"]
        self.assertGreaterEqual(len(table_cases), 8)
        named = [case for case in table_cases if case.probe_axes]
        self.assertGreaterEqual(len(named), 4)
        for case in named:
            for axis in case.probe_axes:
                self.assertIn(
                    axis,
                    {
                        "ctrl_outer_margin",
                        "ctrl_common_attr",
                        "table_attr",
                        "table_tail",
                    },
                )

    def test_page_count_not_owned(self) -> None:
        for case in CASES:
            blob = " ".join(
                (case.lowering_contract, case.next_probe, *case.notes)
            )
            self.assertNotIn("serializer page count", blob)

    def test_catalog_coverage_helper(self) -> None:
        assert_catalog_coverage()

    def test_each_case_builds_two_inventories(self) -> None:
        for case in CASES:
            oracle, generated = case.build()
            self.assertGreater(len(oracle), 3, case.case_id)
            self.assertGreater(len(generated), 2, case.case_id)
            oracle_uids = [item.record_uid for item in oracle]
            self.assertEqual(len(oracle_uids), len(set(oracle_uids)), case.case_id)
            for item in oracle + generated:
                self.assertTrue(item.tag_name)
                self.assertTrue(item.tuple_role)
                self.assertTrue(item.payload_hash.startswith("blake3:"))
                self.assertIn(item.tag_name, {tag.tag_name for tag in TAGS})

    def test_identical_sentinel_has_no_tree_gap(self) -> None:
        case = next(item for item in CASES if item.case_id == "X16")
        oracle, generated = case.build()
        self.assertEqual(
            [(item.tag_name, item.record_index) for item in oracle],
            [(item.tag_name, item.record_index) for item in generated],
        )


if __name__ == "__main__":
    unittest.main()
