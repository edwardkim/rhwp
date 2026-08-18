"""[#5259] 릴리스 게이트 예외 경로 — 부재·판별 실패·표면/회귀 분리.

바이너리 없이 순수 함수와 목으로 고정한다. 이 파일이 지키는 것:

- 구 바이너리 부재는 skipped / pass. 신 바이너리 부재는 fail(1).
- 판별 감사 실패는 fail(1). regression(3) 이나 review(2) 로 위장하지 않는다.
- surface-changed 는 review. regression 은 block. 둘이 섞이면 표면이 이긴다.
- probe-failed / 보고 손상은 pass 가 아니다.
- 치명 예외는 삼키지 않는다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
GATE_RUNNER = REPO_ROOT / "gym/tools/release_gate.py"
GATE_WF = REPO_ROOT / ".github/workflows/gym-release-gate.yml"


def load_rg():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    spec = importlib.util.spec_from_file_location("gym_release_gate_exc", GATE_RUNNER)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _exists_new_only(path):
    text = str(path).replace("\\", "/")
    if "ledger.ndjson" in text:
        return False
    if "old" in text.lower() and "rhwp-old" not in text.lower():
        # 구 바이너리 후보만 숨기지 않는다 — 이 헬퍼는 신만 있다고 본다.
        pass
    return True


def _exists_none(path):
    return False


def _exists_both_no_ledger(path):
    text = str(path).replace("\\", "/")
    if "ledger.ndjson" in text:
        return False
    return True


class LoadMixin(unittest.TestCase):
    def setUp(self):
        self.rg = load_rg()


class CatalogTests(LoadMixin):
    def test_verdicts_and_exits_are_the_gate_contract(self):
        self.assertEqual(self.rg.VERDICTS, ("pass", "review", "block", "fail"))
        self.assertEqual(self.rg.EXIT_BY_VERDICT, {
            "pass": 0, "review": 2, "block": 3, "fail": 1,
        })

    def test_diff_classifications_do_not_include_probe_failed(self):
        self.assertEqual(self.rg.DIFF_CLASSIFICATIONS,
                         ("stable", "regression", "surface-changed"))
        self.assertNotIn("probe-failed", self.rg.DIFF_CLASSIFICATIONS)
        self.assertNotIn("skipped", self.rg.DIFF_CLASSIFICATIONS)

    def test_reasons_cover_the_issued_exception_paths(self):
        for key in ("missing-old-bin", "missing-new-bin", "discriminate-fail",
                    "surface-changed", "regression", "probe-failed"):
            self.assertIn(key, self.rg.REASONS, key)

    def test_verdict_by_reason_does_not_disguise(self):
        m = self.rg.VERDICT_BY_REASON
        self.assertEqual(m["missing-old-bin"], "pass")
        self.assertEqual(m["missing-new-bin"], "fail")
        self.assertEqual(m["discriminate-fail"], "fail")
        self.assertEqual(m["surface-changed"], "review")
        self.assertEqual(m["regression"], "block")
        self.assertEqual(m["probe-failed"], "fail")
        self.assertEqual(m["stable"], "pass")
        self.assertNotEqual(m["discriminate-fail"], "block")
        self.assertNotEqual(m["surface-changed"], "block")
        self.assertNotEqual(m["regression"], "review")

    def test_cli_flags_are_unchanged(self):
        args = self.rg.parse_args([])
        self.assertIsNone(args.old)
        self.assertIsNone(args.new)
        self.assertEqual(args.agent, "claude-fable-5")
        self.assertIsNone(args.pack)
        self.assertFalse(args.no_leaderboard)
        self.assertFalse(args.github_summary)
        self.assertIsNone(args.out)
        with self.assertRaises(SystemExit):
            self.rg.parse_args(["--discriminate-fail"])
        with self.assertRaises(SystemExit):
            self.rg.parse_args(["--preflight"])

    def test_kind_and_schema_stay_on_v1(self):
        self.assertEqual(self.rg.REPORT_KIND, "gymReleaseGate")
        self.assertEqual(self.rg.SCHEMA_VERSION, "1.0")

    def test_preflight_tools_match_workflow_scripts(self):
        wf = GATE_WF.read_text(encoding="utf-8")
        for script in self.rg.PREFLIGHT_TOOLS:
            self.assertIn(script, wf, script)


class ExceptionKindTests(LoadMixin):
    def test_fatal_exceptions_are_marked(self):
        self.assertTrue(self.rg.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(self.rg.is_fatal_exception(SystemExit(1)))
        self.assertTrue(self.rg.is_fatal_exception(MemoryError()))
        self.assertTrue(self.rg.is_fatal_exception(GeneratorExit()))
        self.assertFalse(self.rg.is_fatal_exception(FileNotFoundError("x")))
        self.assertFalse(self.rg.is_fatal_exception(ValueError("x")))

    def test_file_not_found_depends_on_context(self):
        exc = FileNotFoundError("nope")
        self.assertEqual(self.rg.exception_kind(exc, "bin"), "missing-bin")
        self.assertEqual(self.rg.exception_kind(exc, "diff-report"), "diff-report-missing")
        self.assertEqual(self.rg.exception_kind(exc, "gate"), "missing-bin")

    def test_catalog_types(self):
        cases = [
            (PermissionError("p"), "permission"),
            (TimeoutError("t"), "timeout"),
            (subprocess.TimeoutExpired(cmd="x", timeout=1), "timeout"),
            (UnicodeDecodeError("utf-8", b"\xff", 0, 1, "bad"), "decode-error"),
            (json.JSONDecodeError("bad", "x", 0), "invalid-json"),
            (TypeError("t"), "type-error"),
            (ValueError("v"), "value-error"),
            (OSError("o"), "os-error"),
            (RuntimeError("r"), "runtime-error"),
            (KeyError("k"), "key-error"),
            (IndexError("i"), "index-error"),
        ]
        for exc, kind in cases:
            self.assertEqual(self.rg.exception_kind(exc, "gate"), kind, kind)

    def test_none_and_unknown_are_unexpected(self):
        self.assertEqual(self.rg.exception_kind(None), "unexpected")
        self.assertEqual(self.rg.exception_kind(RuntimeWarning("w")), "unexpected")

    def test_truncate_head(self):
        self.assertEqual(self.rg.truncate_head(None), "")
        self.assertEqual(self.rg.truncate_head("ab", 0), "")
        self.assertEqual(self.rg.truncate_head("abcd", 3), "abc")
        self.assertEqual(self.rg.truncate_head("abcd", 10), "abcd")
        self.assertEqual(self.rg.truncate_head(12), "12")

    def test_exception_record_does_not_reraise(self):
        rec = self.rg.exception_record(FileNotFoundError("z"), context="bin", path="p")
        self.assertEqual(rec["kind"], "missing-bin")
        self.assertEqual(rec["error"], "FileNotFoundError")
        self.assertEqual(rec["path"], "p")
        self.assertIn("z", rec["head"])


class AuditReasonTests(LoadMixin):
    def test_discriminate_zero_is_ok(self):
        self.assertIsNone(self.rg.reason_for_audit("discriminate.py", 0))
        self.assertIsNone(self.rg.reason_for_audit("discriminate", 0))

    def test_discriminate_nonzero_is_discriminate_fail(self):
        self.assertEqual(self.rg.reason_for_audit("discriminate.py", 1), "discriminate-fail")
        self.assertEqual(self.rg.reason_for_audit("gym/tools/discriminate.py", 2),
                         "discriminate-fail")
        self.assertEqual(self.rg.reason_for_audit("DISCRIMINATE.PY", 1), "discriminate-fail")

    def test_discriminate_fail_is_not_regression(self):
        reason = self.rg.reason_for_audit("discriminate.py", 1)
        self.assertNotEqual(reason, "regression")
        self.assertNotEqual(reason, "surface-changed")
        self.assertEqual(self.rg.VERDICT_BY_REASON[reason], "fail")
        self.assertEqual(self.rg.EXIT_BY_VERDICT["fail"], 1)

    def test_trajectory_nonzero_is_trajectory_fail(self):
        self.assertEqual(self.rg.reason_for_audit("trajectory.py", 1), "trajectory-fail")
        self.assertIsNone(self.rg.reason_for_audit("trajectory.py", 0))

    def test_unknown_tool_nonzero_is_audit_fail(self):
        self.assertEqual(self.rg.reason_for_audit("other.py", 1), "audit-fail")

    def test_non_int_exit_is_audit_fail(self):
        self.assertEqual(self.rg.reason_for_audit("discriminate.py", "nope"), "audit-fail")
        self.assertEqual(self.rg.reason_for_audit("discriminate.py", None), "audit-fail")

    def test_normalize_tool_name(self):
        self.assertEqual(self.rg.normalize_tool_name("gym/tools/discriminate.py"),
                         "discriminate")
        self.assertEqual(self.rg.normalize_tool_name("discriminate.py"), "discriminate")
        self.assertEqual(self.rg.normalize_tool_name(""), "")
        self.assertEqual(self.rg.normalize_tool_name(None), "")


class FoldPreflightTests(LoadMixin):
    def test_none_and_empty_are_ok(self):
        for raw in (None, [], {}, {"audits": []}):
            folded = self.rg.fold_preflight(raw)
            self.assertTrue(folded["ok"], raw)
            self.assertFalse(folded["failed"], raw)
            self.assertEqual(folded["reasons"], [])

    def test_discriminate_row(self):
        folded = self.rg.fold_preflight({"tool": "discriminate.py", "exit": 1})
        self.assertFalse(folded["ok"])
        self.assertEqual(folded["reasons"], ["discriminate-fail"])
        self.assertEqual(folded["audits"][0]["tool"], "discriminate")

    def test_list_of_audits(self):
        folded = self.rg.fold_preflight([
            {"tool": "discriminate.py", "exit": 0},
            {"tool": "trajectory.py", "exit": 1},
        ])
        self.assertEqual(folded["reasons"], ["trajectory-fail"])
        self.assertFalse(folded["ok"])

    def test_ok_false_dict(self):
        folded = self.rg.fold_preflight({"ok": False, "reason": "discriminate-fail"})
        self.assertEqual(folded["reasons"], ["discriminate-fail"])
        self.assertTrue(folded["failed"])

    def test_garbage_input_is_audit_fail(self):
        folded = self.rg.fold_preflight("not-a-structure")
        self.assertTrue(folded["failed"])
        self.assertEqual(folded["reasons"], ["audit-fail"])

    def test_non_dict_row(self):
        folded = self.rg.fold_preflight(["x"])
        self.assertEqual(folded["reasons"], ["audit-fail"])


class MapDiffTests(LoadMixin):
    def test_three_way(self):
        self.assertEqual(self.rg.map_diff_classification("stable"), "stable")
        self.assertEqual(self.rg.map_diff_classification("regression"), "regression")
        self.assertEqual(self.rg.map_diff_classification("surface-changed"), "surface-changed")

    def test_skipped_and_probe_failed(self):
        self.assertEqual(self.rg.map_diff_classification("skipped"), "missing-old-bin")
        self.assertEqual(self.rg.map_diff_classification("probe-failed"), "probe-failed")

    def test_invalid_values(self):
        self.assertEqual(self.rg.map_diff_classification(None), "diff-report-invalid")
        self.assertEqual(self.rg.map_diff_classification(""), "diff-report-invalid")
        self.assertEqual(self.rg.map_diff_classification(3), "diff-report-invalid")
        self.assertEqual(self.rg.map_diff_classification("mystery"), "unexpected")

    def test_surface_wins_over_regression_flag(self):
        # 오라클이 실수로 regression + surfaceChanged 를 내도 게이트는 review.
        self.assertEqual(
            self.rg.surface_wins_over_regression("regression", True, 4),
            "surface-changed",
        )
        self.assertEqual(
            self.rg.surface_wins_over_regression("stable", True, 0),
            "surface-changed",
        )
        self.assertEqual(
            self.rg.surface_wins_over_regression("regression", False, 4),
            "regression",
        )
        self.assertEqual(
            self.rg.surface_wins_over_regression("surface-changed", False, 0),
            "surface-changed",
        )


class DecideVerdictTests(LoadMixin):
    """우선순위 표 — 이슈가 지목한 네 예외 경로가 여기서 갈린다."""

    CASES = (
        # diff_reason, board_ok, preflight, verdict, exit, reason
        ("stable", None, None, "pass", 0, "stable"),
        ("missing-old-bin", None, None, "pass", 0, "missing-old-bin"),
        ("skipped", None, None, "pass", 0, "skipped"),
        ("missing-new-bin", None, None, "fail", 1, "missing-new-bin"),
        ("discriminate-fail", None, None, "fail", 1, "discriminate-fail"),
        ("probe-failed", None, None, "fail", 1, "probe-failed"),
        ("diff-report-missing", None, None, "fail", 1, "diff-report-missing"),
        ("diff-report-invalid", None, None, "fail", 1, "diff-report-invalid"),
        ("surface-changed", None, None, "review", 2, "surface-changed"),
        ("regression", None, None, "block", 3, "regression"),
        ("stable", False, None, "block", 3, "leaderboard-broken"),
        ("surface-changed", False, None, "block", 3, "leaderboard-broken"),
        ("regression", False, None, "block", 3, "regression"),
        ("stable", None, ["discriminate-fail"], "fail", 1, "discriminate-fail"),
        ("regression", None, ["discriminate-fail"], "fail", 1, "discriminate-fail"),
        ("surface-changed", None, ["discriminate-fail"], "fail", 1, "discriminate-fail"),
        ("missing-new-bin", None, ["discriminate-fail"], "fail", 1, "discriminate-fail"),
        ("missing-old-bin", True, None, "pass", 0, "missing-old-bin"),
        ("stable", True, None, "pass", 0, "stable"),
    )

    def test_matrix(self):
        for diff, board, pre, verdict, exit_code, reason in self.CASES:
            with self.subTest(diff=diff, board=board, pre=pre):
                d = self.rg.decide_verdict(diff, board, pre)
                self.assertEqual(d["verdict"], verdict)
                self.assertEqual(d["exit"], exit_code)
                self.assertEqual(d["reason"], reason)
                self.assertEqual(d["ok"], verdict == "pass")
                self.assertEqual(d["reviewRequired"], verdict == "review")
                self.assertEqual(d["blocked"], verdict == "block")
                self.assertEqual(d["failed"], verdict == "fail")

    def test_surface_changed_is_never_block_without_board(self):
        d = self.rg.decide_verdict("surface-changed", None, None)
        self.assertEqual(d["verdict"], "review")
        self.assertNotEqual(d["verdict"], "block")
        self.assertEqual(d["exit"], 2)

    def test_regression_is_never_review(self):
        d = self.rg.decide_verdict("regression", None, None)
        self.assertEqual(d["verdict"], "block")
        self.assertNotEqual(d["verdict"], "review")
        self.assertEqual(d["exit"], 3)

    def test_discriminate_fail_is_never_regression_or_review(self):
        d = self.rg.decide_verdict("stable", None, ["discriminate-fail"])
        self.assertEqual(d["verdict"], "fail")
        self.assertEqual(d["exit"], 1)
        self.assertNotEqual(d["reason"], "regression")
        self.assertNotEqual(d["reason"], "surface-changed")

    def test_missing_old_is_not_missing_new(self):
        old = self.rg.decide_verdict("missing-old-bin", None, None)
        new = self.rg.decide_verdict("missing-new-bin", None, None)
        self.assertEqual(old["verdict"], "pass")
        self.assertEqual(new["verdict"], "fail")
        self.assertNotEqual(old["exit"], new["exit"])

    def test_empty_reasons_default_to_stable(self):
        d = self.rg.decide_verdict(None, None, None)
        self.assertEqual((d["verdict"], d["exit"]), ("pass", 0))


class ValidateVerdictTests(LoadMixin):
    def _base(self, **over):
        v = {
            "kind": "gymReleaseGate",
            "schemaVersion": "1.0",
            "diff": {"classification": "stable"},
            "leaderboard": {"ok": None},
            "verdict": "pass",
            "exit": 0,
            "reason": "stable",
            "ok": True,
            "reviewRequired": False,
            "blocked": False,
            "failed": False,
        }
        v.update(over)
        return v

    def test_clean_pass(self):
        self.assertEqual(self.rg.validate_verdict(self._base()), [])

    def test_not_dict(self):
        self.assertTrue(self.rg.validate_verdict("x"))

    def test_wrong_kind(self):
        issues = self.rg.validate_verdict(self._base(kind="nope"))
        self.assertTrue(any("kind" in i for i in issues))

    def test_pass_cannot_wear_regression(self):
        issues = self.rg.validate_verdict(self._base(reason="regression"))
        self.assertTrue(issues)

    def test_review_must_be_surface_changed(self):
        issues = self.rg.validate_verdict(self._base(
            verdict="review", exit=2, reason="regression",
            ok=False, reviewRequired=True, blocked=False, failed=False,
        ))
        self.assertTrue(any("review" in i for i in issues))

    def test_block_cannot_be_surface_changed(self):
        issues = self.rg.validate_verdict(self._base(
            verdict="block", exit=3, reason="surface-changed",
            ok=False, reviewRequired=False, blocked=True, failed=False,
        ))
        self.assertTrue(issues)

    def test_fail_cannot_wear_regression(self):
        issues = self.rg.validate_verdict(self._base(
            verdict="fail", exit=1, reason="regression",
            ok=False, reviewRequired=False, blocked=False, failed=True,
        ))
        self.assertTrue(any("regression" in i for i in issues))

    def test_probe_failed_cannot_pass(self):
        issues = self.rg.validate_verdict(self._base(
            diff={"classification": "probe-failed"},
        ))
        self.assertTrue(any("probe-failed" in i for i in issues))

    def test_exit_mismatch(self):
        issues = self.rg.validate_verdict(self._base(exit=3))
        self.assertTrue(any("exit" in i for i in issues))

    def test_flags_must_match_verdict(self):
        issues = self.rg.validate_verdict(self._base(ok=False))
        self.assertTrue(any("ok" in i for i in issues))


class MissingBinGateTests(LoadMixin):
    def test_missing_new_fails(self):
        with mock.patch("os.path.exists", return_value=False):
            v = self.rg.gate(None, "missing-new", "agent", None, verify_board=False)
        self.assertEqual(v["verdict"], "fail")
        self.assertEqual(v["exit"], 1)
        self.assertEqual(v["reason"], "missing-new-bin")
        self.assertNotEqual(v["verdict"], "pass")
        self.assertNotEqual(v["verdict"], "block")

    def test_missing_new_does_not_run_diff(self):
        called = []

        def fake(script, args):
            called.append(script)
            return (0, "")

        with mock.patch("os.path.exists", return_value=False), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            self.rg.gate("old", "new", "agent", None, verify_board=False)
        self.assertEqual(called, [])

    def test_missing_old_with_present_new_passes(self):
        def exists(path):
            text = str(path).replace("\\", "/")
            return "ledger.ndjson" not in text

        with mock.patch("os.path.exists", side_effect=exists):
            v = self.rg.gate(None, "new", "agent", None, verify_board=False)
        self.assertEqual(v["verdict"], "pass")
        self.assertEqual(v["diff"]["classification"], "skipped")
        self.assertEqual(v["reason"], "missing-old-bin")

    def test_empty_new_is_missing(self):
        with mock.patch("os.path.exists", return_value=True):
            v = self.rg.gate(None, "", "agent", None, verify_board=False)
        self.assertEqual(v["reason"], "missing-new-bin")
        self.assertEqual(v["exit"], 1)

    def test_none_new_is_missing_when_find_bin_yields_absent(self):
        with mock.patch("os.path.exists", return_value=False):
            v = self.rg.gate(None, None, "agent", None, verify_board=False)
        self.assertEqual(v["reason"], "missing-new-bin")

    def test_missing_old_given_path_skips(self):
        def exists(path):
            text = str(path).replace("\\", "/")
            if "ledger.ndjson" in text:
                return False
            # 구 경로만 없다. 신 후보는 있다.
            if text.endswith("/old") or text.endswith("\\old") or text.endswith("old-bin"):
                return False
            if text.replace("\\", "/").endswith("old"):
                return False
            return True

        with mock.patch.object(self.rg.runner, "find_bin", side_effect=lambda p: p), \
                mock.patch("os.path.exists", side_effect=exists):
            v = self.rg.gate("old-bin", "new", "agent", None, verify_board=False)
        self.assertEqual(v["diff"]["classification"], "skipped")
        self.assertEqual(v["verdict"], "pass")


class DiscriminateFailGateTests(LoadMixin):
    def _present(self, path):
        return "ledger.ndjson" not in str(path).replace("\\", "/")

    def test_discriminate_fail_is_fail_not_block(self):
        with mock.patch("os.path.exists", side_effect=self._present):
            v = self.rg.gate(
                None, "new", "agent", None, verify_board=False,
                preflight={"tool": "discriminate.py", "exit": 1},
            )
        self.assertEqual(v["verdict"], "fail")
        self.assertEqual(v["exit"], 1)
        self.assertEqual(v["reason"], "discriminate-fail")
        self.assertNotEqual(v["verdict"], "block")
        self.assertNotEqual(v["verdict"], "review")
        self.assertNotEqual(v["verdict"], "pass")

    def test_discriminate_ok_does_not_change_stable(self):
        with mock.patch("os.path.exists", side_effect=self._present):
            v = self.rg.gate(
                None, "new", "agent", None, verify_board=False,
                preflight={"tool": "discriminate.py", "exit": 0},
            )
        self.assertEqual(v["verdict"], "pass")

    def test_discriminate_fail_beats_regression(self):
        def fake(script, args):
            out = args[args.index("-o") + 1]
            with io.open(out, "w", encoding="utf-8") as fh:
                fh.write(json.dumps({
                    "classification": "regression",
                    "divergences": 4,
                    "surfaceChanged": False,
                    "tasksCompared": 10,
                }))
            return (3, "")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            v = self.rg.gate(
                "old", "new", "agent", None, verify_board=False,
                preflight={"tool": "discriminate.py", "exit": 1},
            )
        self.assertEqual(v["reason"], "discriminate-fail")
        self.assertEqual(v["exit"], 1)

    def test_discriminate_fail_beats_surface_changed(self):
        with mock.patch("os.path.exists", side_effect=self._present):
            v = self.rg.gate(
                None, "new", "agent", None, verify_board=False,
                preflight={"tool": "discriminate.py", "exit": 1},
            )
        self.assertEqual(v["reason"], "discriminate-fail")
        self.assertEqual(v["exit"], 1)

    def test_preflight_is_not_a_cli_flag(self):
        with self.assertRaises(SystemExit):
            self.rg.parse_args(["--preflight", "discriminate"])


class SurfaceVsRegressionGateTests(LoadMixin):
    def _gate(self, classification, divergences, surface, board=False, board_exit=0):
        def fake(script, args):
            if script == "leaderboard.py":
                return (board_exit, "board")
            out = args[args.index("-o") + 1]
            with io.open(out, "w", encoding="utf-8") as fh:
                fh.write(json.dumps({
                    "classification": classification,
                    "divergences": divergences,
                    "surfaceChanged": surface,
                    "tasksCompared": 91,
                }))
            return (0, "")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            return self.rg.gate("old", "new", "agent", ["core-cli"], verify_board=board)

    def test_stable(self):
        v = self._gate("stable", 0, False)
        self.assertEqual((v["verdict"], v["exit"], v["reason"]), ("pass", 0, "stable"))
        self.assertEqual(self.rg.validate_verdict(v), [])

    def test_regression(self):
        v = self._gate("regression", 4, False)
        self.assertEqual((v["verdict"], v["exit"], v["reason"]), ("block", 3, "regression"))
        self.assertEqual(self.rg.validate_verdict(v), [])

    def test_surface_changed_zero_div(self):
        v = self._gate("surface-changed", 0, True)
        self.assertEqual((v["verdict"], v["exit"]), ("review", 2))
        self.assertEqual(v["reason"], "surface-changed")

    def test_surface_changed_with_divergences_still_review(self):
        v = self._gate("surface-changed", 70, True)
        self.assertEqual(v["verdict"], "review")
        self.assertNotEqual(v["verdict"], "block")
        self.assertEqual(v["exit"], 2)

    def test_mislabelled_regression_with_surface_flag_is_review(self):
        v = self._gate("regression", 4, True)
        self.assertEqual(v["verdict"], "review")
        self.assertEqual(v["reason"], "surface-changed")

    def test_probe_failed_is_fail(self):
        v = self._gate("probe-failed", 0, False)
        self.assertEqual(v["verdict"], "fail")
        self.assertEqual(v["exit"], 1)
        self.assertEqual(v["reason"], "probe-failed")
        self.assertNotEqual(v["verdict"], "pass")

    def test_surface_plus_broken_board_blocks(self):
        v = self._gate("surface-changed", 0, True, board=True, board_exit=3)
        self.assertEqual(v["verdict"], "block")
        self.assertEqual(v["reason"], "leaderboard-broken")

    def test_regression_plus_broken_board_still_blocks(self):
        v = self._gate("regression", 2, False, board=True, board_exit=3)
        self.assertEqual(v["verdict"], "block")
        self.assertIn(v["reason"], ("regression", "leaderboard-broken"))


class DiffReportExceptionTests(LoadMixin):
    def test_missing_report_is_fail(self):
        def fake(script, args):
            return (1, "no report")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake), \
                mock.patch.object(self.rg, "load_json_safe",
                                  return_value=(None, {"kind": "diff-report-missing"})):
            v = self.rg.gate("old", "new", "agent", None, verify_board=False)
        self.assertEqual(v["verdict"], "fail")
        self.assertIn(v["reason"], ("diff-report-missing", "diff-tool-error"))

    def test_invalid_json_is_fail(self):
        def fake(script, args):
            out = args[args.index("-o") + 1]
            with io.open(out, "w", encoding="utf-8") as fh:
                fh.write("{not json")
            return (0, "")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            v = self.rg.gate("old", "new", "agent", None, verify_board=False)
        self.assertEqual(v["verdict"], "fail")
        self.assertEqual(v["reason"], "diff-report-invalid")

    def test_report_without_classification_is_fail(self):
        def fake(script, args):
            out = args[args.index("-o") + 1]
            with io.open(out, "w", encoding="utf-8") as fh:
                fh.write(json.dumps({"divergences": 0}))
            return (0, "")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            v = self.rg.gate("old", "new", "agent", None, verify_board=False)
        self.assertEqual(v["reason"], "diff-report-invalid")
        self.assertEqual(v["exit"], 1)

    def test_run_tool_oserror_is_fail(self):
        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=OSError("boom")):
            v = self.rg.gate("old", "new", "agent", None, verify_board=False)
        self.assertEqual(v["verdict"], "fail")
        self.assertEqual(v["reason"], "diff-tool-error")

    def test_keyboard_interrupt_is_not_swallowed(self):
        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                self.rg.gate("old", "new", "agent", None, verify_board=False)

    def test_system_exit_is_not_swallowed(self):
        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=SystemExit(2)):
            with self.assertRaises(SystemExit):
                self.rg.gate("old", "new", "agent", None, verify_board=False)


class LeaderboardExceptionTests(LoadMixin):
    def test_leaderboard_oserror_is_fail(self):
        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=OSError("dead")):
            v = self.rg.gate(None, "new", "agent", None, verify_board=True)
        # old omitted → skip diff. leaderboard throws → fail, not pass.
        self.assertEqual(v["verdict"], "fail")
        self.assertEqual(v["reason"], "leaderboard-error")

    def test_leaderboard_nonzero_is_block(self):
        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", return_value=(3, "broken")):
            v = self.rg.gate(None, "new", "agent", None, verify_board=True)
        self.assertEqual(v["exit"], 3)
        self.assertEqual(v["reason"], "leaderboard-broken")

    def test_verify_board_false_skips_even_if_ledger_exists(self):
        called = []

        def fake(script, args):
            called.append(script)
            return (0, "")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            v = self.rg.gate(None, "new", "agent", None, verify_board=False)
        self.assertEqual(called, [])
        self.assertEqual(v["verdict"], "pass")


class SummaryAndWriteTests(LoadMixin):
    def test_render_summary_mentions_surface_honesty(self):
        v = {
            "verdict": "review", "exit": 2, "reason": "surface-changed",
            "diff": {"classification": "surface-changed", "divergences": 2},
            "leaderboard": {"ok": None, "reason": "생략"},
            "old": {"status": "present"}, "new": {"status": "present"},
            "preflight": {"audits": []},
        }
        text = "\n".join(self.rg.render_summary_lines(v))
        self.assertIn("리뷰 신호", text)
        self.assertIn("surface-changed", text)

    def test_render_summary_mentions_discriminate(self):
        v = {
            "verdict": "fail", "exit": 1, "reason": "discriminate-fail",
            "diff": {"classification": "unavailable"},
            "leaderboard": {"ok": None},
            "old": {"status": "omitted"}, "new": {"status": "present"},
            "preflight": {"audits": [{"tool": "discriminate", "ok": False,
                                      "reason": "discriminate-fail"}]},
        }
        text = "\n".join(self.rg.render_summary_lines(v))
        self.assertIn("약한 오라클", text)
        self.assertIn("discriminate", text)

    def test_render_summary_mentions_missing_new(self):
        v = {
            "verdict": "fail", "exit": 1, "reason": "missing-new-bin",
            "diff": {"classification": "unavailable"},
            "leaderboard": {"ok": None},
            "old": {"status": "omitted"}, "new": {"status": "missing"},
            "preflight": {},
        }
        text = "\n".join(self.rg.render_summary_lines(v))
        self.assertIn("신 바이너리 부재", text)

    def test_write_verdict_roundtrip(self):
        payload = {"kind": "gymReleaseGate", "verdict": "pass", "exit": 0}
        fd, path = tempfile.mkstemp(suffix=".json")
        os.close(fd)
        try:
            err = self.rg.write_verdict_safe(payload, path)
            self.assertIsNone(err)
            raw = Path(path).read_bytes()
            self.assertFalse(raw.startswith(b"\xef\xbb\xbf"))
            self.assertNotIn(b"\r\n", raw)
            back = json.loads(raw.decode("utf-8"))
            self.assertEqual(back["verdict"], "pass")
        finally:
            os.remove(path)

    def test_write_verdict_oserror(self):
        err = self.rg.write_verdict_safe({"a": 1}, os.path.join("no-such-dir", "x.json"))
        self.assertIsNotNone(err)
        self.assertEqual(err["context"], "write")

    def test_github_summary_noop_without_env(self):
        with mock.patch.dict(os.environ, {}, clear=False):
            os.environ.pop("GITHUB_STEP_SUMMARY", None)
            self.assertIsNone(self.rg.write_github_summary({
                "verdict": "pass", "exit": 0, "diff": {}, "leaderboard": {},
            }))

    def test_github_summary_appends(self):
        fd, path = tempfile.mkstemp(suffix=".md")
        os.close(fd)
        try:
            Path(path).write_text("head\n", encoding="utf-8")
            with mock.patch.dict(os.environ, {"GITHUB_STEP_SUMMARY": path}):
                self.rg.write_github_summary({
                    "verdict": "review", "exit": 2, "reason": "surface-changed",
                    "diff": {"classification": "surface-changed"},
                    "leaderboard": {"ok": True},
                    "old": {"status": "present"}, "new": {"status": "present"},
                    "preflight": {},
                })
            text = Path(path).read_text(encoding="utf-8")
            self.assertIn("head", text)
            self.assertIn("운동장 릴리스 게이트", text)
            self.assertIn("review", text)
        finally:
            os.remove(path)


class MainEntryTests(LoadMixin):
    def test_main_missing_new_exits_1(self):
        with mock.patch("os.path.exists", return_value=False):
            code = self.rg.main(["--new", "no-such-bin", "--no-leaderboard"])
        self.assertEqual(code, 1)

    def test_main_writes_out(self):
        fd, path = tempfile.mkstemp(suffix=".json")
        os.close(fd)
        try:
            def exists(p):
                return "ledger.ndjson" not in str(p).replace("\\", "/")

            with mock.patch("os.path.exists", side_effect=exists):
                code = self.rg.main(["--new", "new", "--no-leaderboard", "-o", path])
            self.assertEqual(code, 0)
            data = json.loads(Path(path).read_text(encoding="utf-8"))
            self.assertEqual(data["kind"], "gymReleaseGate")
            self.assertEqual(data["verdict"], "pass")
        finally:
            os.remove(path)

    def test_main_no_new_cli_flags(self):
        with self.assertRaises(SystemExit):
            self.rg.main(["--help-discriminate"])


class ExtractDiffFieldTests(LoadMixin):
    def test_happy(self):
        fields, err = self.rg.extract_diff_fields({
            "classification": "stable",
            "divergences": 0,
            "surfaceChanged": False,
            "tasksCompared": 3,
        })
        self.assertIsNone(err)
        self.assertEqual(fields["classification"], "stable")

    def test_not_dict(self):
        fields, err = self.rg.extract_diff_fields([1])
        self.assertIsNone(fields)
        self.assertEqual(err["kind"], "diff-report-invalid")

    def test_missing_classification(self):
        fields, err = self.rg.extract_diff_fields({"divergences": 0})
        self.assertIsNotNone(err)


class FindBinSafeTests(LoadMixin):
    def test_delegates(self):
        with mock.patch.object(self.rg.runner, "find_bin", return_value="/abs/rhwp"):
            found, err = self.rg.find_bin_safe("rhwp")
        self.assertEqual(found, "/abs/rhwp")
        self.assertIsNone(err)

    def test_oserror(self):
        with mock.patch.object(self.rg.runner, "find_bin", side_effect=OSError("x")):
            found, err = self.rg.find_bin_safe("rhwp")
        self.assertIsNotNone(err)
        self.assertEqual(err["kind"], "os-error")

    def test_keyboard_interrupt(self):
        with mock.patch.object(self.rg.runner, "find_bin", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                self.rg.find_bin_safe("rhwp")


class HonestyMatrixGeneratedTests(LoadMixin):
    """표면 × 분기 × 전제 × 원장 의 생성 표. 위장 조합이 없는지 전수."""

    def test_no_disguise_in_generated_table(self):
        surfaces = (False, True)
        divergences = (0, 3)
        pres = (None, ["discriminate-fail"])
        boards = (None, True, False)
        for surface in surfaces:
            for div in divergences:
                for pre in pres:
                    for board in boards:
                        if surface:
                            cls = "surface-changed"
                        elif div:
                            cls = "regression"
                        else:
                            cls = "stable"
                        reason = self.rg.surface_wins_over_regression(cls, surface, div)
                        d = self.rg.decide_verdict(reason, board if board is not True else None,
                                                   pre)
                        if pre:
                            self.assertEqual(d["verdict"], "fail", (cls, pre, board))
                            self.assertEqual(d["exit"], 1)
                            continue
                        if board is False:
                            self.assertEqual(d["verdict"], "block", (cls, board))
                            continue
                        if surface:
                            self.assertEqual(d["verdict"], "review", (cls, div))
                            self.assertEqual(d["exit"], 2)
                        elif div:
                            self.assertEqual(d["verdict"], "block")
                            self.assertEqual(d["exit"], 3)
                        else:
                            self.assertEqual(d["verdict"], "pass")
                            self.assertEqual(d["exit"], 0)


class ResolveBinRecordTests(LoadMixin):
    def test_none_is_omitted(self):
        rec = self.rg.resolve_bin_record(None, "old")
        self.assertEqual(rec["status"], "omitted")
        self.assertEqual(rec["role"], "old")

    def test_blank_is_omitted(self):
        rec = self.rg.resolve_bin_record("   ", "new")
        self.assertEqual(rec["status"], "omitted")

    def test_present_when_exists(self):
        with mock.patch.object(self.rg.runner, "find_bin", return_value="/abs/rhwp"), \
                mock.patch("os.path.exists", return_value=True):
            rec = self.rg.resolve_bin_record("rhwp", "new")
        self.assertEqual(rec["status"], "present")
        self.assertEqual(rec["resolved"], "/abs/rhwp")

    def test_missing_when_not_exists(self):
        with mock.patch.object(self.rg.runner, "find_bin", return_value="/no/rhwp"), \
                mock.patch("os.path.exists", return_value=False):
            rec = self.rg.resolve_bin_record("rhwp", "new")
        self.assertEqual(rec["status"], "missing")
        self.assertEqual(rec["reason"], "not-found")

    def test_find_bin_error(self):
        with mock.patch.object(self.rg.runner, "find_bin", side_effect=OSError("x")):
            rec = self.rg.resolve_bin_record("rhwp", "new")
        self.assertEqual(rec["status"], "error")


class ValidateDisguiseTableTests(LoadMixin):
    """문서 22절의 허용/거절 표를 기계로 다시 읽는다."""

    ALLOWED = (
        ("pass", "stable", 0),
        ("pass", "missing-old-bin", 0),
        ("pass", "skipped", 0),
        ("review", "surface-changed", 2),
        ("block", "regression", 3),
        ("block", "leaderboard-broken", 3),
        ("fail", "missing-new-bin", 1),
        ("fail", "discriminate-fail", 1),
        ("fail", "probe-failed", 1),
        ("fail", "diff-report-missing", 1),
        ("fail", "diff-report-invalid", 1),
    )
    REJECTED = (
        ("pass", "regression", 0),
        ("pass", "surface-changed", 0),
        ("pass", "discriminate-fail", 0),
        ("pass", "missing-new-bin", 0),
        ("pass", "probe-failed", 0),
        ("review", "regression", 2),
        ("block", "surface-changed", 3),
        ("block", "discriminate-fail", 3),
        ("fail", "regression", 1),
        ("fail", "surface-changed", 1),
    )

    def _envelope(self, verdict, reason, exit_code):
        return {
            "kind": "gymReleaseGate",
            "schemaVersion": "1.0",
            "diff": {"classification": reason if reason != "leaderboard-broken" else "skipped"},
            "leaderboard": {"ok": None},
            "verdict": verdict,
            "exit": exit_code,
            "reason": reason,
            "ok": verdict == "pass",
            "reviewRequired": verdict == "review",
            "blocked": verdict == "block",
            "failed": verdict == "fail",
        }

    def test_allowed_rows_are_clean(self):
        for verdict, reason, exit_code in self.ALLOWED:
            issues = self.rg.validate_verdict(self._envelope(verdict, reason, exit_code))
            self.assertEqual(issues, [], (verdict, reason, issues))

    def test_rejected_rows_are_dirty(self):
        for verdict, reason, exit_code in self.REJECTED:
            issues = self.rg.validate_verdict(self._envelope(verdict, reason, exit_code))
            self.assertTrue(issues, (verdict, reason))


class PacksForwardedTests(LoadMixin):
    def test_pack_flags_are_forwarded_to_release_diff(self):
        seen = []

        def fake(script, args):
            seen.append((script, list(args)))
            out = args[args.index("-o") + 1]
            with io.open(out, "w", encoding="utf-8") as fh:
                fh.write(json.dumps({
                    "classification": "stable",
                    "divergences": 0,
                    "surfaceChanged": False,
                    "tasksCompared": 1,
                }))
            return (0, "")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            v = self.rg.gate("old", "new", "agent", ["core-cli", "security"],
                             verify_board=False)
        self.assertEqual(v["verdict"], "pass")
        _script, args = seen[0]
        self.assertIn("--pack", args)
        self.assertIn("core-cli", args)
        self.assertIn("security", args)
        self.assertNotIn("automation", args)


if __name__ == "__main__":
    unittest.main()
