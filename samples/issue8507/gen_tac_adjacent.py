#!/usr/bin/env python3
"""글자처럼 취급한 3행 표(행 2512HU, 합 7536HU) 두 개가 붙은 HWPX 를 만든다 (#8507).

gen_tac_adjacent.py <서식 HWPX> <출력 HWPX>
- 앞 문단, 표 A, 표 B, 「B 뒤 첫 줄」, 「끝」. 두 표의 선언 높이는 행 합 7536HU 다.
- 서식 HWPX 는 머리·구역 정의만 빌린다(samples/issue1549_multipositive_float_tables.hwpx).
이 출력을 한/글 2020 이 HWP → HWPX 로 저장한 것이 일관된 저장본(tac_adjacent-saved.hwpx)이다."""
import re, sys, zipfile
_id = [1000]
def nid():
    _id[0] += 1; return _id[0]
def run(t): return f'<hp:run charPrIDRef="0"><hp:t>{t}</hp:t></hp:run>'
def para(t, lead=""):
    return f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{lead}{run(t)}</hp:p>'
def cell(t, c, r, w, h):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{para(t)}</hp:subList>'
            f'<hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')
def tac(label, rows, h, sz):
    trs = "".join(f'<hp:tr>{cell(label+str(r)+"가",0,r,10000,h)}{cell(label+str(r)+"나",1,r,30000,h)}</hp:tr>' for r in range(rows))
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
            f'dropcapstyle="None" pageBreak="NONE" repeatHeader="0" rowCnt="{rows}" colCnt="2" cellSpacing="0" borderFillIDRef="3" noAdjust="0">'
            f'<hp:sz width="40000" widthRelTo="ABSOLUTE" height="{sz}" heightRelTo="ABSOLUTE" protect="0"/>'
            f'<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
            f'horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            f'<hp:outMargin left="138" right="138" top="138" bottom="138"/><hp:inMargin left="141" right="141" top="141" bottom="141"/>{trs}</hp:tbl>')
tpl, out = sys.argv[1], sys.argv[2]
z = zipfile.ZipFile(tpl); files = [(i.filename, z.read(i.filename)) for i in z.infolist()]; data = dict(files)
sec = data["Contents/section0.xml"].decode()
head = sec[:sec.index("<hp:p ")]
inner = re.search(r'<hp:p [^>]*><hp:run charPrIDRef="0">(<hp:secPr.*?</hp:secPr>.*?)</hp:run>', sec, re.S).group(1)
lead = f'<hp:run charPrIDRef="0">{inner}</hp:run>'
def tpara(label, sz):
    return f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">{tac(label,3,2512,sz)}</hp:run></hp:p>'
paras = [para("앞 문단", lead), tpara("A", 7536), tpara("B", 7536), para("B 뒤 첫 줄"), para("끝")]
body = (head + "".join(paras) + "</hs:sec>").encode()
with zipfile.ZipFile(out, "w") as zo:
    for n, raw in files:
        if n.startswith("Preview/"): continue
        if n == "Contents/section0.xml": raw = body
        zo.writestr(n, raw, compress_type=zipfile.ZIP_STORED if n == "mimetype" else zipfile.ZIP_DEFLATED)
