#!/usr/bin/env python3
"""쪽 아래쪽 줄이 긴 각주를 참조할 때 같은 문단 뒤 줄의 쪽 소속 — 합성 입력(#8468).

  python gen_line_reserve.py samples/issue2527_empty_linesegs.hwpx <out_dir>

line_reserve_F{FILL}_L{LINES}_A{AT}_R{REP}.hwpx: 한 줄 채움 문단 FILL 개 뒤에 LINES 줄쯤 되는 긴 문단 하나.
그 문단의 AT 번째 문장 뒤(대략 AT/2 번째 줄)에 각주 1개(본문 REP 회 반복). 저장 줄 없음 —
한/글이 다시 저장한 HWP(저장 줄·쪽 경계 포함)와 Print PDF 가 기준이다.
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
LINES = int(os.environ.get('LINES', '30'))
AT = int(os.environ.get('AT', '20'))
REP = int(os.environ.get('REP', '12'))
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
    body = [head + '</hp:run>' + t(f'line-reserve F={FILL} L={LINES} A={AT} R={REP}') + '</hp:p>\n']
    for i in range(FILL):
        body.append(p(nid, t(f'채움 {i + 1:02d}')) + '\n')
    runs = ['<hp:run charPrIDRef="0">']
    for s in range(2 * LINES):
        runs.append(f'<hp:t>{SENT}{s + 1:03d} </hp:t>')
        if s == AT:
            runs.append(footnote(nid, 1))
    runs.append('<hp:t/></hp:run>')
    body.append(p(nid, ''.join(runs)) + '\n')
    body.append(p(nid, t('문단 뒤 TAIL')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 11, 2, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


out = os.path.join(out_dir, f'line_reserve_F{FILL}_L{LINES}_A{AT}_R{REP}.hwpx')
with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for name in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, name, src.read(name))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', doc().encode('utf8'))
print(out)
