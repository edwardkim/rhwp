#!/usr/bin/env python3
"""공개 합성 입력 생성기 (#7330 float-sb 경계 재현용).

같은 문단의 문단(PARA) 기준 자리차지(TOP_AND_BOTTOM) 비-글자취급 표가 host 첫 글줄을 표 아래로
미는 형상의 **경계**를 쪽마다 하나씩 둔다. 기대 위치는 이 생성물을 한/글(hwp2024Convert MCP
engine 2020)로 HWP 저장한 저장 LineSeg 와 같은 엔진의 Print PDF 가 정한다.

- 1쪽 M: 앞 간격(4000HU)이 표 점유(한 행 1000HU, 바깥 여백 0)보다 큰 host — 글줄 원점이
  `문단 상단 + 앞 간격` 쪽으로 정해지는 경계. 대조: 같은 앞 간격의 표 없는 문단, 앞 간격 없는 host.
- 2쪽 T: 같은 형상의 host 가 쪽 나눔 뒤 쪽 맨 위에 있다(단 맨 위). T 뒤를 채움 문단으로
  채워, 측정이 첫 글줄 위치를 과소 계상하면 채움 문단의 쪽 소속이 달라지게 한다.
- 4쪽 L: 앞 간격(1400HU) host 의 표 뒤 글이 여러 줄이다 — 첫 글줄만 앞 간격 경계이고
  둘째 줄부터는 문단 안의 줄이다.

`--continuation` 을 주면 이어받기 경계 두 쪽을 더 붙인다(회귀 시험 입력이 아니라 기존 결함의
공개 재현용이다).
- S: 앞 간격(1400HU) host 의 여러 행 표가 쪽 아래에서 시작해 다음 쪽으로 넘어간다 —
  이어받은 표 조각 뒤의 host 글줄.
- C: L 과 같은 host 가 쪽 아래에 있어 표 뒤 글줄이 다음 쪽으로 이어진다(이어받은 줄).

뼈대(secPr·colPr·header·패키지 파일)는 공개 샘플 samples/issue2527_empty_linesegs.hwpx 에서
가져오고 rhwp 출처 표식(META-INF/rhwp-hwp5-origin)은 넣지 않는다. 생성물에는 linesegarray 가 없다.

사용: python3 samples/float_push_spacing_before/make_float_push_spacing_before_boundaries.py [--continuation] <출력.hwpx>
(저장소 루트에서)
"""
import re
import sys
import zipfile

src = zipfile.ZipFile('samples/issue2527_empty_linesegs.hwpx')
continuation = '--continuation' in sys.argv[1:]
out = [a for a in sys.argv[1:] if not a.startswith('--')][0]
hdr = src.read('Contents/header.xml').decode('utf8')
pp0 = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', hdr, re.S).group(0)
pp1 = pp0.replace('id="0"', 'id="1"').replace('<hc:prev value="0"', '<hc:prev value="1400"')
pp2 = pp0.replace('id="0"', 'id="2"').replace('<hc:prev value="0"', '<hc:prev value="4000"')
hdr = hdr.replace('<hh:paraProperties itemCnt="1">' + pp0,
                  '<hh:paraProperties itemCnt="3">' + pp0 + pp1 + pp2)
assert 'itemCnt="3">' in hdr
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]  # 첫 문단의 secPr·colPr 까지


def para(pid, runs, pp=0, page_break=False):
    return (f'<hp:p id="{pid}" paraPrIDRef="{pp}" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{runs}</hp:p>\n')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell(c, r, w, h, text):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + para(100000 + r * 10 + c, t(text)) +
            f'</hp:subList><hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="510" right="510" top="0" bottom="0"/></hp:tc>')


