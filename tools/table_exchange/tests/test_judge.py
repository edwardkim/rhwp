"""csv-to-table judge: collect all reasons, never write on invalid."""

from __future__ import annotations

import unittest

from table_exchange.csv_codec import write_csv
from table_exchange.judge import csv_to_table_envelope, judge_csv_to_table
from table_exchange.tables import recipe02_table, table001


class JudgeTests(unittest.TestCase):
    def test_row_short_collects_row_mismatch_and_writes_nothing(self) -> None:
        table = recipe02_table()
        grid = table.occupancy.grid_texts()[:-1]
        judgment = judge_csv_to_table(table.occupancy, write_csv(grid), 0)
        self.assertFalse(judgment.ok)
        self.assertEqual(judgment.exit_code, 2)
        self.assertFalse(judgment.writes)
        self.assertTrue(any(item.reason == "rowCountMismatch" for item in judgment.invalid))
        self.assertEqual(judgment.changed, [])

    def test_col_long_collects_every_row(self) -> None:
        table = recipe02_table()
        grid = [row + ["남는열"] for row in table.occupancy.grid_texts()]
        judgment = judge_csv_to_table(table.occupancy, write_csv(grid), 0)
        cols = [item for item in judgment.invalid if item.reason == "colCountMismatch"]
        self.assertEqual(len(cols), 4)

    def test_both_mismatch_collects_all(self) -> None:
        table = table001()
        judgment = judge_csv_to_table(table.occupancy, write_csv([["a", "b"], ["c", "d"]]), 0)
        reasons = {item.reason for item in judgment.invalid}
        self.assertIn("rowCountMismatch", reasons)
        self.assertIn("colCountMismatch", reasons)
        self.assertEqual(judgment.changed, [])

    def test_covered_not_empty(self) -> None:
        table = table001()
        grid = table.occupancy.grid_texts()
        grid[0][2] = "덮인칸값"
        judgment = judge_csv_to_table(table.occupancy, write_csv(grid), 0)
        item = next(i for i in judgment.invalid if i.reason == "coveredCellNotEmpty")
        self.assertEqual((item.row, item.col), (0, 2))
        self.assertEqual((item.anchor_row, item.anchor_col), (0, 1))

    def test_empty_covered_is_not_invalid(self) -> None:
        table = table001()
        judgment = judge_csv_to_table(
            table.occupancy, write_csv(table.occupancy.grid_texts()), 0
        )
        self.assertFalse(any(i.reason == "coveredCellNotEmpty" for i in judgment.invalid))

    def test_control_character_even_when_quoted(self) -> None:
        table = recipe02_table()
        grid = table.occupancy.grid_texts()
        grid[1][0] = "줄\n바꿈"
        judgment = judge_csv_to_table(table.occupancy, write_csv(grid), 0)
        self.assertTrue(any(i.reason == "controlCharacter" for i in judgment.invalid))

    def test_recipe02_changed_count_nine(self) -> None:
        table = recipe02_table()
        edited = [
            ["제목", "담당자", "세부 내용"],
            ["서버 이관", "홍길동", "1차 완료"],
            ["DB 백업", "김철수", "진행중"],
            ["문서 정리", "박영희", "대기"],
        ]
        judgment = judge_csv_to_table(table.occupancy, write_csv(edited), 0)
        self.assertTrue(judgment.ok)
        self.assertEqual(len(judgment.changed), 9)
        self.assertNotIn(0, {item.row for item in judgment.changed})

    def test_dry_run_envelope_null_pages(self) -> None:
        table = recipe02_table()
        edited = table.occupancy.grid_texts()
        edited[1][0] = "미리보기"
        judgment = judge_csv_to_table(table.occupancy, write_csv(edited), 0)
        env = csv_to_table_envelope(
            source=table.sample,
            table_index=0,
            occupancy=table.occupancy,
            judgment=judgment,
            csv_name="dry.csv",
            mode="dry-run",
            output="out/dry.hwp",
        )
        self.assertTrue(env["dryRun"])
        self.assertIsNone(env["changedPages"])
        self.assertIsNone(env["output"])
        self.assertFalse(env["_skillMeta"]["writes"])
        self.assertEqual(env["_skillMeta"]["exit"], 0)

    def test_verify_fail_keeps_output(self) -> None:
        table = recipe02_table()
        edited = table.occupancy.grid_texts()
        edited[1][0] = "x"
        judgment = judge_csv_to_table(table.occupancy, write_csv(edited), 0)
        env = csv_to_table_envelope(
            source=table.sample,
            table_index=0,
            occupancy=table.occupancy,
            judgment=judgment,
            csv_name="v.csv",
            mode="verify",
            output="out/v.hwp",
            verify_diff_count=2,
        )
        self.assertEqual(env["_skillMeta"]["exit"], 3)
        self.assertTrue(env["_skillMeta"]["outputKept"])
        self.assertFalse(env["verify"]["identical"])
        self.assertEqual(env["output"], "out/v.hwp")
        self.assertEqual(env["invalid"], [])
