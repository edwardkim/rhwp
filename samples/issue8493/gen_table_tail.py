#!/usr/bin/env python3
"""표가 쪽 중간에서 끝난 뒤 이어지는 여러 줄 문단 — 합성 HWPX 입력(#8493).

  python gen_table_tail.py <out.hwpx> 31 40 30

인자: 표 행 수, 문단 줄 수(대략), 표 앞 한 줄 문단 수.
앞 문단 30개 뒤에서 시작한 31행 표(행 1800HU, 셀 단위 나눔)가 1쪽 아래에서 2쪽 중간까지
이어지고, 바로 뒤에 50줄쯤의 문단과 `END` 문단이 온다. 저장 줄은 쓰지 않는다.

`table_tail.hwp` 는 이 입력을 한/글 2022(12.0.0.4547)로 다시 저장한 HWP 다 — 저장 줄
사다리(표 host vpos 49600, 뒤 문단 첫 줄 36000 = 2쪽 좌표)는 한/글이 쓴 값이다.
기준 PDF `pdf/issue8493_table_tail-2020.pdf` 는 그 HWP 의 한/글 2022 Print(3쪽)다.
"""
import sys, zipfile
out, ROWS, LINES, PRE = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
src = zipfile.ZipFile(r'C:\Users\planet\rhwp-wt-mp2\samples\issue2527_empty_linesegs.hwpx')
parts = [(i.filename, src.read(i.filename)) for i in src.infolist()]
sec = dict(parts)['Contents/section0.xml'].decode('utf-8', 'surrogateescape')
head = sec[:sec.find('<hp:bookmark')]
W = 48000; RH = 1800
def cell(r):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'<hp:p id="{5000+r}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>ROW {r+1:02d}</hp:t></hp:run></hp:p>'
            f'</hp:subList><hp:cellAddr colAddr="0" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="{W}" height="{RH}"/><hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')
tbl = (f'<hp:tbl id="100" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{ROWS}" colCnt="1" cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
       f'<hp:sz width="{W}" widthRelTo="ABSOLUTE" height="{RH*ROWS}" heightRelTo="ABSOLUTE" protect="0"/>'
       '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
       '<hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="141" right="141" top="141" bottom="141"/>'
       + ''.join(f'<hp:tr>{cell(r)}</hp:tr>' for r in range(ROWS)) + '</hp:tbl>')
x = head + '</hp:run><hp:run charPrIDRef="0"><hp:t>table then paragraph</hp:t></hp:run></hp:p>\n'
for i in range(PRE):
    x += f'<hp:p id="{600+i}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>PRE {i+1:02d}</hp:t></hp:run></hp:p>' + chr(10)
x += f'<hp:p id="901" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">{tbl}</hp:run></hp:p>\n'
body = ' '.join(f'LINE{i+1:02d} ' + 'filler words ' * 9 for i in range(LINES))
x += f'<hp:p id="902" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>{body}</hp:t></hp:run></hp:p>\n'
x += '<hp:p id="903" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>END</hp:t></hp:run></hp:p>\n</hs:sec>'
with zipfile.ZipFile(out, 'w') as d:
    for f, data in parts:
        info = zipfile.ZipInfo(f, date_time=(2026, 11, 3, 0, 0, 0))
        info.compress_type = zipfile.ZIP_STORED if f == 'mimetype' else zipfile.ZIP_DEFLATED
        d.writestr(info, x.encode('utf-8', 'surrogateescape') if f == 'Contents/section0.xml' else data)
