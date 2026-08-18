from __future__ import annotations
import unittest
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1].parent))
from w2_docdiff_page import verify as V
from w2_docdiff_page.logic import decide_row

class UnitTests(unittest.TestCase):
    def test_corpus(self) -> None:
        result = V.verify()
        self.assertTrue(result["ok"])
        self.assertGreaterEqual(result["rows"], V.MIN_ROWS)
        self.assertGreater(len(result["byVerdict"]), 0)

    def test_decide_smoke(self) -> None:
        self.assertIsInstance(decide_row("0", "0", "0", "0"), str)

if __name__ == "__main__":
    unittest.main()
