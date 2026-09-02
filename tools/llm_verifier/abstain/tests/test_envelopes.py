from __future__ import annotations

import unittest

from support import PKG
from abstain.envelopes import decide_envelope, load_fixture
from abstain.schema import VERDICT_ABSTAIN, VERDICT_FAIL, VERDICT_PASS


class EnvelopeBindTests(unittest.TestCase):
    def test_identical_has_signal_fixture(self) -> None:
        raw = load_fixture("identical_and_has_signal.json")
        got = decide_envelope(raw, command="layout-anomaly", exit_code=0)
        self.assertEqual(got.verdict, VERDICT_ABSTAIN)
        self.assertEqual(got.contradiction_id, "identical_and_has_signal")

    def test_reproduced_exit3_fixture(self) -> None:
        raw = load_fixture("reproduced_and_exit3.json")
        got = decide_envelope(raw, command="replay", exit_code=3)
        self.assertEqual(got.verdict, VERDICT_ABSTAIN)
        self.assertEqual(got.contradiction_id, "reproduced_and_exit3")

    def test_same_node_fixture(self) -> None:
        raw = load_fixture("pagecount_struct_same_node.json")
        got = decide_envelope(raw, command="render-diff", exit_code=1)
        self.assertEqual(got.verdict, VERDICT_ABSTAIN)
        self.assertEqual(got.contradiction_id, "pagecount_match_and_struct_same_node")

    def test_other_node_fixture_is_fail(self) -> None:
        raw = load_fixture("pagecount_struct_other_node.json")
        got = decide_envelope(raw, command="render-diff", exit_code=1)
        self.assertEqual(got.verdict, VERDICT_FAIL)

    def test_consistent_pass_fixture(self) -> None:
        raw = load_fixture("consistent_pass.json")
        got = decide_envelope(raw, command="ir-diff", exit_code=0)
        self.assertEqual(got.verdict, VERDICT_PASS)

    def test_consistent_fail_fixture(self) -> None:
        raw = load_fixture("consistent_fail.json")
        got = decide_envelope(raw, command="ir-diff", exit_code=3)
        self.assertEqual(got.verdict, VERDICT_FAIL)

    def test_verify_block_fixture(self) -> None:
        raw = load_fixture("verify_identical_diffcount.json")
        got = decide_envelope(raw, command="fill-fields", exit_code=3)
        self.assertEqual(got.verdict, VERDICT_ABSTAIN)
        self.assertEqual(got.contradiction_id, "verify_identical_and_diffcount")

    def test_fixtures_exist(self) -> None:
        names = [
            "identical_and_has_signal.json",
            "reproduced_and_exit3.json",
            "pagecount_struct_same_node.json",
            "pagecount_struct_other_node.json",
            "consistent_pass.json",
            "consistent_fail.json",
            "verify_identical_diffcount.json",
        ]
        for name in names:
            self.assertTrue((PKG / "fixtures" / "envelopes" / name).is_file(), name)


if __name__ == "__main__":
    unittest.main()
