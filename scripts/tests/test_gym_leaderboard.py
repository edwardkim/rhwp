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
import os
import sys
import tempfile
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

    def test_missing_file_returns_empty_and_none(self):
        missing = os.path.join(tempfile.gettempdir(),
                               "rhwp-lb-missing-chain-walk.ndjson")
        if os.path.exists(missing):
            os.remove(missing)
        entries, err = self.lb.chain_walk(missing, "settlementLedger")
        self.assertEqual(entries, [])
        self.assertIsNone(err)

    def test_empty_file_returns_empty_and_none(self):
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write("")
            path = fh.name
        entries, err = self.lb.chain_walk(path, "settlementLedger")
        Path(path).unlink()
        self.assertEqual(entries, [])
        self.assertIsNone(err)

    def test_blank_lines_only_file_is_empty_chain(self):
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write("\n\n  \n")
            path = fh.name
        entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertEqual(entries, [])
        self.assertIsNone(err)


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

    def test_empty_leaves_is_none(self):
        self.assertIsNone(self.lb.merkle_root([]))

    def test_two_leaf_pairing_is_hash_of_concat(self):
        a = hashlib.sha256(b"left").hexdigest()
        b = hashlib.sha256(b"right").hexdigest()
        root = self.lb.merkle_root([a, b])
        expected = hashlib.sha256((a + b).encode("ascii")).hexdigest()
        self.assertEqual(root, expected)


class CommittedBoardTests(unittest.TestCase):
    """커밋된 리더보드가 있으면 해시 체인 무결·스냅샷 봉인을 상시 확인한다."""

    def setUp(self):
        self.lb = load_lb()

    def test_committed_ledger_and_anchor_are_intact(self):
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
        if not os.path.exists(self.lb.LEDGER):
            self.skipTest("커밋된 리더보드 없음")
        entries, _ = self.lb.chain_walk(self.lb.LEDGER, "settlementLedger")
        claim_hashes = {self.lb.sha256_of(os.path.join(self.lb.CLAIMS, n))
                        for n in os.listdir(self.lb.CLAIMS)} if os.path.isdir(self.lb.CLAIMS) else set()
        for e in entries:
            self.assertIn(e.get("claimSha256"), claim_hashes,
                          f"seq {e['seq']} 의 claim 파일이 원장 해시와 일치하지 않는다")


class InviteTests(unittest.TestCase):
    """[#4664] 친구 초대 — 판 지문으로 신참이 위조본이 아님을 확인한다."""

    def setUp(self):
        self.lb = load_lb()

    def test_board_fingerprint_reports_committed_state(self):
        if not os.path.exists(self.lb.LEDGER):
            self.skipTest("커밋된 리더보드 없음")
        fp = self.lb.board_fingerprint()
        for key in ("members", "ledgerEntries", "ledgerChain", "anchorChain",
                    "merkleRoot", "workorderSha256", "ledgerSnapshotSha256"):
            self.assertIn(key, fp, f"지문에 {key} 가 없다")
        # 지문의 원장 항목 수는 실제 체인 길이와 같아야 한다(자기신고 아님).
        entries, _ = self.lb.chain_walk(self.lb.LEDGER, "settlementLedger")
        self.assertEqual(fp["ledgerEntries"], len(entries))
        # 스냅샷 해시는 커밋된 원장 파일 바이트에서 재계산 가능해야 한다.
        self.assertEqual(fp["ledgerSnapshotSha256"], self.lb.sha256_of(self.lb.LEDGER))


def _ok(agent, score, max_, seq, packs=None, commit="abc1234567890"):
    return {
        "ok": True,
        "agent": agent,
        "score": score,
        "max": max_,
        "seq": seq,
        "runner": {"rhwpCommit": commit},
        "packs": packs if packs is not None else {},
    }


def _bad(seq, why="claim 파일 없음"):
    return {"ok": False, "seq": seq, "why": why}


class RankResultsTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_higher_score_ranks_first(self):
        results = [
            _ok("low", 3, 10, 0),
            _ok("high", 9, 10, 1),
            _ok("mid", 6, 10, 2),
        ]
        ranked, unverified = self.lb.rank_results(results)
        self.assertEqual([r["agent"] for r in ranked], ["high", "mid", "low"])
        self.assertEqual(unverified, [])

    def test_tie_score_breaks_by_lower_seq(self):
        results = [
            _ok("late", 8, 10, 4),
            _ok("early", 8, 10, 1),
            _ok("mid", 8, 10, 2),
        ]
        ranked, unverified = self.lb.rank_results(results)
        self.assertEqual([r["seq"] for r in ranked], [1, 2, 4])
        self.assertEqual([r["agent"] for r in ranked], ["early", "mid", "late"])
        self.assertEqual(unverified, [])

    def test_unverified_excluded_from_ranked(self):
        results = [
            _ok("ok-a", 5, 10, 0),
            _bad(1),
            _ok("ok-b", 7, 10, 2),
            _bad(3, why="서명 실패"),
        ]
        ranked, unverified = self.lb.rank_results(results)
        self.assertEqual([r["agent"] for r in ranked], ["ok-b", "ok-a"])
        self.assertEqual([r["seq"] for r in unverified], [1, 3])
        self.assertTrue(all(not r.get("ok") for r in unverified))
        self.assertTrue(all(r.get("ok") for r in ranked))

    def test_empty_results_split_to_empty_lists(self):
        ranked, unverified = self.lb.rank_results([])
        self.assertEqual(ranked, [])
        self.assertEqual(unverified, [])

    def test_all_unverified_leaves_ranked_empty(self):
        results = [_bad(0), _bad(1)]
        ranked, unverified = self.lb.rank_results(results)
        self.assertEqual(ranked, [])
        self.assertEqual(len(unverified), 2)

    def test_missing_ok_key_is_treated_as_unverified(self):
        results = [{"seq": 0, "score": 99}, _ok("ok", 1, 1, 1)]
        ranked, unverified = self.lb.rank_results(results)
        self.assertEqual(len(ranked), 1)
        self.assertEqual(ranked[0]["agent"], "ok")
        self.assertEqual(unverified[0]["seq"], 0)


class BestPackTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_empty_packs_is_none(self):
        self.assertIsNone(self.lb.best_pack({}))
        self.assertIsNone(self.lb.best_pack(None or {}))

    def test_single_pack_is_returned(self):
        picked = self.lb.best_pack({"core-cli": {"score": 4, "max": 6}})
        self.assertEqual(picked, ("core-cli", 4, 6))

    def test_mixed_ratios_picks_highest_ratio(self):
        packs = {
            "easy": {"score": 10, "max": 10},
            "hard": {"score": 9, "max": 10},
            "wide": {"score": 20, "max": 40},
        }
        picked = self.lb.best_pack(packs)
        self.assertEqual(picked[0], "easy")
        self.assertEqual(picked[1:], (10, 10))

    def test_equal_ratio_breaks_by_larger_max(self):
        packs = {
            "narrow": {"score": 5, "max": 10},
            "wide": {"score": 10, "max": 20},
        }
        picked = self.lb.best_pack(packs)
        self.assertEqual(picked, ("wide", 10, 20))

    def test_zero_max_does_not_divide(self):
        packs = {
            "empty": {"score": 0, "max": 0},
            "real": {"score": 1, "max": 2},
        }
        picked = self.lb.best_pack(packs)
        self.assertEqual(picked, ("real", 1, 2))


class RenderMarkdownTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_header_unverified_and_score_numbers(self):
        results = [
            _ok("alpha", 12, 20, 0, packs={"core": {"score": 12, "max": 20}}),
            _bad(1),
        ]
        md = self.lb.render_markdown(results, None)
        self.assertIn("위조 불가능", md)
        self.assertIn("**unverified**", md)
        self.assertIn("12", md)
        self.assertIn("20", md)
        self.assertIn("| — | seq 1 |", md)
        self.assertIn("정직 조항", md)

    def test_honesty_clause_and_verified_row(self):
        results = [_ok("beta", 3, 5, 2, commit="deadbeefcafebabe")]
        md = self.lb.render_markdown(results, None)
        self.assertIn("등재되었고 이후 변조되지 않았다", md)
        self.assertIn("beta", md)
        self.assertIn("**3 / 5**", md)
        self.assertIn("`deadbeefca`", md)
        self.assertIn("| 1 |", md)
        self.assertIn("검증됨", md)
        self.assertNotIn("**unverified**", md)

    def test_broken_chain_is_labeled_in_footer(self):
        md = self.lb.render_markdown([_bad(0)], "1행: prevEntryHash 불일치")
        self.assertIn("원장 체인: 파손", md)
        self.assertIn("unverified 1", md)

    def test_empty_results_still_have_header_and_honesty(self):
        md = self.lb.render_markdown([], None)
        self.assertIn("# 운동장 리더보드 — 위조 불가능한 점수판", md)
        self.assertIn("위조 불가능", md)
        self.assertIn("정직 조항", md)
        self.assertIn("원장 체인: 무결", md)
        self.assertIn("항목 0", md)
        self.assertTrue(md.endswith("\n"))

    def test_explicit_ranked_argument_is_honored(self):
        results = [_ok("first", 1, 1, 0), _ok("second", 9, 9, 1)]
        ranked, _ = self.lb.rank_results(results)
        md = self.lb.render_markdown(results, None, ranked=ranked)
        first = md.index("first")
        second = md.index("second")
        self.assertLess(second, first)

    def test_ability_grid_appears_for_two_scored_agents(self):
        results = [
            _ok("a", 4, 4, 0, packs={"p1": {"score": 4, "max": 4}}),
            _ok("b", 2, 4, 1, packs={"p1": {"score": 2, "max": 4}}),
        ]
        md = self.lb.render_markdown(results, None)
        self.assertIn("## 능력 격자 (pack 별 점수)", md)
        self.assertIn("**4**", md)
        self.assertIn("2/4", md)


