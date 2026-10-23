#!/usr/bin/env python3
"""[#7993] 한/글 표기 `<hp:hypen/>` 소프트 하이픈 — 공개 합성 입력.

  python3 gen_soft_hyphen.py samples/issue2527_empty_linesegs.hwpx soft-hyphen.hwpx

모든 문단에 linesegarray 없음(한/글이 저장할 때 줄을 만든다). 긴 합성어 안에 `<hp:hypen/>`
을 넣은 문단을 길이를 조금씩 바꿔 여러 개 둔다 — 한/글이 일부는 하이픈 자리에서 줄을 끊고
(줄 끝 `-`), 일부는 줄 중간에 남긴다(보이지 않음). 마지막 문단은 리터럴 U+00AD 대조군이다.
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


HY = '<hp:hypen/>'
WORDS = [
    f'Bundes{HY}verfassungs{HY}gerichts',
    f'Verwaltungs{HY}gerichts{HY}ordnung',
    f'Rastede-Entscheidung',
    f'Gesetz{HY}gebungs{HY}kompetenz',
    f'Selbst{HY}verwaltungs{HY}garantie',
]


def para(inner):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0"><hp:t>{inner}</hp:t></hp:run></hp:p>\n')


body = [head + '</hp:run><hp:run charPrIDRef="0"><hp:t>#7993 소프트 하이픈(hp:hypen) 합성 입력</hp:t></hp:run></hp:p>\n']
for k in range(12):
    lead = '가' * k
    words = ' '.join(WORDS[(k + i) % len(WORDS)] for i in range(9))
    body.append(para(f'{lead} 사례 {k + 1}: {words} 끝.'))
# 대조군: 리터럴 U+00AD 는 한/글이 보이는 하이픈으로 그린다.
body.append(para('대조군 리터럴: K­MOOC 와 Bundes­verfassungs­gericht'))
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
