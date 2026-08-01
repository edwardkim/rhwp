# -*- coding: utf-8 -*-
"""#3765 회귀 표본 생성 — zone 전환 지점에서 사다리가 실소비를 크게 밑도는 문서.

`samples/tac-host-spacing.hwpx` 를 껍데기로 쓰고 `Contents/section0.xml` 만 갈아 끼운다.

한글이 이미 쪽을 끊은 자리를 재현한다. 앞 문단들은 쪽을 거의 채우도록 쌓되, 사다리는
중간에서 0 으로 리셋시켜 **zone 전환 직전 문단의 vpos 가 작게** 보이도록 한다.

    pi 0..24   vpos 0 → 가득          (rhwp 가 한 쪽에 쌓는다)
    pi 25..27  vpos 400 → 작음        (한글은 여기서 쪽을 끊었다는 뜻)
    pi 28      단정의(1단) 컨트롤 = zone 전환
    pi 29..    새 zone 본문

수정 전에는 zone 전환 가드가 pi=27 의 작은 vpos 를 직전 zone 높이로 읽어 "여유 있다" 로
오판하고, 두 zone 이 한 쪽에 겹쳐 얹힌다.

저장소 루트에서 실행한다:  python tools/make_issue3765_fixture.py
"""
import zipfile
import os

TPL = "samples/tac-host-spacing.hwpx"
OUT = "samples/issue3765/zone_switch_ladder_understates_page.hwpx"

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


def seg(vpos):
    return ('<hp:linesegarray><hp:lineseg textpos="0" vertpos="%d" vertsize="%d" '
            'textheight="%d" baseline="%d" spacing="1200" horzpos="0" horzsize="48188" '
            'flags="393216"/></hp:linesegarray>' % (vpos, LH, LH, int(LH * 0.85)))


def para(pid, text, vpos, extra="", secpr=False):
    return ('<hp:p id="%d" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" '
            'merged="0"><hp:run charPrIDRef="0">%s%s<hp:t>%s</hp:t></hp:run>%s</hp:p>'
            % (pid, SECPR if secpr else "", extra, text, seg(vpos)))


paras = []
pid = 1
# ① 쪽을 가득 채우는 앞 문단 25개 (0 → 25*32 = 800px)
for k in range(25):
    paras.append(para(pid, "앞 채움 %02d" % (k + 1), k * STEP, secpr=(k == 0)))
    pid += 1
# ② 사다리가 되감긴 문단 3개 — 한글은 여기서 쪽을 끊었다는 뜻.
#    실제 문서(2990099 pi=202)는 정확히 0 이 아니라 400 HU 에서 다시 시작한다.
#    0 이면 rhwp 의 vpos-reset 가드가 정상 발동해 쪽을 끊어 버려 이 경로를 안 탄다.
RESET_BASE = 400
for k in range(3):
    paras.append(para(pid, "리셋 뒤 %02d" % (k + 1), RESET_BASE + k * STEP))
    pid += 1
# ③ zone 전환 (단정의 컨트롤) — 직전 문단 vpos 는 작다
paras.append(para(pid, "새 구역 머리", RESET_BASE + 3 * STEP, extra=COLDEF))
pid += 1
# ④ 새 zone 본문 20줄
for k in range(20):
    paras.append(para(pid, "새 구역 본문 %02d" % (k + 1), RESET_BASE + (4 + k) * STEP))
    pid += 1

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
