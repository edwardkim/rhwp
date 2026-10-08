#!/usr/bin/env python3
"""공개 합성 입력 생성기 (#7548 3단계 재현용 — 21_언어 14쪽 형상의 linesegarray 없는 판).

형상: 본문이 있는 host 문단(두세 줄)에 문단 기준 어울림(SQUARE) 좁은 표 하나.
- 표: 3행x1열, treatAsChar=0, textWrap=SQUARE, vertRelTo=PARA vertOffset=0, horzRelTo=PARA LEFT,
  폭 2400HU, 높이가 host 본문보다 길어 다음 문단 첫 줄까지 띠가 내려온다.
- 한/글은 띠와 겹치는 줄(host 줄 + 다음 문단 앞 줄)을 표 오른쪽으로 좁히고, 띠 아래 줄은 전폭으로 쓴다.
뼈대는 공개 샘플 samples/issue2527_empty_linesegs.hwpx 에서 가져오고 rhwp 출처 표식은 넣지 않는다.
모든 문단에 linesegarray 가 없다(재조판 경로).

사용: python3 samples/para_square_lane/make_para_square_lane.py <출력.hwpx> [full_fontfaces]  (저장소 루트에서)
full_fontfaces: 뼈대 header 는 HANGUL fontface 만 선언한다. 한/글은 선언 없는 LATIN 글자를
  자체 기본 글꼴(Haansoft Batang)로 그리고 rhwp 는 HANGUL face 로 재므로 라틴 폭이 갈린다.
  이 인자는 7개 언어 모두 같은 face(함초롬바탕)를 선언해 그 변수를 없앤다.
"""
import sys
import zipfile

src = zipfile.ZipFile('samples/issue2527_empty_linesegs.hwpx')
out = sys.argv[1]
hdr = src.read('Contents/header.xml').decode('utf8')
if len(sys.argv) > 2:
    assert sys.argv[2] == 'full_fontfaces', sys.argv[2]
    one = '<hh:fontface lang="HANGUL" fontCnt="1"><hh:font id="0" face="함초롬바탕" type="TTF" isEmbedded="0"/></hh:fontface>'
    assert one in hdr
    langs = ['HANGUL', 'LATIN', 'HANJA', 'JAPANESE', 'OTHER', 'SYMBOL', 'USER']
    hdr = hdr.replace('<hh:fontfaces itemCnt="1">' + one,
                      '<hh:fontfaces itemCnt="7">' + ''.join(one.replace('HANGUL', l) for l in langs))
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]


def para(pid, runs):
    return (f'<hp:p id="{pid}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
            f'columnBreak="0" merged="0">{runs}</hp:p>\n')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell(r, w, h, text):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + para(1000 + r, t(text)) +
            f'</hp:subList><hp:cellAddr colAddr="0" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')


def table(tid):
    w, h = 2400, 2400
    trs = ''.join('<hp:tr>' + cell(r, w, h, s) + '</hp:tr>' for r, s in enumerate(['A', 'B', 'C']))
    return (f'<hp:tbl id="{tid}" zOrder="{tid}" numberingType="TABLE" textWrap="SQUARE" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="3" colCnt="1" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w}" widthRelTo="ABSOLUTE" height="{h * 3}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="0" right="850" top="0" bottom="0"/>'
            '<hp:inMargin left="141" right="141" top="141" bottom="141"/>'
            + trs + '</hp:tbl>')


first = head + '</hp:run>' + t('문단 기준 어울림 표 옆 차선 합성 입력 (#7548)') + '</hp:p>\n'
body = para(10, t('1. 표 앞 문단이다. 이 문단은 전폭으로 흐른다.'))
host_text = ('2. 좁은 어울림 표를 품은 host 문단이다. 이 문단의 글은 표 오른쪽 차선으로 좁혀 흐르며 '
             '여러 줄에 걸친다. 표가 이 문단보다 길어서 다음 문단 첫 줄도 표 옆에 놓인다.')
host = para(20, '<hp:run charPrIDRef="0">' + table(301) + '</hp:run>' + t(host_text))
nxt_text = ('3. host 다음 문단이다. 첫 줄은 표 띠 옆 차선에서 시작하고, 표 아래로 내려가면 '
            '전폭으로 돌아온다. 이 문장은 여러 줄이 되도록 충분히 길게 쓴다. 한/글은 띠와 겹치는 '
            '줄만 좁히고 띠 아래 줄은 본문 전폭을 쓴다.')
nxt = para(30, t(nxt_text))
tail = para(40, t('4. 끝.'))
sec_new = first + body + host + nxt + tail + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 4, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
