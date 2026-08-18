"""[competitive_bench] 추가 순수 함수 경우 — 기존 89건을 지우지 않는다.

파서 래퍼, 충실도 쌍, 평결 가지, 정직성 조립만 더 고정한다.
바이너리 없이 돈다. 새 CLI 플래그를 만들지 않는다.
"""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "competitive_bench.py"


def load():
    spec = importlib.util.spec_from_file_location("competitive_bench_extra", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class NumberAndPathTests(unittest.TestCase):
    def test_is_number_rejects_bool_and_string(self):
        m = load()
        self.assertTrue(m.is_number(0))
        self.assertTrue(m.is_number(0.0))
        self.assertTrue(m.is_number(-1.5))
        self.assertFalse(m.is_number(True))
        self.assertFalse(m.is_number(False))
        self.assertFalse(m.is_number("1"))
        self.assertFalse(m.is_number(None))

    def test_posix_rel_and_ext(self):
        m = load()
        self.assertEqual(m.posix_rel(r"samples\a.HWP"), "samples/a.HWP")
        self.assertEqual(m.ext_of("samples/a.HWPX"), ".hwpx")
        self.assertEqual(m.ext_of("no-suffix"), "")
        self.assertEqual(m.display_path(None), "")
        self.assertEqual(m.display_path(r"x\y.json"), "x/y.json")


class UnwrapAndInfoFieldsTests(unittest.TestCase):
    def test_unwrap_only_when_body_keys_present(self):
        m = load()
        self.assertIsNone(m.unwrap_json_envelope("x"))
        plain = {"ok": True, "data": {"note": "no-body"}}
        self.assertIs(m.unwrap_json_envelope(plain), plain)
        wrapped = {"payload": {"pageCount": 2}}
        self.assertEqual(m.unwrap_json_envelope(wrapped), {"pageCount": 2})

    def test_info_fields_keeps_present_keys_only(self):
        m = load()
        self.assertIsNone(m.parse_rhwp_info_fields(""))
        self.assertIsNone(m.parse_rhwp_info_fields("[]"))
        raw = json.dumps({"data": {
            "format": "hwp5", "pageCount": 3, "sectionCount": 1, "extra": 9,
        }})
        parsed = m.parse_rhwp_info_fields(raw)
        self.assertEqual(parsed["format"], "hwp5")
        self.assertEqual(parsed["pageCount"], 3)
        self.assertEqual(parsed["sectionCount"], 1)
        self.assertNotIn("extra", parsed)
        self.assertNotIn("paraCount", parsed)

    def test_structure_nodes_from_list_or_count(self):
        m = load()
        self.assertIsNone(m.parse_rhwp_structure_nodes("{"))
        self.assertIsNone(m.parse_rhwp_structure_nodes(json.dumps({"ok": True})))
        self.assertEqual(
            m.parse_rhwp_structure_nodes(json.dumps({"nodeCount": 7})), 7,
        )
        self.assertEqual(
            m.parse_rhwp_structure_nodes(json.dumps({"nodes": [1, 2, 3]})),
            3,
        )
        self.assertEqual(
            m.parse_rhwp_structure_nodes(json.dumps({
                "result": {"structure": {}, "nodes": [1, 2, 3]},
            })),
            3,
        )
        self.assertEqual(
            m.parse_rhwp_structure_nodes(json.dumps({"children": []})), 0,
        )
        self.assertIsNone(m.parse_rhwp_structure_nodes(json.dumps({
            "nodeCount": True,
        })))


class FidelityPairExtraTests(unittest.TestCase):
    def test_pairs_skip_failed_and_zero_base(self):
        m = load()
        ref = [
            {"file": "a", "ok": True, "chars": 10},
            {"file": "b", "ok": True, "chars": 0},
            {"file": "c", "ok": False, "chars": 99},
        ]
        tool = [
            {"file": "a", "ok": True, "chars": 5},
            {"file": "b", "ok": True, "chars": 1},
            {"file": "c", "ok": True, "chars": 1},
            {"file": "d", "ok": True, "chars": 1},
        ]
        pairs = m.fidelity_pairs(tool, ref)
        self.assertEqual(pairs, [("a", 0.5)])

    def test_overlap_ok_pairs_other_field(self):
        m = load()
        pairs = m.overlap_ok_pairs(
            [{"file": "a", "ok": True, "chars": 3},
             {"file": "b", "ok": True, "chars": True}],
            [{"file": "a", "ok": True, "chars": 6},
             {"file": "b", "ok": True, "chars": 1}],
            "chars",
        )
        self.assertEqual(pairs, [("a", 3, 6)])


class ExclusiveYesExtraTests(unittest.TestCase):
    def test_empty_and_malformed_matrix(self):
        m = load()
        self.assertEqual(m.exclusive_yes({}, "rhwp"), [])
        self.assertEqual(m.exclusive_yes({"columns": "x", "rows": None}, "rhwp"), [])
        self.assertEqual(m.capability_label({}, "mcp"), "mcp")
        matrix = m.capability_matrix()
        self.assertEqual(m.capability_label(matrix, "mcp"), "MCP 서버")
        self.assertEqual(m.capability_label(matrix, "no-such"), "no-such")

    def test_exclusive_yes_skips_non_yes(self):
        m = load()
        matrix = {
            "columns": [{"key": "edit"}, {"key": "mcp"}],
            "rows": [
                {"tool": "rhwp", "edit": "yes", "mcp": "yes"},
                {"tool": "alt", "edit": "yes", "mcp": "no"},
            ],
        }
        self.assertEqual(m.exclusive_yes(matrix, "rhwp"), ["mcp"])
        self.assertEqual(m.exclusive_yes(matrix, "ghost"), [])


class InventedMetricsExtraTests(unittest.TestCase):
    def test_available_is_never_invented(self):
        m = load()
        self.assertFalse(m.invented_metrics({"available": True, "summary": {"ok": 1}}))
        self.assertFalse(m.invented_metrics("x"))
        dirty_fid = m.unavailable_result("pyhwp", "n/a")
        dirty_fid["fidelityVsRhwp"] = 0.7
        self.assertTrue(m.invented_metrics(dirty_fid))
        dirty_ov = m.unavailable_result("soffice", "n/a")
        dirty_ov["overlapMs"] = {"n": 1}
        self.assertTrue(m.invented_metrics(dirty_ov))
        dirty_runs = m.unavailable_result("hwplib", "n/a")
        dirty_runs["runs"] = []
        self.assertFalse(m.invented_metrics(dirty_runs))
        dirty_runs["runs"] = [{"file": "a"}]
        self.assertTrue(m.invented_metrics(dirty_runs))


class VerdictExtraTests(unittest.TestCase):
    def test_soffice_available_states_counts(self):
        m = load()
        self.assertEqual(m.soffice_verdict(None), [])
        self.assertEqual(m.soffice_verdict("x"), [])
        lines = m.soffice_verdict({
            "available": True,
            "summary": {"ok": 1, "attempted": 4},
        })
        self.assertEqual(len(lines), 1)
        self.assertIn("1/4", lines[0])
        self.assertIn("HWP5 임포트 필터", lines[0])

    def test_width_silent_when_other_available(self):
        m = load()
        lines = m.width_verdict({
            "info": {"results": [
                {"tool": "rhwp", "available": True},
                {"tool": "pyhwp", "available": True},
            ]},
        })
        self.assertEqual(lines, [])

    def test_export_text_without_rhwp_still_states_pyhwp(self):
        m = load()
        lines = m.export_text_verdict(None, {
            "available": True,
            "summary": {"byExt": {".hwp": {"ok": 2, "attempted": 2}}},
        })
        self.assertTrue(any("pyhwp" in line for line in lines))
        self.assertFalse(any(line.startswith("rhwp 는") for line in lines))

    def test_capability_verdict_empty_exclusive(self):
        m = load()
        matrix = {
            "columns": [{"key": "edit", "label": "편집"}],
            "rows": [
                {"tool": "rhwp", "edit": "yes"},
                {"tool": "alt", "edit": "yes"},
            ],
        }
        text = m.capability_verdict(matrix)
        self.assertIn("겹치는 축만", text)

    def test_result_index_skips_non_dict(self):
        m = load()
        idx = m._result_index([
            "x",
            {"available": True},
            {"tool": "rhwp", "available": True},
        ])
        self.assertEqual(list(idx), ["rhwp"])
        self.assertEqual(m._summary_of(None), {})
        self.assertEqual(m._summary_of({"summary": "bad"}), {})


class AssembleEnvExtraTests(unittest.TestCase):
    def test_counts_hwp_and_hwpx_only(self):
        m = load()
        env = m.assemble_env(
            os_name="Win", python="3.12", rhwp_version="v", rhwp_profile="debug",
            files=["a.hwp", "b.hwpx", "c.txt", "d.HWP"],
            tools={"rhwp": {"available": True}},
        )
        self.assertEqual(env["corpus"]["total"], 4)
        self.assertEqual(env["corpus"]["hwp"], 2)
        self.assertEqual(env["corpus"]["hwpx"], 1)

    def test_available_result_omits_empty_overlap_and_note(self):
        m = load()
        rec = m.available_result("rhwp", [
            {"file": "a.hwp", "ok": True, "ms": 2, "chars": 1},
        ])
        self.assertNotIn("overlapMs", rec)
        self.assertNotIn("note", rec)
        rec2 = m.available_result(
            "pyhwp",
            [{"file": "a.hwp", "ok": True, "ms": 1, "chars": 1}],
            overlap={"n": 1, "tool": 1, "ref": 2},
            note="한계",
        )
        self.assertEqual(rec2["note"], "한계")
        self.assertEqual(rec2["overlapMs"]["n"], 1)


class NormalizeRunExtraTests(unittest.TestCase):
    def test_ok_coerced_and_ext_lowered(self):
        m = load()
        out = m.normalize_run({"file": "A.HWP", "ext": ".HWP", "ok": 1, "ms": 3})
        self.assertEqual(out["ext"], ".hwp")
        self.assertIs(out["ok"], True)
        out2 = m.normalize_run({"file": "doc.hwpx", "ok": 0})
        self.assertEqual(out2["ext"], ".hwpx")
        self.assertIs(out2["ok"], False)


class SelectCorpusExtraTests(unittest.TestCase):
    def test_ignores_doc_and_keeps_order_hwp_then_hwpx(self):
        m = load()
        picked = m.select_corpus_paths([
            "z.hwpx", "m.doc", "b.hwp", "a.hwp", "n.pdf",
        ], 0)
        self.assertEqual(picked, ["a.hwp", "b.hwp", "z.hwpx"])

    def test_per_ext_limit_does_not_steal_across_formats(self):
        m = load()
        picked = m.select_corpus_paths([
            "c.hwp", "a.hwp", "b.hwp", "z.hwpx",
        ], 1)
        self.assertEqual(picked, ["a.hwp", "z.hwpx"])


class RoundAndDumpExtraTests(unittest.TestCase):
    def test_round_helpers(self):
        m = load()
        self.assertIsNone(m._round_ms(None))
        self.assertEqual(m._round_ms(1.24), 1.2)
        self.assertIsNone(m._round_int(None))
        self.assertEqual(m._round_int(1.6), 2)

    def test_dump_payload_keeps_hangul(self):
        m = load()
        text = m.dump_payload_json({"kind": "gymCompetitiveBench", "note": "한글"})
        self.assertIn("한글", text)
        self.assertTrue(text.endswith("\n"))
        self.assertNotIn("\\u", text)


class RenderReportExtraTests(unittest.TestCase):
    def test_non_dict_task_and_missing_result_cell(self):
        m = load()
        md = m.render_report({
            "toolOrder": ["rhwp", "pyhwp"],
            "env": {"tools": {"rhwp": {"available": True, "detail": "v"}}},
            "tasks": [
                "skip-me",
                {"task": "info", "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 1, "ok": 1, "successRate": 1.0,
                                 "medianMs": 4}},
                ]},
            ],
            "capabilityMatrix": m.capability_matrix(),
            "verdict": [],
            "kind": "gymCompetitiveBench",
            "schemaVersion": "1.0",
        })
        self.assertIn("| **info** |", md)
        self.assertIn("n/a", md)
        self.assertIn("실행됨", md)

    def test_notes_and_generated_comment(self):
        m = load()
        md = m.render_report({
            "generatedAt": "2026-08-18T00:00:00",
            "env": {"tools": {}},
            "tasks": [{
                "task": "convert",
                "results": [
                    {"tool": "rhwp", "available": True,
                     "summary": {"attempted": 1, "ok": 1, "successRate": 1.0,
                                 "medianMs": 8},
                     "note": "HWP→markdown"},
                ],
            }],
            "capabilityMatrix": m.capability_matrix(),
            "verdict": ["측정 한 줄"],
        })
        self.assertIn("HWP→markdown", md)
        self.assertIn("generated by competitive_bench.py", md)
        self.assertIn("측정 한 줄", md)


