from __future__ import annotations

import unittest

from support import PKG
from third_party_replay.corpus_io import load_corpus, load_manifest
from third_party_replay.decide import VERDICT_CLASSES, decide_observation
from third_party_replay.generate_corpus import FAMILIES, make_case


class CorpusContractTests(unittest.TestCase):
    def test_manifest_meets_size_floor(self) -> None:
        man = load_manifest(PKG / "corpus")
        self.assertEqual(man["claim"], "V-replay")
        self.assertEqual(man["axis"], "third-party-replay")
        self.assertGreaterEqual(man["rowCount"], 100_000)
        self.assertGreaterEqual(len(man["shards"]), 8)
        self.assertEqual(set(man["verdicts"]), set(VERDICT_CLASSES))
        self.assertEqual(
            man["uniqueness"],
            "plan,expect_sha,reproduced,toolVersion,verdict",
        )

    def test_every_row_matches_decide_and_is_distinct(self) -> None:
        cases = load_corpus(PKG / "corpus")
        self.assertGreaterEqual(len(cases), 100_000)
        keys = set()
        hangul = 0
        by_verdict = {name: 0 for name in VERDICT_CLASSES}
        for case in cases:
            key = case.identity_key()
            self.assertNotIn(key, keys, case.case_id)
            keys.add(key)
            got = decide_observation(case.observation())
            self.assertEqual(got.verdict, case.verdict, case.case_id)
            self.assertEqual(got.labor_accepted, case.labor_accepted, case.case_id)
            lower = (case.plan + " " + case.implementer_claim).lower()
            for marker in ("lorem", "ipsum", "asdf", "qwerty", "padding", "xxx"):
                self.assertNotIn(marker, lower, case.case_id)
            if any("가" <= ch <= "힣" for ch in case.plan + case.implementer_claim):
                hangul += 1
            by_verdict[case.verdict] += 1
        self.assertEqual(len(keys), len(cases))
        self.assertGreaterEqual(hangul, len(cases) * 8 // 10)
        for name in VERDICT_CLASSES:
            self.assertGreater(by_verdict[name], 0, name)

    def test_generator_family_closed_set(self) -> None:
        self.assertEqual(tuple(FAMILIES), VERDICT_CLASSES)
        for index in range(len(FAMILIES) * 3):
            case = make_case(index)
            self.assertEqual(case.verdict, FAMILIES[index % len(FAMILIES)])
            self.assertEqual(decide_observation(case.observation()).verdict, case.verdict)


if __name__ == "__main__":
    unittest.main()
