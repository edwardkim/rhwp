"""Fixture envelopes must rank by machine fields only."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from rank import expected_ranks_match, rank_mapping

FIXTURES = HERE / "fixtures"


class FixtureTests(unittest.TestCase):
    def test_all_fixtures_match_ranker(self) -> None:
        paths = sorted(FIXTURES.glob("*.json"))
        self.assertGreaterEqual(len(paths), 3)
        for path in paths:
            blob = json.loads(path.read_text(encoding="utf-8"))
            mismatches = expected_ranks_match(blob)
            self.assertEqual(mismatches, [], msg=path.name)
            ranked = rank_mapping(blob)
            self.assertNotIn("process_steps", blob)
            self.assertNotIn("processSteps", blob)
            self.assertTrue(ranked.winner_id)

    def test_ir_diff_fixture_prefers_identical(self) -> None:
        blob = json.loads((FIXTURES / "golden_ir_diff.json").read_text(encoding="utf-8"))
        ranked = rank_mapping(blob)
        self.assertEqual(ranked.winner_id, "c_identical")

    def test_invalid_fixture_loses(self) -> None:
        blob = json.loads((FIXTURES / "mixed_invalid.json").read_text(encoding="utf-8"))
        ranked = rank_mapping(blob)
        last = ranked.ranking[-1].candidate
        self.assertTrue(last.is_invalid())


if __name__ == "__main__":
    unittest.main()
