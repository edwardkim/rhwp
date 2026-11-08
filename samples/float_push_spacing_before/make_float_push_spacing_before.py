#!/usr/bin/env python3
"""공개 합성 입력 생성기 (#7330 float-sb 재현용).

형상: 문단 앞 간격(prev=1400 (HWP 2800HU))이 있는 host 문단이 문단(PARA) 기준 자리차지(TOP_AND_BOTTOM)
비-글자취급 표를 품고 같은 문단에 글이 있다. 표가 host 의 첫 글줄을 표 아래로 민다.
대조군: 같은 문단 모양의 표 없는 문단(앞 간격이 그대로 적용돼야 한다)과 앞 간격이 없는 host.
뼈대(secPr·colPr·header·패키지 파일)는 공개 샘플 samples/issue2527_empty_linesegs.hwpx 에서
가져오고 rhwp 출처 표식(META-INF/rhwp-hwp5-origin)은 넣지 않는다. 생성물에는 linesegarray 가 없다.
저장 LineSeg 가 있는 입력은 이 생성물을 한/글(hwp2024Convert MCP engine 2020)로 HWP 저장한 것이다.

사용: python3 samples/float_push_spacing_before/make_float_push_spacing_before.py <출력.hwpx>  (저장소 루트에서)
"""
import re
import sys
import zipfile

src = zipfile.ZipFile('samples/issue2527_empty_linesegs.hwpx')
out = sys.argv[1]
hdr = src.read('Contents/header.xml').decode('utf8')
pp0 = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', hdr, re.S).group(0)
pp1 = pp0.replace('id="0"', 'id="1"').replace('<hc:prev value="0"', '<hc:prev value="1400"')
hdr = hdr.replace('<hh:paraProperties itemCnt="1">' + pp0, '<hh:paraProperties itemCnt="2">' + pp0 + pp1)
assert 'itemCnt="2">' in hdr
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]  # 첫 문단의 secPr·colPr 까지


def para(pid, runs, pp=0):
    return (f'<hp:p id="{pid}" paraPrIDRef="{pp}" styleIDRef="0" pageBreak="0" '
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
    w = 48000 // 3
    trs = ''.join('<hp:tr>' + ''.join(cell(c, r, w, rows[r][c]) for c in range(3)) + '</hp:tr>'
                  for r in range(len(rows)))
    return (f'<hp:tbl id="{tid}" zOrder="{tid}" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{len(rows)}" colCnt="3" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w * 3}" widthRelTo="ABSOLUTE" height="{2000 * len(rows)}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="140" right="140" top="140" bottom="852"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>'
            + trs + '</hp:tbl>')


rows = [['구분', '금액', '비고'], ['가', '1,000', '-'], ['나', '2,000', '-']]
first = head + '</hp:run>' + t('문단 앞 간격과 자리차지 표 합성 입력 (#7330)') + '</hp:p>\n'
body = para(10, t('1. 표 앞 문단이다.'))
host1 = para(20, '<hp:run charPrIDRef="0">' + table(301, rows) + '</hp:run>'
             + t('2. 앞 간격이 있는 host 문단의 첫 글줄 — 표 아래로 밀린다.'), pp=1)
mid = [para(30, t('3. host 뒤 첫째 문단이다.')),
       para(31, t('4. 앞 간격이 있는 표 없는 대조 문단 — 앞 간격이 그대로 붙는다.'), pp=1),
       para(32, t('5. 대조 문단 뒤 문단이다.'))]
host2 = para(40, '<hp:run charPrIDRef="0">' + table(302, rows) + '</hp:run>'
             + t('6. 앞 간격 없는 host 문단의 첫 글줄 — 대조군.'))
tail = [para(50 + i, t(s)) for i, s in enumerate(['7. host 뒤 첫째 문단이다.', '8. 끝.'])]
sec_new = first + body + host1 + ''.join(mid) + host2 + ''.join(tail) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 7, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
