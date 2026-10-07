#!/usr/bin/env python3
"""공개 합성 입력 생성기 (#7548 2단계 재현용).

형상: 본문이 있는 host 문단에 쪽(PAGE) 기준 어울림(SQUARE) 표 하나.
- 표: 3행x4열, treatAsChar=0, textWrap=SQUARE, vertRelTo=PAGE vertOffset=13000,
  horzRelTo=PAGE horzAlign=CENTER, 폭 = 본문 폭(바깥 여백 포함) → 표 옆에 글이 들어갈 자리가 없다.
- host 뒤 문단들: 앞쪽은 표 위로 흐르고, 표 띠와 겹치는 문단은 한/글이 띠 아래로 넘긴다.
뼈대(secPr·colPr·header·패키지 파일)는 공개 샘플 samples/issue2527_empty_linesegs.hwpx 에서
가져오고 rhwp 출처 표식(META-INF/rhwp-hwp5-origin)은 넣지 않는다. 생성물에는 linesegarray 가 없다.
저장 LineSeg 가 있는 입력은 이 생성물을 한/글(hwp2024Convert MCP engine 2020)로 HWPX 저장한 것이다.

사용: python3 samples/page_anchored_square/make_page_anchored_square.py <출력.hwpx>  (저장소 루트에서)
"""
import re
import sys
import zipfile

src = zipfile.ZipFile('samples/issue2527_empty_linesegs.hwpx')
out = sys.argv[1]
hdr = src.read('Contents/header.xml').decode('utf8')
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]  # 첫 문단의 secPr·colPr 까지

BODY_W = 59528 - 5669 * 2  # 48190


def para(pid, runs):
    return (f'<hp:p id="{pid}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
            f'columnBreak="0" merged="0">{runs}</hp:p>\n')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell(c, r, w, text):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + para(1000 + r * 10 + c, t(text)) +
            f'</hp:subList><hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="2000"/><hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>')


def table(tid, rows):
    out_l = out_r = 140
    total = BODY_W - out_l - out_r
    w = total // 4
    width = w * 4
    trs = ''.join('<hp:tr>' + ''.join(cell(c, r, w, rows[r][c]) for c in range(4)) + '</hp:tr>'
                  for r in range(len(rows)))
    return (f'<hp:tbl id="{tid}" zOrder="{tid}" numberingType="TABLE" textWrap="SQUARE" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{len(rows)}" colCnt="4" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{2000 * len(rows)}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PAGE" horzRelTo="PAGE" vertAlign="TOP" horzAlign="CENTER" vertOffset="13000" horzOffset="0"/>'
            f'<hp:outMargin left="{out_l}" right="{out_r}" top="140" bottom="852"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>'
            + trs + '</hp:tbl>')


rows = [['구분', '수량', '금액', '비고'], ['가', '1', '1,000', '-'], ['나', '2', '2,000', '-']]
first = head + '</hp:run>' + t('쪽 기준 어울림 표 합성 입력 (#7548)') + '</hp:p>\n'
lines = [
    '1. 표 앞 첫째 문단이다.',
    '2. 표 앞 둘째 문단이다.',
]
body = ''.join(para(10 + i, t(s)) for i, s in enumerate(lines))
host = para(20, '<hp:run charPrIDRef="0">' + table(301, rows) + '</hp:run>'
            + t('3. 쪽 기준 어울림 표를 품은 host 문단이다.'))
after = [
    '4. host 뒤 첫째 문단이다.',
    '5. host 뒤 둘째 문단이다.',
    '6. host 뒤 셋째 문단이다.',
    '7. host 뒤 넷째 문단이다.',
    '8. host 뒤 다섯째 문단이다.',
    '9. host 뒤 여섯째 문단이다.',
    '10. host 뒤 일곱째 문단이다.',
    '11. host 뒤 여덟째 문단이다.',
    '12. host 뒤 아홉째 문단이다.',
    '13. 끝.',
]
tail = ''.join(para(30 + i, t(s)) for i, s in enumerate(after))
sec_new = first + body + host + tail + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 3, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
