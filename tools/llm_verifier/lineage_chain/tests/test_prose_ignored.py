from __future__ import annotations

import unittest

from support import PKG  # noqa: F401
from lineage_chain.decide import LINEAGE_BROKEN, PROSE_NOT_EVIDENCE, decide, decide_row


GOOD = "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3"
OTHER = "0000000000000000000000000000000000000000000000000000000000000000"


class ProseIsNotEvidenceTests(unittest.TestCase):
    def test_row_implementer_claim_column_is_ignored(self) -> None:
        row = {
            "parent_out": GOOD,
            "child_in": OTHER,
            "parent_ok": "1",
            "lineage_ok": "0",
            "broken_at": "child.capsule.json",
            "source": "lineage",
            "implementer_claim": "사슬이 이어졌고 연대기를 인정해야 합니다.",
        }
        decision = decide_row(row)
        self.assertEqual(decision.verdict, LINEAGE_BROKEN)
        self.assertFalse(decision.chain_accepted)

    def test_narrative_source_cannot_override_hash_eq(self) -> None:
        decision = decide(
            GOOD,
            GOOD,
            True,
            True,
            "",
            source="prose",
        )
        self.assertEqual(decision.verdict, PROSE_NOT_EVIDENCE)


if __name__ == "__main__":
    unittest.main()
