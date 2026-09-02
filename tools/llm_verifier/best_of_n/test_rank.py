"""Unit tests for V-bon outcome ranking. No prose scores, no process_steps."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from envelopes import lift_envelope
from rank import CLAIM_ID, outcome_key, rank_candidates, rank_mapping
from schema import CandidateOutcome, FORBIDDEN_KEYS


def cand(
    cid: str,
    *,
    changed: int,
    invalid,
    identical,
    exit_class: int,
) -> CandidateOutcome:
    return CandidateOutcome(
        candidate_id=cid,
        changed_count=changed,
        invalid=invalid,
        verify_identical=identical,
        exit_class=exit_class,
    )


class RankOrderTests(unittest.TestCase):
    def test_exact_verify_beats_over_edit(self) -> None:
        ranked = rank_candidates(
            [
                cand("over", changed=5, invalid=[], identical=True, exit_class=0),
                cand("exact", changed=3, invalid=[], identical=True, exit_class=0),
            ],
            intended_changed_count=3,
            set_id="t-over",
            command="fill-fields",
            mode="verify",
        )
        self.assertEqual(ranked.winner_id, "exact")
        by_id = {row.candidate.candidate_id: row.expected_rank for row in ranked.ranking}
        self.assertEqual(by_id["exact"], 1)
        self.assertEqual(by_id["over"], 2)

    def test_identical_true_beats_false_at_same_exit(self) -> None:
        ranked = rank_candidates(
            [
                cand("bad", changed=4, invalid=[], identical=False, exit_class=0),
                cand("good", changed=4, invalid=[], identical=True, exit_class=0),
            ],
            intended_changed_count=4,
        )
        self.assertEqual(ranked.winner_id, "good")

    def test_exit0_beats_exit3_even_if_identical_true(self) -> None:
        ranked = rank_candidates(
            [
                cand("judging", changed=2, invalid=[], identical=True, exit_class=3),
                cand("ok", changed=2, invalid=[], identical=True, exit_class=0),
            ],
            intended_changed_count=2,
        )
        self.assertEqual(ranked.winner_id, "ok")

    def test_invalid_always_worse_than_clean(self) -> None:
        ranked = rank_candidates(
            [
                cand("inv", changed=0, invalid=[{"reason": "rowCountMismatch"}], identical=None, exit_class=2),
                cand("io", changed=0, invalid=[], identical=None, exit_class=1),
                cand("ok", changed=2, invalid=[], identical=True, exit_class=0),
            ],
            intended_changed_count=2,
        )
        order = [row.candidate.candidate_id for row in ranked.ranking]
        self.assertEqual(order[0], "ok")
        self.assertEqual(order[-1], "inv")

    def test_exit_order_is_ok_judgment_page_io_usage(self) -> None:
        ranked = rank_candidates(
            [
                cand("usage", changed=0, invalid=[], identical=None, exit_class=2),
                cand("io", changed=0, invalid=[], identical=None, exit_class=1),
                cand("page", changed=0, invalid=[], identical=False, exit_class=4),
                cand("judge", changed=3, invalid=[], identical=False, exit_class=3),
                cand("ok", changed=3, invalid=[], identical=True, exit_class=0),
            ],
            intended_changed_count=3,
        )
        order = [row.candidate.candidate_id for row in ranked.ranking]
        self.assertEqual(order, ["ok", "judge", "page", "io", "usage"])

    def test_dry_run_null_identical_ranks_by_changed_delta(self) -> None:
        ranked = rank_candidates(
            [
                cand("over", changed=6, invalid=[], identical=None, exit_class=0),
                cand("exact", changed=4, invalid=[], identical=None, exit_class=0),
                cand("under", changed=3, invalid=[], identical=None, exit_class=0),
            ],
            intended_changed_count=4,
        )
        order = [row.candidate.candidate_id for row in ranked.ranking]
        self.assertEqual(order[0], "exact")
        self.assertEqual(set(order[1:]), {"over", "under"})

    def test_competition_rank_ties_share_rank(self) -> None:
        ranked = rank_candidates(
            [
                cand("a", changed=2, invalid=[], identical=True, exit_class=0),
                cand("b", changed=2, invalid=[], identical=True, exit_class=0),
                cand("c", changed=4, invalid=[], identical=True, exit_class=0),
            ],
            intended_changed_count=2,
        )
        by_id = {row.candidate.candidate_id: row.expected_rank for row in ranked.ranking}
        self.assertEqual(by_id["a"], 1)
        self.assertEqual(by_id["b"], 1)
        self.assertEqual(by_id["c"], 3)

    def test_key_uses_only_four_fields_plus_id(self) -> None:
        key = outcome_key(
            cand("x", changed=7, invalid=[], identical=True, exit_class=0),
            5,
        )
        self.assertEqual(key.invalid_rank, 0)
        self.assertEqual(key.exit_rank, 0)
        self.assertEqual(key.identical_rank, 0)
        self.assertEqual(key.change_delta, 2)
        self.assertEqual(key.changed_count, 7)

    def test_mapping_roundtrip_and_claim(self) -> None:
        blob = {
            "setId": "map-1",
            "command": "csv-to-table",
            "mode": "verify",
            "intendedChangedCount": 2,
            "candidates": [
                {
                    "candidateId": "c0",
                    "changedCount": 2,
                    "invalid": [],
                    "verify": {"identical": True, "diffCount": 0},
                    "exitClass": 0,
                },
                {
                    "candidateId": "c1",
                    "changedCount": 2,
                    "invalid": [],
                    "verify": {"identical": False, "diffCount": 4},
                    "exitClass": 3,
                },
            ],
        }
        ranked = rank_mapping(blob)
        self.assertEqual(ranked.claim if hasattr(ranked, "claim") else CLAIM_ID, CLAIM_ID)
        self.assertEqual(ranked.winner_id, "c0")
        self.assertEqual(ranked.to_json()["rankFields"], [
            "changedCount",
            "invalid",
            "verify.identical",
            "exitClass",
        ])

    def test_refuses_process_steps(self) -> None:
        blob = {
            "setId": "bad",
            "intendedChangedCount": 1,
            "process_steps": [{"score": 0.9}],
            "candidates": [
                {
                    "candidateId": "c0",
                    "changedCount": 1,
                    "invalid": [],
                    "verify": {"identical": True},
                    "exitClass": 0,
                }
            ],
        }
        with self.assertRaises(ValueError) as ctx:
            rank_mapping(blob)
        self.assertIn("5490", str(ctx.exception))

    def test_forbidden_keys_are_closed(self) -> None:
        self.assertIn("process_steps", FORBIDDEN_KEYS)
        self.assertIn("proseScore", FORBIDDEN_KEYS)


class EnvelopeLiftTests(unittest.TestCase):
    def test_ir_diff_lifts_top_level_identical(self) -> None:
        env = {
            "schemaVersion": "1.0",
            "identical": False,
            "diffCount": 3,
            "categories": {"cc": 3},
            "exitClass": 3,
        }
        out = lift_envelope(env, candidate_id="ir0")
        self.assertEqual(out.changed_count, 3)
        self.assertEqual(out.verify_identical, False)
        self.assertEqual(out.exit_class, 3)

    def test_fill_fields_uses_filled_count_fallback(self) -> None:
        env = {
            "filledCount": 4,
            "invalid": [],
            "verify": {"identical": True, "diffCount": 0},
            "exitClass": "ok",
        }
        out = lift_envelope(env, candidate_id="f0")
        self.assertEqual(out.changed_count, 4)
        self.assertEqual(out.exit_class, 0)
        self.assertTrue(out.verify_identical)

    def test_invalid_list_and_bool(self) -> None:
        a = lift_envelope(
            {"changedCount": 0, "invalid": [{"reason": "notFound"}], "exitClass": 2},
            candidate_id="a",
        )
        b = lift_envelope(
            {"changedCount": 0, "invalid": True, "exitClass": 2},
            candidate_id="b",
        )
        self.assertTrue(a.is_invalid())
        self.assertTrue(b.is_invalid())

    def test_exit_name_runtime_maps_to_io(self) -> None:
        out = lift_envelope(
            {"changedCount": 0, "invalid": [], "exitClass": "runtime"},
            candidate_id="io",
        )
        self.assertEqual(out.exit_class, 1)

    def test_refuses_prose_score_on_envelope(self) -> None:
        with self.assertRaises(ValueError):
            lift_envelope(
                {"changedCount": 1, "invalid": [], "exitClass": 0, "llmScore": 0.8},
                candidate_id="x",
            )


if __name__ == "__main__":
    unittest.main()
