#!/usr/bin/env python3
"""M06-f 픽스처 하네스 단위 시험."""

from __future__ import annotations

import unittest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from harness import expected_trace, replay_kinds, validate


class HarnessTests(unittest.TestCase):
    def test_background_reorders_first(self):
        ops = [
            {"kind": "rectangle", "x": 1, "y": 2, "w": 3, "h": 4},
            {"kind": "pageBackground", "x": 0, "y": 0, "w": 10, "h": 10},
        ]
        self.assertEqual(replay_kinds(ops), ["pageBackground", "rectangle"])

    def test_trace_header_uses_two_decimals(self):
        ops = [{"kind": "line", "x": 0, "y": 0, "w": 50, "h": 0}]
        lines = expected_trace(400, 300, ops)
        self.assertEqual(lines[0], "begin_page 400.00x300.00")
        self.assertEqual(lines[1], "  line bbox=0.00,0.00,50.00,0.00")
        self.assertEqual(lines[2], "end_page ops=1")

    def test_empty_page_has_zero_ops(self):
        lines = expected_trace(50, 50, [])
        self.assertEqual(lines, ["begin_page 50.00x50.00", "end_page ops=0"])

    def test_validate_fixtures_on_disk(self):
        errors = validate()
        self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
