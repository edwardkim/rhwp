#!/usr/bin/env python3
"""여러 쪽에 걸친 쪽 나눔 표의 칸 각주가 어느 쪽에 놓이는가 — 합성 입력.

  python gen_cell_notes_multipage.py samples/issue2527_empty_linesegs.hwpx <out_dir>

cellnotes_{plain|span}_N{NOTE_REP}[_H{ROW_H}][_B][_L{LINES}].hwpx: 채움 문단 FILL 개 뒤에 2열 ROWS 행 표(pageBreak=CELL).
span 은 왼쪽 열을 10행씩 병합한다. ROW_H 는 행 선언 높이(HU), BODY_NOTE=1 은 표 앞 본문 각주, LINES 는 오른쪽 칸 문단 수(행을 내용으로 채움). 각주는 NOTE_ROWS 행 오른쪽 칸에 하나씩. 저장 줄 없음.
저장소 입력: 기본값 → cellnotes_span_N2 · ROWS=110 NOTE_ROWS=5,15,50,95 → r110_cellnotes_span_N2(이름 앞 r110_).
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
ROWS = int(os.environ.get('ROWS', '70'))
ROW_H = int(os.environ.get('ROW_H', '1500'))
BODY_NOTE = os.environ.get('BODY_NOTE', '0') == '1'
LINES = int(os.environ.get('LINES', '1'))
NOTE_ROWS = tuple(int(x) for x in os.environ.get('NOTE_ROWS', '5,15,50').split(','))
NOTE_REP = int(os.environ.get('NOTE_REP', '2'))
NOTE_TEXT = '각주 본문 줄을 길게 채운다 ' * NOTE_REP


class Ids:
    def __init__(self):
        self.value = 900

    def __call__(self):
        self.value += 1
        return self.value


def p(nid, runs):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{runs}</hp:p>')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def footnote(nid, number):
    sub = (f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
           f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
           f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
           f'<hp:run charPrIDRef="0"><hp:ctrl><hp:autoNum num="{number}" numType="FOOTNOTE">'
           f'<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>'
           f'</hp:autoNum></hp:ctrl><hp:t> {NOTE_TEXT}</hp:t></hp:run></hp:p></hp:subList>')
    return f'<hp:ctrl><hp:footNote number="{number}" suffixChar="41" instId="{nid()}">{sub}</hp:footNote></hp:ctrl>'


def cell(nid, c, r, w, inner, row_span=1, valign="CENTER"):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="{valign}" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{inner}</hp:subList>'
            f'<hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="{row_span}"/>'
            f'<hp:cellSz width="{w}" height="{ROW_H * row_span}"/>'
            f'<hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>')


def table(nid, span):
    w0, w1 = 9000, 33000
    rows = []
    note = 0
    for r in range(ROWS):
        cells = ''
        if span:
            if r % 10 == 0:
                cells += cell(nid, 0, r, w0, p(nid, t(f'묶음 {r // 10 + 1}')), row_span=min(10, ROWS - r), valign=os.environ.get('SPAN_VALIGN', 'TOP'))
        else:
            cells += cell(nid, 0, r, w0, p(nid, t(f'라벨 {r}')))
        runs = t(f'행 R{r:02d} 내용')
        if r in NOTE_ROWS:
            note += 1
            runs = f'<hp:run charPrIDRef="0"><hp:t>행 R{r:02d} 내용</hp:t>{footnote(nid, note)}<hp:t/></hp:run>'
        cells += cell(nid, 1, r, w1, p(nid, runs) + ''.join(p(nid, t(f'R{r:02d} 줄 {k + 2}')) for k in range(LINES - 1)))
        rows.append(f'<hp:tr>{cells}</hp:tr>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{ROWS}" colCnt="2" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w0 + w1}" widthRelTo="ABSOLUTE" height="{ROW_H * ROWS}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>' + ''.join(rows) + '</hp:tbl>')


def doc(fill, span):
    nid = Ids()
    body = [head + '</hp:run>' + t(f'fn-table span={int(span)} F={fill}') + '</hp:p>\n']
    for i in range(fill):
        body.append(p(nid, t(f'채움 {i + 1:02d}')) + '\n')
    if BODY_NOTE:
        body.append(p(nid, f'<hp:run charPrIDRef="0"><hp:t>표 앞 본문 각주</hp:t>{footnote(nid, 0)}<hp:t/></hp:run>') + '\n')
    body.append(p(nid, '<hp:run charPrIDRef="0">' + table(nid, span) + '<hp:t/></hp:run>') + '\n')
    body.append(p(nid, t('표 뒤 TAIL')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 21, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


FILL = int(os.environ.get('FILL', '0'))
for span in (False, True):
    out = os.path.join(out_dir, f'cellnotes_{"span" if span else "plain"}_N{NOTE_REP}' + (f'_H{ROW_H}' if ROW_H != 1500 else '') + ('_B' if BODY_NOTE else '') + (f'_L{LINES}' if LINES != 1 else '') + '.hwpx')
    with zipfile.ZipFile(out, 'w') as z:
        put(z, 'mimetype', src.read('mimetype'), stored=True)
        for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
            put(z, n, src.read(n))
        put(z, 'Contents/header.xml', hdr.encode('utf8'))
        put(z, 'Contents/section0.xml', doc(FILL, span).encode('utf8'))
    print(out)
