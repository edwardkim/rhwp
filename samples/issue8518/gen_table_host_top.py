#!/usr/bin/env python3
"""쪽 끝에 host 줄이 들지 않아 새 쪽 첫 문단이 되는 빈 표 host 문단 — 합성 HWPX 입력(#8518).

  python gen_table_host_top.py <out.hwpx> ROWS PRE SB   # 회귀 입력: 20 28 830

모든 문단의 문단 위 간격을 SB(HWPUNIT)로 둔다. 한 줄 문단 PRE 개 뒤에 빈 문단이 ROWS 행
표(행 1800HU, 셀 단위 나눔, 글자처럼 취급 아님, 문단 기준 자리차지)를 품고, 그 뒤에 `AFTER`
문단이 온다. 저장 줄은 쓰지 않는다 — 한/글로 다시 저장한 HWP 의 저장 줄이 기준이다.

`table_host_top.hwp` 는 `20 28 830` 입력을 한/글 2022(12.0.0.4547)로 다시 저장한 HWP 다.
앞 문단 28개가 1쪽을 채워 빈 host 문단(표 host)의 줄이 1쪽에 들지 않으므로, 한/글은 host 를
2쪽 첫 문단으로 두고 저장 vpos 를 문단 위 간격(830HU)으로 적는다. 기준 PDF
`pdf/issue8518_table_host_top-2020.pdf` 는 그 HWP 의 한/글 2022 Print(2쪽)다.
"""
import sys, zipfile
out, ROWS, PRE, SB = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
src = zipfile.ZipFile('samples/issue2527_empty_linesegs.hwpx')
parts = [(i.filename, src.read(i.filename)) for i in src.infolist()]
hdr = dict(parts)['Contents/header.xml'].decode('utf-8', 'surrogateescape')
i0 = hdr.find('<hh:paraPr id="0"')
j0 = hdr.find('</hh:paraPr>', i0)
block = hdr[i0:j0].replace('<hc:prev value="0" unit="HWPUNIT"/>', f'<hc:prev value="{SB}" unit="HWPUNIT"/>')
hdr = hdr[:i0] + block + hdr[j0:]
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
       '<hp:outMargin left="141" right="141" top="141" bottom="141"/><hp:inMargin left="141" right="141" top="141" bottom="141"/>'
       + ''.join(f'<hp:tr>{cell(r)}</hp:tr>' for r in range(ROWS)) + '</hp:tbl>')
x = head + '</hp:run><hp:run charPrIDRef="0"><hp:t>table host at page top</hp:t></hp:run></hp:p>\n'
for i in range(PRE):
    x += f'<hp:p id="{600+i}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>PRE {i+1:02d}</hp:t></hp:run></hp:p>\n'
x += f'<hp:p id="901" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">{tbl}<hp:t/></hp:run></hp:p>\n'
x += '<hp:p id="902" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>AFTER</hp:t></hp:run></hp:p>\n</hs:sec>'
with zipfile.ZipFile(out, 'w') as d:
    for f, data in parts:
        info = zipfile.ZipInfo(f, date_time=(2026, 11, 3, 0, 0, 0))
        info.compress_type = zipfile.ZIP_STORED if f == 'mimetype' else zipfile.ZIP_DEFLATED
        if f == 'Contents/section0.xml': data = x.encode('utf-8', 'surrogateescape')
        elif f == 'Contents/header.xml': data = hdr.encode('utf-8', 'surrogateescape')
        d.writestr(info, data)
