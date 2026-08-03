# -*- coding: utf-8 -*-
"""#3798 회귀 표본 생성 — 쪽 끝에서 말미 줄간격이 한도보다 큰 문단.

`samples/tac-host-spacing.hwpx` 를 껍데기로 쓰고 `Contents/section0.xml` 만 갈아 끼운다.

쪽 마지막 문단의 적합 판정은 말미 줄간격을 뺀 높이로 한다(Task #359). 그 트림에
한도가 없어서, 말미 줄간격이 큰 문단은 쪽을 그만큼 넘겨도 현재 쪽에 얹힌다.

    앞 채움 28개  각 32.0px  ->  누적 896.0px   (본문 933.6px)
    경계 문단     줄 16.0 + 말미 간격 40.0 = 56.0px
                  트림 후 16.0 -> 896.0 + 16.0 = 912.0 <= 933.6  이라 들어간다
                  실제로는 896.0 + 56.0 = 952.0 으로 18.4px 넘친다

한도 6px 을 두면 적합 높이가 50.0 이 되어 946.0 > 933.6 이므로 다음 쪽으로 간다.

저장소 루트에서 실행한다:  python tools/make_issue3798_fixture.py
"""
import zipfile
import os

TPL = "samples/tac-host-spacing.hwpx"
OUT = "samples/issue3798/page_end_trailing_spill.hwpx"

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

COLDEF = ('<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" sameSz="1" '
          'sameGap="0"><hp:colSz width="48188" gap="0"/></hp:colPr></hp:ctrl>')

STEP = 2400   # 줄 간격 HU = 32.0px
LH = 1200     # 줄 높이 HU = 16.0px


def seg(vpos, spacing=1200):
    return ('<hp:linesegarray><hp:lineseg textpos="0" vertpos="%d" vertsize="%d" '
            'textheight="%d" baseline="%d" spacing="%d" horzpos="0" horzsize="48188" '
            'flags="393216"/></hp:linesegarray>'
            % (vpos, LH, LH, int(LH * 0.85), spacing))


def para(pid, text, vpos, extra="", secpr=False, spacing=1200):
    return ('<hp:p id="%d" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" '
            'merged="0"><hp:run charPrIDRef="0">%s%s<hp:t>%s</hp:t></hp:run>%s</hp:p>'
            % (pid, SECPR if secpr else "", extra, text, seg(vpos, spacing)))


paras = []
pid = 1
FILL = 28          # 앞 채움 문단 수 — 누적 28 * 32.0 = 896.0px
BIG_SPACING = 3000  # 말미 줄간격 40.0px (한도 6px 보다 훨씬 크다)
vpos = 0
for k in range(FILL):
    paras.append(para(pid, "앞채움%02d" % (k + 1), vpos, secpr=(k == 0)))
    pid += 1
    vpos += STEP
# 경계 문단 — 줄 16.0px + 말미 간격 40.0px. 트림하면 들어가고 전량이면 넘친다.
paras.append(para(pid, "경계문단-말미간격40px", vpos, spacing=BIG_SPACING))
pid += 1
vpos += LH + BIG_SPACING
# 뒤따르는 본문 — 경계 문단이 문서 마지막이 아니어야 저장 page-last 예외를 안 탄다.
for k in range(6):
    paras.append(para(pid, "다음쪽본문%02d" % (k + 1), vpos))
    pid += 1
    vpos += STEP

sec = HDR + "".join(paras) + "\n</hs:sec>\n"

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
