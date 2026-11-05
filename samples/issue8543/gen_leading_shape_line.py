#!/usr/bin/env python3
"""gen_leading_shape_line.py <tac_adjacent-saved.hwpx> <out.hwpx>

한/글 2020 저장본 `samples/issue8507/tac_adjacent-saved.hwpx` 에서 표 A host 문단을 2쪽 머리로 넘기고
(pageBreak), 표 A 앞에 글자처럼 취급 글뒤로 사각형(3105HU)을 넣어 저장 줄을 두 줄로 만든다:
줄 0(ts 0, vpos 0) = 사각형, 줄 1(ts 8, vpos 4005) = 표 A. 뒤 최상위 문단의 저장 vpos 는 2쪽 기준으로 다시 쓴다."""
import re, sys, zipfile
src, dst = sys.argv[1], sys.argv[2]
z = zipfile.ZipFile(src); s = z.read('Contents/section0.xml').decode()
H, LS, W = 3105, 900, 40000
rect = ('<hp:rect id="8001" zOrder="1" numberingType="PICTURE" textWrap="BEHIND_TEXT" textFlow="BOTH_SIDES" lock="0" '
        'dropcapstyle="None" href="" groupLevel="0" instid="8001" ratio="0"><hp:offset x="0" y="0"/>'
        f'<hp:orgSz width="{W}" height="{H}"/><hp:curSz width="{W}" height="{H}"/><hp:flip horizontal="0" vertical="0"/>'
        f'<hp:rotationInfo angle="0" centerX="{W // 2}" centerY="{H // 2}" rotateimage="1"/><hp:renderingInfo>'
        '<hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/><hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '<hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo>'
        '<hp:lineShape color="#000000" width="33" style="SOLID" endCap="FLAT" headStyle="NORMAL" tailStyle="NORMAL" headfill="1" '
        'tailfill="1" headSz="MEDIUM_MEDIUM" tailSz="MEDIUM_MEDIUM" outlineStyle="NORMAL" alpha="0"/>'
        '<hp:shadow type="NONE" color="#B2B2B2" offsetX="0" offsetY="0" alpha="0"/>'
        f'<hc:pt0 x="0" y="0"/><hc:pt1 x="{W}" y="0"/><hc:pt2 x="{W}" y="{H}"/><hc:pt3 x="0" y="{H}"/>'
        f'<hp:sz width="{W}" widthRelTo="ABSOLUTE" height="{H}" heightRelTo="ABSOLUTE" protect="0"/>'
        '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="1" holdAnchorAndSO="0" vertRelTo="PARA" '
        'horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
        '<hp:outMargin left="0" right="0" top="0" bottom="0"/></hp:rect>')
# 표 A host 문단(두 번째 최상위 문단): 쪽 나누기 + 표 앞 사각형 + 저장 줄 두 줄
paras = list(re.finditer(r'<hp:p id="(\d+)"[^>]*>', s))
host = paras[1]
assert 'pageBreak="0"' in host.group(0)
s = s[:host.start()] + host.group(0).replace('pageBreak="0"', 'pageBreak="1"') + s[host.end():]
i = s.index('<hp:tbl ')
s = s[:i] + rect + s[i:]
old_seg = re.search(r'<hp:lineseg textpos="0" vertpos="1600" vertsize="7812"[^>]*/>', s).group(0)
seg0 = (f'<hp:lineseg textpos="0" vertpos="0" vertsize="{H}" textheight="{H}" baseline="{int(H * 0.85)}" '
        f'spacing="{LS}" horzpos="0" horzsize="42520" flags="393216"/>')
seg1 = old_seg.replace('textpos="0" vertpos="1600"', f'textpos="8" vertpos="{H + LS}"')
s = s.replace(old_seg, seg0 + seg1, 1)
# 뒤 최상위 문단 저장 vpos: 원래 1600 기준 → 2쪽 기준(표 A 줄 끝 = H+LS+7812+600)
shift = (H + LS) - 1600
def fix(m):
    v = int(m.group(1))
    return f'vertpos="{v + shift}"' if v >= 10012 else m.group(0)
out, depth, pos = [], 0, 0
for m in re.finditer(r'<hp:subList|</hp:subList>|<hp:linesegarray>.*?</hp:linesegarray>', s, re.S):
    out.append(s[pos:m.start()]); pos = m.end(); t = m.group(0)
    if t.startswith('<hp:subList'): depth += 1
    elif t.startswith('</hp:subList'): depth -= 1
    elif depth == 0: t = re.sub(r'vertpos="(\d+)"', fix, t)
    out.append(t)
out.append(s[pos:]); s = ''.join(out)
with zipfile.ZipFile(dst, 'w') as zo:
    for info in z.infolist():
        raw = s.encode() if info.filename == 'Contents/section0.xml' else z.read(info.filename)
        zo.writestr(info, raw, compress_type=zipfile.ZIP_STORED if info.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
