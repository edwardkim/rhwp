#!/usr/bin/env python3
"""gen_band_host_spacing.py <tac_adjacent-saved.hwpx> <out.hwpx> <filler_lines> (#8525)

한/글 2020 이 저장한 `samples/issue8507/tac_adjacent-saved.hwpx` 에서 표 A host 문단(글자 없음,
저장 줄 간격 600HU)의 표 A 앞에 쪽 아래 정렬 자리차지 틀을 끼우고, 문서 끝에 1600HU 글줄을 더한다.
틀: 본문 폭 1x1(바깥 여백 0, 안 여백 0, 칸 hasMargin=0, 선언 7593HU) 안에 글자처럼 취급 1x3 표(7593HU).
발신명의 틀과 같은 꼴 — 틀 칸에 단순 글자가 아니라 표가 들어 있어 빈 자리표시 틀로 보지 않는다."""
import sys, zipfile
src, dst, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
SUB = ('<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
       'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">')
def para(pid, inner, lseg):
    return (f'<hp:p id="{pid}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{inner}'
            f'<hp:linesegarray>{lseg}</hp:linesegarray></hp:p>')
def seg(h, w, sp=600):
    return (f'<hp:lineseg textpos="0" vertpos="0" vertsize="{h}" textheight="{h}" baseline="{int(h * 0.85)}" '
            f'spacing="{sp}" horzpos="0" horzsize="{w}" flags="393216"/>')
def tc(col, w, h, inner, margin):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">{SUB}{inner}'
            f'</hp:subList><hp:cellAddr colAddr="{col}" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="{margin}" right="{margin}" top="{margin}" bottom="{margin}"/></hp:tc>')
widths = [7946, 25898, 8190]          # 합 42034 ≤ 본문 폭 42520
inner_cells = ''.join(tc(i, w, 7593, para(7100 + i, '<hp:run charPrIDRef="0"/>', seg(1000, w - 282)), 141)
                      for i, w in enumerate(widths))
inner = ('<hp:tbl id="7001" zOrder="7" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
         'dropcapstyle="None" pageBreak="CELL" repeatHeader="1" rowCnt="1" colCnt="3" cellSpacing="0" borderFillIDRef="3" noAdjust="0">'
         f'<hp:sz width="{sum(widths)}" widthRelTo="ABSOLUTE" height="7593" heightRelTo="ABSOLUTE" protect="0"/>'
         '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
         'horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
         '<hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="141" right="141" top="141" bottom="141"/>'
         f'<hp:tr>{inner_cells}</hp:tr></hp:tbl>')
outer_cell = tc(0, 42520, 1282, para(7000, f'<hp:run charPrIDRef="0">{inner}<hp:t/></hp:run>', seg(7593, 42518, 720)), 141)
frame = ('<hp:tbl id="7000" zOrder="6" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
         'dropcapstyle="None" pageBreak="CELL" repeatHeader="1" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="3" noAdjust="0">'
         '<hp:sz width="42520" widthRelTo="ABSOLUTE" height="7593" heightRelTo="ABSOLUTE" protect="0"/>'
         '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="1" vertRelTo="PAGE" '
         'horzRelTo="COLUMN" vertAlign="BOTTOM" horzAlign="CENTER" vertOffset="0" horzOffset="0"/>'
         '<hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="0" right="0" top="0" bottom="0"/>'
         f'<hp:tr>{outer_cell}</hp:tr></hp:tbl>')
z = zipfile.ZipFile(src); s = z.read('Contents/section0.xml').decode()
i = s.index('<hp:tbl ')                     # 첫 표 = 표 A
s = s[:i] + frame + s[i:]
fill = ''.join(f'<hp:p id="{6000 + k}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
               f'<hp:run charPrIDRef="0"><hp:t>채움 {k}</hp:t></hp:run></hp:p>' for k in range(1, n + 1))
s = s.replace('</hs:sec>', fill + '</hs:sec>')
with zipfile.ZipFile(dst, 'w') as zo:
    for info in z.infolist():
        raw = s.encode() if info.filename == 'Contents/section0.xml' else z.read(info.filename)
        zo.writestr(info, raw, compress_type=zipfile.ZIP_STORED if info.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
