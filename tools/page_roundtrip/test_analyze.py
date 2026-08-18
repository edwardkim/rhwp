#!/usr/bin/env python3
"""analyze / catalog_ops / transcript 단위 시험."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import analyze as az  # noqa: E402
import catalog_ops as cop  # noqa: E402
import note_probe as np  # noqa: E402
import transcript as tr  # noqa: E402
from harness import CatalogEntry, load_catalog  # noqa: E402


class AnalyzeTests(unittest.TestCase):
    def test_4882_pre_fix_primary_is_note_or_pages(self) -> None:
        xml = """
        <hp:footNote number="1" instId="421">
          <hp:subList><hp:p><hp:run><hp:t>x</hp:t></hp:run>
          <hp:linesegarray>
            <hp:lineseg textpos="0" vertpos="0" vertsize="1172" textheight="1172" baseline="0" spacing="0" horzpos="0" horzsize="1" flags="0"/>
            <hp:lineseg textpos="1" vertpos="0" vertsize="1172" textheight="1172" baseline="0" spacing="0" horzpos="0" horzsize="1" flags="0"/>
          </hp:linesegarray></hp:p></hp:subList>
        </hp:footNote>
        """
        notes = np.extract_notes_from_section_xml(xml)
        report = az.analyze(
            doc="samples/정책.hwp",
            before=215,
            after=223,
            notes=notes,
            ir_diffs=["section[0] paragraph[421]/ctrl[0]fn.p[0] linesegs: [1].vertpos"],
            issue=4882,
        )
        self.assertEqual(report.delta, 8)
        names = {a.name for a in report.axes}
        self.assertIn("hwp5_note_zero_vpos", names)
        self.assertTrue(any(a.in_scope and a.issue == 4882 for a in report.axes))
        self.assertIn("215→223", "".join(report.notes))

    def test_post_fix_match_note(self) -> None:
        report = az.analyze(doc="x.hwp", before=215, after=215, notes=[], ir_diffs=[])
        self.assertEqual(report.delta, 0)
        self.assertIn("등식", "".join(report.notes))

    def test_4056_is_foreign(self) -> None:
        report = az.analyze(doc="samples/issue-505-equations.hwp", before=4, after=1, issue=4056)
        self.assertTrue(any(a.issue == 4056 and not a.in_scope for a in report.axes))
        self.assertTrue(any("고치지 않는다" in n for n in report.notes))

    def test_ir_classifier(self) -> None:
        self.assertEqual(az.classify_ir_diff("fn.p[0] linesegs: [1].vertpos"), "hwp5_note_zero_vpos")
        self.assertEqual(az.classify_ir_diff("en.p[0] vertpos"), "hangul_hwpx_note_artifact")
        self.assertEqual(az.classify_ir_diff("char_shapes[2]"), "char_shapes_out_of_scope")
        self.assertEqual(az.classify_ir_diff("ole.bin"), "ole_shape_out_of_scope")
        self.assertEqual(az.classify_ir_diff("secd[1]"), "foreign_seat_4056")

    def test_expected_fail_reason_held_seats(self) -> None:
        self.assertIn("범위 밖", az.expected_fail_reason(4056, 4, 1))
        self.assertIn("4→1", az.expected_fail_reason(4056, 4, 1))
        self.assertIn("다른 좌석", az.expected_fail_reason(5128, 69, 68))
        self.assertIn("69→68", az.expected_fail_reason(5128, 69, 68))


class CatalogOpsTests(unittest.TestCase):
    def test_drop_resolved_removes_4882(self) -> None:
        entries = [
            CatalogEntry("samples/a.hwp", "hwpx", 4882, "x"),
            CatalogEntry("samples/b.hwp", "hwpx", 4056, "y"),
            CatalogEntry("samples/c.hwp", "hwpx", 5128, "z"),
        ]
        kept = cop.drop_resolved(entries)
        self.assertEqual({e.issue for e in kept}, {4056, 5128})

    def test_m05_6_scope_errors(self) -> None:
        bad = [CatalogEntry("samples/a.hwp", "hwpx", 4882, "x")]
        errs = cop.assert_m05_6_scope(bad)
        self.assertTrue(any("4882" in e for e in errs))
        self.assertTrue(any("4056" in e for e in errs))

    def test_shipped_catalog_matches_scope(self) -> None:
        entries = load_catalog(HERE / "catalog.json")
        self.assertEqual(cop.assert_m05_6_scope(entries), [])
        missing = cop.require_held(entries)
        self.assertEqual(missing, [])

    def test_diff_catalog(self) -> None:
        old = [
            CatalogEntry("samples/a.hwp", "hwpx", 4882, "x"),
            CatalogEntry("samples/b.hwp", "hwpx", 4056, "y"),
        ]
        new = [
            CatalogEntry("samples/b.hwp", "hwpx", 4056, "y"),
            CatalogEntry("samples/c.hwp", "hwpx", 5128, "z"),
        ]
        diff = cop.diff_catalog(old, new)
        self.assertEqual(diff.removed, (("samples/a.hwp", "hwpx"),))
        self.assertEqual(diff.added, (("samples/c.hwp", "hwpx"),))
        self.assertEqual(diff.kept, (("samples/b.hwp", "hwpx"),))


class TranscriptTests(unittest.TestCase):
    def test_ingest_issue_body_text(self) -> None:
        t = tr.new_transcript(doc="samples/정책.hwp", issue=4882)
        tr.ingest_cli_text(
            t,
            "저장 완료: /tmp/rt.hwpx (5246KB)\n",
            "검증 실패(--verify-pages): 변환 전 215쪽, 재파싱 후 223쪽\n"
            "검증 실패(--verify): /tmp/rt.hwpx 재파싱 후 IR 차이 5건\n"
            "  [차이] section[0] paragraph[421]/ctrl[0]fn.p[0] linesegs: [1].vertpos: expected=0 actual=1172\n",
            4,
        )
        self.assertEqual(t.pages(), (215, 223))
        self.assertEqual(len(t.ir_diffs()), 1)
        self.assertEqual(tr.classify_from_transcript(t, cataloged=True), "EXPECTED_FAIL")
        self.assertEqual(tr.classify_from_transcript(t, cataloged=False), "MISMATCH")

    def test_jsonl_roundtrip(self) -> None:
        t = tr.new_transcript(doc="samples/x.hwp", issue=4882)
        t.add("pages", before=215, after=215, identical=True)
        t.add("verdict", verdict="MATCH")
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "t.jsonl"
            tr.write_jsonl(path, t)
            loaded = tr.load_jsonl(path)
        self.assertEqual(loaded.pages(), (215, 215))
        self.assertEqual(loaded.verdict(), "MATCH")
        self.assertEqual(loaded.doc, "samples/x.hwp")

    def test_shipped_pre_fix_transcript(self) -> None:
        path = HERE / "transcripts" / "issue_4882_pre_fix.jsonl"
        t = tr.load_jsonl(path)
        self.assertEqual(t.issue, 4882)
        self.assertEqual(t.pages(), (215, 223))
        self.assertGreaterEqual(len(t.ir_diffs()), 5)
        self.assertEqual(t.verdict(), "EXPECTED_FAIL")

    def test_shipped_post_fix_transcript(self) -> None:
        path = HERE / "transcripts" / "issue_4882_post_fix.jsonl"
        t = tr.load_jsonl(path)
        self.assertEqual(t.pages(), (215, 215))
        self.assertEqual(t.verdict(), "MATCH")


class MeasuredReportTests(unittest.TestCase):
    def test_measured_report_pins_215(self) -> None:
        path = HERE / "fixtures" / "issue_4882" / "measured_report.json"
        payload = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(payload["pagesOriginal"], 215)
        self.assertEqual(payload["pagesExportReimportAfterFix"], 215)
        self.assertEqual(payload["issue"], 4882)
        self.assertEqual(len(payload["irDiffsBeforeFix"]), 5)


if __name__ == "__main__":
    unittest.main()
