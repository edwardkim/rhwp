#!/usr/bin/env python
"""#2169 줄바꿈 지점 사다리 — 80168 pi=271 r4c2 표본 재현.

표본 조건(80168 hwpx에서 추출, output 스크래치 target_2169.json):
  - 한양중고딕(hangul)/HCI Poppy(latin) 11pt(height=1100), 자간 +2%(spacing=2), ratio 100
  - paraPr: JUSTIFY, breakNonLatinWord="KEEP_WORD" (어절 단위!), lineWrap=BREAK
  - 셀 w=34362HU, 표 inMargin 225 → inner 33912HU

사다리 설계 (1열 표, 행=케이스, ls=100%로 행높이→줄수 역산):
  - K{k}: 표본 문장 앞 k글자, breakNonLatinWord=KEEP_WORD (실문서 재현)
  - B{k}: 동일 프리픽스, breakNonLatinWord=BREAK_WORD (글자 단위 대조군)
  - Kfull/Bfull: 전문(194자) — 총 줄수 대조
한글이 2줄로 넘어가는 임계 k(K vs B)와 rhwp cut_rows 임계를 대조하여
줄바꿈 지점 차이를 문자 단위로 특정한다 (#2169 과제 1).
"""
import re
import sys
import zipfile
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(r"C:\Users\planet\rhwp")
SKEL = ROOT / "samples" / "tac-host-spacing.hwpx"
OUTDIR = ROOT / "output" / "poc" / "task2169"
OUT = OUTDIR / "linebreak_ladder.hwpx"

CELL_W, CELL_H, PAD = 34362, 288, 225

TEXT = (
    "ㅇ 도심복합개발법 제10조제1항은 복합개발계획 입안에 관한 주민 의견청취 "
    "공고가 있는 경우 사업의 진행에 지장이 될 수 있는 건축물의 건축, 공작물의 "
    "설치, 토지의 형질 변경 등 행위에 대하여 시장·군수 등의 허가를 받도록 하고, "
    "같은 조 제2항제3호에서 경작을 위한 토지의 형질 변경 등 사업수행에 지장이 "
    "없는 행위에 대해 시행령에 위임하고 있음"
)
assert len(TEXT) == 194, len(TEXT)

with zipfile.ZipFile(SKEL) as z:
    names = z.namelist()
    data = {n: z.read(n) for n in names}
header = data["Contents/header.xml"].decode("utf-8")
section = data["Contents/section0.xml"].decode("utf-8")

# 1) fontfaces: 7개 lang × (함초롬바탕, 한양중고딕, HCI Poppy)
langs = ["HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER"]
ff = '<hh:fontfaces itemCnt="7">' + "".join(
    f'<hh:fontface lang="{lg}" fontCnt="3">'
    f'<hh:font id="0" face="함초롬바탕" type="TTF" isEmbedded="0"/>'
    f'<hh:font id="1" face="한양중고딕" type="TTF" isEmbedded="0"/>'
    f'<hh:font id="2" face="HCI Poppy" type="TTF" isEmbedded="0"/>'
    f'</hh:fontface>' for lg in langs) + '</hh:fontfaces>'
header = re.sub(r'<hh:fontfaces.*?</hh:fontfaces>', ff, header, flags=re.S)

# 2) charPr: 11pt, hangul=한양중고딕 latin=HCI Poppy, 자간 +2%
char_ids = [int(m) for m in re.findall(r'<hh:charPr id="(\d+)"', header)]
cid = max(char_ids) + 1
m = re.search(r'(<hh:charPr id="0".*?</hh:charPr>)', header, re.S)
cp = m.group(1).replace('id="0"', f'id="{cid}"', 1).replace('height="1000"', 'height="1100"', 1)
cp = re.sub(r'<hh:fontRef [^/]*/>',
            '<hh:fontRef hangul="1" latin="2" hanja="2" japanese="2" other="1" symbol="2" user="1"/>',
            cp, count=1)
