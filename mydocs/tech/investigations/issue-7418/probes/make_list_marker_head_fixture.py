"""#7418 글머리표 머리 모양(paraHead) 속성 fixture 생성기.

마커 영역(글자 폭 + 너비 보정 + 본문과의 거리)과 자동 내어쓰기·정렬이 본문 시작과 둘째 줄
위치, 줄 나눔을 어떻게 바꾸는지 한/글로 잰다. 문단마다 글머리표 정의를 하나씩 두고
(`❍`, 맑은 고딕 10pt, 왼쪽 여백 4000 HWPUNIT) 속성만 바꾼다. 저장 줄은 넣지 않는다.

    0~35  자동 내어쓰기 · 왼쪽 정렬: 너비 보정 {0,-1000,1000,3000} × 거리 {50,0,100}%
          × 들여쓰기 {0,-1200,1000}
    36~38 자동 내어쓰기 끔: 들여쓰기 {0,-1200,1000}
    39~42 가운데·오른쪽 정렬: 너비 보정 {0,3000}

사양 목록은 `<출력>.json` 에 함께 쓴다.

usage: python make_list_marker_head_fixture.py <출력.hwpx>
"""
import glob, re, sys, zipfile, itertools, json

OUT = sys.argv[1]
SPECS = []
for wa, to, ind in itertools.product((0, -1000, 1000, 3000), (50, 0, 100), (0, -1200, 1000)):
    SPECS.append(dict(wa=wa, to=to, ind=ind, auto=1, align='LEFT'))
for ind in (0, -1200, 1000):
    SPECS.append(dict(wa=0, to=50, ind=ind, auto=0, align='LEFT'))
for align in ('CENTER', 'RIGHT'):
    for wa in (0, 3000):
        SPECS.append(dict(wa=wa, to=50, ind=0, auto=1, align=align))
json.dump(SPECS, open(OUT + '.json', 'w'))

TEXT = ('가나다 라마바 사아자 차카타 파하거 너더러 머버서 어저처 커터퍼 허고노 도로모 보소오 '
        '조초가 나다라 마바사 아자차 카타파 하거너 더러머 버서어 저처커 터퍼허 고노도')
tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')
base = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', header, re.S).group(0)
count = int(re.search(r'<hh:paraProperties itemCnt="(\d+)"', header).group(1))
bullets, prs, paras = [], [], []
for i, sp in enumerate(SPECS):
    bid = i + 1
    bullets.append(
        f'<hh:bullet id="{bid}" char="❍" useImage="0"><hh:paraHead level="0" align="{sp["align"]}" '
        f'useInstWidth="0" autoIndent="{sp["auto"]}" widthAdjust="{sp["wa"]}" textOffsetType="PERCENT" '
        f'textOffset="{sp["to"]}" numFormat="DIGIT" charPrIDRef="4294967295" checkable="0"/></hh:bullet>')
    pid = count + i
    pr = (base.replace('id="0"', f'id="{pid}"', 1)
          .replace('<hc:left value="0" unit="HWPUNIT"/>', '<hc:left value="4000" unit="HWPUNIT"/>', 1)
          .replace('<hc:left value="0" unit="HWPUNIT"/>', '<hc:left value="8000" unit="HWPUNIT"/>', 1)
          .replace('<hc:intent value="0" unit="HWPUNIT"/>', f'<hc:intent value="{sp["ind"]}" unit="HWPUNIT"/>', 1)
          .replace('<hc:intent value="0" unit="HWPUNIT"/>', f'<hc:intent value="{sp["ind"] * 2}" unit="HWPUNIT"/>', 1)
          .replace('<hh:heading type="NONE" idRef="0" level="0"/>', f'<hh:heading type="BULLET" idRef="{bid}" level="0"/>', 1))
    prs.append(pr)
    paras.append(f'<hp:p id="0" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                 f'<hp:run charPrIDRef="0"><hp:t>B{i:02d} {TEXT}</hp:t></hp:run></hp:p>')
header = header.replace('</hh:paraProperties>', ''.join(prs) + '</hh:paraProperties>', 1)
header = re.sub(r'<hh:paraProperties itemCnt="\d+"', f'<hh:paraProperties itemCnt="{count + len(SPECS)}"', header, 1)
header = header.replace('</hh:numberings>', f'</hh:numberings><hh:bullets itemCnt="{len(SPECS)}">' + ''.join(bullets) + '</hh:bullets>', 1)
MALGUN = ('<hh:font id="{id}" face="맑은 고딕" type="TTF" isEmbedded="0"><hh:typeInfo familyType="FCAT_GOTHIC" '
          'weight="6" proportion="4" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" xHeight="1"/></hh:font>')
fonts = {m.group(1): int(m.group(2)) for m in re.finditer(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">', header)}
font_id = next(iter(fonts.values()))
header = re.sub(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">(.*?)</hh:fontface>',
                lambda m: f'<hh:fontface lang="{m.group(1)}" fontCnt="{int(m.group(2)) + 1}">{m.group(3)}{MALGUN.format(id=int(m.group(2)))}</hh:fontface>',
                header, flags=re.S)
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)
cp_id = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
cp_new = re.sub(r'<hh:fontRef [^>]*/>', '<hh:fontRef ' + ' '.join(f'{k}="{font_id}"' for k in
                ('hangul', 'latin', 'hanja', 'japanese', 'other', 'symbol', 'user')) + '/>', cp0.replace('id="0"', f'id="{cp_id}"', 1))
header = header.replace('</hh:charProperties>', cp_new + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cp_id + 1}"', header, 1)
paras = [p.replace('charPrIDRef="0"', f'charPrIDRef="{cp_id}"') for p in paras]
section = zi.read('Contents/section0.xml').decode('utf-8').replace('</hs:sec>', ''.join(paras) + '</hs:sec>')
with zipfile.ZipFile(OUT, 'w') as zo:
    for it in zi.infolist():
        data = zi.read(it.filename)
        if it.filename == 'Contents/header.xml':
            data = header.encode('utf-8')
        elif it.filename == 'Contents/section0.xml':
            data = section.encode('utf-8')
        zo.writestr(it, data, compress_type=zipfile.ZIP_STORED if it.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
print(OUT, len(SPECS))
