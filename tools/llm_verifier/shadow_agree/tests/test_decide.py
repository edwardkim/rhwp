from __future__ import annotations

import unittest

from support import PKG
from shadow_agree.decide import (
    JOINT_BOTH_FAIL,
    JOINT_PASS,
    SAME_CHECK_NOT_SHADOW,
    SHADOW_A_ONLY,
    SHADOW_B_ONLY,
    decide,
    decide_row,
)
from shadow_agree.schema import parse_bool


class DecideGoldenTests(unittest.TestCase):
    def test_golden_table(self) -> None:
        path = PKG / "fixtures" / "golden_decision_table.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        self.assertIn("expected_joint", header)
        for line in lines[1:]:
            row = dict(zip(header, line.split("\t"), strict=True))
            got = decide(
                row["check_a"],
                row["check_b"],
                parse_bool(row["a_pass"]),
                parse_bool(row["b_pass"]),
            )
            self.assertEqual(got.verdict_class, row["expected_verdict_class"], msg=row["case_id"])
            self.assertEqual(got.expected_joint, parse_bool(row["expected_joint"]), msg=row["case_id"])

    def test_both_pass_is_joint(self) -> None:
        decision = decide("ir-diff", "verify-pages", True, True)
        self.assertEqual(decision.verdict_class, JOINT_PASS)
        self.assertTrue(decision.expected_joint)
        self.assertTrue(decision.distinct_commands)

    def test_one_side_is_not_enough(self) -> None:
        only_a = decide("ir-diff", "layout-anomaly", True, False)
        only_b = decide("ir-diff", "layout-anomaly", False, True)
        self.assertEqual(only_a.verdict_class, SHADOW_A_ONLY)
        self.assertEqual(only_b.verdict_class, SHADOW_B_ONLY)
        self.assertFalse(only_a.expected_joint)
        self.assertFalse(only_b.expected_joint)

    def test_both_fail(self) -> None:
        decision = decide("fill-verify", "layout-anomaly", False, False)
        self.assertEqual(decision.verdict_class, JOINT_BOTH_FAIL)
        self.assertFalse(decision.expected_joint)

    def test_same_command_is_not_shadow(self) -> None:
        decision = decide("ir-diff", "ir-diff", True, True)
        self.assertEqual(decision.verdict_class, SAME_CHECK_NOT_SHADOW)
        self.assertFalse(decision.expected_joint)
        self.assertFalse(decision.distinct_commands)

    def test_example_pairs_from_issue(self) -> None:
        ir_and_pages = decide("ir-diff", "verify-pages", True, True)
        verify_and_anomaly = decide("fill-verify", "layout-anomaly", True, True)
        self.assertEqual(ir_and_pages.verdict_class, JOINT_PASS)
        self.assertEqual(verify_and_anomaly.verdict_class, JOINT_PASS)
        self.assertEqual(ir_and_pages.check_a.pass_field, "identical")
        self.assertEqual(verify_and_anomaly.check_a.pass_field, "verify.identical")
        self.assertEqual(verify_and_anomaly.check_b.pass_field, "hasSignal")

    def test_decide_row_roundtrip(self) -> None:
        got = decide_row(
            {"check_a": "dump-pages", "check_b": "info", "a_pass": "1", "b_pass": "1"}
        )
        self.assertEqual(got.verdict_class, JOINT_PASS)

    def test_not_abstain_and_not_repeat_flags(self) -> None:
        got = decide("replay", "audit", True, False)
        payload = got.to_json()
        self.assertTrue(payload["notAbstain"])
        self.assertTrue(payload["notRepeat"])
        self.assertNotIn("ABSTAIN", got.verdict_class)


if __name__ == "__main__":
    unittest.main()
