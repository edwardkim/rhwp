#!/usr/bin/env python3
"""[#7972] hasMargin=0 칸의 보존 cellMargin(비활성)과 표 안 여백(실효) — 공개 합성 입력.

  python3 gen_inactive_cell_margin.py samples/issue2527_empty_linesegs.hwpx inactive-cell-margin.hwpx

한 쪽에 1×1 바깥 표 칸 안의 3×4 중첩 표 두 개. 모든 칸은 `hasMargin="0"` + 보존 cellMargin 위·아래 141HU(비활성 값),
선언 높이 200HU 인 **모순 칸**(저장 줄 900HU 보다 작다)이다. 0·2행은 글자, 1행은 빈 문단이다.
  표 1  표 안 여백(inMargin) 위·아래 0   → 한/글은 칸을 저장 줄 12.0px 로 그린다(#7872 〈보기〉 형상).
  표 2  표 안 여백(inMargin) 위·아래 141 → 한/글은 칸을 줄 + 141×2 로 키운다(156678235 #3386 형상).
hasMargin=0 칸은 표 안 여백을 쓰고 자기 cellMargin 은 쓰지 않는다.
"""
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
out = sys.argv[2]
hdr = src.read('Contents/header.xml').decode('utf8')
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]

_id = [500]


def nid():
    _id[0] += 1
    return _id[0]


def lineseg(width):
    return (f'<hp:linesegarray><hp:lineseg textpos="0" vertpos="0" vertsize="900" textheight="900" '
            f'baseline="765" spacing="540" horzpos="0" horzsize="{width}" flags="393216"/></hp:linesegarray>')


def cell_para(text, width):
    run = f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>' if text else '<hp:run charPrIDRef="0"/>'
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{run}{lineseg(width)}</hp:p>')


def cell(c, r, w, text):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + cell_para(text, w - 1020) +
            f'</hp:subList><hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="200"/><hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>')


def table(inner_tb, label):
    rows, cols, w = 3, 4, 11000
    texts = [[f'{label} 0행 {c}열' for c in range(cols)], ['' for _ in range(cols)],
             [f'{label} 끝 {c}' for c in range(cols)]]
    trs = ''.join('<hp:tr>' + ''.join(cell(c, r, w, texts[r][c]) for c in range(cols)) + '</hp:tr>'
                  for r in range(rows))
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{rows}" colCnt="{cols}" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w * cols}" widthRelTo="ABSOLUTE" height="{200 * rows}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            f'<hp:inMargin left="510" right="510" top="{inner_tb}" bottom="{inner_tb}"/>'
            + trs + '</hp:tbl>')


def para(runs):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{runs}</hp:p>\n')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


body = [head + '</hp:run>' + t('#7972 hasMargin=0 칸의 비활성 cellMargin') + '</hp:p>\n']
def outer(inner):
    """#7872 〈보기〉처럼 1×1 바깥 표 칸 안에 둔다(칸 안 중첩 표 경로)."""
    w = 46000
    para_inner = (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                  f'<hp:run charPrIDRef="0">{inner}<hp:t/></hp:run></hp:p>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="1" colCnt="1" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w}" widthRelTo="ABSOLUTE" height="3000" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>'
            '<hp:tr><hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
            'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + para_inner +
            '</hp:subList><hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="3000"/><hp:cellMargin left="510" right="510" top="141" bottom="141"/>'
            '</hp:tc></hp:tr></hp:tbl>')


body.append(para(t('표 1 — 표 안 여백 0')))
body.append(para('<hp:run charPrIDRef="0">' + outer(table(0, '표1')) + '<hp:t/></hp:run>'))
body.append(para(t('표 2 — 표 안 여백 141')))
body.append(para('<hp:run charPrIDRef="0">' + outer(table(141, '표2')) + '<hp:t/></hp:run>'))
body.append(para(t('표 뒤 문단.')))
sec_new = ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 18, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
