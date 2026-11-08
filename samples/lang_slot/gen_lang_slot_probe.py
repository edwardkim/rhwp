#!/usr/bin/env python3
"""한/글이 글자를 어느 언어 슬롯 글꼴로 재고 그리는가 — 합성 입력.

  python gen_lang_slot_probe.py samples/issue2527_empty_linesegs.hwpx <out.hwpx>

글자 모양 하나에 일곱 슬롯마다 서로 다른 설치 글꼴을 준다(한글 맑은 고딕 · 영문 Arial · 한자 바탕 ·
일본어 Times New Roman · 기타 돋움 · 기호 궁서 · 사용자 HY헤드라인M). 함초롬바탕은 어느 슬롯에도 두지 않아
한/글의 글리프 대체와 구별한다. 문단마다 `가<글자>가`(CONTEXTS 로 앞뒤 글자를 바꿈)를
적어 PDF 의 글꼴 이름으로 그 글자의 슬롯을 읽는다. CHARS 는 16진 코드 목록,
PATTERN 은 문단 글(`{c}`=앞뒤 글자, `{x}`=시험 글자), FACES 는 일곱 슬롯 글꼴(쉼표). 저장 줄 없음.

저장소 입력 symbol_slot_bullets.hwpx:
  FACES='맑은 고딕,Arial,바탕,Times New Roman,돋움,함초롬바탕,HY헤드라인M' CHARS=2981,27E8,2A00 \\
  PATTERN='{x}현장 검토본을 학교 현장의 수업 적용 과정에서의 어려움 모니터링' python gen_lang_slot_probe.py ...
"""
import os
import sys
import zipfile

CHARS = [
    0x2981, 0x2980, 0x2983, 0x29BE, 0x29FA,          # Misc Math Symbols-B
    0x27C0, 0x27E8, 0x27F5,                          # Misc Math-A, Supp Arrows-A
    0x2900, 0x2933,                                  # Supp Arrows-B
    0x2A00, 0x2A2F,                                  # Supp Math Operators
    0x2B05, 0x2B50,                                  # Misc Symbols and Arrows
    0x2022, 0x2219, 0x25CF, 0x2192, 0x2200, 0x2460,  # control: already symbol / punct
    0x2030, 0x203B, 0x20AC, 0x2103, 0x2116, 0x2153, 0x2160,
    0x00D7, 0x0391, 0x0410, 0x3200, 0x3300, 0xFF01, 0x3008,
]
FACES = ['맑은 고딕', 'Arial', '바탕', 'Times New Roman', '돋움', '궁서', 'HY헤드라인M']
LANGS = ['HANGUL', 'LATIN', 'HANJA', 'JAPANESE', 'OTHER', 'SYMBOL', 'USER']

if os.environ.get('FACES'):
    FACES = os.environ['FACES'].split(',')
if os.environ.get('CHARS'):
    CHARS = [int(x, 16) for x in os.environ['CHARS'].split(',')]
CONTEXTS = os.environ.get('CONTEXTS', '가').split(',')
PATTERN = os.environ.get('PATTERN', '{c}{x}{c}')

src = zipfile.ZipFile(sys.argv[1])
hdr = src.read('Contents/header.xml').decode('utf8')
sec = src.read('Contents/section0.xml').decode('utf8')
start = hdr.index('<hh:fontfaces')
end = hdr.index('</hh:fontfaces>') + len('</hh:fontfaces>')
faces = ''.join(
    f'<hh:fontface lang="{lang}" fontCnt="1"><hh:font id="0" face="{face}" type="TTF" isEmbedded="0"/></hh:fontface>'
    for lang, face in zip(LANGS, FACES))
hdr = hdr[:start] + f'<hh:fontfaces itemCnt="7">{faces}</hh:fontfaces>' + hdr[end:]
head = sec[:sec.find('<hp:bookmark')]
body = [head + '</hp:run><hp:run charPrIDRef="0"><hp:t>lang-slot probe</hp:t></hp:run></hp:p>\n']
for i, cp in enumerate(CHARS):
    text = ' '.join(PATTERN.format(c=c, x=chr(cp)) for c in CONTEXTS)
    body.append(f'<hp:p id="{900 + i}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                f'<hp:run charPrIDRef="0"><hp:t>{cp:04X} {text}</hp:t></hp:run></hp:p>\n')
xml = ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 21, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(sys.argv[2], 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', xml.encode('utf8'))
print(sys.argv[2])