class ProfilePathExtraTests(unittest.TestCase):
    def test_release_wins_over_prerelease_segment(self):
        m = load()
        self.assertEqual(
            m.rhwp_profile_from_path("target/prerelease/rhwp"), "debug",
        )
        self.assertEqual(
            m.rhwp_profile_from_path("C:/build/release/rhwp.exe"), "release",
        )
        self.assertEqual(m.rhwp_profile_from_path("just-rhwp"), "debug")


class ResolveToolExtraTests(unittest.TestCase):
    def test_names_scan_when_path_missing(self):
        m = load()
        found = m.resolve_tool(
            None, ["hwp5txt", "soffice"],
            exists=lambda _p: False,
            which=lambda n: "/usr/bin/soffice" if n == "soffice" else None,
        )
        self.assertEqual(found, "/usr/bin/soffice")
        self.assertEqual(
            m.probe(None, ["missing"],),
            None,
        )


class DiscoverCorpusExtraTests(unittest.TestCase):
    def test_uppercase_ext_and_limit_one(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "B.HWP").write_bytes(b"b")
            (root / "a.hwp").write_bytes(b"a")
            (root / "only.hwpx").write_bytes(b"x")
            picked = m.discover_corpus(str(root), 1)
        names = [Path(p).name for p in picked]
        self.assertEqual(len(names), 2)
        self.assertTrue(any(n.lower().endswith(".hwp") for n in names))
        self.assertTrue(any(n.lower().endswith(".hwpx") for n in names))


class StampAndRefreshExtraTests(unittest.TestCase):
    def test_stamp_does_not_overwrite_existing(self):
        m = load()
        kept = m.stamp_report_contract({
            "kind": "gymCompetitiveBench",
            "schemaVersion": "1.0",
            "custom": True,
        })
        self.assertTrue(kept["custom"])
        self.assertEqual(kept["schemaVersion"], "1.0")

    def test_refresh_returns_new_dict(self):
        m = load()
        src = {"tasks": [], "verdict": ["손글"]}
        out = m.refresh_verdict(src)
        self.assertIsNot(out, src)
        self.assertEqual(src["verdict"], ["손글"])
        self.assertNotEqual(out["verdict"], ["손글"])


if __name__ == "__main__":
    unittest.main()
