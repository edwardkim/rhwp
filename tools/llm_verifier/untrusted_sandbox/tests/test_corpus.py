from __future__ import annotations

import unittest

import support  # noqa: F401 — puts tools/llm_verifier on sys.path
from untrusted_sandbox.corpus_io import iter_axis_table, iter_corpus, load_manifest
from untrusted_sandbox.decide import decide
from untrusted_sandbox.schema import CASE_COLUMNS, parse_bool
from untrusted_sandbox.slot import SLOT_VALUES


class CorpusTests(unittest.TestCase):
    def test_manifest_meets_size_gate(self) -> None:
        manifest = load_manifest()
        self.assertGreaterEqual(manifest["rowCount"], 100_000)
        self.assertEqual(manifest["columns"], list(CASE_COLUMNS))
        self.assertEqual(manifest["claim"], "V-nonce")

    def test_every_row_matches_decide(self) -> None:
        seen: set[tuple] = set()
        count = 0
        for case in iter_corpus():
            count += 1
            key = case.contract_tuple()
            self.assertNotIn(key, seen)
            seen.add(key)
            got = decide(
                case.slot,
                case.leaked_into_criteria,
                case.nonce,
                case.excerpt,
                case.source_label_kind,
                case.wrap_state,
                case.untrusted_content,
            )
            self.assertEqual(got.expected_block, case.expected_block, case.case_id)
            if case.leaked_into_criteria:
                self.assertTrue(case.expected_block, case.case_id)
        self.assertGreaterEqual(count, 100_000)

    def test_axis_closed_set_covers_slots(self) -> None:
        slots = {row["slot"] for row in iter_axis_table()}
        self.assertEqual(slots, set(SLOT_VALUES))
        for row in iter_axis_table():
            nonce, excerpt = _axis_pair(row["nonce_kind"])
            got = decide(
                row["slot"],
                parse_bool(row["leaked_into_criteria"]),
                nonce,
                excerpt,
                row["source_label_kind"],
                row["wrap_state"],
                True,
            )
            self.assertEqual(
                got.expected_block,
                parse_bool(row["expected_block"]),
                msg=str(row),
            )


def _axis_pair(kind: str) -> tuple[str, str]:
    if kind == "collision":
        return "TOKEN16axis", "axis-collision-TOKEN16axis"
    if kind == "static":
        return "DOCUMENT", "axis-static"
    if kind == "empty":
        return "", "axis-empty"
    if kind == "reused":
        return "reuse00deadbeef", "axis-reused"
    return "0123456789abcdef", "axis-fresh"