cp = re.sub(r'<hh:spacing [^/]*/>',
            '<hh:spacing hangul="2" latin="2" hanja="2" japanese="2" other="2" symbol="2" user="2"/>',
            cp, count=1)
cm = re.search(r'<hh:charProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:charProperties itemCnt="{cm.group(1)}"',
                        f'<hh:charProperties itemCnt="{int(cm.group(1)) + 1}"')
header = re.sub(r'(</hh:charProperties>)', cp + r"\1", header, count=1)

# 3) paraPr 2종: K=KEEP_WORD / B=BREAK_WORD (둘 다 JUSTIFY, ls=100%)
para_ids = [int(m) for m in re.findall(r'<hh:paraPr id="(\d+)"', header)]
base = re.search(r'(<hh:paraPr id="0".*?</hh:paraPr>)', header, re.S).group(1)
pids = {}
add = ""
for tag, brk in (("K", "KEEP_WORD"), ("B", "BREAK_WORD")):
    pid = max(para_ids) + 1 + len(pids)
    pids[tag] = pid
    pr = base.replace('id="0"', f'id="{pid}"', 1)
    pr = re.sub(r'<hh:align [^/]*/>',
                '<hh:align horizontal="JUSTIFY" vertical="BASELINE"/>', pr, count=1)
    pr = re.sub(r'<hh:breakSetting [^/]*/>',
                f'<hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="{brk}" '
                f'widowOrphan="0" keepWithNext="0" keepLines="0" pageBreakBefore="0" '
                f'lineWrap="BREAK"/>', pr, count=1)
    pr = re.sub(r'<hh:lineSpacing[^/]*/>',
                '<hh:lineSpacing type="PERCENT" value="100" unit="HWPUNIT"/>', pr, count=1)
    add += pr
cm = re.search(r'<hh:paraProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:paraProperties itemCnt="{cm.group(1)}"',
                        f'<hh:paraProperties itemCnt="{int(cm.group(1)) + 2}"')
header = re.sub(r'(</hh:paraProperties>)', add + r"\1", header, count=1)

pidc = [100]
rows = []
labels = []


def para(text: str, pid: int) -> str:
    pidc[0] += 1
    return (f'<hp:p id="{pidc[0]}" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" '
            f'columnBreak="0" merged="0"><hp:run charPrIDRef="{cid}"><hp:t>{text}</hp:t>'
            f'</hp:run></hp:p>')


def cell(label: str, text: str, ptag: str) -> None:
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


# 프리픽스 사다리: 1줄→2줄 임계 구간 k=22..46
for k in range(22, 47):
    cell(f"K{k}", TEXT[:k], "K")
for k in range(22, 47):
    cell(f"B{k}", TEXT[:k], "B")
cell("Kfull", TEXT, "K")
cell("Bfull", TEXT, "B")

tm = re.search(r'(<hp:tbl .*?</hp:tbl>)', section, re.S)
new_tbl = (f'<hp:tbl id="100" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" '
           f'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" '
           f'repeatHeader="0" rowCnt="{len(rows)}" colCnt="1" cellSpacing="0" '
           f'borderFillIDRef="2" noAdjust="0">'
           f'<hp:sz width="{CELL_W}" widthRelTo="ABSOLUTE" height="{CELL_H * len(rows)}" '
           f'heightRelTo="ABSOLUTE" protect="0"/>'
           f'<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" '
           f'holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" '
           f'horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
           f'<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
           f'<hp:inMargin left="{PAD}" right="{PAD}" top="{PAD}" bottom="{PAD}"/>'
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
with open(OUTDIR / "linebreak_ladder_labels.txt", "w", encoding="utf-8") as f:
    f.write("\n".join(f"{i}\t{l}" for i, l in enumerate(labels)))
print(f"OK {OUT} rows={len(rows)} innerW={CELL_W - 2 * PAD}HU")
