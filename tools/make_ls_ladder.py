#!/usr/bin/env python
"""#2150 통제 프로브 - ls% 사다리 HWPX 생성 (A:1줄 / B:장문 다중줄 / C:2문단).

선언 셀높이 극소(288HU)의 1열 표에 ls% 사다리 문단을 넣는다. 한글이 열면
행높이 = 한글 fresh 콘텐츠 높이 + 패딩 -> tools/probe_ls_ladder.py 로 직독.

확정 공식 (2026-07-10): 한글 NO_LS fresh 셀 콘텐츠 높이 =
sum 줄박스(em=max_fs) + 연속 줄 사이 gap (ls-100%)x fs (문단 경계 포함),
셀 마지막 줄 뒤 trailing 없음. 1줄 셀은 ls% 완전 무시(em).
"""
import re
import sys
import zipfile
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(r"C:\Users\planet\rhwp")
SKEL = ROOT / "samples" / "tac-host-spacing.hwpx"
OUT = ROOT / "output" / "poc" / "task2150" / "ls_ladder3.hwpx"

LS = [100, 130, 160, 200, 230, 300]
CELL_W, CELL_H, PAD = 20000, 288, 141
LONG = "가나다라마바사아자차카타파하" * 4  # 56자 — 다중 줄 래핑 유도

with zipfile.ZipFile(SKEL) as z:
    names = z.namelist()
    data = {n: z.read(n) for n in names}

header = data["Contents/header.xml"].decode("utf-8")
section = data["Contents/section0.xml"].decode("utf-8")

para_ids = [int(m) for m in re.findall(r'<hh:paraPr id="(\d+)"', header)]
base_id = max(para_ids) + 1
m = re.search(r'(<hh:paraPr id="0".*?</hh:paraPr>)', header, re.S)
assert m
skel_pr = m.group(1)
new_prs, id_map = [], {}
for i, ls in enumerate(LS):
    pid = base_id + i
    id_map[ls] = pid
    pr = re.sub(r'id="0"', f'id="{pid}"', skel_pr, count=1)
    pr, n = re.subn(r'<hh:lineSpacing[^/]*/>',
                    f'<hh:lineSpacing type="PERCENT" value="{ls}" unit="HWPUNIT"/>', pr)
    assert n == 1, f"ls={ls}"
    new_prs.append(pr)
cm = re.search(r'<hh:paraProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:paraProperties itemCnt="{cm.group(1)}"',
                        f'<hh:paraProperties itemCnt="{int(cm.group(1)) + len(LS)}"')
header = re.sub(r'(</hh:paraProperties>)', "".join(new_prs) + r"\1", header, count=1)

rows = []
ridx = 0
pid_ctr = 100


def para(ls: int, text: str) -> str:
    global pid_ctr
    pid_ctr += 1
    return (f'<hp:p id="{pid_ctr}" paraPrIDRef="{id_map[ls]}" styleIDRef="0" '
            f'pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run></hp:p>')


def cell(paras_xml: str) -> str:
    global ridx
    r = (f'<hp:tr><hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" '
         f'dirty="0" borderFillIDRef="2">'
         f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" '
         f'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" '
         f'hasTextRef="0" hasNumRef="0">{paras_xml}</hp:subList>'
         f'<hp:cellAddr colAddr="0" rowAddr="{ridx}"/>'
         f'<hp:cellSpan colSpan="1" rowSpan="1"/>'
         f'<hp:cellSz width="{CELL_W}" height="{CELL_H}"/>'
         f'<hp:cellMargin left="{PAD}" right="{PAD}" top="{PAD}" bottom="{PAD}"/>'
         f'</hp:tc></hp:tr>')
    ridx += 1
    return r


for ls in LS:
    rows.append(cell(para(ls, f"단일{ls}")))
for ls in LS:
    rows.append(cell(para(ls, LONG)))
for ls in LS:
    rows.append(cell(para(ls, "문단일") + para(ls, "문단이")))

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
print(f"OK {OUT} rows={ridx} (A:1줄 B:장문 C:2문단 x ls {LS})")
