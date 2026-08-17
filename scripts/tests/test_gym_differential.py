"""[#5224] gym 교차형식 차등 오라클 계약 — 순수 판정·보고.

바이너리 없이 쌍둥이 짝짓기·본문 해시·관측 대조·오검출 관문·보고 봉투를 고정한다.
CLI run 은 주입해 "관측이 갈렸을 때" 를 합성한다.
"""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path

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
        self.assertEqual(loaded, report)


if __name__ == "__main__":
    unittest.main()
