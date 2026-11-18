#!/usr/bin/env python3
"""[#8575] 도형 그림 채우기(배치 모드)의 그림 크기 — 공개 합성 입력.

  python3 gen_shape_image_fill.py samples/issue2527_empty_linesegs.hwpx shape_image_fill.hwpx

그림: 240x160px PNG(pHYs 300dpi). 바깥 4px 검정 테두리, 네 사분면 빨강·초록·파랑·노랑,
가운데 검정 십자. 모든 사각형은 종이 기준(글 앞으로), 테두리 없음.
  사례 A  가운데로, 상자 320x240px  — 그림이 상자보다 작다.
  사례 B  가운데로, 상자 160x240px  — 가로만 넘친다.
  사례 C  가운데로, 상자 320x100px  — 세로만 넘친다.
  사례 D  가운데로, 상자 120x80px   — 두 축 모두 넘친다.
  사례 E  가운데 아래로, 상자 160x240px — 가로만 넘친다.
  사례 F  가운데로, 현재 상자 320x240px · 원래 크기 640x480px(scaMatrix 0.5) — 도형 축소.
  2쪽     글꼴 「한컴 윤고딕 250」 22pt 문단 「2021. 9.  . 윤고딕 250 한글」(모든 언어 슬롯).
1px = 75HU(96dpi). 모든 문단에 linesegarray 없음.
"""
import struct
import sys
import zipfile
import zlib

src = zipfile.ZipFile(sys.argv[1])
out = sys.argv[2]
PX = 75  # HU per 96dpi px


def png():
    w, h = 240, 160
    rows = []
    for y in range(h):
        row = bytearray([0])
        for x in range(w):
            if x < 4 or y < 4 or x >= w - 4 or y >= h - 4 or abs(x - w // 2) < 2 or abs(y - h // 2) < 2:
                c = (0, 0, 0)
            elif x < w // 2 and y < h // 2:
                c = (230, 40, 40)
            elif y < h // 2:
                c = (40, 170, 40)
            elif x < w // 2:
                c = (40, 70, 230)
            else:
                c = (240, 200, 0)
            row += bytes(c)
        rows.append(bytes(row))

    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))

    ppm = round(300 / 0.0254)
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
            + chunk(b'pHYs', struct.pack('>IIB', ppm, ppm, 1))
            + chunk(b'IDAT', zlib.compress(b''.join(rows), 9)) + chunk(b'IEND', b''))


sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]
_id = [800]


def nid():
    _id[0] += 1
    return _id[0]


def para(runs):
    return f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{runs}</hp:p>\n'


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def rect(mode, x, y, w, h, org=None):
    i = nid()
    cw, ch = w * PX, h * PX
    ow, oh = (org[0] * PX, org[1] * PX) if org else (cw, ch)
    sx, sy = cw / ow, ch / oh
    return (f'<hp:rect id="{i}" zOrder="{i}" numberingType="PICTURE" textWrap="IN_FRONT_OF_TEXT" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" href="" groupLevel="0" instid="{i}" ratio="0">'
            '<hp:offset x="0" y="0"/>'
            f'<hp:orgSz width="{ow}" height="{oh}"/><hp:curSz width="{cw}" height="{ch}"/>'
            '<hp:flip horizontal="0" vertical="0"/>'
            f'<hp:rotationInfo angle="0" centerX="{cw // 2}" centerY="{ch // 2}" rotateimage="1"/>'
            '<hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
            f'<hc:scaMatrix e1="{sx:g}" e2="0" e3="0" e4="0" e5="{sy:g}" e6="0"/>'
            '<hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo>'
            '<hp:lineShape color="#000000" width="0" style="NONE" endCap="FLAT" headStyle="NORMAL" '
            'tailStyle="NORMAL" headfill="1" tailfill="1" headSz="SMALL_SMALL" tailSz="SMALL_SMALL" '
            'outlineStyle="NORMAL" alpha="0"/>'
            f'<hc:fillBrush><hc:imgBrush mode="{mode}"><hc:img binaryItemIDRef="image1" bright="0" '
            'contrast="0" effect="REAL_PIC" alpha="0"/></hc:imgBrush></hc:fillBrush>'
            '<hp:shadow type="NONE" color="#B2B2B2" offsetX="0" offsetY="0" alpha="0"/>'
            f'<hc:pt0 x="0" y="0"/><hc:pt1 x="{ow}" y="0"/><hc:pt2 x="{ow}" y="{oh}"/><hc:pt3 x="0" y="{oh}"/>'
            f'<hp:sz width="{cw}" widthRelTo="ABSOLUTE" height="{ch}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="0" allowOverlap="1" holdAnchorAndSO="0" '
            'vertRelTo="PAPER" horzRelTo="PAPER" vertAlign="TOP" horzAlign="LEFT" '
            f'vertOffset="{y * PX}" horzOffset="{x * PX}"/>'
            '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            '<hp:shapeComment>사각형입니다.</hp:shapeComment></hp:rect>')


CASES = [
    ('CENTER', 40, 60, 320, 240, None),
    ('CENTER', 420, 60, 160, 240, None),
    ('CENTER', 40, 340, 320, 100, None),
    ('CENTER', 420, 340, 120, 80, None),
    ('CENTER_BOTTOM', 40, 480, 160, 240, None),
    ('CENTER', 420, 480, 320, 240, (640, 480)),
]
shapes = ''.join(rect(*c) for c in CASES)
body = head + '</hp:run>' + t('#8575 도형 그림 채우기 크기 합성 입력') + '</hp:p>\n'
body += para(f'<hp:run charPrIDRef="0">{shapes}<hp:t>앵커</hp:t></hp:run>')
body += (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="1" columnBreak="0" merged="0">'
         '<hp:run charPrIDRef="1"><hp:t>2021. 9.  . 윤고딕 250 한글</hp:t></hp:run></hp:p>\n')
sec_new = body + '</hs:sec>'

LANGS = ('HANGUL', 'LATIN', 'HANJA', 'JAPANESE', 'OTHER', 'SYMBOL', 'USER')
hdr = src.read('Contents/header.xml').decode('utf8')
faces = ''.join(f'<hh:fontface lang="{lang}" fontCnt="2"><hh:font id="0" face="함초롬바탕" type="TTF" isEmbedded="0"/>'
                '<hh:font id="1" face="한컴 윤고딕 250" type="TTF" isEmbedded="0"/></hh:fontface>' for lang in LANGS)
hdr = hdr.replace(hdr[hdr.find('<hh:fontfaces'):hdr.find('</hh:fontfaces>')],
                  f'<hh:fontfaces itemCnt="7">{faces}')
char0 = hdr[hdr.find('<hh:charPr id="0"'):hdr.find('</hh:charPr>') + len('</hh:charPr>')]
char1 = (char0.replace('id="0"', 'id="1"', 1).replace('height="1000"', 'height="2200"')
         .replace('<hh:fontRef hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/>',
                  '<hh:fontRef hangul="1" latin="1" hanja="1" japanese="1" other="1" symbol="1" user="1"/>'))
assert char1 != char0 and 'hangul="1"' in char1
hdr = hdr.replace('<hh:charProperties itemCnt="1">' + char0, '<hh:charProperties itemCnt="2">' + char0 + char1)

hpf = src.read('Contents/content.hpf').decode('utf8').replace(
    '<opf:manifest>\n',
    '<opf:manifest>\n<opf:item id="image1" href="BinData/image1.png" media-type="image/png" isEmbeded="1"/>\n')


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 11, 4, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/content.hpf', hpf.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
    put(z, 'BinData/image1.png', png())
