"""Corpus gates: distinct sets, expectedRank matches ranker, no process_steps."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from generate_corpus import identity_key
from rank import expected_ranks_match
from schema import FORBIDDEN_KEYS, RANK_FIELDS

CORPUS = HERE / "corpus"
MIN_LINES = 100000


class CorpusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        manifest_path = CORPUS / "manifest.json"
        if not manifest_path.is_file():
            raise unittest.SkipTest("corpus not generated")
        cls.manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        cls.sets: list[dict] = []
        for shard in cls.manifest["shards"]:
            path = HERE / shard["path"]
            payload = json.loads(path.read_text(encoding="utf-8"))
            cls.sets.extend(payload)

    def test_line_count_meets_hard_gate(self) -> None:
        counted = 0
        for shard in self.manifest["shards"]:
            text = (HERE / shard["path"]).read_text(encoding="utf-8")
            counted += text.count("\n")
        self.assertGreaterEqual(counted, MIN_LINES)
        self.assertGreaterEqual(self.manifest["lineCount"], MIN_LINES)
        self.assertEqual(self.manifest["lineCount"], counted)

    def test_set_count_matches_manifest(self) -> None:
        self.assertEqual(self.manifest["setCount"], len(self.sets))
        self.assertGreaterEqual(len(self.sets), 1600)

    def test_set_ids_and_identity_keys_are_unique(self) -> None:
        ids = [blob["setId"] for blob in self.sets]
        self.assertEqual(len(ids), len(set(ids)))
        keys = [identity_key(blob) for blob in self.sets]
        self.assertEqual(len(keys), len(set(keys)))

    def test_every_set_has_machine_fields_and_expected_rank(self) -> None:
        for blob in self.sets:
            self.assertEqual(blob["rankFields"], list(RANK_FIELDS))
            self.assertGreaterEqual(blob["n"], 2)
            self.assertEqual(blob["n"], len(blob["candidates"]))
            self.assertNotIn("process_steps", blob)
            self.assertNotIn("processSteps", blob)
            for rec in blob["candidates"]:
                self.assertIn("changedCount", rec)
                self.assertIn("invalid", rec)
                self.assertIn("exitClass", rec)
                self.assertIn("expectedRank", rec)
                self.assertIn(rec["exitClass"], (0, 1, 2, 3, 4))
                for forbidden in FORBIDDEN_KEYS:
                    self.assertNotIn(forbidden, rec)
                    if isinstance(rec.get("envelope"), dict):
                        self.assertNotIn(forbidden, rec["envelope"])

    def test_expected_rank_matches_ranker(self) -> None:
        for blob in self.sets:
            mismatches = expected_ranks_match(blob)
            self.assertEqual(mismatches, [], msg=blob["setId"])

    def test_commands_and_modes_are_populated(self) -> None:
        commands = {blob["command"] for blob in self.sets}
        modes = {blob["mode"] for blob in self.sets}
        self.assertIn("fill-fields", commands)
        self.assertIn("csv-to-table", commands)
        self.assertIn("ir-diff", commands)
        self.assertIn("dry-run", modes)
        self.assertIn("verify", modes)
        self.assertIn("ir-diff", modes)

    def test_no_comment_padding_markers(self) -> None:
        for blob in self.sets:
            dumped = json.dumps(blob, ensure_ascii=False)
            self.assertNotIn("TODO", dumped)
            self.assertNotIn("lorem ipsum", dumped.lower())
            self.assertNotIn("padding", dumped.lower())


if __name__ == "__main__":
    unittest.main()
