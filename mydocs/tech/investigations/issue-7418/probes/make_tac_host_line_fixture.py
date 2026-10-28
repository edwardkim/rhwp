"""#7418 글자처럼 취급 표 하나만 든 host 문단의 줄 회계 fixture 생성기.

저장 줄(`hp:linesegarray`)이 없는 문서에서 글자처럼 취급(treatAsChar=1) 표 하나만 든 문단이
흐름에서 얼마를 차지하는지 한/글로 잰다. 한/글이 저장한 본의 `hp:lineseg` 가 그 문단 줄의
높이·줄간격을 기록한다. `samples/task1768/distribution_doc.hwpx`(배포용, 저장 줄 없음) 2쪽의
형상이 기준이다 — 17pt 글자 모양 · 줄간격 200% · 1×3 표(높이 3014, 바깥 여백 141).

문단 쌍 (표 host, 뒤따르는 본문 한 줄)을 사양마다 하나씩 둔다. 사양은
글자 크기 {10, 17}pt × 줄간격 {100, 160, 200}% × 표 높이 {1716, 3014} 이다.

usage: python make_tac_host_line_fixture.py <출력.hwpx>
"""
import glob
import itertools
import json
import re
import sys
import zipfile

OUT = sys.argv[1]
SPECS = [dict(pt=pt, ls=ls, th=th) for pt, ls, th in itertools.product((10, 17), (100, 160, 200), (1716, 3014))]

tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')
base = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', header, re.S).group(0)
pr_count = int(re.search(r'<hh:paraProperties itemCnt="(\d+)"', header).group(1))
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)
cp_count = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))

prs, cps = {}, {}
for sp in SPECS:
    if sp['ls'] not in prs:
        prs[sp['ls']] = pr_count + len(prs)
    if sp['pt'] not in cps:
        cps[sp['pt']] = cp_count + len(cps)
header = header.replace('</hh:paraProperties>', ''.join(
    base.replace('id="0"', f'id="{pid}"', 1).replace('value="160"', f'value="{ls}"')
    for ls, pid in prs.items()) + '</hh:paraProperties>', 1)
header = re.sub(r'<hh:paraProperties itemCnt="\d+"', f'<hh:paraProperties itemCnt="{pr_count + len(prs)}"', header, 1)
header = header.replace('</hh:charProperties>', ''.join(
    cp0.replace('id="0"', f'id="{cid}"', 1).replace('height="1000"', f'height="{pt * 100}"', 1)
    for pt, cid in cps.items()) + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cp_count + len(cps)}"', header, 1)

WIDTHS = (3223, 1414, 15109)


def cell(col, x, th, text):
    return ('<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
            '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run></hp:p></hp:subList>'
            f'<hp:cellAddr colAddr="{col}" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{WIDTHS[col]}" height="{th}"/>'
            '<hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')


def table(tid, th):
    cells = ''.join(cell(c, 0, th, t) for c, t in enumerate(('Ⅰ', '', '관련근거')))
    return (f'<hp:tbl id="{tid}" zOrder="{tid}" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="1" rowCnt="1" colCnt="3" cellSpacing="0" '
            'borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{sum(WIDTHS)}" widthRelTo="ABSOLUTE" height="{th}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="141" right="141" top="141" bottom="141"/>'
            '<hp:inMargin left="141" right="141" top="141" bottom="141"/>'
            f'<hp:tr>{cells}</hp:tr></hp:tbl>')


paras = []
for i, sp in enumerate(SPECS):
    pid, cid = prs[sp['ls']], cps[sp['pt']]
    paras.append(f'<hp:p id="0" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                 f'<hp:run charPrIDRef="{cid}">{table(2000 + i, sp["th"])}<hp:t/></hp:run></hp:p>')
    paras.append(f'<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                 f'<hp:run charPrIDRef="0"><hp:t>T{i:02d} 뒤따르는 본문</hp:t></hp:run></hp:p>')
section = zi.read('Contents/section0.xml').decode('utf-8')
# 템플릿 첫 문단(구역 정의)의 저장 줄도 걷어 문서 전체를 저장 줄 없는 입력으로 만든다 — 배포용
# 원본과 같은 형상이다. 저장 줄이 하나라도 있으면 rhwp 는 그 구역을 저장 사다리 구역으로 보고
# 다른 합성 규칙(#2243)을 쓴다.
section = re.sub(r'<hp:linesegarray>.*?</hp:linesegarray>', '', section, flags=re.S)
section = section.replace('</hs:sec>', ''.join(paras) + '</hs:sec>')

json.dump(SPECS, open(OUT + '.json', 'w'))
with zipfile.ZipFile(OUT, 'w') as zo:
    for it in zi.infolist():
        data = zi.read(it.filename)
        if it.filename == 'Contents/header.xml':
            data = header.encode('utf-8')
        elif it.filename == 'Contents/section0.xml':
            data = section.encode('utf-8')
        zo.writestr(it, data, compress_type=zipfile.ZIP_STORED if it.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
print(OUT, len(SPECS))
