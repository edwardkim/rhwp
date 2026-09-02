from __future__ import annotations

import unittest

from support import PKG
from shadow_agree.decide import JOINT_PASS, SHADOW_A_ONLY, SHADOW_B_ONLY
from shadow_agree.envelopes import decide_envelopes, load_pair_fixture, pass_from_envelope


class EnvelopeBindTests(unittest.TestCase):
    def test_ir_diff_and_pagecount_both_pass(self) -> None:
        decision = load_pair_fixture(PKG / "fixtures" / "envelopes" / "ir_diff_and_pages_pass.json")
        self.assertEqual(decision.verdict_class, JOINT_PASS)
        self.assertTrue(decision.expected_joint)

    def test_verify_and_anomaly_both_pass(self) -> None:
        decision = load_pair_fixture(
            PKG / "fixtures" / "envelopes" / "verify_and_anomaly_pass.json"
        )
        self.assertEqual(decision.verdict_class, JOINT_PASS)
        self.assertEqual(decision.check_a.pass_field, "verify.identical")
        self.assertEqual(decision.check_b.pass_field, "hasSignal")

    def test_ir_diff_pass_pages_fail(self) -> None:
        decision = load_pair_fixture(PKG / "fixtures" / "envelopes" / "ir_diff_pass_pages_fail.json")
        self.assertEqual(decision.verdict_class, SHADOW_A_ONLY)
        self.assertFalse(decision.expected_joint)

    def test_verify_fail_anomaly_pass(self) -> None:
        decision = load_pair_fixture(
            PKG / "fixtures" / "envelopes" / "verify_fail_anomaly_pass.json"
        )
        self.assertEqual(decision.verdict_class, SHADOW_B_ONLY)

    def test_same_object_is_abstain_territory(self) -> None:
        envelope = {"identical": True, "diffCount": 3}
        with self.assertRaises(ValueError):
            decide_envelopes("ir-diff", envelope, "ir-diff", envelope)

    def test_pass_from_envelope_reads_existing_fields(self) -> None:
        self.assertTrue(pass_from_envelope("ir-diff", {"identical": True, "diffCount": 0}))
        self.assertFalse(pass_from_envelope("ir-diff", {"identical": False, "diffCount": 4}))
        self.assertTrue(pass_from_envelope("layout-anomaly", {"hasSignal": False}))
        self.assertFalse(pass_from_envelope("layout-anomaly", {"hasSignal": True}))
        self.assertTrue(
            pass_from_envelope("dump-pages", {"pageCount": 7, "expectedPageCount": 7})
        )
        self.assertFalse(
            pass_from_envelope("dump-pages", {"pageCount": 7, "expectedPageCount": 8})
        )


if __name__ == "__main__":
    unittest.main()
