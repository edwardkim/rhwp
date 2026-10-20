# 저장 줄의 낱말 꼬리를 한/글이 다시 나누는가 — 문단 머리(번호) 유무·저장 줄 폭 대조 (#8105).
# 1342000(한/글 2014 저장 HWP) 의 칸 문단은 번호 문단만 한/글 2020 이 저장 줄 꼬리를 앞 줄에 붙였다
# (번호 있음 39/40 붙임, 번호 없음 «…제|정]» «…일|부개정]» 은 저장 줄 그대로).
# 인자: 틀(samples/hwpx/basic-table-01.hwpx) 출력 [저장 제품 appVersion, 예 «9, 1, 1, 3933»]
# 블록마다 1×1 표(칸 폭 30000, 여백 510/141)에 문단 하나. 본문은 자연 폭으로 칸 폭의 절반쯤이고,
# 저장 줄은 마지막 낱말의 끝 음절 «기» 만 다음 줄로 보낸다(한/글이 다시 나누면 한 줄).
#   D2 번호(«^1.») · 저장 2줄    H2 번호(«-») · 저장 2줄    H1 번호(«-») · 저장 1줄
#   P2 번호 없음 · 저장 2줄      L2 번호 없음, 본문에 «- » 를 직접 씀 · 저장 2줄
#   P0 번호 없음 · 저장 줄 없음(직접 조판 대조군)
#   D2f·H2f·P2f 위 D2·H2·P2 에 1342000 과 같은 줄 flags(번호 문단 첫 줄 0x260000, 이어지는 줄 0x160000)
#   H2a «-» 번호 · 첫 줄만 0x260000    H2b «-» 번호 · 이어지는 줄만 0x160000
#   H2w·P2w·L2w 위 H2·P2·L2 의 저장 줄 폭을 칸 안쪽 폭보다 1HU 좁게    P2W 번호 없음 · 저장 줄 폭 1HU 넓게
#
# 한/글 2024(13.0.0.3901) 결과 — 같은 폴더의 PDF:
#   numbered_stale_tail.hwp (`rhwp convert` 로 HWP5 변환) → numbered_stale_tail-hwp-2024.pdf:
#     번호(«-»·«1.») 를 그린 문단은 모두 한 줄(D2·H2·D2f·H2f·H2a·H2b·H2w). 번호 없는 문단은 폭이 맞으면
#     저장 줄 두 줄(P2·L2·P2f), 폭이 1HU 라도 다르면 한 줄(P2w·L2w·P2W). 줄 flags 는 무관.
#   numbered_stale_tail.hwpx → numbered_stale_tail-hwpx-2024.pdf: 한/글이 여기서 덧붙인 «-» 번호를
#     그리지 않았다(H2 글줄 시작이 번호 없는 P2 와 같은 x) — «-» 번호 행은 증거가 아니다. «1.» 번호(D2)와
#     폭 불일치 행은 HWP5 와 같다. appVersion 9.1.1(2014)·11.0(2020) 둘 다 같은 결과.
import sys, zipfile, re
tpl, out = sys.argv[1], sys.argv[2]
APP = sys.argv[3] if len(sys.argv) > 3 else None
z = zipfile.ZipFile(tpl)
files = {n: z.read(n) for n in z.namelist()}

head = files['Contents/header.xml'].decode()
num1 = re.search(r'<hh:numbering id="1".*?</hh:numbering>', head, re.S).group(0)
num2 = re.sub(r'(<hh:paraHead [^>]*level="1"[^>]*>)\^1\.(</hh:paraHead>)', r'\1-\2', num1.replace('id="1"', 'id="2"', 1))
assert num2 != num1.replace('id="1"', 'id="2"', 1)
head = head.replace(num1, num1 + num2, 1)
head = re.sub(r'(<hh:numberings itemCnt=")(\d+)', lambda m: m.group(1) + str(int(m.group(2)) + 1), head, 1)
pp0 = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', head, re.S).group(0)
ids = [int(x) for x in re.findall(r'<hh:paraPr id="(\d+)"', head)]
DIGIT, DASH = max(ids) + 1, max(ids) + 2
def numbered(pid, num):
    return pp0.replace('id="0"', 'id="%d"' % pid, 1).replace(
        '<hh:heading type="NONE" idRef="0" level="0"/>', '<hh:heading type="NUMBER" idRef="%d" level="0"/>' % num)
head = head.replace(pp0, pp0 + numbered(DIGIT, 1) + numbered(DASH, 2), 1)
head = re.sub(r'(<hh:paraProperties itemCnt=")(\d+)', lambda m: m.group(1) + str(int(m.group(2)) + 2), head, 1)
files['Contents/header.xml'] = head.encode()

