#!/usr/bin/env python3
"""[#7696] 쪽·종이 기준 자리차지(TopAndBottom) 개체의 세로 밴드 — 공개 합성 입력.

  python3 gen_page_anchored_band.py samples/issue2527_empty_linesegs.hwpx page-anchored-band.hwpx

모든 문단에 linesegarray 없음(한/글이 저장할 때 줄을 만든다).
  사례 1  종이 기준 사각형(세로 오프셋 50000HU, 높이 34000HU — 본문 아래까지)을 앵커한 문단 뒤 «줄 1..40».
          밴드 안에서 시작하는 줄은 다음 쪽으로 간다.
  사례 2  쪽 기준 사각형(세로 오프셋 20000HU, 높이 15000HU — 본문 안에서 끝남) 뒤 «줄 1..40».
          밴드 안에서 시작하는 줄은 밴드 아래로 간다.
"""
import re
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


def para(runs, page_break=False):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{runs}</hp:p>\n')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def rect(vert_rel, v_off, width, height, label):
    i = nid()
    return (f'<hp:rect id="{i}" zOrder="1" numberingType="NONE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" href="" groupLevel="0" instid="{i}" ratio="0">'
            '<hp:offset x="0" y="0"/>'
            f'<hp:orgSz width="{width}" height="{height}"/><hp:curSz width="{width}" height="{height}"/>'
            '<hp:flip horizontal="0" vertical="0"/>'
            f'<hp:rotationInfo angle="0" centerX="{width // 2}" centerY="{height // 2}" rotateimage="1"/>'
            '<hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
            '<hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
            '<hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo>'
            '<hp:lineShape color="#000000" width="33" style="SOLID" endCap="FLAT" headStyle="NORMAL" '
            'tailStyle="NORMAL" headfill="1" tailfill="1" headSz="SMALL_SMALL" tailSz="SMALL_SMALL" '
            'outlineStyle="NORMAL" alpha="0"/>'
            '<hc:fillBrush><hc:winBrush faceColor="#D9D9D9" hatchColor="#000000" alpha="0"/></hc:fillBrush>'
            '<hp:shadow type="NONE" color="#B2B2B2" offsetX="0" offsetY="0" alpha="0"/>'
            f'<hp:drawText lastWidth="{width}" name="" editable="0">'
            '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + para(t(label)) +
            '</hp:subList><hp:textMargin left="283" right="283" top="283" bottom="283"/></hp:drawText>'
            f'<hc:pt0 x="0" y="0"/><hc:pt1 x="{width}" y="0"/><hc:pt2 x="{width}" y="{height}"/>'
            f'<hc:pt3 x="0" y="{height}"/>'
            f'<hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{height}" heightRelTo="ABSOLUTE" protect="0"/>'
            f'<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="0" allowOverlap="0" holdAnchorAndSO="0" '
            f'vertRelTo="{vert_rel}" horzRelTo="PAGE" vertAlign="TOP" horzAlign="LEFT" '
            f'vertOffset="{v_off}" horzOffset="0"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            '<hp:shapeComment>사각형입니다.</hp:shapeComment></hp:rect>')


def anchor(text, shape):
    return para(f'<hp:run charPrIDRef="0">{shape}<hp:t>{text}</hp:t></hp:run>')


body = [head + '</hp:run>' + t('#7696 쪽·종이 기준 자리차지 밴드 합성 입력') + '</hp:p>\n']
cases = [
    ('사례 1 종이 기준 밴드가 본문 아래까지', rect('PAPER', 50000, 42520, 34000, '종이 기준 자리차지 사각형')),
    ('사례 2 쪽 기준 밴드가 본문 안에서 끝남', rect('PAGE', 20000, 42520, 15000, '쪽 기준 자리차지 사각형')),
]
for i, (name, shape) in enumerate(cases):
    body.append(para(t(name), page_break=i > 0))
    body.append(anchor('앵커 문단', shape))
    for k in range(1, 41):
        body.append(para(t(f'사례 {i + 1} 줄 {k}')))
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
