from __future__ import annotations

import unittest

from support import PKG
from shadow_agree.decide import VERDICT_CLASSES, decide
from shadow_agree.generate_corpus import axis_space
from shadow_agree.schema import parse_bool


class PairClosedSetTests(unittest.TestCase):
    def test_axis_space_covers_distinct_and_same(self) -> None:
        space = axis_space()
        self.assertGreaterEqual(len(space), 16 * 15 * 4)
        classes = {row[5] for row in space}
        self.assertEqual(classes, set(VERDICT_CLASSES))

    def test_every_axis_row_matches_decide(self) -> None:
        path = PKG / "fixtures" / "pair_closed_set.tsv"
        self.assertTrue(path.is_file(), "run generate_corpus.py first")
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        seen_keys = set()
        seen_classes = set()
        for line in lines[1:]:
            row = dict(zip(header, line.split("\t"), strict=True))
            key = (row["check_a"], row["check_b"], row["a_pass"], row["b_pass"])
            self.assertNotIn(key, seen_keys)
            seen_keys.add(key)
            got = decide(
                row["check_a"],
                row["check_b"],
                parse_bool(row["a_pass"]),
                parse_bool(row["b_pass"]),
            )
            self.assertEqual(got.verdict_class, row["expected_verdict_class"], msg=key)
            self.assertEqual(got.expected_joint, parse_bool(row["expected_joint"]), msg=key)
            seen_classes.add(got.verdict_class)
        self.assertEqual(seen_classes, set(VERDICT_CLASSES))

    def test_joint_true_only_when_both_pass_and_distinct(self) -> None:
        for check_a, check_b, a_pass, b_pass, joint, verdict in axis_space():
            if joint:
                self.assertTrue(a_pass and b_pass)
                self.assertNotEqual(check_a, check_b)
                self.assertEqual(verdict, "JOINT_PASS")
            else:
                self.assertNotEqual(verdict, "JOINT_PASS")


if __name__ == "__main__":
    unittest.main()
