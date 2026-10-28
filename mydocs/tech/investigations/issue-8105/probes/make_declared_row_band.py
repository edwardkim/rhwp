# 쪽 끝에서 나뉘는 행의 선언 높이 합성 (#8105).
# 블록마다 새 쪽: 채움 문단 K 줄 + 3행 표(머리 · 시험 · 다음). 시험 행은 라벨 칸(한 줄)과
# 내용 칸(5 문단)으로, 내용은 쪽 끝 남은 공간에 들지만 선언 높이는 들지 않게 K 를 고른다.
# 변형: V1 라벨 칸만 선언 9366 · V2 V1 + 왼쪽 병합 칸이 세 행을 덮음 · V3 두 칸 모두 9366 ·
# V4 내용 칸만 9366 · V5 V2 + 시험 행 저장 줄 · V6 V5 + 내용 칸 선언 4170(내용보다 작음).
# V10 V1 + 시험 행 저장 줄(병합 칸 없음). 인자: 틀 출력 [변형,..] [선언 HU,..] [K,..] [V8·V9 시험 행 칸 여백 HU] [표 기본 칸 위아래 여백 HU] [칸 글자 모양 id].
# K=20 은 쪽 중간 대조군. V1–V4 는 LineSeg 없음(한/글이 직접 조판).
import sys, zipfile, re
tpl, out = sys.argv[1], sys.argv[2]
DECLS = [int(d) for d in sys.argv[4].split(',')] if len(sys.argv) > 4 else [9366]
DECL = DECLS[0]
MARGIN = int(sys.argv[6]) if len(sys.argv) > 6 else 283
IN_TB = int(sys.argv[7]) if len(sys.argv) > 7 else 141
CELL_CHAR = int(sys.argv[8]) if len(sys.argv) > 8 else 0
KS = [int(k) for k in sys.argv[5].split(',')] if len(sys.argv) > 5 else [20, 33, 34, 35]
WIDTH = 42520
z = zipfile.ZipFile(tpl)
files = {n: z.read(n) for n in z.namelist()}
s = files['Contents/section0.xml'].decode()
# 템플릿 첫 문단의 표를 걷어 낸 뒤 그 문단(구역 설정)만 남긴다.
i = s.index('<hp:run charPrIDRef="0"><hp:tbl')
j = s.index('</hp:tbl><hp:t/></hp:run>', i) + len('</hp:tbl><hp:t/></hp:run>')
s = s[:i] + s[j:]
first_end = s.find('</hp:p>') + len('</hp:p>')

def para(text, page_break=False, vpos=None, sw=0, char=0):
    seg = ('' if vpos is None else
           '<hp:linesegarray><hp:lineseg textpos="0" vertpos="%d" vertsize="1000" textheight="1000" '
           'baseline="850" spacing="600" horzpos="0" horzsize="%d" flags="393216"/></hp:linesegarray>' % (vpos, sw))
    return ('<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="%d" columnBreak="0" merged="0">'
            '<hp:run charPrIDRef="%d"><hp:t>%s</hp:t></hp:run>%s</hp:p>' % (1 if page_break else 0, char, text, seg))

def cell(row, col, rs, w, h, texts, stored=False, margin=None):
    has = '0' if margin is None else '1'
    sw = w - 2 * (510 if margin is None else margin)  # 위아래 기본 여백은 IN_TB
    body = ''.join(para(t, vpos=(i * 1600 if stored else None), sw=sw, char=CELL_CHAR) for i, t in enumerate(texts))
    m = 'left="510" right="510" top="%d" bottom="%d"' % (IN_TB, IN_TB) if margin is None else 'left="%d" right="%d" top="%d" bottom="%d"' % ((margin,) * 4)
    return ('<hp:tc name="" header="0" hasMargin="%s" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
            '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">%s</hp:subList>'
            '<hp:cellAddr colAddr="%d" rowAddr="%d"/><hp:cellSpan colSpan="1" rowSpan="%d"/>'
            '<hp:cellSz width="%d" height="%d"/><hp:cellMargin %s/></hp:tc>'
            % (has, body, col, row, rs, w, h, m))

