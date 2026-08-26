from __future__ import annotations

import unittest

from support import PKG
from third_party_replay.decide import (
    ATTEST_NOT_THIRD_PARTY,
    INVALID_EXPECT_SHA,
    LABOR_ACCEPTED,
    LABOR_REJECTED,
    NO_EXPECT_SHA,
    NO_PLAN,
    PROSE_NOT_EVIDENCE,
    TOOL_VERSION_MISMATCH,
    TOOL_VERSION_MISSING,
    decide,
)
from third_party_replay.schema import parse_reproduced


class DecideGoldenTests(unittest.TestCase):
    def test_golden_table(self) -> None:
        path = PKG / "fixtures" / "golden_decision_table.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        self.assertIn("expected_verdict_class", header)
        for line in lines[1:]:
            row = dict(zip(header, line.split("\t"), strict=True))
            got = decide(
                row["plan"],
                row["expect_sha"],
                parse_reproduced(row["reproduced"]),
                row["tool_version"],
                mode=row["mode"],
                source=row["source"],
                expected_tool_version=row["expected_tool_version"],
            )
            self.assertEqual(got.verdict, row["expected_verdict_class"], msg=row["case_id"])

    def test_prose_never_accepts_labor(self) -> None:
        decision = decide(
            "재실행 성공",
            "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
            True,
            "0.8.4",
            mode="absent",
            source="prose",
        )
        self.assertEqual(decision.verdict, PROSE_NOT_EVIDENCE)
        self.assertFalse(decision.labor_accepted)

    def test_attest_is_not_third_party(self) -> None:
        decision = decide(
            '{"planVersion":"1.0","input":"a.hwp"}',
            None,
            None,
            "0.8.4",
            mode="attest",
            source="replay",
        )
        self.assertEqual(decision.verdict, ATTEST_NOT_THIRD_PARTY)
        self.assertFalse(decision.labor_accepted)

    def test_reproduced_false_rejects(self) -> None:
        decision = decide(
            '{"planVersion":"1.0","input":"a.hwp"}',
            "0000000000000000000000000000000000000000000000000000000000000000",
            False,
            "0.8.4",
        )
        self.assertEqual(decision.verdict, LABOR_REJECTED)
        self.assertEqual(decision.exit_class, "3")
        self.assertEqual(decision.evidence_kind, "reproduced_field")

    def test_reproduced_true_accepts(self) -> None:
        decision = decide(
            '{"planVersion":"1.0","input":"a.hwp"}',
            "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
            True,
            "0.8.4",
        )
        self.assertEqual(decision.verdict, LABOR_ACCEPTED)
        self.assertTrue(decision.labor_accepted)

    def test_missing_expect_sha(self) -> None:
        self.assertEqual(
            decide('{"planVersion":"1.0","input":"a.hwp"}', "", None, "0.8.4").verdict,
            NO_EXPECT_SHA,
        )

    def test_invalid_expect_sha(self) -> None:
        self.assertEqual(
            decide('{"planVersion":"1.0","input":"a.hwp"}', "deadbeef", False, "0.8.4").verdict,
            INVALID_EXPECT_SHA,
        )

    def test_tool_version_missing(self) -> None:
        self.assertEqual(
            decide(
                '{"planVersion":"1.0","input":"a.hwp"}',
                "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
                True,
                "",
            ).verdict,
            TOOL_VERSION_MISSING,
        )

    def test_tool_version_mismatch_beats_reproduced_true(self) -> None:
        decision = decide(
            '{"planVersion":"1.0","input":"a.hwp"}',
            "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
            True,
            "0.7.15",
            expected_tool_version="0.8.4",
        )
        self.assertEqual(decision.verdict, TOOL_VERSION_MISMATCH)
        self.assertFalse(decision.labor_accepted)

    def test_no_plan(self) -> None:
        self.assertEqual(
            decide(
                "",
                "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
                True,
                "0.8.4",
            ).verdict,
            NO_PLAN,
        )

    def test_uppercase_hex_is_normalized(self) -> None:
        decision = decide(
            '{"planVersion":"1.0","input":"a.hwp"}',
            "3C1C839B9B750E90A88239CF7052F46858EBEFAC8D4E2F985D4FAC7699C7A5B3",
            True,
            "0.8.4",
        )
        self.assertEqual(decision.verdict, LABOR_ACCEPTED)


if __name__ == "__main__":
    unittest.main()