_BOARD_PATH_NAMES = (
    "BOARD", "KEYS", "CLAIMS", "LEDGER", "ANCHOR",
    "CHECKPOINT", "KEYRING", "WORKORDER",
)


class InviteEnvelopeTests(unittest.TestCase):
    """초대 봉투는 임시 판에서만 읽고, 커밋된 gym/leaderboard 에 쓰지 않는다."""

    def setUp(self):
        self.lb = load_lb()
        self._saved = {name: getattr(self.lb, name) for name in _BOARD_PATH_NAMES}
        self._real_invite = Path(self._saved["BOARD"]) / "invite.json"
        self._invite_before = (
            self._real_invite.stat().st_mtime_ns if self._real_invite.exists() else None
        )

    def tearDown(self):
        for name, value in self._saved.items():
            setattr(self.lb, name, value)
        if self._invite_before is None:
            self.assertFalse(
                self._real_invite.exists(),
                "커밋된 판에 invite.json 이 생겼다",
            )
        else:
            self.assertEqual(self._real_invite.stat().st_mtime_ns, self._invite_before)

    def _redirect(self, tmp):
        board = Path(tmp) / "leaderboard"
        board.mkdir()
        (board / "keys").mkdir()
        (board / "claims").mkdir()
        self.lb.BOARD = str(board)
        self.lb.KEYS = str(board / "keys")
        self.lb.CLAIMS = str(board / "claims")
        self.lb.LEDGER = str(board / "ledger.ndjson")
        self.lb.ANCHOR = str(board / "anchor.ndjson")
        self.lb.CHECKPOINT = str(board / "checkpoint.json")
        self.lb.KEYRING = str(board / "keyring.json")
        self.lb.WORKORDER = str(board / "workorder.json")
        return board

    def test_fingerprint_on_empty_tmp_board(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._redirect(tmp)
            fp = self.lb.board_fingerprint()
            self.assertEqual(fp["members"], 0)
            self.assertEqual(fp["ledgerEntries"], 0)
            self.assertEqual(fp["ledgerChain"], "ok")
            self.assertEqual(fp["anchorChain"], "ok")
            self.assertIsNone(fp["merkleRoot"])
            self.assertIsNone(fp["workorderSha256"])
            self.assertIsNone(fp["ledgerSnapshotSha256"])

    def test_construct_invite_uses_tmp_fingerprint(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = self._redirect(tmp)
            workorder = {
                "schemaVersion": "1.0",
                "kind": "workorder",
                "workorderId": "tmp-invite",
            }
            self.lb.write_json(self.lb.WORKORDER, workorder)
            self.lb.write_json(self.lb.KEYRING, {
                "schemaVersion": "1.0", "kind": "keyring",
                "keys": [{"keyId": "gym/tmp", "publicKey": "aa", "revoked": None}],
            })
            Path(self.lb.LEDGER).write_text("", encoding="utf-8")
            Path(self.lb.ANCHOR).write_text("", encoding="utf-8")
            self.lb.write_json(self.lb.CHECKPOINT, {"merkleRoot": "ab" * 32})
            invite = self.lb.construct_invite("tmp-guest")
            self.assertEqual(invite["kind"], "gymLeaderboardInvite")
            self.assertEqual(invite["guest"], "tmp-guest")
            self.assertEqual(invite["board"]["path"], "gym/leaderboard")
            self.assertEqual(invite["fingerprint"]["members"], 1)
            self.assertEqual(invite["fingerprint"]["ledgerEntries"], 0)
            self.assertEqual(invite["fingerprint"]["merkleRoot"], "ab" * 32)
            self.assertEqual(
                invite["fingerprint"]["workorderSha256"],
                self.lb.sha256_of(self.lb.WORKORDER),
            )
            self.assertTrue(any("tmp-guest" in step for step in invite["join"]))
            self.assertEqual(len(invite["join"]), 3)
            self.assertFalse((board / "invite.json").exists())

    def test_construct_invite_does_not_touch_committed_board(self):
        real_board = Path(self._saved["BOARD"])
        before = {}
        if real_board.is_dir():
            for p in real_board.rglob("*"):
                if p.is_file():
                    before[str(p.relative_to(real_board))] = p.stat().st_mtime_ns
        with tempfile.TemporaryDirectory() as tmp:
            self._redirect(tmp)
            _invite = self.lb.construct_invite("probe-agent")
            fp = self.lb.board_fingerprint()
            self.assertEqual(fp["ledgerEntries"], 0)
        if real_board.is_dir():
            after = {
                str(p.relative_to(real_board)): p.stat().st_mtime_ns
                for p in real_board.rglob("*") if p.is_file()
            }
            self.assertEqual(after, before)


if __name__ == "__main__":
    unittest.main()
