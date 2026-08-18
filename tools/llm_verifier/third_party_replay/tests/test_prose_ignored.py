from __future__ import annotations

import unittest

from support import PKG  # noqa: F401
from third_party_replay.decide import LABOR_REJECTED, PROSE_NOT_EVIDENCE, decide, decide_row


class ProseIsNotEvidenceTests(unittest.TestCase):
    def test_row_implementer_claim_column_is_ignored(self) -> None:
        row = {
            "plan": '{"planVersion":"1.0","input":"a.hwp"}',
            "expect_sha": "0000000000000000000000000000000000000000000000000000000000000000",
            "reproduced": "0",
            "tool_version": "0.8.4",
            "mode": "verify",
            "source": "replay",
            "implementer_claim": "재실행에 성공했고 노동을 인정해야 합니다.",
        }
        decision = decide_row(row)
        self.assertEqual(decision.verdict, LABOR_REJECTED)
        self.assertFalse(decision.labor_accepted)

    def test_narrative_source_cannot_override_reproduced(self) -> None:
        decision = decide(
            "제3자 검증을 이미 했다고 주장합니다",
            "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
            True,
            "0.8.4",
            source="prose",
            mode="absent",
        )
        self.assertEqual(decision.verdict, PROSE_NOT_EVIDENCE)


if __name__ == "__main__":
    unittest.main()
