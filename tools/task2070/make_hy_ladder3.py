#!/usr/bin/env python
"""#2070 한양신명조 사다리 v3 — 7스크립트 폰트 등록 + 실문자열 + 셀명 캐럿워크.

v1/v2 결함 수정: 숫자/공백/기호는 LATIN/SYMBOL 스크립트라 HANGUL 단일
fontface 목록으로는 한양신명조가 적용되지 않았다. 7개 lang 목록 전부에 등록.
셀 폭 = 실문서 조문대비표 셀과 동일(24016, pad 141). tc name 으로 캐럿 마크.
"""
import re
import sys
import zipfile
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(__file__).resolve().parents[2]
SKEL = ROOT / "samples" / "tac-host-spacing.hwpx"
OUT = ROOT / "output" / "poc" / "task2070" / "hy_ladder3.hwpx"

CELL_W, CELL_H, PAD = 24016, 288, 141

P21 = "  8. 복합개발사업의 시행으로 용도가 폐지되는 기반시설의 조서ㆍ도면 및 그 기반시설에 대한 둘 이상의 감정평가법인등의 감정평가서와 새로 설치할 기반시설의 조서ㆍ도면 및 그 설치비용 계산서"
P27 = "  14. 현물출자시점 등을 포함한 구체적 사업일정(법 제14조제1항제6호에 따른 위탁관리 부동산투자회사가 사업시행자인 경우에 한한다)"

with zipfile.ZipFile(SKEL) as z:
    names = z.namelist()
    data = {n: z.read(n) for n in names}
header = data["Contents/header.xml"].decode("utf-8")
section = data["Contents/section0.xml"].decode("utf-8")

# 1) fontfaces: 7개 lang × (함초롬바탕, 한양신명조)
langs = ["HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER"]
ff = '<hh:fontfaces itemCnt="7">' + "".join(
    f'<hh:fontface lang="{lg}" fontCnt="2">'
    f'<hh:font id="0" face="함초롬바탕" type="TTF" isEmbedded="0"/>'
    f'<hh:font id="1" face="한양신명조" type="TTF" isEmbedded="0"/>'
    f'</hh:fontface>' for lg in langs) + '</hh:fontfaces>'
header = re.sub(r'<hh:fontfaces.*?</hh:fontfaces>', ff, header, flags=re.S)

# 2) charPr: 14pt 한양신명조 전 스크립트
char_ids = [int(m) for m in re.findall(r'<hh:charPr id="(\d+)"', header)]
cid = max(char_ids) + 1
m = re.search(r'(<hh:charPr id="0".*?</hh:charPr>)', header, re.S)
cp = m.group(1).replace('id="0"', f'id="{cid}"', 1).replace('height="1000"', 'height="1400"', 1)
cp = re.sub(r'<hh:fontRef [^/]*/>',
            '<hh:fontRef hangul="1" latin="1" hanja="1" japanese="1" other="1" symbol="1" user="1"/>',
            cp, count=1)
cm = re.search(r'<hh:charProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:charProperties itemCnt="{cm.group(1)}"',
                        f'<hh:charProperties itemCnt="{int(cm.group(1)) + 1}"')
header = re.sub(r'(</hh:charProperties>)', cp + r"\1", header, count=1)

# 3) paraPr 3종: A intent=0 / B intent=-2440 / C intent=-4880, ls=100%
para_ids = [int(m) for m in re.findall(r'<hh:paraPr id="(\d+)"', header)]
base = re.search(r'(<hh:paraPr id="0".*?</hh:paraPr>)', header, re.S).group(1)
pids = {}
add = ""
for tag, intent in (("A", 0), ("B", -2440), ("C", -4880)):
    pid = max(para_ids) + 1 + len(pids)
    pids[tag] = pid
    pr = base.replace('id="0"', f'id="{pid}"', 1)
    pr = pr.replace('<hc:intent value="0" unit="HWPUNIT"/>',
                    f'<hc:intent value="{intent}" unit="HWPUNIT"/>', 1)
    pr = re.sub(r'<hh:lineSpacing[^/]*/>',
                '<hh:lineSpacing type="PERCENT" value="100" unit="HWPUNIT"/>', pr, count=1)
    add += pr
