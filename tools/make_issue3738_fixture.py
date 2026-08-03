# -*- coding: utf-8 -*-
"""#3738 회귀 표본 생성 — 한 문단에 자리차지 표 + 자리차지 글상자.

`samples/tac-host-spacing.hwpx` 를 껍데기로 쓰고 `Contents/section0.xml` 만 갈아 끼운다.
호스트 문단 하나에 자리차지 표(줄 30600 HU)와 자리차지 글상자(줄 6600 HU)를 넣고, 그 뒤에
본문 18줄을 둔다 — 두 줄 높이 차가 쪽 경계를 넘도록 맞춘 값이다.

저장소 루트에서 실행한다:  python tools/make_issue3738_fixture.py
"""
import zipfile
import os

TPL = "samples/tac-host-spacing.hwpx"
OUT = "samples/issue3738/tac_sibling_shape_line_advance.hwpx"

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

# 자리차지 표: 선언 30034 + 바깥여백 283*2 = 30600 HU (= ls[0] vertsize)
TBL = ('<hp:tbl id="100" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" '
       'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" '
       'rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
       '<hp:sz width="40000" widthRelTo="ABSOLUTE" height="30034" heightRelTo="ABSOLUTE" '
       'protect="0"/>'
       '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" '
       'holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" '
       'vertOffset="0" horzOffset="0"/>'
       '<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
       '<hp:inMargin left="141" right="141" top="141" bottom="141"/>'
       '<hp:tr><hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" '
       'borderFillIDRef="2">'
       '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" '
       'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" '
       'hasNumRef="0">'
       '<hp:p id="2" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
       '<hp:run charPrIDRef="0"><hp:t>TABLE</hp:t></hp:run>'
       '<hp:linesegarray><hp:lineseg textpos="0" vertpos="14000" vertsize="1000" '
       'textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="39000" '
       'flags="393216"/></hp:linesegarray></hp:p>'
       '</hp:subList><hp:cellAddr colAddr="0" rowAddr="0"/>'
       '<hp:cellSpan colSpan="1" rowSpan="1"/>'
       '<hp:cellSz width="40000" height="30034"/>'
       '<hp:cellMargin left="141" right="141" top="141" bottom="141"/>'
       '</hp:tc></hp:tr></hp:tbl>')

# 자리차지 글상자(사각형): 높이 6600 HU (= ls[1] vertsize)
RECT = ('<hp:rect id="200" zOrder="1" numberingType="NONE" textWrap="TOP_AND_BOTTOM" '
        'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" href="" groupLevel="0" '
        'instid="200" ratio="0">'
        '<hp:sz width="40000" widthRelTo="ABSOLUTE" height="6600" heightRelTo="ABSOLUTE" '
        'protect="0"/>'
        '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" '
        'holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" '
        'horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
        '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
        '<hp:offset x="0" y="0"/><hp:orgSz width="40000" height="6600"/>'
        '<hp:curSz width="40000" height="6600"/>'
        '<hp:flip horizontal="0" vertical="0"/>'
        '<hp:rotationInfo angle="0" centerX="20000" centerY="3300" rotateimage="1"/>'
        '<hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '<hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '<hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo>'
        '<hp:lineShape color="#000000" width="33" style="SOLID" endCap="FLAT" '
        'headStyle="NORMAL" tailStyle="NORMAL" headfill="1" tailfill="1" '
        'headSz="SMALL_SMALL" tailSz="SMALL_SMALL" outlineStyle="NORMAL" alpha="0"/>'
        '<hc:fillBrush><hc:winBrush faceColor="#FFFFFF" hatchColor="#000000" alpha="0"/>'
        '</hc:fillBrush>'
        '<hp:shadow type="NONE" color="#B2B2B2" offsetX="0" offsetY="0" alpha="0"/>'
        '<hp:drawText lastWidth="40000" name="" editable="0">'
        '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" '
        'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" '
        'hasTextRef="0" hasNumRef="0">'
        '<hp:p id="3" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        '<hp:run charPrIDRef="0"><hp:t>SHAPE</hp:t></hp:run>'
        '<hp:linesegarray><hp:lineseg textpos="0" vertpos="0" vertsize="1000" '
        'textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="39000" '
        'flags="393216"/></hp:linesegarray></hp:p></hp:subList>'
        '<hp:textMargin left="283" right="283" top="283" bottom="283"/></hp:drawText>'
        '<hp:pt0 x="0" y="0"/><hp:pt1 x="40000" y="0"/><hp:pt2 x="40000" y="6600"/>'
        '<hp:pt3 x="0" y="6600"/>'
        '</hp:rect>')

# 호스트 문단: 자리차지 개체 2개, 줄 2개.
#   ls[0] 표   vertpos 0     vertsize 30600 spacing 600 → 416.0px
#   ls[1] 도형 vertpos 31200 vertsize  6600 spacing 600 →  96.0px
HOST = ('<hp:p id="1" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        '<hp:run charPrIDRef="0">' + SECPR + TBL + RECT + '</hp:run>'
        '<hp:linesegarray>'
        '<hp:lineseg textpos="0" vertpos="0" vertsize="30600" textheight="30600" '
        'baseline="26010" spacing="600" horzpos="0" horzsize="48188" flags="393216"/>'
        '<hp:lineseg textpos="1" vertpos="31200" vertsize="6600" textheight="6600" '
        'baseline="5610" spacing="600" horzpos="0" horzsize="48188" flags="393216"/>'
        '</hp:linesegarray></hp:p>')

N = 18
BODY = []
for k in range(N):
    vp = 37800 + k * 1600
    BODY.append(
        '<hp:p id="%d" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
        '<hp:run charPrIDRef="0"><hp:t>본문 %02d 자리차지 '
        '개체 뒤 문단</hp:t></hp:run>'
        '<hp:linesegarray><hp:lineseg textpos="0" vertpos="%d" vertsize="1000" '
        'textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="48188" '
        'flags="393216"/></hp:linesegarray></hp:p>' % (10 + k, k + 1, vp))

sec = HDR + HOST + "".join(BODY) + "\n</hs:sec>\n"

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
