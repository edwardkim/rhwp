# -*- coding: utf-8 -*-
"""#3751 회귀 표본 생성 — 쪽을 넘나들어 사다리가 중간에서 리셋되는 문단.

`samples/tac-host-spacing.hwpx` 를 껍데기로 쓰고 `Contents/section0.xml` 만 갈아 끼운다.

앞 문단들로 쪽을 거의 채운 뒤, **34줄짜리 문단**을 둔다. 그 문단의 저장 사다리는
한글이 쪽을 넘기며 적은 그대로다 — 앞 8줄은 쪽 하단까지 오르고, 9번째 줄에서 0 으로
리셋된 뒤 다시 오른다.

    ls[0..7]   vpos 48000 → 62400   (1쪽)
    ls[8..33]  vpos     0 → 62400   (2쪽)

끝점만 비교하면 62400 > 48000 이라 "증가" 로 보이지만, span(=끝−처음)은 208px 로
실제 높이 1088px 의 1/5 이다. 적합 판정이 그 span 을 쓰면 문단이 안 쪼개지고 통째로
1쪽에 얹혀 쪽 밖으로 나간다.

저장소 루트에서 실행한다:  python tools/make_issue3751_fixture.py
"""
import zipfile
import os

TPL = "samples/tac-host-spacing.hwpx"
OUT = "samples/issue3751/vpos_reset_midparagraph_fit.hwpx"

HDR = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
       '<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" '
       'xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" '
       'xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">\n')

SECPR = ('<hp:secPr id="" textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" '
         'tabStopVal="4000" tabStopUnit="HWPUNIT" outlineShapeIDRef="1" memoShapeIDRef="0" '
         'textVerticalWidthHead="0" masterPageCnt="0">'
         '<hp:grid lineGrid="0" charGrid="0" wonggojiFormat="0" strikeContinue="0"/>'
         '<hp:startNum pageStartsOn="BOTH" page="0" pic="0" tbl="0" equation="0"/>'
         '<hp:visibility hideFirstHeader="0" hideFirstFooter="0" hideFirstMasterPage="0" '
         'border="SHOW_ALL" fill="SHOW_ALL" hideFirstPageNum="0" hideFirstEmptyLine="0" '
         'showLineNumber="0"/>'
         '<hp:lineNumberShape restartType="0" countBy="0" distance="0" startNumber="0"/>'
         '<hp:pagePr landscape="WIDELY" width="59528" height="84188" gutterType="LEFT_ONLY">'
         '<hp:margin header="2834" footer="2834" gutter="0" left="5669" right="5669" '
         'top="4251" bottom="4251"/>'
         '</hp:pagePr>'
         '<hp:footNotePr><hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" '
         'suffixChar=")" supscript="0"/>'
         '<hp:noteLine length="-1" type="SOLID" width="0.12 mm" color="#000000"/>'
         '<hp:noteSpacing betweenNotes="283" belowLine="567" aboveLine="850"/>'
         '<hp:numbering type="CONTINUOUS" newNum="1"/>'
         '<hp:placement place="EACH_COLUMN" beneathText="0"/></hp:footNotePr>'
         '<hp:endNotePr><hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" '
         'suffixChar=")" supscript="0"/>'
         '<hp:noteLine length="14692" type="SOLID" width="0.12 mm" color="#000000"/>'
         '<hp:noteSpacing betweenNotes="0" belowLine="567" aboveLine="850"/>'
         '<hp:numbering type="CONTINUOUS" newNum="1"/>'
         '<hp:placement place="END_OF_DOCUMENT" beneathText="0"/></hp:endNotePr>'
         '<hp:pageBorderFill type="BOTH" borderFillIDRef="1" textBorder="PAPER" '
         'headerInside="0" footerInside="0" fillArea="PAPER">'
         '<hp:offset left="1417" right="1417" top="1417" bottom="1417"/></hp:pageBorderFill>'
         '</hp:secPr>'
         '<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" sameSz="1" '
         'sameGap="0"/></hp:ctrl>')

STEP = 2400          # 줄 간격 HU (= 32.0px)
LH = 1200            # 줄 높이 HU (= 16.0px)
LEAD = 20            # 앞 채움 문단 수
SPLIT_AT = 8         # 긴 문단이 쪽을 넘는 줄 인덱스
TAIL_LINES = 26      # 넘어간 뒤의 줄 수


def lineseg(vpos, ts=0):
    return ('<hp:lineseg textpos="%d" vertpos="%d" vertsize="%d" textheight="%d" '
            'baseline="%d" spacing="1200" horzpos="0" horzsize="42520" flags="393216"/>'
            % (ts, vpos, LH, LH, int(LH * 0.85)))


def filler(pid, k, vpos):
    return ('<hp:p id="%d" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" '
            'merged="0"><hp:run charPrIDRef="0"><hp:t>채움 %02d</hp:t></hp:run>'
            '<hp:linesegarray>%s</hp:linesegarray></hp:p>' % (pid, k, lineseg(vpos)))


# 앞 채움: 0 .. (LEAD-1)*STEP
lead_paras = []
vp = 0
for k in range(LEAD):
    lead_paras.append(filler(100 + k, k + 1, vp))
    vp += STEP

# 긴 문단: 앞 8줄은 vp 부터 오르고, 9번째 줄에서 0 으로 리셋 후 다시 오른다.
long_segs = []
ts = 0
v = vp
for i in range(SPLIT_AT):
    long_segs.append(lineseg(v, ts))
    v += STEP
    ts += 40
v2 = 0
for i in range(TAIL_LINES):
    long_segs.append(lineseg(v2, ts))
    v2 += STEP
    ts += 40

body = "".join(
    "쪽을 넘나드는 긴 문단 %02d 저장 사다리가 중간에서 리셋된다. " % (i + 1)
    for i in range(34)
)
LONG = ('<hp:p id="200" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" '
        'merged="0"><hp:run charPrIDRef="0"><hp:t>%s</hp:t></hp:run>'
        '<hp:linesegarray>%s</hp:linesegarray></hp:p>' % (body, "".join(long_segs)))

TAILMARK = ('<hp:p id="300" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" '
            'merged="0"><hp:run charPrIDRef="0"><hp:t>끝표식</hp:t></hp:run>'
            '<hp:linesegarray>%s</hp:linesegarray></hp:p>' % lineseg(v2))

# 첫 문단에 secPr 을 얹는다.
first = lead_paras[0].replace('<hp:run charPrIDRef="0">',
                              '<hp:run charPrIDRef="0">' + SECPR, 1)
sec = HDR + first + "".join(lead_paras[1:]) + LONG + TAILMARK + "\n</hs:sec>\n"

src = zipfile.ZipFile(TPL)
os.makedirs(os.path.dirname(OUT), exist_ok=True)
with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as z:
    for info in src.infolist():
        data = src.read(info.filename)
        if info.filename == "Contents/section0.xml":
            data = sec.encode("utf-8")
        if info.filename == "mimetype":
            z.writestr(zipfile.ZipInfo("mimetype"), data, zipfile.ZIP_STORED)
        else:
            z.writestr(info.filename, data)
print("wrote", OUT, os.path.getsize(OUT), "bytes")
