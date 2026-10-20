#!/usr/bin/env python3
"""쪽 끝에 걸린 쪽 나눔 표 행의 선언 높이 처리 — 합성 입력.

  python gen_row_trim.py samples/issue2527_empty_linesegs.hwpx <out_dir>

trim_{N}_F{F}.hwpx: 채움 문단 F 개 뒤에 2열 ROWS 행 표(pageBreak=CELL). 행마다 선언 높이
ROW_H(HU, 기본 6000)이고 내용은 LINES 줄(기본 1)이다. F 를 바꿔 쪽 끝 행의 선언 하단이 쪽 바닥을
넘는 양을 바꾼다. NOTE=1 이면 첫 채움 문단에 각주 하나(본문 위 쪽 각주)를 단다.
저장 줄 없음(한/글이 조판).
"""
import os
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
out_dir = sys.argv[2]
os.makedirs(out_dir, exist_ok=True)
hdr = src.read('Contents/header.xml').decode('utf8')
# BIG=1 이면 기본 글자 모양(id 0)의 크기를 두 배로 해 줄 높이를 2000HU 로 만든다.
if os.environ.get('BIG', '0') == '1':
    hdr = hdr.replace('<hh:charPr id="0" height="1000"', '<hh:charPr id="0" height="2000"', 1)
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]
ROWS = int(os.environ.get('ROWS', '14'))
ROW_H = int(os.environ.get('ROW_H', '6000'))
NOTE = os.environ.get('NOTE', '0') == '1'
LINES = int(os.environ.get('LINES', '1'))
PAD = int(os.environ.get('PAD', '141'))
NOTE_TEXT = '쪽 각주 본문 ' * int(os.environ.get('NOTE_REP', '6'))


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


def cell(nid, c, r, w, inner):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{inner}</hp:subList>'
            f'<hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{ROW_H}"/>'
            f'<hp:cellMargin left="510" right="510" top="{PAD}" bottom="{PAD}"/></hp:tc>')


def table(nid):
    w0, w1 = 9000, 33000
    rows = []
    for r in range(ROWS):
        cells = cell(nid, 0, r, w0, p(nid, t(f'라벨 {r:02d}')))
        cells += cell(nid, 1, r, w1, ''.join(p(nid, t(f'행 R{r:02d} 내용 {i + 1}')) for i in range(LINES)))
        rows.append(f'<hp:tr>{cells}</hp:tr>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{ROWS}" colCnt="2" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w0 + w1}" widthRelTo="ABSOLUTE" height="{ROW_H * ROWS}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            f'<hp:inMargin left="510" right="510" top="{PAD}" bottom="{PAD}"/>' + ''.join(rows) + '</hp:tbl>')


def doc(fill):
    nid = Ids()
    body = [head + '</hp:run>' + t(f'row-trim ROW_H={ROW_H} F={fill} NOTE={int(NOTE)}') + '</hp:p>\n']
    for i in range(fill):
        if NOTE and i == 0:
            body.append(p(nid, f'<hp:run charPrIDRef="0"><hp:t>채움 01 각주</hp:t>{footnote(nid, 1)}<hp:t/></hp:run>') + '\n')
        else:
            body.append(p(nid, t(f'채움 {i + 1:02d}')) + '\n')
    body.append(p(nid, '<hp:run charPrIDRef="0">' + table(nid) + '<hp:t/></hp:run>') + '\n')
    body.append(p(nid, t('표 뒤 TAIL')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 21, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


for fill in (int(x) for x in os.environ.get('FILLS', '0,1,2,3,4').split(',')):
    big = '_big' if os.environ.get('BIG', '0') == '1' else ''
    out = os.path.join(out_dir, f'trim_n{int(NOTE)}_l{LINES}{big}_p{PAD}_h{ROW_H}_F{fill:02d}.hwpx')
    with zipfile.ZipFile(out, 'w') as z:
        put(z, 'mimetype', src.read('mimetype'), stored=True)
        for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
            put(z, n, src.read(n))
        put(z, 'Contents/header.xml', hdr.encode('utf8'))
        put(z, 'Contents/section0.xml', doc(fill).encode('utf8'))
    print(out)
