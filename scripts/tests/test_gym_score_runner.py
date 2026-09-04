"""[#5260] gym score/runner 예외 경로 계약.

채점 성공 칸(exit 3 판정, T12, 잘못된 대상 거부)은 test_gym_score.py 가
지킨다. 이 파일은 그 칸을 바꾸지 않고, 예전이 삼키거나 전체를 죽이던
자리를 kind 로 남기는 계약을 고정한다.

새 CLI 플래그는 없다. pack JSON 도 건드리지 않는다.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[2]


def load_runner():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    from gym.core import runner
    return runner


def load_score():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    import importlib
    return importlib.import_module("gym.score")


def _write(path, payload):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        if isinstance(payload, str):
            fh.write(payload)
        else:
            json.dump(payload, fh, ensure_ascii=False)


def _task(task_id="T01", tier=1, title="t", checks=None, **extra):
    body = {
        "id": task_id,
        "tier": tier,
        "title": title,
        "input": "samples/x.hwp",
        "instructions": "do",
        "submit": {"kind": "answer"},
        "checks": checks if checks is not None else [
            {"name": "exists", "op": "file_exists", "file": "answer.json"},
        ],
    }
    body.update(extra)
    return body


def _plant_pack(root, pack_id, tasks, manifest=None):
    pack_dir = os.path.join(root, pack_id)
    tasks_dir = os.path.join(pack_dir, "tasks")
    os.makedirs(tasks_dir, exist_ok=True)
    body = {
        "schemaVersion": "1.0",
        "kind": "gymPack",
        "id": pack_id,
        "title": pack_id,
        "axis": "시험",
        "requires": {"commands": ["info"]},
        "runner": {
            "rhwpVersion": "0.0.0",
            "rhwpCommit": "c" * 40,
            "capabilitiesSha256": "a" * 64,
        },
    }
    if manifest:
        body.update(manifest)
    _write(os.path.join(pack_dir, "pack.json"), body)
    for task in tasks:
        _write(os.path.join(tasks_dir, f"{task['id']}.json"), task)
    return pack_dir


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_exception_kinds_are_unique_and_documented(self):
        kinds = self.r.EXCEPTION_KINDS
        self.assertEqual(len(kinds), len(set(kinds)))
        for kind in kinds:
            self.assertIn(kind, self.r.EXCEPTION_KIND_HELP)
            self.assertTrue(self.r.EXCEPTION_KIND_HELP[kind])
            self.assertTrue(self.r.is_known_exception_kind(kind))
            self.assertTrue(self.r.describe_exception_kind(kind))

    def test_unknown_kind_falls_back_to_unexpected_help(self):
        self.assertEqual(
            self.r.describe_exception_kind("not-a-kind"),
            self.r.EXCEPTION_KIND_HELP["unexpected"],
        )

    def test_pack_statuses_are_exactly_three(self):
        self.assertEqual(self.r.PACK_STATUSES, ("scored", "unavailable", "error"))
        self.assertTrue(self.r.is_known_pack_status("error"))
        self.assertFalse(self.r.is_known_pack_status("failed"))

    def test_exit_codes_are_not_invented(self):
        self.assertEqual(self.r.EXIT_PERFECT, 0)
        self.assertEqual(self.r.EXIT_IMPERFECT, 3)

    def test_schema_kind_unchanged(self):
        self.assertEqual(self.r.REPORT_KIND, "gymScorecard")
        self.assertEqual(self.r.SCHEMA_VERSION, "2.0")
        self.assertEqual(self.r.ADMISSION_KIND, "gymAdmission")
        self.assertEqual(self.r.ADMISSION_SCHEMA, "1.0")

    def test_head_limits(self):
        self.assertEqual(self.r.HEAD_LIMIT, 200)
        self.assertEqual(self.r.ERROR_HEAD_LIMIT, 160)

    def test_fatal_tuple(self):
        self.assertTrue(self.r.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(self.r.is_fatal_exception(SystemExit(1)))
        self.assertTrue(self.r.is_fatal_exception(MemoryError()))
        self.assertTrue(self.r.is_fatal_exception(GeneratorExit()))
        self.assertFalse(self.r.is_fatal_exception(ValueError("x")))
        self.assertFalse(self.r.is_fatal_exception(None))


class SafeIdTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_accepts_hyphenated_pack_ids(self):
        for name in ("core-cli", "casual-rides", "objects-media", "T01", "a_b"):
            self.assertTrue(self.r.is_safe_id(name), name)

    def test_rejects_traversal_and_separators(self):
        for name in ("", ".", "..", "../x", "a/b", "a\\b", "C:foo", "x\x00y", None, 1):
            self.assertFalse(self.r.is_safe_id(name), name)

    def test_require_safe_id_raises(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.require_safe_id("../evil", "pack")
        self.assertEqual(ctx.exception.kind, "unsafe-id")

    def test_require_safe_id_returns_name(self):
        self.assertEqual(self.r.require_safe_id("core-cli", "pack"), "core-cli")

    def test_relpath_allows_nested_file(self):
        self.assertTrue(self.r.is_safe_relpath("conv.hwpx"))
        self.assertTrue(self.r.is_safe_relpath("out/o1.hwp"))

    def test_relpath_rejects_parent_and_abs(self):
        for name in ("", "..", "../x", "/abs", "\\abs", "a/../b", "a//b", "a/./b"):
            self.assertFalse(self.r.is_safe_relpath(name), name)


class TruncateAndRowTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_truncate_none_and_non_str(self):
        self.assertEqual(self.r.truncate_head(None), "")
        self.assertEqual(self.r.truncate_head(12), "12")
        self.assertEqual(self.r.truncate_head("abcd", 2), "ab")
        self.assertEqual(self.r.truncate_head("abcd", 0), "")
        self.assertEqual(self.r.truncate_head("abcd", "nope"), "abcd")

    def test_error_head_none(self):
        self.assertEqual(self.r.error_head(None), "")
        self.assertIn("boom", self.r.error_head(ValueError("boom")))

    def test_exception_row_unknown_kind(self):
        row = self.r.exception_row("not-real", where="x", message="m")
        self.assertEqual(row["kind"], "unexpected")
        self.assertEqual(row["where"], "x")

    def test_exception_row_extra_does_not_clobber(self):
        row = self.r.exception_row("os-error", where="a", extra={"where": "b", "k": 1})
        self.assertEqual(row["where"], "a")
        self.assertEqual(row["k"], 1)


class ExceptionKindMapTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_file_not_found_depends_on_context(self):
        exc = FileNotFoundError("x")
        self.assertEqual(self.r.exception_kind(exc, "bin"), "missing-bin")
        self.assertEqual(self.r.exception_kind(exc, "pack"), "missing-pack")
        self.assertEqual(self.r.exception_kind(exc, "profile"), "missing-profile")
        self.assertEqual(self.r.exception_kind(exc, "submit"), "missing-submit")
        self.assertEqual(self.r.exception_kind(exc, "file"), "missing-file")
        self.assertEqual(self.r.exception_kind(exc, "write"), "write-error")

    def test_json_decode_depends_on_context(self):
        exc = json.JSONDecodeError("e", "d", 0)
        self.assertEqual(self.r.exception_kind(exc, "answer"), "malformed-answer")
        self.assertEqual(self.r.exception_kind(exc, "pack"), "malformed-pack")
        self.assertEqual(self.r.exception_kind(exc, "profile"), "malformed-profile")
        self.assertEqual(self.r.exception_kind(exc, "check"), "malformed-json")

    def test_permission_and_timeout_and_os(self):
        self.assertEqual(self.r.exception_kind(PermissionError("p"), "check"), "permission")
        self.assertEqual(self.r.exception_kind(PermissionError("p"), "write"), "write-error")
        self.assertEqual(self.r.exception_kind(TimeoutError("t"), "check"), "timeout")
        self.assertEqual(self.r.exception_kind(OSError("o"), "check"), "os-error")
        self.assertEqual(self.r.exception_kind(UnicodeDecodeError("utf-8", b"\xff", 0, 1, "x"), "check"),
                         "decode-error")

    def test_path_eval_only_in_check_context(self):
        self.assertEqual(self.r.exception_kind(KeyError("k"), "check"), "path-eval")
        self.assertEqual(self.r.exception_kind(KeyError("k"), "pack"), "value-error")
        self.assertEqual(self.r.exception_kind(TypeError("t"), "check"), "path-eval")
        self.assertEqual(self.r.exception_kind(TypeError("t"), "pack"), "type-error")

    def test_score_runner_error_keeps_kind(self):
        err = self.r.ScoreRunnerError("missing-pack", "x")
        self.assertEqual(self.r.exception_kind(err), "missing-pack")
        self.assertEqual(self.r.exception_kind(None), "unexpected")

    def test_wrap_exception_reraises_fatal(self):
        with self.assertRaises(KeyboardInterrupt):
            self.r.wrap_exception(KeyboardInterrupt(), "check")

    def test_wrap_exception_passthrough(self):
        err = self.r.ScoreRunnerError("os-error", "x")
        self.assertIs(self.r.wrap_exception(err), err)

    def test_wrap_value_error(self):
        wrapped = self.r.wrap_exception(ValueError("v"), "check")
        self.assertEqual(wrapped.kind, "value-error")
        self.assertTrue(self.r.is_catchable_exception(wrapped))
        self.assertFalse(self.r.is_catchable_exception(SystemExit()))


class FindBinTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_absolute_returned_as_is(self):
        path = os.path.abspath(os.path.join(tempfile.gettempdir(), "no-such-rhwp"))
        self.assertEqual(self.r.find_bin(path), path)

    def test_relative_resolved_when_exists(self):
        with tempfile.TemporaryDirectory() as d:
            name = "fake-rhwp.bin"
            target = os.path.join(d, name)
            with open(target, "wb") as fh:
                fh.write(b"x")
            old = os.getcwd()
            try:
                os.chdir(d)
                self.assertEqual(self.r.find_bin(name), os.path.abspath(target))
            finally:
                os.chdir(old)

    def test_fallback_is_rhwp_name(self):
        with mock.patch.dict(os.environ, {}, clear=False):
            os.environ.pop("RHWP_BIN", None)
            found = self.r.find_bin(None)
        self.assertTrue(isinstance(found, str) and found)

    def test_bin_looks_like_path(self):
        self.assertTrue(self.r.bin_looks_like_path(os.path.join("a", "b")))
        self.assertTrue(self.r.bin_looks_like_path(os.path.abspath("x")))
        self.assertFalse(self.r.bin_looks_like_path("rhwp"))
        self.assertFalse(self.r.bin_looks_like_path(""))

    def test_bin_is_missing_bare_name_is_not_claimed(self):
        self.assertFalse(self.r.bin_is_missing("rhwp"))
        self.assertTrue(self.r.bin_is_missing(""))
        self.assertTrue(self.r.bin_is_missing(os.path.join("no", "such", "rhwp")))


class ResolveArgsTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_input_and_file_and_literal(self):
        args = self.r.resolve_args(
            ["info", "{input}", "{file:out.hwp}", "--json"],
            {"input": "samples/a.hwp"},
            "sub",
        )
        self.assertEqual(args[1], "samples/a.hwp")
        self.assertEqual(args[2], os.path.join("sub", "out.hwp"))
        self.assertEqual(args[3], "--json")

    def test_missing_input_is_kind(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.resolve_args(["info", "{input}"], {}, "sub")
        self.assertEqual(ctx.exception.kind, "missing-input")

    def test_unsafe_file_placeholder(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.resolve_args(["x", "{file:../secret}"], {}, "sub")
        self.assertEqual(ctx.exception.kind, "unsafe-id")

    def test_unsafe_sha256_placeholder(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.resolve_args(["x", "{sha256:../secret}"], {}, "sub")
        self.assertEqual(ctx.exception.kind, "unsafe-id")

    def test_cmd_must_be_string_list(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.resolve_args("info", {}, "sub")
        self.assertEqual(ctx.exception.kind, "malformed-cmd")
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.resolve_args([1, 2], {}, "sub")
        self.assertEqual(ctx.exception.kind, "malformed-cmd")

    def test_sha256_placeholder_live_hash(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "o1.hwp"
            path.write_bytes(b"gym")
            args = self.r.resolve_args(
                ["replay", "{sha256:o1.hwp}"], {}, d)
            from gym.core.checks import sha256_of
            self.assertEqual(args[-1], sha256_of(str(path)))

    def test_tuple_cmd_is_accepted(self):
        args = self.r.resolve_args(("info", "--json"), {}, "sub")
        self.assertEqual(args, ["info", "--json"])

    def test_sha256_missing_file_is_not_missing_bin(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                self.r.resolve_args(["replay", "{sha256:nope.hwp}"], {}, d)
        self.assertEqual(ctx.exception.kind, "missing-file")
        self.assertIn("파일 없음:", ctx.exception.message)


class PrepareCliTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_empty_bin_is_missing_bin(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.prepare_cli("", ["info"])
        self.assertEqual(ctx.exception.kind, "missing-bin")

    def test_prepare_joins_argv(self):
        self.assertEqual(self.r.prepare_cli("rhwp", ["info"]), ["rhwp", "info"])

    def test_parse_envelope_rejects_non_object(self):
        self.assertIsNone(self.r.parse_envelope(""))
        self.assertIsNone(self.r.parse_envelope("[1]"))
        self.assertIsNone(self.r.parse_envelope("not-json"))
        self.assertEqual(self.r.parse_envelope('{"a":1}'), {"a": 1})

    def test_decode_cli_stdout(self):
        self.assertEqual(self.r.decode_cli_stdout(None), "")
        self.assertEqual(self.r.decode_cli_stdout("hi"), "hi")
        self.assertEqual(self.r.decode_cli_stdout(b"hi"), "hi")
        self.assertTrue(self.r.decode_cli_stdout(b"\xff"))


class EvalCheckExceptionTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_non_dict_check(self):
        detail = self.r.eval_check("nope", {}, "s", {}, "rhwp")
        self.assertFalse(detail["ok"])
        self.assertEqual(detail["kind"], "malformed-check")

    def test_missing_op(self):
        detail = self.r.eval_check({"name": "x"}, {}, "s", {}, "rhwp")
        self.assertEqual(detail["kind"], "malformed-check")
        self.assertEqual(detail["error"], "op 없음")

    def test_unknown_op_keeps_legacy_message(self):
        detail = self.r.eval_check({"op": "nope_op"}, {}, "s", {}, "rhwp")
        self.assertEqual(detail["error"], "미지 op: nope_op")
        self.assertEqual(detail["kind"], "unknown-op")

    def test_malformed_cmd_on_cli_op(self):
        check = {"name": "n", "op": "value_eq", "cmd": "info", "path": "x", "value": 1}
        detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertEqual(detail["kind"], "malformed-cmd")
        self.assertFalse(detail["ok"])

    def test_bad_expect_exits_kind(self):
        check = {
            "name": "n", "op": "value_eq", "cmd": ["info"],
            "path": "x", "value": 1, "expect_exits": "0",
        }
        with mock.patch.object(self.r, "run_cli", return_value=(0, {"x": 1}, "")):
            detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertEqual(detail["kind"], "bad-expect-exits")
        self.assertIn("0", detail["error"])

    def test_cli_exit_kind(self):
        check = {
            "name": "n", "op": "value_eq", "cmd": ["info"],
            "path": "x", "value": 1, "expect_exits": [0],
        }
        with mock.patch.object(self.r, "run_cli", return_value=(2, None, "boom")):
            detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertEqual(detail["kind"], "cli-exit")
        self.assertIn("exit 2", detail["error"])

    def test_envelope_parse_kind(self):
        check = {
            "name": "n", "op": "value_eq", "cmd": ["info"],
            "path": "x", "value": 1,
        }
        with mock.patch.object(self.r, "run_cli", return_value=(0, None, "nope")):
            detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertEqual(detail["kind"], "envelope-parse")
        self.assertIn("봉투 파싱 실패", detail["error"])

    def test_file_not_found_keeps_legacy_prefix(self):
        check = {
            "name": "n", "op": "value_eq", "cmd": ["info"],
            "path": "x", "value": 1,
        }
        with mock.patch.object(self.r, "run_cli", side_effect=FileNotFoundError("rhwp")):
            detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertTrue(detail["error"].startswith("파일 없음:"))
        self.assertEqual(detail["kind"], "missing-bin")

    def test_permission_kind(self):
        check = {
            "name": "n", "op": "value_eq", "cmd": ["info"],
            "path": "x", "value": 1,
        }
        with mock.patch.object(self.r, "run_cli", side_effect=PermissionError("no")):
            detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertEqual(detail["kind"], "permission")
        self.assertTrue(detail["error"].startswith("권한 없음:"))

    def test_score_runner_error_from_resolve(self):
        check = {
            "name": "n", "op": "value_eq", "cmd": ["info", "{input}"],
            "path": "x", "value": 1,
        }
        detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertEqual(detail["kind"], "missing-input")

    def test_path_eval_legacy_message(self):
        check = {
            "name": "n", "op": "value_eq", "cmd": ["info"],
            "path": "missing", "value": 1,
        }
        with mock.patch.object(self.r, "run_cli", return_value=(0, {}, "")):
            detail = self.r.eval_check(check, {}, "s", {}, "rhwp")
        self.assertFalse(detail["ok"])
        self.assertTrue(detail["error"].startswith("경로 평가 실패:"))
        self.assertEqual(detail["kind"], "path-eval")


class ValidateExpectExitsTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_none_uses_fallback(self):
        values, err = self.r.validate_expect_exits(None, 0)
        self.assertEqual(values, [0])
        self.assertIsNone(err)

    def test_rejects_empty_and_non_int(self):
        values, err = self.r.validate_expect_exits([], 0)
        self.assertIsNone(values)
        self.assertIn("expect_exits", err)
        values, err = self.r.validate_expect_exits([0, "3"], 0)
        self.assertIsNone(values)
        values, err = self.r.validate_expect_exits(0, 0)
        self.assertIsNone(values)

    def test_accepts_int_list(self):
        values, err = self.r.validate_expect_exits([0, 3], 0)
        self.assertEqual(values, [0, 3])
        self.assertIsNone(err)

    def test_bool_is_not_int(self):
        # bool 은 int 의 하위라 type(v) is not int 로 막는다.
        values, err = self.r.validate_expect_exits([True], 0)
        self.assertIsNone(values)


class ScoreTaskExceptionTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_non_dict_task(self):
        result = self.r.score_task("nope", "s", "rhwp")
        self.assertFalse(result["pass"])
        self.assertEqual(result["kind"], "malformed-task")

    def test_missing_keys(self):
        result = self.r.score_task({"id": "T"}, "s", "rhwp")
        self.assertEqual(result["kind"], "malformed-task")
        self.assertIn("필수 키", result["error"])

    def test_missing_submit_keeps_legacy_message(self):
        with tempfile.TemporaryDirectory() as d:
            result = self.r.score_task(_task(), d, "rhwp")
        self.assertEqual(result["error"], "제출 폴더 없음")
        self.assertEqual(result["kind"], "missing-submit")
        self.assertFalse(result["pass"])

    def test_malformed_answer_json(self):
        with tempfile.TemporaryDirectory() as d:
            sub = Path(d) / "T01"
            sub.mkdir()
            (sub / "answer.json").write_text("{", encoding="utf-8")
            result = self.r.score_task(_task(), d, "rhwp")
        self.assertEqual(result["kind"], "malformed-answer")
        self.assertIn("파싱 실패", result["error"])

    def test_answer_must_be_object(self):
        with tempfile.TemporaryDirectory() as d:
            sub = Path(d) / "T01"
            sub.mkdir()
            (sub / "answer.json").write_text("[1,2]", encoding="utf-8")
            result = self.r.score_task(_task(), d, "rhwp")
        self.assertEqual(result["kind"], "malformed-answer")
        self.assertIn("객체가 아니다", result["error"])

    def test_empty_checks(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "T01").mkdir()
            result = self.r.score_task(_task(checks=[]), d, "rhwp")
        self.assertEqual(result["kind"], "empty-checks")
        self.assertFalse(result["pass"])

    def test_passing_file_exists(self):
        with tempfile.TemporaryDirectory() as d:
            sub = Path(d) / "T01"
            sub.mkdir()
            (sub / "answer.json").write_text("{}", encoding="utf-8")
            result = self.r.score_task(_task(), d, "rhwp")
        self.assertTrue(result["pass"], result)

    def test_non_int_tier(self):
        task = _task()
        task["tier"] = "1"
        result = self.r.score_task(task, "s", "rhwp")
        self.assertEqual(result["kind"], "malformed-task")


class LoadPackTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_unsafe_pack_id(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.load_pack("../etc")
        self.assertEqual(ctx.exception.kind, "unsafe-id")

    def test_missing_pack(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.load_pack("no-such-pack-zzzz")
        self.assertEqual(ctx.exception.kind, "missing-pack")

    def test_malformed_pack_json(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                os.makedirs(os.path.join(d, "p1"))
                _write(os.path.join(d, "p1", "pack.json"), "{")
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_pack("p1")
        self.assertEqual(ctx.exception.kind, "malformed-pack")

    def test_pack_not_object(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                os.makedirs(os.path.join(d, "p1"))
                _write(os.path.join(d, "p1", "pack.json"), [1])
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_pack("p1")
        self.assertEqual(ctx.exception.kind, "malformed-pack")

    def test_missing_title_and_tasks(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "p1", [_task()], manifest={"title": ""})
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_pack("p1")
                self.assertEqual(ctx.exception.kind, "malformed-pack")
                _plant_pack(d, "p2", [_task()], manifest={"title": "ok", "axis": ""})
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_pack("p2")
                self.assertEqual(ctx.exception.kind, "malformed-pack")
                pack = os.path.join(d, "p3")
                os.makedirs(pack)
                _write(os.path.join(pack, "pack.json"), {
                    "title": "t", "axis": "a", "id": "p3",
                })
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_pack("p3")
                self.assertEqual(ctx.exception.kind, "missing-tasks-dir")

    def test_malformed_task_json(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "p1", [_task()])
                _write(os.path.join(d, "p1", "tasks", "T99.json"), "{")
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_pack("p1")
        self.assertEqual(ctx.exception.kind, "malformed-task")

    def test_task_not_object(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "p1", [_task()])
                _write(os.path.join(d, "p1", "tasks", "T99.json"), [1])
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_pack("p1")
        self.assertEqual(ctx.exception.kind, "malformed-task")

    def test_happy_load_sorted(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "p1", [_task("T02"), _task("T01")])
                manifest, tasks = self.r.load_pack("p1")
        self.assertEqual(manifest["title"], "p1")
        self.assertEqual([t["id"] for t in tasks], ["T01", "T02"])


class DiscoverAndProfileTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_discover_skips_unsafe_and_files(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "good-pack", [_task()])
                os.makedirs(os.path.join(d, "..sneaky"), exist_ok=True)
                with open(os.path.join(d, "readme.txt"), "w", encoding="utf-8") as fh:
                    fh.write("x")
                found = self.r.discover_packs()
        self.assertEqual(found, ["good-pack"])

    def test_discover_missing_dir(self):
        with mock.patch.object(self.r, "PACKS_DIR", os.path.join("no", "packs")):
            self.assertEqual(self.r.discover_packs(), [])

    def test_missing_profile(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.load_profile("no-such-profile-zzzz")
        self.assertEqual(ctx.exception.kind, "missing-profile")

    def test_unsafe_profile(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.load_profile("../x")
        self.assertEqual(ctx.exception.kind, "unsafe-id")

    def test_profile_not_object_and_empty_packs(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PROFILES_DIR", d):
                _write(os.path.join(d, "p.json"), [1])
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_profile("p")
                self.assertEqual(ctx.exception.kind, "malformed-profile")
                _write(os.path.join(d, "q.json"), {"packs": []})
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_profile("q")
                self.assertEqual(ctx.exception.kind, "malformed-profile")
                _write(os.path.join(d, "r.json"), {"packs": ["../x"]})
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_profile("r")
                self.assertEqual(ctx.exception.kind, "unsafe-id")

    def test_profile_ok(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PROFILES_DIR", d):
                _write(os.path.join(d, "family.json"), {"packs": ["casual-rides"]})
                profile = self.r.load_profile("family")
        self.assertEqual(profile["packs"], ["casual-rides"])

    def test_malformed_profile_json(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PROFILES_DIR", d):
                _write(os.path.join(d, "z.json"), "{")
                with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                    self.r.load_profile("z")
        self.assertEqual(ctx.exception.kind, "malformed-profile")


class ScorePackTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_unavailable_is_not_zero(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "p1", [_task(tier=2)])
                entry = self.r.score_pack("p1", d, "rhwp", available=set())
        self.assertEqual(entry["status"], "unavailable")
        self.assertIsNone(entry["score"])
        self.assertIn("info", entry["missingCommands"])

    def test_error_status_on_missing_pack(self):
        with mock.patch.object(self.r, "PACKS_DIR", tempfile.gettempdir()):
            entry = self.r.score_pack("no-such-pack-zzzz", "s", "rhwp", None)
        self.assertEqual(entry["status"], "error")
        self.assertEqual(entry["kind"], "missing-pack")
        self.assertIsNone(entry["score"])

    def test_scored_counts_pass_only(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "p1", [_task("T01", tier=2), _task("T02", tier=3)])
                sub = Path(d) / "agent"
                (sub / "T01").mkdir(parents=True)
                (sub / "T01" / "answer.json").write_text("{}", encoding="utf-8")
                entry = self.r.score_pack("p1", str(sub), "rhwp", None)
        self.assertEqual(entry["status"], "scored")
        self.assertEqual(entry["score"], 2)
        self.assertEqual(entry["passed"], 1)
        self.assertEqual(entry["max"], 5)

    def test_pack_subdir_preferred(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PACKS_DIR", d):
                _plant_pack(d, "p1", [_task()])
                nested = Path(d) / "agent" / "p1" / "T01"
                nested.mkdir(parents=True)
                (nested / "answer.json").write_text("{}", encoding="utf-8")
                flat = Path(d) / "agent" / "T01"
                flat.mkdir(parents=True)
                entry = self.r.score_pack("p1", str(Path(d) / "agent"), "rhwp", None)
        self.assertTrue(entry["tasks"][0]["pass"], entry)

    def test_unsafe_pack_is_error_not_crash(self):
        entry = self.r.score_pack("../x", "s", "rhwp", None)
        self.assertEqual(entry["status"], "error")
        self.assertEqual(entry["kind"], "unsafe-id")


class ScoreAllTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def _card(self, d, **kwargs):
        with mock.patch.object(self.r, "PACKS_DIR", d):
            with mock.patch.object(self.r, "safe_known_commands", return_value=None):
                with mock.patch.object(self.r, "safe_runner_identity",
                                       return_value={"rhwpVersion": "0",
                                                     "rhwpCommit": "c" * 40,
                                                     "capabilitiesSha256": "a" * 64}):
                    return self.r.score_all(os.path.join(d, "sub"), "rhwp", **kwargs)

    def test_error_pack_not_counted_as_unavailable(self):
        with tempfile.TemporaryDirectory() as d:
            _plant_pack(d, "p1", [_task()])
            card = self._card(d, pack_ids=["p1", "no-such-pack-zzzz"])
        self.assertEqual(card["total"]["packsScored"], 1)
        self.assertEqual(card["total"]["packsUnavailable"], 0)
        self.assertEqual(card["total"]["packsErrored"], 1)
        self.assertFalse(card["trusted"])

    def test_bad_profile_returns_empty_card(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(self.r, "PROFILES_DIR", d):
                card = self._card(d, profile_id="missing")
        self.assertEqual(card["total"]["packsScored"], 0)
        self.assertEqual(card["exceptions"][0]["kind"], "missing-profile")

    def test_unsafe_pack_ids_empty_card(self):
        with tempfile.TemporaryDirectory() as d:
            card = self._card(d, pack_ids=["../x"])
        self.assertEqual(card["exceptions"][0]["kind"], "unsafe-id")
        self.assertEqual(card["packs"], [])

    def test_profile_selects_packs(self):
        with tempfile.TemporaryDirectory() as d:
            _plant_pack(d, "p1", [_task()])
            _plant_pack(d, "p2", [_task("T02")])
            with mock.patch.object(self.r, "PROFILES_DIR", d):
                _write(os.path.join(d, "ed.json"), {"packs": ["p1"]})
                with mock.patch.object(self.r, "PACKS_DIR", d):
                    with mock.patch.object(self.r, "safe_known_commands", return_value=None):
                        with mock.patch.object(self.r, "safe_runner_identity",
                                               return_value={"rhwpVersion": "",
                                                             "rhwpCommit": "",
                                                             "capabilitiesSha256": ""}):
                            card = self.r.score_all(os.path.join(d, "sub"), "rhwp",
                                                    profile_id="ed")
        self.assertEqual([p["id"] for p in card["packs"]], ["p1"])

    def test_missing_bin_recorded_but_scoring_continues(self):
        with tempfile.TemporaryDirectory() as d:
            _plant_pack(d, "p1", [_task()])
            missing = os.path.join(d, "no-rhwp")
            with mock.patch.object(self.r, "PACKS_DIR", d):
                with mock.patch.object(self.r, "safe_known_commands", return_value=None):
                    with mock.patch.object(self.r, "safe_runner_identity",
                                           return_value={"rhwpVersion": "",
                                                         "rhwpCommit": "",
                                                         "capabilitiesSha256": ""}):
                        card = self.r.score_all(os.path.join(d, "sub"), missing,
                                                pack_ids=["p1"])
        self.assertTrue(card["binMissing"])
        self.assertEqual(card["exceptions"][0]["kind"], "missing-bin")
        self.assertEqual(len(card["packs"]), 1)

    def test_discover_when_no_pack_ids(self):
        with tempfile.TemporaryDirectory() as d:
            _plant_pack(d, "alpha", [_task()])
            card = self._card(d)
        self.assertEqual([p["id"] for p in card["packs"]], ["alpha"])

    def test_scorecard_validates(self):
        with tempfile.TemporaryDirectory() as d:
            _plant_pack(d, "p1", [_task()])
            card = self._card(d, pack_ids=["p1"])
        self.assertEqual(self.r.validate_scorecard(card), [])

    def test_validate_scorecard_rejects_junk(self):
        self.assertTrue(self.r.validate_scorecard("nope"))
        self.assertTrue(self.r.validate_scorecard({"kind": "x"}))
        self.assertTrue(self.r.validate_scorecard({
            "kind": self.r.REPORT_KIND,
            "schemaVersion": self.r.SCHEMA_VERSION,
            "profile": None,
            "runner": {},
            "total": {},
            "packs": [{"status": "nope"}],
        }))


class AdmissionAndReportTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_allow_requires_one_scored_pack(self):
        card = {"total": {"packsScored": 1, "score": 0, "max": 4}, "runner": {}}
        adm = self.r.admission_from_card(card, "alice")
        self.assertEqual(adm["verdict"], "allow")
        self.assertEqual(adm["agent"], "alice")

    def test_deny_when_nothing_scored(self):
        adm = self.r.admission_from_card({"total": {"packsScored": 0}}, "bob")
        self.assertEqual(adm["verdict"], "deny")

    def test_admission_from_broken_card(self):
        adm = self.r.admission_from_card(None, "z")
        self.assertEqual(adm["verdict"], "deny")
        self.assertEqual(adm["kind"], "gymAdmission")

    def test_render_report_includes_error_and_exceptions(self):
        card = {
            "total": {"score": 0, "max": 1, "packsScored": 0,
                      "packsUnavailable": 1, "packsErrored": 1},
            "runner": {"rhwpVersion": "1", "rhwpCommit": "c" * 40,
                       "capabilitiesSha256": "a" * 64},
            "packs": [
                {"id": "u", "axis": "a", "status": "unavailable",
                 "missingCommands": ["info"]},
                {"id": "e", "axis": "a", "status": "error",
                 "error": "boom", "kind": "missing-pack"},
                {"id": "s", "axis": "a", "status": "scored", "title": "S",
                 "score": 0, "max": 1, "passed": 0, "taskCount": 1,
                 "tasks": [{"id": "T", "title": "t", "tier": 1, "pass": False,
                            "error": "제출 폴더 없음"}]},
            ],
            "exceptions": [{"kind": "missing-bin", "where": "bin", "message": "x"}],
        }
        text = self.r.render_report(card, "agent")
        self.assertIn("unavailable", text)
        self.assertIn("error", text)
        self.assertIn("제출 폴더 없음", text)
        self.assertIn("missing-bin", text)
        self.assertIn("짐 스코어카드", text)

    def test_render_report_non_dict(self):
        text = self.r.render_report(None, "a")
        self.assertIn("객체가 아니다", text)

    def test_console_summary(self):
        card = {
            "total": {"score": 1, "max": 2, "packsScored": 1,
                      "packsUnavailable": 1, "packsErrored": 1},
            "packs": [
                {"id": "p", "status": "scored", "score": 1, "max": 2,
                 "passed": 1, "taskCount": 2},
                {"id": "u", "status": "unavailable", "missingCommands": ["x"]},
                {"id": "e", "status": "error", "kind": "missing-pack",
                 "error": "no"},
            ],
        }
        text = self.r.format_console_summary(card, "ag", "out.json")
        self.assertIn("unavailable", text)
        self.assertIn("error", text)
        self.assertIn("ag:", text)

    def test_exit_from_card(self):
        self.assertEqual(self.r.exit_from_card({
            "total": {"score": 2, "max": 2, "packsScored": 1, "packsErrored": 0},
        }), 0)
        self.assertEqual(self.r.exit_from_card({
            "total": {"score": 1, "max": 2, "packsScored": 1, "packsErrored": 0},
        }), 3)
        self.assertEqual(self.r.exit_from_card({
            "total": {"score": 0, "max": 0, "packsScored": 0, "packsErrored": 1},
        }), 3)
        self.assertEqual(self.r.exit_from_card(None), 3)

    def test_empty_scorecard_validates_after_attach(self):
        card = self.r.empty_scorecard()
        self.r.attach_card_counts(card)
        self.assertEqual(self.r.validate_scorecard(card), [])
        self.assertTrue(card["trusted"])


class SafeIdentityTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_known_commands_swallows_file_not_found(self):
        with mock.patch.object(self.r.pack_schema, "known_commands",
                               side_effect=FileNotFoundError("x")):
            self.assertIsNone(self.r.safe_known_commands("nope"))

    def test_runner_identity_swallows(self):
        with mock.patch.object(self.r.pack_schema, "runner_identity",
                               side_effect=FileNotFoundError("x")):
            ident = self.r.safe_runner_identity("nope")
        self.assertEqual(ident["kind"], "missing-bin")
        self.assertEqual(ident["rhwpCommit"], "")

    def test_runner_identity_non_dict(self):
        with mock.patch.object(self.r.pack_schema, "runner_identity", return_value=None):
            ident = self.r.safe_runner_identity("x")
        self.assertEqual(ident["rhwpVersion"], "")


class NormalizePackIdsTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_none_and_string(self):
        self.assertIsNone(self.r.normalize_pack_ids(None))
        self.assertEqual(self.r.normalize_pack_ids("core-cli"), ["core-cli"])

    def test_rejects_bad_types(self):
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.normalize_pack_ids(1)
        self.assertEqual(ctx.exception.kind, "value-error")
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.r.normalize_pack_ids([""])
        self.assertEqual(ctx.exception.kind, "unsafe-id")


class ScoreEntryTests(unittest.TestCase):
    def setUp(self):
        self.mod = load_score()
        self.r = load_runner()

    def test_parser_flags_are_unchanged(self):
        ap = self.mod.build_parser()
        actions = {a.dest for a in ap._actions if a.dest != "help"}
        self.assertEqual(actions, {"agent", "submissions", "bin", "out", "pack", "profile"})

    def test_normalize_agent(self):
        self.assertEqual(self.mod.normalize_agent("  alice  "), "alice")
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.mod.normalize_agent("   ")
        self.assertEqual(ctx.exception.kind, "empty-agent")
        with self.assertRaises(self.r.ScoreRunnerError) as ctx:
            self.mod.normalize_agent("a/b")
        self.assertEqual(ctx.exception.kind, "unsafe-id")

    def test_write_artifacts_and_admission(self):
        with tempfile.TemporaryDirectory() as d:
            card = self.r.empty_scorecard()
            card["agent"] = "alice"
            self.r.attach_card_counts(card)
            art = self.mod.write_score_artifacts(d, card, "alice")
            self.assertEqual(art["admission"]["verdict"], "deny")
            self.assertTrue(os.path.isfile(os.path.join(d, "scorecard.json")))
            self.assertTrue(os.path.isfile(os.path.join(d, "report.md")))
            self.assertTrue(os.path.isfile(os.path.join(d, "admission.json")))
            self.assertEqual(art["errors"], [])

    def test_run_score_deny_empty_agent_dir(self):
        with tempfile.TemporaryDirectory() as d:
            code, card, art = self.mod.run_score(
                "ghost",
                submissions=os.path.join(d, "sub"),
                out=os.path.join(d, "out"),
                bin_arg=os.path.join(d, "no-bin"),
                pack_ids=["no-such-pack-zzzz"],
            )
        self.assertEqual(code, 3)
        self.assertEqual(card["agent"], "ghost")
        self.assertEqual(art["admission"]["verdict"], "deny")
        self.assertGreaterEqual(card["total"]["packsErrored"], 1)

    def test_main_empty_agent_returns_3(self):
        err = os.devnull
        with mock.patch.object(sys, "stderr", io_like()):
            code = self.mod.main(["--agent", "   "])
        self.assertEqual(code, 3)

    def test_deny_card_records_exception(self):
        card = self.mod.deny_card("a", "rhwp", self.r.ScoreRunnerError("write-error", "boom"))
        self.assertEqual(card["exceptions"][0]["kind"], "write-error")
        self.assertEqual(card["agent"], "a")


def io_like():
    import io
    return io.StringIO()


class RunCliHardeningTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_file_not_found_still_raises(self):
        with self.assertRaises(FileNotFoundError):
            self.r.run_cli(os.path.join("no", "such", "rhwp-bin"), ["info"])

    def test_oserror_becomes_score_error(self):
        with mock.patch("subprocess.run", side_effect=OSError("win 2")):
            with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                self.r.run_cli("rhwp", ["info"])
        self.assertEqual(ctx.exception.kind, "os-error")

    def test_timeout_expired(self):
        import subprocess as sp
        with mock.patch("subprocess.run", side_effect=sp.TimeoutExpired("rhwp", 1)):
            with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                self.r.run_cli("rhwp", ["info"])
        self.assertEqual(ctx.exception.kind, "timeout")

    def test_happy_json_object(self):
        class Proc:
            returncode = 0
            stdout = b'{"ok": true}'
        with mock.patch("subprocess.run", return_value=Proc()):
            code, env, head = self.r.run_cli("rhwp", ["info"])
        self.assertEqual(code, 0)
        self.assertEqual(env, {"ok": True})
        self.assertIn("ok", head)

    def test_non_json_stdout(self):
        class Proc:
            returncode = 0
            stdout = b"hello"
        with mock.patch("subprocess.run", return_value=Proc()):
            code, env, head = self.r.run_cli("rhwp", ["info"])
        self.assertIsNone(env)
        self.assertEqual(head, "hello")


class HonestyMatrixTests(unittest.TestCase):
    """오류를 다른 칸으로 부르면 안 되는 표."""

    def setUp(self):
        self.r = load_runner()

    def test_error_is_not_unavailable(self):
        entry = self.r.error_pack_entry("p", self.r.ScoreRunnerError("missing-pack", "x"))
        self.assertEqual(entry["status"], "error")
        self.assertNotEqual(entry["status"], "unavailable")
        self.assertIsNone(entry["score"])

    def test_missing_submit_is_not_pass(self):
        result = self.r.empty_task_result("T", 1, "t", "제출 폴더 없음", "missing-submit")
        self.assertFalse(result["pass"])

    def test_unknown_op_is_not_path_eval(self):
        detail = self.r.failed_check("n", "nope", "미지 op: nope", "unknown-op")
        self.assertEqual(detail["kind"], "unknown-op")
        self.assertFalse(detail["ok"])

    def test_attach_does_not_count_error_as_unavailable(self):
        card = {
            "packs": [
                {"status": "scored", "score": 1, "max": 2},
                {"status": "unavailable"},
                {"status": "error"},
            ],
            "exceptions": [{"kind": "x"}],
        }
        self.r.attach_card_counts(card)
        self.assertEqual(card["total"]["packsScored"], 1)
        self.assertEqual(card["total"]["packsUnavailable"], 1)
        self.assertEqual(card["total"]["packsErrored"], 1)
        self.assertEqual(card["total"]["score"], 1)
        self.assertEqual(card["total"]["max"], 2)
        self.assertFalse(card["trusted"])


class RealGymSmokeTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_discover_real_packs(self):
        packs = self.r.discover_packs()
        self.assertIn("core-cli", packs)
        self.assertIn("casual-rides", packs)
        for pid in packs:
            self.assertTrue(self.r.is_safe_id(pid), pid)

    def test_load_real_core_cli(self):
        manifest, tasks = self.r.load_pack("core-cli")
        self.assertEqual(manifest["id"], "core-cli")
        ids = [t["id"] for t in tasks]
        self.assertIn("T12", ids)
        self.assertFalse(self.r.task_shape_error(next(t for t in tasks if t["id"] == "T12")))

    def test_load_real_family_profile(self):
        profile = self.r.load_profile("family")
        self.assertIn("casual-rides", profile["packs"])

    def test_score_task_missing_on_real_shape(self):
        _manifest, tasks = self.r.load_pack("core-cli")
        t01 = next(t for t in tasks if t["id"] == "T01")
        with tempfile.TemporaryDirectory() as d:
            result = self.r.score_task(t01, d, "rhwp")
        self.assertEqual(result["error"], "제출 폴더 없음")


class GeneratedCatalogSyncTests(unittest.TestCase):
    """문서가 카탈로그를 빼먹으면 시험이 실패한다."""

    def setUp(self):
        self.r = load_runner()
        self.doc = (REPO_ROOT / "gym" / "docs" / "score_runner.md").read_text(encoding="utf-8")
        self.work = (REPO_ROOT / "mydocs" / "working" / "archives" / "gym_score_runner.md").read_text(
            encoding="utf-8")

    def test_docs_mention_every_kind(self):
        for kind in self.r.EXCEPTION_KINDS:
            self.assertIn(f"`{kind}`", self.doc, kind)

    def test_docs_mention_pack_statuses(self):
        for status in self.r.PACK_STATUSES:
            self.assertIn(status, self.doc)

    def test_docs_forbid_new_cli(self):
        self.assertIn("새 플래그는 없다", self.doc)
        self.assertIn("#5260", self.work)

    def test_working_mentions_verification(self):
        self.assertIn("unittest", self.work)
        self.assertIn("audit.py", self.work)
        self.assertIn("cargo fmt --all", self.work)


class CatchableAndFatalBoundaryTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_is_catchable_false_for_none_and_fatal(self):
        self.assertFalse(self.r.is_catchable_exception(None))
        self.assertFalse(self.r.is_catchable_exception(KeyboardInterrupt()))

    def test_is_catchable_true_for_os_and_json(self):
        self.assertTrue(self.r.is_catchable_exception(OSError("x")))
        self.assertTrue(self.r.is_catchable_exception(json.JSONDecodeError("e", "d", 0)))

    def test_task_detail_line_non_dict(self):
        self.assertIn("객체가 아니다", self.r.task_detail_line("x"))

    def test_check_name_of(self):
        self.assertEqual(self.r.check_name_of({"name": "n", "op": "o"}), "n")
        self.assertEqual(self.r.check_name_of({"op": "o"}), "o")
        self.assertIsNone(self.r.check_name_of("x"))

    def test_task_tier_default(self):
        self.assertEqual(self.r.task_tier({"tier": 3}), 3)
        self.assertEqual(self.r.task_tier({"tier": "3"}), 0)
        self.assertEqual(self.r.task_tier(None), 0)

    def test_unsafe_id_reason_types(self):
        self.assertIn("문자열", self.r.unsafe_id_reason(1, "pack"))
        self.assertIn("비었다", self.r.unsafe_id_reason("", "pack"))
        self.assertEqual(self.r.unsafe_id_reason("ok-id", "pack"), "")


class ScoreTaskCheckKindPropagationTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_unknown_op_in_task_does_not_crash(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "T01").mkdir()
            result = self.r.score_task(
                _task(checks=[{"name": "n", "op": "nope"}]), d, "rhwp")
        self.assertFalse(result["pass"])
        self.assertEqual(result["checks"][0]["kind"], "unknown-op")

    def test_non_dict_check_in_task(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "T01").mkdir()
            result = self.r.score_task(_task(checks=["bad"]), d, "rhwp")
        self.assertEqual(result["checks"][0]["kind"], "malformed-check")
        self.assertFalse(result["pass"])


class ScoreAllStringPackIdTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_single_string_pack_id(self):
        with tempfile.TemporaryDirectory() as d:
            _plant_pack(d, "solo", [_task()])
            with mock.patch.object(self.r, "PACKS_DIR", d):
                with mock.patch.object(self.r, "safe_known_commands", return_value=None):
                    with mock.patch.object(self.r, "safe_runner_identity",
                                           return_value={"rhwpVersion": "",
                                                         "rhwpCommit": "",
                                                         "capabilitiesSha256": ""}):
                        card = self.r.score_all(os.path.join(d, "s"), "rhwp",
                                                pack_ids="solo")
        self.assertEqual(card["packs"][0]["id"], "solo")


class WriteErrorPathTests(unittest.TestCase):
    def setUp(self):
        self.mod = load_score()
        self.r = load_runner()

    def test_write_artifacts_records_oserror(self):
        card = self.r.empty_scorecard()
        with mock.patch.object(self.mod, "dump_json", side_effect=OSError("disk")):
            with tempfile.TemporaryDirectory() as d:
                with mock.patch.object(self.mod, "write_text", side_effect=OSError("disk")):
                    art = self.mod.write_score_artifacts(d, card, "a")
        kinds = [row["kind"] for row in art["errors"]]
        self.assertIn("write-error", kinds)
        self.assertGreaterEqual(len(art["errors"]), 2)

    def test_ensure_out_dir_raises(self):
        with mock.patch("os.makedirs", side_effect=OSError("nope")):
            with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                self.mod.ensure_out_dir("/no/out")
        self.assertEqual(ctx.exception.kind, "write-error")


class ScoreModuleMainMoreTests(unittest.TestCase):
    def setUp(self):
        self.mod = load_score()

    def test_main_missing_agent_exits_argparse(self):
        with self.assertRaises(SystemExit) as ctx:
            self.mod.main([])
        self.assertEqual(ctx.exception.code, 2)

    def test_resolve_paths_defaults(self):
        sub, out = self.mod.resolve_paths("alice", None, None)
        self.assertTrue(sub.endswith(os.path.join("submissions", "alice")))
        self.assertEqual(sub, out)
        sub, out = self.mod.resolve_paths("alice", "S", "O")
        self.assertEqual((sub, out), ("S", "O"))


class AttachCardEdgeTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()

    def test_broken_packs_field(self):
        card = {"packs": "nope", "exceptions": None}
        self.r.attach_card_counts(card)
        self.assertEqual(card["total"]["packsScored"], 0)

    def test_score_none_treated_as_zero(self):
        card = {"packs": [{"status": "scored", "score": None, "max": None}]}
        self.r.attach_card_counts(card)
        self.assertEqual(card["total"]["score"], 0)
        self.assertEqual(card["total"]["max"], 0)


class DocsExistTests(unittest.TestCase):
    def test_guide_front_matter(self):
        text = (REPO_ROOT / "gym" / "docs" / "score_runner.md").read_text(encoding="utf-8")
        self.assertIn("canonical: gym/docs/score_runner.md", text)
        self.assertIn("kind: guide", text)

    def test_working_front_matter(self):
        text = (REPO_ROOT / "mydocs" / "working" / "archives" / "gym_score_runner.md").read_text(
            encoding="utf-8")
        self.assertIn("canonical: mydocs/working/gym_score_runner.md", text)
        self.assertIn("kind: working", text)
        self.assertIn("feat/gym-score-runner-hardening", text)

    def test_guide_lists_implementation_map(self):
        text = (REPO_ROOT / "gym" / "docs" / "score_runner.md").read_text(encoding="utf-8")
        for name in ("find_bin", "eval_check", "score_task", "score_all",
                     "admission_from_card", "write_score_artifacts"):
            self.assertIn(f"`{name}`", text, name)

    def test_working_records_before_after(self):
        text = (REPO_ROOT / "mydocs" / "working" / "archives" / "gym_score_runner.md").read_text(
            encoding="utf-8")
        self.assertIn("answer.json 배열", text)
        self.assertIn("경로형 바이너리 부재", text)
        self.assertIn("5210", text)


class CheckContextAndDumpTests(unittest.TestCase):
    def setUp(self):
        self.r = load_runner()
        self.mod = load_score()

    def test_check_context_paths(self):
        ctx = self.r.CheckContext(
            {"path": "a"}, {"input": "samples/x.hwp"}, "sub", {}, {"a": 1})
        self.assertEqual(ctx.sub_path("o.hwp"), os.path.join("sub", "o.hwp"))
        self.assertEqual(ctx.root_path("samples/x.hwp"),
                         os.path.join(self.r.ROOT, "samples/x.hwp"))
        self.assertEqual(ctx.dug(), 1)

    def test_dump_json_roundtrip(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "c.json")
            self.mod.dump_json(path, {"k": "한글"})
            body = json.loads(Path(path).read_text(encoding="utf-8"))
            raw = Path(path).read_bytes()
        self.assertEqual(body["k"], "한글")
        self.assertFalse(raw.startswith(b"\xef\xbb\xbf"))

    def test_write_text_lf(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "r.md")
            self.mod.write_text(path, "a\nb\n")
            raw = Path(path).read_bytes()
        self.assertEqual(raw, b"a\nb\n")

    def test_rhwp_bin_env(self):
        with tempfile.TemporaryDirectory() as d:
            target = os.path.join(d, "env-rhwp")
            Path(target).write_bytes(b"x")
            with mock.patch.dict(os.environ, {"RHWP_BIN": target}):
                self.assertEqual(self.r.find_bin(None), target)

    def test_failed_check_unknown_kind_falls_back(self):
        detail = self.r.failed_check("n", "o", "e", "not-a-kind")
        self.assertEqual(detail["kind"], "unexpected")

    def test_score_runner_error_unknown_kind_falls_back(self):
        err = self.r.ScoreRunnerError("nope", "m")
        self.assertEqual(err.kind, "unexpected")
        self.assertEqual(err.as_row("w")["where"], "w")

    def test_load_json_object_rejects_array(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "a.json")
            Path(path).write_text("[1]", encoding="utf-8")
            with self.assertRaises(self.r.ScoreRunnerError) as ctx:
                self.r.load_json_object(path, "malformed-pack")
        self.assertEqual(ctx.exception.kind, "malformed-pack")

    def test_subprocess_error_kind(self):
        import subprocess as sp
        self.assertEqual(
            self.r.exception_kind(sp.SubprocessError("x"), "bin"),
            "subprocess",
        )

    def test_main_run_score_writes_files(self):
        with tempfile.TemporaryDirectory() as d:
            packs = os.path.join(d, "packs")
            _plant_pack(packs, "solo", [_task()])
            out = os.path.join(d, "out")
            with mock.patch.object(self.r, "PACKS_DIR", packs):
                with mock.patch.object(self.r, "safe_known_commands", return_value=None):
                    with mock.patch.object(self.r, "safe_runner_identity",
                                           return_value={"rhwpVersion": "",
                                                         "rhwpCommit": "",
                                                         "capabilitiesSha256": ""}):
                        code, card, art = self.mod.run_score(
                            "alice",
                            submissions=os.path.join(d, "sub"),
                            out=out,
                            bin_arg=os.path.join(d, "no-bin"),
                            pack_ids=["solo"],
                        )
            self.assertEqual(code, 3)
            self.assertTrue(os.path.isfile(os.path.join(out, "scorecard.json")))
            self.assertEqual(art["admission"]["agent"], "alice")
            self.assertEqual(card["packs"][0]["id"], "solo")


if __name__ == "__main__":
    unittest.main()
