"""[#5205] gym 지목 채점 연산자 — 파일 픽스처만으로 판정한다.

에이전트 산출(JSON/CSV/NDJSON/텍스트)을 전역 훑기 없이 좌표로 잰다.
새 연산자는 CLI 를 부르지 않으므로 바이너리 없이 돈다.
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

NEW_OPS = (
    "json_len_eq",
    "csv_row_count_eq",
    "ndjson_count_eq",
    "ndjson_field_eq",
    "json_keys_contain",
    "text_line_eq",
)

EXTRA_OPS = (
    "json_type_eq",
    "json_len_ge",
    "json_array_item_eq",
    "csv_col_count_eq",
    "csv_header_eq",
    "csv_row_eq",
    "ndjson_keys_contain",
    "ndjson_len_eq",
    "text_line_count_eq",
    "text_line_contains",
)

PINPOINT_FILE_OPS = NEW_OPS + EXTRA_OPS

EXISTING_FILE_OPS = (
    "same_hash",
    "differs_from_input",
    "file_exists",
    "files_differ",
    "xml_root_eq",
    "json_value_eq",
    "csv_cell_eq",
    "utf8_bom",
)


def load_core():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    from gym.core import checks, runner, schema  # noqa: WPS433

    return checks, runner, schema


def _write(root, name, content, *, encoding="utf-8", newline="\n"):
    path = Path(root) / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(content, bytes):
        path.write_bytes(content)
        return path
    with path.open("w", encoding=encoding, newline=newline) as fh:
        fh.write(content)
    return path


class _OpCase(unittest.TestCase):
    """제출 폴더에 픽스처를 두고 eval_check 로 연산자를 호출한다."""

    def eval_files(self, files, check, encoding="utf-8"):
        _checks, runner, _schema = load_core()
        with tempfile.TemporaryDirectory() as sub_dir:
            for name, content in files.items():
                _write(sub_dir, name, content, encoding=encoding)
            return runner.eval_check(check, {}, sub_dir, {}, "unused-rhwp")


class RegistryContractTests(unittest.TestCase):
    def test_new_ops_are_registered_without_cli(self):
        checks, _runner, _schema = load_core()
        for op in PINPOINT_FILE_OPS:
            self.assertIn(op, checks.REGISTRY, op)
            self.assertFalse(checks.needs_cli(op), op)

    def test_registry_keeps_existing_keys(self):
        checks, _runner, _schema = load_core()
        required = EXISTING_FILE_OPS + NEW_OPS + (
            "answer_eq", "len_answer_eq", "len_ge", "value_eq", "value_ge",
            "value_in", "deep_contains", "not_contains", "cell_text_eq",
        )
        for op in required:
            self.assertIn(op, checks.REGISTRY, op)

    def test_existing_file_ops_still_skip_cli(self):
        checks, _runner, _schema = load_core()
        for op in EXISTING_FILE_OPS:
            self.assertIn(op, checks.REGISTRY, op)
            self.assertFalse(checks.needs_cli(op), op)

    def test_global_scan_ops_unchanged(self):
        checks, _runner, _schema = load_core()
        self.assertEqual(checks.GLOBAL_SCAN_OPS, {"deep_contains", "not_contains"})
        for op in PINPOINT_FILE_OPS:
            self.assertNotIn(op, checks.GLOBAL_SCAN_OPS)

    def test_cli_ops_still_need_cli(self):
        checks, _runner, _schema = load_core()
        self.assertTrue(checks.needs_cli("value_eq"))
        self.assertTrue(checks.needs_cli("cell_text_eq"))


class SchemaAcceptanceTests(unittest.TestCase):
    def _task(self, check):
        return {
            "id": "X",
            "tier": 2,
            "title": "t",
            "input": "samples/x.hwp",
            "instructions": "i",
            "submit": {"kind": "artifact"},
            "checks": [check],
        }

    def _sample_check(self, op):
        samples = {
            "json_len_eq": {"name": "c", "op": op, "file": "out.json", "value": 1},
            "csv_row_count_eq": {"name": "c", "op": op, "file": "out.csv", "value": 1},
            "ndjson_count_eq": {"name": "c", "op": op, "file": "out.ndjson", "value": 1},
            "ndjson_field_eq": {"name": "c", "op": op, "file": "out.ndjson", "row": 0,
                                "path": "id", "value": 1},
            "json_keys_contain": {"name": "c", "op": op, "file": "out.json", "keys": ["a"]},
            "text_line_eq": {"name": "c", "op": op, "file": "out.txt", "line": 0, "value": "x"},
            "json_type_eq": {"name": "c", "op": op, "file": "out.json", "path": "a",
                             "value": "string"},
            "json_len_ge": {"name": "c", "op": op, "file": "out.json", "path": "items",
                            "value": 1},
            "json_array_item_eq": {"name": "c", "op": op, "file": "out.json", "path": "items",
                                   "index": 0, "value": 1},
            "csv_col_count_eq": {"name": "c", "op": op, "file": "out.csv", "row": 0, "value": 2},
            "csv_header_eq": {"name": "c", "op": op, "file": "out.csv",
                              "values": ["name", "qty"]},
            "csv_row_eq": {"name": "c", "op": op, "file": "out.csv", "row": 1,
                           "values": ["갑", "1"]},
            "ndjson_keys_contain": {"name": "c", "op": op, "file": "out.ndjson", "row": 0,
                                    "keys": ["id"]},
            "ndjson_len_eq": {"name": "c", "op": op, "file": "out.ndjson", "row": 0,
                              "path": "tags", "value": 2},
            "text_line_count_eq": {"name": "c", "op": op, "file": "out.txt", "value": 3},
            "text_line_contains": {"name": "c", "op": op, "file": "out.txt", "line": 0,
                                   "value": "갑"},
        }
        return samples[op]

    def test_new_file_ops_do_not_require_cmd(self):
        _checks, _runner, schema = load_core()
        pack = {"id": "p", "axis": "편집 (좌표 지정)"}
        for op in PINPOINT_FILE_OPS:
            errors = []
            schema.validate_task(self._task(self._sample_check(op)), pack, None, errors)
            self.assertEqual(errors, [], f"{op}: {errors}")

    def test_cmd_on_file_op_is_rejected(self):
        _checks, _runner, schema = load_core()
        pack = {"id": "p", "axis": "자동화"}
        check = {"name": "c", "op": "json_len_eq", "file": "out.json",
                 "path": "items", "value": 1, "cmd": ["info"]}
        errors = []
        schema.validate_task(self._task(check), pack, None, errors)
        self.assertTrue(any("CLI" in e for e in errors), errors)


class JsonLenEqTests(_OpCase):
    FILE = "out.json"

    def _check(self, **kwargs):
        check = {"name": "len", "op": "json_len_eq", "file": self.FILE, "value": 3}
        check.update(kwargs)
        return check

    def test_array_length_at_path(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"items": [1, 2, 3], "extra": []})},
            self._check(path="items", value=3),
        )
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 3)

    def test_object_length_at_root(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"a": 1, "b": 2})},
            self._check(path="", value=2),
        )
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 2)

    def test_numeric_string_value_matches(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"items": [1, 2]})},
            self._check(path="items", value="2"),
        )
        self.assertTrue(detail["ok"], detail)

    def test_wrong_length_fails(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"items": [1]})},
            self._check(path="items", value=3),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], 1)

    def test_missing_path_fails(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"items": [1, 2, 3]})},
            self._check(path="missing", value=0),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertIn("실패", str(detail["actual"]))

    def test_bad_json_fails(self):
        detail = self.eval_files({self.FILE: "{not json"}, self._check())
        self.assertFalse(detail["ok"], detail)
        self.assertIn("실패", str(detail["actual"]))

    def test_empty_file_fails(self):
        detail = self.eval_files({self.FILE: ""}, self._check())
        self.assertFalse(detail["ok"], detail)

    def test_scalar_at_path_fails(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"n": 7})},
            self._check(path="n", value=1),
        )
        self.assertFalse(detail["ok"], detail)

    def test_missing_file_fails(self):
        detail = self.eval_files({}, self._check())
        self.assertFalse(detail["ok"], detail)


class CsvRowCountEqTests(_OpCase):
    FILE = "out.csv"

    def _check(self, value):
        return {"name": "rows", "op": "csv_row_count_eq", "file": self.FILE, "value": value}

    def test_counts_header_and_data_rows(self):
        body = "name,qty\n갑,1\n을,2\n"
        detail = self.eval_files({self.FILE: body}, self._check(3))
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 3)

    def test_utf8_sig_bom_does_not_add_a_row(self):
        body = "name,qty\n갑,1\n"
        detail = self.eval_files({self.FILE: body}, self._check(2), encoding="utf-8-sig")
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 2)

    def test_empty_file_is_zero_rows(self):
        detail = self.eval_files({self.FILE: ""}, self._check(0))
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 0)

    def test_empty_file_rejects_nonzero(self):
        detail = self.eval_files({self.FILE: ""}, self._check(1))
        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], 0)

    def test_wrong_count_fails(self):
        detail = self.eval_files({self.FILE: "a,b\n1,2\n"}, self._check(9))
        self.assertFalse(detail["ok"], detail)

    def test_missing_file_fails(self):
        detail = self.eval_files({}, self._check(1))
        self.assertFalse(detail["ok"], detail)


class NdjsonCountEqTests(_OpCase):
    FILE = "out.ndjson"

    def _check(self, value):
        return {"name": "n", "op": "ndjson_count_eq", "file": self.FILE, "value": value}

    def test_counts_nonempty_lines_only(self):
        body = '{"id":1}\n\n{"id":2}\n  \n{"id":3}\n'
        detail = self.eval_files({self.FILE: body}, self._check(3))
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 3)

    def test_empty_file_is_zero(self):
        detail = self.eval_files({self.FILE: ""}, self._check(0))
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 0)

    def test_empty_file_rejects_nonzero(self):
        detail = self.eval_files({self.FILE: "\n\n"}, self._check(1))
        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], 0)

    def test_wrong_count_fails(self):
        detail = self.eval_files({self.FILE: '{"a":1}\n'}, self._check(2))
        self.assertFalse(detail["ok"], detail)

    def test_missing_file_fails(self):
        detail = self.eval_files({}, self._check(1))
        self.assertFalse(detail["ok"], detail)


class NdjsonFieldEqTests(_OpCase):
    FILE = "out.ndjson"

    def _check(self, **kwargs):
        check = {"name": "field", "op": "ndjson_field_eq", "file": self.FILE,
                 "row": 1, "path": "id", "value": 2}
        check.update(kwargs)
        return check

    def test_field_at_nonempty_row(self):
        body = '{"id":1,"name":"갑"}\n\n{"id":2,"name":"을"}\n{"id":3}\n'
        detail = self.eval_files({self.FILE: body}, self._check(row=1, path="id", value=2))
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 2)

    def test_nested_path(self):
        body = '{"meta":{"ok":true}}\n'
        detail = self.eval_files(
            {self.FILE: body},
            self._check(row=0, path="meta.ok", value=True),
        )
        self.assertTrue(detail["ok"], detail)
        self.assertIs(detail["actual"], True)

    def test_missing_path_fails(self):
        body = '{"id":1}\n'
        detail = self.eval_files(
            {self.FILE: body},
            self._check(row=0, path="missing", value=1),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertIn("실패", str(detail["actual"]))

    def test_bad_json_on_target_row_fails(self):
        body = '{"id":1}\n{not-json\n'
        detail = self.eval_files(
            {self.FILE: body},
            self._check(row=1, path="id", value=1),
        )
        self.assertFalse(detail["ok"], detail)

    def test_negative_row_fails(self):
        detail = self.eval_files(
            {self.FILE: '{"id":1}\n'},
            self._check(row=-1, path="id", value=1),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertIn("음수", str(detail["actual"]))

    def test_empty_file_fails(self):
        detail = self.eval_files({self.FILE: ""}, self._check(row=0, path="id", value=1))
        self.assertFalse(detail["ok"], detail)
        self.assertIn("없음", str(detail["actual"]))

    def test_row_past_end_fails(self):
        detail = self.eval_files(
            {self.FILE: '{"id":1}\n'},
            self._check(row=3, path="id", value=1),
        )
        self.assertFalse(detail["ok"], detail)

    def test_bool_row_is_rejected(self):
        detail = self.eval_files(
            {self.FILE: '{"id":1}\n'},
            self._check(row=True, path="id", value=1),
        )
        self.assertFalse(detail["ok"], detail)


class JsonKeysContainTests(_OpCase):
    FILE = "out.json"

    def _check(self, **kwargs):
        check = {"name": "keys", "op": "json_keys_contain", "file": self.FILE,
                 "keys": ["id", "name"]}
        check.update(kwargs)
        return check

    def test_object_contains_all_keys(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"id": 1, "name": "갑", "extra": True})},
            self._check(keys=["id", "name"]),
        )
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], ["extra", "id", "name"])

    def test_nested_object_path(self):
        body = json.dumps({"row": {"id": 1, "qty": 2}})
        detail = self.eval_files({self.FILE: body}, self._check(path="row", keys=["id", "qty"]))
        self.assertTrue(detail["ok"], detail)

    def test_missing_key_fails(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"id": 1})},
            self._check(keys=["id", "name"]),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], ["id"])

    def test_missing_path_fails(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"row": {"id": 1}})},
            self._check(path="gone", keys=["id"]),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertIn("실패", str(detail["actual"]))

    def test_bad_json_fails(self):
        detail = self.eval_files({self.FILE: "[}"}, self._check())
        self.assertFalse(detail["ok"], detail)

    def test_empty_file_fails(self):
        detail = self.eval_files({self.FILE: ""}, self._check())
        self.assertFalse(detail["ok"], detail)

    def test_array_at_path_fails(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"row": [1, 2]})},
            self._check(path="row", keys=["id"]),
        )
        self.assertFalse(detail["ok"], detail)

    def test_non_list_keys_fails(self):
        detail = self.eval_files(
            {self.FILE: json.dumps({"id": 1})},
            self._check(keys="id"),
        )
        self.assertFalse(detail["ok"], detail)


class TextLineEqTests(_OpCase):
    FILE = "out.txt"

    def _check(self, **kwargs):
        check = {"name": "line", "op": "text_line_eq", "file": self.FILE,
                 "line": 0, "value": "첫째"}
        check.update(kwargs)
        return check

    def test_zero_based_line_match(self):
        body = "첫째\n둘째\n셋째"
        detail = self.eval_files({self.FILE: body}, self._check(line=1, value="둘째"))
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], "둘째")

    def test_last_line_without_trailing_newline(self):
        detail = self.eval_files(
            {self.FILE: "첫째\n둘째"},
            self._check(line=1, value="둘째"),
        )
        self.assertTrue(detail["ok"], detail)

    def test_wrong_line_fails(self):
        detail = self.eval_files(
            {self.FILE: "첫째\n둘째\n"},
            self._check(line=0, value="둘째"),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], "첫째")

    def test_negative_line_fails(self):
        detail = self.eval_files(
            {self.FILE: "첫째\n"},
            self._check(line=-1, value="첫째"),
        )
        self.assertFalse(detail["ok"], detail)
        self.assertIn("음수", str(detail["actual"]))

    def test_empty_file_fails(self):
        detail = self.eval_files({self.FILE: ""}, self._check(line=0, value=""))
        self.assertFalse(detail["ok"], detail)
        self.assertIn("없음", str(detail["actual"]))

    def test_line_past_end_fails(self):
        detail = self.eval_files(
            {self.FILE: "한줄\n"},
            self._check(line=4, value="한줄"),
        )
        self.assertFalse(detail["ok"], detail)

    def test_missing_file_fails(self):
        detail = self.eval_files({}, self._check())
        self.assertFalse(detail["ok"], detail)

    def test_crlf_line_strips_newline_only(self):
        detail = self.eval_files(
            {self.FILE: "갑\r\n을\r\n"},
            self._check(line=1, value="을"),
        )
        self.assertTrue(detail["ok"], detail)

    def test_bool_line_is_rejected(self):
        detail = self.eval_files({self.FILE: "x\n"}, self._check(line=False, value="x"))
        self.assertFalse(detail["ok"], detail)


class NoCliInvocationTests(_OpCase):
    def test_eval_check_does_not_call_run_cli(self):
        _checks, runner, _schema = load_core()
        original = runner.run_cli

        def boom(*_args, **_kwargs):
            raise AssertionError("file op 가 CLI 를 부르면 안 된다")

        runner.run_cli = boom
        try:
            with tempfile.TemporaryDirectory() as sub_dir:
                _write(sub_dir, "out.json", json.dumps({"items": [1, 2]}))
                detail = runner.eval_check(
                    {"name": "len", "op": "json_len_eq", "file": "out.json",
                     "path": "items", "value": 2},
                    {},
                    sub_dir,
                    {},
                    "unused-rhwp",
                )
        finally:
            runner.run_cli = original
        self.assertTrue(detail["ok"], detail)


class PinpointExceptionPathTests(_OpCase):
    """1차 6연산자의 예외 — 부재·깨진 바이트·bool 좌표·스칼라 길이."""

    def test_json_len_eq_rejects_directory_and_truncated(self):
        detail = self.eval_files({"out.json": '{"xs":'}, {
            "name": "len", "op": "json_len_eq", "file": "out.json", "path": "xs", "value": 0,
        })
        self.assertFalse(detail["ok"], detail)
        self.assertIn("실패", str(detail["actual"]))

    def test_json_len_eq_nested_index_out_of_range(self):
        detail = self.eval_files({"out.json": json.dumps({"rows": [{"xs": [1]}]})}, {
            "name": "len", "op": "json_len_eq", "file": "out.json",
            "path": "rows[3].xs", "value": 0,
        })
        self.assertFalse(detail["ok"], detail)

    def test_json_len_eq_bool_false_does_not_equal_two(self):
        detail = self.eval_files({"out.json": json.dumps([1, 2])}, {
            "name": "len", "op": "json_len_eq", "file": "out.json", "value": False,
        })
        self.assertFalse(detail["ok"], detail)

    def test_csv_row_count_eq_quoted_newline_is_one_logical_row(self):
        body = "name,note\n\"갑\",\"여러\n줄\"\n"
        detail = self.eval_files({"out.csv": body}, {
            "name": "rows", "op": "csv_row_count_eq", "file": "out.csv", "value": 2,
        })
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 2)

    def test_csv_row_count_eq_crlf_three_rows(self):
        detail = self.eval_files({"out.csv": "a,b\r\n1,2\r\n3,4\r\n"}, {
            "name": "rows", "op": "csv_row_count_eq", "file": "out.csv", "value": 3,
        })
        self.assertTrue(detail["ok"], detail)

    def test_csv_row_count_eq_invalid_utf8(self):
        _checks, runner, _schema = load_core()
        with tempfile.TemporaryDirectory() as sub_dir:
            _write(sub_dir, "out.csv", b"\xff\xfe a,b\n")
            detail = runner.eval_check(
                {"name": "rows", "op": "csv_row_count_eq", "file": "out.csv", "value": 1},
                {}, sub_dir, {}, "unused-rhwp",
            )
        self.assertFalse(detail["ok"], detail)

    def test_ndjson_count_eq_ignores_whitespace_only_lines(self):
        body = "{\"id\":1}\n\t\n  \n{\"id\":2}\n"
        detail = self.eval_files({"out.ndjson": body}, {
            "name": "n", "op": "ndjson_count_eq", "file": "out.ndjson", "value": 2,
        })
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 2)

    def test_ndjson_field_eq_skips_blank_before_target_row(self):
        body = "{\"id\":1}\n\n\n{\"id\":2,\"name\":\"을\"}\n"
        detail = self.eval_files({"out.ndjson": body}, {
            "name": "f", "op": "ndjson_field_eq", "file": "out.ndjson",
            "row": 1, "path": "name", "value": "을",
        })
        self.assertTrue(detail["ok"], detail)

    def test_ndjson_field_eq_string_row_rejected(self):
        detail = self.eval_files({"out.ndjson": "{\"id\":1}\n"}, {
            "name": "f", "op": "ndjson_field_eq", "file": "out.ndjson",
            "row": "0", "path": "id", "value": 1,
        })
        self.assertFalse(detail["ok"], detail)

    def test_json_keys_contain_empty_keys_on_empty_object(self):
        detail = self.eval_files({"out.json": "{}"}, {
            "name": "k", "op": "json_keys_contain", "file": "out.json", "keys": [],
        })
        self.assertTrue(detail["ok"], detail)

    def test_json_keys_contain_rejects_number_in_keys(self):
        detail = self.eval_files({"out.json": json.dumps({"id": 1})}, {
            "name": "k", "op": "json_keys_contain", "file": "out.json", "keys": [1],
        })
        self.assertFalse(detail["ok"], detail)

    def test_text_line_eq_keeps_internal_spaces(self):
        detail = self.eval_files({"out.txt": "  갑  \n을\n"}, {
            "name": "ln", "op": "text_line_eq", "file": "out.txt",
            "line": 0, "value": "  갑  ",
        })
        self.assertTrue(detail["ok"], detail)

    def test_text_line_eq_does_not_strip_internal_tab(self):
        detail = self.eval_files({"out.txt": "갑\t을\n"}, {
            "name": "ln", "op": "text_line_eq", "file": "out.txt",
            "line": 0, "value": "갑을",
        })
        self.assertFalse(detail["ok"], detail)
        self.assertEqual(detail["actual"], "갑\t을")

    def test_text_line_eq_string_line_rejected(self):
        detail = self.eval_files({"out.txt": "갑\n"}, {
            "name": "ln", "op": "text_line_eq", "file": "out.txt",
            "line": "0", "value": "갑",
        })
        self.assertFalse(detail["ok"], detail)

    def test_all_six_ops_fail_on_missing_file(self):
        checks = [
            {"name": "a", "op": "json_len_eq", "file": "m.json", "value": 0},
            {"name": "b", "op": "csv_row_count_eq", "file": "m.csv", "value": 1},
            {"name": "c", "op": "ndjson_count_eq", "file": "m.ndjson", "value": 1},
            {"name": "d", "op": "ndjson_field_eq", "file": "m.ndjson", "row": 0,
             "path": "id", "value": 1},
            {"name": "e", "op": "json_keys_contain", "file": "m.json", "keys": ["id"]},
            {"name": "f", "op": "text_line_eq", "file": "m.txt", "line": 0, "value": "x"},
        ]
        for check in checks:
            with self.subTest(check["op"]):
                self.assertFalse(self.eval_files({}, check)["ok"], check)


class ExistingPacksStayValidTests(unittest.TestCase):
    def test_registered_ops_do_not_break_pack_schema(self):
        _checks, _runner, schema = load_core()
        packs = REPO_ROOT / "gym" / "packs"
        errors = []
        for pack_dir in sorted(p for p in packs.iterdir() if (p / "pack.json").is_file()):
            manifest = json.loads((pack_dir / "pack.json").read_text(encoding="utf-8"))
            for path in sorted((pack_dir / "tasks").glob("*.json")):
                schema.validate_task(
                    json.loads(path.read_text(encoding="utf-8")),
                    manifest,
                    None,
                    errors,
                )
        self.assertEqual(errors, [], "\n".join(errors))


if __name__ == "__main__":
    unittest.main()
