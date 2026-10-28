"""#7418 condense(공백 최소값) 단일 변수 실험 fixture 생성기.

같은 틀(`samples/종이기준.hwpx`, 한글 2024 저장본 · 10pt · 양쪽 정렬)에 글꼴을 맑은 고딕으로
바꾸고 문단 모양만 다른 13문단을 넣는다. 저장 줄(`hp:linesegarray`)은 넣지 않으므로
한/글과 rhwp 가 각자 줄을 나눈다.

    A. 글자 단위(breakNonLatinWord=KEEP_WORD) · 낱말 길이 1~6 · condense 0/15/30/50/75
    B. 글자 단위 · 짧은 낱말(1~2자, 공백 많음) · condense 50/75
    C. 글자 단위 · 긴 낱말(5~8자, 공백 적음) · condense 50/75
    D. 낱말 단위(breakNonLatinWord=BREAK_WORD) · 낱말 길이 1~6 · condense 0/30/50/75

(breakNonLatinWord 의 KEEP_WORD 가 글자 단위, BREAK_WORD 가 낱말 단위라는 역해석은
#2185 와 이 실험의 D 가 함께 확인한다.)

usage: python make_condense_fixture.py <출력.hwpx>
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
