#!/usr/bin/env python3
"""#8149 빈 host 문단기준 자리차지 표(아래 캡션)의 저장 사다리 상자를 보는 HWPX 생성기.

  python3 gen_captioned_host.py samples/issue1549_multipositive_float_tables.hwpx captioned-host.hwpx

모든 문단에 linesegarray 가 없다(한/글이 직접 조판). 한/글로 HWP 저장한 뒤 그 HWP 를 Print 해
기준 PDF 를 만든다. 템플릿은 저장소 공개 샘플이며 header.xml·settings 만 빌려 쓰고, 앞 간격
500HU 문단 모양(모양 12 복제) 하나를 덧붙인 뒤 section0.xml 을 새로 만든다.

1→2쪽: L(빈 host 긴 표가 쪽을 넘김) 뒤 A(글 뒤 빈 host·아래 캡션) · B·C(빈 host 연속 — C 는
앞 간격 흡수) · E(캡션 없는 대조군)
다음 쪽: D(위 캡션 — 이 수정 범위 밖 반례)
"""
import re
import sys
import zipfile

_id = [1000]
SPACED = None  # 앞 간격 500HU 문단 모양 id (main 에서 정함)


def nid():
    _id[0] += 1
    return _id[0]


def run_text(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell_para(text):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{run_text(text)}</hp:p>')


def caption(side, text):
    return (f'<hp:caption side="{side}" fullSz="0" width="8504" gap="852" lastWidth="30000">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'{cell_para(text)}</hp:subList></hp:caption>')


def table(label, cap_side=None, rows=3, row_height=1600):
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
    cap = caption(cap_side, f"{label} 캡션") if cap_side else ""
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{rows}" colCnt="1" '
            f'cellSpacing="0" borderFillIDRef="3" noAdjust="0"><hp:sz width="{width}" widthRelTo="ABSOLUTE" '
            f'height="{rows * row_height}" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="0" '
            f'affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="1" vertRelTo="PARA" '
            f'horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            f'<hp:outMargin left="283" right="283" top="566" bottom="1984"/>{cap}'
            f'<hp:inMargin left="141" right="141" top="141" bottom="141"/>{"".join(trs)}</hp:tbl>')


def para(text, page_break=False):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{SPACED}" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{run_text(text)}</hp:p>')


def host(tbl):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{SPACED}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0">{tbl}</hp:run></hp:p>')


def body():
    # 쪽 스냅 기준이 서지 않는 형상: 앞 쪽에서 넘어온 빈 host 표 조각으로 둘째 쪽이 시작한다
    # (한글문서파일형식 39쪽은 표 234 조각으로 시작). 그 뒤 사례들이 같은 쪽에 놓인다.
    return [
        para("L 앞 글"),
        host(table("L", rows=48)),
        para("A 앞 글"),
        host(table("A", "BOTTOM")),
        para("A 뒤 글"),
        host(table("B", "BOTTOM")),
        host(table("C", "BOTTOM")),
        para("C 뒤 글"),
        host(table("E")),
        para("E 뒤 글"),
        para("D 앞 글", page_break=True),
        host(table("D", "TOP")),
        para("D 뒤 글"),
    ]


def add_spaced_para_pr(header):
    global SPACED
    items = re.findall(r'<hh:paraPr id="(\d+)"', header)
    SPACED = max(int(i) for i in items) + 1
    # 모양 0 을 복제해 앞 간격만 둔다. 앞 간격이 조판되는 모양 12 처럼 HwpUnitChar 갈래와 기본
    # 갈래에 1:2 값을 둔다. 한/글은 문단 모양을 id 가 아니라 목록 순서로 찾으므로 목록 끝에 붙인다.
    base = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', header, re.S).group(0)
    spaced = base.replace('<hh:paraPr id="0"', f'<hh:paraPr id="{SPACED}"', 1)
    prev = iter(["500", "1000"])
    spaced = re.sub(r'<hc:prev value="-?\d+"', lambda m: f'<hc:prev value="{next(prev)}"', spaced)
    spaced = re.sub(r'<hc:next value="-?\d+"', '<hc:next value="0"', spaced)
    end = header.index("</hh:paraProperties>")
    header = header[:end] + spaced + header[end:]
    return re.sub(r'(<hh:paraProperties itemCnt=")(\d+)(")',
                  lambda m: f'{m.group(1)}{int(m.group(2)) + 1}{m.group(3)}', header, count=1)


def main():
    template, out = sys.argv[1], sys.argv[2]
    zin = zipfile.ZipFile(template)
    header = add_spaced_para_pr(zin.read("Contents/header.xml").decode("utf-8"))
    sec = zin.read("Contents/section0.xml").decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run = re.search(r'<hp:p [^>]*>(<hp:run charPrIDRef="0"><hp:secPr.*?</hp:run>)', sec, re.S).group(1)
    first = (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
             f'{sec_run}{run_text("#8149 아래 캡션 빈 host 자리차지 표")}</hp:p>')
    new_sec = sec_head + first + "".join(body()) + "</hs:sec>"
    with zipfile.ZipFile(out, "w") as zout:
        for item in zin.infolist():
            if item.filename.startswith("Preview/"):
                continue
            if item.filename == "Contents/section0.xml":
                data = new_sec.encode("utf-8")
            elif item.filename == "Contents/header.xml":
                data = header.encode("utf-8")
            else:
                data = zin.read(item.filename)
            ct = zipfile.ZIP_STORED if item.filename == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(item, data, compress_type=ct)


if __name__ == "__main__":
    main()
