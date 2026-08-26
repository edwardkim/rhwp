from __future__ import annotations

import unittest

from support import PKG
from oracle_vs_self.decide import (
    NO_ORACLE_SELF_CONSISTENT,
    ORACLE_CHEAP_FAIL,
    ORACLE_MULTIVER_DISAGREE,
    ORACLE_PAGECOUNT_MISMATCH,
    ORACLE_TRUSTED,
)
from oracle_vs_self.envelopes import load_envelope


class EnvelopeBindTests(unittest.TestCase):
    def test_page_smoke_mixed(self) -> None:
        rows = load_envelope(PKG / "fixtures" / "envelopes" / "page_smoke_mixed.json")
        classes = [decision.verdict_class for _signals, decision in rows]
        self.assertEqual(
            classes,
            [ORACLE_TRUSTED, ORACLE_PAGECOUNT_MISMATCH, ORACLE_CHEAP_FAIL],
        )

    def test_resolver_unmatched_is_self_only(self) -> None:
        rows = load_envelope(PKG / "fixtures" / "envelopes" / "resolver_mini.json")
        classes = [decision.verdict_class for _signals, decision in rows]
        self.assertEqual(classes[:2], [ORACLE_TRUSTED, ORACLE_TRUSTED])
        self.assertEqual(classes[2], NO_ORACLE_SELF_CONSISTENT)

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

    def test_visual_sweep_complete_run(self) -> None:
        rows = load_envelope(PKG / "fixtures" / "envelopes" / "visual_sweep_run.json")
        self.assertEqual(rows[0][1].verdict_class, ORACLE_TRUSTED)
        self.assertTrue(rows[0][0].has_hangul_pdf)


if __name__ == "__main__":
    unittest.main()
