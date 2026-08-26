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


class ExceptionClassifyTests(unittest.TestCase):
    """I/O·형식·사슬·CLI·스키마 예외 분류. 새 암호 코드는 없다."""

    def setUp(self):
        self.lb = load_lb()

    def test_error_codes_are_the_declared_set(self):
        self.assertEqual(
            set(self.lb.ERROR_CODES),
            {"io", "format", "chain", "cli", "schema", "leaderboard"},
        )

    def test_hierarchy_codes(self):
        cases = (
            (self.lb.LeaderboardError("x"), "leaderboard"),
            (self.lb.LeaderboardIOError("x"), "io"),
            (self.lb.LeaderboardFormatError("x"), "format"),
            (self.lb.LeaderboardChainError("x"), "chain"),
            (self.lb.LeaderboardCliError("x"), "cli"),
            (self.lb.LeaderboardSchemaError("x"), "schema"),
        )
        for exc, code in cases:
            self.assertEqual(self.lb.classify_exception(exc), code, code)
            self.assertEqual(exc.code, code)

    def test_stdlib_exceptions_fold(self):
        self.assertEqual(self.lb.classify_exception(FileNotFoundError("n")), "io")
        self.assertEqual(self.lb.classify_exception(PermissionError("p")), "io")
        self.assertEqual(self.lb.classify_exception(OSError("o")), "io")
        self.assertEqual(
            self.lb.classify_exception(json.JSONDecodeError("m", "d", 0)),
            "format",
        )
        self.assertEqual(self.lb.classify_exception(UnicodeDecodeError("utf-8", b"\xff", 0, 1, "x")), "format")
        self.assertEqual(self.lb.classify_exception(TypeError("t")), "schema")
        self.assertEqual(self.lb.classify_exception(ValueError("v")), "schema")
        self.assertEqual(self.lb.classify_exception(KeyError("k")), "schema")
        self.assertEqual(self.lb.classify_exception(RuntimeError("r")), "leaderboard")

    def test_as_dict_includes_path_and_cause(self):
        cause = FileNotFoundError("nope")
        exc = self.lb.LeaderboardIOError("읽기 실패", path="/tmp/x", cause=cause)
        body = exc.as_dict()
        self.assertFalse(body["ok"])
        self.assertEqual(body["code"], "io")
        self.assertIn("읽기 실패", body["why"])
        self.assertEqual(body["path"], "/tmp/x")
        self.assertEqual(body["causeType"], "FileNotFoundError")

    def test_format_classified_includes_code_and_path(self):
        exc = self.lb.LeaderboardFormatError("깨짐", path="a.json")
        text = self.lb.format_classified(exc, "verify 실패")
        self.assertIn("verify 실패", text)
        self.assertIn("(format)", text)
        self.assertIn("a.json", text)
        self.assertIn("깨짐", text)

    def test_empty_fingerprint_has_all_keys(self):
        fp = self.lb.empty_fingerprint()
        self.assertEqual(set(fp), set(self.lb.FINGERPRINT_KEYS))
        self.assertEqual(fp["members"], 0)
        self.assertEqual(fp["ledgerEntries"], 0)
        self.assertEqual(fp["ledgerChain"], "ok")
        self.assertIsNone(fp["merkleRoot"])


class JsonIoExceptionTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_sha256_of_missing_raises_io(self):
        missing = os.path.join(tempfile.gettempdir(), "rhwp-lb-no-such-sha256.bin")
        if os.path.exists(missing):
            os.remove(missing)
        with self.assertRaises(self.lb.LeaderboardIOError) as ctx:
            self.lb.sha256_of(missing)
        self.assertEqual(ctx.exception.code, "io")
        self.assertEqual(ctx.exception.path, missing)

    def test_try_sha256_missing_is_none_and_err(self):
        missing = os.path.join(tempfile.gettempdir(), "rhwp-lb-no-such-try-sha.bin")
        if os.path.exists(missing):
            os.remove(missing)
        digest, err = self.lb.try_sha256(missing)
        self.assertIsNone(digest)
        self.assertIsNotNone(err)
        self.assertIn("sha256", err)

    def test_sha256_of_bytes_matches_hashlib(self):
        with tempfile.NamedTemporaryFile("wb", delete=False) as fh:
            fh.write(b"gym-leaderboard-sha")
            path = fh.name
        digest = self.lb.sha256_of(path)
        Path(path).unlink()
        self.assertEqual(digest, hashlib.sha256(b"gym-leaderboard-sha").hexdigest())
        self.assertEqual(len(digest), 64)

    def test_read_json_missing_raises_io(self):
        missing = os.path.join(tempfile.gettempdir(), "rhwp-lb-missing.json")
        if os.path.exists(missing):
            os.remove(missing)
        with self.assertRaises(self.lb.LeaderboardIOError):
            self.lb.read_json(missing)

    def test_read_json_malformed_raises_format(self):
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False,
                                         encoding="utf-8") as fh:
            fh.write("{not json")
            path = fh.name
        with self.assertRaises(self.lb.LeaderboardFormatError) as ctx:
            self.lb.read_json(path)
        Path(path).unlink()
        self.assertEqual(ctx.exception.code, "format")

    def test_try_read_json_malformed_is_none(self):
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False,
                                         encoding="utf-8") as fh:
            fh.write("[")
            path = fh.name
        obj, err = self.lb.try_read_json(path)
        Path(path).unlink()
        self.assertIsNone(obj)
        self.assertIsNotNone(err)

    def test_write_json_rejects_unserializable(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "bad.json")
            with self.assertRaises(self.lb.LeaderboardFormatError):
                self.lb.write_json(path, {"fn": lambda: None})

    def test_try_write_json_success_roundtrip(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "ok.json")
            err = self.lb.try_write_json(path, {"k": "한글", "n": 1})
            self.assertIsNone(err)
            body = self.lb.read_json(path)
            self.assertEqual(body["k"], "한글")
            self.assertEqual(body["n"], 1)

    def test_try_write_json_to_directory_is_error(self):
        with tempfile.TemporaryDirectory() as tmp:
            err = self.lb.try_write_json(tmp, {"k": 1})
            self.assertIsNotNone(err)

    def test_safe_listdir_missing_is_empty(self):
        missing = os.path.join(tempfile.gettempdir(), "rhwp-lb-no-dir-xxxx")
        if os.path.exists(missing):
            os.remove(missing) if os.path.isfile(missing) else None
        names, err = self.lb.safe_listdir(missing)
        self.assertEqual(names, [])
        self.assertIsNone(err)

    def test_safe_listdir_file_is_error(self):
        with tempfile.NamedTemporaryFile("w", delete=False, encoding="utf-8") as fh:
            fh.write("x")
            path = fh.name
        names, err = self.lb.safe_listdir(path)
        Path(path).unlink()
        self.assertEqual(names, [])
        self.assertIsNotNone(err)

    def test_safe_listdir_sorts(self):
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, "b.txt").write_text("b", encoding="utf-8")
            Path(tmp, "a.txt").write_text("a", encoding="utf-8")
            names, err = self.lb.safe_listdir(tmp)
            self.assertIsNone(err)
            self.assertEqual(names, ["a.txt", "b.txt"])


class ChainWalkMalformedTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def _write(self, text):
        fh = tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n")
        fh.write(text)
        fh.close()
        return fh.name

    def test_empty_path_is_error(self):
        entries, err = self.lb.chain_walk("", "settlementLedger")
        self.assertEqual(entries, [])
        self.assertIsNotNone(err)

    def test_malformed_json_is_row_error_not_raise(self):
        path = self._write('{"seq": 0, "kind": "anchorLog", "prevEntryHash": null}\n{bad\n')
        entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertEqual(len(entries), 1)
        self.assertIsNotNone(err)
        self.assertIn("JSON", err)

    def test_non_object_row_is_rejected(self):
        path = self._write("[1, 2, 3]\n")
        entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertEqual(entries, [])
        self.assertIn("객체", err)

    def test_kind_mismatch_keeps_prior_rows(self):
        text = make_chain("settlementLedger", [{"x": 1}, {"x": 2}])
        lines = text.splitlines()
        second = json.loads(lines[1])
        second["kind"] = "other"
        path = self._write(lines[0] + "\n" + json.dumps(second, ensure_ascii=False) + "\n")
        entries, err = self.lb.chain_walk(path, "settlementLedger")
        Path(path).unlink()
        self.assertEqual(len(entries), 1)
        self.assertIn("kind", err)

    def test_wrong_kind_argument_on_valid_chain(self):
        path = self._write(make_chain("settlementLedger", [{"x": 1}]))
        entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertEqual(entries, [])
        self.assertIn("kind", err)

    def test_line_hashes_missing_is_empty(self):
        missing = os.path.join(tempfile.gettempdir(), "rhwp-lb-no-lines.ndjson")
        if os.path.exists(missing):
            os.remove(missing)
        self.assertEqual(self.lb.line_hashes(missing), [])
        self.assertEqual(self.lb.line_hashes(""), [])

    def test_line_hashes_skips_blank(self):
        path = self._write("\n\nabc\n\n")
        hashes = self.lb.line_hashes(path)
        Path(path).unlink()
        self.assertEqual(hashes, [hashlib.sha256(b"abc").hexdigest()])

    def test_try_line_hashes_empty_path(self):
        hashes, err = self.lb.try_line_hashes("")
        self.assertEqual(hashes, [])
        self.assertIsNone(err)


class MerkleSchemaTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_string_leaf_is_schema_error(self):
        with self.assertRaises(self.lb.LeaderboardSchemaError):
            self.lb.merkle_root("abcd")

    def test_bytes_leaf_container_is_schema_error(self):
        with self.assertRaises(self.lb.LeaderboardSchemaError):
            self.lb.merkle_root(b"abcd")

    def test_non_string_item_is_schema_error(self):
        with self.assertRaises(self.lb.LeaderboardSchemaError):
            self.lb.merkle_root([1, 2])

    def test_none_container_is_schema_or_empty(self):
        # None 은 거짓이라 빈 잎과 같다.
        self.assertIsNone(self.lb.merkle_root(None))

    def test_odd_three_duplicates_last(self):
        leaves = [hashlib.sha256(x).hexdigest() for x in (b"a", b"b", b"c")]
        root = self.lb.merkle_root(leaves)
        pair_ab = hashlib.sha256((leaves[0] + leaves[1]).encode("ascii")).hexdigest()
        pair_cc = hashlib.sha256((leaves[2] + leaves[2]).encode("ascii")).hexdigest()
        expected = hashlib.sha256((pair_ab + pair_cc).encode("ascii")).hexdigest()
        self.assertEqual(root, expected)


class FingerprintAndInviteSchemaTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()
        self._saved = {name: getattr(self.lb, name) for name in _BOARD_PATH_NAMES}

    def tearDown(self):
        for name, value in self._saved.items():
            setattr(self.lb, name, value)

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

    def test_validate_fingerprint_empty_ok(self):
        issues = self.lb.validate_fingerprint(self.lb.empty_fingerprint())
        self.assertEqual(issues, [])

    def test_validate_fingerprint_missing_key(self):
        fp = self.lb.empty_fingerprint()
        del fp["merkleRoot"]
        issues = self.lb.validate_fingerprint(fp)
        self.assertTrue(any("merkleRoot" in i for i in issues))

    def test_validate_fingerprint_bad_types(self):
        fp = self.lb.empty_fingerprint()
        fp["members"] = "1"
        fp["ledgerEntries"] = 1.5
        fp["workorderSha256"] = 12
        issues = self.lb.validate_fingerprint(fp)
        self.assertGreaterEqual(len(issues), 3)

    def test_validate_fingerprint_short_hash(self):
        fp = self.lb.empty_fingerprint()
        fp["workorderSha256"] = "ab"
        issues = self.lb.validate_fingerprint(fp)
        self.assertTrue(any("64" in i for i in issues))

    def test_validate_fingerprint_not_object(self):
        self.assertEqual(self.lb.validate_fingerprint(None), ["지문이 객체가 아니다"])
        self.assertEqual(self.lb.validate_fingerprint([]), ["지문이 객체가 아니다"])

    def test_construct_invite_blank_guest_uses_default(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._redirect(tmp)
            invite = self.lb.construct_invite("   ")
            self.assertEqual(invite["guest"], self.lb.DEFAULT_GUEST)
            invite2 = self.lb.construct_invite(None)
            self.assertEqual(invite2["guest"], self.lb.DEFAULT_GUEST)

    def test_construct_invite_rejects_non_string_guest(self):
        with self.assertRaises(self.lb.LeaderboardSchemaError):
            self.lb.construct_invite(123)

    def test_validate_invite_of_construct(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._redirect(tmp)
            invite = self.lb.construct_invite("guest-a")
            self.assertEqual(self.lb.validate_invite(invite), [])
            self.assertEqual(invite["kind"], self.lb.INVITE_KIND)
            self.assertEqual(invite["schemaVersion"], self.lb.SCHEMA_VERSION)
            self.assertEqual(len(invite["join"]), self.lb.JOIN_STEP_COUNT)

    def test_validate_invite_detects_broken_join(self):
        invite = {
            "schemaVersion": "9.9",
            "kind": "nope",
            "guest": 1,
            "board": {"repo": "x"},
            "fingerprint": {},
            "join": ["only-one"],
            "promise": "",
            "note": "",
        }
        issues = self.lb.validate_invite(invite)
        self.assertTrue(any("schemaVersion" in i for i in issues))
        self.assertTrue(any("kind" in i for i in issues))
        self.assertTrue(any("guest" in i for i in issues))
        self.assertTrue(any("join" in i for i in issues))
        self.assertTrue(any("board.path" in i for i in issues))

    def test_validate_invite_not_object(self):
        self.assertEqual(self.lb.validate_invite("x"), ["초대장이 객체가 아니다"])

    def test_validate_invite_join_missing_verbs(self):
        invite = self.lb.construct_invite("g")
        invite["join"] = ["a", "b", "c"]
        issues = self.lb.validate_invite(invite)
        self.assertTrue(any("score.py" in i for i in issues))
        self.assertTrue(any("attest" in i for i in issues))
        self.assertTrue(any("verify" in i for i in issues))

    def test_fingerprint_bad_keyring_does_not_raise(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = self._redirect(tmp)
            Path(self.lb.KEYRING).write_text("{bad", encoding="utf-8")
            Path(self.lb.LEDGER).write_text("", encoding="utf-8")
            fp = self.lb.board_fingerprint()
            self.assertEqual(fp["members"], 0)
            self.assertEqual(fp["ledgerEntries"], 0)
            self.assertIsNone(fp["workorderSha256"])
            self.assertFalse((board / "invite.json").exists())

    def test_fingerprint_keyring_keys_not_list(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._redirect(tmp)
            self.lb.write_json(self.lb.KEYRING, {"keys": "nope"})
            fp = self.lb.board_fingerprint()
            self.assertEqual(fp["members"], 0)

    def test_cmd_invite_writes_only_tmp_board(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = self._redirect(tmp)
            ns = argparse_ns(agent="tmp-cmd")
            code = self.lb.cmd_invite(ns, None)
            self.assertEqual(code, 0)
            invite = self.lb.read_json(str(board / "invite.json"))
            self.assertEqual(invite["guest"], "tmp-cmd")
            self.assertEqual(self.lb.validate_invite(invite), [])

    def test_cmd_invite_default_guest(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = self._redirect(tmp)
            ns = argparse_ns(agent=None)
            self.assertEqual(self.lb.cmd_invite(ns, None), 0)
            invite = self.lb.read_json(str(board / "invite.json"))
            self.assertEqual(invite["guest"], self.lb.DEFAULT_GUEST)


def argparse_ns(**kwargs):
    class NS:
        pass
    ns = NS()
    for key, value in kwargs.items():
        setattr(ns, key, value)
    return ns


class RankAndPackExceptionTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_rank_results_non_list_is_empty(self):
        self.assertEqual(self.lb.rank_results(None), ([], []))
        self.assertEqual(self.lb.rank_results("abc"), ([], []))
        self.assertEqual(self.lb.rank_results(b"xx"), ([], []))

    def test_rank_results_non_mapping_row_is_unverified(self):
        ranked, unverified = self.lb.rank_results(["nope", 1, None])
        self.assertEqual(ranked, [])
        self.assertEqual(len(unverified), 3)
        self.assertTrue(all(not r.get("ok") for r in unverified))

    def test_ok_row_without_score_is_unverified(self):
        results = [{"ok": True, "agent": "ghost", "seq": 0}, _ok("ok", 1, 1, 1)]
        ranked, unverified = self.lb.rank_results(results)
        self.assertEqual([r["agent"] for r in ranked], ["ok"])
        self.assertEqual(unverified[0]["agent"], "ghost")
        self.assertIn("score", unverified[0].get("why", ""))

    def test_ok_row_with_non_numeric_score_is_unverified(self):
        results = [{"ok": True, "score": "NaN", "seq": 0, "agent": "bad"}]
        ranked, unverified = self.lb.rank_results(results)
        self.assertEqual(ranked, [])
        self.assertEqual(len(unverified), 1)

    def test_tuple_of_rows_is_accepted(self):
        ranked, unverified = self.lb.rank_results((_ok("a", 2, 2, 1), _ok("b", 3, 3, 0)))
        self.assertEqual([r["agent"] for r in ranked], ["b", "a"])
        self.assertEqual(unverified, [])

    def test_pack_ratio_missing_and_zero(self):
        self.assertIsNone(self.lb.pack_ratio(None))
        self.assertIsNone(self.lb.pack_ratio("x"))
        self.assertIsNone(self.lb.pack_ratio({"score": 1}))
        self.assertIsNone(self.lb.pack_ratio({"score": "a", "max": 2}))
        self.assertEqual(self.lb.pack_ratio({"score": 0, "max": 0}), (0.0, 0.0))
        self.assertEqual(self.lb.pack_ratio({"score": 1, "max": 4})[0], 0.25)

    def test_best_pack_skips_malformed(self):
        packs = {
            "bad": {"score": 9},
            "also": "nope",
            "good": {"score": 2, "max": 4},
        }
        self.assertEqual(self.lb.best_pack(packs), ("good", 2, 4))

    def test_best_pack_none_and_list(self):
        self.assertIsNone(self.lb.best_pack(None))
        self.assertIsNone(self.lb.best_pack([{"score": 1, "max": 1}]))

    def test_best_pack_all_malformed_is_none(self):
        self.assertIsNone(self.lb.best_pack({"a": {}, "b": {"max": 1}}))

    def test_validate_rank_row_ok_and_broken(self):
        ok = _ok("a", 1, 1, 0)
        self.assertEqual(self.lb.validate_rank_row(ok), [])
        self.assertTrue(self.lb.validate_rank_row("x"))
        issues = self.lb.validate_rank_row({"ok": True, "agent": "z"})
        self.assertTrue(any("score" in i for i in issues))

    def test_rank_key_none_for_missing(self):
        self.assertIsNone(self.lb.rank_key(None))
        self.assertIsNone(self.lb.rank_key({"ok": True}))
        self.assertEqual(self.lb.rank_key({"score": 3, "seq": 2}), (-3.0, 2))


class RenderDefensiveTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_missing_runner_does_not_raise(self):
        row = {"ok": True, "agent": "solo", "score": 1, "max": 1, "seq": 0}
        md = self.lb.render_markdown([row], None)
        self.assertIn("solo", md)
        self.assertIn("—", md)
        self.assertIn("정직 조항", md)

    def test_non_list_results_still_header(self):
        md = self.lb.render_markdown(None, None)
        self.assertIn("# 운동장 리더보드", md)
        self.assertIn("항목 0", md)

    def test_malformed_packs_in_grid_are_dash(self):
        results = [
            _ok("a", 4, 4, 0, packs={"p1": {"score": 4, "max": 4}}),
            _ok("b", 2, 4, 1, packs={"p1": "broken"}),
        ]
        md = self.lb.render_markdown(results, None)
        self.assertIn("## 능력 격자", md)
        self.assertIn("—", md)

    def test_short_commit_variants(self):
        self.assertEqual(self.lb.short_commit(None), "—")
        self.assertEqual(self.lb.short_commit({}), "—")
        self.assertEqual(self.lb.short_commit({"rhwpCommit": ""}), "—")
        self.assertEqual(self.lb.short_commit({"rhwpCommit": "abcdefghijklmn"}), "`abcdefghij`")

    def test_unverified_without_seq(self):
        md = self.lb.render_markdown([{"ok": False}], None)
        self.assertIn("**unverified**", md)
        self.assertIn("seq —", md)

    def test_broken_ranked_argument_does_not_raise(self):
        md = self.lb.render_markdown([_ok("a", 1, 1, 0)], None, ranked="nope")
        self.assertIn("위조 불가능", md)

    def test_grid_skips_non_mapping_ranked(self):
        results = [_ok("a", 1, 1, 0, packs={"p": {"score": 1, "max": 1}}),
                   _ok("b", 1, 1, 1, packs={"p": {"score": 1, "max": 1}})]
        md = self.lb.render_markdown(results, None, ranked=["x", results[0], results[1]])
        self.assertIn("a", md)


class CommandWrapperTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()
        self._saved = {name: getattr(self.lb, name) for name in _BOARD_PATH_NAMES}

    def tearDown(self):
        for name, value in self._saved.items():
            setattr(self.lb, name, value)

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

    def test_cmd_attest_requires_agent(self):
        with self.assertRaises(SystemExit) as ctx:
            self.lb.cmd_attest(argparse_ns(agent=None), "bin")
        self.assertIn("agent", str(ctx.exception))

    def test_cmd_attest_missing_scorecard(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._redirect(tmp)
            saved_gym = self.lb.GYM
            self.lb.GYM = tmp
            try:
                with self.assertRaises(SystemExit) as ctx:
                    self.lb.cmd_attest(argparse_ns(agent="ghost"), "bin")
                self.assertIn("없음", str(ctx.exception))
            finally:
                self.lb.GYM = saved_gym

    def test_cmd_verify_empty_board_returns_3(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._redirect(tmp)
            code = self.lb.cmd_verify(argparse_ns(), None)
            self.assertEqual(code, 3)
            ver = self.lb.read_json(self.lb.BOARD + os.sep + "verification.json")
            self.assertEqual(ver["kind"], self.lb.VERIFICATION_KIND)
            self.assertEqual(ver["verified"], 0)
            self.assertEqual(ver["ledgerEntries"], 0)

    def test_cmd_render_empty_board_writes_markdown(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = self._redirect(tmp)
            code = self.lb.cmd_render(argparse_ns(), None)
            self.assertEqual(code, 0)
            text = (board / "leaderboard.md").read_text(encoding="utf-8")
            self.assertIn("정직 조항", text)
            self.assertIn("항목 0", text)

    def test_run_cli_empty_bin_is_cli_error(self):
        with self.assertRaises(self.lb.LeaderboardCliError):
            self.lb.run_cli("", ["info"])
        with self.assertRaises(self.lb.LeaderboardCliError):
            self.lb.run_cli(None, ["info"])

    def test_run_cli_missing_binary_is_cli_error(self):
        missing = os.path.join(tempfile.gettempdir(), "rhwp-lb-no-bin-xxxx.exe")
        with self.assertRaises(self.lb.LeaderboardCliError):
            self.lb.run_cli(missing, ["--version"])

    def test_main_invite_does_not_need_bin(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = self._redirect(tmp)
            code = self.lb.main(["invite", "--agent", "main-guest"])
            self.assertEqual(code, 0)
            invite = self.lb.read_json(str(board / "invite.json"))
            self.assertEqual(invite["guest"], "main-guest")

    def test_main_attest_without_agent_exits(self):
        with self.assertRaises(SystemExit) as ctx:
            self.lb.main(["attest"])
        self.assertIn("agent", str(ctx.exception))

    def test_modes_tuple_matches_parser(self):
        self.assertEqual(set(self.lb.MODES), {"attest", "verify", "render", "invite"})

    def test_default_workorder_and_keyring_schema(self):
        wo = self.lb.default_workorder()
        ring = self.lb.default_keyring()
        self.assertEqual(wo["schemaVersion"], self.lb.SCHEMA_VERSION)
        self.assertEqual(wo["kind"], "workorder")
        self.assertEqual(ring["kind"], "keyring")
        self.assertEqual(ring["keys"], [])

    def test_ensure_board_creates_defaults(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = self._redirect(tmp)
            self.lb.ensure_board()
            self.assertTrue((board / "keys").is_dir())
            self.assertTrue((board / "claims").is_dir())
            wo = self.lb.read_json(self.lb.WORKORDER)
            self.assertEqual(wo["workorderId"], "gym-leaderboard-standing")
            ring = self.lb.read_json(self.lb.KEYRING)
            self.assertEqual(ring["keys"], [])


class VerifyEntryExceptionTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()
        self._saved = {name: getattr(self.lb, name) for name in _BOARD_PATH_NAMES}

    def tearDown(self):
        for name, value in self._saved.items():
            setattr(self.lb, name, value)

    def test_non_dict_entry(self):
        result = self.lb.verify_entry("bin", "nope", [])
        self.assertFalse(result["ok"])
        self.assertIn("객체", result["why"])

    def test_missing_claim_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = Path(tmp) / "leaderboard"
            board.mkdir()
            (board / "claims").mkdir()
            self.lb.BOARD = str(board)
            self.lb.CLAIMS = str(board / "claims")
            result = self.lb.verify_entry("bin", {"seq": 0, "claimSha256": "ab" * 32}, [])
            self.assertFalse(result["ok"])
            self.assertIn("claim", result["why"])

    def test_claim_json_broken(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = Path(tmp) / "leaderboard"
            claims = board / "claims"
            claims.mkdir(parents=True)
            payload = b'{"not": "matching-hash"}'
            path = claims / "broken.claim.json"
            path.write_bytes(payload)
            digest = hashlib.sha256(payload).hexdigest()
            self.lb.BOARD = str(board)
            self.lb.CLAIMS = str(claims)
            # 깨진 JSON 이지만 해시가 원장과 같으면 읽기 단계에서 접힌다.
            path.write_text("{bad", encoding="utf-8")
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            result = self.lb.verify_entry("bin", {"seq": 1, "claimSha256": digest}, [])
            self.assertFalse(result["ok"])
            self.assertTrue("claim" in result["why"] or "읽기" in result["why"])

    def test_claims_path_is_file(self):
        with tempfile.NamedTemporaryFile("w", delete=False, encoding="utf-8") as fh:
            fh.write("x")
            path = fh.name
        self.lb.CLAIMS = path
        result = self.lb.verify_entry("bin", {"seq": 0, "claimSha256": "00"}, [])
        Path(path).unlink()
        self.assertFalse(result["ok"])
        self.assertIn("claims", result["why"])


class ConstantContractTests(unittest.TestCase):
    """문서·시험·코드가 같은 키 집합을 본다."""

    def setUp(self):
        self.lb = load_lb()

    def test_invite_keys(self):
        self.assertEqual(
            set(self.lb.INVITE_KEYS),
            {"schemaVersion", "kind", "guest", "board", "fingerprint",
             "join", "promise", "note"},
        )
        self.assertEqual(set(self.lb.INVITE_BOARD_KEYS), {"repo", "path"})

    def test_fingerprint_keys(self):
        self.assertEqual(
            set(self.lb.FINGERPRINT_KEYS),
            {"members", "ledgerEntries", "ledgerChain", "anchorChain",
             "merkleRoot", "workorderSha256", "ledgerSnapshotSha256"},
        )

    def test_schema_and_kinds(self):
        self.assertEqual(self.lb.SCHEMA_VERSION, "1.0")
        self.assertEqual(self.lb.INVITE_KIND, "gymLeaderboardInvite")
        self.assertEqual(self.lb.VERIFICATION_KIND, "gymLeaderboardVerification")
        self.assertEqual(self.lb.JOIN_STEP_COUNT, 3)
        self.assertEqual(self.lb.COMMIT_SHORT, 10)
        self.assertEqual(self.lb.DEFAULT_GUEST, "친구-에이전트")

    def test_pack_and_rank_key_names(self):
        self.assertEqual(set(self.lb.PACK_SCORE_KEYS), {"score", "max"})
        for key in ("ok", "agent", "score", "max", "seq", "runner"):
            self.assertIn(key, self.lb.RANK_OK_KEYS)

    def test_no_new_cli_modes(self):
        self.assertEqual(len(self.lb.MODES), 4)
        self.assertNotIn("sign", self.lb.MODES)
        self.assertNotIn("keygen", self.lb.MODES)

    def test_hashlib_sha256_is_the_only_digest(self):
        # 모듈이 새 암호 원시함수를 들이지 않았는지 — hashlib.sha256 만 쓴다.
        source = Path(LB_PATH).read_text(encoding="utf-8")
        self.assertNotIn("ed25519", source.lower().replace("ed25519 서명", ""))
        self.assertIn("hashlib.sha256", source)
        self.assertNotIn("nacl", source)
        self.assertNotIn("cryptography", source)
        self.assertNotIn("from hashlib import blake2", source)

    def test_module_doc_points_at_docs(self):
        source = Path(LB_PATH).read_text(encoding="utf-8")
        self.assertIn("gym/docs/leaderboard.md", source)
        self.assertIn("README_leaderboard.md", source)
        self.assertIn("새 암호", source)


class AttestAdmissionExceptionTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()
        self._saved = {name: getattr(self.lb, name) for name in _BOARD_PATH_NAMES}
        self._gym = self.lb.GYM

    def tearDown(self):
        for name, value in self._saved.items():
            setattr(self.lb, name, value)
        self.lb.GYM = self._gym

    def test_admission_not_object(self):
        with tempfile.TemporaryDirectory() as tmp:
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
            self.lb.GYM = tmp
            sub = Path(tmp) / "submissions" / "x"
            sub.mkdir(parents=True)
            (sub / "scorecard.json").write_text("{}", encoding="utf-8")
            (sub / "admission.json").write_text("[1, 2]", encoding="utf-8")
            with self.assertRaises(SystemExit) as ctx:
                self.lb.cmd_attest(argparse_ns(agent="x"), "bin")
            self.assertIn("객체", str(ctx.exception))

    def test_admission_deny(self):
        with tempfile.TemporaryDirectory() as tmp:
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
            self.lb.GYM = tmp
            sub = Path(tmp) / "submissions" / "x"
            sub.mkdir(parents=True)
            (sub / "scorecard.json").write_text("{}", encoding="utf-8")
            (sub / "admission.json").write_text(
                json.dumps({"verdict": "deny"}), encoding="utf-8")
            with self.assertRaises(SystemExit) as ctx:
                self.lb.cmd_attest(argparse_ns(agent="x"), "bin")
            self.assertIn("deny", str(ctx.exception))


class ResolveBinTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_empty_path_from_find_bin(self):
        saved = self.lb.runner.find_bin

        def fake(_explicit):
            return ""

        self.lb.runner.find_bin = fake
        try:
            with self.assertRaises(self.lb.LeaderboardCliError):
                self.lb.resolve_bin(None)
        finally:
            self.lb.runner.find_bin = saved

    def test_systemexit_from_find_bin(self):
        saved = self.lb.runner.find_bin

        def fake(_explicit):
            raise SystemExit("없음")

        self.lb.runner.find_bin = fake
        try:
            with self.assertRaises(self.lb.LeaderboardCliError) as ctx:
                self.lb.resolve_bin("nope")
            self.assertIn("없음", str(ctx.exception))
        finally:
            self.lb.runner.find_bin = saved

    def test_other_exception_from_find_bin(self):
        saved = self.lb.runner.find_bin

        def fake(_explicit):
            raise RuntimeError("boom")

        self.lb.runner.find_bin = fake
        try:
            with self.assertRaises(self.lb.LeaderboardCliError):
                self.lb.resolve_bin(None)
        finally:
            self.lb.runner.find_bin = saved


class RenderMoreEdgeTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_single_ranked_has_no_grid(self):
        md = self.lb.render_markdown(
            [_ok("only", 1, 1, 0, packs={"p": {"score": 1, "max": 1}})], None)
        self.assertNotIn("능력 격자", md)

    def test_two_ranked_without_packs_has_no_grid(self):
        md = self.lb.render_markdown([_ok("a", 1, 1, 0), _ok("b", 2, 2, 1)], None)
        self.assertNotIn("능력 격자", md)

    def test_chain_broken_footer(self):
        md = self.lb.render_markdown([], "읽기 실패")
        self.assertIn("원장 체인: 파손", md)

    def test_pack_full_score_is_bold(self):
        results = [
            _ok("a", 2, 2, 0, packs={"core": {"score": 2, "max": 2}}),
            _ok("b", 1, 2, 1, packs={"core": {"score": 1, "max": 2}}),
        ]
        md = self.lb.render_markdown(results, None)
        self.assertIn("**2**", md)
        self.assertIn("1/2", md)

    def test_missing_pack_cell_is_dash(self):
        results = [
            _ok("a", 2, 2, 0, packs={"core": {"score": 2, "max": 2}, "sec": {"score": 1, "max": 1}}),
            _ok("b", 1, 1, 1, packs={"core": {"score": 1, "max": 1}}),
        ]
        md = self.lb.render_markdown(results, None)
        self.assertIn("—", md)

    def test_explicit_ranked_unverified_from_results(self):
        results = [_ok("a", 1, 1, 0), _bad(3)]
        md = self.lb.render_markdown(results, None, ranked=[results[0]])
        self.assertIn("**unverified**", md)
        self.assertIn("seq 3", md)

    def test_zero_max_pack_does_not_break_render(self):
        results = [
            _ok("a", 1, 1, 0, packs={"z": {"score": 0, "max": 0}, "r": {"score": 1, "max": 1}}),
            _ok("b", 1, 1, 1, packs={"r": {"score": 1, "max": 1}}),
        ]
        md = self.lb.render_markdown(results, None)
        self.assertIn("a", md)
        self.assertIn("b", md)


class ChainWalkMoreTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()

    def test_first_row_seq_must_be_zero(self):
        text = json.dumps({"seq": 1, "kind": "anchorLog", "prevEntryHash": None})
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write(text + "\n")
            path = fh.name
        entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertEqual(entries, [])
        self.assertIn("seq", err)

    def test_first_row_prev_must_be_null(self):
        text = json.dumps({"seq": 0, "kind": "anchorLog", "prevEntryHash": "aa" * 32})
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write(text + "\n")
            path = fh.name
        entries, err = self.lb.chain_walk(path, "anchorLog")
        Path(path).unlink()
        self.assertEqual(entries, [])
        self.assertIn("prevEntryHash", err)

    def test_three_valid_rows(self):
        text = make_chain("settlementLedger", [{"a": 1}, {"a": 2}, {"a": 3}])
        with tempfile.NamedTemporaryFile("w", suffix=".ndjson", delete=False,
                                         encoding="utf-8", newline="\n") as fh:
            fh.write(text)
            path = fh.name
        entries, err = self.lb.chain_walk(path, "settlementLedger")
        Path(path).unlink()
        self.assertIsNone(err)
        self.assertEqual(len(entries), 3)
        self.assertEqual(entries[2]["seq"], 2)


class InviteJoinContractTests(unittest.TestCase):
    def setUp(self):
        self.lb = load_lb()
        self._saved = {name: getattr(self.lb, name) for name in _BOARD_PATH_NAMES}

    def tearDown(self):
        for name, value in self._saved.items():
            setattr(self.lb, name, value)

    def test_join_mentions_guest_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            board = Path(tmp) / "leaderboard"
            board.mkdir()
            self.lb.BOARD = str(board)
            self.lb.KEYS = str(board / "keys")
            self.lb.CLAIMS = str(board / "claims")
            self.lb.LEDGER = str(board / "ledger.ndjson")
            self.lb.ANCHOR = str(board / "anchor.ndjson")
            self.lb.CHECKPOINT = str(board / "checkpoint.json")
            self.lb.KEYRING = str(board / "keyring.json")
            self.lb.WORKORDER = str(board / "workorder.json")
            invite = self.lb.construct_invite("한굴-손님")
            self.assertTrue(any("한굴-손님" in s for s in invite["join"]))
            self.assertIn("keys/", invite["promise"])
            self.assertIn("권한이 아니라", invite["note"])
            self.assertEqual(invite["board"]["repo"], "edwardkim/rhwp")

    def test_missing_invite_keys_listed(self):
        issues = self.lb.validate_invite({"schemaVersion": "1.0"})
        for key in ("kind", "guest", "board", "fingerprint", "join", "promise", "note"):
            self.assertTrue(any(key in i for i in issues), key)


if __name__ == "__main__":
    unittest.main()
