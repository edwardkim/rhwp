#!/usr/bin/env python3
"""쪽 바닥 근처에서 시작하는 쪽 나눔 표의 뒤쪽 행에만 각주가 있을 때 — 합성 입력.

  python gen_fn_table.py samples/issue2527_empty_linesegs.hwpx samples/issue8081

span_{F}.hwpx: 채움 문단 F 개 뒤에 2열 ROWS 행 표(pageBreak=CELL). 왼쪽 열은 10행씩
병합(세로 정렬 SPAN_VALIGN, 기본 TOP), 오른쪽 열은 행마다 칸. 각주는 ROWS-10 ~ ROWS-5 행
오른쪽 칸에 하나씩(본문 NOTE_REP 회 반복). 저장 줄 없음(한/글이 조판).
기본값 ROWS=24 · NOTE_REP=12 · FILLS=36,38 이 저장소 입력이다.
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
ROWS = int(os.environ.get('ROWS', '24'))
NOTE_ROWS = tuple(range(ROWS - 10, ROWS - 4))
NOTE_TEXT = '각주 본문 줄을 길게 채운다 ' * int(os.environ.get('NOTE_REP', '12'))


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
            f'<hp:cellSz width="{w}" height="{1500 * row_span}"/>'
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
        cells += cell(nid, 1, r, w1, p(nid, runs))
        rows.append(f'<hp:tr>{cells}</hp:tr>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{ROWS}" colCnt="2" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w0 + w1}" widthRelTo="ABSOLUTE" height="{1500 * ROWS}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>' + ''.join(rows) + '</hp:tbl>')


def doc(fill, span):
    nid = Ids()
    body = [head + '</hp:run>' + t(f'fn-table span={int(span)} F={fill}') + '</hp:p>\n']
    for i in range(fill):
        body.append(p(nid, t(f'채움 {i + 1:02d}')) + '\n')
    body.append(p(nid, '<hp:run charPrIDRef="0">' + table(nid, span) + '<hp:t/></hp:run>') + '\n')
    body.append(p(nid, t('표 뒤 TAIL')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 19, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


for span in (True,):
    for fill in (int(x) for x in os.environ.get('FILLS', '36,38').split(',')):
        out = os.path.join(out_dir, f'{"span" if span else "plain"}_{fill}.hwpx')
        with zipfile.ZipFile(out, 'w') as z:
            put(z, 'mimetype', src.read('mimetype'), stored=True)
            for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
                put(z, n, src.read(n))
            put(z, 'Contents/header.xml', hdr.encode('utf8'))
            put(z, 'Contents/section0.xml', doc(fill, span).encode('utf8'))
        print(out)
