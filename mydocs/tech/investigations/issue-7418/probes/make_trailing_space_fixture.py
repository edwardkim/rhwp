"""#7418 문단 끝 공백이 상자를 넘는 경우의 줄 수 fixture 생성기.

같은 틀(`samples/종이기준.hwpx`, 10pt · 양쪽 정렬 · 가용 42520 HWPUNIT)에 글꼴을 맑은 고딕으로
바꾸고, 한글 음절(전진폭 1000)과 끝 공백 수만 다른 문단을 넣는다. 저장 줄(`hp:linesegarray`)은
넣지 않으므로 한/글과 rhwp 가 각자 줄을 나눈다.

    1. 42자 + 공백 0   (대조군: 42000, 넘치지 않음)
    2. 42자 + 공백 1
    3. 42자 + 공백 2   (공백이 상자를 넘는다)
    4. 42자 + 공백 3
    5. 42자 + 공백 2 + 42자   (대조군: 넘친 공백 뒤에 글이 이어진다)

한/글은 문단 끝의 넘친 공백을 줄 밖에 걸고 새 줄을 만들지 않는다.

usage: python make_trailing_space_fixture.py <출력.hwpx>
"""
import glob
import re
import sys
import zipfile

OUT = sys.argv[1]
SYL = '가나다라마바사아자차카타파하거너더러머버서어저처커터퍼허고노도로모보소오조초'
LINE = (SYL * 2)[:42]
TEXTS = [LINE, LINE + ' ', LINE + '  ', LINE + '   ', LINE + '  ' + LINE]

tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')

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

paras = [
    f'<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
    f'<hp:run charPrIDRef="{cp_id}"><hp:t>{text}</hp:t></hp:run></hp:p>'
    for text in TEXTS
]
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
print(f'{OUT}: {len(TEXTS)} paragraphs')