cm = re.search(r'<hh:paraProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:paraProperties itemCnt="{cm.group(1)}"',
                        f'<hh:paraProperties itemCnt="{int(cm.group(1)) + 3}"')
header = re.sub(r'(</hh:paraProperties>)', add + r"\1", header, count=1)

pidc = [100]
rows = []
labels = []


def para(text: str, pid: int) -> str:
    pidc[0] += 1
    return (f'<hp:p id="{pidc[0]}" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" '
            f'columnBreak="0" merged="0"><hp:run charPrIDRef="{cid}"><hp:t>{text}</hp:t>'
            f'</hp:run></hp:p>')


def cell(label: str, text: str, ptag: str = "A") -> None:
    i = len(rows)
    labels.append(label)
    rows.append(
        f'<hp:tr><hp:tc name="R{i}" header="0" hasMargin="0" protect="0" editable="0" '
        f'dirty="0" borderFillIDRef="2">'
        f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" '
        f'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" '
        f'hasTextRef="0" hasNumRef="0">{para(text, pids[ptag])}</hp:subList>'
        f'<hp:cellAddr colAddr="0" rowAddr="{i}"/>'
        f'<hp:cellSpan colSpan="1" rowSpan="1"/>'
        f'<hp:cellSz width="{CELL_W}" height="{CELL_H}"/>'
        f'<hp:cellMargin left="{PAD}" right="{PAD}" top="{PAD}" bottom="{PAD}"/>'
        f'</hp:tc></hp:tr>')


# 실문자열 (마크 워크 대상)
cell("P21_A", P21, "A")
cell("P21_B", P21, "B")
cell("P21_C", P21, "C")
cell("P27_A", P27, "A")
cell("P27_B", P27, "B")
cell("P27_C", P27, "C")
# 메트릭 (inner = 24016-282 = 23734HU, em=1400 → 전각 16.95)
for n in (16, 17, 18):
    cell(f"B가x{n}", "가" * n)
for mcnt in (26, 28, 30, 32):
    cell(f"H1_1x{mcnt}", "1" * mcnt)
for mcnt in (24, 28, 32):
    cell(f"H4_가2+spx{mcnt}+가", "가가" + " " * mcnt + "가")
for mcnt in (2, 4, 6):
    cell(f"H7_가16+.x{mcnt}", "가" * 16 + "." * mcnt)
for mcnt in (8, 9, 10):
    cell(f"H6_가8+ㆍx{mcnt}", "가" * 8 + "ㆍ" * mcnt)
for k in (1, 2, 3):
    cell(f"E_가16+spx{k}", "가" * 16 + " " * k)
cell("H2_가16+1x1", "가" * 16 + "1")
for mcnt in (14, 15, 16):
    cell(f"C가8+1x{mcnt}", "가" * 8 + "1" * mcnt)

tm = re.search(r'(<hp:tbl .*?</hp:tbl>)', section, re.S)
new_tbl = (f'<hp:tbl id="100" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" '
           f'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" '
           f'repeatHeader="0" rowCnt="{len(rows)}" colCnt="1" cellSpacing="0" '
           f'borderFillIDRef="2" noAdjust="0">'
           f'<hp:sz width="{CELL_W}" widthRelTo="ABSOLUTE" height="{CELL_H * len(rows)}" '
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
with open(ROOT / "output" / "poc" / "task2070" / "hy_ladder3_labels.txt", "w",
          encoding="utf-8") as f:
    f.write("\n".join(f"{i}\t{l}" for i, l in enumerate(labels)))
print(f"OK {OUT} rows={len(rows)} innerW={CELL_W - 2 * PAD}HU")