def wide_table(variant):
    # V7: 왼쪽·오른쪽 병합 칸이 시험 행 앞뒤로 걸친다(5행: 머리 · 시험 · 다음 셋).
    # V8: V7 + 시험 행 칸 여백 283 · V9: V8 + 저장 줄 + 내용 칸 선언 4170.
    margin = (MARGIN if variant in ('V8', 'V9') else None)
    stored = variant == 'V9'
    body_h = 4170 if variant == 'V9' else 1882
    cols = [5000, 6000, WIDTH - 22000, 11000]
    n = 5
    content = ['내용 %d 줄' % (i + 1) for i in range(5)]
    total = 1882 * (n - 1) + DECL
    r = [[cell(0, 0, n, cols[0], total, ['병합 왼쪽']),
          cell(0, 1, 1, cols[1], 1882, ['머리']), cell(0, 2, 1, cols[2], 1882, ['머리 내용']),
          cell(0, 3, n, cols[3], total, ['병합 오른쪽'])],
         [cell(1, 1, 1, cols[1], DECL, ['라벨'], stored, margin),
          cell(1, 2, 1, cols[2], body_h, content, stored, margin)]]
    for row in range(2, n):
        r.append([cell(row, 1, 1, cols[1], 1882, ['다음']), cell(row, 2, 1, cols[2], 1882, ['다음 행 %d' % row])])
    rows = ''.join('<hp:tr>%s</hp:tr>' % ''.join(x) for x in r)
    return wrap(len(cols), n, total, rows)

def wrap(ncols, nrows, total, rows):
    return ('<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">'
            '<hp:tbl id="1" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
            'dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="%d" colCnt="%d" cellSpacing="0" '
            'borderFillIDRef="3" noAdjust="0"><hp:sz width="%d" widthRelTo="ABSOLUTE" height="%d" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
            'horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="283" right="283" top="283" bottom="283"/><hp:inMargin left="510" right="510" top="%d" bottom="%d"/>'
            '%s</hp:tbl><hp:t/></hp:run></hp:p>' % (nrows, ncols, WIDTH, total, IN_TB, IN_TB, rows))

def table(variant):
    if variant in ('V7', 'V8', 'V9'):
        return wide_table(variant)
    span = variant in ('V2', 'V5', 'V6')
    stored = variant in ('V5', 'V6', 'V10')
    cols = [6000, 8000, WIDTH - 14000] if span else [8000, WIDTH - 8000]
    label_h = DECL if variant in ('V1', 'V2', 'V3', 'V5', 'V6', 'V10') else 1882
    body_h = DECL if variant in ('V3', 'V4') else (4170 if variant == 'V6' else 1882)
    content = ['내용 %d 줄' % (i + 1) for i in range(5)]
    rows = []
    c = 0
    r0 = []
    if span:
        r0.append(cell(0, 0, 3, cols[0], 1882 * 2 + DECL, ['병합']))
        c = 1
    r0 += [cell(0, c, 1, cols[c], 1882, ['머리']), cell(0, c + 1, 1, cols[c + 1], 1882, ['머리 내용'])]
    r1 = [cell(1, c, 1, cols[c], label_h, ['라벨'], stored),
          cell(1, c + 1, 1, cols[c + 1], body_h, content, stored)]
    r2 = [cell(2, c, 1, cols[c], 1882, ['다음']), cell(2, c + 1, 1, cols[c + 1], 1882, ['다음 행 내용'])]
    rows = ''.join('<hp:tr>%s</hp:tr>' % ''.join(r) for r in (r0, r1, r2))
    total = 1882 * 2 + max(label_h, body_h)
    return ('<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">'
            '<hp:tbl id="1" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" '
            'dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="3" colCnt="%d" cellSpacing="0" '
            'borderFillIDRef="3" noAdjust="0"><hp:sz width="%d" widthRelTo="ABSOLUTE" height="%d" heightRelTo="ABSOLUTE" protect="0"/>'
            '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
            'horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            '<hp:outMargin left="283" right="283" top="283" bottom="283"/><hp:inMargin left="510" right="510" top="141" bottom="141"/>'
            '%s</hp:tbl><hp:t/></hp:run></hp:p>' % (len(cols), WIDTH, total, rows))

blocks = []
VARIANTS = sys.argv[3].split(',') if len(sys.argv) > 3 else ['V1', 'V2', 'V3', 'V4']
for variant in VARIANTS:
  for DECL in DECLS:
    for k in KS:
        blocks.append(para('%s D=%d K=%d' % (variant, DECL, k), page_break=bool(blocks)))
        blocks += [para('채움 %d' % i) for i in range(1, k)]
        blocks.append(table(variant))
s = s[:first_end] + ''.join(blocks) + '</hs:sec>'
files['Contents/section0.xml'] = s.encode()
with zipfile.ZipFile(out, 'w') as o:
    for n in z.namelist():
        o.writestr(z.getinfo(n), files[n], compress_type=zipfile.ZIP_STORED if n == 'mimetype' else zipfile.ZIP_DEFLATED)
