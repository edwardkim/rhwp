#!/usr/bin/env python3
"""세로쓰기 칸이 있는 표를 한/글이 쪽 경계에서 나누는가 — 가로쓰기 대조군과 짝지은 HWPX 생성기.

  python3 gen_vertical_cell_table.py samples/issue1549_multipositive_float_tables.hwpx oversize out.hwpx
  python3 gen_vertical_cell_table.py samples/issue1549_multipositive_float_tables.hwpx fit out.hwpx

oversize: H3·V3 — 쪽 맨 위 자리차지 표(행 36개, 본문보다 943HU 김). 걸침 칸 행 30..35.
fit: H6·V6 — 채움 20줄 뒤 표(행 30개, 걸침 행 5..7). 쪽 바닥에서 나눌 자리가 있다.
H 는 0열 걸침 칸이 가로쓰기, V 는 세로쓰기다. 그 밖은 같다.

모든 사례는 쪽 나눔 뒤 빈 문단의 문단 기준 자리차지 표(2열, 행 1849HU)다. 0열 걸침 칸 하나와
1열 낱칸들. 본문 65762HU 에 행 35개(0..34)가 들어가고 행 35 에서 쪽이 끝난다.
변수: 걸침 칸 글 방향(HORIZONTAL/VERTICAL) × 걸침 범위 × 행 수.
"""
import re
import sys
import zipfile

_id = [1000]
ROW_H = 1849
W0, W1 = 3000, 27000


def nid():
    _id[0] += 1
    return _id[0]


def run_text(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell_para(text):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{run_text(text)}</hp:p>')


def tc(text, c, r, rs, width, height, direction="HORIZONTAL"):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
            f'<hp:subList id="" textDirection="{direction}" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'{cell_para(text)}</hp:subList><hp:cellAddr colAddr="{c}" rowAddr="{r}"/>'
            f'<hp:cellSpan colSpan="1" rowSpan="{rs}"/><hp:cellSz width="{width}" height="{height}"/>'
            f'<hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')


def tbl(label, rows, span, direction, page_break="CELL"):
    s0, s1 = span
    trs = []
    for r in range(rows):
        cells = []
        if r == s0:
            cells.append(tc("임용전" if s1 == s0 else "임용전경력", 0, r, s1 - s0 + 1, W0, ROW_H * (s1 - s0 + 1), direction))
        elif not (s0 < r <= s1):
            cells.append(tc("", 0, r, 1, W0, ROW_H))
        cells.append(tc(f"{label} 행{r + 1}", 1, r, 1, W1, ROW_H))
        trs.append(f'<hp:tr>{"".join(cells)}</hp:tr>')
    pos = ('<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
           'vertRelTo="PARA" horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="{page_break}" repeatHeader="0" rowCnt="{rows}" colCnt="2" '
            f'cellSpacing="0" borderFillIDRef="3" noAdjust="0"><hp:sz width="{W0 + W1}" widthRelTo="ABSOLUTE" '
            f'height="{rows * ROW_H}" heightRelTo="ABSOLUTE" protect="0"/>{pos}'
            f'<hp:outMargin left="141" right="141" top="141" bottom="141"/>'
            f'<hp:inMargin left="141" right="141" top="141" bottom="141"/>{"".join(trs)}</hp:tbl>')


def para(text, page_break=False):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{run_text(text)}</hp:p>')


def host(obj):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="1" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0">{obj}</hp:run></hp:p>')


# (라벨, 행 수, 걸침 행 범위, 글 방향, 쪽 나눔, 앞 채움 줄 수)
SETS = {
    "oversize": [
        ("H3", 36, (30, 35), "HORIZONTAL", "CELL", 0),
        ("V3", 36, (30, 35), "VERTICAL", "CELL", 0),
    ],
    "fit": [
        ("H6", 30, (5, 7), "HORIZONTAL", "CELL", 20),
        ("V6", 30, (5, 7), "VERTICAL", "CELL", 20),
    ],
}


def body(cases):
    out = []
    for label, rows, span, direction, pb, fill in cases:
        if fill:
            out.append(para(f"{label} 머리", page_break=True))
            out.extend(para(f"{label} 채움 {n}") for n in range(1, fill + 1))
            out.append(host(tbl(label, rows, span, direction, pb)).replace('pageBreak="1"', 'pageBreak="0"', 1))
        else:
            out.append(host(tbl(label, rows, span, direction, pb)))
        out.append(para(f"{label} 끝"))
    return out


def main():
    template, which, out = sys.argv[1], sys.argv[2], sys.argv[3]
    zin = zipfile.ZipFile(template)
    files = [(i.filename, zin.read(i.filename)) for i in zin.infolist()]
    sec = dict(files)["Contents/section0.xml"].decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run = re.search(r'<hp:p [^>]*>(<hp:run charPrIDRef="0"><hp:secPr.*?</hp:run>)', sec, re.S).group(1)
    first = (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
             f'{sec_run}{run_text(f"세로쓰기 칸 표 쪽 나눔 — {which}")}</hp:p>')
    new_sec = sec_head + first + "".join(body(SETS[which])) + "</hs:sec>"
    with zipfile.ZipFile(out, "w") as zout:
        for name, data in files:
            if name.startswith("Preview/"):
                continue
            if name == "Contents/section0.xml":
                data = new_sec.encode("utf-8")
            ct = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(name, data, compress_type=ct)


if __name__ == "__main__":
    main()
