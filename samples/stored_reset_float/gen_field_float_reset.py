#!/usr/bin/env python3
"""저장 쪽 경계(vpos=0) 앞에 필드, 뒤에 용지 기준·글 앞으로 그리기 개체가 있는 문단 — 합성 입력.

  python gen_field_float_reset.py samples/issue2527_empty_linesegs.hwpx <out_dir>

field_float_O{OVERLAP}_F{FILL}….hwpx: 자리차지(위아래) 사각형 A(높이 A_H, 기준 A_REL·세로 A_Y, A_IN0=1 이면
구역 정의를 가진 빈 첫 문단 소유), 채움 문단 FILL 개, 문단 P 는 하이퍼링크 필드로 시작하고 끝 근처에
용지 기준·글 앞으로 사각형 B(겹침 허용 OVERLAP)를 둔다. PLAIN=1 은 필드·B 없는 대조군.
저장 줄 없음 — 한/글 2020 이 열어 HWP 로 저장한 결과(저장 줄·쪽 경계)를 입력으로 쓴다.

저장소 입력(한/글 2020 HWP 저장 · Print PDF):
  A_IN0=1 A_REL=PAPER A_Y=8000 FILL=20          → field_float_reset.hwp
  A_IN0=1 A_REL=PAPER A_Y=8000 FILL=20 PLAIN=1  → field_float_reset_plain.hwp
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
FILL = int(os.environ.get('FILL', '6'))
A_H = int(os.environ.get('A_H', '30000'))
A_REL = os.environ.get('A_REL', 'PARA')
A_Y = int(os.environ.get('A_Y', '0'))
PLAIN = os.environ.get('PLAIN', '0') == '1'
A_IN0 = os.environ.get('A_IN0', '0') == '1'
n = [900]
BACKSLASH = chr(92)  # 한/글 하이퍼링크 명령은 ':' 앞에 역슬래시를 둔다


def nid():
    n[0] += 1
    return n[0]


def p(runs):
    return f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{runs}</hp:p>'


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def rect(name, wrap, vert_rel, y, height, overlap):
    i = nid()
    return (f'<hp:rect id="{i}" zOrder="{i % 100}" numberingType="PICTURE" textWrap="{wrap}" textFlow="BOTH_SIDES" lock="0" '
            f'dropcapstyle="None" href="" groupLevel="0" instid="{i}" ratio="0"><hp:offset x="0" y="0"/>'
            f'<hp:orgSz width="30000" height="{height}"/><hp:curSz width="30000" height="{height}"/>'
            '<hp:flip horizontal="0" vertical="0"/>'
            f'<hp:rotationInfo angle="0" centerX="15000" centerY="{height // 2}" rotateimage="1"/>'
            '<hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
            '<hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
            '<hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo>'
            '<hp:lineShape color="#000000" width="33" style="SOLID" endCap="FLAT" headStyle="NORMAL" tailStyle="NORMAL" '
            'headfill="1" tailfill="1" headSz="MEDIUM_MEDIUM" tailSz="MEDIUM_MEDIUM" outlineStyle="NORMAL" alpha="0"/>'
            '<hc:fillBrush><hc:winBrush faceColor="#DDDDDD" hatchColor="#000000" alpha="0"/></hc:fillBrush>'
            '<hp:shadow type="NONE" color="#B2B2B2" offsetX="0" offsetY="0" alpha="0"/>'
            f'<hc:pt0 x="0" y="0"/><hc:pt1 x="30000" y="0"/><hc:pt2 x="30000" y="{height}"/><hc:pt3 x="0" y="{height}"/>'
            f'<hp:sz width="30000" widthRelTo="ABSOLUTE" height="{height}" heightRelTo="ABSOLUTE" protect="0"/>'
            f'<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="{overlap}" holdAnchorAndSO="0" '
            f'vertRelTo="{vert_rel}" horzRelTo="PAPER" vertAlign="TOP" horzAlign="LEFT" vertOffset="{y}" horzOffset="8000"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            f'<hp:shapeComment>{name}</hp:shapeComment></hp:rect>')


def doc(overlap):
    if A_IN0:
        # 구역 정의를 가진 빈 첫 문단이 자리차지 사각형 A 를 소유한다.
        body = [head + rect('A', 'TOP_AND_BOTTOM', A_REL, A_Y, A_H, 0) + '</hp:run></hp:p>\n']
        body.append(p(t(f'field-float overlap={overlap} FILL={FILL}')) + '\n')
    else:
        body = [head + '</hp:run>' + t(f'field-float overlap={overlap} FILL={FILL}') + '</hp:p>\n']
        body.append(p('<hp:run charPrIDRef="0">' + rect('A', 'TOP_AND_BOTTOM', A_REL, A_Y, A_H, 0) + '<hp:t>자리차지 사각형 A 의 기준 문단</hp:t></hp:run>') + '\n')
    for i in range(FILL):
        body.append(p(t(f'채움 문단 {i + 1:02d} — 본문을 아래로 민다.')) + '\n')
    words = ' '.join(f'낱말{k:02d}' for k in range(1, 61))
    tail = ' '.join(f'뒤낱말{k:02d}' for k in range(1, 9))
    field = ('<hp:ctrl><hp:fieldBegin id="7001" type="HYPERLINK" name="" editable="0" dirty="0" zorder="-1" fieldid="7001">'
             '<hp:parameters cnt="1" name=""><hp:stringParam name="Command">http' + BACKSLASH + '://example.com;1;0;0;</hp:stringParam>'
             '</hp:parameters></hp:fieldBegin></hp:ctrl>')
    if PLAIN:
        runs = f'<hp:run charPrIDRef="0"><hp:t>참조 문단 P {words} {tail}</hp:t></hp:run>'
    else:
        runs = ('<hp:run charPrIDRef="0">' + field + '<hp:t>참조</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="7001" fieldid="7001"/></hp:ctrl>'
                f'<hp:t> 문단 P {words} </hp:t>' + rect('B', 'IN_FRONT_OF_TEXT', 'PAPER', 20000, 30000, overlap)
                + f'<hp:t>{tail}</hp:t></hp:run>')
    body.append(p(runs) + '\n')
    body.append(p(t('P 뒤 문단')) + '\n')
    return ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 21, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


for overlap in (0, 1):
    out = os.path.join(out_dir, f'field_float_O{overlap}_F{FILL}' + (f'_A{A_H}' if A_H != 30000 else '') + (f'_{A_REL.lower()}{A_Y}' if A_REL != 'PARA' else '') + ('_p0' if A_IN0 else '') + ('_plain' if PLAIN else '') + '.hwpx')
    with zipfile.ZipFile(out, 'w') as z:
        put(z, 'mimetype', src.read('mimetype'), stored=True)
        for name in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
            put(z, name, src.read(name))
        put(z, 'Contents/header.xml', hdr.encode('utf8'))
        put(z, 'Contents/section0.xml', doc(overlap).encode('utf8'))
    print(out)
