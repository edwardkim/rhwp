#!/usr/bin/env python3
"""빈 host 문단의 자리차지 표 두 개(offset 0 / offset 15000HU) 사이 틈을 host 줄·빈 문단·글 줄이
어떻게 채우는지 보는 HWPX 생성기. 모든 문단에 linesegarray 가 없다(한/글이 직접 조판).

  python3 gen_empty_host.py samples/issue1549_multipositive_float_tables.hwpx empty-host.hwpx

템플릿은 저장소 공개 샘플이며 header.xml·settings 만 빌려 쓰고 section0.xml 을 새로 만든다.
쪽마다 사례 하나(H1~H6).
"""
import re
import sys
import zipfile

_id = [1000]


def nid():
    _id[0] += 1
    return _id[0]


def cell_para(text):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run></hp:p>')


def table(vert, horz, height, label):
    width, rows = 18000, 2
    trs = []
    for r in range(rows):
        tc = (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
              f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
              f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
              f'{cell_para(f"{label} 행{r + 1}")}</hp:subList><hp:cellAddr colAddr="0" rowAddr="{r}"/>'
              f'<hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="{width}" height="{height // rows}"/>'
              f'<hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>')
        trs.append(f'<hp:tr>{tc}</hp:tr>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="NONE" repeatHeader="0" rowCnt="{rows}" colCnt="1" '
            f'cellSpacing="0" borderFillIDRef="3" noAdjust="0"><hp:sz width="{width}" widthRelTo="ABSOLUTE" '
            f'height="{height}" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="0" affectLSpacing="0" '
            f'flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" '
            f'vertAlign="TOP" horzAlign="LEFT" vertOffset="{vert}" horzOffset="{horz}"/>'
            f'<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
            f'<hp:inMargin left="510" right="510" top="141" bottom="141"/>{"".join(trs)}</hp:tbl>')


def para(text, page_break=False, para_pr=0):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{para_pr}" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run></hp:p>')


T1 = (0, 0, 8000, "T1")
T2 = (15000, 20000, 8000, "T2")


def host(text, para_pr=0):
    ctrls = table(*T1) + table(*T2)
    body = f'<hp:t>{text}</hp:t>' if text is not None else ''
    return (f'<hp:p id="{nid()}" paraPrIDRef="{para_pr}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0">{ctrls}{body}</hp:run></hp:p>')


def empty(para_pr=0):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{para_pr}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0"/></hp:p>')


CASES = [
    ("H1 빈 host 뒤 글 줄", lambda: [host(None)] + [para(f"H1 글 {i}") for i in range(1, 9)]),
    ("H2 빈 host 뒤 빈 문단 둘", lambda: [host(None), empty(), empty(), para("H2 표 뒤 글")]),
    ("H3 빈 host 뒤 빈 문단 하나", lambda: [host(None), empty(), para("H3 표 뒤 글")]),
    ("H4 빈 host 뒤 빈 문단 넷", lambda: [host(None)] + [empty() for _ in range(4)] + [para("H4 표 뒤 글")]),
    ("H5 공백 host 뒤 빈 문단 둘", lambda: [host(" "), empty(), empty(), para("H5 표 뒤 글")]),
    ("H6 앞 간격 있는 빈 문단", lambda: [host(None, 12), empty(12), empty(12), para("H6 표 뒤 글", para_pr=12)]),
]


def main():
    template, out = sys.argv[1], sys.argv[2]
    zin = zipfile.ZipFile(template)
    sec = zin.read("Contents/section0.xml").decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run = re.search(r'<hp:p [^>]*>(<hp:run charPrIDRef="0"><hp:secPr.*?</hp:run>)', sec, re.S).group(1)
    body = [f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{sec_run}<hp:run charPrIDRef="0"><hp:t>#7680 빈 host 실험</hp:t></hp:run></hp:p>']
    for i, (name, build) in enumerate(CASES):
        body.append(para(f"사례 {name}", page_break=i > 0))
        body.extend(build())
    new_sec = sec_head + "".join(body) + "</hs:sec>"
    with zipfile.ZipFile(out, "w") as zout:
        for item in zin.infolist():
            if item.filename.startswith("Preview/"):
                continue
            data = new_sec.encode("utf-8") if item.filename == "Contents/section0.xml" else zin.read(item.filename)
            ct = zipfile.ZIP_STORED if item.filename == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(item, data, compress_type=ct)


if __name__ == "__main__":
    main()
