"""#8639 문단 끝 공백 줄 나눔 경계 fixture 생성기.

#7418 끝 공백 실험(`issue-7418/probes/make_trailing_space_fixture.py`)과 같은 틀
(`samples/종이기준.hwpx`, 맑은 고딕 10pt · 양쪽 정렬 · 가용 42520 HWPUNIT)을 쓴다.
#7418 은 **첫 공백이 상자 안에 들고 둘째 공백부터 넘치는** 경우만 쟀다(42자 = 42000).
여기서는 마지막 글자가 상자 안에 겨우 들어 **첫 공백부터 넘치는** 경우를 더한다
(한글 41자 + `W` + `1` ≈ 42504, 공백 500 HU). 저장 줄은 넣지 않는다.

    1. X + 공백 1
    2. X + 공백 2                     (같은 글자모양)
    3. X + 공백 1 + [굵게] 공백 1     (둘째 공백만 다른 글자모양 — docs#636 중간보고서 꼴)
    4. X + 공백 3
    5. 42자 + 공백 2                  (#7418 대조군)
    6. 42자 + 공백 3                  (#7418 대조군)
    7. X + 공백 1 + 한글 10자         (넘친 공백 뒤에 글이 이어진다)
    8. X + 공백 2 + 한글 10자
    9. X + 공백 3 + 한글 10자

usage: python make_trailing_space_overflow_fixture.py <출력.hwpx>
"""
import glob
import re
import sys
import zipfile

OUT = sys.argv[1]
SYL = '가나다라마바사아자차카타파하거너더러머버서어저처커터퍼허고노도로모보소오조초'
LINE42 = (SYL * 2)[:42]
X = (SYL * 2)[:41] + 'W1'

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
cp_plain = re.sub(r'<hh:fontRef [^>]*/>',
                  '<hh:fontRef ' + ' '.join(f'{k}="{font_id}"' for k in
                                           ('hangul', 'latin', 'hanja', 'japanese', 'other', 'symbol', 'user')) + '/>',
                  cp0.replace('id="0"', f'id="{cp_id}"', 1))
cp_bold = cp_plain.replace(f'id="{cp_id}"', f'id="{cp_id + 1}"', 1)
cp_bold = re.sub(r'(<hh:fontRef [^>]*/>)', r'\1', cp_bold)
cp_bold = cp_bold.replace('<hh:underline', '<hh:bold/><hh:underline', 1)
assert '<hh:bold/>' in cp_bold, 'bold 삽입 위치를 찾지 못했다'
header = header.replace('</hh:charProperties>', cp_plain + cp_bold + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cp_id + 2}"', header, 1)


def run(text, cp=cp_id):
    return f'<hp:run charPrIDRef="{cp}"><hp:t>{text}</hp:t></hp:run>'


PARAS = [
    run(X + ' '),
    run(X + '  '),
    run(X + ' ') + run(' ', cp_id + 1),
    run(X + '   '),
    run(LINE42 + '  '),
    run(LINE42 + '   '),
    run(X + ' ' + SYL[:10]),
    run(X + '  ' + SYL[:10]),
    run(X + '   ' + SYL[:10]),
]
paras = ''.join(
    f'<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{body}</hp:p>'
    for body in PARAS
)
section = zi.read('Contents/section0.xml').decode('utf-8')
section = section.replace('</hs:sec>', paras + '</hs:sec>')

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
print(f'{OUT}: {len(PARAS)} paragraphs')
