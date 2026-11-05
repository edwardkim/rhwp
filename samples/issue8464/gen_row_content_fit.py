#!/usr/bin/env python3
"""[#8464] 셀 단위로 나누는 표의 쪽 끝 행 수용 — 합성 입력(`samples/issue1937/gen_row_trim.py` 변형).

  python samples/issue8464/gen_row_content_fit.py samples/issue2527_empty_linesegs.hwpx <out_dir>

trim_{N}_l{LINES}[_f{FONT_H}][_d{ROW0_D}]_p{PAD}_h{ROW_H}_F{F}.hwpx: 2열 ROWS 행 표(pageBreak=CELL),
행마다 선언 높이 ROW_H(HU), 내용은 LINES 줄. FONT_H 는 기본 글자 모양 크기(HU, 기본 1000),
ROW0_D 는 첫 행만 선언 높이를 더해 쪽 끝 행의 남은 공간을 1HU 단위로 옮긴다. 저장 줄 없음.

samples/issue8464 의 입력(ROWS=60, FILLS=0):
  rowbreak_content_fit_h2505.hwpx            ROW_H=2505 — 남은 9.1px < 내용 17.1px, 행을 넘긴다
  rowbreak_content_fit_h2570.hwpx            ROW_H=2570 — 남은 20.0px, 남는 선언 14.3px 버림
  rowbreak_content_fit_h3180.hwpx            ROW_H=3180 — 남은 20.5px, 남는 선언 21.9px 다음 쪽 띠
  rowbreak_content_fit_f1200_h2660_d14.hwpx  FONT_H=1200 ROW_H=2660 ROW0_D=14 — 남은 = 내용 + 2HU
  rowbreak_content_fit_f1200_h2660_d16.hwpx  FONT_H=1200 ROW_H=2660 ROW0_D=16 — 남은 = 내용
"""
import os
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
out_dir = sys.argv[2]
os.makedirs(out_dir, exist_ok=True)
hdr = src.read('Contents/header.xml').decode('utf8')
# BIG=1 이면 기본 글자 모양(id 0)의 크기를 두 배로 해 줄 높이를 2000HU 로 만든다.
FONT_H = os.environ.get('FONT_H')
if FONT_H:
    hdr = hdr.replace('<hh:charPr id="0" height="1000"', f'<hh:charPr id="0" height="{FONT_H}"', 1)
if os.environ.get('BIG', '0') == '1':
    hdr = hdr.replace('<hh:charPr id="0" height="1000"', '<hh:charPr id="0" height="2000"', 1)
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]
ROWS = int(os.environ.get('ROWS', '14'))
ROW_H = int(os.environ.get('ROW_H', '6000'))
NOTE = os.environ.get('NOTE', '0') == '1'
LINES = int(os.environ.get('LINES', '1'))
ROW0_D = int(os.environ.get('ROW0_D', '0'))
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
            f'<hp:cellSz width="{w}" height="{ROW_H + (ROW0_D if r == 0 else 0)}"/>'
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
    big = ('_big' if os.environ.get('BIG', '0') == '1' else '') + (f'_f{FONT_H}' if FONT_H else '') + (f'_d{ROW0_D}' if ROW0_D else '')
    out = os.path.join(out_dir, f'trim_n{int(NOTE)}_l{LINES}{big}_p{PAD}_h{ROW_H}_F{fill:02d}.hwpx')
    with zipfile.ZipFile(out, 'w') as z:
        put(z, 'mimetype', src.read('mimetype'), stored=True)
        for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
            put(z, n, src.read(n))
        put(z, 'Contents/header.xml', hdr.encode('utf8'))
        put(z, 'Contents/section0.xml', doc(fill).encode('utf8'))
    print(out)
