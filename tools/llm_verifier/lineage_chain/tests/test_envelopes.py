from __future__ import annotations

import unittest

from support import PKG
from lineage_chain.decide import (
    CHAIN_ACCEPTED,
    ENVELOPE_CONTRADICTS,
    HASH_DEFECT,
    HEAD_MISSING,
    KIND_NOT_CAPSULE,
    LINEAGE_BROKEN,
    PARENT_FIELD_MISSING,
    PARENT_SHA_MISSING,
    PARENT_TAMPERED,
    PROSE_NOT_EVIDENCE,
    ROOT_ONLY,
    USAGE,
)
from lineage_chain.envelopes import (
    decide_envelope,
    load_json,
    observation_from_lineage,
)


class EnvelopeWrapTests(unittest.TestCase):
    def test_two_link_ok(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_two_link_ok.json")
        obs = observation_from_lineage(blob)
        self.assertEqual(
            obs.parent_out,
            "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
        )
        self.assertEqual(
            obs.child_in,
            "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3",
        )
        self.assertTrue(obs.parent_ok)
        self.assertTrue(obs.lineage_ok)
        self.assertEqual(decide_envelope(blob).verdict, CHAIN_ACCEPTED)

    def test_broken_hash(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_broken_hash.json")
        self.assertEqual(decide_envelope(blob).verdict, LINEAGE_BROKEN)

    def test_parent_tamper(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_parent_tamper.json")
        self.assertEqual(decide_envelope(blob).verdict, PARENT_TAMPERED)

    def test_root(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_root.json")
        self.assertEqual(decide_envelope(blob).verdict, ROOT_ONLY)

    def test_missing_head(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_missing_head.json")
        self.assertEqual(decide_envelope(blob).verdict, HEAD_MISSING)

    def test_usage(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_usage.json")
        self.assertEqual(decide_envelope(blob).verdict, USAGE)

    def test_missing_parent_sha(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_missing_parent_sha.json")
        self.assertEqual(decide_envelope(blob).verdict, PARENT_SHA_MISSING)

    def test_kind_not_capsule(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_kind_not_capsule.json")
        self.assertEqual(decide_envelope(blob).verdict, KIND_NOT_CAPSULE)

    def test_hash_defect(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_hash_defect.json")
        self.assertEqual(decide_envelope(blob).verdict, HASH_DEFECT)

    def test_envelope_contradicts(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_envelope_contradicts.json")
        self.assertEqual(decide_envelope(blob).verdict, ENVELOPE_CONTRADICTS)

    def test_deep_reproduced_is_ignored(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_deep_ignored.json")
        obs = observation_from_lineage(blob)
        self.assertIs(obs.reproduced, False)
        self.assertEqual(decide_envelope(blob).verdict, CHAIN_ACCEPTED)

    def test_prose_claim(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "prose_claim.json")
        self.assertEqual(decide_envelope(blob).verdict, PROSE_NOT_EVIDENCE)

    def test_parent_field_missing(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "capsule_parent_field_missing.json")
        self.assertEqual(decide_envelope(blob).verdict, PARENT_FIELD_MISSING)

    def test_does_not_invent_replay_command(self) -> None:
        blob = load_json(PKG / "fixtures" / "envelopes" / "lineage_two_link_ok.json")
        self.assertNotIn("expectedOutputSha256", blob)
        self.assertIn("brokenAt", blob)
        self.assertIn("parentOk", blob["links"][0])
        self.assertIn("lineageOk", blob["links"][0])


if __name__ == "__main__":
    unittest.main()
