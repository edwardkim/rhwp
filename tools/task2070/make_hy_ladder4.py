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
OUT = ROOT / "output" / "poc" / "task2070" / "hy_ladder4.hwpx"

CELL_W, CELL_H, PAD = 24016, 2880, 141

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
header = header.replace("</hh:charProperties>", cp + "</hh:charProperties>", 1)

# 3) paraPr: 실문서 paraPr30 구조 복제 (switch HwpUnitChar) 변형 4종
para_ids = [int(m) for m in re.findall(r'<hh:paraPr id="(\d+)"', header)]
base_id = max(para_ids) + 1
def real_parapr(pid, intent_huc, condense, ls):
    return (
        f'<hh:paraPr id="{pid}" tabPrIDRef="0" condense="{condense}" fontLineHeight="0" '
        f'snapToGrid="1" suppressLineNumbers="0" checked="0">'
        f'<hh:align horizontal="JUSTIFY" vertical="BASELINE"/>'
        f'<hh:heading type="NONE" idRef="0" level="0"/>'
        f'<hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" '
        f'widowOrphan="0" keepWithNext="0" keepLines="0" pageBreakBefore="0" lineWrap="BREAK"/>'
        f'<hh:autoSpacing eAsianEng="0" eAsianNum="0"/>'
        f'<hp:switch><hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar">'
        f'<hh:margin><hc:intent value="{intent_huc}" unit="HWPUNIT"/>'
        f'<hc:left value="0" unit="HWPUNIT"/><hc:right value="0" unit="HWPUNIT"/>'
        f'<hc:prev value="0" unit="HWPUNIT"/><hc:next value="0" unit="HWPUNIT"/></hh:margin>'
        f'<hh:lineSpacing type="PERCENT" value="{ls}" unit="HWPUNIT"/></hp:case>'
        f'<hp:default><hh:margin><hc:intent value="{intent_huc*2}" unit="HWPUNIT"/>'
        f'<hc:left value="0" unit="HWPUNIT"/><hc:right value="0" unit="HWPUNIT"/>'
        f'<hc:prev value="0" unit="HWPUNIT"/><hc:next value="0" unit="HWPUNIT"/></hh:margin>'
        f'<hh:lineSpacing type="PERCENT" value="{ls}" unit="HWPUNIT"/></hp:default></hp:switch>'
        f'<hh:border borderFillIDRef="1" offsetLeft="0" offsetRight="0" offsetTop="0" '
        f'offsetBottom="0" connect="0" ignoreMargin="0"/></hh:paraPr>')
pids = {}
add = ""
for tag, intent, cnd in (("R", -2440, 25), ("N", -2440, 0), ("Z", 0, 25), ("H", -1220, 25)):
    pid = base_id + len(pids)
    pids[tag] = pid
    add += real_parapr(pid, intent, cnd, 100)
cm = re.search(r'<hh:paraProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:paraProperties itemCnt="{cm.group(1)}"',
                        f'<hh:paraProperties itemCnt="{int(cm.group(1)) + 4}"')
header = header.replace("</hh:paraProperties>", add + "</hh:paraProperties>", 1)

# 자간 -6% charPr 추가 (cid2)
cid2 = cid + 1
cp2 = cp.replace(f'id="{cid}"', f'id="{cid2}"', 1)
cp2 = re.sub(r'<hh:spacing [^/]*/>',
             '<hh:spacing hangul="-6" latin="-6" hanja="-6" japanese="-6" other="-6" symbol="-6" user="-6"/>',
             cp2, count=1)
cm = re.search(r'<hh:charProperties itemCnt="(\d+)"', header)
header = header.replace(f'<hh:charProperties itemCnt="{cm.group(1)}"',
                        f'<hh:charProperties itemCnt="{int(cm.group(1)) + 1}"')
header = header.replace("</hh:charProperties>", cp2 + "</hh:charProperties>", 1)

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


def para_runs(pid: int) -> str:
    """P21 실런: [0:2]sp0 [2:74]sp0 [74:92]sp-6 [92:105]sp0."""
    pidc[0] += 1
    return (f'<hp:p id="{pidc[0]}" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" '
            f'columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="{cid}"><hp:t>{P21[0:74]}</hp:t></hp:run>'
            f'<hp:run charPrIDRef="{cid2}"><hp:t>{P21[74:92]}</hp:t></hp:run>'
            f'<hp:run charPrIDRef="{cid}"><hp:t>{P21[92:105]}</hp:t></hp:run></hp:p>')


def cell_xml(label: str, paras_xml: str) -> None:
    i = len(rows)
    labels.append(label)
    rows.append(
        f'<hp:tr><hp:tc name="R{i}" header="0" hasMargin="0" protect="0" editable="0" '
        f'dirty="0" borderFillIDRef="2">'
        f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" '
        f'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" '
        f'hasTextRef="0" hasNumRef="0">{paras_xml}</hp:subList>'
        f'<hp:cellAddr colAddr="0" rowAddr="{i}"/>'
        f'<hp:cellSpan colSpan="1" rowSpan="1"/>'
        f'<hp:cellSz width="{CELL_W}" height="{CELL_H}"/>'
        f'<hp:cellMargin left="{PAD}" right="{PAD}" top="{PAD}" bottom="{PAD}"/>'
        f'</hp:tc></hp:tr>')


cell_xml("P21_R(실복제)", para_runs(pids["R"]))
cell_xml("P21_N(cnd0)", para_runs(pids["N"]))
cell_xml("P21_Z(intent0)", para_runs(pids["Z"]))
cell_xml("P21_H(intent половина)", para_runs(pids["H"]))
cell_xml("P21_R_자간無", para(P21, pids["R"]))

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
if OUT.exists():
    OUT.unlink()
with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as z:
    for n in names:
        z.writestr(n, data[n])
with open(ROOT / "output" / "poc" / "task2070" / "hy_ladder4_labels.txt", "w",
          encoding="utf-8") as f:
    f.write("\n".join(f"{i}\t{l}" for i, l in enumerate(labels)))
print(f"OK {OUT} rows={len(rows)} innerW={CELL_W - 2 * PAD}HU")
