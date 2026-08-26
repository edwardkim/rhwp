from __future__ import annotations

import unittest

from support import PKG
from abstain.decide import decide
from abstain.schema import EnvelopeFields, VERDICT_ABSTAIN, VERDICT_FAIL, VERDICT_PASS


class DecideGoldenTests(unittest.TestCase):
    def test_golden_table(self) -> None:
        path = PKG / "fixtures" / "golden_contradiction_table.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        self.assertIn("expected", header)
        for line in lines[1:]:
            row = dict(zip(header, line.split("\t"), strict=True))
            got = decide(EnvelopeFields.from_mapping(row))
            self.assertEqual(got.verdict, row["expected"], msg=row["case_id"])
            if row["expected"] == VERDICT_ABSTAIN:
                self.assertEqual(
                    got.contradiction_id,
                    row["contradiction_id"],
                    msg=row["case_id"],
                )
                self.assertTrue(got.abstained)

    def test_identical_and_has_signal_abstains(self) -> None:
        got = decide(
            EnvelopeFields(command="layout-anomaly", exit=0, identical=True, has_signal=True)
        )
        self.assertEqual(got.verdict, VERDICT_ABSTAIN)
        self.assertEqual(got.contradiction_id, "identical_and_has_signal")

    def test_reproduced_and_exit3_abstains(self) -> None:
        got = decide(EnvelopeFields(command="replay", exit=3, reproduced=True))
        self.assertEqual(got.verdict, VERDICT_ABSTAIN)
        self.assertEqual(got.contradiction_id, "reproduced_and_exit3")

    def test_pagecount_struct_same_node_abstains(self) -> None:
        got = decide(
            EnvelopeFields(
                command="render-diff",
                exit=1,
                page_count_a=4,
                page_count_b=4,
                page_count_mismatch=False,
                struct_status="STRUCT_MISMATCH",
                struct_node="page/0/table/0",
                page_count_node="page/0/table/0",
            )
        )
        self.assertEqual(got.verdict, VERDICT_ABSTAIN)
        self.assertEqual(got.contradiction_id, "pagecount_match_and_struct_same_node")

    def test_pagecount_struct_other_node_is_fail(self) -> None:
        got = decide(
            EnvelopeFields(
                command="render-diff",
                exit=1,
                page_count_a=4,
                page_count_b=4,
                page_count_mismatch=False,
                struct_status="STRUCT_MISMATCH",
                struct_node="page/0/table/0",
                page_count_node="document",
            )
        )
        self.assertEqual(got.verdict, VERDICT_FAIL)
        self.assertEqual(got.contradiction_id, "")

    def test_never_invents_pass_on_conflict(self) -> None:
        cases = [
            EnvelopeFields(command="ir-diff", exit=0, identical=True, has_signal=True),
            EnvelopeFields(command="replay", exit=3, reproduced=True, identical=True),
            EnvelopeFields(
                command="fill-fields",
                exit=0,
                verify_identical=True,
                verify_diff_count=2,
            ),
        ]
        for fields in cases:
            got = decide(fields)
            self.assertEqual(got.verdict, VERDICT_ABSTAIN, msg=fields)
            self.assertNotEqual(got.verdict, VERDICT_PASS)
            self.assertNotEqual(got.verdict, VERDICT_FAIL)

    def test_consistent_pass(self) -> None:
        got = decide(
            EnvelopeFields(
                command="ir-diff",
                exit=0,
                identical=True,
                diff_count=0,
                fail_count=0,
                page_count_a=2,
                page_count_b=2,
                page_count_mismatch=False,
            )
        )
        self.assertEqual(got.verdict, VERDICT_PASS)

    def test_consistent_fail(self) -> None:
        got = decide(
            EnvelopeFields(
                command="ir-diff",
                exit=3,
                identical=False,
                diff_count=3,
                fail_count=3,
                verdict="fail",
            )
        )
        self.assertEqual(got.verdict, VERDICT_FAIL)

    def test_layout_exit0_with_signal_is_not_auto_abstain(self) -> None:
        """layout-anomaly default exit is 0; hasSignal is data, not a conflict."""
        got = decide(
            EnvelopeFields(
                command="layout-anomaly",
                exit=0,
                has_signal=True,
                overflow_count=2,
                signal_count=2,
            )
        )
        self.assertEqual(got.verdict, VERDICT_FAIL)


if __name__ == "__main__":
    unittest.main()
