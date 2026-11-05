#!/usr/bin/env python3
"""끝 행을 덮는 병합 칸이 끝 행을 키운 표가 쪽 바닥에 걸릴 때 한/글이 끝 행을 어떻게 그리는가.

  python3 gen_span_page_bottom.py samples/issue1549_multipositive_float_tables.hwpx out.hwpx

쪽마다 바깥 여백 0 의 1x1 자리차지 간격 표(높이 S)를 두고 그 아래에 6행 x 3열 표를 둔다.
P·Q·R·S 표는 행 선언 2048HU, 1~5행 0열 병합 칸 선언 = 행 합 + 849HU, 표 선언 = 행 합 + 849HU 다
(일관된 저장값, 입법예고문 docs#333 11쪽 「포상ㆍ서훈」 칸과 같은 꼴). 간격 표 높이는 표 전체(병합 칸 초과분 포함)가 본문 바닥을
d HU 만큼 넘도록 고른다: d = -150, 225, 450, 849.
U·V(d = 225, 450)는 4행에 1행 칸이 없는 표다 — 0열 3~4행 병합, 1·2열 4~5행 병합이 끝 행을 키우고
5행에는 0열만 1행 칸이 있다(쪽을 넘어온 표의 끝에서 관찰한 꼴). 모든 문단에 linesegarray 가 없다.
"""
import re
import sys
import zipfile

_id = [1000]
ROW = 2048
ROWS = 6
WIDTHS = [6000, 12000, 12000]
SPAN_EXTRA = 849
BODY = 65762  # 템플릿 본문 높이(HU)
OVERSHOOT = [-150, 225, 450, 849]
OPEN_ROW_OVERSHOOT = [225, 450]


def nid():
    _id[0] += 1
    return _id[0]


def run_text(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def para(text, page_break=False, lead=""):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{lead}{run_text(text)}</hp:p>')


def cell(text, col, row, width, height, col_span=1, row_span=1):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'{para(text)}</hp:subList><hp:cellAddr colAddr="{col}" rowAddr="{row}"/>'
            f'<hp:cellSpan colSpan="{col_span}" rowSpan="{row_span}"/><hp:cellSz width="{width}" height="{height}"/>'
            f'<hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')


def tbl(rows_xml, rows, cols, width, height, margin):
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{rows}" colCnt="{cols}" '
            f'cellSpacing="0" borderFillIDRef="3" noAdjust="0"><hp:sz width="{width}" widthRelTo="ABSOLUTE" '
            f'height="{height}" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="0" affectLSpacing="0" '
            f'flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="COLUMN" '
            f'vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            f'<hp:outMargin left="{margin}" right="{margin}" top="{margin}" bottom="{margin}"/>'
            f'<hp:inMargin left="141" right="141" top="141" bottom="141"/>{rows_xml}</hp:tbl>')


def span_table(label):
    width = sum(WIDTHS)
    rows = [f'<hp:tr>{cell(label + " 머리", 0, 0, width, ROW, col_span=3)}</hp:tr>']
    for r in range(1, ROWS):
        cells = ""
        if r == 1:
            cells += cell(label + " 병합", 0, 1, WIDTHS[0], ROW * (ROWS - 1) + SPAN_EXTRA, row_span=ROWS - 1)
        cells += cell(f"{label}{r}가", 1, r, WIDTHS[1], ROW) + cell(f"{label}{r}나", 2, r, WIDTHS[2], ROW)
        rows.append(f'<hp:tr>{cells}</hp:tr>')
    return tbl("".join(rows), ROWS, 3, width, ROW * ROWS + SPAN_EXTRA, 141)


def open_row_table(label):
    """4행에 1행 칸이 없는 표 — 0열 3~4행 병합, 1·2열 4~5행 병합(끝 행을 키운다), 5행은 0열만 1행 칸."""
    width = sum(WIDTHS)
    rows = [f'<hp:tr>{cell(label + " 머리", 0, 0, width, ROW, col_span=3)}</hp:tr>']
    for r in range(1, 3):
        rows.append(f'<hp:tr>{cell(f"{label}{r}다", 0, r, WIDTHS[0], ROW)}'
                    f'{cell(f"{label}{r}가", 1, r, WIDTHS[1], ROW)}{cell(f"{label}{r}나", 2, r, WIDTHS[2], ROW)}</hp:tr>')
    rows.append(f'<hp:tr>{cell(label + " 왼병합", 0, 3, WIDTHS[0], ROW * 2, row_span=2)}'
                f'{cell(f"{label}3가", 1, 3, WIDTHS[1], ROW)}{cell(f"{label}3나", 2, 3, WIDTHS[2], ROW)}</hp:tr>')
    rows.append(f'<hp:tr>{cell(label + " 병합", 1, 4, WIDTHS[1], ROW * 2 + SPAN_EXTRA, row_span=2)}'
                f'{cell(label + " 병합나", 2, 4, WIDTHS[2], ROW * 2 + SPAN_EXTRA, row_span=2)}</hp:tr>')
    rows.append(f'<hp:tr>{cell(f"{label}5가", 0, 5, WIDTHS[0], ROW)}</hp:tr>')
    return tbl("".join(rows), ROWS, 3, width, ROW * ROWS + SPAN_EXTRA, 141)


def spacer(height):
    rows = f'<hp:tr>{cell("", 0, 0, 30000, height)}</hp:tr>'
    return tbl(rows, 1, 1, 30000, height, 0)


def main():
    template, out = sys.argv[1], sys.argv[2]
    zin = zipfile.ZipFile(template)
    files = [(i.filename, zin.read(i.filename)) for i in zin.infolist()]
    data = dict(files)
    sec = data["Contents/section0.xml"].decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run_inner = re.search(r'<hp:p [^>]*><hp:run charPrIDRef="0">(<hp:secPr.*?</hp:secPr>.*?)</hp:run>',
                              sec, re.S).group(1)
    sec_lead = f'<hp:run charPrIDRef="0">{sec_run_inner}</hp:run>'

    def obj_para(obj, page_break=False, lead=""):
        return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
                f'columnBreak="0" merged="0">{lead}<hp:run charPrIDRef="0">{obj}</hp:run></hp:p>')

    paras = []
    table_total = 141 + ROW * ROWS + SPAN_EXTRA + 141
    cases = [("PQRS"[i], d, span_table) for i, d in enumerate(OVERSHOOT)]
    cases += [("UV"[i], d, open_row_table) for i, d in enumerate(OPEN_ROW_OVERSHOOT)]
    for i, (label, d, make) in enumerate(cases):
        lead = sec_lead if i == 0 else ""
        paras.append(obj_para(spacer(BODY - table_total + d), page_break=i > 0, lead=lead))
        paras.append(obj_para(make(label)))
        paras.append(para(f"{label} 뒤"))
    section = (sec_head + "".join(paras) + "</hs:sec>").encode("utf-8")
    with zipfile.ZipFile(out, "w") as zout:
        for name, raw in files:
            if name.startswith("Preview/"):
                continue
            if name == "Contents/section0.xml":
                raw = section
            ct = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(name, raw, compress_type=ct)


if __name__ == "__main__":
    main()
