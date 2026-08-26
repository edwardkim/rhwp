"""Transcripts speak the CLI report language."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from hwp5_inventory.cases import CASES
from hwp5_inventory.fatten_catalog import run


class TranscriptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        run(cls.root)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_diff_header_matches_cli(self) -> None:
        text = (self.root / "transcripts" / "inventory_diff" / "T01.diff.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("# HWP5 Inventory Diff", text)
        self.assertIn("align_mode: `lcs`", text)
        self.assertIn("| `matched` |", text)
        self.assertIn("## Tuple Anchor Summary", text)

    def test_hints_have_probe_suggestions(self) -> None:
        text = (self.root / "transcripts" / "inventory_diff" / "T01.hints.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("# HWP5 Contract Violation Hints", text)
        self.assertIn("## Table Candidates", text)
        self.assertIn("## Next Probe Suggestions", text)

    def test_table_probe_plan_lists_eight_variants(self) -> None:
        text = (
            self.root / "transcripts" / "inventory_diff" / "T01.table-probe-plan.md"
        ).read_text(encoding="utf-8")
        self.assertIn("# HWP5 Table Probe Plan", text)
        for name in (
            "01_ctrl_outer_margin_only",
            "02_table_attr_only",
            "03_table_tail_only",
            "04_ctrl_common_attr_only",
            "05_outer_margin_table_attr",
            "06_outer_margin_table_tail",
            "07_table_attr_tail",
            "08_all_table_axes",
        ):
            self.assertIn(f"`{name}`", text)
        self.assertIn("판정용 HWP를 직접 생성하지 않는다", text)
        self.assertIn("페이지 수 로직은 이 계획이 바꾸지 않는다", text)

    def test_inventory_markdown_columns(self) -> None:
        text = (self.root / "transcripts" / "inventory" / "T01.oracle.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("# HWP5 Inventory", text)
        self.assertIn("| stream | section | idx | uid |", text)
        self.assertIn("CTRL_HEADER", text)
        self.assertIn("TABLE", text)

    def test_identical_sentinel_diff_is_empty(self) -> None:
        text = (self.root / "transcripts" / "inventory_diff" / "X16.diff.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("| `matched` |", text)
        self.assertIn("diff_count: `0`", text)

    def test_section_count_case_is_docinfo_focus(self) -> None:
        case = next(item for item in CASES if item.case_id == "D01")
        self.assertEqual(case.failure_class, "A")
        self.assertEqual(case.focus, "docinfo")
        text = (self.root / "transcripts" / "inventory_diff" / "D01.hints.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("DocInfo", text)

    def test_lineseg_case_does_not_prescribe_page_export(self) -> None:
        text = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (self.root / "transcripts" / "inventory_diff").glob("P01.*")
        )
        self.assertNotIn("export-pdf", text)
        self.assertNotIn("dump-pages --fix", text)

    def test_bundles_include_window_tables(self) -> None:
        text = (self.root / "transcripts" / "inventory_diff" / "T05.bundles.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("# HWP5 Candidate Bundles", text)
        self.assertIn("#### Oracle Window", text)
        self.assertIn("#### Generated Window", text)


if __name__ == "__main__":
    unittest.main()