def table(tid, rows, row_h, margin):
    w = 48000 // 3
    top, bottom = margin
    trs = ''.join('<hp:tr>' + ''.join(cell(c, r, w, row_h, rows[r][c]) for c in range(3)) + '</hp:tr>'
                  for r in range(len(rows)))
    return (f'<hp:tbl id="{tid}" zOrder="{tid}" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{len(rows)}" colCnt="3" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w * 3}" widthRelTo="ABSOLUTE" height="{row_h * len(rows)}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            f'<hp:outMargin left="140" right="140" top="{top}" bottom="{bottom}"/>'
            '<hp:inMargin left="510" right="510" top="0" bottom="0"/>'
            + trs + '</hp:tbl>')


def host(pid, tid, rows, row_h, margin, text, pp, page_break=False):
    return para(pid, '<hp:run charPrIDRef="0">' + table(tid, rows, row_h, margin) + '</hp:run>' + t(text),
                pp=pp, page_break=page_break)


small = [['가', '1,000', '-']]
body = [head + '</hp:run>' + t('문단 앞 간격·자리차지 표 경계 합성 입력 (#7330)') + '</hp:p>\n',
        para(10, t('1. 표 앞 문단이다.')),
        host(20, 301, small, 1000, (0, 0), 'M. 앞 간격이 표 점유보다 큰 host 의 첫 글줄', pp=2),
        para(21, t('M 뒤 문단이다.')),
        para(22, t('대조: 같은 앞 간격의 표 없는 문단'), pp=2),
        para(23, t('대조 뒤 문단이다.')),
        host(24, 302, small, 1000, (0, 0), '대조: 앞 간격 없는 host 의 첫 글줄', pp=0),
        para(25, t('대조 host 뒤 문단이다.')),
        host(30, 303, small, 1000, (0, 0), 'T. 쪽 맨 위 host 의 첫 글줄 — 앞 간격이 표 점유보다 크다', pp=2,
             page_break=True),
        para(31, t('T 뒤 문단이다.'))]
# 2쪽: T 뒤를 채움 문단으로 채운다 — 첫 글줄 위치를 측정이 과소 계상하면 이 쪽에 문단이 더 남는다.
for i in range(42):
    body.append(para(200 + i, t(f'T 채움 {i + 1}')))
# L 쪽: 표 뒤 글이 여러 줄이다.
long_text = ('L. 표 뒤 글이 여러 줄인 host 문단이다. ' * 6).strip()
body.append(para(90, t('L 쪽 첫 문단이다.'), page_break=True))
body.append(host(91, 305, [['구분', '금액', '비고'], ['가', '1,000', '-']], 2000, (140, 852), long_text, pp=1))
body.append(para(92, t('L 뒤 문단이다.')))
body.append(para(93, t('끝.')))
if continuation:
    # S 쪽: 쪽 아래에서 시작해 넘어가는 표 — 이어받은 표 조각 뒤의 host 글줄.
    body.append(para(40, t('S 쪽 첫 문단이다.'), page_break=True))
    for i in range(30):
        body.append(para(41 + i, t(f'채움 문단 {i + 1}')))
    rows = [[f'{r + 1}행', f'{(r + 1) * 1000:,}', '-'] for r in range(20)]
    body.append(host(80, 304, rows, 2000, (140, 852), 'S. 넘어간 표 조각 뒤 host 의 첫 글줄', pp=1))
    body.append(para(81, t('S 뒤 문단이다.')))
    # C 쪽: 표 뒤 여러 줄 글이 쪽 끝을 넘어 다음 쪽으로 이어진다(이어받은 줄).
    body.append(para(300, t('C 쪽 첫 문단이다.'), page_break=True))
    for i in range(37):
        body.append(para(301 + i, t(f'C 채움 {i + 1}')))
    c_text = ('C. 쪽 끝을 넘는 host 문단이다. ' * 12).strip()
    body.append(host(340, 306, [['구분', '금액', '비고'], ['가', '1,000', '-']], 2000, (140, 852), c_text, pp=1))
    body.append(para(341, t('C 뒤 문단이다.')))
    body.append(para(342, t('끝.')))
sec_new = ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 8, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
