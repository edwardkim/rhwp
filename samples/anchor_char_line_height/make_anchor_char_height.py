#!/usr/bin/env python3
"""공개 합성 입력 생성기 (재현용).

형상: linesegarray 가 없는 본문 문단 + 12pt 글자모양을 가진 비-글자취급 자리차지 표 기준 문자
+ 같은 줄의 8pt 텍스트. 뼈대(secPr·colPr·header·패키지 파일)는 공개 샘플
samples/issue2527_empty_linesegs.hwpx 에서 가져오고 rhwp 출처 표식(META-INF/rhwp-hwp5-origin)은
넣지 않는다. 글자모양은 id0=10pt, id1=12pt, id2=8pt 셋으로 바꾼다. 모든 문단에 linesegarray 가 없다.

사용: python3 samples/anchor_char_line_height/make_anchor_char_height.py <출력.hwpx>  (저장소 루트에서)
"""
import zipfile, re, sys
src = zipfile.ZipFile('samples/issue2527_empty_linesegs.hwpx')
out = sys.argv[1]
hdr = src.read('Contents/header.xml').decode('utf8')
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', hdr, re.S).group(0)
cps = ''.join(cp0.replace('id="0"', f'id="{i}"').replace('height="1000"', f'height="{h}"') for i, h in ((0, 1000), (1, 1200), (2, 800)))
hdr = re.sub(r'<hh:charProperties itemCnt="1">.*?</hh:charProperties>', f'<hh:charProperties itemCnt="3">{cps}</hh:charProperties>', hdr, flags=re.S)
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]   # 첫 문단의 secPr·colPr 까지
def para(pid, runs):
    return f'<hp:p id="{pid}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{runs}</hp:p>\n'
def t(cp, text):
    return f'<hp:run charPrIDRef="{cp}"><hp:t>{text}</hp:t></hp:run>'
def cell(c, r, w, text):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + para(1000 + r * 10 + c, t(0, text)) +
            f'</hp:subList><hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="2000"/><hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>')
def table(tid, rows):
    w = 48000 // 3
    trs = ''.join('<hp:tr>' + ''.join(cell(c, r, w, rows[r][c]) for c in range(3)) + '</hp:tr>' for r in range(len(rows)))
    return (f'<hp:tbl id="{tid}" zOrder="{tid}" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{len(rows)}" colCnt="3" cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w*3}" widthRelTo="ABSOLUTE" height="{2000*len(rows)}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="140" right="140" top="140" bottom="852"/><hp:inMargin left="510" right="510" top="141" bottom="141"/>'
            + trs + '</hp:tbl>')
# 첫 문단: secPr/colPr 를 담은 run 을 닫고 텍스트를 붙인다
first = head + '</hp:run>' + t(1, '가. 기준 문자가 큰 자리차지 표 (linesegarray 없음)') + '</hp:p>\n'
host1 = para(10, f'<hp:run charPrIDRef="1">{table(201, [["구분", "금액", "비고"], ["가", "1,000", "-"]])}</hp:run>' + t(2, '      ※ 표 아래로 밀리는 8pt 글줄 — 기준 문자는 12pt 글자모양이다.'))
mid = para(11, '<hp:run charPrIDRef="1"/>') + para(12, t(1, '나. 표 뒤 첫 문단 위치가 기준 문자 줄 높이를 따른다'))
host2 = para(13, f'<hp:run charPrIDRef="1">{table(202, [["항목", "값", "단위"], ["다", "2,000", "원"]])}</hp:run>' + t(2, '      ※ 두 번째 표의 8pt 글줄'))
tail = ''.join(para(20 + i, t(1, s)) for i, s in enumerate([
    '다. 이 문단들은 두 표 뒤의 흐름 위치를 드러낸다.',
    '라. 저장 줄 정보가 없으므로 줄 높이는 글자모양에서 계산된다.',
    '마. 끝.']))
sec_new = first + host1 + mid + host2 + tail + '</hs:sec>'
def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 3, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)
with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
