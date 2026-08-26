"""[#5224/#5228] gym 교차형식 차등 오라클 계약 — 순수 판정·보고·예외.

바이너리 없이 쌍둥이 짝짓기·본문 해시·관측 대조·오검출 관문·보고 봉투를 고정한다.
CLI run 은 주입해 "관측이 갈렸을 때" 를 합성한다. 예외 경로는 관측/도구 오류로
접히되, 짝짓기·해시 정직 조항은 그대로다.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import math
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "differential.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_differential", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _touch(root, rel):
    path = os.path.join(root, *rel.split("/"))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as fh:
        fh.write(b"x")
    return path


def _value(v):
    return {"kind": "value", "value": v}


class TwinDiscoveryTests(unittest.TestCase):
    def test_pairs_same_stem_and_ignores_unpaired(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "a.hwp")
            _touch(d, "a.hwpx")
            _touch(d, "only.hwp")
            _touch(d, "note.txt")
            _touch(d, "nested/c.hwp")
            _touch(d, "nested/c.hwpx")
            pairs = mod.find_twins_in(d)
        self.assertEqual(
            pairs,
            [
                ("a", "a.hwp", "a.hwpx"),
                ("c", "nested/c.hwp", "nested/c.hwpx"),
            ],
        )

    def test_extension_case_insensitive(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "Doc.HWP")
            _touch(d, "Doc.Hwpx")
            self.assertEqual(mod.find_twins_in(d), [("Doc", "Doc.HWP", "Doc.Hwpx")])

    def test_missing_dir_is_empty(self):
        mod = load()
        self.assertEqual(mod.find_twins_in(os.path.join("no", "such", "samples")), [])

    def test_root_prefix_matches_repo_layout(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "samples/x.hwp")
            _touch(d, "samples/x.hwpx")
            pairs = mod.find_twins_in(os.path.join(d, "samples"), root=d)
        self.assertEqual(pairs, [("x", "samples/x.hwp", "samples/x.hwpx")])

    def test_colliding_stems_prefer_same_directory_then_shallow(self):
        mod = load()
        # 루트와 sub 둘 다 짝이 있으면 디렉터리 경로가 앞선 루트를 고른다.
        hwps = ["z.hwp", "sub/z.hwp", "aa/z.hwp"]
        hwpxs = ["sub/z.hwpx", "z.hwpx"]
        self.assertEqual(mod.pick_twin_paths(hwps, hwpxs), ("z.hwp", "z.hwpx"))
        self.assertEqual(
            mod.pick_twin_paths(list(reversed(hwps)), list(reversed(hwpxs))),
            ("z.hwp", "z.hwpx"),
        )
        # 같은 폴더 짝이 sub 에만 있으면 그 짝.
        self.assertEqual(
            mod.pick_twin_paths(["aa/z.hwp", "sub/z.hwp"], ["sub/z.hwpx"]),
            ("sub/z.hwp", "sub/z.hwpx"),
        )
        # 같은 폴더 짝이 없으면 얕은 경로.
        self.assertEqual(
            mod.pick_twin_paths(["deep/n/x.hwp", "x.hwp"], ["other/x.hwpx"]),
            ("x.hwp", "other/x.hwpx"),
        )

    def test_select_pairs_limit_is_prefix(self):
        mod = load()
        pairs = [("b", "b.hwp", "b.hwpx"), ("a", "a.hwp", "a.hwpx")]
        self.assertEqual(mod.select_pairs(pairs, 0), pairs)
        self.assertEqual(mod.select_pairs(pairs, 1), [pairs[0]])


class BodyHashTests(unittest.TestCase):
    def test_whitespace_ignored_and_deterministic(self):
        mod = load()
        env_a = {"pages": [{"text": "가 나\n다"}, {"text": "라"}]}
        env_b = {"pages": [{"text": "가나다라"}]}
        ha = mod.body_hash_from_env(env_a)
        hb = mod.body_hash_from_env(env_b)
        self.assertEqual(ha, hb)
        self.assertEqual(ha, mod.body_hash_from_env(env_a))
        self.assertEqual(len(ha), 64)

    def test_missing_envelope_is_not_identity(self):
        mod = load()
        self.assertIsNone(mod.body_hash_from_env(None))
        self.assertIsNone(mod.body_hash_from_env({"no": "pages"}))
        self.assertFalse(mod.same_body_hash(None, None))
        self.assertFalse(mod.same_body_hash("a" * 64, None))
        self.assertTrue(mod.same_body_hash("a" * 64, "a" * 64))


class ObservationTests(unittest.TestCase):
    def test_value_missing_and_nojson_are_distinct(self):
        mod = load()
        self.assertEqual(
            mod.observation_from_result(0, {"pageCount": 6}, "pageCount"),
            _value(6),
        )
        self.assertEqual(
            mod.observation_from_result(0, {"pageCount": 6}, "tableCount"),
            {"kind": "missing", "key": "tableCount"},
        )
        self.assertEqual(
            mod.observation_from_result(2, None, "pageCount"),
            {"kind": "nojson", "code": 2},
        )
        self.assertEqual(
            mod.observation_from_result(0, ["not-a-dict"], "pageCount"),
            {"kind": "badenv", "code": 0},
        )

    def test_numeric_int_and_float_match(self):
        mod = load()
        self.assertTrue(mod.observations_equal(_value(6), _value(6.0)))
        self.assertTrue(mod.observations_equal(6, 6.0))
        self.assertFalse(mod.observations_equal(_value(6), _value(7)))

    def test_bool_is_not_int(self):
        mod = load()
        self.assertFalse(mod.observations_equal(True, 1))
        self.assertTrue(mod.observations_equal(True, True))

    def test_dict_key_order_does_not_matter(self):
        mod = load()
        self.assertTrue(
            mod.observations_equal(
                {"kind": "value", "value": {"b": 1, "a": 2}},
                {"value": {"a": 2.0, "b": 1}, "kind": "value"},
            )
        )

    def test_nojson_does_not_collide_with_string_value(self):
        mod = load()
        self.assertFalse(
            mod.observations_equal(
                {"kind": "nojson", "code": 1},
                _value("exit1"),
            )
        )

    def test_display_keeps_cli_shape(self):
        mod = load()
        self.assertEqual(mod.observation_display(_value(6)), 6)
        self.assertEqual(mod.observation_display({"kind": "nojson", "code": 3}), "exit3")
        self.assertIsNone(mod.observation_display({"kind": "missing", "key": "x"}))


class ClassificationTests(unittest.TestCase):
    def test_gate_matrix(self):
        mod = load()
        cases = [
            (True, True, [], None),
            (False, True, [{"observation": "pageCount"}], "other-doc"),
            (True, True, [{"observation": "pageCount"}], "contradiction"),
            (True, False, [{"observation": "pageCount"}], "review"),
            (False, False, [{"observation": "pageCount"}], "other-doc"),
        ]
        for body_same, ir_identical, diverged, expected in cases:
            self.assertEqual(
                mod.classify_pair(body_same, ir_identical, diverged),
                expected,
                (body_same, ir_identical, bool(diverged)),
            )


class CompareTwinsTests(unittest.TestCase):
    def _run_from(self, table):
        def run(args):
            key = tuple(args)
            if key not in table:
                raise AssertionError(f"예상하지 않은 CLI: {args}")
            return table[key]

        return run

    def test_matching_pair_is_silent(self):
        mod = load()
        run = self._run_from(
            {
                ("info", "a.hwp", "--json"): (0, {"pageCount": 3}),
                ("info", "a.hwpx", "--json"): (0, {"pageCount": 3.0}),
            }
        )
        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(compared, 1)
        self.assertEqual(other, 0)
        self.assertEqual(findings, [])

    def test_other_document_is_excluded(self):
        mod = load()
        run = self._run_from(
            {
                ("info", "a.hwp", "--json"): (0, {"pageCount": 1}),
                ("info", "a.hwpx", "--json"): (0, {"pageCount": 2}),
                ("export-text", "a.hwp", "--json"): (0, {"pages": [{"text": "가"}]}),
                ("export-text", "a.hwpx", "--json"): (0, {"pages": [{"text": "나"}]}),
            }
        )
        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(compared, 1)
        self.assertEqual(other, 1)
        self.assertEqual(findings, [])

    def test_missing_body_hash_is_not_contradiction(self):
        mod = load()
        run = self._run_from(
            {
                ("info", "a.hwp", "--json"): (0, {"pageCount": 1}),
                ("info", "a.hwpx", "--json"): (0, {"pageCount": 2}),
                ("export-text", "a.hwp", "--json"): (1, None),
                ("export-text", "a.hwpx", "--json"): (1, None),
            }
        )
        _compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(other, 1)
        self.assertEqual(findings, [])

    def test_ir_identical_divergence_is_contradiction(self):
        mod = load()
        body = (0, {"pages": [{"text": "같은본문"}]})
        run = self._run_from(
            {
                ("info", "a.hwp", "--json"): (0, {"pageCount": 6}),
                ("info", "a.hwpx", "--json"): (0, {"pageCount": 7}),
                ("export-text", "a.hwp", "--json"): body,
                ("export-text", "a.hwpx", "--json"): body,
                ("ir-diff", "a.hwp", "a.hwpx", "--json"): (0, {"identical": True, "diffCount": 0}),
            }
        )
        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual((compared, other), (1, 0))
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0]["severity"], "contradiction")
        self.assertTrue(findings[0]["irIdentical"])
        self.assertEqual(findings[0]["irDiffCount"], 0)
        self.assertEqual(findings[0]["diverged"][0]["observation"], "pageCount")
        self.assertEqual(findings[0]["diverged"][0]["hwp"], _value(6))
        self.assertEqual(findings[0]["diverged"][0]["hwpx"], _value(7))

    def test_ir_different_divergence_is_review(self):
        mod = load()
        body = (0, {"pages": [{"text": "같은본문"}]})
        run = self._run_from(
            {
                ("info", "a.hwp", "--json"): (0, {"pageCount": 6}),
                ("info", "a.hwpx", "--json"): (0, {"pageCount": 7}),
                ("export-text", "a.hwp", "--json"): body,
                ("export-text", "a.hwpx", "--json"): body,
                ("ir-diff", "a.hwp", "a.hwpx", "--json"): (0, {"identical": False, "diffCount": 4}),
            }
        )
        _compared, _other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(findings[0]["severity"], "review")
        self.assertEqual(findings[0]["irDiffCount"], 4)

    def test_findings_sorted_by_stem(self):
        mod = load()
        body = (0, {"pages": [{"text": "x"}]})

        def run(args):
            if args[0] == "info":
                return (0, {"pageCount": 1 if args[1].endswith(".hwp") else 2})
            if args[0] == "export-text":
                return body
            if args[0] == "ir-diff":
                return (0, {"identical": True, "diffCount": 0})
            raise AssertionError(args)

        _compared, _other, findings = mod.compare_twins(
            [("z", "z.hwp", "z.hwpx"), ("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual([row["stem"] for row in findings], ["a", "z"])


class ReportTests(unittest.TestCase):
    def test_kind_and_schema_and_counts(self):
        mod = load()
        findings = [
            mod.make_finding(
                "b", "b.hwp", "b.hwpx", False, 1,
                [{"observation": "tableCount", "hwp": _value(1), "hwpx": _value(2)}],
            ),
            mod.make_finding(
                "a", "a.hwp", "a.hwpx", True, 0,
                [{"observation": "pageCount", "hwp": _value(6), "hwpx": _value(7)}],
            ),
        ]
        report = mod.build_report(
            bin_name="rhwp",
            pairs_count=4,
            compared=24,
            other_doc=1,
            findings=findings,
        )
        self.assertEqual(report["kind"], "gymDifferential")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertEqual(report["kind"], mod.REPORT_KIND)
        self.assertEqual(report["schemaVersion"], mod.SCHEMA_VERSION)
        self.assertFalse(report["ok"])
        self.assertEqual(report["pairs"], 4)
        self.assertEqual(report["observationsCompared"], 24)
        self.assertEqual(report["sameNameDifferentDocument"], 1)
        self.assertEqual(report["contradictions"], 1)
        self.assertEqual(report["reviews"], 1)
        self.assertEqual([row["stem"] for row in report["findings"]], ["a", "b"])
        self.assertEqual(report["runner"], {"bin": "rhwp"})

    def test_ok_when_no_contradiction(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=2, compared=12, other_doc=0, findings=[]
        )
        self.assertTrue(report["ok"])
        self.assertEqual(report["contradictions"], 0)
        self.assertEqual(report["reviews"], 0)

    def test_summary_and_report_file_are_deterministic(self):
        mod = load()
        finding = mod.make_finding(
            "doc", "doc.hwp", "doc.hwpx", True, 0,
            [{"observation": "pageCount", "hwp": _value(6), "hwpx": _value(7)}],
        )
        report = mod.build_report(
            bin_name="rhwp", pairs_count=1, compared=6, other_doc=0, findings=[finding]
        )
        lines = mod.render_summary(report, "out.json")
        self.assertIn("쌍둥이 1쌍", lines[0])
        self.assertIn("!!", "\n".join(lines))
        self.assertIn("pageCount 6≠7", "\n".join(lines))
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "differential-report.json")
            mod.write_report(report, path)
            raw = Path(path).read_bytes()
            self.assertFalse(raw.startswith(b"\xef\xbb\xbf"))
            self.assertNotIn(b"\r\n", raw)
            loaded = json.loads(raw.decode("utf-8"))
        self.assertEqual(loaded["kind"], "gymDifferential")
        self.assertEqual(loaded["schemaVersion"], "1.0")
        self.assertEqual(loaded["ok"], report["ok"])
        self.assertEqual(loaded["contradictions"], report["contradictions"])
        self.assertEqual(loaded["findings"], report["findings"])


class ExceptionKindTests(unittest.TestCase):
    def test_timeout_kinds(self):
        mod = load()
        self.assertEqual(mod.exception_kind(TimeoutError("t")), "timeout")
        self.assertEqual(
            mod.exception_kind(subprocess.TimeoutExpired(cmd=["rhwp"], timeout=1)),
            "timeout",
        )

    def test_file_not_found_is_missing_bin_in_cli_hash_ir(self):
        mod = load()
        err = FileNotFoundError("rhwp")
        for context in ("cli", "hash", "ir", "observe", "pair"):
            self.assertEqual(mod.exception_kind(err, context=context), "missing-bin", context)

    def test_permission_and_oserror(self):
        mod = load()
        self.assertEqual(mod.exception_kind(PermissionError("no")), "permission")
        self.assertEqual(mod.exception_kind(OSError("io")), "os-error")

    def test_unicode_and_json(self):
        mod = load()
        self.assertEqual(
            mod.exception_kind(UnicodeDecodeError("utf-8", b"\xff", 0, 1, "x")),
            "decode-error",
        )
        self.assertEqual(mod.exception_kind(UnicodeError("u")), "decode-error")
        self.assertEqual(mod.exception_kind(json.JSONDecodeError("e", "x", 0)), "value-error")

    def test_type_value_runtime_unknown(self):
        mod = load()
        self.assertEqual(mod.exception_kind(TypeError("t")), "type-error")
        self.assertEqual(mod.exception_kind(ValueError("v")), "value-error")
        self.assertEqual(mod.exception_kind(RuntimeError("r")), "cli-error")
        self.assertEqual(mod.exception_kind(Exception("e")), "unexpected")
        self.assertEqual(mod.exception_kind(None), "unexpected")

    def test_fatal_exceptions_are_flagged(self):
        mod = load()
        self.assertTrue(mod.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(mod.is_fatal_exception(SystemExit(2)))
        self.assertTrue(mod.is_fatal_exception(MemoryError()))
        self.assertTrue(mod.is_fatal_exception(GeneratorExit()))
        self.assertFalse(mod.is_fatal_exception(OSError("x")))
        self.assertFalse(mod.is_fatal_exception(ValueError("x")))
        self.assertFalse(mod.is_fatal_exception(FileNotFoundError("x")))

    def test_exception_observation_shape(self):
        mod = load()
        obs = mod.exception_observation(FileNotFoundError("rhwp"), context="cli")
        self.assertEqual(obs["kind"], "missing-bin")
        self.assertEqual(obs["error"], "FileNotFoundError")
        self.assertIn("rhwp", obs["head"])

    def test_exception_tool_error_shape(self):
        mod = load()
        row = mod.exception_tool_error(PermissionError("denied"), "write", extra={"path": "out.json"})
        self.assertEqual(row["where"], "write")
        self.assertEqual(row["kind"], "permission")
        self.assertEqual(row["error"], "PermissionError")
        self.assertEqual(row["path"], "out.json")

    def test_exception_kind_catalog_covers_mapped_types(self):
        mod = load()
        for exc_type, kind in mod.EXCEPTION_KIND_BY_TYPE.items():
            if exc_type is subprocess.TimeoutExpired:
                exc = subprocess.TimeoutExpired(cmd=["rhwp"], timeout=1)
            elif exc_type is UnicodeDecodeError:
                exc = UnicodeDecodeError("utf-8", b"\xff", 0, 1, "x")
            elif exc_type is UnicodeEncodeError:
                exc = UnicodeEncodeError("ascii", "한", 0, 1, "x")
            else:
                try:
                    exc = exc_type("x")
                except Exception:
                    continue
            self.assertEqual(mod.exception_kind(exc), kind, exc_type.__name__)
            self.assertIn(kind, mod.OBSERVATION_KINDS)


class TruncateAndTimeoutHelperTests(unittest.TestCase):
    def test_truncate_head_none_and_limit(self):
        mod = load()
        self.assertEqual(mod.truncate_head(None), "")
        self.assertEqual(mod.truncate_head("abcdef", 3), "abc")
        self.assertEqual(mod.truncate_head("abcdef", 0), "")
        self.assertEqual(mod.truncate_head("abcdef", -1), "")
        self.assertEqual(mod.truncate_head(1234, 2), "12")
        self.assertEqual(mod.truncate_head("abcdef", "nope"), "abcdef"[:mod.HEAD_LIMIT])

    def test_truncate_head_non_str_that_cannot_str(self):
        mod = load()

        class Boom:
            def __str__(self):
                raise RuntimeError("no")

        self.assertEqual(mod.truncate_head(Boom()), "")

    def test_normalize_timeout(self):
        mod = load()
        self.assertEqual(mod.normalize_timeout(30), 30)
        self.assertEqual(mod.normalize_timeout("15"), 15)
        self.assertEqual(mod.normalize_timeout(0), 0)
        self.assertEqual(mod.normalize_timeout(-3), 0)
        self.assertEqual(mod.normalize_timeout(None), 0)
        self.assertEqual(mod.normalize_timeout("x"), 0)
        self.assertEqual(mod.normalize_timeout(1.9), 1)

    def test_normalize_limit(self):
        mod = load()
        self.assertEqual(mod.normalize_limit(0), 0)
        self.assertEqual(mod.normalize_limit(3), 3)
        self.assertEqual(mod.normalize_limit(-1), 0)
        self.assertEqual(mod.normalize_limit(None), 0)
        self.assertEqual(mod.normalize_limit("2"), 2)
        self.assertEqual(mod.normalize_limit("nope"), 0)

    def test_observation_kind_helpers(self):
        mod = load()
        self.assertEqual(mod.observation_kind_of(_value(1)), "value")
        self.assertEqual(mod.observation_kind_of({"kind": "nojson", "code": 2}), "nojson")
        self.assertIsNone(mod.observation_kind_of({}))
        self.assertIsNone(mod.observation_kind_of("x"))
        self.assertTrue(mod.is_known_observation_kind("value"))
        self.assertTrue(mod.is_known_observation_kind("timeout"))
        self.assertFalse(mod.is_known_observation_kind("other-doc"))
        self.assertFalse(mod.is_error_observation(_value(1)))
        self.assertTrue(mod.is_error_observation({"kind": "timeout", "error": "TimeoutError"}))

    def test_is_sha256_hex(self):
        mod = load()
        good = "a" * 64
        self.assertTrue(mod.is_sha256_hex(good))
        self.assertFalse(mod.is_sha256_hex("A" * 64))
        self.assertFalse(mod.is_sha256_hex("a" * 63))
        self.assertFalse(mod.is_sha256_hex("g" * 64))
        self.assertFalse(mod.is_sha256_hex(None))
        self.assertFalse(mod.is_sha256_hex(1))

    def test_hash_text_rejects_non_str(self):
        mod = load()
        self.assertIsNone(mod.hash_text(None))
        with self.assertRaises(TypeError):
            mod.hash_text(b"ab")
        self.assertEqual(
            mod.hash_text("가"),
            hashlib.sha256("가".encode("utf-8", "replace")).hexdigest(),
        )


class ObservationEqualityEdgeTests(unittest.TestCase):
    def test_nan_both_sides_are_equal(self):
        mod = load()
        self.assertTrue(mod.observations_equal(float("nan"), float("nan")))
        self.assertTrue(mod.observations_equal(_value(float("nan")), _value(float("nan"))))

    def test_inf_sign_matters(self):
        mod = load()
        self.assertTrue(mod.observations_equal(float("inf"), float("inf")))
        self.assertFalse(mod.observations_equal(float("inf"), float("-inf")))
        self.assertFalse(mod.observations_equal(_value(float("inf")), _value(float("-inf"))))

    def test_nested_list_and_tuple(self):
        mod = load()
        self.assertTrue(mod.observations_equal([1, [2, 3.0]], [1.0, [2, 3]]))
        self.assertTrue(mod.observations_equal((1, 2.0), (1.0, 2)))
        self.assertFalse(mod.observations_equal([1, 2], [1, 2, 3]))
        self.assertFalse(mod.observations_equal((1,), [1]))

    def test_bytes_are_not_strings(self):
        mod = load()
        self.assertFalse(mod.observations_equal(b"ab", "ab"))
        self.assertFalse(mod.observations_equal(_value(b"ab"), _value("ab")))

    def test_error_payload_equality(self):
        mod = load()
        left = {"kind": "timeout", "error": "TimeoutError", "head": "x"}
        right = {"kind": "timeout", "error": "TimeoutError", "head": "x"}
        other = {"kind": "timeout", "error": "TimeoutError", "head": "y"}
        self.assertTrue(mod.observations_equal(left, right))
        self.assertFalse(mod.observations_equal(left, other))
        self.assertFalse(mod.observations_equal(left, _value("timeout")))

    def test_missing_and_nojson_and_badenv_are_distinct(self):
        mod = load()
        self.assertFalse(mod.observations_equal(
            {"kind": "missing", "key": "pageCount"},
            {"kind": "nojson", "code": None},
        ))
        self.assertFalse(mod.observations_equal(
            {"kind": "badenv", "code": 0},
            {"kind": "nojson", "code": 0},
        ))

    def test_true_false_zero_one_table(self):
        mod = load()
        table = [
            (True, True, True),
            (False, False, True),
            (True, False, False),
            (True, 1, False),
            (False, 0, False),
            (1, 1.0, True),
            (0, 0.0, True),
            ("0", 0, False),
            (None, None, True),
            (None, 0, False),
        ]
        for left, right, expected in table:
            self.assertEqual(mod.observations_equal(left, right), expected, (left, right))
            self.assertEqual(mod.observations_equal(right, left), expected, (right, left))


class PairingHonestyTests(unittest.TestCase):
    """같은 디렉터리 우선, 그다음 얕고 사전순. walk 순서에 의존하지 않는다."""

    def test_same_directory_wins_even_if_other_is_shallower(self):
        mod = load()
        # 같은 폴더 짝이 deep 에만 있으면 그 짝. 루트의 hwp 만으로는 짝이 아니다.
        self.assertEqual(
            mod.pick_twin_paths(["z.hwp", "deep/z.hwp"], ["deep/z.hwpx"]),
            ("deep/z.hwp", "deep/z.hwpx"),
        )

    def test_lexicographically_first_same_dir_wins(self):
        mod = load()
        self.assertEqual(
            mod.pick_twin_paths(
                ["b/z.hwp", "a/z.hwp"],
                ["b/z.hwpx", "a/z.hwpx"],
            ),
            ("a/z.hwp", "a/z.hwpx"),
        )
        self.assertEqual(
            mod.pick_twin_paths(
                ["b/z.hwp", "a/z.hwp"],
                ["a/z.hwpx", "b/z.hwpx"],
            ),
            ("a/z.hwp", "a/z.hwpx"),
        )

    def test_first_file_in_same_dir_is_kept(self):
        mod = load()
        # 같은 디렉터리에 hwp 가 여러 개면 먼저 본(정렬된) 경로.
        self.assertEqual(
            mod.pick_twin_paths(["dir/b.hwp", "dir/a.hwp"], ["dir/x.hwpx"]),
            ("dir/a.hwp", "dir/x.hwpx"),
        )

    def test_empty_or_one_side_is_none(self):
        mod = load()
        self.assertIsNone(mod.pick_twin_paths([], ["a.hwpx"]))
        self.assertIsNone(mod.pick_twin_paths(["a.hwp"], []))
        self.assertIsNone(mod.pick_twin_paths([], []))
        self.assertIsNone(mod.pick_twin_paths(None, ["a.hwpx"]))

    def test_non_string_paths_do_not_change_ranking_of_rest(self):
        mod = load()
        self.assertEqual(
            mod.pick_twin_paths([None, 1, "z.hwp", ""], ["z.hwpx", None]),
            ("z.hwp", "z.hwpx"),
        )

    def test_backslash_normalized_before_dirname(self):
        mod = load()
        self.assertEqual(
            mod.pick_twin_paths(["sub\\z.hwp"], ["sub/z.hwpx"]),
            ("sub\\z.hwp", "sub/z.hwpx"),
        )

    def test_walk_order_does_not_change_pairs(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "b/x.hwp")
            _touch(d, "b/x.hwpx")
            _touch(d, "a/x.hwp")
            _touch(d, "a/x.hwpx")
            first = mod.find_twins_in(d)
            _touch(d, "c/x.hwp")
            _touch(d, "c/x.hwpx")
            second = mod.find_twins_in(d)
        # 같은 줄기 x 의 대표는 디렉터리 사전순 a.
        self.assertEqual(first, [("x", "a/x.hwp", "a/x.hwpx")])
        self.assertEqual(second, [("x", "a/x.hwp", "a/x.hwpx")])

    def test_unpaired_never_invented(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "only.hwp")
            _touch(d, "other.hwpx")
            _touch(d, "nested/only.hwp")
            self.assertEqual(mod.find_twins_in(d), [])

    def test_file_instead_of_dir_is_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            path = _touch(d, "notdir.hwp")
            self.assertEqual(mod.find_twins_in(path), [])

    def test_oserror_on_isdir_is_empty(self):
        mod = load()
        with mock.patch.object(mod.os.path, "isdir", side_effect=OSError("boom")):
            self.assertEqual(mod.find_twins_in("/does/not/matter"), [])

    def test_oserror_on_walk_is_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "a.hwp")
            _touch(d, "a.hwpx")
            with mock.patch.object(mod.os, "walk", side_effect=OSError("boom")):
                self.assertEqual(mod.find_twins_in(d), [])

    def test_find_twins_safe_folds_walk_error(self):
        mod = load()
        with mock.patch.object(mod, "find_twins", side_effect=OSError("boom")):
            pairs, err = mod.find_twins_safe()
        self.assertEqual(pairs, [])
        self.assertEqual(err["where"], "walk")
        self.assertEqual(err["kind"], "os-error")

    def test_path_rank_depth_then_lex(self):
        mod = load()
        paths = ["z.hwp", "a/b.hwp", "a/a.hwp", "aa.hwp"]
        self.assertEqual(
            sorted(paths, key=mod.path_rank),
            ["aa.hwp", "z.hwp", "a/a.hwp", "a/b.hwp"],
        )


class HashHonestyTests(unittest.TestCase):
    """없음은 동일이 아니다. 공백만 접고, 글자는 접지 않는다."""

    def test_none_equals_none_is_false(self):
        mod = load()
        self.assertFalse(mod.same_body_hash(None, None))

    def test_empty_pages_is_hash_of_empty_not_none(self):
        mod = load()
        ha = mod.body_hash_from_env({"pages": []})
        hb = mod.body_hash_from_env({"pages": [{"text": ""}]})
        self.assertIsNotNone(ha)
        self.assertEqual(ha, hb)
        self.assertTrue(mod.same_body_hash(ha, hb))
        self.assertTrue(mod.is_sha256_hex(ha))

    def test_missing_pages_key_is_none(self):
        mod = load()
        self.assertIsNone(mod.body_hash_from_env({}))
        self.assertIsNone(mod.body_hash_from_env({"pages": None}))
        self.assertIsNone(mod.body_hash_from_env({"pages": "not-list"}))
        self.assertIsNone(mod.body_hash_from_env("text"))

    def test_whitespace_variants_collapse(self):
        mod = load()
        variants = [
            {"pages": [{"text": "가나다"}]},
            {"pages": [{"text": "가 나 다"}]},
            {"pages": [{"text": "가\n나\t다"}]},
            {"pages": [{"text": "가"}, {"text": "나다"}]},
            {"pages": [{"text": " 가나다\n"}]},
        ]
        hashes = [mod.body_hash_from_env(env) for env in variants]
        self.assertEqual(len(set(hashes)), 1)
        self.assertTrue(all(mod.same_body_hash(hashes[0], h) for h in hashes))

    def test_different_letters_do_not_collapse(self):
        mod = load()
        ha = mod.body_hash_from_env({"pages": [{"text": "가"}]})
        hb = mod.body_hash_from_env({"pages": [{"text": "나"}]})
        self.assertNotEqual(ha, hb)
        self.assertFalse(mod.same_body_hash(ha, hb))

    def test_non_dict_page_is_skipped_not_crash(self):
        mod = load()
        ha = mod.body_hash_from_env({"pages": ["not-dict", {"text": "가"}, None]})
        hb = mod.body_hash_from_env({"pages": [{"text": "가"}]})
        self.assertEqual(ha, hb)

    def test_missing_text_key_counts_as_empty(self):
        mod = load()
        ha = mod.body_hash_from_env({"pages": [{} , {"text": "가"}]})
        hb = mod.body_hash_from_env({"pages": [{"text": "가"}]})
        self.assertEqual(ha, hb)

    def test_hash_text_none_is_none(self):
        mod = load()
        self.assertIsNone(mod.hash_text(None))
        self.assertIsNone(mod.normalize_body(None))

    def test_body_hash_with_run_exception_is_none(self):
        mod = load()

        def boom(_args):
            raise FileNotFoundError("rhwp")

        self.assertIsNone(mod.body_hash_with_run(boom, "a.hwp"))
        self.assertFalse(mod.same_body_hash(
            mod.body_hash_with_run(boom, "a.hwp"),
            mod.body_hash_with_run(boom, "a.hwpx"),
        ))

    def test_body_hash_with_run_bad_shape_is_none(self):
        mod = load()
        self.assertIsNone(mod.body_hash_with_run(lambda _a: None, "a.hwp"))
        self.assertIsNone(mod.body_hash_with_run(lambda _a: "nope", "a.hwp"))
        self.assertIsNone(mod.body_hash_with_run(lambda _a: (0,), "a.hwp"))

    def test_body_hash_live_exception_is_none(self):
        mod = load()
        with mock.patch.object(mod, "run_cli", side_effect=PermissionError("x")):
            self.assertIsNone(mod.body_hash("rhwp", "a.hwp"))

    def test_pages_text_rejects_non_dict_env(self):
        mod = load()
        self.assertIsNone(mod.pages_text(None))
        self.assertIsNone(mod.pages_text([]))
        self.assertEqual(mod.pages_text({"pages": []}), "")


class ClassifyHonestyTests(unittest.TestCase):
    """관문이 네 칸만 낸다. 해시 부재를 contradiction 으로 부르지 않는다."""

    def test_classify_pair_never_returns_unknown(self):
        mod = load()
        for body in (True, False, 1, 0, "", "x"):
            for ir_id in (True, False, 1, 0, "", "x"):
                for diverged in (None, 0, 1, [], [{}], False, True, "", "row"):
                    got = mod.classify_pair(body, ir_id, diverged)
                    self.assertIn(got, mod.GATE_LABELS)

    def test_no_divergence_always_none(self):
        mod = load()
        for empty in (None, 0, 0.0, "", [], {}, False):
            self.assertIsNone(mod.classify_pair(True, True, empty), empty)
            self.assertIsNone(mod.classify_pair(False, True, empty), empty)
            self.assertIsNone(mod.classify_pair(True, False, empty), empty)

    def test_missing_body_never_contradiction(self):
        mod = load()
        row = [{"observation": "pageCount"}]
        self.assertEqual(mod.classify_pair(False, True, row), "other-doc")
        self.assertEqual(mod.classify_pair(False, False, row), "other-doc")
        self.assertNotEqual(mod.classify_pair(False, True, row), "contradiction")

    def test_ir_false_never_contradiction(self):
        mod = load()
        row = [{"observation": "pageCount"}]
        self.assertEqual(mod.classify_pair(True, False, row), "review")
        self.assertNotEqual(mod.classify_pair(True, False, row), "contradiction")

    def test_finding_severity_matches_ir_flag(self):
        mod = load()
        self.assertEqual(mod.finding_severity(True), "contradiction")
        self.assertEqual(mod.finding_severity(False), "review")
        self.assertEqual(mod.finding_severity(0), "review")
        self.assertEqual(mod.finding_severity(""), "review")


class CompareTwinsExceptionTests(unittest.TestCase):
    def test_same_cli_error_both_sides_is_not_divergence(self):
        mod = load()

        def run(_args):
            raise FileNotFoundError("rhwp")

        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(compared, 1)
        self.assertEqual(other, 0)
        self.assertEqual(findings, [])

    def test_one_side_cli_error_is_divergence_then_hash_gate(self):
        mod = load()

        def run(args):
            if args[0] == "info" and args[1].endswith(".hwpx"):
                raise PermissionError("x")
            if args[0] == "info":
                return (0, {"pageCount": 6})
            if args[0] == "export-text":
                return (0, {"pages": [{"text": "같은본문"}]})
            if args[0] == "ir-diff":
                return (0, {"identical": True, "diffCount": 0})
            raise AssertionError(args)

        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(compared, 1)
        self.assertEqual(other, 0)
        self.assertEqual(findings[0]["severity"], "contradiction")
        self.assertEqual(findings[0]["diverged"][0]["hwpx"]["kind"], "permission")

    def test_timeout_one_side_is_observation_not_crash(self):
        mod = load()

        def run(args):
            if args[0] == "info" and args[1].endswith(".hwp"):
                raise subprocess.TimeoutExpired(cmd=args, timeout=1)
            if args[0] == "info":
                return (0, {"pageCount": 1})
            if args[0] == "export-text":
                return (1, None)
            raise AssertionError(args)

        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(compared, 1)
        self.assertEqual(other, 1)
        self.assertEqual(findings, [])

    def test_pair_loop_keyboardinterrupt_is_not_swallowed(self):
        mod = load()

        def run(_args):
            raise KeyboardInterrupt

        with self.assertRaises(KeyboardInterrupt):
            mod.compare_twins(
                [("a", "a.hwp", "a.hwpx")],
                run,
                observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
            )

    def test_malformed_pair_is_skipped(self):
        mod = load()
        detailed = mod.compare_twins_detailed(
            [("nope",), ("a", "a.hwp", "a.hwpx")],
            lambda args: (0, {"pageCount": 1}),
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(detailed["skippedPairs"], 1)
        self.assertEqual(len(detailed["pairErrors"]), 1)
        self.assertEqual(detailed["findings"], [])

    def test_malformed_pairs_type_is_empty_detailed(self):
        mod = load()
        detailed = mod.compare_twins_detailed(None, lambda args: (0, {}))
        self.assertEqual(detailed["compared"], 0)
        self.assertEqual(detailed["findings"], [])
        self.assertTrue(detailed["pairErrors"])


class IrFailureIsNotContradictionTests(unittest.TestCase):
    def test_ir_diff_exception_is_review(self):
        mod = load()
        body = (0, {"pages": [{"text": "같은본문"}]})

        def run(args):
            if args[0] == "info":
                return (0, {"pageCount": 1 if args[1].endswith(".hwp") else 2})
            if args[0] == "export-text":
                return body
            if args[0] == "ir-diff":
                raise OSError("broken pipe")
            raise AssertionError(args)

        _c, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(other, 0)
        self.assertEqual(findings[0]["severity"], "review")
        self.assertFalse(findings[0]["irIdentical"])
        self.assertIsNone(findings[0]["irDiffCount"])

    def test_ir_diff_none_envelope_is_review(self):
        mod = load()
        body = (0, {"pages": [{"text": "같은본문"}]})

        def run(args):
            if args[0] == "info":
                return (0, {"pageCount": 1 if args[1].endswith(".hwp") else 2})
            if args[0] == "export-text":
                return body
            if args[0] == "ir-diff":
                return (2, None)
            raise AssertionError(args)

        _c, _o, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(findings[0]["severity"], "review")
        self.assertFalse(findings[0]["irIdentical"])

    def test_ir_identity_with_run_folds_type_error(self):
        mod = load()
        identical, count = mod.ir_identity_with_run(lambda _a: "nope", "a.hwp", "a.hwpx")
        self.assertFalse(identical)
        self.assertIsNone(count)

    def test_ir_identity_non_dict(self):
        mod = load()
        self.assertEqual(mod.ir_identity(None), (False, None))
        self.assertEqual(mod.ir_identity([]), (False, None))
        self.assertEqual(mod.ir_identity({"identical": 1, "diffCount": 0}), (True, 0))
        self.assertEqual(mod.ir_identity({"identical": 0, "diffCount": 3}), (False, 3))


class MissingHashIsNotContradictionTests(unittest.TestCase):
    def test_hash_exception_both_sides_is_other_doc(self):
        mod = load()

        def run(args):
            if args[0] == "info":
                return (0, {"pageCount": 1 if args[1].endswith(".hwp") else 2})
            if args[0] == "export-text":
                raise TimeoutError("slow")
            raise AssertionError(args)

        _c, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(other, 1)
        self.assertEqual(findings, [])

    def test_hash_exception_one_side_is_other_doc(self):
        mod = load()

        def run(args):
            if args[0] == "info":
                return (0, {"pageCount": 1 if args[1].endswith(".hwp") else 2})
            if args[0] == "export-text" and args[1].endswith(".hwpx"):
                raise FileNotFoundError("gone")
            if args[0] == "export-text":
                return (0, {"pages": [{"text": "가"}]})
            raise AssertionError(args)

        _c, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[("pageCount", ["info", "{f}", "--json"], "pageCount")],
        )
        self.assertEqual(other, 1)
        self.assertEqual(findings, [])

    def test_same_missing_hash_is_still_other_doc(self):
        """양쪽 해시가 둘 다 None 이어도 동일 문서가 아니다."""
        mod = load()
        self.assertFalse(mod.same_body_hash(None, None))
        self.assertEqual(
            mod.classify_pair(False, True, [{"observation": "pageCount"}]),
            "other-doc",
        )


class ReportHonestyTests(unittest.TestCase):
    def test_validate_report_accepts_clean_ok(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=2, compared=12, other_doc=0, findings=[]
        )
        self.assertEqual(mod.validate_report(report), [])
        self.assertTrue(report["ok"])
        self.assertEqual(report["exit"], mod.EXIT_OK)
        self.assertFalse(report["toolFailed"])

    def test_validate_report_catches_ok_lie(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=1, compared=1, other_doc=0,
            findings=[mod.make_finding(
                "a", "a.hwp", "a.hwpx", True, 0,
                [{"observation": "pageCount", "hwp": _value(1), "hwpx": _value(2)}],
            )],
        )
        report["ok"] = True
        issues = mod.validate_report(report)
        self.assertTrue(any("ok" in item for item in issues))

    def test_validate_report_rejects_other_doc_finding(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=1, compared=1, other_doc=1, findings=[]
        )
        report["findings"] = [{"stem": "a", "severity": "other-doc", "irIdentical": False, "diverged": [{}]}]
        report["reviews"] = 0
        report["contradictions"] = 0
        issues = mod.validate_report(report)
        self.assertTrue(any("other-doc" in item for item in issues))

    def test_validate_report_rejects_contradiction_without_ir(self):
        mod = load()
        finding = mod.make_finding(
            "a", "a.hwp", "a.hwpx", False, 1,
            [{"observation": "pageCount", "hwp": _value(1), "hwpx": _value(2)}],
        )
        finding["severity"] = "contradiction"
        report = {
            "kind": "gymDifferential",
            "schemaVersion": "1.0",
            "ok": False,
            "runner": {"bin": "rhwp"},
            "pairs": 1,
            "observationsCompared": 1,
            "sameNameDifferentDocument": 0,
            "findings": [finding],
            "contradictions": 1,
            "reviews": 0,
            "exit": 3,
            "toolFailed": False,
        }
        issues = mod.validate_report(report)
        self.assertTrue(any("irIdentical" in item for item in issues))

    def test_tool_failed_does_not_invent_findings(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=0, compared=0, other_doc=0, findings=[],
            tool_errors=[{"where": "find-bin", "kind": "missing-bin", "error": "FileNotFoundError", "head": "x"}],
        )
        self.assertTrue(report["toolFailed"])
        self.assertEqual(report["findings"], [])
        self.assertTrue(report["ok"])
        self.assertEqual(report["exit"], mod.EXIT_TOOL_FAILED)
        self.assertEqual(mod.validate_report(report), [])
        self.assertEqual(mod.status_exit(report), 1)

    def test_status_exit_prefers_tool_failed(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=1, compared=1, other_doc=0,
            findings=[mod.make_finding(
                "a", "a.hwp", "a.hwpx", True, 0,
                [{"observation": "pageCount", "hwp": _value(1), "hwpx": _value(2)}],
            )],
            tool_errors=[{"where": "write", "kind": "os-error", "error": "OSError", "head": "x"}],
        )
        self.assertFalse(report["ok"])
        self.assertEqual(mod.status_exit(report), 1)

    def test_validate_report_non_dict(self):
        mod = load()
        self.assertEqual(mod.validate_report(None), ["report 가 dict 가 아니다"])

    def test_validate_report_missing_keys(self):
        mod = load()
        issues = mod.validate_report({"kind": "nope"})
        self.assertTrue(any("키 없음" in item for item in issues))
        self.assertTrue(any("kind" in item for item in issues))


class WriteReportSafeTests(unittest.TestCase):
    def test_write_error_is_string_not_exception(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=0, compared=0, other_doc=0, findings=[]
        )
        err = mod.write_report_safe(report, os.path.join("no", "such", "dir", "out.json"))
        self.assertIsInstance(err, str)
        self.assertIn("write_report", err)

    def test_write_success_is_none(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=0, compared=0, other_doc=0, findings=[]
        )
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "out.json")
            self.assertIsNone(mod.write_report_safe(report, path))
            self.assertTrue(os.path.isfile(path))

    def test_write_does_not_swallow_keyboardinterrupt(self):
        mod = load()
        with mock.patch.object(mod, "write_report", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                mod.write_report_safe({}, "x.json")

    def test_attach_write_error_does_not_change_ok(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=0, compared=0, other_doc=0, findings=[]
        )
        mod.attach_write_error(report, "disk full")
        self.assertTrue(report["ok"])
        self.assertEqual(report["writeError"], "disk full")
        self.assertEqual(report["contradictions"], 0)


class RunCliSafeTests(unittest.TestCase):
    def test_missing_bin_is_observation(self):
        mod = load()
        with mock.patch.object(mod.subprocess, "run", side_effect=FileNotFoundError("rhwp")):
            code, env = mod.run_cli_safe("missing", ["info", "a.hwp", "--json"])
        self.assertIsNone(code)
        self.assertEqual(env["kind"], "missing-bin")
        self.assertEqual(env["error"], "FileNotFoundError")

    def test_permission_is_observation(self):
        mod = load()
        with mock.patch.object(mod.subprocess, "run", side_effect=PermissionError("x")):
            code, env = mod.run_cli_safe("rhwp", ["info"])
        self.assertIsNone(code)
        self.assertEqual(env["kind"], "permission")

    def test_timeout_is_observation(self):
        mod = load()
        with mock.patch.object(
            mod.subprocess, "run",
            side_effect=subprocess.TimeoutExpired(cmd=["rhwp"], timeout=1),
        ):
            code, env = mod.run_cli_safe("rhwp", ["info"], timeout=1)
        self.assertIsNone(code)
        self.assertEqual(env["kind"], "timeout")

    def test_oserror_is_observation(self):
        mod = load()
        with mock.patch.object(mod.subprocess, "run", side_effect=OSError(22, "bad")):
            _code, env = mod.run_cli_safe("rhwp", ["info"])
        self.assertEqual(env["kind"], "os-error")

    def test_does_not_swallow_keyboardinterrupt(self):
        mod = load()
        with mock.patch.object(mod.subprocess, "run", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                mod.run_cli_safe("rhwp", ["info"])

    def test_does_not_swallow_systemexit(self):
        mod = load()
        with mock.patch.object(mod.subprocess, "run", side_effect=SystemExit(2)):
            with self.assertRaises(SystemExit):
                mod.run_cli_safe("rhwp", ["info"])

    def test_empty_bin_path_is_missing_bin(self):
        mod = load()
        code, env = mod.run_cli_safe("", ["info"])
        self.assertIsNone(code)
        self.assertEqual(env["kind"], "missing-bin")

    def test_success_json(self):
        mod = load()
        proc = mock.Mock(stdout=b'{"pageCount": 6}', returncode=0)
        with mock.patch.object(mod.subprocess, "run", return_value=proc):
            code, env = mod.run_cli_safe("rhwp", ["info", "a.hwp", "--json"])
        self.assertEqual(code, 0)
        self.assertEqual(env, {"pageCount": 6})

    def test_non_json_stdout_is_none_env(self):
        mod = load()
        proc = mock.Mock(stdout=b"not json", returncode=2)
        with mock.patch.object(mod.subprocess, "run", return_value=proc):
            code, env = mod.run_cli("rhwp", ["info"])
        self.assertEqual(code, 2)
        self.assertIsNone(env)

    def test_timeout_kwarg_passed(self):
        mod = load()
        proc = mock.Mock(stdout=b"{}", returncode=0)
        with mock.patch.object(mod.subprocess, "run", return_value=proc) as run:
            mod.run_cli("rhwp", ["info"], timeout=5)
        self.assertEqual(run.call_args.kwargs.get("timeout"), 5)

    def test_timeout_zero_not_passed(self):
        mod = load()
        proc = mock.Mock(stdout=b"{}", returncode=0)
        with mock.patch.object(mod.subprocess, "run", return_value=proc) as run:
            mod.run_cli("rhwp", ["info"], timeout=0)
        self.assertNotIn("timeout", run.call_args.kwargs)

    def test_decode_cli_stdout_rejects_int(self):
        mod = load()
        with self.assertRaises(TypeError):
            mod.decode_cli_stdout(1)
        self.assertEqual(mod.decode_cli_stdout(None), "")
        self.assertEqual(mod.decode_cli_stdout("abc"), "abc")
        self.assertEqual(mod.decode_cli_stdout(b"abc"), "abc")

    def test_loads_cli_json_empty_is_none(self):
        mod = load()
        self.assertIsNone(mod.loads_cli_json(None))
        self.assertIsNone(mod.loads_cli_json("  "))
        self.assertEqual(mod.loads_cli_json('{"a": 1}'), {"a": 1})
        with self.assertRaises(TypeError):
            mod.loads_cli_json(b"{}")

    def test_observe_uses_safe_cli(self):
        mod = load()
        with mock.patch.object(mod.subprocess, "run", side_effect=FileNotFoundError("rhwp")):
            obs = mod.observe("missing", "a.hwp", ["info", "{f}", "--json"], "pageCount")
        self.assertEqual(obs["kind"], "missing-bin")

    def test_observation_from_result_passes_through_error_obs(self):
        mod = load()
        env = {"kind": "timeout", "error": "TimeoutError", "head": "x"}
        self.assertEqual(mod.observation_from_result(None, env, "pageCount"), env)


class FindBinAndMainTests(unittest.TestCase):
    def test_find_bin_safe_success(self):
        mod = load()
        with mock.patch.object(mod.runner, "find_bin", return_value="/opt/rhwp"):
            found, err = mod.find_bin_safe("rhwp")
        self.assertEqual(found, "/opt/rhwp")
        self.assertIsNone(err)

    def test_find_bin_safe_folds_oserror(self):
        mod = load()
        with mock.patch.object(mod.runner, "find_bin", side_effect=OSError("x")):
            found, err = mod.find_bin_safe("rhwp")
        self.assertEqual(found, "rhwp")
        self.assertEqual(err["where"], "find-bin")
        self.assertEqual(err["kind"], "os-error")

    def test_find_bin_safe_does_not_swallow_keyboardinterrupt(self):
        mod = load()
        with mock.patch.object(mod.runner, "find_bin", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                mod.find_bin_safe("rhwp")

    def test_parse_args_defaults(self):
        mod = load()
        a = mod.parse_args([])
        self.assertEqual(a.limit, 0)
        self.assertIsNone(a.bin)
        self.assertIsNone(a.out)
        self.assertEqual(a.cli_timeout, 0)

    def test_main_exit_zero_when_no_pairs(self):
        mod = load()
        with mock.patch.object(mod, "find_bin_safe", return_value=("rhwp", None)), \
                mock.patch.object(mod, "find_twins_safe", return_value=([], None)), \
                mock.patch.object(mod, "write_report_safe", return_value=None), \
                mock.patch.object(mod, "render_summary", return_value=["ok"]):
            code = mod.main(["--limit", "0"])
        self.assertEqual(code, 0)

    def test_main_exit_one_when_find_bin_fails(self):
        mod = load()
        err = {"where": "find-bin", "kind": "os-error", "error": "OSError", "head": "x"}
        with mock.patch.object(mod, "find_bin_safe", return_value=("rhwp", err)), \
                mock.patch.object(mod, "find_twins_safe", return_value=([], None)), \
                mock.patch.object(mod, "write_report_safe", return_value=None), \
                mock.patch.object(mod, "render_summary", return_value=["fail"]):
            code = mod.main([])
        self.assertEqual(code, 1)

    def test_main_exit_three_on_contradiction(self):
        mod = load()
        finding = {
            "stem": "a", "hwp": "a.hwp", "hwpx": "a.hwpx",
            "irIdentical": True, "irDiffCount": 0,
            "diverged": [{"observation": "pageCount", "hwp": _value(1), "hwpx": _value(2)}],
            "severity": "contradiction",
        }
        with mock.patch.object(mod, "find_bin_safe", return_value=("rhwp", None)), \
                mock.patch.object(mod, "find_twins_safe", return_value=([("a", "a.hwp", "a.hwpx")], None)), \
                mock.patch.object(mod, "compare_twins", return_value=(1, 0, [finding])), \
                mock.patch.object(mod, "write_report_safe", return_value=None), \
                mock.patch.object(mod, "render_summary", return_value=["bad"]):
            code = mod.main([])
        self.assertEqual(code, 3)

    def test_main_write_error_does_not_change_ok_exit(self):
        mod = load()
        with mock.patch.object(mod, "find_bin_safe", return_value=("rhwp", None)), \
                mock.patch.object(mod, "find_twins_safe", return_value=([], None)), \
                mock.patch.object(mod, "write_report_safe", return_value="disk full"), \
                mock.patch.object(mod, "render_summary", return_value=["ok"]):
            code = mod.main([])
        self.assertEqual(code, 0)


class CatalogContractTests(unittest.TestCase):
    def test_report_kind_and_schema(self):
        mod = load()
        self.assertEqual(mod.REPORT_KIND, "gymDifferential")
        self.assertEqual(mod.SCHEMA_VERSION, "1.0")

    def test_observation_catalog_contains_core_kinds(self):
        mod = load()
        for kind in ("value", "nojson", "badenv", "missing", "timeout", "missing-bin"):
            self.assertIn(kind, mod.OBSERVATION_KINDS)

    def test_severities_do_not_include_other_doc(self):
        mod = load()
        self.assertEqual(mod.SEVERITIES, ("contradiction", "review"))
        self.assertNotIn("other-doc", mod.SEVERITIES)

    def test_gate_labels_include_none_and_other_doc(self):
        mod = load()
        self.assertIn(None, mod.GATE_LABELS)
        self.assertIn("other-doc", mod.GATE_LABELS)
        self.assertIn("contradiction", mod.GATE_LABELS)
        self.assertIn("review", mod.GATE_LABELS)

    def test_exits(self):
        mod = load()
        self.assertEqual(mod.EXIT_OK, 0)
        self.assertEqual(mod.EXIT_TOOL_FAILED, 1)
        self.assertEqual(mod.EXIT_CONTRADICTION, 3)

    def test_observations_six_labels(self):
        mod = load()
        labels = [row[0] for row in mod.OBSERVATIONS]
        self.assertEqual(
            labels,
            ["pageCount", "tableCount", "paragraphCount", "fieldCount", "footnoteCount", "endnoteCount"],
        )
        for _name, args, key in mod.OBSERVATIONS:
            self.assertIn("{f}", args)
            self.assertTrue(key)

    def test_twin_exts(self):
        mod = load()
        self.assertEqual(mod.TWIN_EXTS, (".hwp", ".hwpx"))

    def test_fatal_tuple(self):
        mod = load()
        self.assertIn(KeyboardInterrupt, mod.FATAL_EXCEPTIONS)
        self.assertIn(SystemExit, mod.FATAL_EXCEPTIONS)
        self.assertIn(MemoryError, mod.FATAL_EXCEPTIONS)
        self.assertIn(GeneratorExit, mod.FATAL_EXCEPTIONS)

    def test_report_keys(self):
        mod = load()
        for key in (
            "kind", "schemaVersion", "ok", "runner", "pairs",
            "observationsCompared", "sameNameDifferentDocument",
            "findings", "contradictions", "reviews",
        ):
            self.assertIn(key, mod.REPORT_KEYS)


class GeneratedClassifyTableTests(unittest.TestCase):
    def test_full_truth_table(self):
        mod = load()
        diverged_shapes = (
            [{"observation": "pageCount"}],
            [{"observation": "a"}, {"observation": "b"}],
            "x",
            1,
            True,
        )
        empty_shapes = (None, 0, 0.0, "", [], {}, False)
        for diverged in empty_shapes:
            for body in (True, False):
                for ir_id in (True, False):
                    self.assertIsNone(
                        mod.classify_pair(body, ir_id, diverged),
                        (body, ir_id, diverged),
                    )
        for diverged in diverged_shapes:
            self.assertEqual(mod.classify_pair(False, True, diverged), "other-doc")
            self.assertEqual(mod.classify_pair(False, False, diverged), "other-doc")
            self.assertEqual(mod.classify_pair(True, True, diverged), "contradiction")
            self.assertEqual(mod.classify_pair(True, False, diverged), "review")


class GeneratedPairingTableTests(unittest.TestCase):
    CASES = [
        (["a.hwp"], ["a.hwpx"], ("a.hwp", "a.hwpx")),
        (["b/a.hwp", "a.hwp"], ["a.hwpx"], ("a.hwp", "a.hwpx")),
        (["b/a.hwp"], ["a.hwpx", "b/a.hwpx"], ("b/a.hwp", "b/a.hwpx")),
        (["z/a.hwp", "a/a.hwp"], ["z/a.hwpx", "a/a.hwpx"], ("a/a.hwp", "a/a.hwpx")),
        (["deep/n/x.hwp", "x.hwp"], ["other/x.hwpx"], ("x.hwp", "other/x.hwpx")),
        (["deep/n/x.hwp"], ["other/x.hwpx"], ("deep/n/x.hwp", "other/x.hwpx")),
        (["aa/z.hwp", "sub/z.hwp"], ["sub/z.hwpx"], ("sub/z.hwp", "sub/z.hwpx")),
        (["m/n/z.hwp", "m/z.hwp"], ["m/n/z.hwpx"], ("m/n/z.hwp", "m/n/z.hwpx")),
        ([], ["a.hwpx"], None),
        (["a.hwp"], [], None),
        (["a.hwp", "b.hwp"], ["c.hwpx"], ("a.hwp", "c.hwpx")),
        (["b.hwp", "a.hwp"], ["c.hwpx"], ("a.hwp", "c.hwpx")),
        (["dir/b.hwp", "dir/a.hwp"], ["dir/z.hwpx"], ("dir/a.hwp", "dir/z.hwpx")),
        (["0/a.hwp", "1/a.hwp"], ["0/a.hwpx"], ("0/a.hwp", "0/a.hwpx")),
        (["samples/a.hwp"], ["fixtures/a.hwpx", "samples/a.hwpx"], ("samples/a.hwp", "samples/a.hwpx")),
    ]

    def test_table_and_reversed_inputs(self):
        mod = load()
        for hwps, hwpxs, expected in self.CASES:
            self.assertEqual(mod.pick_twin_paths(hwps, hwpxs), expected, (hwps, hwpxs))
            self.assertEqual(
                mod.pick_twin_paths(list(reversed(hwps)), list(reversed(hwpxs))),
                expected,
                ("rev", hwps, hwpxs),
            )


class GeneratedEqualityTableTests(unittest.TestCase):
    def test_symmetric_table(self):
        mod = load()
        nan = float("nan")
        table = [
            (_value(6), _value(6.0), True),
            (_value(6), _value(7), False),
            (_value(True), _value(1), False),
            (_value(False), _value(0), False),
            (_value(nan), _value(nan), True),
            (_value(float("inf")), _value(float("-inf")), False),
            ({"kind": "nojson", "code": 1}, _value("exit1"), False),
            ({"kind": "missing", "key": "x"}, None, False),
            ({"kind": "timeout", "error": "TimeoutError", "head": "a"},
             {"kind": "timeout", "error": "TimeoutError", "head": "a"}, True),
            ({"kind": "timeout", "error": "TimeoutError", "head": "a"},
             {"kind": "permission", "error": "PermissionError", "head": "a"}, False),
            ([1, {"a": 2.0}], [1.0, {"a": 2}], True),
            ({"b": 1, "a": 2}, {"a": 2.0, "b": 1}, True),
            ("가", "가", True),
            ("가", "나", False),
            (b"x", "x", False),
        ]
        for left, right, expected in table:
            self.assertEqual(mod.observations_equal(left, right), expected, (left, right))
            if not (isinstance(left, float) and math.isnan(left)):
                self.assertEqual(mod.observations_equal(right, left), expected, (right, left))


class GeneratedHashTableTests(unittest.TestCase):
    VARIANTS_SAME = [
        "한글본문",
        "한 글 본 문",
        "한글\n본문",
        "한글\t본문",
        "  한글본문  ",
        "한글본문\r\n",
        "한\r글본문",
    ]
    VARIANTS_DIFFERENT = [
        "한글본문",
        "한글본문.",
        "한글본문2",
        "한글본",
        "본문한글",
        "Hangul",
    ]

    def test_whitespace_family_shares_hash(self):
        mod = load()
        hashes = [
            mod.body_hash_from_env({"pages": [{"text": text}]})
            for text in self.VARIANTS_SAME
        ]
        self.assertEqual(len(set(hashes)), 1)
        self.assertTrue(mod.is_sha256_hex(hashes[0]))

    def test_letter_family_all_distinct(self):
        mod = load()
        hashes = [
            mod.body_hash_from_env({"pages": [{"text": text}]})
            for text in self.VARIANTS_DIFFERENT
        ]
        self.assertEqual(len(set(hashes)), len(hashes))
        for i, left in enumerate(hashes):
            for j, right in enumerate(hashes):
                self.assertEqual(mod.same_body_hash(left, right), i == j, (i, j))

    def test_split_pages_match_joined(self):
        mod = load()
        joined = mod.body_hash_from_env({"pages": [{"text": "가나다라마"}]})
        split = mod.body_hash_from_env({
            "pages": [{"text": "가 나"}, {"text": "다"}, {"text": "라마"}],
        })
        self.assertEqual(joined, split)


class SelectPairsEdgeTests(unittest.TestCase):
    def test_negative_and_bad_limit_mean_all(self):
        mod = load()
        pairs = [("a", "a.hwp", "a.hwpx"), ("b", "b.hwp", "b.hwpx")]
        self.assertEqual(mod.select_pairs(pairs, -1), pairs)
        self.assertEqual(mod.select_pairs(pairs, None), pairs)
        self.assertEqual(mod.select_pairs(pairs, "nope"), pairs)
        self.assertEqual(mod.select_pairs(pairs, 2), pairs)
        self.assertEqual(mod.select_pairs(pairs, 99), pairs)

    def test_non_iterable_pairs_is_empty(self):
        mod = load()
        self.assertEqual(mod.select_pairs(None, 1), [])
        self.assertEqual(mod.select_pairs(1, 1), [])


class RenderSummaryHonestyTests(unittest.TestCase):
    def test_tool_failed_summary_does_not_say_clean(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=0, compared=0, other_doc=0, findings=[],
            tool_errors=[{"where": "find-bin", "kind": "missing-bin", "error": "FileNotFoundError", "head": "x"}],
        )
        text = "\n".join(mod.render_summary(report, "out.json"))
        self.assertIn("도구 실패", text)
        self.assertIn("exit 1", text)
        self.assertIn("→ out.json", text)
        self.assertNotIn("!!", text)

    def test_review_is_not_double_bang(self):
        mod = load()
        finding = mod.make_finding(
            "doc", "doc.hwp", "doc.hwpx", False, 2,
            [{"observation": "tableCount", "hwp": _value(1), "hwpx": _value(2)}],
        )
        report = mod.build_report(
            bin_name="rhwp", pairs_count=1, compared=1, other_doc=0, findings=[finding]
        )
        text = "\n".join(mod.render_summary(report))
        self.assertNotIn("!!", text)
        self.assertIn("tableCount 1≠2", text)

    def test_pair_errors_and_write_error_appear(self):
        mod = load()
        report = mod.build_report(
            bin_name="rhwp", pairs_count=1, compared=0, other_doc=0, findings=[],
            pair_errors=[{"stem": "a", "kind": "os-error", "error": "OSError", "head": "x"}],
            write_error="disk full",
        )
        text = "\n".join(mod.render_summary(report, "out.json"))
        self.assertIn("쌍 오류", text)
        self.assertIn("쓰기 오류", text)


class ObserveWithRunTests(unittest.TestCase):
    def test_value_path(self):
        mod = load()
        obs = mod.observe_with_run(
            lambda args: (0, {"pageCount": 6}),
            "a.hwp",
            ["info", "{f}", "--json"],
            "pageCount",
        )
        self.assertEqual(obs, _value(6))

    def test_exception_path(self):
        mod = load()
        obs = mod.observe_with_run(
            lambda args: (_ for _ in ()).throw(PermissionError("x")),
            "a.hwp",
            ["info", "{f}", "--json"],
            "pageCount",
        )
        self.assertEqual(obs["kind"], "permission")

    def test_placeholder_replaced(self):
        mod = load()
        seen = []

        def run(args):
            seen.append(args)
            return (0, {"pageCount": 1})

        mod.observe_with_run(run, "doc.hwp", ["info", "{f}", "--json"], "pageCount")
        self.assertEqual(seen, [["info", "doc.hwp", "--json"]])

    def test_systemexit_not_swallowed(self):
        mod = load()
        with self.assertRaises(SystemExit):
            mod.observe_with_run(
                lambda args: (_ for _ in ()).throw(SystemExit(2)),
                "a.hwp",
                ["info", "{f}"],
                "pageCount",
            )


class FindingHonestyScanTests(unittest.TestCase):
    def test_make_finding_severity_tied_to_ir(self):
        mod = load()
        row = [{"observation": "pageCount", "hwp": _value(1), "hwpx": _value(2)}]
        contra = mod.make_finding("a", "a.hwp", "a.hwpx", True, 0, row)
        review = mod.make_finding("a", "a.hwp", "a.hwpx", False, 4, row)
        self.assertEqual(contra["severity"], "contradiction")
        self.assertTrue(contra["irIdentical"])
        self.assertEqual(review["severity"], "review")
        self.assertFalse(review["irIdentical"])

    def test_diverged_rows_sorted_and_skip_equal(self):
        mod = load()
        observed = [
            ("tableCount", _value(1), _value(1)),
            ("pageCount", _value(6), _value(7)),
            ("fieldCount", _value(0), _value(1)),
        ]
        rows = mod.diverged_rows(observed)
        self.assertEqual([r["observation"] for r in rows], ["fieldCount", "pageCount"])

    def test_diverged_numeric_int_float_skipped(self):
        mod = load()
        self.assertEqual(mod.diverged_rows([("pageCount", _value(3), _value(3.0))]), [])

    def test_build_report_sorts_findings(self):
        mod = load()
        row = [{"observation": "pageCount", "hwp": _value(1), "hwpx": _value(2)}]
        report = mod.build_report(
            bin_name="rhwp", pairs_count=2, compared=2, other_doc=0,
            findings=[
                mod.make_finding("m", "m.hwp", "m.hwpx", False, 1, row),
                mod.make_finding("a", "a.hwp", "a.hwpx", True, 0, row),
            ],
        )
        self.assertEqual([f["stem"] for f in report["findings"]], ["a", "m"])
        self.assertEqual(mod.validate_report(report), [])


class MultiObservationCompareTests(unittest.TestCase):
    def test_two_observations_one_diverges(self):
        mod = load()
        body = (0, {"pages": [{"text": "같은본문"}]})

        def run(args):
            if args[0] == "info":
                return (0, {"pageCount": 6})
            if args[0] == "export-tables":
                return (0, {"tableCount": 1 if args[1].endswith(".hwp") else 2})
            if args[0] == "export-text":
                return body
            if args[0] == "ir-diff":
                return (0, {"identical": False, "diffCount": 1})
            raise AssertionError(args)

        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
            observations=[
                ("pageCount", ["info", "{f}", "--json"], "pageCount"),
                ("tableCount", ["export-tables", "{f}", "--json"], "tableCount"),
            ],
        )
        self.assertEqual(compared, 2)
        self.assertEqual(other, 0)
        self.assertEqual(len(findings[0]["diverged"]), 1)
        self.assertEqual(findings[0]["diverged"][0]["observation"], "tableCount")
        self.assertEqual(findings[0]["severity"], "review")

    def test_default_observations_count(self):
        mod = load()

        def run(args):
            if args[0] == "info":
                return (0, {"pageCount": 1})
            if args[0] == "export-tables":
                return (0, {"tableCount": 0})
            if args[0] == "explain":
                return (0, {"paragraphCount": 2, "footnoteCount": 0, "endnoteCount": 0})
            if args[0] == "fields":
                return (0, {"fieldCount": 0})
            raise AssertionError(args)

        compared, other, findings = mod.compare_twins(
            [("a", "a.hwp", "a.hwpx")],
            run,
        )
        self.assertEqual(compared, len(mod.OBSERVATIONS))
        self.assertEqual(other, 0)
        self.assertEqual(findings, [])


class CoerceAndDecodeEdgeTests(unittest.TestCase):
    def test_coerce_run_result(self):
        mod = load()
        self.assertEqual(mod.coerce_run_result((0, {"a": 1})), (0, {"a": 1}))
        self.assertEqual(mod.coerce_run_result([1, None]), (1, None))
        with self.assertRaises(TypeError):
            mod.coerce_run_result(None)
        with self.assertRaises(TypeError):
            mod.coerce_run_result((0,))

    def test_decode_memoryview_and_bytearray(self):
        mod = load()
        self.assertEqual(mod.decode_cli_stdout(bytearray(b"ab")), "ab")
        self.assertEqual(mod.decode_cli_stdout(memoryview(b"cd")), "cd")

    def test_run_cli_str_stdout(self):
        mod = load()
        proc = mock.Mock(stdout='{"pageCount": 2}', returncode=0)
        with mock.patch.object(mod.subprocess, "run", return_value=proc):
            code, env = mod.run_cli("rhwp", ["info"])
        self.assertEqual((code, env), (0, {"pageCount": 2}))

    def test_run_cli_empty_stdout(self):
        mod = load()
        proc = mock.Mock(stdout=b"", returncode=1)
        with mock.patch.object(mod.subprocess, "run", return_value=proc):
            code, env = mod.run_cli("rhwp", ["info"])
        self.assertEqual(code, 1)
        self.assertIsNone(env)


class StatusExitTests(unittest.TestCase):
    def test_non_dict_is_tool_failed(self):
        mod = load()
        self.assertEqual(mod.status_exit(None), 1)
        self.assertEqual(mod.status_exit("x"), 1)

    def test_ok_true_is_zero(self):
        mod = load()
        self.assertEqual(mod.status_exit({"ok": True}), 0)

    def test_ok_false_is_three(self):
        mod = load()
        self.assertEqual(mod.status_exit({"ok": False}), 3)

    def test_tool_failed_overrides_ok(self):
        mod = load()
        self.assertEqual(mod.status_exit({"ok": True, "toolFailed": True}), 1)
        self.assertEqual(mod.status_exit({"ok": False, "toolFailed": True}), 1)


class DisplayAndFormatTests(unittest.TestCase):
    def test_display_error_kinds(self):
        mod = load()
        self.assertEqual(mod.observation_display({"kind": "timeout"}), "timeout")
        self.assertEqual(mod.observation_display({"kind": "permission"}), "permission")
        self.assertEqual(mod.observation_display({"kind": "badenv", "code": 0}), "badenv")
        self.assertEqual(mod.observation_display("raw"), "raw")
        self.assertEqual(mod.observation_display({"kind": ""}), {"kind": ""})

    def test_format_finding_detail_empty(self):
        mod = load()
        self.assertEqual(mod.format_finding_detail({"diverged": []}), "")
        detail = mod.format_finding_detail({
            "diverged": [
                {"observation": "pageCount", "hwp": _value(1), "hwpx": _value(2)},
                {"observation": "tableCount", "hwp": {"kind": "timeout"}, "hwpx": _value(0)},
            ]
        })
        self.assertIn("pageCount 1≠2", detail)
        self.assertIn("tableCount timeout≠0", detail)


class WalkAndDiscoveryMoreTests(unittest.TestCase):
    def test_mixed_case_and_nested_and_junk(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "Keep.HWP")
            _touch(d, "Keep.hwpx")
            _touch(d, "skip.doc")
            _touch(d, "skip.hwp.bak")
            _touch(d, "deep/nested/z.hwp")
            _touch(d, "deep/nested/z.HWPX")
            _touch(d, "deep/nested/only.hwpx")
            pairs = mod.find_twins_in(d)
        stems = [p[0] for p in pairs]
        self.assertEqual(stems, ["Keep", "z"])
        self.assertTrue(pairs[0][1].lower().endswith(".hwp"))
        self.assertTrue(pairs[0][2].lower().endswith(".hwpx"))

    def test_same_stem_different_dirs_prefers_local_pair(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _touch(d, "root.hwp")
            _touch(d, "nested/root.hwp")
            _touch(d, "nested/root.hwpx")
            pairs = mod.find_twins_in(d)
        self.assertEqual(pairs, [("root", "nested/root.hwp", "nested/root.hwpx")])

    def test_is_dir_safe_false_on_oserror(self):
        mod = load()
        with mock.patch.object(mod.os.path, "isdir", side_effect=OSError("x")):
            self.assertFalse(mod.is_dir_safe("anywhere"))


if __name__ == "__main__":
    unittest.main()
