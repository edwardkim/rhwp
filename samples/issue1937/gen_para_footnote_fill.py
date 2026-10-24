#!/usr/bin/env python3
"""쪽 각주가 있는 쪽을 한 줄 문단으로 채울 때 마지막 줄과 각주 영역 경계 — 합성 입력.

  python gen_para_footnote_fill.py samples/issue2527_empty_linesegs.hwpx <out_dir>

para_fn_N{N}_R{R}.hwpx: 첫 문단에 각주 하나(본문 R 회 반복), 그 뒤 한 줄 문단 N 개. 저장 줄 없음.
N 과 각주 길이 R 을 바꿔 1쪽 마지막 줄이 각주 구분선에 얼마나 붙는지 잰다.
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


def footnote(nid, number, rep):
    text = '쪽 각주 본문을 길게 채운다 ' * rep
    sub = (f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
           f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
           f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
           f'<hp:run charPrIDRef="0"><hp:ctrl><hp:autoNum num="{number}" numType="FOOTNOTE">'
           f'<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>'
           f'</hp:autoNum></hp:ctrl><hp:t> {text}</hp:t></hp:run></hp:p></hp:subList>')
    return f'<hp:ctrl><hp:footNote number="{number}" suffixChar="41" instId="{nid()}">{sub}</hp:footNote></hp:ctrl>'


def doc(n, rep):
    nid = Ids()
    body = [head + '</hp:run>' + t(f'para-fn N={n} R={rep}') + '</hp:p>\n']
    body.append(p(nid, f'<hp:run charPrIDRef="0"><hp:t>각주 앵커 문단</hp:t>{footnote(nid, 1, rep)}<hp:t/></hp:run>') + '\n')
    for i in range(n):
        body.append(p(nid, t(f'줄 L{i + 1:02d} 본문')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 21, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


for rep in (int(x) for x in os.environ.get('REPS', '2,6').split(',')):
    for n in (int(x) for x in os.environ.get('NS', '44').split(',')):
        out = os.path.join(out_dir, f'para_fn_N{n}_R{rep}.hwpx')
        with zipfile.ZipFile(out, 'w') as z:
            put(z, 'mimetype', src.read('mimetype'), stored=True)
            for name in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
                put(z, name, src.read(name))
            put(z, 'Contents/header.xml', hdr.encode('utf8'))
            put(z, 'Contents/section0.xml', doc(n, rep).encode('utf8'))
        print(out)
