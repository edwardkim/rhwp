from __future__ import annotations

import unittest

from shadow_agree.decide import SAME_CHECK_NOT_SHADOW, decide


class BoundaryTests(unittest.TestCase):
    def test_one_envelope_contradiction_is_out_of_scope(self) -> None:
        # identical=true and diffCount=3 would be V-abstain inside one envelope.
        # V-shadow never inspects two fields of the same command.
        decision = decide("ir-diff", "layout-anomaly", True, True)
        self.assertNotEqual(decision.verdict_class, "ABSTAIN")
        self.assertNotIn("abstain", decision.verdict_class.lower())
        self.assertTrue(decision.to_json()["notAbstain"])

    def test_same_command_twice_is_not_repeat_eval(self) -> None:
        decision = decide("render-diff", "render-diff", True, False)
        self.assertEqual(decision.verdict_class, SAME_CHECK_NOT_SHADOW)
        self.assertTrue(decision.to_json()["notRepeat"])
        self.assertNotEqual(decision.verdict_class, "REPEAT")

    def test_joint_requires_two_command_keys(self) -> None:
        a = decide("inspect-hidden", "inspect-injection", True, True)
        b = decide("inspect-hidden", "inspect-hidden", True, True)
        self.assertTrue(a.expected_joint)
        self.assertFalse(b.expected_joint)
        self.assertNotEqual(a.check_a.command_key, a.check_b.command_key)


if __name__ == "__main__":
    unittest.main()
