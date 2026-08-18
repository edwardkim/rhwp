"""[agent_session] 예외 계층·입출력 가드·CLI 실패 접기.

재생(score-replay)은 바이너리 없이 픽스처만으로 실패 유형을 분류해야 한다.
record 는 --bin 없거나 실행 파일이 없으면 RecordRefused 로 거절하고
트레이스를 쓰지 않는다. 새 rhwp CLI 는 만들지 않는다.
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

PASS_TRACE = (
    '{"ts":"2026-08-18T00:00:00Z","argv":["info","samples/x.hwp","--json"],'
    '"exit":0,"ok":true}\n'
    '{"ts":"2026-08-18T00:00:01Z","argv":["export-text","samples/x.hwp","-o",'
    '"work/out.txt"],"exit":0,"ok":true}\n'
)


def load():
    spec = importlib.util.spec_from_file_location("gym_agent_session_errors", TOOL)
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


def write_text(path, text):
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    return path


class ExceptionHierarchyTests(unittest.TestCase):
    def test_all_codes_are_unique_and_mapped(self):
        mod = load()
        names = mod.error_code_names()
        self.assertEqual(len(names), len(set(names)))
        self.assertGreaterEqual(len(names), 11)
        for code in names:
            cls = mod.error_class_for_code(code)
            self.assertTrue(issubclass(cls, mod.SessionError))
            inst = cls("x")
            self.assertEqual(inst.code, code)
            payload = inst.to_dict()
            self.assertEqual(payload["code"], code)
            self.assertEqual(payload["type"], cls.__name__)
            self.assertIn("message", payload)
            self.assertIn("exitCode", payload)

    def test_unknown_code_falls_back_to_session_error(self):
        mod = load()
        self.assertIs(mod.error_class_for_code("no-such-code"), mod.SessionError)

    def test_record_refused_is_usage_exit(self):
        mod = load()
        exc = mod.RecordRefused("거절")
        self.assertEqual(exc.exit_code, 2)
        self.assertEqual(exc.code, "recordRefused")
        self.assertIsInstance(exc, mod.SessionError)
        self.assertIsInstance(exc, ValueError)

    def test_to_dict_includes_optional_fields(self):
        mod = load()
        exc = mod.TraceParseError("bad", path="t.jsonl", line=3, detail=["x"])
        payload = exc.to_dict()
        self.assertEqual(payload["path"], "t.jsonl")
        self.assertEqual(payload["line"], 3)
        self.assertEqual(payload["detail"], ["x"])

    def test_subclass_identity(self):
        mod = load()
        pairs = [
            (mod.SessionFileError, "sessionFile"),
            (mod.SessionParseError, "sessionParse"),
            (mod.SessionSchemaError, "sessionSchema"),
            (mod.TraceFileError, "traceFile"),
            (mod.TraceParseError, "traceParse"),
            (mod.TraceSchemaError, "traceSchema"),
            (mod.ExecuteError, "executeError"),
            (mod.WriteError, "writeError"),
            (mod.PlaceholderError, "placeholderError"),
        ]
        for cls, code in pairs:
            self.assertTrue(issubclass(cls, mod.SessionError), cls)
            self.assertEqual(cls.code, code)


class ClassifyExceptionTests(unittest.TestCase):
    def test_session_side_maps_to_bad_session(self):
        mod = load()
        self.assertEqual(mod.classify_exception(mod.SessionFileError("x")), "badSession")
        self.assertEqual(mod.classify_exception(mod.SessionParseError("x")), "badSession")
        self.assertEqual(mod.classify_exception(mod.SessionSchemaError("x")), "badSession")
        self.assertEqual(mod.classify_exception(mod.RecordRefused("x")), "badSession")

    def test_trace_side_maps_to_bad_trace(self):
        mod = load()
        self.assertEqual(mod.classify_exception(mod.TraceFileError("x")), "badTrace")
        self.assertEqual(mod.classify_exception(mod.TraceParseError("x")), "badTrace")
        self.assertEqual(mod.classify_exception(mod.TraceSchemaError("x")), "badTrace")
        self.assertEqual(mod.classify_exception(mod.SessionError("x")), "badTrace")
        self.assertEqual(mod.classify_exception(RuntimeError("x")), "badTrace")

    def test_fail_score_report_shape(self):
        mod = load()
        report = mod.fail_score_report("badSession", "세션 없음", session_id="s")
        self.assertEqual(report["kind"], "gymAgentSession")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertFalse(report["ok"])
        self.assertEqual(report["sessionId"], "s")
        self.assertEqual(report["mismatches"][0]["reason"], "badSession")
        self.assertEqual(report["steps"], [])


class WrapIoErrorTests(unittest.TestCase):
    def test_file_not_found_and_permission(self):
        mod = load()
        missing = mod.wrap_io_error(FileNotFoundError("no"), "gone.json", reading=True)
        self.assertIsInstance(missing, mod.SessionFileError)
        self.assertIn("찾을 수 없다", str(missing))
        perm = mod.wrap_io_error(PermissionError("no"), "locked.json", reading=True)
        self.assertIsInstance(perm, mod.SessionFileError)
        self.assertIn("권한", str(perm))

    def test_directory_is_classified(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            wrapped = mod.wrap_io_error(PermissionError("win"), tmp, reading=True)
            self.assertIn("디렉터리", str(wrapped))
            self.assertEqual(wrapped.path, tmp)

    def test_write_side_uses_write_error(self):
        mod = load()
        wrapped = mod.wrap_io_error(OSError("disk"), "out.jsonl", reading=False)
        self.assertIsInstance(wrapped, mod.WriteError)
        self.assertIn("쓰기", str(wrapped))

    def test_is_a_directory_error_when_available(self):
        mod = load()
        wrapped = mod.wrap_io_error(IsADirectoryError("dir"), "d", reading=True)
        self.assertIn("디렉터리", str(wrapped))


class LoadJsonFileErrorTests(unittest.TestCase):
    def test_empty_path(self):
        mod = load()
        with self.assertRaises(mod.SessionFileError):
            mod.load_json_file("")
        with self.assertRaises(mod.SessionFileError):
            mod.load_json_file(None)

    def test_missing_file(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "no.json")
            with self.assertRaises(mod.SessionFileError) as ctx:
                mod.load_json_file(path)
            self.assertIn("찾을 수 없다", str(ctx.exception))
            self.assertEqual(ctx.exception.path, path)

    def test_directory_path(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(mod.SessionFileError) as ctx:
                mod.load_json_file(tmp)
            self.assertIn("디렉터리", str(ctx.exception))

    def test_invalid_json(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "bad.json"), "{not json")
            with self.assertRaises(mod.SessionParseError) as ctx:
                mod.load_json_file(path)
            self.assertIn("JSON", str(ctx.exception))
            self.assertEqual(ctx.exception.path, path)

    def test_non_utf8(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "latin.json")
            with open(path, "wb") as fh:
                fh.write(b"\xff\xfe{")
            with self.assertRaises(mod.SessionParseError) as ctx:
                mod.load_json_file(path)
            self.assertIn("UTF-8", str(ctx.exception))

    def test_valid_object(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "ok.json"), '{"id":"x"}')
            self.assertEqual(mod.load_json_file(path)["id"], "x")


class LoadTextAndTraceFileErrorTests(unittest.TestCase):
    def test_empty_trace_path(self):
        mod = load()
        with self.assertRaises(mod.TraceFileError):
            mod.load_text_file("")
        with self.assertRaises(mod.TraceFileError):
            mod.load_text_file("   ")

    def test_missing_trace_file(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "no.jsonl")
            with self.assertRaises(mod.TraceFileError):
                mod.load_trace_file(path)

    def test_directory_as_trace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(mod.TraceFileError):
                mod.load_text_file(tmp)

    def test_empty_trace_body(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "empty.jsonl"), "\n\n")
            with self.assertRaises(mod.TraceParseError) as ctx:
                mod.load_trace_file(path)
            self.assertIn("이벤트", str(ctx.exception))

    def test_bad_jsonl_line(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "bad.jsonl"), "not-json\n")
            with self.assertRaises(mod.TraceParseError) as ctx:
                mod.load_trace_file(path)
            self.assertEqual(ctx.exception.line, 1)

    def test_jsonl_array_line_is_rejected(self):
        mod = load()
        with self.assertRaises(mod.TraceParseError):
            mod.parse_trace_jsonl("[1,2]\n")

    def test_jsonl_scalar_line_is_rejected(self):
        mod = load()
        with self.assertRaises(mod.TraceParseError):
            mod.parse_trace_jsonl("12\n")

    def test_bom_is_stripped(self):
        mod = load()
        text = "\ufeff" + PASS_TRACE
        events = mod.parse_trace_jsonl(text)
        self.assertEqual(len(events), 2)
        self.assertEqual(events[0]["argv"][0], "info")

    def test_none_and_non_string_trace(self):
        mod = load()
        with self.assertRaises(mod.TraceParseError):
            mod.parse_trace_jsonl(None)
        with self.assertRaises(mod.TraceParseError):
            mod.parse_trace_jsonl(123)

    def test_schema_invalid_event_raises_trace_schema(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(
                os.path.join(tmp, "schema.jsonl"),
                '{"ts":"t","argv":["info"],"exit":0}\n',
            )
            with self.assertRaises(mod.TraceSchemaError) as ctx:
                mod.load_trace_file(path)
            self.assertTrue(ctx.exception.detail)
            self.assertTrue(any("ok" in i for i in ctx.exception.detail))

    def test_non_utf8_trace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "bin.jsonl")
            with open(path, "wb") as fh:
                fh.write(b"\xff\xfe{")
            with self.assertRaises(mod.TraceParseError):
                mod.load_trace_file(path)


class LoadSessionFileErrorTests(unittest.TestCase):
    def test_schema_error_carries_issues(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "s.json"), '{"id":"","steps":[]}')
            with self.assertRaises(mod.SessionSchemaError) as ctx:
                mod.load_session_file(path)
            self.assertTrue(ctx.exception.detail)
            self.assertTrue(any("id" in i or "steps" in i for i in ctx.exception.detail))

    def test_good_session_returns_doc(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(
                os.path.join(tmp, "s.json"),
                json.dumps(session_doc(), ensure_ascii=False),
            )
            doc = mod.load_session_file(path)
            self.assertEqual(doc["id"], "inspect-then-export")


class WriteJsonlErrorTests(unittest.TestCase):
    def test_empty_out_path(self):
        mod = load()
        with self.assertRaises(mod.WriteError):
            mod.write_jsonl("", [{"ts": "t", "argv": ["info"], "exit": 0, "ok": True}])

    def test_events_must_be_list(self):
        mod = load()
        with self.assertRaises(mod.WriteError):
            mod.write_jsonl("x.jsonl", {"not": "list"})

    def test_directory_as_out_path(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(mod.WriteError):
                mod.write_jsonl(tmp, [{"ts": "t", "argv": ["info"], "exit": 0, "ok": True}])

    def test_unserializable_event(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "t.jsonl")
            with self.assertRaises(mod.WriteError):
                mod.write_jsonl(path, [{"ts": "t", "argv": [object()]}])

    def test_writes_trailing_newline(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "nested", "t.jsonl")
            ev = {"ts": "t", "argv": ["info"], "exit": 0, "ok": True}
            mod.write_jsonl(path, [ev])
            text = Path(path).read_text(encoding="utf-8")
            self.assertTrue(text.endswith("\n"))
            self.assertEqual(len(text.strip().splitlines()), 1)

    def test_empty_events_writes_empty_file(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "empty.jsonl")
            mod.write_jsonl(path, [])
            self.assertEqual(Path(path).read_text(encoding="utf-8"), "")


class PlaceholderStrictTests(unittest.TestCase):
    def test_strict_missing_input(self):
        mod = load()
        with self.assertRaises(mod.PlaceholderError):
            mod.resolve_token("{input}", None, strict=True)

    def test_strict_missing_sub(self):
        mod = load()
        with self.assertRaises(mod.PlaceholderError):
            mod.resolve_token("{sub:a.txt}", mod.SessionContext(), strict=True)

    def test_strict_unknown(self):
        mod = load()
        with self.assertRaises(mod.PlaceholderError):
            mod.resolve_token("{nope}", mod.SessionContext("in.hwp", "w"), strict=True)

    def test_non_strict_passthrough(self):
        mod = load()
        self.assertEqual(mod.resolve_token("{input}", None), "{input}")
        self.assertEqual(mod.resolve_token("{nope}", None), "{nope}")

    def test_resolve_argv_none_is_schema_error(self):
        mod = load()
        with self.assertRaises(mod.SessionSchemaError):
            mod.resolve_argv(None, None)

    def test_strict_argv_resolves(self):
        mod = load()
        ctx = mod.SessionContext("in.hwp", "w")
        argv = mod.resolve_argv(["info", "{input}"], ctx, strict=True)
        self.assertEqual(argv, ["info", "in.hwp"])


class RecordExecuteErrorTests(unittest.TestCase):
    def test_executor_exception_is_execute_error(self):
        mod = load()

        def boom(_bin, _argv):
            raise RuntimeError("boom")

        with tempfile.TemporaryDirectory() as tmp:
            dummy = write_text(os.path.join(tmp, "dummy"), "x")
            with self.assertRaises(mod.ExecuteError) as ctx:
                mod.record_session(session_doc(), dummy, execute=boom)
            self.assertIn("실행기 예외", str(ctx.exception))

    def test_executor_without_exit(self):
        mod = load()

        def bad(_bin, _argv):
            return {"stdout": b""}

        with tempfile.TemporaryDirectory() as tmp:
            dummy = write_text(os.path.join(tmp, "dummy"), "x")
            with self.assertRaises(mod.ExecuteError) as ctx:
                mod.record_session(session_doc(), dummy, execute=bad)
            self.assertIn("exit", str(ctx.exception))

    def test_executor_non_int_exit(self):
        mod = load()

        def bad(_bin, _argv):
            return {"exit": "zero"}

        with tempfile.TemporaryDirectory() as tmp:
            dummy = write_text(os.path.join(tmp, "dummy"), "x")
            with self.assertRaises(mod.ExecuteError):
                mod.record_session(session_doc(), dummy, execute=bad)

    def test_clock_exception_is_session_error(self):
        mod = load()

        def fake(_bin, _argv):
            return {"exit": 0, "stdout": b""}

        def bad_clock():
            raise RuntimeError("clock")

        with tempfile.TemporaryDirectory() as tmp:
            dummy = write_text(os.path.join(tmp, "dummy"), "x")
            with self.assertRaises(mod.SessionError) as ctx:
                mod.record_session(
                    session_doc(), dummy, execute=fake, clock=bad_clock,
                )
            self.assertIn("시계", str(ctx.exception))

    def test_invalid_session_is_schema_error(self):
        mod = load()
        with self.assertRaises(mod.SessionSchemaError):
            mod.record_session({"id": ""}, "dummy")

    def test_session_error_from_executor_is_not_wrapped(self):
        mod = load()

        def refuse(_bin, _argv):
            raise mod.RecordRefused("안 함")

        with tempfile.TemporaryDirectory() as tmp:
            dummy = write_text(os.path.join(tmp, "dummy"), "x")
            with self.assertRaises(mod.RecordRefused):
                mod.record_session(session_doc(), dummy, execute=refuse)


class DefaultExecuteErrorTests(unittest.TestCase):
    def test_empty_bin(self):
        mod = load()
        with self.assertRaises(mod.ExecuteError):
            mod.default_execute("", ["info"])

    def test_missing_bin_file(self):
        mod = load()
        with self.assertRaises(mod.ExecuteError):
            mod.default_execute(os.path.join("no", "such", "rhwp-bin"), ["info"])


class RequireRecordBinMessageTests(unittest.TestCase):
    def test_messages_stay_korean_and_refuse_forgery(self):
        mod = load()
        with self.assertRaises(mod.RecordRefused) as ctx:
            mod.require_record_bin(None)
        self.assertIn("위조", str(ctx.exception))
        with self.assertRaises(mod.RecordRefused) as ctx:
            mod.require_record_bin(os.path.join("no", "rhwp"))
        self.assertIn("가장해", str(ctx.exception))

    def test_existing_file_is_accepted(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "dummy"), "x")
            self.assertEqual(mod.require_record_bin(path), path)


class CliValidateErrorTests(unittest.TestCase):
    def test_missing_session_json_exit_1(self):
        mod = load()
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = mod.main(["validate", "--session", "no-such-session.json", "--json"])
        self.assertEqual(code, 1)
        payload = json.loads(buf.getvalue())
        self.assertFalse(payload["ok"])
        self.assertGreaterEqual(payload["issueCount"], 1)
        self.assertIn("찾을 수 없다", payload["issues"][0])

    def test_bad_json_session(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "s.json"), "{")
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["validate", "--session", path, "--json"])
            self.assertEqual(code, 1)
            payload = json.loads(buf.getvalue())
            self.assertFalse(payload["ok"])
            self.assertTrue(any("JSON" in i for i in payload["issues"]))

    def test_schema_invalid_session(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = write_text(os.path.join(tmp, "s.json"), '{"id":"x","steps":[]}')
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["validate", "--session", path, "--json"])
            self.assertEqual(code, 1)
            payload = json.loads(buf.getvalue())
            self.assertEqual(payload["kind"], "gymAgentSessionValidate")
            self.assertTrue(any("steps" in i for i in payload["issues"]))

    def test_directory_as_session(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["validate", "--session", tmp, "--json"])
            self.assertEqual(code, 1)
            payload = json.loads(buf.getvalue())
            self.assertTrue(any("디렉터리" in i for i in payload["issues"]))


class CliScoreReplayErrorTests(unittest.TestCase):
    def _write(self, tmp, name, text):
        return write_text(os.path.join(tmp, name), text)

    def test_missing_session_is_bad_session(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            replay = self._write(tmp, "t.jsonl", PASS_TRACE)
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main([
                    "score-replay",
                    "--session", os.path.join(tmp, "no.json"),
                    "--replay", replay,
                    "--json",
                ])
            self.assertEqual(code, 1)
            report = json.loads(buf.getvalue())
            self.assertFalse(report["ok"])
            self.assertEqual(report["mismatches"][0]["reason"], "badSession")

    def test_missing_replay_is_bad_trace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = self._write(tmp, "s.json", json.dumps(session_doc(), ensure_ascii=False))
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main([
                    "score-replay",
                    "--session", session,
                    "--replay", os.path.join(tmp, "no.jsonl"),
                    "--json",
                ])
            self.assertEqual(code, 1)
            report = json.loads(buf.getvalue())
            self.assertFalse(report["ok"])
            self.assertEqual(report["sessionId"], "inspect-then-export")
            self.assertEqual(report["mismatches"][0]["reason"], "badTrace")

    def test_bad_jsonl_is_bad_trace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = self._write(tmp, "s.json", json.dumps(session_doc(), ensure_ascii=False))
            replay = self._write(tmp, "t.jsonl", "{{{")
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main([
                    "score-replay", "--session", session, "--replay", replay, "--json",
                ])
            self.assertEqual(code, 1)
            report = json.loads(buf.getvalue())
            self.assertEqual(report["mismatches"][0]["reason"], "badTrace")

    def test_schema_invalid_session_is_bad_session(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = self._write(tmp, "s.json", '{"id":"","steps":[{"run":["info"]}]}')
            replay = self._write(tmp, "t.jsonl", PASS_TRACE)
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main([
                    "score-replay", "--session", session, "--replay", replay, "--json",
                ])
            self.assertEqual(code, 1)
            report = json.loads(buf.getvalue())
            self.assertEqual(report["mismatches"][0]["reason"], "badSession")

    def test_replay_does_not_look_for_rhwp_binary(self):
        """score-replay 는 rhwp 를 PATH 에서 찾지 않는다."""
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = self._write(tmp, "s.json", json.dumps(session_doc(), ensure_ascii=False))
            replay = self._write(tmp, "t.jsonl", PASS_TRACE)
            env_path = os.environ.get("PATH")
            try:
                os.environ["PATH"] = tmp
                buf = io.StringIO()
                with redirect_stdout(buf):
                    code = mod.main([
                        "score-replay", "--session", session, "--replay", replay, "--json",
                    ])
            finally:
                if env_path is None:
                    os.environ.pop("PATH", None)
                else:
                    os.environ["PATH"] = env_path
            self.assertEqual(code, 0)
            self.assertTrue(json.loads(buf.getvalue())["ok"])


class CliRecordErrorTests(unittest.TestCase):
    def test_missing_bin_is_usage_2(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = write_text(
                os.path.join(tmp, "s.json"),
                json.dumps(session_doc(), ensure_ascii=False),
            )
            out = os.path.join(tmp, "t.jsonl")
            err = io.StringIO()
            with redirect_stderr(err):
                code = mod.main(["record", "--session", session, "--out", out])
            self.assertEqual(code, 2)
            self.assertFalse(os.path.exists(out))

    def test_missing_bin_file_is_usage_2(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = write_text(
                os.path.join(tmp, "s.json"),
                json.dumps(session_doc(), ensure_ascii=False),
            )
            out = os.path.join(tmp, "t.jsonl")
            err = io.StringIO()
            with redirect_stderr(err):
                code = mod.main([
                    "record",
                    "--session", session,
                    "--bin", os.path.join(tmp, "no-rhwp"),
                    "--out", out,
                ])
            self.assertEqual(code, 2)
            self.assertIn("찾을 수 없", err.getvalue())
            self.assertFalse(os.path.exists(out))

    def test_bad_session_is_fail_1(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            session = write_text(os.path.join(tmp, "s.json"), '{"id":""}')
            dummy = write_text(os.path.join(tmp, "dummy"), "x")
            out = os.path.join(tmp, "t.jsonl")
            err = io.StringIO()
            with redirect_stderr(err):
                code = mod.main([
                    "record", "--session", session, "--bin", dummy, "--out", out,
                ])
            self.assertEqual(code, 1)
            self.assertFalse(os.path.exists(out))


class RenderErrorTests(unittest.TestCase):
    def test_render_error_prefixes(self):
        mod = load()
        self.assertTrue(mod.render_error(mod.RecordRefused("x")).startswith("기록 거절"))
        text = mod.render_error(mod.TraceParseError("bad", path="t", line=2))
        self.assertIn("traceParse", text)
        self.assertIn("줄=2", text)
        self.assertIn("미분류", mod.render_error(RuntimeError("z")))

    def test_validate_render_ok_and_fail(self):
        mod = load()
        self.assertIn("유효", mod.render_validate([], "sid"))
        fail = mod.render_validate(["id 가 비어 있다"])
        self.assertIn("위반", fail)
        self.assertIn("id", fail)


class ValidateSessionMoreEdgeTests(unittest.TestCase):
    def test_non_string_input_and_subdir(self):
        mod = load()
        issues = " ".join(mod.validate_session({
            "id": "x",
            "input": 1,
            "subDir": 2,
            "steps": [{"run": ["info"]}],
        }))
        self.assertIn("input", issues)
        self.assertIn("subDir", issues)

    def test_bool_expect_exit_rejected(self):
        mod = load()
        issues = " ".join(mod.validate_session({
            "id": "x",
            "steps": [{"run": ["info"], "expectExit": True}],
        }))
        self.assertIn("expectExit", issues)

    def test_non_dict_step(self):
        mod = load()
        issues = " ".join(mod.validate_session({"id": "x", "steps": [None, "info"]}))
        self.assertIn("객체", issues)

    def test_empty_sub_name(self):
        mod = load()
        issues = " ".join(mod.validate_session({
            "id": "x",
            "steps": [{"run": ["info", "{sub:}"]}],
        }))
        self.assertIn("이름", issues)

    def test_whitespace_id(self):
        mod = load()
        issues = " ".join(mod.validate_session({
            "id": "   ",
            "steps": [{"run": ["info"]}],
        }))
        self.assertIn("id", issues)

    def test_expect_path_unknown_placeholder(self):
        mod = load()
        issues = " ".join(mod.validate_session({
            "id": "x",
            "steps": [{"run": ["info"], "expectPath": "{mystery}"}],
        }))
        self.assertIn("자리표", issues)


class ValidateTraceEventMoreTests(unittest.TestCase):
    def test_non_dict_event(self):
        mod = load()
        self.assertTrue(any("객체" in i for i in mod.validate_trace_event([], 0)))

    def test_empty_argv_and_non_int_exit(self):
        mod = load()
        issues = " ".join(mod.validate_trace_event({
            "ts": "t", "argv": [], "exit": True, "ok": "yes",
        }, 1))
        self.assertIn("argv", issues)
        self.assertIn("exit", issues)
        self.assertIn("ok", issues)

    def test_argv_non_string(self):
        mod = load()
        issues = " ".join(mod.validate_trace_event({
            "ts": "t", "argv": [1], "exit": 0, "ok": True,
        }, 0))
        self.assertIn("문자열", issues)

    def test_valid_hex_digest_accepted(self):
        mod = load()
        digest = "a" * 64
        issues = mod.validate_trace_event({
            "ts": "t", "argv": ["info"], "exit": 0, "ok": True, "stdoutSha256": digest,
        }, 0)
        self.assertEqual(issues, [])

    def test_trace_not_list(self):
        mod = load()
        self.assertTrue(any("배열" in i for i in mod.validate_trace({})))


class ScoreInvalidTraceTests(unittest.TestCase):
    def test_invalid_trace_is_bad_trace(self):
        mod = load()
        report = mod.score_session(session_doc(), [{"argv": ["info"]}])
        self.assertFalse(report["ok"])
        self.assertEqual(report["mismatches"][0]["reason"], "badTrace")

    def test_non_list_events(self):
        mod = load()
        report = mod.score_session(session_doc(), "nope")
        self.assertFalse(report["ok"])
        self.assertEqual(report["mismatches"][0]["reason"], "badTrace")

    def test_non_dict_session(self):
        mod = load()
        report = mod.score_session([], [])
        self.assertFalse(report["ok"])
        self.assertEqual(report["mismatches"][0]["reason"], "badSession")


class LcsAndCollapseMoreTests(unittest.TestCase):
    def test_insert_then_delete_collapses_to_sub(self):
        mod = load()
        ops = mod.collapse_ops([("ins", None, 0), ("del", 0, None)])
        self.assertEqual(ops[0][0], "sub")

    def test_identical_lcs_is_all_match(self):
        mod = load()
        ops = mod.lcs_ops(["a", "b"], ["a", "b"])
        self.assertEqual([op[0] for op in ops], ["match", "match"])

    def test_empty_right_is_all_del(self):
        mod = load()
        ops = mod.lcs_ops(["a"], [])
        self.assertEqual([op[0] for op in ops], ["del"])

    def test_empty_left_is_all_ins(self):
        mod = load()
        ops = mod.lcs_ops([], ["a"])
        self.assertEqual([op[0] for op in ops], ["ins"])


class BuildTraceEventMoreTests(unittest.TestCase):
    def test_bytes_stdout_hashed(self):
        mod = load()
        ev = mod.build_trace_event(["info"], 0, stdout=b"hi", ts="t")
        self.assertEqual(ev["stdoutSha256"], mod.sha256_bytes(b"hi"))

    def test_path_ok_false_marks_not_ok(self):
        mod = load()
        ev = mod.build_trace_event(["info"], 0, expect_exit=0, path_ok=False, ts="t")
        self.assertFalse(ev["ok"])


class NormalizeAndFamilyTests(unittest.TestCase):
    def test_normalize_expect_exit_none_is_zero(self):
        mod = load()
        self.assertEqual(mod.normalize_expect_exit({}), 0)
        self.assertEqual(mod.normalize_expect_exit({"expectExit": None}), 0)
        self.assertEqual(mod.normalize_expect_exit({"expectExit": 3}), 3)

    def test_declared_family_empty_run(self):
        mod = load()
        self.assertEqual(mod.declared_family({}), "")
        self.assertEqual(mod.declared_family({"run": []}), "")


class CompareStepMissingEventTests(unittest.TestCase):
    def test_missing_event_is_not_ok(self):
        mod = load()
        row = mod.compare_step(0, {"run": ["info"], "expectExit": 0}, None, None)
        self.assertFalse(row["ok"])
        self.assertFalse(row["familyOk"])
        self.assertIsNone(row["observedFamily"])

    def test_unresolved_expect_path_skips_disk(self):
        mod = load()
        step = {"run": ["export-text"], "expectExit": 0, "expectPath": "{sub:out.txt}"}
        event = {"argv": ["export-text"], "exit": 0, "ok": True}
        row = mod.compare_step(0, step, event, None, check_paths=True)
        self.assertTrue(row["ok"])
        self.assertIsNone(row["pathOk"])


class SessionContextTests(unittest.TestCase):
    def test_from_session_cli_override(self):
        mod = load()
        ctx = mod.SessionContext.from_session(
            session_doc(), input_path="other.hwp", sub_dir="tmp",
        )
        self.assertEqual(ctx.input_path, "other.hwp")
        self.assertEqual(ctx.sub_dir, "tmp")

    def test_from_session_defaults(self):
        mod = load()
        ctx = mod.SessionContext.from_session(session_doc())
        self.assertEqual(ctx.input_path, "samples/x.hwp")
        self.assertEqual(ctx.sub_dir, "work")


class EmitAndMainTests(unittest.TestCase):
    def test_emit_text_and_json(self):
        mod = load()
        buf = io.StringIO()
        with redirect_stdout(buf):
            mod.emit({"ok": True}, True, "ignored")
        self.assertEqual(json.loads(buf.getvalue())["ok"], True)
        buf = io.StringIO()
        with redirect_stdout(buf):
            mod.emit({"ok": True}, False, "안녕")
        self.assertIn("안녕", buf.getvalue())

    def test_unknown_command_is_usage(self):
        mod = load()
        err = io.StringIO()
        with self.assertRaises(SystemExit) as ctx:
            with redirect_stderr(err):
                mod.main(["nope"])
        self.assertNotEqual(ctx.exception.code, 0)


class ErrorCatalogContractTests(unittest.TestCase):
    def test_catalog_tuples_have_hint(self):
        mod = load()
        for code, cls, hint in mod.ERROR_CODE_CATALOG:
            self.assertTrue(code)
            self.assertTrue(issubclass(cls, mod.SessionError))
            self.assertTrue(hint)
            self.assertIsInstance(hint, str)

    def test_render_error_uses_code(self):
        mod = load()
        for code, cls, _hint in mod.ERROR_CODE_CATALOG:
            if cls is mod.RecordRefused:
                continue
            text = mod.render_error(cls("메시지"))
            self.assertIn(code, text)
            self.assertIn("메시지", text)


if __name__ == "__main__":
    unittest.main()
