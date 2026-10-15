from __future__ import annotations

import unittest

from support import PKG
from oracle_vs_self.contracts import bind_visual_sweep_manifest
from oracle_vs_self.decide import (
    NO_ORACLE_SELF_CONSISTENT,
    ORACLE_CHEAP_FAIL,
    ORACLE_MULTIVER_DISAGREE,
    ORACLE_PAGECOUNT_MISMATCH,
    ORACLE_TRUSTED,
)
from oracle_vs_self.envelopes import load_envelope


class EnvelopeBindTests(unittest.TestCase):

    def test_visual_sweep_current_and_legacy_complete_manifests(self) -> None:
        for version in (1, 2):
            with self.subTest(version=version):
                signals = bind_visual_sweep_manifest({
                    "schema_version": version,
                    "provenance": {"pdf": {"path": "reference.pdf"}},
                    "run_state": "complete",
                    "font_mismatch_evidence": None,
                })
                self.assertTrue(signals.has_hangul_pdf)
                self.assertTrue(signals.cheap_ok)

    def test_visual_sweep_noncomplete_or_unsupported_manifest_is_rejected(self) -> None:
        for version, state in (
            (1, "incomplete"), (1, "failed"), (1, ""),
            (2, "incomplete"), (2, "failed"), (2, ""), (3, "complete"),
        ):
            with self.subTest(version=version, state=state):
                signals = bind_visual_sweep_manifest({
                    "schema_version": version,
                    "provenance": {"pdf": {"path": "reference.pdf"}},
                    "run_state": state,
                })
                self.assertFalse(signals.cheap_ok)

    def test_visual_sweep_complete_manifest_still_requires_reference_pdf(self) -> None:
        signals = bind_visual_sweep_manifest({
            "schema_version": 2,
            "provenance": {"hwp": {"path": "original.hwpx"}},
            "run_state": "complete",
        })
        self.assertFalse(signals.has_hangul_pdf)
        self.assertFalse(signals.cheap_ok)


    def test_multiver_disagree(self) -> None:
        rows = load_envelope(PKG / "fixtures" / "envelopes" / "multiver_disagree.json")
        self.assertEqual(len(rows), 1)
        signals, decision = rows[0]
        self.assertEqual(signals.versions, "2010!2020!2024")
        self.assertEqual(decision.verdict_class, ORACLE_MULTIVER_DISAGREE)

    def test_fidelity_ledger_match_and_mismatch(self) -> None:
        match = load_envelope(PKG / "fixtures" / "envelopes" / "page_count_ledger_match.tsv")
        self.assertEqual(match[0][1].verdict_class, ORACLE_TRUSTED)
        mismatch = load_envelope(
            PKG / "fixtures" / "envelopes" / "page_count_ledger_mismatch.tsv"
        )
        self.assertEqual(mismatch[0][1].verdict_class, ORACLE_PAGECOUNT_MISMATCH)



if __name__ == "__main__":
    unittest.main()
