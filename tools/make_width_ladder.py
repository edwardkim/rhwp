#!/usr/bin/env python
"""#2156 문자폭 프로브 — 문자 클래스 x 반복수 사다리 HWPX 생성.

원리 (#2150 줄높이 공식 역이용): ls=100% 에서 행높이 = n x em + pad 이므로
행높이 -> 줄수 n 이 정확히 역산된다. 고정 폭 셀에 동일 문자 k 반복 문자열을
넣으면 n = ceil(k x w / inner) 에서 유효 문자폭 w 구간이 나온다:
  w ∈ [(n-1) x inner / k, n x inner / k)
k 사다리(100/200/400/800)의 구간 교집합으로 서브픽셀 정밀도 확보.

rhwp 도 동일 픽스처로 대조 가능 — ls=100% 에선 양쪽 줄높이 모델이 일치(em).
"""
import re
import sys
import zipfile
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(r"C:\Users\planet\rhwp")
SKEL = ROOT / "samples" / "tac-host-spacing.hwpx"
OUT_DIR = ROOT / "output" / "poc" / "task2156"
OUT = OUT_DIR / "width_ladder.hwpx"
OUT_DIR.mkdir(parents=True, exist_ok=True)

# 문자 클래스 (10pt 기본 글꼴). 단독 반복이 무의미한 문자(공백 등)는
# "가"와의 페어 반복으로 프로브: w(pair) = w(가) + w(char).
CLASSES = [
    ("hangul", "가"),
    ("digit", "0"),
    ("upper", "A"),
    ("lower", "a"),
    ("paren", "("),
    ("comma", ","),
    ("parenR", ")"),
    ("dot", "."),
    ("space+가", "가 "),
    ("middot", "·"),
]
K_VALUES = [100, 200, 400, 800]
CELL_W = 40000  # 넓은 셀 — 줄당 문자수를 키워 정밀도 확보
CELL_H = 288
PAD = 141

with zipfile.ZipFile(SKEL) as z:
    names = z.namelist()
    data = {n: z.read(n) for n in names}

header = data["Contents/header.xml"].decode("utf-8")
section = data["Contents/section0.xml"].decode("utf-8")

# paraPr: ls=100% 하나 추가
para_ids = [int(m) for m in re.findall(r'<hh:paraPr id="(\d+)"', header)]
pid = max(para_ids) + 1
m = re.search(r'(<hh:paraPr id="0".*?</hh:paraPr>)', header, re.S)
assert m
pr = re.sub(r'id="0"', f'id="{pid}"', m.group(1), count=1)
pr, n = re.subn(r'<hh:lineSpacing[^/]*/>',
                '<hh:lineSpacing type="PERCENT" value="100" unit="HWPUNIT"/>', pr)
assert n == 1
cm = re.search(r'<hh:paraProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:paraProperties itemCnt="{cm.group(1)}"',
                        f'<hh:paraProperties itemCnt="{int(cm.group(1)) + 1}"')
header = re.sub(r'(</hh:paraProperties>)', pr + r"\1", header, count=1)

rows = []
ridx = 0
pid_ctr = 100
manifest = []  # (row, class, k)

for cname, ch in CLASSES:
    for k in K_VALUES:
        pid_ctr += 1
        text = ch * k
        p = (f'<hp:p id="{pid_ctr}" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" '
             f'columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>{text}</hp:t>'
             f'</hp:run></hp:p>')
        rows.append(
            f'<hp:tr><hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" '
            f'dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" '
            f'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" '
            f'hasTextRef="0" hasNumRef="0">{p}</hp:subList>'
            f'<hp:cellAddr colAddr="0" rowAddr="{ridx}"/>'
            f'<hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{CELL_W}" height="{CELL_H}"/>'
            f'<hp:cellMargin left="{PAD}" right="{PAD}" top="{PAD}" bottom="{PAD}"/>'
            f'</hp:tc></hp:tr>')
        manifest.append((ridx, cname, k))
        ridx += 1

tm = re.search(r'(<hp:tbl .*?</hp:tbl>)', section, re.S)
assert tm
new_tbl = (f'<hp:tbl id="100" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" '
           f'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" '
           f'repeatHeader="0" rowCnt="{ridx}" colCnt="1" cellSpacing="0" '
           f'borderFillIDRef="2" noAdjust="0">'
           f'<hp:sz width="{CELL_W}" widthRelTo="ABSOLUTE" height="{CELL_H * ridx}" '
           f'heightRelTo="ABSOLUTE" protect="0"/>'
           f'<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" '
           f'holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" '
           f'horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
           f'<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
           f'<hp:inMargin left="{PAD}" right="{PAD}" top="{PAD}" bottom="{PAD}"/>'
           f'{"".join(rows)}</hp:tbl>')
section = section.replace(tm.group(1), new_tbl, 1)

data["Contents/header.xml"] = header.encode("utf-8")
data["Contents/section0.xml"] = section.encode("utf-8")
if OUT.exists():
    OUT.unlink()
with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as z:
    for n in names:
        z.writestr(n, data[n])

with open(OUT_DIR / "width_ladder_manifest.tsv", "w", encoding="utf-8") as f:
    f.write("row\tclass\tk\n")
    for r, c, k in manifest:
        f.write(f"{r}\t{c}\t{k}\n")
print(f"OK {OUT} rows={ridx} ({len(CLASSES)} classes x {K_VALUES})")
