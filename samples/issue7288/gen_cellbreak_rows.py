#!/usr/bin/env python3
"""«쪽 경계에서» 값 1 «셀 단위로 나눔» 표가 쪽 끝 행을 자르는가 — 합성 입력.

  python gen_cellbreak_rows.py samples/issue2527_empty_linesegs.hwpx <out_dir>

cellbreak_{TABLE|CELL}_v{VOFF}_L{LINES}.hwpx: 채움 문단 FILL 개 뒤에 2열 ROWS 행 자리차지 표.
왼쪽 칸은 행 번호 한 줄, 오른쪽 칸은 한 줄 문단 LINES 개로 행 높이를 내용으로 채운다.
pageBreak="TABLE" 은 HWP5 값 1(«셀 단위로 나눔»), "CELL" 은 값 2(«나눔»)다. VOFF 는 표의
문단 기준 세로 오프셋(HU). 저장 줄 없음(한/글이 조판).
저장소 입력: VOFFS=502 의 TABLE·CELL 두 파일(기본값 ROWS=30 · LINES=7 · FILL=2). VOFF=0 도 한/글 2020·2024 에서 같은 쪽별 행이다.
"""
import os
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
out_dir = sys.argv[2]
os.makedirs(out_dir, exist_ok=True)
hdr = src.read('Contents/header.xml').decode('utf8')
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]
ROWS = int(os.environ.get('ROWS', '30'))
LINES = int(os.environ.get('LINES', '7'))
FILL = int(os.environ.get('FILL', '2'))


class Ids:
    def __init__(self):
        self.value = 900

    def __call__(self):
        self.value += 1
        return self.value


def p(nid, text):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run></hp:p>')


def cell(c, r, w, inner):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{inner}</hp:subList>'
            f'<hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="1500"/>'
            f'<hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>')


def table(nid, page_break, voff):
    w0, w1 = 9000, 33000
    rows = []
    for r in range(ROWS):
        right = p(nid, f'행 R{r:02d} 내용') + ''.join(p(nid, f'R{r:02d} 줄 {k + 2}') for k in range(LINES - 1))
        rows.append(f'<hp:tr>{cell(0, r, w0, p(nid, f"라벨 {r}"))}{cell(1, r, w1, right)}</hp:tr>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="{page_break}" repeatHeader="0" rowCnt="{ROWS}" colCnt="2" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w0 + w1}" widthRelTo="ABSOLUTE" height="{1500 * ROWS}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            f'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="{voff}" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>' + ''.join(rows) + '</hp:tbl>')


def doc(page_break, voff):
    nid = Ids()
    body = [head + '</hp:run>' + f'<hp:run charPrIDRef="0"><hp:t>cellbreak {page_break} v={voff} L={LINES}</hp:t></hp:run></hp:p>\n']
    for i in range(FILL):
        body.append(p(nid, f'채움 {i + 1:02d}') + '\n')
    body.append(f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                '<hp:run charPrIDRef="0">' + table(nid, page_break, voff) + '<hp:t/></hp:run></hp:p>\n')
    body.append(p(nid, '표 뒤 TAIL') + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 21, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


for page_break in os.environ.get('PB', 'TABLE,CELL').split(','):
    for voff in (int(x) for x in os.environ.get('VOFFS', '0,502').split(',')):
        out = os.path.join(out_dir, f'cellbreak_{page_break}_v{voff}_L{LINES}.hwpx')
        with zipfile.ZipFile(out, 'w') as z:
            put(z, 'mimetype', src.read('mimetype'), stored=True)
            for name in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
                put(z, name, src.read(name))
            put(z, 'Contents/header.xml', hdr.encode('utf8'))
            put(z, 'Contents/section0.xml', doc(page_break, voff).encode('utf8'))
        print(out)
