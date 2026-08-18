from __future__ import annotations

import unittest

from support import PKG
from third_party_replay.decide import (
    ATTEST_NOT_THIRD_PARTY,
    INVALID_EXPECT_SHA,
    LABOR_ACCEPTED,
    LABOR_REJECTED,
    NO_PLAN,
    PROSE_NOT_EVIDENCE,
    TOOL_VERSION_MISMATCH,
    TOOL_VERSION_MISSING,
)
from third_party_replay.envelopes import (
    decide_envelope,
    load_json,
    observation_from_capsule,
    observation_from_replay,
)


class EnvelopeWrapTests(unittest.TestCase):
    def test_replay_verify_match(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "replay_verify_match.json")
        obs = observation_from_replay(blob)
        self.assertEqual(obs.mode.value, "verify")
        self.assertTrue(obs.reproduced)
        self.assertEqual(decide_envelope(blob).verdict, LABOR_ACCEPTED)

    def test_replay_verify_mismatch(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "replay_verify_mismatch.json")
        self.assertEqual(decide_envelope(blob).verdict, LABOR_REJECTED)

    def test_replay_attest_is_not_third_party(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "replay_attest.json")
        obs = observation_from_replay(blob)
        self.assertIsNone(obs.reproduced)
        self.assertEqual(obs.expect_sha, "")
        self.assertEqual(decide_envelope(blob).verdict, ATTEST_NOT_THIRD_PARTY)

    def test_capsule_verify_match(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "capsule_verify_match.json")
        obs = observation_from_capsule(blob)
        self.assertEqual(obs.source.value, "capsule")
        self.assertTrue(obs.reproduced)
        self.assertEqual(decide_envelope(blob).verdict, LABOR_ACCEPTED)

    def test_capsule_attest(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "capsule_attest.json")
        self.assertEqual(decide_envelope(blob).verdict, ATTEST_NOT_THIRD_PARTY)

    def test_prose_claim(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "prose_claim.json")
        self.assertEqual(decide_envelope(blob).verdict, PROSE_NOT_EVIDENCE)

    def test_invalid_expect_sha(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "invalid_expect_sha.json")
        self.assertEqual(decide_envelope(blob).verdict, INVALID_EXPECT_SHA)

    def test_toolversion_mismatch_uses_expected_arg(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "toolversion_mismatch.json")
        self.assertEqual(
            decide_envelope(blob, expected_tool_version="0.8.4").verdict,
            TOOL_VERSION_MISMATCH,
        )

    def test_missing_plan(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "missing_plan.json")
        self.assertEqual(decide_envelope(blob).verdict, NO_PLAN)

    def test_missing_tool_version(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "missing_tool_version.json")
        self.assertEqual(decide_envelope(blob).verdict, TOOL_VERSION_MISSING)

    def test_does_not_invent_receipt_command(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "replay_verify_match.json")
        self.assertNotIn("receiptCommand", blob)
        self.assertIn("expectedOutputSha256", blob)
        self.assertIn("reproduced", blob)
        self.assertIn("toolVersion", blob)


if __name__ == "__main__":
    unittest.main()
