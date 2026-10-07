"""#7418 자간 × condense 실험 fixture 생성기.

`make_condense_fixture.py` 와 같은 13문단(맑은 고딕 10pt, condense 0~75, 글자/낱말 단위)에
모든 글자의 자간(`hh:spacing`)만 SP% 로 둔다. 저장 줄은 넣지 않는다.

한/글 2024 가 저장한 줄은 다음 규칙으로 전부 재현된다(자간 −20·−12·−6·+10%, 777줄).
1. 줄에 들어가는가(자연폭·줄인 폭): 줄 끝 글자의 자간은 빼고 잰다.
2. 새 낱말 허용(앞 줄 자연폭 < 상자): 공백 앞 글자는 줄 끝이 아니므로 자간을 넣고 잰다.
3. 공백의 자간은 공백 폭 비례(반각 → 자간의 절반)다.
4. condense 는 자간 **적용 전** 공백 폭(반각)의 c% 를 줄인다.

usage: python make_condense_letter_spacing_fixture.py <출력.hwpx> <자간%>
"""
import glob
import random
import re
import sys
import zipfile

OUT = sys.argv[1]
SP = int(sys.argv[2])
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
cp_new = re.sub(r'<hh:spacing [^>]*/>', f'<hh:spacing hangul="{SP}" latin="{SP}" hanja="{SP}" japanese="{SP}" other="{SP}" symbol="{SP}" user="{SP}"/>', cp_new)
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
