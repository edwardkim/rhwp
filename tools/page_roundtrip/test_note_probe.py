#!/usr/bin/env python3
"""note_probe 단위 시험 — 가짜 XML, 실문서 ZIP 이 있으면 추가 검증."""

from __future__ import annotations

import json
import sys
import unittest
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import note_probe as np  # noqa: E402

REPO = HERE.parents[1]
SAMPLE_HWPX = REPO / "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx"
ZERO_TWO_LINES = """
<hp:footNote number="7" instId="421">
  <hp:subList>
    <hp:p paraPrIDRef="0" styleIDRef="0">
      <hp:run charPrIDRef="0"><hp:t>간장 기증자 선별</hp:t></hp:run>
      <hp:linesegarray>
        <hp:lineseg textpos="0" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="38276" flags="393216"/>
        <hp:lineseg textpos="20" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="38276" flags="393216"/>
      </hp:linesegarray>
    </hp:p>
  </hp:subList>
</hp:footNote>
"""
HANGUL_ARTIFACT = """
<hp:endNote number="161" instId="99">
  <hp:subList>
    <hp:p>
      <hp:run><hp:t>SO-SUEOP</hp:t></hp:run>
      <hp:linesegarray>
        <hp:lineseg textpos="0" vertpos="2344" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="1000" flags="0"/>
        <hp:lineseg textpos="8" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="1000" flags="0"/>
      </hp:linesegarray>
    </hp:p>
  </hp:subList>
</hp:endNote>
"""
TABLE_NOTE = """
<hp:tbl rowCnt="1" colCnt="1"><hp:tr><hp:tc><hp:subList>
<hp:p><hp:run><hp:ctrl>
<hp:footNote number="2" instId="728">
  <hp:subList>
    <hp:p>
      <hp:run><hp:t>표 안</hp:t></hp:run>
      <hp:linesegarray>
        <hp:lineseg textpos="0" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="8000" flags="0"/>
        <hp:lineseg textpos="4" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="8000" flags="0"/>
        <hp:lineseg textpos="8" vertpos="0" vertsize="1172" textheight="1172" baseline="996" spacing="0" horzpos="0" horzsize="8000" flags="0"/>
      </hp:linesegarray>
    </hp:p>
  </hp:subList>
</hp:footNote>
</hp:ctrl></hp:run></hp:p>
</hp:subList></hp:tc></hp:tr></hp:tbl>
"""


class ParseAttrsTests(unittest.TestCase):
    def test_lineseg_nine_fields(self) -> None:
        seg = np.parse_lineseg_attrs(
            ' textpos="20" vertpos="0" vertsize="1172" textheight="1172" '
            'baseline="996" spacing="0" horzpos="0" horzsize="38276" flags="393216"'
        )
        self.assertEqual(seg.textpos, 20)
        self.assertEqual(seg.vertpos, 0)
        self.assertEqual(seg.vertsize, 1172)
        self.assertEqual(seg.stacked_advance, 1172)

    def test_bad_int_uses_default(self) -> None:
        self.assertEqual(np.parse_int_attr({"vertpos": "x"}, "vertpos", 7), 7)
        self.assertEqual(np.parse_int_attr({}, "vertpos", 3), 3)


class ExtractNotesTests(unittest.TestCase):
    def test_hwp5_zero_pattern(self) -> None:
        notes = np.extract_notes_from_section_xml(ZERO_TWO_LINES)
        self.assertEqual(len(notes), 1)
        self.assertTrue(notes[0].has_hwp5_zero_pattern)
        self.assertEqual(notes[0].paragraphs[0].vpos, [0, 0])
        self.assertEqual(notes[0].number, "7")
        self.assertEqual(notes[0].inst_id, "421")
        self.assertFalse(notes[0].in_table)

    def test_hangul_artifact_is_not_hwp5_zero(self) -> None:
        notes = np.extract_notes_from_section_xml(HANGUL_ARTIFACT)
        self.assertEqual(len(notes), 1)
        self.assertFalse(notes[0].has_hwp5_zero_pattern)
        self.assertTrue(notes[0].paragraphs[0].trailing_zero_after_nonzero)
        self.assertEqual(notes[0].kind.lower(), "endnote")

    def test_table_nested_footnote(self) -> None:
        notes = np.extract_notes_from_section_xml(TABLE_NOTE)
        self.assertEqual(len(notes), 1)
        self.assertTrue(notes[0].in_table)
        self.assertGreaterEqual(notes[0].table_depth, 1)
        self.assertEqual(notes[0].paragraphs[0].vpos, [0, 0, 0])

    def test_summary_counts(self) -> None:
        xml = ZERO_TWO_LINES + HANGUL_ARTIFACT + TABLE_NOTE
        notes = np.extract_notes_from_section_xml(xml)
        summary = np.summarize_notes(notes)
        self.assertEqual(summary["notes"], 3)
        self.assertEqual(summary["footnotes"], 2)
        self.assertEqual(summary["endnotes"], 1)
        self.assertEqual(summary["hwp5ZeroPattern"], 2)
        self.assertEqual(summary["hangulArtifact"], 1)
        self.assertEqual(summary["inTable"], 1)

    def test_notes_to_json_roundtrip_shape(self) -> None:
        notes = np.extract_notes_from_section_xml(ZERO_TWO_LINES)
        blob = np.notes_to_json(notes, source="mem")
        self.assertEqual(blob["kind"], "pageRoundtripNoteProbe")
        self.assertEqual(blob["notes"][0]["paragraphs"][0]["allZeroVpos"], True)


class FixtureTests(unittest.TestCase):
    def test_shipped_notes_index_is_probe_payload(self) -> None:
        path = HERE / "fixtures" / "issue_4882" / "notes_index.json"
        if not path.is_file():
            self.skipTest("notes_index 없음")
        payload = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(payload["kind"], "pageRoundtripNoteProbe")
        self.assertGreaterEqual(payload["summary"]["notes"], 200)
        self.assertGreater(payload["summary"]["totalSegs"], 0)
        first = payload["notes"][0]
        self.assertIn("paragraphs", first)
        self.assertIn("instId", first)

    def test_ndjson_rows_have_nine_fields(self) -> None:
        path = HERE / "fixtures" / "issue_4882" / "note_linesegs.ndjson"
        if not path.is_file():
            self.skipTest("ndjson 없음")
        line = path.read_text(encoding="utf-8").splitlines()[0]
        row = json.loads(line)
        for key in (
            "textpos",
            "vertpos",
            "vertsize",
            "textheight",
            "baseline",
            "spacing",
            "horzpos",
            "horzsize",
            "flags",
        ):
            self.assertIn(key, row)

    @unittest.skipUnless(SAMPLE_HWPX.is_file(), "정책연구 HWPX 없음")
    def test_live_hwpx_has_footnotes(self) -> None:
        xml = np.hwpx_section_xml(SAMPLE_HWPX)
        notes = np.extract_notes_from_section_xml(xml)
        self.assertGreaterEqual(len(notes), 200)
        names = np.hwpx_entry_names(SAMPLE_HWPX)
        self.assertIn("Contents/section0.xml", names)
        self.assertNotIn("META-INF/rhwp-hwp5-origin", names)


if __name__ == "__main__":
    unittest.main()
