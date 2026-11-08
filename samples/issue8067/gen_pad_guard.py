#!/usr/bin/env python3
"""칸 위·아래 안 여백 합이 선언 칸 높이보다 클 때 (aim=1 칸 여백 / aim=0 표 여백) 한/글이 여백을 줄이는가 — 합성 입력.

  python gen_pad_guard.py <template.hwpx> <out_dir>

변형 {tac|flow}_L{n}: 1x1 표, 칸 선언 높이 1000HU, 칸 위·아래 안 여백 1500HU(hasMargin=1).
변형 flow_L{n}_tablepad: 칸 선언 높이 566HU, hasMargin=0, 표 안 여백 위·아래 368HU.
칸 안 한 줄 문단 n 개. 표 앞뒤에 표지 문단. 저장 줄 없음(한/글이 조판).
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
_id = [900]


def nid():
    _id[0] += 1
    return _id[0]


def p(runs):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{runs}</hp:p>')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def table(lines, tac, aim):
    w = 30000
    h, pad, tpad = (1000, 1500, 141) if aim else (566, 141, 368)
    inner = ''.join(p(t(f'칸 줄 L{i}')) for i in range(lines))
    cell = (f'<hp:tc name="" header="0" hasMargin="{1 if aim else 0}" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{inner}</hp:subList>'
            f'<hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/>'
            f'<hp:cellMargin left="510" right="510" top="{pad}" bottom="{pad}"/></hp:tc>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="1" colCnt="1" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w}" widthRelTo="ABSOLUTE" height="{h}" heightRelTo="ABSOLUTE" protect="0"/>'
            f'<hp:pos treatAsChar="{1 if tac else 0}" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            f'<hp:inMargin left="510" right="510" top="{tpad}" bottom="{tpad}"/>'
            f'<hp:tr>{cell}</hp:tr></hp:tbl>')


def doc(lines, tac, aim):
    body = [head + '</hp:run>' + t(f'pad-guard tac={int(tac)} lines={lines}') + '</hp:p>\n']
    body.append(p(t('표 앞 BEFORE')) + '\n')
    body.append(p('<hp:run charPrIDRef="0">' + table(lines, tac, aim) + '<hp:t/></hp:run>') + '\n')
    body.append(p(t('표 뒤 AFTER')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 19, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


VARIANTS = [(tac, lines, True) for tac in (False, True) for lines in (1, 3)] + [
    (False, 1, False), (False, 3, False)]
for tac, lines, aim in VARIANTS:
    if True:
        out = os.path.join(out_dir, f'{"tac" if tac else "flow"}_L{lines}{"" if aim else "_tablepad"}.hwpx')
        with zipfile.ZipFile(out, 'w') as z:
            put(z, 'mimetype', src.read('mimetype'), stored=True)
            for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
                put(z, n, src.read(n))
            put(z, 'Contents/header.xml', hdr.encode('utf8'))
            put(z, 'Contents/section0.xml', doc(lines, tac, aim).encode('utf8'))
        print(out)
