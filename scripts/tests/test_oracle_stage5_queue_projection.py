#!/usr/bin/env python3
"""W5-5A reuse matrix and terminal-disposition regression tests."""

from __future__ import annotations

import copy
import json
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
INVESTIGATION = ROOT / "mydocs/tech/investigations/issue-4963"
sys.path.insert(0, str(SCRIPTS))

from oracle_stage2_common import canonical_json_bytes, sha256_bytes, sha256_file  # noqa: E402
from oracle_stage4_profile import reject_absolute_paths  # noqa: E402
from oracle_stage5_queue_projection import (  # noqa: E402
    SOURCE_UNAVAILABLE_RANKS,
    validate_queue_projection,
)


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


class OracleStage5QueueProjectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.projection_path = INVESTIGATION / "oracle_stage5_queue_projection.json"
        cls.projection = read_json(cls.projection_path)
        cls.blocked_path = INVESTIGATION / "oracle_stage4_rank13_blocked_disposition.json"
        cls.blocked = read_json(cls.blocked_path)
        cls.by_rank = {
            entry["queueRank"]: entry for entry in cls.projection["candidates"]
        }

    def test_projection_is_canonical_and_covers_the_frozen_queue(self):
        self.assertEqual(validate_queue_projection(self.projection), [])
        self.assertEqual(self.projection["candidateCount"], 17)
        self.assertEqual(list(self.by_rank), list(range(1, 18)))
        self.assertEqual(
            self.projection["counts"],
            {
                "complete-acceptance-ladder": 2,
                "pending-controlled-ladder": 1,
                "pending-read-only-profile": 1,
                "terminal-protected-partial": 3,
                "terminal-source-unavailable": 10,
            },
        )

    def test_only_rank8_and_rank16_remain_actionable(self):
        self.assertEqual(self.projection["actionableRanks"], [8, 16])
        self.assertEqual(self.projection["recommendedExecutionOrder"], [16, 8])
        self.assertEqual(
            self.by_rank[8]["nextAction"],
            "approve-distinct-substitution-and-run-rank8",
        )
        self.assertEqual(
            self.by_rank[16]["nextAction"],
            "run-rank16-read-only-exact-profile",
        )
        for rank, entry in self.by_rank.items():
            if rank not in {8, 16}:
                self.assertNotIn("run-rank", entry["nextAction"])

    def test_completed_ladders_are_reused_without_remeasurement(self):
        for rank in (1, 7):
            entry = self.by_rank[rank]
            self.assertEqual(entry["disposition"], "complete-acceptance-ladder")
            self.assertEqual(entry["nextAction"], "reuse-tracked-profiles")
            self.assertEqual(len(entry["availableProfiles"]), 4)
            for question in (
                "exact-installed",
                "exact-removed",
                "document-subst-font-only",
                "all-related-fonts-missing",
            ):
                self.assertEqual(entry["questions"][question]["status"], "observed-primary")

    def test_source_unavailable_faces_have_terminal_evidence_dispositions(self):
        actual = {
            rank
            for rank, entry in self.by_rank.items()
            if entry["disposition"] == "terminal-source-unavailable"
        }
        self.assertEqual(actual, SOURCE_UNAVAILABLE_RANKS)
        for rank in actual:
            entry = self.by_rank[rank]
            self.assertEqual(entry["nextAction"], "source-discovery-only")
            self.assertEqual(
                entry["questions"]["exact-installed"]["status"],
                "blocked-source-unavailable",
            )

    def test_protected_fonts_are_not_removed_to_fill_missing_values(self):
        for rank in (9, 10, 13):
            entry = self.by_rank[rank]
            self.assertEqual(entry["disposition"], "terminal-protected-partial")
            self.assertNotIn(rank, self.projection["actionableRanks"])
        rank13_questions = self.by_rank[13]["questions"]
        for question in (
            "exact-removed",
            "document-subst-font-only",
            "all-related-fonts-missing",
        ):
            self.assertEqual(
                rank13_questions[question]["status"],
                "blocked-immutable-or-unmanaged-font",
            )

    def test_rank13_block_is_hash_bound_and_path_free(self):
        self.assertEqual(
            self.blocked["status"], "blocked-immutable-or-unmanaged-font"
        )
        self.assertTrue(self.blocked["fontState"]["exactReadbackSurvived"])
        self.assertEqual(self.blocked["fontState"]["managedRelatedFontCount"], 0)
        self.assertEqual(
            self.projection["inputs"]["rank13BlockedDispositionSha256"],
            sha256_file(self.blocked_path),
        )
        reject_absolute_paths(self.blocked)

    def test_every_reused_profile_hash_matches_the_tracked_file(self):
        profile_records = [
            profile
            for candidate in self.projection["candidates"]
            for profile in candidate["availableProfiles"]
        ]
        self.assertEqual(len(profile_records), 11)
        for profile in profile_records:
            path = ROOT / profile["artifact"]
            self.assertTrue(path.is_file())
            self.assertEqual(sha256_file(path), profile["sha256"])

    def test_canonical_and_actionable_rank_drift_fail_closed(self):
        changed = copy.deepcopy(self.projection)
        changed["actionableRanks"] = [8, 9, 16]
        errors = validate_queue_projection(changed)
        self.assertIn("queue projection actionable rank boundary drifted", errors)
        self.assertIn("queue projection canonical hash mismatch", errors)

        changed = copy.deepcopy(self.projection)
        projection = dict(changed)
        projection.pop("canonicalSha256")
        changed["canonicalSha256"] = sha256_bytes(canonical_json_bytes(projection))
        self.assertEqual(validate_queue_projection(changed), [])

    def test_public_artifacts_do_not_publish_private_inputs(self):
        for path in (self.projection_path, self.blocked_path):
            text = path.read_text(encoding="utf-8")
            self.assertNotIn("/home/", text)
            self.assertNotIn("/mnt/", text)
            self.assertNotRegex(text, r"[A-Za-z]:[\\/]")
            reject_absolute_paths(read_json(path))
        self.assertFalse(self.projection["policy"]["privateCorpusRemeasurementRequired"])
        self.assertFalse(self.projection["policy"]["productBehaviorChanged"])


if __name__ == "__main__":
    unittest.main()