s = files['Contents/section0.xml'].decode()
i = s.index('<hp:run charPrIDRef="0"><hp:tbl')
j = s.index('</hp:tbl><hp:t/></hp:run>', i) + len('</hp:tbl><hp:t/></hp:run>')
s = s[:i] + s[j:]
first_end = s.find('</hp:p>') + len('</hp:p>')

W = 30000
SW = W - 2 * 510
TEXT = '여러 가지 차별 현상 탐색하기'
TAIL = len(TEXT) - 1  # 끝 음절 «기»

# 줄 flags: 0x60000 = 줄의 첫·끝 조각, 0x200000 = 문단 머리(번호) 적용, 0x100000 = 들여쓰기 적용.
# 1342000 번호 문단의 저장 줄은 첫 줄 0x260000 · 이어지는 줄 0x160000 이다.
def seg(pos, vpos, flags=0x60000, sw=SW):
    return ('<hp:lineseg textpos="%d" vertpos="%d" vertsize="1000" textheight="1000" baseline="850" '
            'spacing="600" horzpos="0" horzsize="%d" flags="%d"/>' % (pos, vpos, sw, flags))

def para(pid, text, segs=None):
    lines = '' if segs is None else '<hp:linesegarray>%s</hp:linesegarray>' % ''.join(segs)
    return ('<hp:p id="0" paraPrIDRef="%d" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            '<hp:run charPrIDRef="0"><hp:t>%s</hp:t></hp:run>%s</hp:p>' % (pid, text, lines))

def table(inner):
    tc = ('<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
          '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" '
          'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">%s</hp:subList>'
          '<hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
          '<hp:cellSz width="%d" height="1282"/><hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>' % (inner, W))
    return ('<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">'
            '<hp:tbl id="1" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
            'dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="1" colCnt="1" cellSpacing="0" '
            'borderFillIDRef="3" noAdjust="0"><hp:sz width="%d" widthRelTo="ABSOLUTE" height="1282" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
            'horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="283" right="283" top="283" bottom="283"/><hp:inMargin left="510" right="510" top="141" bottom="141"/>'
            '<hp:tr>%s</hp:tr></hp:tbl><hp:t/></hp:run></hp:p>' % (W, tc))

two = [seg(0, 0), seg(TAIL, 1600)]
two_num = [seg(0, 0, 0x260000), seg(TAIL, 1600, 0x160000)]
blocks = [
    ('D2', para(DIGIT, TEXT, two)),
    ('H2', para(DASH, TEXT, two)),
    ('H1', para(DASH, TEXT, [seg(0, 0)])),
    ('P2', para(0, TEXT, two)),
    ('L2', para(0, '- ' + TEXT, [seg(0, 0), seg(TAIL + 2, 1600)])),
    ('P0', para(0, TEXT)),
    ('D2f', para(DIGIT, TEXT, two_num)),
    ('H2f', para(DASH, TEXT, two_num)),
    ('P2f', para(0, TEXT, [seg(0, 0), seg(TAIL, 1600, 0x160000)])),
    ('H2a', para(DASH, TEXT, [seg(0, 0, 0x260000), seg(TAIL, 1600)])),
    ('H2b', para(DASH, TEXT, [seg(0, 0), seg(TAIL, 1600, 0x160000)])),
    ('H2w', para(DASH, TEXT, [seg(0, 0, sw=SW - 1), seg(TAIL, 1600, sw=SW - 1)])),
    ('P2w', para(0, TEXT, [seg(0, 0, sw=SW - 1), seg(TAIL, 1600, sw=SW - 1)])),
    ('L2w', para(0, '- ' + TEXT, [seg(0, 0, sw=SW - 1), seg(TAIL + 2, 1600, sw=SW - 1)])),
    ('P2W', para(0, TEXT, [seg(0, 0, sw=SW + 1), seg(TAIL, 1600, sw=SW + 1)])),
]
body = ''.join(para(0, label) + table(cell) for label, cell in blocks)
files['Contents/section0.xml'] = (s[:first_end] + body + '</hs:sec>').encode()
if APP:
    files['version.xml'] = re.sub(r'appVersion="[^"]*"', 'appVersion="%s WIN32LEWindows_10"' % APP,
                                  files['version.xml'].decode()).encode()
with zipfile.ZipFile(out, 'w') as o:
    for n in z.namelist():
        o.writestr(n, files[n], compress_type=zipfile.ZIP_STORED if n == 'mimetype' else zipfile.ZIP_DEFLATED)
