"""#7418 맨 앞 공백 condense 실험 fixture 생성기.

같은 틀(`samples/종이기준.hwpx`)에 맑은 고딕 10pt · 양쪽 정렬 · 글자 단위(KEEP_WORD) 문단 8개를 넣는다.
문단은 맨 앞 공백 수(0·2·4·6)와 condense(75·50)만 다르다. 첫 줄은 4자 낱말 8개 뒤에 12자
낱말이 오도록 짜서, 맨 앞 공백을 줄이는지에 따라 첫 줄 끝 글자가 갈리게 했다.
저장 줄(`hp:linesegarray`)은 넣지 않는다.

usage: python make_condense_leading_fixture.py <출력.hwpx>
"""
import glob, random, re, sys, zipfile, os

OUT = sys.argv[1]
random.seed(4)
SYL = '가나다라마바사아자차카타파하거너더러머버서어저처커터퍼허고노도로모보소오조초'
def w(n): return ''.join(random.choice(SYL) for _ in range(n))
base = ' '.join([w(4) for _ in range(8)] + [w(12)] + [w(3) for _ in range(20)])
SPECS = [(c, lead) for c in (75, 50) for lead in (0, 2, 4, 6)]

tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')
p0 = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', header, re.S).group(0)
count = int(re.search(r'<hh:paraProperties itemCnt="(\d+)"', header).group(1))
MALGUN = ('<hh:font id="{id}" face="맑은 고딕" type="TTF" isEmbedded="0"><hh:typeInfo familyType="FCAT_GOTHIC" '
          'weight="6" proportion="4" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" xHeight="1"/></hh:font>')
fid = int(re.search(r'<hh:fontface lang="HANGUL" fontCnt="(\d+)">', header).group(1))
header = re.sub(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">(.*?)</hh:fontface>',
                lambda m: f'<hh:fontface lang="{m.group(1)}" fontCnt="{int(m.group(2)) + 1}">{m.group(3)}{MALGUN.format(id=int(m.group(2)))}</hh:fontface>',
                header, flags=re.S)
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)
cpc = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
cpn = re.sub(r'<hh:fontRef [^>]*/>', '<hh:fontRef ' + ' '.join(f'{k}="{fid}"' for k in ('hangul', 'latin', 'hanja', 'japanese', 'other', 'symbol', 'user')) + '/>', cp0.replace('id="0"', f'id="{cpc}"', 1))
header = header.replace('</hh:charProperties>', cpn + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cpc + 1}"', header, 1)
new_pr, paras, texts = [], [], []
for i, (c, lead) in enumerate(SPECS):
    pid = count + i
    new_pr.append(p0.replace('id="0"', f'id="{pid}"', 1).replace('condense="0"', f'condense="{c}"', 1))
    t = ' ' * lead + base
    texts.append(t)
    paras.append(f'<hp:p id="0" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="{cpc}"><hp:t>{t}</hp:t></hp:run></hp:p>')
header = header.replace('</hh:paraProperties>', ''.join(new_pr) + '</hh:paraProperties>', 1)
header = re.sub(r'<hh:paraProperties itemCnt="\d+"', f'<hh:paraProperties itemCnt="{count + len(SPECS)}"', header, 1)
section = zi.read('Contents/section0.xml').decode('utf-8').replace('</hs:sec>', ''.join(paras) + '</hs:sec>')
with zipfile.ZipFile(OUT, 'w') as zo:
    for it in zi.infolist():
        data = zi.read(it.filename)
        if it.filename == 'Contents/header.xml':
            data = header.encode('utf-8')
        elif it.filename == 'Contents/section0.xml':
            data = section.encode('utf-8')
        zo.writestr(it, data, compress_type=zipfile.ZIP_STORED if it.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
print(OUT, SPECS)
