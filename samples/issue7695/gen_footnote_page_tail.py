#!/usr/bin/env python3
"""[#7695] 각주가 있는 쪽 끝의 여러 줄 문단 — 공개 합성 입력.

  python3 gen_footnote_page_tail.py samples/issue2527_empty_linesegs.hwpx footnote-page-tail.hwpx

모든 문단에 linesegarray 없음(한/글이 저장할 때 줄을 만든다). 쪽마다 사례 하나:
첫 문단에 각주 하나, 앞 간격(0~1575HU, 225HU 간격)만 다른 간격 문단 하나, 한 줄 채움 문단
N개(37·38), 책갈피가 든 여러 줄 문단 하나. 앞 간격을 줄 높이보다 잘게 바꿔 여러 줄 문단의 꼬리가 각주 영역
바로 위(안전마진 안)·안·아래에 걸리게 한다. 한/글은 각주 영역 위에 들어가는 줄까지
그 쪽에 두고 나머지 줄을 다음 쪽 첫머리로 넘긴다.
"""
import re
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
out = sys.argv[2]
hdr = src.read('Contents/header.xml').decode('utf8')
sec = src.read('Contents/section0.xml').decode('utf8')
# 앞 간격만 다른 문단 모양 8개(0, 225, 450, ... HU = 0, 3, 6, ... px).
pp0 = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', hdr, re.S).group(0)
cnt = int(re.search(r'<hh:paraProperties itemCnt="(\d+)">', hdr).group(1))
STEPS = 8
extra = ''.join(pp0.replace('id="0"', f'id="{cnt + i}"').replace('<hc:prev value="0"', f'<hc:prev value="{225 * i}"')
                for i in range(STEPS))
hdr = hdr.replace(f'<hh:paraProperties itemCnt="{cnt}">' + pp0,
                  f'<hh:paraProperties itemCnt="{cnt + STEPS}">' + pp0 + extra)
PP_SB = cnt
head = sec[:sec.find('<hp:bookmark')]

_id = [500]


def nid():
    _id[0] += 1
    return _id[0]


def para(runs, page_break=False, pp=0):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{pp}" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{runs}</hp:p>\n')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def footnote(num, text):
    return (f'<hp:ctrl><hp:footNote number="{num}" suffixChar="41" instId="{nid()}">'
            '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
            'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            '<hp:run charPrIDRef="0"><hp:ctrl><hp:autoNum num="' + str(num) + '" numType="FOOTNOTE">'
            '<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>'
            f'</hp:autoNum></hp:ctrl><hp:t> {text}</hp:t></hp:run></hp:p>'
            '</hp:subList></hp:footNote></hp:ctrl>')


def bookmark(name):
    """책갈피(문단의 글자 아닌 컨트롤). 컨트롤이 든 문단은 각주 저장 리셋 전용 갈래 밖이다."""
    return f'<hp:ctrl><hp:bookmark name="{name}"/></hp:ctrl>'


LONG = ('사례 {k} 여러 줄 문단이다. 각주가 있는 쪽의 끝에서 이 문단의 줄들이 각주 영역 위에 들어가는지 '
        '한/글이 줄 단위로 판정하는지 확인한다. 들어가는 줄은 이 쪽에 남고 나머지 줄은 다음 쪽 첫머리로 '
        '넘어간다. 각주 영역 위에 남는 줄이 둘 이상이고 그 둘이 모두 각주 안전마진 안에 있는 자리를 '
        '찾으려고 문단을 여섯 줄 남짓으로 길게 쓴다. 줄 높이보다 잘게 바꾼 앞 간격이 그 자리를 만든다. '
        '이 문장은 줄 수를 채우기 위한 것이다. 끝.')

body = [head + '</hp:run>' + t('#7695 각주가 있는 쪽 끝의 여러 줄 문단 합성 입력') + '</hp:p>\n']
cases = [(n, step) for n in (37, 38) for step in range(STEPS)]
for k, (n, step) in enumerate(cases, start=1):
    first = (f'<hp:run charPrIDRef="0"><hp:t>사례 {k}: 채움 {n}줄·앞 간격 {225 * step}HU 뒤 여러 줄 문단</hp:t>'
             + footnote(k, f'사례 {k} 의 각주다.') + '</hp:run>')
    body.append(para(first, page_break=k > 1))
    body.append(para(t(f'사례 {k} 간격 문단'), pp=PP_SB + step))
    for i in range(1, n + 1):
        body.append(para(t(f'사례 {k} 채움 {i}')))
    text = LONG.format(k=k)
    cut = text.index('문단이다.') + len('문단이다.')
    body.append(para(t(text[:cut]) + '<hp:run charPrIDRef="0">' + bookmark(f'b{k}') + '</hp:run>'
                     + t(text[cut:])))
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
