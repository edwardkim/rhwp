from __future__ import annotations

import unittest

from support import PKG
from abstain.schema import VERDICT_ABSTAIN, VERDICT_FAIL, VERDICT_PASS
from abstain.verify_corpus import MIN_ROWS, verify


class CorpusTests(unittest.TestCase):
    def test_corpus_matches_decide(self) -> None:
        result = verify()
        self.assertTrue(result["ok"], msg=result["errors"])
        self.assertGreaterEqual(result["rows"], MIN_ROWS)
        self.assertGreater(result["byVerdict"].get(VERDICT_ABSTAIN, 0), 0)
        self.assertGreater(result["byVerdict"].get(VERDICT_PASS, 0), 0)
        self.assertGreater(result["byVerdict"].get(VERDICT_FAIL, 0), 0)


if __name__ == "__main__":
    unittest.main()
