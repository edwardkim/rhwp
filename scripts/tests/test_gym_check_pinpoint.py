"""[#5205] 지목 채점 연산자 — 좌표·예외·유니코드 행렬.

기존 json_len_eq/csv_row_count_eq/ndjson_*/json_keys_contain/text_line_eq 와
후속 지목 연산자(json_type_eq, json_len_ge, json_array_item_eq, csv_col_count_eq,
csv_header_eq, csv_row_eq, ndjson_keys_contain, ndjson_len_eq, text_line_count_eq,
text_line_contains)를 파일 픽스처만으로 판정한다. CLI 는 부르지 않는다.
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


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
    def eval_files(self, files, check, encoding="utf-8"):
        _checks, runner, _schema = load_core()
        with tempfile.TemporaryDirectory() as sub_dir:
            for name, content in files.items():
                _write(sub_dir, name, content, encoding=encoding)
            return runner.eval_check(check, {}, sub_dir, {}, "unused-rhwp")

    def eval_bytes(self, name, payload, check):
        _checks, runner, _schema = load_core()
        with tempfile.TemporaryDirectory() as sub_dir:
            _write(sub_dir, name, payload)
            return runner.eval_check(check, {}, sub_dir, {}, "unused-rhwp")


def _json(obj):
    return json.dumps(obj, ensure_ascii=False)


NESTED = {
    "meta": {"schema": "v1", "ok": True, "count": 3},
    "items": [
        {"id": 1, "name": "갑", "tags": ["초안", "표"], "qty": 2},
        {"id": 2, "name": "을", "tags": ["확정"], "qty": 5},
        {"id": 3, "name": "병", "tags": [], "qty": 0},
    ],
    "empty": [],
    "blank": {},
    "note": "한 줄 메모",
    "flag": False,
    "nil": None,
    "pi": 3.14,
}


class JsonLenEqMatrixTests(_OpCase):
    FILE = "out.json"

    def _c(self, **kwargs):
        check = {"name": "len", "op": "json_len_eq", "file": self.FILE, "value": 0}
        check.update(kwargs)
        return check

    CASES = [
        ('root_object_eight_keys', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, '', 8, True, None),
        ('root_object_wrong', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, '', 5, False, None),
        ('items_three', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', 3, True, None),
        ('items_wrong', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', 2, False, None),
        ('nested_tags_first', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0].tags', 2, True, None),
        ('nested_tags_second', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[1].tags', 1, True, None),
        ('nested_tags_empty', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[2].tags', 0, True, None),
        ('empty_array', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'empty', 0, True, None),
        ('empty_object', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'blank', 0, True, None),
        ('meta_object', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'meta', 3, True, None),
        ('numeric_string', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', '3', True, None),
        ('float_string', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', '3.0', True, None),
        ('missing_key', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'gone', 0, False, '실패'),
        ('scalar_string', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'note', 1, False, '실패'),
        ('scalar_number', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'pi', 1, False, '실패'),
        ('scalar_bool', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'flag', 1, False, '실패'),
        ('scalar_null', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'nil', 0, False, '실패'),
        ('index_oob', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[9]', 0, False, '실패'),
        ('index_on_object', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'meta[0]', 0, False, '실패'),
        ('deep_missing', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0].missing', 0, False, '실패'),
        ('array_len_0', {'xs': []}, 'xs', 0, True, None),
        ('array_len_1', {'xs': [0]}, 'xs', 1, True, None),
        ('array_len_2', {'xs': [0, 1]}, 'xs', 2, True, None),
        ('array_len_3', {'xs': [0, 1, 2]}, 'xs', 3, True, None),
        ('array_len_4', {'xs': [0, 1, 2, 3]}, 'xs', 4, True, None),
        ('array_len_5', {'xs': [0, 1, 2, 3, 4]}, 'xs', 5, True, None),
        ('array_len_6', {'xs': [0, 1, 2, 3, 4, 5]}, 'xs', 6, True, None),
        ('array_len_7', {'xs': [0, 1, 2, 3, 4, 5, 6]}, 'xs', 7, True, None),
        ('array_len_8', {'xs': [0, 1, 2, 3, 4, 5, 6, 7]}, 'xs', 8, True, None),
        ('array_len_9', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8]}, 'xs', 9, True, None),
        ('array_len_10', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]}, 'xs', 10, True, None),
        ('array_len_11', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]}, 'xs', 11, True, None),
        ('array_len_12', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]}, 'xs', 12, True, None),
        ('array_len_13', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]}, 'xs', 13, True, None),
        ('array_len_14', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]}, 'xs', 14, True, None),
        ('array_len_15', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]}, 'xs', 15, True, None),
        ('array_len_16', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]}, 'xs', 16, True, None),
        ('array_len_17', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]}, 'xs', 17, True, None),
        ('array_len_18', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]}, 'xs', 18, True, None),
        ('array_len_19', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18]}, 'xs', 19, True, None),
        ('array_len_20', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]}, 'xs', 20, True, None),
        ('object_len_0', {}, '', 0, True, None),
        ('object_len_1', {'k0': 0}, '', 1, True, None),
        ('object_len_2', {'k0': 0, 'k1': 1}, '', 2, True, None),
        ('object_len_5', {'k0': 0, 'k1': 1, 'k2': 2, 'k3': 3, 'k4': 4}, '', 5, True, None),
        ('object_len_8', {'k0': 0, 'k1': 1, 'k2': 2, 'k3': 3, 'k4': 4, 'k5': 5, 'k6': 6, 'k7': 7}, '', 8, True, None),
        ('object_len_13', {'k0': 0, 'k1': 1, 'k2': 2, 'k3': 3, 'k4': 4, 'k5': 5, 'k6': 6, 'k7': 7, 'k8': 8, 'k9': 9, 'k10': 10, 'k11': 11, 'k12': 12}, '', 13, True, None),
        ('object_len_21', {'k0': 0, 'k1': 1, 'k2': 2, 'k3': 3, 'k4': 4, 'k5': 5, 'k6': 6, 'k7': 7, 'k8': 8, 'k9': 9, 'k10': 10, 'k11': 11, 'k12': 12, 'k13': 13, 'k14': 14, 'k15': 15, 'k16': 16, 'k17': 17, 'k18': 18, 'k19': 19, 'k20': 20}, '', 21, True, None),
        ('hangul_keys', {'이름': 1, '수량': 2, '비고': 3}, '', 3, True, None),
        ('hangul_array', {'항목': ['가', '나', '다', '라']}, '항목', 4, True, None),
        ('mixed_unicode', {'name': '이름', '태그': ['α', 'β']}, '태그', 2, True, None),
        ('emoji_keys', {'✅': 1, '❌': 0}, '', 2, True, None),
    ]

    def test_matrix(self):
        for name, obj, path, value, ok, needle in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: _json(obj)}, self._c(path=path, value=value))
                self.assertEqual(detail["ok"], ok, (name, detail))
                if needle:
                    self.assertIn(needle, str(detail["actual"]))

    def test_bad_json_variants_fail(self):
        for name, body in (
            ("empty", ""),
            ("spaces", "   \n"),
            ("truncated", '{"a":'),
            ("trailing_comma", '{"a":1,}'),
            ("single_quotes", "{'a': 1}"),
            ("not_json", "items=3"),
            ("array_then_junk", "[1,2] junk"),
            ("nan", "NaN"),
            ("undefined", "undefined"),
        ):
            with self.subTest(name):
                detail = self.eval_files({self.FILE: body}, self._c(path="", value=0))
                self.assertFalse(detail["ok"], (name, detail))
                self.assertIn("실패", str(detail["actual"]))

    def test_missing_file_and_invalid_utf8(self):
        detail = self.eval_files({}, self._c())
        self.assertFalse(detail["ok"], detail)
        detail = self.eval_bytes(self.FILE, b"\xff\xfe not utf8", self._c())
        self.assertFalse(detail["ok"], detail)

    def test_bool_false_does_not_match_nonzero_length(self):
        detail = self.eval_files({self.FILE: _json([1, 2])}, self._c(path="", value=False))
        self.assertFalse(detail["ok"], detail)


class JsonTypeEqTests(_OpCase):
    FILE = "out.json"

    def _c(self, **kwargs):
        check = {"name": "typ", "op": "json_type_eq", "file": self.FILE, "value": "object"}
        check.update(kwargs)
        return check

    CASES = [
        ('root_object', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, '', 'object', True),
        ('items_array', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', 'array', True),
        ('meta_object', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'meta', 'object', True),
        ('note_string', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'note', 'string', True),
        ('flag_bool', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'flag', 'boolean', True),
        ('ok_bool', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'meta.ok', 'boolean', True),
        ('count_number', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'meta.count', 'number', True),
        ('pi_number', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'pi', 'number', True),
        ('nil_null', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'nil', 'null', True),
        ('first_name', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0].name', 'string', True),
        ('first_qty', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0].qty', 'number', True),
        ('tags_array', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0].tags', 'array', True),
        ('empty_array', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'empty', 'array', True),
        ('empty_object', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'blank', 'object', True),
        ('wrong_type', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'note', 'number', False),
        ('bool_not_number', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'flag', 'number', False),
        ('null_not_object', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'nil', 'object', False),
        ('string_not_array', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'note', 'array', False),
        ('missing', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'gone', 'object', False),
        ('zero_number', {'n': 0}, 'n', 'number', True),
        ('neg_number', {'n': -3}, 'n', 'number', True),
        ('empty_string', {'s': ''}, 's', 'string', True),
        ('true_bool', {'b': True}, 'b', 'boolean', True),
        ('false_bool', {'b': False}, 'b', 'boolean', True),
        ('nested_null', {'a': {'b': None}}, 'a.b', 'null', True),
        ('list_of_null', {'xs': [None]}, 'xs[0]', 'null', True),
    ]

    def test_matrix(self):
        for name, obj, path, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: _json(obj)}, self._c(path=path, value=value))
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_non_string_expected_fails(self):
        detail = self.eval_files({self.FILE: _json({"a": 1})}, self._c(value=1))
        self.assertFalse(detail["ok"], detail)

    def test_bad_json_fails(self):
        detail = self.eval_files({self.FILE: "{"}, self._c())
        self.assertFalse(detail["ok"], detail)

    def test_missing_file_fails(self):
        detail = self.eval_files({}, self._c())
        self.assertFalse(detail["ok"], detail)


class JsonLenGeTests(_OpCase):
    FILE = "out.json"

    def _c(self, **kwargs):
        check = {"name": "ge", "op": "json_len_ge", "file": self.FILE, "value": 1}
        check.update(kwargs)
        return check

    CASES = [
        ('arr_0_ge_0', {'xs': []}, 'xs', 0, True),
        ('arr_0_ge_1', {'xs': []}, 'xs', 1, False),
        ('arr_1_ge_1', {'xs': [0]}, 'xs', 1, True),
        ('arr_1_ge_0', {'xs': [0]}, 'xs', 0, True),
        ('arr_3_ge_3', {'xs': [0, 1, 2]}, 'xs', 3, True),
        ('arr_3_ge_4', {'xs': [0, 1, 2]}, 'xs', 4, False),
        ('arr_5_ge_5', {'xs': [0, 1, 2, 3, 4]}, 'xs', 5, True),
        ('arr_5_ge_2', {'xs': [0, 1, 2, 3, 4]}, 'xs', 2, True),
        ('arr_8_ge_8', {'xs': [0, 1, 2, 3, 4, 5, 6, 7]}, 'xs', 8, True),
        ('arr_8_ge_9', {'xs': [0, 1, 2, 3, 4, 5, 6, 7]}, 'xs', 9, False),
        ('arr_13_ge_10', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]}, 'xs', 10, True),
        ('arr_21_ge_21', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]}, 'xs', 21, True),
        ('arr_21_ge_22', {'xs': [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]}, 'xs', 22, False),
        ('obj_0_ge_0', {}, '', 0, True),
        ('obj_2_ge_2', {'k0': 0, 'k1': 1}, '', 2, True),
        ('obj_2_ge_3', {'k0': 0, 'k1': 1}, '', 3, False),
        ('obj_4_ge_1', {'k0': 0, 'k1': 1, 'k2': 2, 'k3': 3}, '', 1, True),
        ('items_ge_3', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', 3, True),
        ('items_ge_4', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', 4, False),
        ('tags_ge_2', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0].tags', 2, True),
        ('tags_ge_3', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0].tags', 3, False),
        ('empty_ge_0', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'empty', 0, True),
        ('empty_ge_1', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'empty', 1, False),
        ('scalar_fails', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'note', 1, False),
        ('missing_fails', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'gone', 0, False),
    ]

    def test_matrix(self):
        for name, obj, path, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: _json(obj)}, self._c(path=path, value=value))
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_numeric_string_bound(self):
        detail = self.eval_files({self.FILE: _json({"xs": [1, 2]})}, self._c(path="xs", value="2"))
        self.assertTrue(detail["ok"], detail)

    def test_missing_file_fails(self):
        detail = self.eval_files({}, self._c())
        self.assertFalse(detail["ok"], detail)


class JsonArrayItemEqTests(_OpCase):
    FILE = "out.json"

    def _c(self, **kwargs):
        check = {"name": "item", "op": "json_array_item_eq", "file": self.FILE,
                 "index": 0, "value": 1}
        check.update(kwargs)
        return check

    CASES = [
        ('first_int', {'xs': [10, 20, 30]}, 'xs', 0, 10, True),
        ('second_int', {'xs': [10, 20, 30]}, 'xs', 1, 20, True),
        ('third_int', {'xs': [10, 20, 30]}, 'xs', 2, 30, True),
        ('wrong_value', {'xs': [10, 20, 30]}, 'xs', 1, 99, False),
        ('numeric_string', {'xs': [10, 20]}, 'xs', 0, '10', True),
        ('hangul', {'xs': ['갑', '을', '병']}, 'xs', 1, '을', True),
        ('bool_true', {'xs': [True, False]}, 'xs', 0, True, True),
        ('bool_false', {'xs': [True, False]}, 'xs', 1, False, True),
        ('null_item', {'xs': [None]}, 'xs', 0, None, True),
        ('nested_object_eq', {'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}]}, 'items', 1, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, True),
        ('index_oob', {'xs': [1]}, 'xs', 3, 1, False),
        ('not_array', {'xs': {'a': 1}}, 'xs', 0, 1, False),
        ('root_array', [7, 8, 9], '', 2, 9, True),
        ('empty_array_oob', {'xs': []}, 'xs', 0, None, False),
        ('missing_path', {'xs': [1]}, 'gone', 0, 1, False),
        ('hangul_idx_0', {'xs': ['가', '나', '다', '라', '마', '바', '사']}, 'xs', 0, '가', True),
        ('hangul_idx_1', {'xs': ['가', '나', '다', '라', '마', '바', '사']}, 'xs', 1, '나', True),
        ('hangul_idx_2', {'xs': ['가', '나', '다', '라', '마', '바', '사']}, 'xs', 2, '다', True),
        ('hangul_idx_3', {'xs': ['가', '나', '다', '라', '마', '바', '사']}, 'xs', 3, '라', True),
        ('hangul_idx_4', {'xs': ['가', '나', '다', '라', '마', '바', '사']}, 'xs', 4, '마', True),
        ('hangul_idx_5', {'xs': ['가', '나', '다', '라', '마', '바', '사']}, 'xs', 5, '바', True),
        ('hangul_idx_6', {'xs': ['가', '나', '다', '라', '마', '바', '사']}, 'xs', 6, '사', True),
    ]

    def test_matrix(self):
        for name, obj, path, index, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files(
                    {self.FILE: _json(obj)},
                    self._c(path=path, index=index, value=value),
                )
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_bool_index_rejected(self):
        detail = self.eval_files({self.FILE: _json([1, 2])}, self._c(path="", index=True, value=2))
        self.assertFalse(detail["ok"], detail)
        self.assertIn("정수", str(detail["actual"]))

    def test_negative_index_rejected(self):
        detail = self.eval_files({self.FILE: _json([1, 2])}, self._c(path="", index=-1, value=2))
        self.assertFalse(detail["ok"], detail)
        self.assertIn("음수", str(detail["actual"]))

    def test_missing_file_fails(self):
        detail = self.eval_files({}, self._c())
        self.assertFalse(detail["ok"], detail)


class JsonKeysContainExtraTests(_OpCase):
    FILE = "out.json"

    def _c(self, **kwargs):
        check = {"name": "keys", "op": "json_keys_contain", "file": self.FILE, "keys": ["id"]}
        check.update(kwargs)
        return check

    CASES = [
        ('all_present', {'id': 1, 'name': '갑', 'qty': 2}, '', ['id', 'name'], True),
        ('one_missing', {'id': 1, 'qty': 2}, '', ['id', 'name'], False),
        ('empty_required_ok', {'id': 1}, '', [], True),
        ('empty_object_missing', {}, '', ['id'], False),
        ('nested_meta', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'meta', ['schema', 'ok'], True),
        ('nested_item', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0]', ['id', 'name', 'tags'], True),
        ('nested_item_missing', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items[0]', ['id', 'gone'], False),
        ('hangul_keys', {'이름': '갑', '수량': 1}, '', ['이름'], True),
        ('hangul_missing', {'이름': '갑'}, '', ['수량'], False),
        ('unicode_mixed', {'id': 1, '이름': '을'}, '', ['id', '이름'], True),
        ('path_array_fails', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'items', ['id'], False),
        ('path_scalar_fails', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'note', ['id'], False),
        ('missing_path', {'meta': {'schema': 'v1', 'ok': True, 'count': 3}, 'items': [{'id': 1, 'name': '갑', 'tags': ['초안', '표'], 'qty': 2}, {'id': 2, 'name': '을', 'tags': ['확정'], 'qty': 5}, {'id': 3, 'name': '병', 'tags': [], 'qty': 0}], 'empty': [], 'blank': {}, 'note': '한 줄 메모', 'flag': False, 'nil': None, 'pi': 3.14}, 'gone', ['id'], False),
        ('has_f00', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f00'], True),
        ('has_f01', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f01'], True),
        ('has_f02', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f02'], True),
        ('has_f03', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f03'], True),
        ('has_f04', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f04'], True),
        ('has_f05', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f05'], True),
        ('has_f06', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f06'], True),
        ('has_f07', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f07'], True),
        ('has_f08', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f08'], True),
        ('has_f09', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f09'], True),
        ('has_f10', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f10'], True),
        ('has_f11', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f11'], True),
        ('has_f12', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f12'], True),
        ('has_f13', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f13'], True),
        ('has_f14', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f14'], True),
        ('has_f15', {'f00': 0, 'f01': 1, 'f02': 2, 'f03': 3, 'f04': 4, 'f05': 5, 'f06': 6, 'f07': 7, 'f08': 8, 'f09': 9, 'f10': 10, 'f11': 11, 'f12': 12, 'f13': 13, 'f14': 14, 'f15': 15}, '', ['f15'], True),
    ]

    def test_matrix(self):
        for name, obj, path, keys, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: _json(obj)}, self._c(path=path, keys=keys))
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_keys_must_be_string_list(self):
        for keys in ("id", ["id", 1], {"id": True}, None, [None]):
            with self.subTest(keys=keys):
                detail = self.eval_files({self.FILE: _json({"id": 1})}, self._c(keys=keys))
                self.assertFalse(detail["ok"], detail)


CSV_SIMPLE = "name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n"
CSV_QUOTED = 'name,note\n"갑","여러\n줄"\n을,단줄\n'
CSV_BOM_BODY = "name,qty\n갑,1\n을,2\n"
CSV_EMPTY_TRAIL = "a,b\n1,2\n\n"
CSV_WIDE = "c0,c1,c2,c3,c4,c5,c6,c7\n" + ",".join(str(i) for i in range(8)) + "\n"
CSV_HANGUL_HEADER = "이름,수량,비고\n사과,10,특\n배,4,보통\n"


class CsvRowCountEqMatrixTests(_OpCase):
    FILE = "out.csv"

    def _c(self, value):
        return {"name": "rows", "op": "csv_row_count_eq", "file": self.FILE, "value": value}

    CASES = [
        ('simple_four', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 4, True),
        ('simple_wrong', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 3, False),
        ('quoted_multiline_three', 'name,note\n"갑","여러\n줄"\n을,단줄\n', 3, True),
        ('empty_zero', '', 0, True),
        ('empty_nonzero', '', 1, False),
        ('header_only', 'name,qty\\n', 1, True),
        ('wide_two', 'c0,c1,c2,c3,c4,c5,c6,c7\n0,1,2,3,4,5,6,7\n', 2, True),
        ('hangul_three', '이름,수량,비고\n사과,10,특\n배,4,보통\n', 3, True),
        ('numeric_string', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', '4', True),
        ('n_rows_0', '', 0, True),
        ('n_rows_1', 'h\n', 1, True),
        ('n_rows_2', 'h\n0\n', 2, True),
        ('n_rows_3', 'h\n0\n1\n', 3, True),
        ('n_rows_4', 'h\n0\n1\n2\n', 4, True),
        ('n_rows_5', 'h\n0\n1\n2\n3\n', 5, True),
        ('n_rows_6', 'h\n0\n1\n2\n3\n4\n', 6, True),
        ('n_rows_7', 'h\n0\n1\n2\n3\n4\n5\n', 7, True),
        ('n_rows_8', 'h\n0\n1\n2\n3\n4\n5\n6\n', 8, True),
        ('n_rows_9', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n', 9, True),
        ('n_rows_10', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n', 10, True),
        ('n_rows_11', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n', 11, True),
        ('n_rows_12', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n', 12, True),
        ('n_rows_13', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n', 13, True),
        ('n_rows_14', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n', 14, True),
        ('n_rows_15', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n', 15, True),
        ('n_rows_16', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n', 16, True),
        ('n_rows_17', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n', 17, True),
        ('n_rows_18', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n', 18, True),
        ('n_rows_19', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n', 19, True),
        ('n_rows_20', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n', 20, True),
        ('n_rows_21', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n', 21, True),
        ('n_rows_22', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n', 22, True),
        ('n_rows_23', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n', 23, True),
        ('n_rows_24', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n', 24, True),
        ('n_rows_25', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n', 25, True),
        ('n_rows_26', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n', 26, True),
        ('n_rows_27', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n', 27, True),
        ('n_rows_28', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n', 28, True),
        ('n_rows_29', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27\n', 29, True),
        ('n_rows_30', 'h\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27\n28\n', 30, True),
    ]

    def test_matrix(self):
        for name, body, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: body}, self._c(value))
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_utf8_sig_does_not_add_row(self):
        detail = self.eval_files({self.FILE: CSV_BOM_BODY}, self._c(3), encoding="utf-8-sig")
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 3)

    def test_crlf_same_as_lf(self):
        body = "a,b\r\n1,2\r\n3,4\r\n"
        detail = self.eval_files({self.FILE: body}, self._c(3))
        self.assertTrue(detail["ok"], detail)

    def test_missing_file_and_invalid_utf8(self):
        self.assertFalse(self.eval_files({}, self._c(1))["ok"])
        self.assertFalse(self.eval_bytes(self.FILE, b"\xff\xfe a,b\n", self._c(1))["ok"])


class CsvColCountEqTests(_OpCase):
    FILE = "out.csv"

    def _c(self, **kwargs):
        check = {"name": "cols", "op": "csv_col_count_eq", "file": self.FILE, "row": 0, "value": 3}
        check.update(kwargs)
        return check

    CASES = [
        ('header_three', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 0, 3, True),
        ('data_three', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 1, 3, True),
        ('wrong', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 0, 2, False),
        ('wide_eight', 'c0,c1,c2,c3,c4,c5,c6,c7\n0,1,2,3,4,5,6,7\n', 0, 8, True),
        ('wide_data', 'c0,c1,c2,c3,c4,c5,c6,c7\n0,1,2,3,4,5,6,7\n', 1, 8, True),
        ('quoted_two', 'name,note\n"갑","여러\n줄"\n을,단줄\n', 0, 2, True),
        ('quoted_data', 'name,note\n"갑","여러\n줄"\n을,단줄\n', 1, 2, True),
        ('row_oob', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 9, 3, False),
        ('neg_row', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', -1, 3, False),
        ('hangul_three', '이름,수량,비고\n사과,10,특\n배,4,보통\n', 0, 3, True),
        ('simple_row_0_cols', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 0, 3, True),
        ('simple_row_1_cols', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 1, 3, True),
        ('simple_row_2_cols', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 2, 3, True),
        ('simple_row_3_cols', 'name,qty,note\n갑,1,초안\n을,2,확정\n병,3,\n', 3, 3, True),
    ]

    def test_matrix(self):
        for name, body, row, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: body}, self._c(row=row, value=value))
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_bool_row_rejected(self):
        detail = self.eval_files({self.FILE: CSV_SIMPLE}, self._c(row=True, value=3))
        self.assertFalse(detail["ok"], detail)

    def test_empty_file_fails(self):
        detail = self.eval_files({self.FILE: ""}, self._c(row=0, value=0))
        self.assertFalse(detail["ok"], detail)

    def test_missing_file_fails(self):
        self.assertFalse(self.eval_files({}, self._c())["ok"])


class CsvHeaderAndRowEqTests(_OpCase):
    FILE = "out.csv"

    def test_header_match(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "h", "op": "csv_header_eq", "file": self.FILE,
             "values": ["name", "qty", "note"]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_header_mismatch(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "h", "op": "csv_header_eq", "file": self.FILE,
             "values": ["name", "qty"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_header_hangul(self):
        detail = self.eval_files(
            {self.FILE: CSV_HANGUL_HEADER},
            {"name": "h", "op": "csv_header_eq", "file": self.FILE,
             "values": ["이름", "수량", "비고"]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_header_empty_file_fails(self):
        detail = self.eval_files(
            {self.FILE: ""},
            {"name": "h", "op": "csv_header_eq", "file": self.FILE, "values": []},
        )
        self.assertFalse(detail["ok"], detail)

    def test_header_values_must_be_str_list(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "h", "op": "csv_header_eq", "file": self.FILE, "values": "name"},
        )
        self.assertFalse(detail["ok"], detail)

    def test_row_eq_match(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": 1,
             "values": ["갑", "1", "초안"]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_row_eq_second_data(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": 2,
             "values": ["을", "2", "확정"]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_row_eq_empty_note(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": 3,
             "values": ["병", "3", ""]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_row_eq_mismatch(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": 1,
             "values": ["갑", "9", "초안"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_row_eq_oob(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": 9,
             "values": ["갑", "1", "초안"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_row_eq_negative(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": -1,
             "values": ["name", "qty", "note"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_row_eq_bool_row(self):
        detail = self.eval_files(
            {self.FILE: CSV_SIMPLE},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": False,
             "values": ["name", "qty", "note"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_quoted_multiline_row(self):
        detail = self.eval_files(
            {self.FILE: CSV_QUOTED},
            {"name": "r", "op": "csv_row_eq", "file": self.FILE, "row": 1,
             "values": ["갑", "여러\n줄"]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_header_missing_file(self):
        detail = self.eval_files(
            {},
            {"name": "h", "op": "csv_header_eq", "file": self.FILE, "values": ["a"]},
        )
        self.assertFalse(detail["ok"], detail)


NDJSON_SIMPLE = (
    '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n'
    '\n'
    '{"id":2,"name":"을","tags":["확정"],"qty":5}\n'
    '  \n'
    '{"id":3,"name":"병","tags":[],"qty":0}\n'
)
NDJSON_TYPES = (
    '{"kind":"null","v":null}\n'
    '{"kind":"bool","v":true}\n'
    '{"kind":"num","v":3.14}\n'
    '{"kind":"str","v":"한글"}\n'
    '{"kind":"arr","v":[1,2,3]}\n'
    '{"kind":"obj","v":{"a":1,"b":2}}\n'
)
NDJSON_BAD = '{"id":1}\n{not-json\n{"id":3}\n'


class NdjsonCountEqMatrixTests(_OpCase):
    FILE = "out.ndjson"

    def _c(self, value):
        return {"name": "n", "op": "ndjson_count_eq", "file": self.FILE, "value": value}

    CASES = [
        ('simple_three', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 3, True),
        ('simple_wrong', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 4, False),
        ('types_six', '{"kind":"null","v":null}\n{"kind":"bool","v":true}\n{"kind":"num","v":3.14}\n{"kind":"str","v":"한글"}\n{"kind":"arr","v":[1,2,3]}\n{"kind":"obj","v":{"a":1,"b":2}}\n', 6, True),
        ('empty_zero', '', 0, True),
        ('only_blank', '\n\n  \n', 0, True),
        ('numeric_string', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', '3', True),
        ('n_records_0', '', 0, True),
        ('n_records_1', '{"i":0}\n\n', 1, True),
        ('n_records_2', '{"i":0}\n\n{"i":1}\n', 2, True),
        ('n_records_3', '{"i":0}\n\n{"i":1}\n{"i":2}\n', 3, True),
        ('n_records_4', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n', 4, True),
        ('n_records_5', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n', 5, True),
        ('n_records_6', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n', 6, True),
        ('n_records_7', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n', 7, True),
        ('n_records_8', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n', 8, True),
        ('n_records_9', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n', 9, True),
        ('n_records_10', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n', 10, True),
        ('n_records_11', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n', 11, True),
        ('n_records_12', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n', 12, True),
        ('n_records_13', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n', 13, True),
        ('n_records_14', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n', 14, True),
        ('n_records_15', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n', 15, True),
        ('n_records_16', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n', 16, True),
        ('n_records_17', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n', 17, True),
        ('n_records_18', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n', 18, True),
        ('n_records_19', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n{"i":18}\n\n', 19, True),
        ('n_records_20', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n{"i":18}\n\n{"i":19}\n', 20, True),
        ('n_records_21', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n{"i":18}\n\n{"i":19}\n{"i":20}\n', 21, True),
        ('n_records_22', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n{"i":18}\n\n{"i":19}\n{"i":20}\n{"i":21}\n\n', 22, True),
        ('n_records_23', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n{"i":18}\n\n{"i":19}\n{"i":20}\n{"i":21}\n\n{"i":22}\n', 23, True),
        ('n_records_24', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n{"i":18}\n\n{"i":19}\n{"i":20}\n{"i":21}\n\n{"i":22}\n{"i":23}\n', 24, True),
        ('n_records_25', '{"i":0}\n\n{"i":1}\n{"i":2}\n{"i":3}\n\n{"i":4}\n{"i":5}\n{"i":6}\n\n{"i":7}\n{"i":8}\n{"i":9}\n\n{"i":10}\n{"i":11}\n{"i":12}\n\n{"i":13}\n{"i":14}\n{"i":15}\n\n{"i":16}\n{"i":17}\n{"i":18}\n\n{"i":19}\n{"i":20}\n{"i":21}\n\n{"i":22}\n{"i":23}\n{"i":24}\n\n', 25, True),
    ]

    def test_matrix(self):
        for name, body, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: body}, self._c(value))
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_missing_file_and_invalid_utf8(self):
        self.assertFalse(self.eval_files({}, self._c(1))["ok"])
        self.assertFalse(self.eval_bytes(self.FILE, b"\xff\xfe {}", self._c(1))["ok"])


class NdjsonFieldEqMatrixTests(_OpCase):
    FILE = "out.ndjson"

    def _c(self, **kwargs):
        check = {"name": "f", "op": "ndjson_field_eq", "file": self.FILE,
                 "row": 0, "path": "id", "value": 1}
        check.update(kwargs)
        return check

    CASES = [
        ('id0', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 0, 'id', 1, True),
        ('id1', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 1, 'id', 2, True),
        ('id2', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 2, 'id', 3, True),
        ('name1', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 1, 'name', '을', True),
        ('qty0', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 0, 'qty', 2, True),
        ('tag0', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 0, 'tags[0]', '초안', True),
        ('tag1', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 0, 'tags[1]', '표', True),
        ('wrong', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 0, 'id', 99, False),
        ('missing_path', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 0, 'gone', 1, False),
        ('row_oob', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', 9, 'id', 1, False),
        ('neg_row', '{"id":1,"name":"갑","tags":["초안","표"],"qty":2}\n\n{"id":2,"name":"을","tags":["확정"],"qty":5}\n  \n{"id":3,"name":"병","tags":[],"qty":0}\n', -1, 'id', 1, False),
        ('type_null', '{"kind":"null","v":null}\n{"kind":"bool","v":true}\n{"kind":"num","v":3.14}\n{"kind":"str","v":"한글"}\n{"kind":"arr","v":[1,2,3]}\n{"kind":"obj","v":{"a":1,"b":2}}\n', 0, 'v', None, True),
        ('type_bool', '{"kind":"null","v":null}\n{"kind":"bool","v":true}\n{"kind":"num","v":3.14}\n{"kind":"str","v":"한글"}\n{"kind":"arr","v":[1,2,3]}\n{"kind":"obj","v":{"a":1,"b":2}}\n', 1, 'v', True, True),
        ('type_num', '{"kind":"null","v":null}\n{"kind":"bool","v":true}\n{"kind":"num","v":3.14}\n{"kind":"str","v":"한글"}\n{"kind":"arr","v":[1,2,3]}\n{"kind":"obj","v":{"a":1,"b":2}}\n', 2, 'v', 3.14, True),
        ('type_str', '{"kind":"null","v":null}\n{"kind":"bool","v":true}\n{"kind":"num","v":3.14}\n{"kind":"str","v":"한글"}\n{"kind":"arr","v":[1,2,3]}\n{"kind":"obj","v":{"a":1,"b":2}}\n', 3, 'v', '한글', True),
        ('type_arr0', '{"kind":"null","v":null}\n{"kind":"bool","v":true}\n{"kind":"num","v":3.14}\n{"kind":"str","v":"한글"}\n{"kind":"arr","v":[1,2,3]}\n{"kind":"obj","v":{"a":1,"b":2}}\n', 4, 'v[0]', 1, True),
        ('type_obj_a', '{"kind":"null","v":null}\n{"kind":"bool","v":true}\n{"kind":"num","v":3.14}\n{"kind":"str","v":"한글"}\n{"kind":"arr","v":[1,2,3]}\n{"kind":"obj","v":{"a":1,"b":2}}\n', 5, 'v.a', 1, True),
        ('bad_json_row', '{"id":1}\n{not-json\n{"id":3}\n', 1, 'id', 2, False),
        ('after_bad', '{"id":1}\n{not-json\n{"id":3}\n', 2, 'id', 3, True),
        ('row_0_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 0, 'id', 0, True),
        ('row_0_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 0, 'name', '행0', True),
        ('row_1_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 1, 'id', 1, True),
        ('row_1_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 1, 'name', '행1', True),
        ('row_2_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 2, 'id', 2, True),
        ('row_2_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 2, 'name', '행2', True),
        ('row_3_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 3, 'id', 3, True),
        ('row_3_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 3, 'name', '행3', True),
        ('row_4_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 4, 'id', 4, True),
        ('row_4_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 4, 'name', '행4', True),
        ('row_5_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 5, 'id', 5, True),
        ('row_5_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 5, 'name', '행5', True),
        ('row_6_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 6, 'id', 6, True),
        ('row_6_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 6, 'name', '행6', True),
        ('row_7_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 7, 'id', 7, True),
        ('row_7_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 7, 'name', '행7', True),
        ('row_8_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 8, 'id', 8, True),
        ('row_8_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 8, 'name', '행8', True),
        ('row_9_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 9, 'id', 9, True),
        ('row_9_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 9, 'name', '행9', True),
        ('row_10_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 10, 'id', 10, True),
        ('row_10_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 10, 'name', '행10', True),
        ('row_11_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 11, 'id', 11, True),
        ('row_11_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 11, 'name', '행11', True),
        ('row_12_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 12, 'id', 12, True),
        ('row_12_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 12, 'name', '행12', True),
        ('row_13_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 13, 'id', 13, True),
        ('row_13_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 13, 'name', '행13', True),
        ('row_14_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 14, 'id', 14, True),
        ('row_14_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 14, 'name', '행14', True),
        ('row_15_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 15, 'id', 15, True),
        ('row_15_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 15, 'name', '행15', True),
        ('row_16_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 16, 'id', 16, True),
        ('row_16_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 16, 'name', '행16', True),
        ('row_17_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 17, 'id', 17, True),
        ('row_17_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 17, 'name', '행17', True),
        ('row_18_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 18, 'id', 18, True),
        ('row_18_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 18, 'name', '행18', True),
        ('row_19_id', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 19, 'id', 19, True),
        ('row_19_name', '{"id":0,"name":"행0"}\n{"id":1,"name":"행1"}\n{"id":2,"name":"행2"}\n{"id":3,"name":"행3"}\n{"id":4,"name":"행4"}\n{"id":5,"name":"행5"}\n{"id":6,"name":"행6"}\n{"id":7,"name":"행7"}\n{"id":8,"name":"행8"}\n{"id":9,"name":"행9"}\n{"id":10,"name":"행10"}\n{"id":11,"name":"행11"}\n{"id":12,"name":"행12"}\n{"id":13,"name":"행13"}\n{"id":14,"name":"행14"}\n{"id":15,"name":"행15"}\n{"id":16,"name":"행16"}\n{"id":17,"name":"행17"}\n{"id":18,"name":"행18"}\n{"id":19,"name":"행19"}\n', 19, 'name', '행19', True),
    ]

    def test_matrix(self):
        for name, body, row, path, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files(
                    {self.FILE: body},
                    self._c(row=row, path=path, value=value),
                )
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_bool_row_rejected(self):
        detail = self.eval_files({self.FILE: NDJSON_SIMPLE}, self._c(row=True, path="id", value=2))
        self.assertFalse(detail["ok"], detail)

    def test_empty_file_fails(self):
        detail = self.eval_files({self.FILE: ""}, self._c())
        self.assertFalse(detail["ok"], detail)
        self.assertIn("없음", str(detail["actual"]))


class NdjsonKeysAndLenTests(_OpCase):
    FILE = "out.ndjson"

    def test_keys_on_row(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": 0,
             "keys": ["id", "name", "tags"]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_keys_missing(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": 0,
             "keys": ["id", "missing"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_keys_nested_path(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_TYPES},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": 5,
             "path": "v", "keys": ["a", "b"]},
        )
        self.assertTrue(detail["ok"], detail)

    def test_keys_path_not_object(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": 0,
             "path": "tags", "keys": ["x"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_keys_row_oob(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": 8,
             "keys": ["id"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_keys_neg_row(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": -1,
             "keys": ["id"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_keys_bool_row(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": True,
             "keys": ["id"]},
        )
        self.assertFalse(detail["ok"], detail)

    def test_keys_not_list(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "k", "op": "ndjson_keys_contain", "file": self.FILE, "row": 0,
             "keys": "id"},
        )
        self.assertFalse(detail["ok"], detail)

    def test_len_tags_row0(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "l", "op": "ndjson_len_eq", "file": self.FILE, "row": 0,
             "path": "tags", "value": 2},
        )
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 2)

    def test_len_tags_row1(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "l", "op": "ndjson_len_eq", "file": self.FILE, "row": 1,
             "path": "tags", "value": 1},
        )
        self.assertTrue(detail["ok"], detail)

    def test_len_tags_empty(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "l", "op": "ndjson_len_eq", "file": self.FILE, "row": 2,
             "path": "tags", "value": 0},
        )
        self.assertTrue(detail["ok"], detail)

    def test_len_root_object(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "l", "op": "ndjson_len_eq", "file": self.FILE, "row": 0,
             "path": "", "value": 4},
        )
        self.assertTrue(detail["ok"], detail)

    def test_len_scalar_fails(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "l", "op": "ndjson_len_eq", "file": self.FILE, "row": 0,
             "path": "name", "value": 1},
        )
        self.assertFalse(detail["ok"], detail)

    def test_len_wrong(self):
        detail = self.eval_files(
            {self.FILE: NDJSON_SIMPLE},
            {"name": "l", "op": "ndjson_len_eq", "file": self.FILE, "row": 0,
             "path": "tags", "value": 9},
        )
        self.assertFalse(detail["ok"], detail)

    def test_len_missing_file(self):
        detail = self.eval_files(
            {},
            {"name": "l", "op": "ndjson_len_eq", "file": self.FILE, "row": 0, "value": 1},
        )
        self.assertFalse(detail["ok"], detail)


TEXT_SIMPLE = "첫째\n둘째\n셋째"
TEXT_CRLF = "첫째\r\n둘째\r\n셋째\r\n"
TEXT_BLANK = "갑\n\n을\n"
TEXT_HANGUL = "이름: 홍길동\n수량: 12\n비고: 없음\n"
TEXT_LONG = "".join(f"줄{i:02d} 내용-{i}\n" for i in range(40))


class TextLineEqMatrixTests(_OpCase):
    FILE = "out.txt"

    def _c(self, **kwargs):
        check = {"name": "ln", "op": "text_line_eq", "file": self.FILE, "line": 0, "value": ""}
        check.update(kwargs)
        return check

    CASES = [
        ('first', '첫째\n둘째\n셋째', 0, '첫째', True),
        ('second', '첫째\n둘째\n셋째', 1, '둘째', True),
        ('third_no_nl', '첫째\n둘째\n셋째', 2, '셋째', True),
        ('wrong', '첫째\n둘째\n셋째', 0, '둘째', False),
        ('crlf_second', '첫째\r\n둘째\r\n셋째\r\n', 1, '둘째', True),
        ('blank_middle', '갑\n\n을\n', 1, '', True),
        ('blank_third', '갑\n\n을\n', 2, '을', True),
        ('hangul_0', '이름: 홍길동\n수량: 12\n비고: 없음\n', 0, '이름: 홍길동', True),
        ('hangul_1', '이름: 홍길동\n수량: 12\n비고: 없음\n', 1, '수량: 12', True),
        ('oob', '첫째\n둘째\n셋째', 9, '첫째', False),
        ('neg', '첫째\n둘째\n셋째', -1, '첫째', False),
        ('long_00', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 0, '줄00 내용-0', True),
        ('long_01', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 1, '줄01 내용-1', True),
        ('long_01_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 1, '줄00 내용-0', False),
        ('long_02', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 2, '줄02 내용-2', True),
        ('long_02_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 2, '줄01 내용-1', False),
        ('long_03', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 3, '줄03 내용-3', True),
        ('long_03_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 3, '줄02 내용-2', False),
        ('long_04', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 4, '줄04 내용-4', True),
        ('long_04_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 4, '줄03 내용-3', False),
        ('long_05', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 5, '줄05 내용-5', True),
        ('long_05_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 5, '줄04 내용-4', False),
        ('long_06', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 6, '줄06 내용-6', True),
        ('long_06_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 6, '줄05 내용-5', False),
        ('long_07', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 7, '줄07 내용-7', True),
        ('long_07_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 7, '줄06 내용-6', False),
        ('long_08', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 8, '줄08 내용-8', True),
        ('long_08_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 8, '줄07 내용-7', False),
        ('long_09', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 9, '줄09 내용-9', True),
        ('long_09_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 9, '줄08 내용-8', False),
        ('long_10', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 10, '줄10 내용-10', True),
        ('long_10_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 10, '줄09 내용-9', False),
        ('long_11', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 11, '줄11 내용-11', True),
        ('long_11_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 11, '줄10 내용-10', False),
        ('long_12', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 12, '줄12 내용-12', True),
        ('long_12_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 12, '줄11 내용-11', False),
        ('long_13', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 13, '줄13 내용-13', True),
        ('long_13_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 13, '줄12 내용-12', False),
        ('long_14', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 14, '줄14 내용-14', True),
        ('long_14_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 14, '줄13 내용-13', False),
        ('long_15', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 15, '줄15 내용-15', True),
        ('long_15_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 15, '줄14 내용-14', False),
        ('long_16', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 16, '줄16 내용-16', True),
        ('long_16_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 16, '줄15 내용-15', False),
        ('long_17', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 17, '줄17 내용-17', True),
        ('long_17_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 17, '줄16 내용-16', False),
        ('long_18', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 18, '줄18 내용-18', True),
        ('long_18_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 18, '줄17 내용-17', False),
        ('long_19', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 19, '줄19 내용-19', True),
        ('long_19_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 19, '줄18 내용-18', False),
        ('long_20', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 20, '줄20 내용-20', True),
        ('long_20_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 20, '줄19 내용-19', False),
        ('long_21', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 21, '줄21 내용-21', True),
        ('long_21_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 21, '줄20 내용-20', False),
        ('long_22', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 22, '줄22 내용-22', True),
        ('long_22_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 22, '줄21 내용-21', False),
        ('long_23', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 23, '줄23 내용-23', True),
        ('long_23_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 23, '줄22 내용-22', False),
        ('long_24', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 24, '줄24 내용-24', True),
        ('long_24_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 24, '줄23 내용-23', False),
        ('long_25', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 25, '줄25 내용-25', True),
        ('long_25_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 25, '줄24 내용-24', False),
        ('long_26', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 26, '줄26 내용-26', True),
        ('long_26_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 26, '줄25 내용-25', False),
        ('long_27', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 27, '줄27 내용-27', True),
        ('long_27_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 27, '줄26 내용-26', False),
        ('long_28', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 28, '줄28 내용-28', True),
        ('long_28_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 28, '줄27 내용-27', False),
        ('long_29', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 29, '줄29 내용-29', True),
        ('long_29_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 29, '줄28 내용-28', False),
        ('long_30', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 30, '줄30 내용-30', True),
        ('long_30_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 30, '줄29 내용-29', False),
        ('long_31', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 31, '줄31 내용-31', True),
        ('long_31_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 31, '줄30 내용-30', False),
        ('long_32', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 32, '줄32 내용-32', True),
        ('long_32_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 32, '줄31 내용-31', False),
        ('long_33', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 33, '줄33 내용-33', True),
        ('long_33_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 33, '줄32 내용-32', False),
        ('long_34', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 34, '줄34 내용-34', True),
        ('long_34_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 34, '줄33 내용-33', False),
        ('long_35', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 35, '줄35 내용-35', True),
        ('long_35_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 35, '줄34 내용-34', False),
        ('long_36', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 36, '줄36 내용-36', True),
        ('long_36_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 36, '줄35 내용-35', False),
        ('long_37', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 37, '줄37 내용-37', True),
        ('long_37_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 37, '줄36 내용-36', False),
        ('long_38', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 38, '줄38 내용-38', True),
        ('long_38_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 38, '줄37 내용-37', False),
        ('long_39', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 39, '줄39 내용-39', True),
        ('long_39_wrong', '줄00 내용-0\n줄01 내용-1\n줄02 내용-2\n줄03 내용-3\n줄04 내용-4\n줄05 내용-5\n줄06 내용-6\n줄07 내용-7\n줄08 내용-8\n줄09 내용-9\n줄10 내용-10\n줄11 내용-11\n줄12 내용-12\n줄13 내용-13\n줄14 내용-14\n줄15 내용-15\n줄16 내용-16\n줄17 내용-17\n줄18 내용-18\n줄19 내용-19\n줄20 내용-20\n줄21 내용-21\n줄22 내용-22\n줄23 내용-23\n줄24 내용-24\n줄25 내용-25\n줄26 내용-26\n줄27 내용-27\n줄28 내용-28\n줄29 내용-29\n줄30 내용-30\n줄31 내용-31\n줄32 내용-32\n줄33 내용-33\n줄34 내용-34\n줄35 내용-35\n줄36 내용-36\n줄37 내용-37\n줄38 내용-38\n줄39 내용-39\n', 39, '줄38 내용-38', False),
    ]

    def test_matrix(self):
        for name, body, line, value, ok in self.CASES:
            with self.subTest(name):
                detail = self.eval_files({self.FILE: body}, self._c(line=line, value=value))
                self.assertEqual(detail["ok"], ok, (name, detail))

    def test_bool_line_rejected(self):
        detail = self.eval_files({self.FILE: TEXT_SIMPLE}, self._c(line=False, value="첫째"))
        self.assertFalse(detail["ok"], detail)

    def test_empty_file_fails(self):
        detail = self.eval_files({self.FILE: ""}, self._c(line=0, value=""))
        self.assertFalse(detail["ok"], detail)
        self.assertIn("없음", str(detail["actual"]))

    def test_missing_file_and_invalid_utf8(self):
        self.assertFalse(self.eval_files({}, self._c())["ok"])
        self.assertFalse(self.eval_bytes(self.FILE, b"\xff\xfe hi\n", self._c())["ok"])


class TextLineCountAndContainsTests(_OpCase):
    FILE = "out.txt"

    def test_count_simple_three(self):
        detail = self.eval_files(
            {self.FILE: TEXT_SIMPLE},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": 3},
        )
        self.assertTrue(detail["ok"], detail)
        self.assertEqual(detail["actual"], 3)

    def test_count_crlf_three(self):
        detail = self.eval_files(
            {self.FILE: TEXT_CRLF},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": 3},
        )
        self.assertTrue(detail["ok"], detail)

    def test_count_empty_zero(self):
        detail = self.eval_files(
            {self.FILE: ""},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": 0},
        )
        self.assertTrue(detail["ok"], detail)

    def test_count_blank_three(self):
        detail = self.eval_files(
            {self.FILE: TEXT_BLANK},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": 3},
        )
        self.assertTrue(detail["ok"], detail)

    def test_count_long_forty(self):
        detail = self.eval_files(
            {self.FILE: TEXT_LONG},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": 40},
        )
        self.assertTrue(detail["ok"], detail)

    def test_count_wrong(self):
        detail = self.eval_files(
            {self.FILE: TEXT_SIMPLE},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": 2},
        )
        self.assertFalse(detail["ok"], detail)

    def test_count_numeric_string(self):
        detail = self.eval_files(
            {self.FILE: TEXT_SIMPLE},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": "3"},
        )
        self.assertTrue(detail["ok"], detail)

    def test_count_missing_file(self):
        detail = self.eval_files(
            {},
            {"name": "n", "op": "text_line_count_eq", "file": self.FILE, "value": 1},
        )
        self.assertFalse(detail["ok"], detail)

    def test_contains_hit(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": 0,
             "value": "홍길동"},
        )
        self.assertTrue(detail["ok"], detail)

    def test_contains_miss(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": 0,
             "value": "이순신"},
        )
        self.assertFalse(detail["ok"], detail)

    def test_contains_full_line(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": 1,
             "value": "수량: 12"},
        )
        self.assertTrue(detail["ok"], detail)

    def test_contains_oob(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": 9,
             "value": "홍"},
        )
        self.assertFalse(detail["ok"], detail)

    def test_contains_neg(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": -1,
             "value": "홍"},
        )
        self.assertFalse(detail["ok"], detail)

    def test_contains_bool_line(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": True,
             "value": "수량"},
        )
        self.assertFalse(detail["ok"], detail)

    def test_contains_non_string_value(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": 1,
             "value": 12},
        )
        self.assertFalse(detail["ok"], detail)

    def test_contains_empty_needle_matches(self):
        detail = self.eval_files(
            {self.FILE: TEXT_HANGUL},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": 0,
             "value": ""},
        )
        self.assertTrue(detail["ok"], detail)

    def test_contains_missing_file(self):
        detail = self.eval_files(
            {},
            {"name": "c", "op": "text_line_contains", "file": self.FILE, "line": 0,
             "value": "x"},
        )
        self.assertFalse(detail["ok"], detail)


class ExceptionPathCatalogTests(_OpCase):
    """모든 신규 지목 연산자의 공통 예외 — 파일 없음·깨진 바이트·좌표 거부."""

    def test_missing_file_fails_every_pinpoint_op(self):
        checks = [
            {"name": "a", "op": "json_len_eq", "file": "m.json", "value": 0},
            {"name": "b", "op": "json_type_eq", "file": "m.json", "value": "object"},
            {"name": "c", "op": "json_len_ge", "file": "m.json", "value": 0},
            {"name": "d", "op": "json_array_item_eq", "file": "m.json", "index": 0, "value": 1},
            {"name": "e", "op": "json_keys_contain", "file": "m.json", "keys": ["id"]},
            {"name": "f", "op": "csv_row_count_eq", "file": "m.csv", "value": 1},
            {"name": "g", "op": "csv_col_count_eq", "file": "m.csv", "row": 0, "value": 1},
            {"name": "h", "op": "csv_header_eq", "file": "m.csv", "values": ["a"]},
            {"name": "i", "op": "csv_row_eq", "file": "m.csv", "row": 0, "values": ["a"]},
            {"name": "j", "op": "ndjson_count_eq", "file": "m.ndjson", "value": 1},
            {"name": "k", "op": "ndjson_field_eq", "file": "m.ndjson", "row": 0, "value": 1},
            {"name": "l", "op": "ndjson_keys_contain", "file": "m.ndjson", "row": 0, "keys": ["id"]},
            {"name": "m", "op": "ndjson_len_eq", "file": "m.ndjson", "row": 0, "value": 1},
            {"name": "n", "op": "text_line_eq", "file": "m.txt", "line": 0, "value": "x"},
            {"name": "o", "op": "text_line_count_eq", "file": "m.txt", "value": 1},
            {"name": "p", "op": "text_line_contains", "file": "m.txt", "line": 0, "value": "x"},
        ]
        for check in checks:
            with self.subTest(check["op"]):
                detail = self.eval_files({}, check)
                self.assertFalse(detail["ok"], detail)

    def test_invalid_utf8_fails_textish_ops(self):
        payload = b"\xff\xfe\x00 broken"
        checks = [
            ("out.json", {"name": "a", "op": "json_len_eq", "file": "out.json", "value": 0}),
            ("out.csv", {"name": "b", "op": "csv_row_count_eq", "file": "out.csv", "value": 0}),
            ("out.ndjson", {"name": "c", "op": "ndjson_count_eq", "file": "out.ndjson", "value": 0}),
            ("out.txt", {"name": "d", "op": "text_line_count_eq", "file": "out.txt", "value": 0}),
            ("out.txt", {"name": "e", "op": "text_line_eq", "file": "out.txt", "line": 0, "value": "x"}),
        ]
        for name, check in checks:
            with self.subTest(check["op"]):
                detail = self.eval_bytes(name, payload, check)
                self.assertFalse(detail["ok"], detail)

    def test_no_cli_on_extra_ops(self):
        _checks, runner, _schema = load_core()
        original = runner.run_cli

        def boom(*_a, **_k):
            raise AssertionError("file op 가 CLI 를 부르면 안 된다")

        runner.run_cli = boom
        try:
            with tempfile.TemporaryDirectory() as sub_dir:
                _write(sub_dir, "out.json", _json({"items": [1, 2, 3], "name": "갑"}))
                _write(sub_dir, "out.csv", "name,qty\n갑,1\n")
                _write(sub_dir, "out.ndjson", '{"id":1,"tags":[1,2]}\n')
                _write(sub_dir, "out.txt", "하나\n둘\n")
                samples = [
                    {"name": "a", "op": "json_type_eq", "file": "out.json", "path": "items",
                     "value": "array"},
                    {"name": "b", "op": "json_len_ge", "file": "out.json", "path": "items",
                     "value": 3},
                    {"name": "c", "op": "json_array_item_eq", "file": "out.json", "path": "items",
                     "index": 1, "value": 2},
                    {"name": "d", "op": "csv_col_count_eq", "file": "out.csv", "row": 0, "value": 2},
                    {"name": "e", "op": "csv_header_eq", "file": "out.csv",
                     "values": ["name", "qty"]},
                    {"name": "f", "op": "csv_row_eq", "file": "out.csv", "row": 1,
                     "values": ["갑", "1"]},
                    {"name": "g", "op": "ndjson_keys_contain", "file": "out.ndjson", "row": 0,
                     "keys": ["id", "tags"]},
                    {"name": "h", "op": "ndjson_len_eq", "file": "out.ndjson", "row": 0,
                     "path": "tags", "value": 2},
                    {"name": "i", "op": "text_line_count_eq", "file": "out.txt", "value": 2},
                    {"name": "j", "op": "text_line_contains", "file": "out.txt", "line": 0,
                     "value": "나"},
                ]
                for check in samples:
                    with self.subTest(check["op"]):
                        detail = runner.eval_check(check, {}, sub_dir, {}, "unused-rhwp")
                        self.assertTrue(detail["ok"], detail)
        finally:
            runner.run_cli = original


class SchemaPinpointOpsTests(unittest.TestCase):
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

    def test_cmd_rejected_on_every_extra_op(self):
        _checks, _runner, schema = load_core()
        pack = {"id": "p", "axis": "자동화"}
        extras = [
            {"name": "c", "op": "json_type_eq", "file": "out.json", "value": "object",
             "cmd": ["info"]},
            {"name": "c", "op": "csv_header_eq", "file": "out.csv", "values": ["a"],
             "cmd": ["info"]},
            {"name": "c", "op": "text_line_contains", "file": "out.txt", "line": 0,
             "value": "x", "cmd": ["info"]},
        ]
        for check in extras:
            errors = []
            schema.validate_task(self._task(check), pack, None, errors)
            self.assertTrue(any("CLI" in e for e in errors), (check["op"], errors))


if __name__ == "__main__":
    unittest.main()
