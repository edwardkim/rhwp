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
    decide,
)
from lineage_chain.schema import parse_optional_bool


GOOD = "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3"
OTHER = "0000000000000000000000000000000000000000000000000000000000000000"


class DecideGoldenTests(unittest.TestCase):
    def test_golden_table(self) -> None:
        path = PKG / "fixtures" / "golden_decision_table.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        self.assertIn("expected_verdict_class", header)
        for line in lines[1:]:
            row = dict(zip(header, line.split("\t"), strict=True))
            got = decide(
                row["parent_out"],
                row["child_in"],
                parse_optional_bool(row["parent_ok"]),
                parse_optional_bool(row["lineage_ok"]),
                row["broken_at"],
                source=row["source"],
                kind=row["kind"],
                parent_state=row["parent_state"] or "ok",
                reproduced=parse_optional_bool(row["reproduced"]),
            )
            self.assertEqual(got.verdict, row["expected_verdict_class"], msg=row["case_id"])

    def test_matching_hashes_accept(self) -> None:
        decision = decide(GOOD, GOOD, True, True, "")
        self.assertEqual(decision.verdict, CHAIN_ACCEPTED)
        self.assertTrue(decision.chain_accepted)
        self.assertEqual(decision.evidence_kind, "lineage_hash_eq")
        self.assertEqual(decision.exit_class, "0")

    def test_mismatching_hashes_break(self) -> None:
        decision = decide(GOOD, OTHER, True, False, "child.capsule.json")
        self.assertEqual(decision.verdict, LINEAGE_BROKEN)
        self.assertFalse(decision.chain_accepted)
        self.assertEqual(decision.exit_class, "3")

    def test_parent_tamper_beats_matching_hashes(self) -> None:
        decision = decide(GOOD, GOOD, False, True, "parent.capsule.json")
        self.assertEqual(decision.verdict, PARENT_TAMPERED)

    def test_root_does_not_claim_chain(self) -> None:
        decision = decide(GOOD, OTHER, None, None, "")
        self.assertEqual(decision.verdict, ROOT_ONLY)
        self.assertFalse(decision.chain_accepted)

    def test_prose_never_accepts_chain(self) -> None:
        decision = decide(GOOD, GOOD, True, True, "", source="prose")
        self.assertEqual(decision.verdict, PROSE_NOT_EVIDENCE)
        self.assertFalse(decision.chain_accepted)

    def test_head_missing(self) -> None:
        self.assertEqual(decide(GOOD, GOOD, None, None, "", source="io").verdict, HEAD_MISSING)

    def test_usage(self) -> None:
        self.assertEqual(decide("", "", None, None, "", source="usage").verdict, USAGE)

    def test_parent_sha_missing(self) -> None:
        self.assertEqual(
            decide(GOOD, GOOD, True, True, "x", parent_state="sha_missing").verdict,
            PARENT_SHA_MISSING,
        )

    def test_parent_field_missing(self) -> None:
        self.assertEqual(
            decide(GOOD, GOOD, True, True, "x", parent_state="field_missing").verdict,
            PARENT_FIELD_MISSING,
        )

    def test_kind_not_capsule(self) -> None:
        self.assertEqual(
            decide(GOOD, GOOD, True, True, "x", source="capsule", kind="note").verdict,
            KIND_NOT_CAPSULE,
        )

    def test_hash_defect(self) -> None:
        self.assertEqual(decide("deadbeef", GOOD, True, True, "").verdict, HASH_DEFECT)

    def test_envelope_contradicts_when_ok_true_but_hashes_differ(self) -> None:
        self.assertEqual(
            decide(GOOD, OTHER, True, True, "liar.capsule.json").verdict,
            ENVELOPE_CONTRADICTS,
        )

    def test_reproduced_false_does_not_reject_matching_chain(self) -> None:
        decision = decide(GOOD, GOOD, True, True, "", reproduced=False)
        self.assertEqual(decision.verdict, CHAIN_ACCEPTED)


if __name__ == "__main__":
    unittest.main()
