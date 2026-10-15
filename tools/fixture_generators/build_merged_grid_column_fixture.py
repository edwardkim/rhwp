#!/usr/bin/env python3
"""Build samples/table_merged_grid_column_boundaries.hwpx (synthetic).

Takes samples/issue5590_per_row_column_widths.hwpx as the package template and
replaces its body with four non-TAC tables whose stored cell widths are the only
source of column boundaries:

T1  6 cols, rowspan-covered row; every boundary is fixed only by merged cells.
T2  4 cols, rows contradict each other at boundary 1 (fallback path).
T3  3 cols, consistent rows whose total (35800) is below the declared 36000.
T4  3 cols, single row, no merge.
T5  14 cols, reduced copy of the 80168 "규제 개요" 24x14 grid: the stored cell
    widths of its distinct row partitions, a 7-row label rowspan, a 4-row
    rowspan in column 1; first column narrowed 4800 -> 4498 HU so the table
    equals the 48188 HU body width.

The header declares one face, 한컴바탕 (Haansoft Batang), for all seven script
slots so that the Hancom reference PDF and rhwp use the same face (the template
declared only a HANGUL 함초롬바탕 slot, which the Hancom print server replaced).

Usage: python3 tools/fixture_generators/build_merged_grid_column_fixture.py
"""
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TEMPLATE = ROOT / "samples/issue5590_per_row_column_widths.hwpx"
OUT = ROOT / "samples/table_merged_grid_column_boundaries.hwpx"

# (row, col, row_span, col_span, width)
TABLES = [
    (36000, 6, 3, [
        (0, 0, 2, 2, 21000), (0, 2, 1, 4, 15000),
        (1, 2, 1, 2, 9000), (1, 4, 1, 2, 6000),
        (2, 0, 1, 3, 27000), (2, 3, 1, 3, 9000),
    ]),
    (36000, 4, 2, [
        (0, 0, 1, 1, 9000), (0, 1, 1, 3, 27000),
        (1, 0, 1, 1, 12000), (1, 1, 1, 3, 24000),
    ]),
    (36000, 3, 2, [
        (0, 0, 1, 1, 10000), (0, 1, 1, 2, 25800),
        (1, 0, 1, 2, 22000), (1, 2, 1, 1, 13800),
    ]),
    (36000, 3, 1, [
        (0, 0, 1, 1, 8000), (0, 1, 1, 1, 12000), (0, 2, 1, 1, 16000),
    ]),
    (48188, 14, 7, [
        (0, 0, 7, 1, 4498), (0, 1, 1, 1, 9328), (0, 2, 1, 12, 34362),
        (1, 1, 1, 1, 9328), (1, 2, 1, 5, 12414), (1, 7, 1, 2, 8535), (1, 9, 1, 5, 13413),
        (2, 1, 1, 1, 9328), (2, 2, 1, 4, 11403), (2, 6, 1, 4, 11403), (2, 10, 1, 4, 11556),
        (3, 1, 4, 1, 9328), (3, 2, 1, 1, 5194), (3, 3, 1, 11, 29168),
        (4, 2, 1, 1, 5194), (4, 3, 1, 9, 23973), (4, 12, 1, 2, 5195),
        (5, 2, 1, 3, 10039), (5, 5, 1, 6, 14284), (5, 11, 1, 3, 10039),
        (6, 2, 1, 3, 10039), (6, 5, 1, 6, 14284), (6, 11, 1, 3, 10039),
    ]),
]
ROW_H = 2000
PARA_ID = [1000]


def para_id():
    PARA_ID[0] += 1
    return PARA_ID[0]


def cell_xml(t, r, c, rs, cs, w):
    h = ROW_H * rs
    return (
        '<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
        '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
        'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
        f'<hp:p id="{para_id()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        f'<hp:run charPrIDRef="0"><hp:t>T{t}-{r}-{c}</hp:t></hp:run><hp:linesegarray>'
        f'<hp:lineseg textpos="0" vertpos="0" vertsize="1000" textheight="1000" baseline="850" spacing="600" '
        f'horzpos="0" horzsize="{w - 284}" flags="393216"/></hp:linesegarray></hp:p></hp:subList>'
        f'<hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="{cs}" rowSpan="{rs}"/>'
        f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>'
    )


