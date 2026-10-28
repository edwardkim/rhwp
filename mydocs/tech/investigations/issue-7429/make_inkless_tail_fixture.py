"""#7429 쪽 나누기 앞 빈 문단 흡수 경계 실험.

블록 i: [첫 문단 글자 크기 z_i, 쪽 나누기] + 한 줄 문단 39개(10pt) + 빈 문단(10pt).
다음 블록의 첫 문단이 쪽 나누기를 선언한다. 빈 문단 줄의 넘침량은 첫 문단 글자 크기 z 로
조절한다(z 가 62.5 HWPUNIT 늘 때마다 100 HWPUNIT). `samples/issue7429/inkless_tail_synthetic.hwpx`
는 아래 명령으로 만들었다.

    python make_inkless_tail_fixture.py out.hwpx 1000,1414,1476,1539,1601,1664,1726,1789,1851,1914,1976,2039,2101,2164
"""
import glob, re, sys, zipfile

OUT = sys.argv[1]
ZS = [int(v) for v in sys.argv[2].split(',')]
tpl = [p for p in glob.glob('samples/*.hwpx') if '종이기준' in p][0]
zi = zipfile.ZipFile(tpl)
header = zi.read('Contents/header.xml').decode('utf-8')
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)
cp_count = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
new = []
for i, z in enumerate(ZS):
    new.append(re.sub(r'height="\d+"', f'height="{z}"', cp0.replace('id="0"', f'id="{cp_count + i}"', 1), count=1))
header = header.replace('</hh:charProperties>', ''.join(new) + '</hh:charProperties>', 1)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{cp_count + len(ZS)}"', header, 1)


def para(text, cp=0, page_break=0):
    t = f'<hp:t>{text}</hp:t>' if text else ''
    return (f'<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="{page_break}" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="{cp}">{t}</hp:run></hp:p>')


body = []
for i, z in enumerate(ZS):
    body.append(para(f'B{i}Z{z}', cp_count + i, 1 if i else 0))
    body += [para(f'줄{k}') for k in range(39)]
    body.append(para(''))
body.append(para('END', 0, 1))
section = zi.read('Contents/section0.xml').decode('utf-8')
section = section.replace('</hs:sec>', ''.join(body) + '</hs:sec>')
with zipfile.ZipFile(OUT, 'w') as zo:
    for it in zi.infolist():
        data = zi.read(it.filename)
        if it.filename == 'Contents/header.xml':
            data = header.encode('utf-8')
        elif it.filename == 'Contents/section0.xml':
            data = section.encode('utf-8')
        zo.writestr(it, data, compress_type=zipfile.ZIP_STORED if it.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
print(OUT, ZS)
