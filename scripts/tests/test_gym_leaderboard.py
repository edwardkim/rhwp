"""[#4659] 리더보드 해시 체인 계약 — 바이너리 없이 도는 CI 가드.

Ed25519 서명 검증은 rhwp 바이너리가 필요하므로 바이너리 게이트에서 돈다. 이
가드는 바이너리 없이도 검증 가능한 것 — 3해시 고정·원장 줄 체인·앵커 줄 체인·
머클 루트·원장 스냅샷 봉인 — 을 상시 고정한다. 커밋된 리더보드가 있으면 그것을,
없으면 합성 픽스처를 검증한다.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
LB_PATH = REPO_ROOT / "gym" / "tools" / "leaderboard.py"


def load_lb():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    spec = importlib.util.spec_from_file_location("gym_leaderboard_test", LB_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def make_chain(kind, payloads):
    """줄 해시 체인 ndjson 을 규약대로 만든다 — prevEntryHash = 직전 줄 바이트 sha256."""
    lines, prev = [], None
    for i, extra in enumerate(payloads):
        entry = {"seq": i, "kind": kind, "prevEntryHash": prev}
        entry.update(extra)
        line = json.dumps(entry, ensure_ascii=False)
        prev = hashlib.sha256(line.encode("utf-8")).hexdigest()
        lines.append(line)
    return "\n".join(lines) + "\n"


class ChainWalkTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_valid_chain_walks_clean(self, kind="settlementLedger"):
        import tempfile
        text = make_chain(kind, [{"capsuleSha256": "a" * 64, "verdict": "accepted"},
                                 {"capsuleSha256": "b" * 64, "verdict": "accepted"}])
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write(text)
            path = fh.name
        entries, err = self.lb.chain_walk(path, kind)
        Path(path).unlink()
        self.assertIsNone(err, err)
        self.assertEqual(len(entries), 2)

    def test_retroactive_edit_is_exposed_by_next_line(self):
        """과거 줄을 수정하면 다음 줄의 prevEntryHash 가 어긋나 폭로된다."""
        import tempfile
        text = make_chain("anchorLog", [{"capsuleSha256": "a" * 64},
                                        {"capsuleSha256": "b" * 64}])
        lines = text.splitlines()
        first = json.loads(lines[0])
        first["capsuleSha256"] = "0" * 64            # 과거 줄 변조
        tampered = json.dumps(first, ensure_ascii=False) + "\n" + lines[1] + "\n"
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write(tampered)
            path = fh.name
        entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertIsNotNone(err, "과거 줄 변조가 잡히지 않았다")
        self.assertIn("prevEntryHash", err)

    def test_seq_gap_is_rejected(self):
        import tempfile
        text = make_chain("anchorLog", [{"x": 1}, {"x": 2}])
        lines = text.splitlines()
        second = json.loads(lines[1])
        second["seq"] = 5                             # 연번 위반
        broken = lines[0] + "\n" + json.dumps(second, ensure_ascii=False) + "\n"
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write(broken)
            path = fh.name
        _entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertIsNotNone(err)


class MerkleTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_merkle_root_is_deterministic_and_order_sensitive(self):
        leaves = [hashlib.sha256(x).hexdigest() for x in (b"a", b"b", b"c")]
        root1 = self.lb.merkle_root(leaves)
        root2 = self.lb.merkle_root(leaves)
        self.assertEqual(root1, root2)
        self.assertNotEqual(root1, self.lb.merkle_root(list(reversed(leaves))))

    def test_odd_layer_duplicates_last(self):
        one = self.lb.merkle_root([hashlib.sha256(b"x").hexdigest()])
        self.assertEqual(one, hashlib.sha256(b"x").hexdigest())


class CommittedBoardTests(unittest.TestCase):
    """커밋된 리더보드가 있으면 해시 체인 무결·스냅샷 봉인을 상시 확인한다."""

    def setUp(self):
        self.lb = load_lb()

    def test_committed_ledger_and_anchor_are_intact(self):
        import os
        if not os.path.exists(self.lb.LEDGER):
            self.skipTest("커밋된 리더보드 없음")
        _le, lerr = self.lb.chain_walk(self.lb.LEDGER, "settlementLedger")
        self.assertIsNone(lerr, f"원장 체인 파손: {lerr}")
        aentries, aerr = self.lb.chain_walk(self.lb.ANCHOR, "anchorLog")
        self.assertIsNone(aerr, f"앵커 체인 파손: {aerr}")
        # 원장 스냅샷 봉인 — 앵커의 마지막 항목이 현재 원장 바이트 해시여야 한다.
        if aentries:
            self.assertEqual(aentries[-1].get("capsuleSha256"),
                             self.lb.sha256_of(self.lb.LEDGER),
                             "원장이 앵커 이후 변경됨(꼬리 봉인 실패)")

    def test_ledger_entries_have_matching_claim_files(self):
        import os
        if not os.path.exists(self.lb.LEDGER):
            self.skipTest("커밋된 리더보드 없음")
        entries, _ = self.lb.chain_walk(self.lb.LEDGER, "settlementLedger")
        claim_hashes = {self.lb.sha256_of(os.path.join(self.lb.CLAIMS, n))
                        for n in os.listdir(self.lb.CLAIMS)} if os.path.isdir(self.lb.CLAIMS) else set()
        for e in entries:
            self.assertIn(e.get("claimSha256"), claim_hashes,
                          f"seq {e['seq']} 의 claim 파일이 원장 해시와 일치하지 않는다")


if __name__ == "__main__":
    unittest.main()
