from __future__ import annotations

import unittest

from support import PKG
from oracle_vs_self.decide import VERDICT_CLASSES, decide
from oracle_vs_self.generate_corpus import all_flag_tuples, all_version_tokens, axis_space
from oracle_vs_self.schema import parse_bool


class AxisClosedSetTests(unittest.TestCase):
    def test_axis_space_covers_every_token_and_flag(self) -> None:
        space = axis_space()
        expected = len(all_version_tokens()) * len(all_flag_tuples())
        self.assertEqual(len(space), expected)
        self.assertGreaterEqual(expected, 16 * 16)

    def test_every_axis_row_matches_decide(self) -> None:
        path = PKG / "fixtures" / "axis_closed_set.tsv"
        self.assertTrue(path.is_file(), "run generate_corpus.py first")
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        seen_classes = set()
        seen_keys = set()
        for line in lines[1:]:
            row = dict(zip(header, line.split("\t"), strict=True))
            key = (
                row["has_hangul_pdf"],
                row["versions"],
                row["page_count_match"],
                row["render_self_pass"],
                row["cheap_ok"],
            )
            self.assertNotIn(key, seen_keys)
            seen_keys.add(key)
            got = decide(
                parse_bool(row["has_hangul_pdf"]),
                row["versions"],
                parse_bool(row["page_count_match"]),
                parse_bool(row["render_self_pass"]),
                parse_bool(row["cheap_ok"]),
            )
            self.assertEqual(got.verdict_class, row["expected_verdict_class"], msg=key)
            seen_classes.add(got.verdict_class)
        self.assertEqual(seen_classes, set(VERDICT_CLASSES))

    def test_no_oracle_path_ignores_page_match_when_choosing_class_prefix(self) -> None:
        for page in (False, True):
            for render in (False, True):
                for cheap in (False, True):
                    decision = decide(False, "none", page, render, cheap)
                    self.assertTrue(decision.verdict_class.startswith("NO_ORACLE_"))


if __name__ == "__main__":
    unittest.main()
