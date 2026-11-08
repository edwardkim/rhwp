#!/usr/bin/env python3
"""[#7679] 글줄 뒤 같은 문단 둘째 줄의 글자처럼 취급(TAC) 자리차지 표 — 공개 합성 입력.

  python3 gen_tac_after_text.py samples/issue2527_empty_linesegs.hwpx tac-after-text.hwpx

쪽마다 사례 하나. 모든 문단에 linesegarray 없음(한/글이 저장할 때 줄을 만든다).
  T1  앞 간격 2000HU 문단: 「가. 협의 대상」 + 줄 바꿈 + 3×4 TAC 표(바깥 여백 283HU)
  T2  같은 형상, 앞 간격 0 (대조군)
  T3  앞 간격 2000HU, 표 앞 글이 두 줄
"""
import re
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
out = sys.argv[2]
hdr = src.read('Contents/header.xml').decode('utf8')
pp0 = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', hdr, re.S).group(0)
pp1 = pp0.replace('id="0"', 'id="1"').replace('<hc:prev value="0"', '<hc:prev value="2000"')
cnt = int(re.search(r'<hh:paraProperties itemCnt="(\d+)">', hdr).group(1))
hdr = hdr.replace(f'<hh:paraProperties itemCnt="{cnt}">' + pp0,
                  f'<hh:paraProperties itemCnt="{cnt + 1}">' + pp0 + pp1.replace('id="1"', f'id="{cnt}"'))
PP_SB = cnt
sec = src.read('Contents/section0.xml').decode('utf8')
head = sec[:sec.find('<hp:bookmark')]

_id = [500]


def nid():
    _id[0] += 1
    return _id[0]


def para(runs, pp=0, page_break=False):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{pp}" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{runs}</hp:p>\n')


def t(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def cell(c, r, w, h, text):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            + para(t(text)) +
            f'</hp:subList><hp:cellAddr colAddr="{c}" rowAddr="{r}"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
            f'<hp:cellSz width="{w}" height="{h}"/><hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>')


def tac_table(rows=3, cols=4, row_h=1800):
    w = 44000 // cols
    trs = ''.join('<hp:tr>' + ''.join(cell(c, r, w, row_h, f'행{r + 1} 열{c + 1}') for c in range(cols)) + '</hp:tr>'
                  for r in range(rows))
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{rows}" colCnt="{cols}" '
            f'cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
            f'<hp:sz width="{w * cols}" widthRelTo="ABSOLUTE" height="{row_h * rows}" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
            'vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
            '<hp:inMargin left="510" right="510" top="141" bottom="141"/>'
            + trs + '</hp:tbl>')


def host(text_lines, pp):
    runs = ''
    for line in text_lines:
        runs += f'<hp:run charPrIDRef="0"><hp:t>{line}</hp:t><hp:lineBreak/></hp:run>'
    runs += '<hp:run charPrIDRef="0">' + tac_table() + '<hp:t/></hp:run>'
    return para(runs, pp=pp)


body = [head + '</hp:run>' + t('#7679 글줄 뒤 TAC 표 합성 입력') + '</hp:p>\n']
cases = [
    ('T1 앞 간격 2000, 글 한 줄 뒤 표', ['  가. 협의 대상 시유지'], PP_SB),
    ('T2 앞 간격 0, 글 한 줄 뒤 표', ['  가. 협의 대상 시유지'], 0),
    ('T3 앞 간격 2000, 글 두 줄 뒤 표', ['  가. 협의 대상 시유지', '    (첨부 목록 참조)'], PP_SB),
]
for i, (name, lines, pp) in enumerate(cases):
    body.append(para(t(f'사례 {name}'), page_break=i > 0))
    body.append(para(t('표 앞 문단이다.')))
    body.append(host(lines, pp))
    body.append(para(t('  나. 회신내용'), pp=PP_SB))
    body.append(para(t('표 뒤 문단이다.')))
sec_new = ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 18, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
