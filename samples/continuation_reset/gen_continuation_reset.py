#!/usr/bin/env python3
"""한 문단이 세 쪽에 걸치고 둘째 쪽 줄에 자기 각주가 있을 때 둘째 쪽 끝 — 합성 입력.

  python gen_continuation_reset.py samples/issue2527_empty_linesegs.hwpx <out_dir>

cont_reset_F{FILL}_L{LINES}_N{NOTES}.hwpx: 한 줄 채움 문단 FILL 개 뒤에 LINES 줄쯤 되는 긴 문단 하나.
그 문단의 가운데(둘째 쪽에 놓일 자리)에 각주 NOTES 개(본문 REP 회 반복). 저장 줄 없음 —
한/글이 다시 저장한 HWP(저장 줄·쪽 reset 포함)와 Print PDF 가 기준이다.
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
FILL = int(os.environ.get('FILL', '20'))
LINES = int(os.environ.get('LINES', '70'))
NOTES = int(os.environ.get('NOTES', '4'))
REP = int(os.environ.get('REP', '6'))
SENT = '긴 문단의 본문 문장을 여러 줄로 이어 쓴다 '  # 한 줄에 약 두 번


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
    text = f'각주 {number} 본문을 길게 채운다 ' * REP
    sub = (f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
           f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
           f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
           f'<hp:run charPrIDRef="0"><hp:ctrl><hp:autoNum num="{number}" numType="FOOTNOTE">'
           f'<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>'
           f'</hp:autoNum></hp:ctrl><hp:t> {text}</hp:t></hp:run></hp:p></hp:subList>')
    return f'<hp:ctrl><hp:footNote number="{number}" suffixChar="41" instId="{nid()}">{sub}</hp:footNote></hp:ctrl>'


def doc():
    nid = Ids()
    body = [head + '</hp:run>' + t(f'cont-reset F={FILL} L={LINES} N={NOTES}') + '</hp:p>\n']
    for i in range(FILL):
        body.append(p(nid, t(f'채움 {i + 1:02d}')) + '\n')
    # 긴 문단: 문장 2*LINES 개. 각주는 문장 0.5*LINES .. 0.9*LINES 사이에 고르게.
    total = 2 * LINES
    marks = {int(total * (0.5 + 0.4 * k / max(NOTES, 1))): k + 1 for k in range(NOTES)}
    runs = ['<hp:run charPrIDRef="0">']
    for s in range(total):
        runs.append(f'<hp:t>{SENT}{s + 1:03d} </hp:t>')
        if s in marks:
            runs.append(footnote(nid, marks[s]))
    runs.append('<hp:t/></hp:run>')
    body.append(p(nid, ''.join(runs)) + '\n')
    body.append(p(nid, t('문단 뒤 TAIL')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 21, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


out = os.path.join(out_dir, f'cont_reset_F{FILL}_L{LINES}_N{NOTES}.hwpx')
with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for name in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, name, src.read(name))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', doc().encode('utf8'))
print(out)