def table_para(t, width, cols, rows, cells, vpos):
    height = ROW_H * rows
    trs = []
    for r in range(rows):
        tcs = "".join(cell_xml(t, *cell) for cell in cells if cell[0] == r)
        trs.append(f"<hp:tr>{tcs}</hp:tr>")
    tbl = (
        f'<hp:tbl id="{200 + t}" zOrder="{t}" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
        'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" '
        f'rowCnt="{rows}" colCnt="{cols}" cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
        f'<hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{height}" heightRelTo="ABSOLUTE" protect="0"/>'
        '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
        'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
        '<hp:outMargin left="0" right="0" top="283" bottom="283"/>'
        '<hp:inMargin left="141" right="141" top="141" bottom="141"/>'
        + "".join(trs)
        + "</hp:tbl>"
    )
    return (
        f'<hp:p id="{para_id()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        f'<hp:run charPrIDRef="0">{tbl}<hp:t>T{t}</hp:t></hp:run>'
        f'<hp:linesegarray><hp:lineseg textpos="0" vertpos="{vpos}" vertsize="1000" textheight="1000" '
        'baseline="850" spacing="600" horzpos="0" horzsize="48188" flags="393216"/></hp:linesegarray></hp:p>\n'
    )


LANGS = ("HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER")
FONTFACES = (
    f'<hh:fontfaces itemCnt="{len(LANGS)}">'
    + "".join(
        f'<hh:fontface lang="{lang}" fontCnt="1">'
        '<hh:font id="0" face="한컴바탕" type="TTF" isEmbedded="0"/></hh:fontface>'
        for lang in LANGS
    )
    + "</hh:fontfaces>"
)


def main():
    with zipfile.ZipFile(TEMPLATE) as src:
        entries = [(info, src.read(info.filename)) for info in src.infolist()]
    section = next(data for info, data in entries if info.filename == "Contents/section0.xml").decode()
    # Keep the first paragraph's section definition, drop its table and the trailing paragraph.
    head = section[: section.index("<hp:tbl ")]
    head = head.replace('<hp:bookmark name="fixture-anchor"/>', "")
    first = head + '<hp:t>merged grid column boundaries</hp:t></hp:run>' \
        '<hp:linesegarray><hp:lineseg textpos="0" vertpos="0" vertsize="1000" textheight="1000" baseline="850" ' \
        'spacing="600" horzpos="0" horzsize="48188" flags="393216"/></hp:linesegarray></hp:p>\n'
    body = []
    # Stored LINE_SEG ladder as Hancom 2020 writes it for this file (checked on its
    # HWP re-save): the anchor line sits below its table, i.e. table top + table
    # height + outer margins (283 + 283), and the next table starts one line
    # (1000 + 600) below that line.
    top = 1600
    for t, (width, cols, rows, cells) in enumerate(TABLES, start=1):
        vpos = top + ROW_H * rows + 566
        body.append(table_para(t, width, cols, rows, cells, vpos))
        top = vpos + 1600
    new_section = first + "".join(body) + "</hs:sec>\n"
    assert re.search(r"<hs:sec ", new_section)
    with zipfile.ZipFile(OUT, "w") as dst:
        for info, data in entries:
            if info.filename == "Contents/section0.xml":
                data = new_section.encode()
            elif info.filename == "Contents/header.xml":
                header = data.decode()
                header, count = re.subn(r"<hh:fontfaces .*?</hh:fontfaces>", FONTFACES, header, flags=re.S)
                assert count == 1
                data = header.encode()
            dst.writestr(info, data)
    print(OUT.relative_to(ROOT))


if __name__ == "__main__":
    main()
