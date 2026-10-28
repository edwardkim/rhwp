"""#7418 글머리표 문단의 줄 나눔 상자 실험 fixture 생성기.

한/글은 글머리표 뒤에서 본문을 시작하고 둘째 줄부터도 같은 자리에 맞춘다(행잉). 줄 나눔이
마커 폭을 빼지 않으면 줄마다 마커 폭만큼 더 담는다. 이 fixture 는 그 차이를 condense 와 함께
가른다.

같은 틀(`samples/종이기준.hwpx`, 10pt · 양쪽 정렬 · 가용 42520 HWPUNIT)에 글꼴을 맑은 고딕으로
바꾸고, 글머리표 `❍`(본문과의 거리 50%) 하나를 더한 뒤 같은 글의 문단 5개를 넣는다.
저장 줄(`hp:linesegarray`)은 넣지 않으므로 한/글과 rhwp 가 각자 줄을 나눈다.

    1. 글머리표 · 글자 단위 · condense 0
    2. 글머리표 · 글자 단위 · condense 20
    3. 글머리표 · 글자 단위 · condense 50
    4. 글머리표 · 낱말 단위 · condense 20
    5. 대조군: 글머리표 없음 · 글자 단위 · condense 20 (나머지 모양 같음)

모든 문단의 왼쪽 여백은 2000 HWPUNIT 이다.

usage: python make_condense_bullet_fixture.py <출력.hwpx>
"""
import glob
import random
import re
import sys
import zipfile

OUT = sys.argv[1]
random.seed(74181)
SYL = '가나다라마바사아자차카타파하거너더러머버서어저처커터퍼허고노도로모보소오조초'


def words(lengths, n):
    return ' '.join(''.join(random.choice(SYL) for _ in range(random.choice(lengths))) for _ in range(n))


TEXT = words([1, 2, 2, 3, 3, 3, 4, 4, 5, 6], 120)
BULLET_ID = 1

# (글머리표, condense, breakNonLatinWord)
SPECS = [
    (True, 0, 'KEEP_WORD'),
    (True, 20, 'KEEP_WORD'),
    (True, 50, 'KEEP_WORD'),
    (True, 20, 'BREAK_WORD'),
    (False, 20, 'KEEP_WORD'),
]

tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')
assert '<hh:bullets' not in header, '틀에 글머리표 정의가 이미 있다 — 번호 매김을 다시 확인할 것'
base = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', header, re.S).group(0)
assert 'breakNonLatinWord="KEEP_WORD"' in base and 'condense="0"' in base
assert '<hh:heading type="NONE" idRef="0" level="0"/>' in base
# HWPX 여백은 `HwpUnitChar` case 가 실제 값, default 가 그 두 배다.
assert base.count('<hc:left value="0" unit="HWPUNIT"/>') == 2
count = int(re.search(r'<hh:paraProperties itemCnt="(\d+)"', header).group(1))

new_pr, paras = [], []
for i, (bullet, condense, unit) in enumerate(SPECS):
    pid = count + i
    pr = (
        base.replace('id="0"', f'id="{pid}"', 1)
        .replace('condense="0"', f'condense="{condense}"', 1)
        .replace('breakNonLatinWord="KEEP_WORD"', f'breakNonLatinWord="{unit}"', 1)
        .replace('<hc:left value="0" unit="HWPUNIT"/>', '<hc:left value="2000" unit="HWPUNIT"/>', 1)
        .replace('<hc:left value="0" unit="HWPUNIT"/>', '<hc:left value="4000" unit="HWPUNIT"/>', 1)
    )
    if bullet:
        pr = pr.replace(
            '<hh:heading type="NONE" idRef="0" level="0"/>',
            f'<hh:heading type="BULLET" idRef="{BULLET_ID}" level="0"/>',
            1,
        )
    new_pr.append(pr)
    paras.append(
        f'<hp:p id="0" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        f'<hp:run charPrIDRef="0"><hp:t>{TEXT}</hp:t></hp:run></hp:p>'
    )
header = header.replace('</hh:paraProperties>', ''.join(new_pr) + '</hh:paraProperties>', 1)
header = re.sub(
    r'<hh:paraProperties itemCnt="\d+"', f'<hh:paraProperties itemCnt="{count + len(SPECS)}"', header, 1
)
BULLETS = (
    f'<hh:bullets itemCnt="1"><hh:bullet id="{BULLET_ID}" char="❍" useImage="0">'
    '<hh:paraHead level="0" align="LEFT" useInstWidth="0" autoIndent="1" widthAdjust="0" '
    'textOffsetType="PERCENT" textOffset="50" numFormat="DIGIT" charPrIDRef="4294967295" checkable="0"/>'
    '</hh:bullet></hh:bullets>'
)
assert '</hh:numberings>' in header
header = header.replace('</hh:numberings>', '</hh:numberings>' + BULLETS, 1)

# 글꼴: 맑은 고딕 — rhwp 의 폭 측정이 한/글과 같은 face 여야 상자 폭만 따로 잴 수 있다.
MALGUN = ('<hh:font id="{id}" face="맑은 고딕" type="TTF" isEmbedded="0"><hh:typeInfo familyType="FCAT_GOTHIC" '
          'weight="6" proportion="4" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" '
          'xHeight="1"/></hh:font>')


def add_font(m):
    n = int(m.group(2))
    return f'<hh:fontface lang="{m.group(1)}" fontCnt="{n + 1}">{m.group(3)}{MALGUN.format(id=n)}</hh:fontface>'


fonts_before = {m.group(1): int(m.group(2)) for m in re.finditer(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">', header)}
assert len(set(fonts_before.values())) == 1, fonts_before
font_id = next(iter(fonts_before.values()))
header = re.sub(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">(.*?)</hh:fontface>', add_font, header, flags=re.S)
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)
cp_id = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
cp_new = re.sub(r'<hh:fontRef [^>]*/>',
                '<hh:fontRef ' + ' '.join(f'{k}="{font_id}"' for k in
                                         ('hangul', 'latin', 'hanja', 'japanese', 'other', 'symbol', 'user')) + '/>',
                cp0.replace('id="0"', f'id="{cp_id}"', 1))
header = header.replace('</hh:charProperties>', cp_new + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cp_id + 1}"', header, 1)
paras = [p.replace('charPrIDRef="0"', f'charPrIDRef="{cp_id}"') for p in paras]
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
