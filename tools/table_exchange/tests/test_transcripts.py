"""Transcripts and loops stay on the existing CLI."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from table_exchange.fatten_catalog import run


class TranscriptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        run(cls.root)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_roundtrip_loop_forbids_output_on_dry_run(self) -> None:
        data = json.loads(
            (self.root / "fixtures" / "loops" / "roundtrip_plain.json").read_text(encoding="utf-8")
        )
        dry = next(step for step in data["steps"] if step["id"] == "dry-run")
        self.assertIn("--dry-run", dry["command"])
        self.assertIn("output", dry["forbidFieldsPresent"])
        write = next(step for step in data["steps"] if step["id"] == "write-verify")
        self.assertIn("--verify", write["command"])
        self.assertEqual(write["expect"]["changedCount"], 9)

    def test_merge_fallback_uses_set_cell_only(self) -> None:
        data = json.loads(
            (self.root / "fixtures" / "loops" / "merge_fallback.json").read_text(encoding="utf-8")
        )
        self.assertIn("csv-to-table", data["forbiddenNext"])
        self.assertEqual(data["steps"][1]["command"][1:3], ["edit", "set-cell"])

    def test_dimension_loop_collects_both(self) -> None:
        data = json.loads(
            (self.root / "fixtures" / "loops" / "dimension_reject.json").read_text(encoding="utf-8")
        )
        self.assertEqual(
            set(data["steps"][1]["expectReasons"]),
            {"rowCountMismatch", "colCountMismatch"},
        )

    def test_summary_mentions_issue(self) -> None:
        text = (self.root / "reports" / "fatten_summary.md").read_text(encoding="utf-8")
        self.assertIn("#5485", text)
        self.assertIn("새 CLI 없음", text)
        self.assertIn("gym/", text)

    def test_no_gym_paths(self) -> None:
        for path in self.root.rglob("*"):
            if path.is_file():
                self.assertNotIn("gym/", path.as_posix())
