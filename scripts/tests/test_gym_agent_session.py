"""[agent_session] gym 에이전트 세션 트레이스 계약 — 경로(명령 열) 채점.

핵심: 선언된 세션과 JSONL 트레이스를 명령 계열(argv[0])·종료 코드·순서로
대조한다. 종점 산출이 아니라 밟은 경로를 잰다. trajectory.py 의 마지막 스텝
절단과 겹치지 않는다. 픽스처만으로 재생 채점하며 바이너리는 쓰지 않는다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import os
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "agent_session.py"

# 픽스처 트레이스 — 선언 세션 inspect-then-export (info → export-text)
PASS_TRACE = """\
{"ts":"2026-08-18T00:00:00Z","argv":["info","samples/x.hwp","--json"],"exit":0,"stdoutSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","ok":true}
{"ts":"2026-08-18T00:00:01Z","argv":["export-text","samples/x.hwp","-o","work/out.txt"],"exit":0,"ok":true}
"""

WRONG_COMMAND_TRACE = """\
{"ts":"2026-08-18T00:00:00Z","argv":["search","samples/x.hwp","--json"],"exit":0,"ok":true}
{"ts":"2026-08-18T00:00:01Z","argv":["export-text","samples/x.hwp","-o","work/out.txt"],"exit":0,"ok":true}
"""

WRONG_ORDER_TRACE = """\
{"ts":"2026-08-18T00:00:00Z","argv":["export-text","samples/x.hwp","-o","work/out.txt"],"exit":0,"ok":true}
{"ts":"2026-08-18T00:00:01Z","argv":["info","samples/x.hwp","--json"],"exit":0,"ok":true}
"""

EXTRA_STEP_TRACE = """\
{"ts":"2026-08-18T00:00:00Z","argv":["info","samples/x.hwp","--json"],"exit":0,"ok":true}
{"ts":"2026-08-18T00:00:01Z","argv":["export-text","samples/x.hwp","-o","work/out.txt"],"exit":0,"ok":true}
{"ts":"2026-08-18T00:00:02Z","argv":["digest","samples/x.hwp","--json"],"exit":0,"ok":true}
"""

MISSING_STEP_TRACE = """\
{"ts":"2026-08-18T00:00:00Z","argv":["info","samples/x.hwp","--json"],"exit":0,"ok":true}
"""

WRONG_EXIT_TRACE = """\
{"ts":"2026-08-18T00:00:00Z","argv":["info","samples/x.hwp","--json"],"exit":2,"ok":false}
{"ts":"2026-08-18T00:00:01Z","argv":["export-text","samples/x.hwp","-o","work/out.txt"],"exit":0,"ok":true}
"""


def load():
    spec = importlib.util.spec_from_file_location("gym_agent_session", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def session_doc():
    return {
        "id": "inspect-then-export",
        "input": "samples/x.hwp",
        "subDir": "work",
        "steps": [
            {"run": ["info", "{input}", "--json"], "expectExit": 0},
            {
                "run": ["export-text", "{input}", "-o", "{sub:out.txt}"],
                "expectExit": 0,
                "expectPath": "{sub:out.txt}",
            },
        ],
    }


def events_of(text):
    return load().parse_trace_jsonl(text)


def reasons_of(report):
    return [m["reason"] for m in report.get("mismatches", [])]


class ResolveTests(unittest.TestCase):
    def test_input_and_sub_placeholders(self):
        mod = load()
        ctx = mod.SessionContext("samples/x.hwp", os.path.join("work", "sub"))
        self.assertEqual(mod.resolve_token("{input}", ctx), "samples/x.hwp")
        resolved = mod.resolve_token("{sub:out.txt}", ctx)
        self.assertEqual(resolved, os.path.join("work", "sub", "out.txt"))

    def test_embedded_sub_and_unknown_passthrough(self):
        mod = load()
        ctx = mod.SessionContext("in.hwp", "w")
        self.assertEqual(
            mod.resolve_token("pre-{sub:a.txt}-post", ctx),
            "pre-" + os.path.join("w", "a.txt") + "-post",
        )
        self.assertEqual(mod.resolve_token("{nope}", ctx), "{nope}")
        self.assertEqual(mod.resolve_token("literal", ctx), "literal")

    def test_resolve_argv_keeps_family(self):
        mod = load()
        ctx = mod.SessionContext.from_session(session_doc())
        argv = mod.resolve_argv(session_doc()["steps"][0]["run"], ctx)
        self.assertEqual(argv[0], "info")
        self.assertEqual(argv[1], "samples/x.hwp")
        self.assertEqual(mod.command_family(argv), "info")

    def test_missing_context_leaves_placeholder(self):
        mod = load()
        self.assertEqual(mod.resolve_token("{input}", None), "{input}")
        self.assertEqual(mod.resolve_token("{sub:a}", mod.SessionContext()), "{sub:a}")


class ValidateSessionTests(unittest.TestCase):
    def test_good_session_has_no_issues(self):
        self.assertEqual(load().validate_session(session_doc()), [])

    def test_missing_id_and_empty_steps(self):
        mod = load()
        self.assertTrue(any("id" in i for i in mod.validate_session({"steps": [{"run": ["info"]}]})))
        self.assertTrue(any("steps" in i for i in mod.validate_session({"id": "x", "steps": []})))
        self.assertTrue(any("객체" in i for i in mod.validate_session([])))

    def test_bad_run_and_expect_fields(self):
        mod = load()
        bad = {
            "id": "x",
            "steps": [
                {"run": []},
                {"run": ["info"], "expectExit": "zero", "expectPath": ""},
                {"run": [""]},
                {"run": ["info", "{mystery}"]},
            ],
        }
        issues = " ".join(mod.validate_session(bad))
        self.assertIn("run", issues)
        self.assertIn("expectExit", issues)
        self.assertIn("expectPath", issues)
        self.assertIn("자리표", issues)

    def test_unbalanced_placeholder_is_flagged(self):
        issues = load().validate_session({
            "id": "x",
            "steps": [{"run": ["info", "{input"]}],
        })
        self.assertTrue(any("중괄호" in i or "자리표" in i for i in issues))


class ParseTraceTests(unittest.TestCase):
    def test_parses_fixture_and_skips_blank_lines(self):
        ev = events_of(PASS_TRACE + "\n\n")
        self.assertEqual(len(ev), 2)
        self.assertEqual(ev[0]["argv"][0], "info")
        self.assertEqual(ev[1]["exit"], 0)

    def test_rejects_empty_and_bad_json(self):
        mod = load()
        with self.assertRaises(mod.SessionError):
            mod.parse_trace_jsonl("")
        with self.assertRaises(mod.SessionError):
            mod.parse_trace_jsonl("not-json\n")

    def test_event_schema_requires_ts_argv_exit_ok(self):
        mod = load()
        issues = mod.validate_trace_event({"argv": ["info"], "exit": 0, "ok": True}, 0)
        self.assertTrue(any(".ts" in i for i in issues))
        issues = mod.validate_trace_event({
            "ts": "t", "argv": ["info"], "exit": 0, "ok": True,
            "stdoutSha256": "zzzz",
        }, 3)
        self.assertTrue(any("stdoutSha256" in i for i in issues))


class ScorePassTests(unittest.TestCase):
    def test_matching_trace_passes(self):
        mod = load()
        report = mod.score_session(session_doc(), events_of(PASS_TRACE))
        self.assertEqual(report["kind"], "gymAgentSession")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertTrue(report["ok"])
        self.assertTrue(report["orderOk"])
        self.assertEqual(report["declared"], 2)
        self.assertEqual(report["observed"], 2)
        self.assertEqual(report["matched"], 2)
        self.assertEqual(report["extraSteps"], [])
        self.assertEqual(report["missingSteps"], [])
        self.assertEqual(report["mismatches"], [])
        self.assertEqual(report["steps"][0]["declaredFamily"], "info")
        self.assertEqual(report["steps"][1]["declaredFamily"], "export-text")

    def test_build_event_hashes_stdout_and_sets_ok(self):
        mod = load()
        ev = mod.build_trace_event(["info"], 0, stdout="hello", expect_exit=0, ts="t")
        self.assertEqual(ev["ts"], "t")
        self.assertTrue(ev["ok"])
        self.assertEqual(ev["stdoutSha256"], mod.sha256_text("hello"))
        ev_bad = mod.build_trace_event(["info"], 1, expect_exit=0, ts="t")
        self.assertFalse(ev_bad["ok"])
        self.assertNotIn("stdoutSha256", ev_bad)


class ScoreWrongCommandTests(unittest.TestCase):
    def test_wrong_family_is_wrong_command(self):
        report = load().score_session(session_doc(), events_of(WRONG_COMMAND_TRACE))
        self.assertFalse(report["ok"])
        self.assertIn("wrongCommand", reasons_of(report))
        self.assertFalse(report["steps"][0]["familyOk"])
        self.assertEqual(report["steps"][0]["observedFamily"], "search")
        self.assertEqual(report["steps"][0]["declaredFamily"], "info")


class ScoreWrongOrderTests(unittest.TestCase):
    def test_swapped_steps_are_wrong_order(self):
        report = load().score_session(session_doc(), events_of(WRONG_ORDER_TRACE))
        self.assertFalse(report["ok"])
        self.assertFalse(report["orderOk"])
        self.assertIn("wrongOrder", reasons_of(report))
        self.assertNotIn("wrongCommand", reasons_of(report))
        self.assertEqual(report["steps"][0]["observedFamily"], "export-text")
        self.assertEqual(report["steps"][1]["observedFamily"], "info")


class ScoreExtraStepTests(unittest.TestCase):
    def test_trailing_extra_step(self):
        report = load().score_session(session_doc(), events_of(EXTRA_STEP_TRACE))
        self.assertFalse(report["ok"])
        self.assertIn("extraStep", reasons_of(report))
        self.assertEqual(len(report["extraSteps"]), 1)
        self.assertEqual(report["extraSteps"][0]["family"], "digest")
        self.assertEqual(report["declared"], 2)
        self.assertEqual(report["observed"], 3)
        # 선언 두 스텝은 그대로 일치한다.
        self.assertTrue(report["steps"][0]["familyOk"])
        self.assertTrue(report["steps"][1]["familyOk"])


class ScoreMissingAndExitTests(unittest.TestCase):
    def test_missing_step(self):
        report = load().score_session(session_doc(), events_of(MISSING_STEP_TRACE))
        self.assertFalse(report["ok"])
        self.assertIn("missingStep", reasons_of(report))
        self.assertEqual(len(report["missingSteps"]), 1)
        self.assertEqual(report["missingSteps"][0]["family"], "export-text")

    def test_wrong_exit_same_family(self):
        report = load().score_session(session_doc(), events_of(WRONG_EXIT_TRACE))
        self.assertFalse(report["ok"])
        self.assertIn("wrongExit", reasons_of(report))
        self.assertTrue(report["steps"][0]["familyOk"])
        self.assertFalse(report["steps"][0]["exitOk"])
        self.assertEqual(report["steps"][0]["observedExit"], 2)

    def test_invalid_session_scores_as_bad_session(self):
        report = load().score_session({"id": ""}, [])
        self.assertFalse(report["ok"])
        self.assertIn("badSession", reasons_of(report))


class ClassifySequenceTests(unittest.TestCase):
    def test_prefix_extra_and_middle_insert(self):
        mod = load()
        self.assertEqual(mod.classify_sequence(["a", "b"], ["a", "b", "c"]), ["extraStep"])
        self.assertEqual(mod.classify_sequence(["a", "b"], ["a"]), ["missingStep"])
        self.assertEqual(mod.classify_sequence(["a", "b"], ["b", "a"]), ["wrongOrder"])
        self.assertEqual(mod.classify_sequence(["a", "b"], ["a", "x"]), ["wrongCommand"])

    def test_lcs_collapses_adjacent_replace(self):
        mod = load()
        ops = mod.collapse_ops(mod.lcs_ops(["info", "export-text"], ["search", "export-text"]))
        kinds = [op[0] for op in ops]
        self.assertIn("sub", kinds)
        self.assertIn("match", kinds)


class RecordRefuseTests(unittest.TestCase):
    def test_missing_bin_is_refused(self):
        mod = load()
        with self.assertRaises(mod.RecordRefused) as ctx:
            mod.require_record_bin(None)
        self.assertIn("--bin", str(ctx.exception))
        with self.assertRaises(mod.RecordRefused):
            mod.require_record_bin("   ")
        with self.assertRaises(mod.RecordRefused) as ctx:
            mod.require_record_bin(os.path.join("no", "such", "rhwp-binary"))
        self.assertIn("찾을 수 없", str(ctx.exception))

    def test_record_without_bin_does_not_write(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            out = os.path.join(tmp, "t.jsonl")
            with self.assertRaises(mod.RecordRefused):
                mod.record_session(session_doc(), None, out_path=out)
            self.assertFalse(os.path.exists(out))

    def test_injected_execute_still_requires_bin_flag(self):
        mod = load()
        fake = lambda _bin, argv: {"exit": 0, "stdout": b""}
        with self.assertRaises(mod.RecordRefused):
            mod.record_session(session_doc(), None, execute=fake)

    def test_injected_execute_records_jsonl(self):
        mod = load()
        calls = []

        def fake(bin_path, argv):
            calls.append((bin_path, list(argv)))
            return {"exit": 0, "stdout": b"{}"}

        with tempfile.TemporaryDirectory() as tmp:
            # 존재하는 더미 파일은 --bin 자리만 채운다. 실행은 fake 가 맡는다.
            dummy = os.path.join(tmp, "dummy-rhwp")
            with open(dummy, "w", encoding="utf-8", newline="\n") as fh:
                fh.write("not-a-real-binary\n")
            out = os.path.join(tmp, "trace.jsonl")
            sub = os.path.join(tmp, "work")
            ctx = mod.SessionContext("samples/x.hwp", sub)
            events = mod.record_session(
                session_doc(), dummy, out_path=out, context=ctx, execute=fake,
                clock=lambda: "2026-08-18T00:00:00Z",
            )
            self.assertEqual(len(events), 2)
            self.assertEqual(events[0]["argv"][0], "info")
            self.assertEqual(events[0]["argv"][1], "samples/x.hwp")
            self.assertEqual(events[1]["argv"][0], "export-text")
            self.assertTrue(os.path.isfile(out))
            replayed = mod.parse_trace_jsonl(Path(out).read_text(encoding="utf-8"))
            self.assertEqual(len(replayed), 2)
            self.assertEqual(calls[0][0], dummy)
        self.assertEqual(len(calls), 2)


class CliTests(unittest.TestCase):
    def _write(self, tmp, name, text):
        path = os.path.join(tmp, name)
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
        return path

    def test_validate_cli_json(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = self._write(tmp, "s.json", json.dumps(session_doc(), ensure_ascii=False))
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["validate", "--session", path, "--json"])
            self.assertEqual(code, 0)
            payload = json.loads(buf.getvalue())
            self.assertEqual(payload["kind"], "gymAgentSessionValidate")
            self.assertTrue(payload["ok"])

    def test_score_replay_cli_pass_and_wrong_order(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = self._write(tmp, "s.json", json.dumps(session_doc(), ensure_ascii=False))
            good = self._write(tmp, "ok.jsonl", PASS_TRACE)
            bad = self._write(tmp, "order.jsonl", WRONG_ORDER_TRACE)
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["score-replay", "--session", session, "--replay", good, "--json"])
            self.assertEqual(code, 0)
            self.assertTrue(json.loads(buf.getvalue())["ok"])

            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["score-replay", "--session", session, "--replay", bad, "--json"])
            self.assertEqual(code, 1)
            report = json.loads(buf.getvalue())
            self.assertFalse(report["ok"])
            self.assertIn("wrongOrder", reasons_of(report))

    def test_record_cli_refuses_without_bin(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = self._write(tmp, "s.json", json.dumps(session_doc(), ensure_ascii=False))
            out = os.path.join(tmp, "t.jsonl")
            err = io.StringIO()
            with redirect_stderr(err):
                code = mod.main(["record", "--session", session, "--out", out])
            self.assertEqual(code, 2)
            self.assertIn("--bin", err.getvalue())
            self.assertFalse(os.path.exists(out))

    def test_no_subcommand_is_usage(self):
        mod = load()
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = mod.main([])
        self.assertEqual(code, 2)
        self.assertIn("usage", buf.getvalue().lower())


class RenderTests(unittest.TestCase):
    def test_human_render_mentions_order(self):
        mod = load()
        text = mod.render_score(mod.score_session(session_doc(), events_of(WRONG_ORDER_TRACE)))
        self.assertIn("실패", text)
        self.assertIn("순서", text)

    def test_human_render_pass_and_extra_and_exit(self):
        mod = load()
        ok = mod.render_score(mod.score_session(session_doc(), events_of(PASS_TRACE)))
        self.assertIn("통과", ok)
        extra = mod.render_score(mod.score_session(session_doc(), events_of(EXTRA_STEP_TRACE)))
        self.assertIn("여분", extra)
        missing = mod.render_score(mod.score_session(session_doc(), events_of(MISSING_STEP_TRACE)))
        self.assertIn("누락", missing)
        wrong = mod.render_score(mod.score_session(session_doc(), events_of(WRONG_COMMAND_TRACE)))
        self.assertIn("계열", wrong)
        exit_txt = mod.render_score(mod.score_session(session_doc(), events_of(WRONG_EXIT_TRACE)))
        self.assertIn("종료", exit_txt)

    def test_reason_labels_cover_all_reason_constants(self):
        mod = load()
        for reason in (
            mod.REASON_WRONG_COMMAND,
            mod.REASON_WRONG_ORDER,
            mod.REASON_WRONG_EXIT,
            mod.REASON_EXTRA_STEP,
            mod.REASON_MISSING_STEP,
            mod.REASON_WRONG_PATH,
            mod.REASON_BAD_TRACE,
            mod.REASON_BAD_SESSION,
        ):
            label = mod.reason_label_ko(reason)
            self.assertTrue(label)
            self.assertNotEqual(label, "")


class ReplayWithoutBinaryContractTests(unittest.TestCase):
    def test_score_replay_never_requires_bin(self):
        """재생 채점은 --bin 없이 픽스처만으로 끝나야 한다."""
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = os.path.join(tmp, "s.json")
            replay = os.path.join(tmp, "t.jsonl")
            with open(session, "w", encoding="utf-8", newline="\n") as fh:
                json.dump(session_doc(), fh, ensure_ascii=False)
            with open(replay, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(PASS_TRACE)
            parser = mod.build_parser()
            args = parser.parse_args([
                "score-replay", "--session", session, "--replay", replay, "--json",
            ])
            self.assertFalse(hasattr(args, "bin") and getattr(args, "bin", None))
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.cmd_score_replay(args)
            self.assertEqual(code, 0)
            self.assertTrue(json.loads(buf.getvalue())["ok"])

    def test_parser_score_replay_has_no_bin_flag(self):
        # record 도움말에만 --bin 이 있다. score-replay 서브파서는 bin 을 받지 않는다.
        replay_help = None
        parser = load().build_parser()
        for action in parser._actions:
            choices = getattr(action, "choices", None)
            if not isinstance(choices, dict):
                continue
            if "score-replay" in choices:
                replay_help = choices["score-replay"].format_help()
        self.assertIsNotNone(replay_help)
        self.assertNotIn("--bin", replay_help)


class CheckPathsScoringTests(unittest.TestCase):
    def test_replay_ignores_missing_expect_path(self):
        mod = load()
        report = mod.score_session(session_doc(), events_of(PASS_TRACE), check_paths=False)
        self.assertTrue(report["ok"])
        self.assertIsNone(report["steps"][1]["pathOk"])

    def test_check_paths_false_path_is_wrong_path(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            ctx = mod.SessionContext("samples/x.hwp", tmp)
            report = mod.score_session(
                session_doc(), events_of(PASS_TRACE), context=ctx, check_paths=True,
            )
            self.assertFalse(report["ok"])
            self.assertIn("wrongPath", reasons_of(report))


class ClassifyMoreSequenceTests(unittest.TestCase):
    def test_empty_and_prefix_and_mixed(self):
        mod = load()
        self.assertEqual(mod.classify_sequence([], []), [])
        self.assertEqual(mod.classify_sequence([], ["a"]), ["extraStep"])
        self.assertEqual(mod.classify_sequence(["a"], []), ["missingStep"])
        self.assertEqual(mod.classify_sequence(["a", "b", "c"], ["a", "c"]), ["missingStep"])
        reasons = mod.classify_sequence(["a", "b"], ["x", "y", "z"])
        self.assertTrue(reasons)
        self.assertTrue(any(r in reasons for r in ("wrongCommand", "extraStep", "missingStep")))

    def test_same_multiset_helper(self):
        mod = load()
        self.assertTrue(mod.same_multiset(["a", "b"], ["b", "a"]))
        self.assertFalse(mod.same_multiset(["a", "a"], ["a"]))
        self.assertFalse(mod.same_multiset(["a"], ["b"]))


class ShaAndFamilyTests(unittest.TestCase):
    def test_sha256_none_and_bytes(self):
        mod = load()
        self.assertIsNone(mod.sha256_text(None))
        self.assertEqual(mod.sha256_text("ab"), mod.sha256_bytes(b"ab"))
        self.assertEqual(mod.command_family([]), "")
        self.assertEqual(mod.command_family([1]), "")
        self.assertEqual(mod.command_family(["export-text", "x"]), "export-text")

    def test_expected_ok_path_false_blocks(self):
        mod = load()
        self.assertTrue(mod.expected_ok(0, 0, None))
        self.assertTrue(mod.expected_ok(0, 0, True))
        self.assertFalse(mod.expected_ok(0, 0, False))
        self.assertFalse(mod.expected_ok(1, 0, True))


if __name__ == "__main__":
    unittest.main()
