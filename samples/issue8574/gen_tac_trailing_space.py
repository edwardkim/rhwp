#!/usr/bin/env python3
"""저장 줄 없는 문단: 줄 머리의 글자처럼 취급 표 + 문단 끝 공백 한 칸 HWPX 를 만든다.

gen_tac_trailing_space.py <서식 HWPX> <출력 HWPX> [표 폭 HU]
- 앞 문단, [표 + " "], 「표 뒤 첫 줄」, 「끝」. 본문 폭 42520HU, 표 폭 기본 42400HU 라
  표 + 공백이 줄 폭을 넘는다. 비교군은 공백 없는 같은 문단(--no-space).
- 서식 HWPX 는 머리·구역 정의만 빌린다(samples/issue1549_multipositive_float_tables.hwpx)."""
import re, sys, zipfile
args = [a for a in sys.argv[1:] if not a.startswith("--")]
no_space = "--no-space" in sys.argv
tpl, out = args[0], args[1]
width = int(args[2]) if len(args) > 2 else 42400
_id = [1000]
def nid():
    _id[0] += 1; return _id[0]
def run(t): return f'<hp:run charPrIDRef="0"><hp:t>{t}</hp:t></hp:run>'
def para(t, lead=""):
    return f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{lead}{run(t)}</hp:p>'
def cell(t, w, h):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{para(t)}</hp:subList>'
            f'<hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')
def tac(w, h):
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
            f'dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="3" noAdjust="0">'
            f'<hp:sz width="{w}" widthRelTo="ABSOLUTE" height="{h}" heightRelTo="ABSOLUTE" protect="0"/>'
            f'<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
            f'horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            f'<hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="141" right="141" top="141" bottom="141"/>'
            f'<hp:tr>{cell("표 안 글", w, h)}</hp:tr></hp:tbl>')
z = zipfile.ZipFile(tpl); files = [(i.filename, z.read(i.filename)) for i in z.infolist()]; data = dict(files)
sec = data["Contents/section0.xml"].decode()
head = sec[:sec.index("<hp:p ")]
inner = re.search(r'<hp:p [^>]*><hp:run charPrIDRef="0">(<hp:secPr.*?</hp:secPr>.*?)</hp:run>', sec, re.S).group(1)
lead = f'<hp:run charPrIDRef="0">{inner}</hp:run>'
tail = "" if no_space else "<hp:t> </hp:t>"
host = (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        f'<hp:run charPrIDRef="0">{tac(width, 8000)}{tail}</hp:run></hp:p>')
paras = [para("앞 문단", lead), host, para("표 뒤 첫 줄"), para("끝")]
body = (head + "".join(paras) + "</hs:sec>").encode()
with zipfile.ZipFile(out, "w") as zo:
    for n, raw in files:
        if n.startswith("Preview/"): continue
        if n == "Contents/section0.xml": raw = body
        zo.writestr(n, raw, compress_type=zipfile.ZIP_STORED if n == "mimetype" else zipfile.ZIP_DEFLATED)
