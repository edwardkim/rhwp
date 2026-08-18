from __future__ import annotations

import json
import unittest

from support import PKG
from shadow_agree.corpus_io import iter_corpus, load_manifest
from shadow_agree.decide import VERDICT_CLASSES, decide
from shadow_agree.verify_corpus import MIN_ROWS, verify


class CorpusTests(unittest.TestCase):
    def test_manifest_and_verify(self) -> None:
        manifest = load_manifest()
        self.assertGreaterEqual(manifest["rowCount"], MIN_ROWS)
        self.assertEqual(manifest["claim"], "V-shadow")
        self.assertTrue(manifest["shards"])
        report = verify()
        self.assertTrue(report["ok"], report.get("errors"))
        self.assertEqual(report["rows"], manifest["rowCount"])
        self.assertEqual(set(report["byVerdict"]), set(VERDICT_CLASSES))
        self.assertGreater(report["jointPass"], 0)
        self.assertLess(report["jointPass"], report["rows"])

    def test_first_and_last_shard_rows_are_distinct_cases(self) -> None:
        manifest = load_manifest()
        first = PKG / manifest["shards"][0]["path"]
        last = PKG / manifest["shards"][-1]["path"]
        self.assertTrue(first.is_file())
        self.assertTrue(last.is_file())
        self.assertNotEqual(first.read_bytes(), last.read_bytes())

    def test_sample_of_rows_round_trip_decide(self) -> None:
        checked = 0
        for index, case in enumerate(iter_corpus()):
            if index % 1024 != 0:
                continue
            got = decide(case.check_a, case.check_b, case.a_pass, case.b_pass)
            self.assertEqual(got.verdict_class, case.expected_verdict_class)
            self.assertEqual(got.expected_joint, case.expected_joint)
            checked += 1
        self.assertGreaterEqual(checked, 100)

    def test_no_bom_and_unix_newlines(self) -> None:
        manifest_path = PKG / "corpus" / "manifest.json"
        raw = manifest_path.read_bytes()
        self.assertFalse(raw.startswith(b"\xef\xbb\xbf"))
        json.loads(raw.decode("utf-8"))
        shard = next((PKG / "corpus").glob("shard_0000.tsv"))
        data = shard.read_bytes()
        self.assertNotIn(b"\r\n", data)
        self.assertFalse(data.startswith(b"\xef\xbb\xbf"))

    def test_rows_are_not_comment_padding(self) -> None:
        for index, case in enumerate(iter_corpus()):
            if index > 200:
                break
            self.assertFalse(case.case_id.startswith("#"))
            self.assertIn(case.check_a, {case.check_a})
            self.assertTrue(case.command_a.startswith("rhwp "))
            self.assertTrue(case.command_b.startswith("rhwp "))
            self.assertTrue(case.not_abstain)
            self.assertTrue(case.not_repeat)


if __name__ == "__main__":
    unittest.main()
