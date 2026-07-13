#!/usr/bin/env python
"""#2097 선언-이하 압축 사다리 — 한글이 선언 셀높이보다 작게 렌더하는 규칙 실측.

1741000-202100087 관측: 선언 3709/3651/6024HU 셀(1줄 콘텐츠, valign=Center,
pad=141)을 한글이 각각 ~-6.5/-10.6px 압축. 선언·콘텐츠·pad·valign 성분 분해용.

1열 표, 행=케이스. 각 행 선언 cellSz height 를 명시. COM get_row_height 로
한글 실측 → 선언 유지/압축/성장 판별.
"""
import re
import sys
import zipfile
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(r"C:\Users\planet\rhwp")
SKEL = ROOT / "samples" / "tac-host-spacing.hwpx"
OUTDIR = ROOT / "output" / "poc" / "task2097"
OUT = OUTDIR / "cellcompress_ladder.hwpx"

CELL_W = 40315
TEXT1 = "중복검토 방법 : 정책연구관리시스템(PRISM)"
TEXT2 = "유사ㆍ중복 여부: [  ] 있다    [V] 없다"

with zipfile.ZipFile(SKEL) as z:
    names = z.namelist()
    data = {n: z.read(n) for n in names}
header = data["Contents/header.xml"].decode("utf-8")
section = data["Contents/section0.xml"].decode("utf-8")

# charPr: 10pt (height=1000, 스켈레톤 id=0 그대로 사용)
CID = 0

pidc = [100]
rows = []
labels = []


def para(text: str) -> str:
    pidc[0] += 1
    return (f'<hp:p id="{pidc[0]}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
            f'columnBreak="0" merged="0"><hp:run charPrIDRef="{CID}"><hp:t>{text}</hp:t>'
            f'</hp:run></hp:p>')


def cell(label: str, text: str, h: int, valign: str = "CENTER", pad: int = 141) -> None:
    i = len(rows)
    labels.append(f"{label} (선언 {h}HU={h/75:.1f}px)")
    rows.append(
        f'<hp:tr><hp:tc name="R{i}" header="0" hasMargin="0" protect="0" editable="0" '
        f'dirty="0" borderFillIDRef="2">'
        f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="{valign}" '
        f'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" '
        f'hasTextRef="0" hasNumRef="0">{para(text)}</hp:subList>'
        f'<hp:cellAddr colAddr="0" rowAddr="{i}"/>'
        f'<hp:cellSpan colSpan="1" rowSpan="1"/>'
        f'<hp:cellSz width="{CELL_W}" height="{h}"/>'
        f'<hp:cellMargin left="{pad}" right="{pad}" top="{pad}" bottom="{pad}"/>'
        f'</hp:tc></hp:tr>')


# 선언 높이 사다리 (1줄, Center, pad 141)
for h in (284, 1000, 1500, 2000, 3000, 3651, 3709, 6024, 8000):
    cell(f"C_h{h}", TEXT1, h)
# valign Top 대조
for h in (3709, 6024):
    cell(f"T_h{h}", TEXT1, h, valign="TOP")
# pad 확대 대조
cell("C_h3709_pad510", TEXT1, 3709, pad=510)
# 2줄 콘텐츠 대조 (선언 근접)
cell("C_h3709_2L", TEXT1 + " " + TEXT2 + " " + TEXT1, 3709)

tm = re.search(r'(<hp:tbl .*?</hp:tbl>)', section, re.S)
total_h = sum(int(re.search(r'height="(\d+)"', r).group(1)) for r in rows)
new_tbl = (f'<hp:tbl id="100" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" '
           f'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" '
           f'repeatHeader="0" rowCnt="{len(rows)}" colCnt="1" cellSpacing="0" '
           f'borderFillIDRef="2" noAdjust="0">'
           f'<hp:sz width="{CELL_W}" widthRelTo="ABSOLUTE" height="{total_h}" '
           f'heightRelTo="ABSOLUTE" protect="0"/>'
           f'<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" '
           f'holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" '
           f'horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
           f'<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
           f'<hp:inMargin left="141" right="141" top="141" bottom="141"/>'
           f'{"".join(rows)}</hp:tbl>')
section = section.replace(tm.group(1), new_tbl, 1)
data["Contents/header.xml"] = header.encode("utf-8")
data["Contents/section0.xml"] = section.encode("utf-8")
OUTDIR.mkdir(parents=True, exist_ok=True)
if OUT.exists():
    OUT.unlink()
with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as z:
    for n in names:
        z.writestr(n, data[n])
with open(OUTDIR / "cellcompress_labels.txt", "w", encoding="utf-8") as f:
    f.write("\n".join(f"{i}\t{l}" for i, l in enumerate(labels)))
print(f"OK {OUT} rows={len(rows)}")
