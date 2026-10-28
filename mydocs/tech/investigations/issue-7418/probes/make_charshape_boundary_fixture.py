"""#7418 글자 모양 경계와 condense 줄 나눔 실험 fixture 생성기.

`samples/80168_regulatory_analysis.hwp` 75쪽 칸 문단(`issue_7080` 검사)은 한 낱말 `복리시설의`
가운데에서 글자 모양이 바뀐다(`복리` 자간 +1% / `시설의` 자간 0%). 한/글은 바로 그 경계에서
줄을 끊고, rhwp 는 condense(25%)로 `시` 를 한 글자 더 담는다. 경계가 줄 나눔에 영향을 주는지
한/글로 가른다.

같은 문장·14pt·condense 25%·글자 단위 줄 나눔 문단을 세 변형 × 상자 폭 13가지로 둔다.

    V1  원문 글자 모양: [5,21) 자간 +1%, [21,36) 0%, [36,..) −5%
    V2  글자 모양 하나(자간 0%)
    V3  속성이 같은 두 글자 모양을 [21] 경계에서만 나눈다(자간 모두 0%)

상자 폭은 오른쪽 여백으로 조절한다(본문 42520 − 여백). 글꼴은 둘째 인자(기본 맑은 고딕)다.

usage: python make_charshape_boundary_fixture.py <출력.hwpx> [글꼴 이름]
"""
import glob
import json
import re
import sys
import zipfile

OUT = sys.argv[1]
FACE = sys.argv[2] if len(sys.argv) > 2 else '맑은 고딕'
TEXT = '  2. 건축물이 아닌 부대시설ㆍ복리시설의 설치규모를 확대하는 때(위치가 변경되는 경우는 제외한다)'
WIDTHS = list(range(22534, 24935, 200))
VARIANTS = {
    'V1': [(0, 'z'), (5, 'p1'), (21, 'z'), (36, 'm5')],
    'V2': [(0, 'z')],
    'V3': [(0, 'z'), (21, 'z2')],
}
SPEC = [dict(v=v, w=w) for v in VARIANTS for w in WIDTHS]

tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')

MALGUN = ('<hh:font id="{id}" face="' + FACE + '" type="TTF" isEmbedded="0"><hh:typeInfo familyType="FCAT_GOTHIC" '
          'weight="6" proportion="4" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" '
          'xHeight="1"/></hh:font>')
fonts = {m.group(1): int(m.group(2)) for m in re.finditer(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">', header)}
font_id = next(iter(fonts.values()))
header = re.sub(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">(.*?)</hh:fontface>',
                lambda m: f'<hh:fontface lang="{m.group(1)}" fontCnt="{int(m.group(2)) + 1}">{m.group(3)}{MALGUN.format(id=int(m.group(2)))}</hh:fontface>',
                header, flags=re.S)

cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)
cp_count = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
SPACING = {'z': 0, 'z2': 0, 'p1': 1, 'm5': -5}
cps, new_cp = {}, []
for k, sp in SPACING.items():
    cid = cp_count + len(cps)
    cps[k] = cid
    c = cp0.replace('id="0"', f'id="{cid}"', 1).replace('height="1000"', 'height="1400"', 1)
    c = re.sub(r'<hh:fontRef [^>]*/>', '<hh:fontRef ' + ' '.join(f'{x}="{font_id}"' for x in
               ('hangul', 'latin', 'hanja', 'japanese', 'other', 'symbol', 'user')) + '/>', c)
    c = re.sub(r'<hh:spacing [^>]*/>', '<hh:spacing ' + ' '.join(f'{x}="{sp}"' for x in
               ('hangul', 'latin', 'hanja', 'japanese', 'other', 'symbol', 'user')) + '/>', c)
    new_cp.append(c)
header = header.replace('</hh:charProperties>', ''.join(new_cp) + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cp_count + len(cps)}"', header, 1)

base = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', header, re.S).group(0)
pr_count = int(re.search(r'<hh:paraProperties itemCnt="(\d+)"', header).group(1))
prs, new_pr = {}, []
for w in WIDTHS:
    pid = pr_count + len(prs)
    prs[w] = pid
    right = 42520 - w
    new_pr.append(base.replace('id="0"', f'id="{pid}"', 1).replace('condense="0"', 'condense="25"', 1)
                  .replace('<hc:right value="0" unit="HWPUNIT"/>', f'<hc:right value="{right}" unit="HWPUNIT"/>', 1)
                  .replace('<hc:right value="0" unit="HWPUNIT"/>', f'<hc:right value="{right * 2}" unit="HWPUNIT"/>', 1))
header = header.replace('</hh:paraProperties>', ''.join(new_pr) + '</hh:paraProperties>', 1)
header = re.sub(r'<hh:paraProperties itemCnt="\d+"', f'<hh:paraProperties itemCnt="{pr_count + len(prs)}"', header, 1)


def runs(variant):
    cuts = VARIANTS[variant] + [(len(TEXT), None)]
    return ''.join(f'<hp:run charPrIDRef="{cps[k]}"><hp:t>{TEXT[a:b]}</hp:t></hp:run>'
                   for (a, k), (b, _) in zip(cuts, cuts[1:]))


paras = [f'<hp:p id="0" paraPrIDRef="{prs[s["w"]]}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
         f'{runs(s["v"])}</hp:p>' for s in SPEC]
section = zi.read('Contents/section0.xml').decode('utf-8')
section = re.sub(r'<hp:linesegarray>.*?</hp:linesegarray>', '', section, flags=re.S)
section = section.replace('</hs:sec>', ''.join(paras) + '</hs:sec>')
json.dump(SPEC, open(OUT + '.json', 'w'))
with zipfile.ZipFile(OUT, 'w') as zo:
    for it in zi.infolist():
        data = zi.read(it.filename)
        if it.filename == 'Contents/header.xml':
            data = header.encode('utf-8')
        elif it.filename == 'Contents/section0.xml':
            data = section.encode('utf-8')
        zo.writestr(it, data, compress_type=zipfile.ZIP_STORED if it.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
print(OUT, len(SPEC))
