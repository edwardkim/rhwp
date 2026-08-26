#!/usr/bin/env python3
"""spec_probe 단위 시험 — 가짜 섹션 XML, 실문서 ZIP 불필요."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import spec_probe as sp  # noqa: E402

SNIPPET = """
<hs:sec>
  <hp:p id="0" pageBreak="none">
    <hp:run><hp:t>표지</hp:t></hp:run>
    <hp:linesegarray>
      <hp:lineseg textpos="0" vertpos="0" vertsize="2000" textheight="2000" baseline="1600" spacing="0" horzpos="0" horzsize="40000" flags="1"/>
    </hp:linesegarray>
  </hp:p>
  <hp:p id="1">
    <hp:run><hp:t>본문 앞</hp:t></hp:run>
    <hp:linesegarray>
      <hp:lineseg textpos="0" vertpos="40000" vertsize="1200" textheight="1200" baseline="1000" spacing="0" horzpos="0" horzsize="40000" flags="393216"/>
      <hp:lineseg textpos="20" vertpos="0" vertsize="1200" textheight="1200" baseline="1000" spacing="0" horzpos="0" horzsize="40000" flags="393216"/>
    </hp:linesegarray>
  </hp:p>
  <hp:p id="2">
    <hp:run><hp:t></hp:t></hp:run>
    <hp:tbl rowCnt="2" colCnt="2" pageBreak="TABLE">
      <hp:sz width="10000" height="20000"/>
      <hp:tr><hp:tc><hp:p><hp:run><hp:t>셀</hp:t></hp:run></hp:p></hp:tc></hp:tr>
    </hp:tbl>
    <hp:linesegarray>
      <hp:lineseg textpos="0" vertpos="1000" vertsize="8000" textheight="8000" baseline="1000" spacing="0" horzpos="0" horzsize="40000" flags="1"/>
    </hp:linesegarray>
  </hp:p>
</hs:sec>
"""


class ParseTests(unittest.TestCase):
    def test_lineseg_flags(self) -> None:
        seg = sp.parse_lineseg_attrs(
            'textpos="0" vertpos="0" vertsize="1" textheight="1" baseline="1" spacing="0" horzpos="0" horzsize="1" flags="3"'
        )
        self.assertTrue(seg.first_of_page)
        self.assertTrue(seg.first_of_column)
        self.assertEqual(seg.stacked_advance, 1)

    def test_extract_paragraphs_skips_cell_paras(self) -> None:
        paras = sp.extract_paragraphs(SNIPPET, 0)
        self.assertEqual(len(paras), 3)
        self.assertEqual(paras[0].text, "표지")
        self.assertTrue(paras[0].has_page_first_seg)
        self.assertTrue(paras[1].has_vpos_reset)
        self.assertEqual(paras[1].vpos, [40000, 0])
        self.assertEqual(len(paras[2].tables), 1)
        self.assertEqual(paras[2].tables[0].row_cnt, 2)
        self.assertEqual(paras[2].tables[0].height, 20000)
        self.assertNotIn("셀", paras[2].text)

    def test_pinned_contract(self) -> None:
        c = sp.pinned_contract()
        self.assertEqual(c["pages"], 69)
        self.assertEqual(c["sections"], 6)
        self.assertEqual(c["paragraphs"], 619)
        self.assertEqual(c["p015"]["paraIndex"], 73)
        self.assertEqual(c["p016"]["paraIndex"], 84)
        self.assertIn(174, c["splitTables"])

    def test_summarize(self) -> None:
        paras = sp.extract_paragraphs(SNIPPET, 0)
        s = sp.summarize_paragraphs(paras)
        self.assertEqual(s["paragraphs"], 3)
        self.assertEqual(s["tables"], 1)
        self.assertEqual(s["vposResetParas"], 1)
        self.assertGreaterEqual(s["lineSegs"], 4)

    def test_strip_inner_tables(self) -> None:
        xml = "<hp:p>앞<hp:tbl><hp:p>안</hp:p></hp:tbl>뒤</hp:p>"
        self.assertEqual(sp.strip_inner_tables(xml), "<hp:p>앞뒤</hp:p>")


if __name__ == "__main__":
    unittest.main()
