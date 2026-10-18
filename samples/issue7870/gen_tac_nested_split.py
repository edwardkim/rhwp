#!/usr/bin/env python3
"""[#7870] 셀 안 중첩 표가 쪽 경계에 걸릴 때의 분할 — 합성 입력(저장 줄 없음, 한/글이 직접 조판).

  ONLY=tac,flow FILLS=37 python3 gen_tac_nested_split.py samples/issue2527_empty_linesegs.hwpx samples/issue7870

변형(이름_채움문단수.hwpx):
  tac      중첩 표 treatAsChar=1, pageBreak=CELL
  flow     중첩 표 treatAsChar=0(자리차지), pageBreak=CELL   — 대조군
  tacnone  중첩 표 treatAsChar=1, pageBreak=NONE
  tacbig   treatAsChar=1, pageBreak=CELL, 행 30개(줄이 쪽보다 크다)
  tac1x1 / flow1x1  1×1 중첩 표(문단 8개) treatAsChar=1 / 0
바깥 표는 1×1, 비TAC 자리차지, pageBreak=CELL. 칸 문단: 앞 글줄 → 중첩 표 문단 → 뒤 글줄.
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

_id = [700]


def nid():
    _id[0] += 1
    return _id[0]


def p(runs, extra=''):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{runs}{extra}</hp:p>')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell(c, r, w, h, inner, margin=(510, 510, 141, 141)):
    l, rr, tp, b = margin
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{inner}</hp:subList>'
            f'<hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="{l}" right="{rr}" top="{tp}" bottom="{b}"/></hp:tc>')


def tbl(rows, cols, w_cell, h_row, cells_xml, tac, page_break):
    tac_attr = '1' if tac else '0'
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="{page_break}" repeatHeader="0" rowCnt="{rows}" colCnt="{cols}" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w_cell * cols}" widthRelTo="ABSOLUTE" height="{h_row * rows}" heightRelTo="ABSOLUTE" protect="0"/>'
            f'<hp:pos treatAsChar="{tac_attr}" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>' + cells_xml + '</hp:tbl>')


def nested(tac, page_break, rows, label):
    cols, w, h = 4, 9000, 3000
    trs = ''.join('<hp:tr>' + ''.join(cell(c, r, w, h, p(t(f'{label}R{r}C{c}'))) for c in range(cols)) + '</hp:tr>'
                  for r in range(rows))
    return tbl(rows, cols, w, h, trs, tac, page_break)


def nested_single(tac, page_break, paras, label):
    w = 36000
    inner = ''.join(p(t(f'{label}R{i}C0 단일 칸 문단')) for i in range(paras))
    return tbl(1, 1, w, 3000, '<hp:tr>' + cell(0, 0, w, 3000, inner) + '</hp:tr>', tac, page_break)


def outer(inner_para):
    w = 42000
    paras = p(t('칸 앞 글줄 BEFORE')) + inner_para + p(t('칸 뒤 글줄 AFTER'))
    return tbl(1, 1, w, 3000, '<hp:tr>' + cell(0, 0, w, 3000, paras) + '</hp:tr>', False, 'CELL')


def doc(fill, tac, page_break, rows, label):
    body = [head + '</hp:run>' + t(f'#7870 {label} fill={fill}') + '</hp:p>\n']
    for i in range(fill):
        body.append(p(t(f'채움 문단 {i + 1:02d}')) + '\n')
    table = (nested_single(tac, page_break, -rows, label) if rows < 0
             else nested(tac, page_break, rows, label))
    inner_para = p('<hp:run charPrIDRef="0">' + table + '<hp:t/></hp:run>')
    body.append(p('<hp:run charPrIDRef="0">' + outer(inner_para) + '<hp:t/></hp:run>') + '\n')
    body.append(p(t('표 뒤 문단 TAIL')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 18, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


VARIANTS = {
    'tac': (True, 'CELL', 4),
    'flow': (False, 'CELL', 4),
    'tacnone': (True, 'NONE', 4),
    'tacbig': (True, 'CELL', 30),
    'tac1x1': (True, 'CELL', -8),
    'flow1x1': (False, 'CELL', -8),
}
FILLS = [int(x) for x in os.environ.get('FILLS', '36,38,40').split(',')]
ONLY = os.environ.get('ONLY')
for name, (tac, pb, rows) in VARIANTS.items():
    if ONLY and name not in ONLY.split(','):
        continue
    for fill in FILLS:
        _id[0] = 700  # 변형마다 같은 id 로 시작해 출력이 생성 순서와 무관하다.
        out = os.path.join(out_dir, f'{name}_{fill}.hwpx')
        with zipfile.ZipFile(out, 'w') as z:
            put(z, 'mimetype', src.read('mimetype'), stored=True)
            for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
                put(z, n, src.read(n))
            put(z, 'Contents/header.xml', hdr.encode('utf8'))
            put(z, 'Contents/section0.xml', doc(fill, tac, pb, rows, name).encode('utf8'))
        print(out)
