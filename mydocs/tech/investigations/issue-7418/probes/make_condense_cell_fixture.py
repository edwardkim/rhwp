"""#7418 condense(공백 최소값) 표 칸 실험 fixture 생성기.

본문 실험(`make_condense_fixture.py`)과 같은 13문단을 각각 **1×1 표 칸 하나**에 넣는다
(칸 폭 42520, 칸 안 여백 좌우 510 → 가용 41500 HWPUNIT, 맑은 고딕 10pt, 양쪽 정렬).
문단 모양·글·난수 씨앗은 본문 실험과 같다. 저장 줄(`hp:linesegarray`)은 넣지 않는다.

칸은 본문보다 상자가 좁아 앞 줄이 자연폭으로 **상자를 꼭 채우는** 경우가 생긴다. 한/글은 그
줄에 condense 로 새 낱말을 들이지 않는다(새 낱말은 앞 줄 자연폭 < 상자일 때만).

usage: python make_condense_cell_fixture.py <출력.hwpx>
"""
import glob
import random
import re
import sys
import zipfile

OUT = sys.argv[1]
random.seed(7418)
SYL = '가나다라마바사아자차카타파하거너더러머버서어저처커터퍼허고노도로모보소오조초'


def words(lengths, n):
    return ' '.join(''.join(random.choice(SYL) for _ in range(random.choice(lengths))) for _ in range(n))


MIXED = words([1, 2, 2, 3, 3, 3, 4, 4, 5, 6], 170)
SHORT = words([1, 1, 2, 2, 2], 330)
LONG = words([5, 6, 7, 8], 110)

# (condense, breakNonLatinWord, 글)
SPECS = (
    [(c, 'KEEP_WORD', MIXED) for c in (0, 15, 30, 50, 75)]
    + [(c, 'KEEP_WORD', SHORT) for c in (50, 75)]
    + [(c, 'KEEP_WORD', LONG) for c in (50, 75)]
    + [(c, 'BREAK_WORD', MIXED) for c in (0, 30, 50, 75)]
)

tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')
base = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', header, re.S).group(0)
assert 'breakNonLatinWord="KEEP_WORD"' in base and 'condense="0"' in base
count = int(re.search(r'<hh:paraProperties itemCnt="(\d+)"', header).group(1))

new_pr, paras = [], []
for i, (condense, unit, text) in enumerate(SPECS):
    pid = count + i
    new_pr.append(
        base.replace('id="0"', f'id="{pid}"', 1)
        .replace('condense="0"', f'condense="{condense}"', 1)
        .replace('breakNonLatinWord="KEEP_WORD"', f'breakNonLatinWord="{unit}"', 1)
    )
    paras.append(
        f'<hp:p id="0" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run></hp:p>'
    )
header = header.replace('</hh:paraProperties>', ''.join(new_pr) + '</hh:paraProperties>', 1)
header = re.sub(
    r'<hh:paraProperties itemCnt="\d+"', f'<hh:paraProperties itemCnt="{count + len(SPECS)}"', header, 1
)

# 글꼴: 맑은 고딕. rhwp 의 폭 측정이 한/글과 같은 face 여야 condense 규칙만 따로 잴 수 있다
# (함초롬바탕은 한글 전진폭이 rhwp 0.970em / 한/글 0.9725em 로 갈려 경계에서 한 글자씩 어긋난다).
MALGUN = ('<hh:font id="{id}" face="맑은 고딕" type="TTF" isEmbedded="0"><hh:typeInfo familyType="FCAT_GOTHIC" '
          'weight="6" proportion="4" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" '
          'xHeight="1"/></hh:font>')


def add_font(m):
    n = int(m.group(2))
    body = m.group(3)
    return f'<hh:fontface lang="{m.group(1)}" fontCnt="{n + 1}">{body}{MALGUN.format(id=n)}</hh:fontface>'


fonts_before = {m.group(1): int(m.group(2)) for m in re.finditer(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">', header)}
assert len(set(fonts_before.values())) == 1, fonts_before
font_id = next(iter(fonts_before.values()))
header = re.sub(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">(.*?)</hh:fontface>', add_font, header, flags=re.S)
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)
cp_count = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
cp_id = cp_count
cp_new = re.sub(r'<hh:fontRef [^>]*/>',
                '<hh:fontRef ' + ' '.join(f'{k}="{font_id}"' for k in
                                         ('hangul', 'latin', 'hanja', 'japanese', 'other', 'symbol', 'user')) + '/>',
                cp0.replace('id="0"', f'id="{cp_id}"', 1))
header = header.replace('</hh:charProperties>', cp_new + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cp_count + 1}"', header, 1)
paras = [p.replace('charPrIDRef="0"', f'charPrIDRef="{cp_id}"') for p in paras]
# [칸 실험] 각 문단을 1x1 표(칸 폭 42520, 안 여백 510) 하나에 넣는다
TBL = ('<hp:tbl id="{tid}" zOrder="{z}" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
       'dropcapstyle="None" pageBreak="CELL" repeatHeader="1" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
       '<hp:sz width="42520" widthRelTo="ABSOLUTE" height="1716" heightRelTo="ABSOLUTE" protect="0"/>'
       '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
       'horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
       '<hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="510" right="510" top="141" bottom="141"/>'
       '<hp:tr><hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
       '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" '
       'textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{body}</hp:subList><hp:cellAddr colAddr="0" rowAddr="0"/>'
       '<hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="42520" height="1716"/>'
       '<hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc></hp:tr></hp:tbl>')
paras = [f'<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="{1 if i else 0}" columnBreak="0" merged="0"><hp:run charPrIDRef="{cp_id}">'
         + TBL.format(tid=1000 + i, z=i, body=p) + '<hp:t/></hp:run></hp:p>' for i, p in enumerate(paras)]
section = zi.read('Contents/section0.xml').decode('utf-8')
section = section.replace('</hs:sec>', ''.join(paras) + '</hs:sec>')

with zipfile.ZipFile(OUT, 'w') as zo:
    for it in zi.infolist():
        data = zi.read(it.filename)
        if it.filename == 'Contents/header.xml':
            data = header.encode('utf-8')
        elif it.filename == 'Contents/section0.xml':
            data = section.encode('utf-8')
        zo.writestr(
            it,
            data,
            compress_type=zipfile.ZIP_STORED if it.filename == 'mimetype' else zipfile.ZIP_DEFLATED,
        )
print(f'{OUT}: {len(SPECS)} paragraphs')
