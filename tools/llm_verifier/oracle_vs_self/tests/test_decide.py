from __future__ import annotations

import unittest

from support import PKG
from oracle_vs_self.decide import (
    NO_ORACLE_SELF_CHEAP_FAIL,
    NO_ORACLE_SELF_CONSISTENT,
    NO_ORACLE_SELF_RENDER_FAIL,
    NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF,
    ORACLE_BLOCKED_BY_SELF,
    ORACLE_CHEAP_FAIL,
    ORACLE_MULTIVER_DISAGREE,
    ORACLE_PAGECOUNT_MISMATCH,
    ORACLE_TRUSTED,
    ORACLE_UNVERSIONED,
    ORACLE_YEAR_OUT_OF_CONTRACT,
    INDEPENDENT_ORACLE_TOOLS,
    decide,
    decide_row,
)
from oracle_vs_self.schema import parse_bool


class DecideGoldenTests(unittest.TestCase):
    def test_golden_table(self) -> None:
        path = PKG / "fixtures" / "golden_decision_table.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        self.assertIn("expected_verdict_class", header)
        for line in lines[1:]:
            row = dict(zip(header, line.split("\t"), strict=True))
            got = decide(
                parse_bool(row["has_hangul_pdf"]),
                row["versions"],
                parse_bool(row["page_count_match"]),
                parse_bool(row["render_self_pass"]),
                parse_bool(row["cheap_ok"]),
            )
            self.assertEqual(
                got.verdict_class,
                row["expected_verdict_class"],
                msg=row["case_id"],
            )

    def test_self_only_never_opens_fidelity_compare(self) -> None:
        decision = decide(False, "none", True, True, True)
        self.assertEqual(decision.verdict_class, NO_ORACLE_SELF_CONSISTENT)
        self.assertTrue(decision.self_only)
        self.assertFalse(decision.independent_oracle)
        for tool in INDEPENDENT_ORACLE_TOOLS:
            self.assertIn(tool, decision.blocked_tools)
            self.assertNotIn(tool, decision.allowed_tools)

    def test_trusted_opens_independent_tools(self) -> None:
        decision = decide(True, "2022", True, True, True)
        self.assertEqual(decision.verdict_class, ORACLE_TRUSTED)
        self.assertTrue(decision.independent_oracle)
        for tool in INDEPENDENT_ORACLE_TOOLS:
            self.assertIn(tool, decision.allowed_tools)

    def test_pagecount_mismatch_is_independent_cheap_claim(self) -> None:
        decision = decide(True, "2020", False, True, True)
        self.assertEqual(decision.verdict_class, ORACLE_PAGECOUNT_MISMATCH)
        self.assertFalse(decision.independent_oracle)
        self.assertIn("tools/oracle_public/page_smoke.py", decision.allowed_tools)
        for tool in INDEPENDENT_ORACLE_TOOLS:
            self.assertIn(tool, decision.blocked_tools)

    def test_multiver_does_not_pin_year(self) -> None:
        decision = decide(True, "2010!2024", False, True, True)
        self.assertEqual(decision.verdict_class, ORACLE_MULTIVER_DISAGREE)
        self.assertIn("pin-year-without-evidence", decision.blocked_tools)

    def test_year_without_pdf_is_inconsistent_input(self) -> None:
        decision = decide(False, "2022", True, True, True)
        self.assertEqual(decision.verdict_class, NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF)

    def test_invalid_year_is_not_official(self) -> None:
        decision = decide(True, "2025", True, True, True)
        self.assertEqual(decision.verdict_class, ORACLE_YEAR_OUT_OF_CONTRACT)

    def test_blocked_by_self_keeps_oracle_closed(self) -> None:
        decision = decide(True, "2018+2022", True, False, True)
        self.assertEqual(decision.verdict_class, ORACLE_BLOCKED_BY_SELF)
        for tool in INDEPENDENT_ORACLE_TOOLS:
            self.assertIn(tool, decision.blocked_tools)

    def test_cheap_fail_and_render_fail_order(self) -> None:
        self.assertEqual(
            decide(False, "none", False, False, False).verdict_class,
            NO_ORACLE_SELF_CHEAP_FAIL,
        )
        self.assertEqual(
            decide(False, "none", True, False, True).verdict_class,
            NO_ORACLE_SELF_RENDER_FAIL,
        )
        self.assertEqual(
            decide(True, "2022", True, False, False).verdict_class,
            ORACLE_CHEAP_FAIL,
        )

    def test_unversioned_pdf_is_not_hangul_official(self) -> None:
        self.assertEqual(decide(True, "unknown", True, True, True).verdict_class, ORACLE_UNVERSIONED)
        self.assertEqual(decide(True, "none", True, True, True).verdict_class, ORACLE_UNVERSIONED)

    def test_decide_row_accepts_tsv_cells(self) -> None:
        decision = decide_row(
            {
                "has_hangul_pdf": "1",
                "versions": "2024",
                "page_count_match": "1",
                "render_self_pass": "1",
                "cheap_ok": "1",
            }
        )
        self.assertEqual(decision.verdict_class, ORACLE_TRUSTED)

    def test_honest_claim_is_nonempty_for_every_class(self) -> None:
        samples = [
            (False, "none", True, True, True),
            (False, "none", True, False, True),
            (False, "none", False, True, False),
            (False, "2022", True, True, True),
            (True, "unknown", True, True, True),
            (True, "2025", True, True, True),
            (True, "2018!2024", False, True, True),
            (True, "2020", False, True, True),
            (True, "2022", True, True, False),
            (True, "2022", True, False, True),
            (True, "2022", True, True, True),
        ]
        seen = set()
        for args in samples:
            decision = decide(*args)
            self.assertTrue(decision.honest_claim)
            self.assertGreater(len(decision.tree_path), 0)
            seen.add(decision.verdict_class)
        self.assertEqual(len(seen), 11)


if __name__ == "__main__":
    unittest.main()
