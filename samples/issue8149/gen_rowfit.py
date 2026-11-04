#!/usr/bin/env python3
"""#8149 쪽 바닥에서 빈 host 자리차지 표의 첫 조각이 몇 행에서 끊기는지 보는 HWPX 생성기.

  python3 gen_rowfit.py samples/issue1549_multipositive_float_tables.hwpx rowfit.hwpx

사례마다 쪽 나눔으로 새 쪽을 열고 채움 글줄 뒤에 빈 host 문단기준 자리차지 표(1열, 행 30개)를
둔다. 표는 쪽 바닥에서 나뉜다. 변수: 아래 캡션 유무 × 바깥 아래 여백(1984/0HU) × 행 높이
(1300/1500/1700/1900HU). 한/글로 HWP 저장한 뒤 Print 해 각 쪽 첫 조각의 마지막 행 바닥을 잰다.
모든 문단에 linesegarray 가 없다(한/글이 직접 조판).
"""
import re
import sys
import zipfile

_id = [1000]


def nid():
    _id[0] += 1
    return _id[0]


def run_text(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell_para(text):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{run_text(text)}</hp:p>')


def table(label, caption, outer_bottom, row_height, rows=30):
    width = 30000
    trs = []
    for r in range(rows):
        tc = (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
              f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
              f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
              f'{cell_para(f"{label} 행{r + 1}")}</hp:subList><hp:cellAddr colAddr="0" rowAddr="{r}"/>'
              f'<hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="{width}" height="{row_height}"/>'
              f'<hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')
        trs.append(f'<hp:tr>{tc}</hp:tr>')
    cap = ""
    if caption:
        cap = (f'<hp:caption side="BOTTOM" fullSz="0" width="8504" gap="852" lastWidth="{width}">'
               f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
               f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
               f'{cell_para(f"{label} 캡션")}</hp:subList></hp:caption>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{rows}" colCnt="1" '
            f'cellSpacing="0" borderFillIDRef="3" noAdjust="0"><hp:sz width="{width}" widthRelTo="ABSOLUTE" '
            f'height="{rows * row_height}" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="0" '
            f'affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="1" vertRelTo="PARA" '
            f'horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            f'<hp:outMargin left="283" right="283" top="566" bottom="{outer_bottom}"/>{cap}'
            f'<hp:inMargin left="141" right="141" top="141" bottom="141"/>{"".join(trs)}</hp:tbl>')


def para(text, page_break=False):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{run_text(text)}</hp:p>')


def host(tbl):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0">{tbl}</hp:run></hp:p>')


CASES = [(cap, ob, rh) for cap in (True, False) for ob in (1984, 0) for rh in (1300, 1500, 1700, 1900)]


def body():
    out = []
    for i, (cap, ob, rh) in enumerate(CASES):
        label = f"R{i + 1:02d}"
        out.append(para(f"{label} 캡션={'있음' if cap else '없음'} 아래여백={ob} 행높이={rh}", page_break=i > 0))
        out.extend(para(f"{label} 채움 {n}") for n in range(1, 25))
        out.append(host(table(label, cap, ob, rh)))
    out.append(para("끝", page_break=True))
    return out


def main():
    template, out = sys.argv[1], sys.argv[2]
    zin = zipfile.ZipFile(template)
    sec = zin.read("Contents/section0.xml").decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run = re.search(r'<hp:p [^>]*>(<hp:run charPrIDRef="0"><hp:secPr.*?</hp:run>)', sec, re.S).group(1)
    first = (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
             f'{sec_run}{run_text("#8149 쪽 바닥 표 첫 조각 행 수용")}</hp:p>')
    new_sec = sec_head + first + "".join(body()) + "</hs:sec>"
    with zipfile.ZipFile(out, "w") as zout:
        for item in zin.infolist():
            if item.filename.startswith("Preview/"):
                continue
            data = new_sec.encode("utf-8") if item.filename == "Contents/section0.xml" else zin.read(item.filename)
            ct = zipfile.ZIP_STORED if item.filename == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(item, data, compress_type=ct)


if __name__ == "__main__":
    main()
