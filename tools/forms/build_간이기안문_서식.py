# 간이기안문(별지 제2호서식) 표준 서식 HWPX 빌더 — 결재란 표 포함
import re, zipfile, os, io, sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

BASE = os.path.dirname(os.path.abspath(__file__))
X = os.path.join(BASE, "x2")
OUT = os.path.join(BASE, "간이기안문_서식.hwpx")

# ── header.xml 패치 ──────────────────────────────────────────────────────────
h = open(os.path.join(X, "Contents/header.xml"), encoding="utf-8").read()
h = re.sub(r'<hh:borderFill[^>]*id="1".*?</hh:borderFill>',
           lambda m: m.group(0).replace('type="SOLID"', 'type="NONE"'), h, count=1, flags=re.S)
def bf(idn, ltype, width="0.12 mm"):
    b = f'type="{ltype}" width="{width}" color="#000000"'
    return (f'<hh:borderFill id="{idn}"><hh:slash type="NONE" Crooked="0" isCounter="0"/>'
            f'<hh:backSlash type="NONE" Crooked="0" isCounter="0"/>'
            f'<hh:leftBorder {b}/><hh:rightBorder {b}/><hh:topBorder {b}/><hh:bottomBorder {b}/>'
            f'<hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/></hh:borderFill>')
h = h.replace("</hh:borderFills>", bf(3, "SOLID") + bf(4, "DASH") + "</hh:borderFills>")
h = re.sub(r'(<hh:borderFills itemCnt=")\d+(")', r"\g<1>4\g<2>", h)

m = re.search(r'<hh:charPr id="0".*?</hh:charPr>', h, flags=re.S)
base_cp = m.group(0)
def cvar(cid, height, bold):
    v = base_cp.replace('id="0"', f'id="{cid}"').replace('height="1000"', f'height="{height}"')
    if bold:
        v = re.sub(r"(<hh:offset[^>]*/>)", "\\1<hh:bold/>", v, count=1)
    return v
h = h.replace(base_cp, base_cp + cvar(1, 2000, True) + cvar(2, 1100, True) + cvar(3, 850, False))
h = re.sub(r'(<hh:charProperties itemCnt=")\d+(")', r"\g<1>4\g<2>", h)

m = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', h, flags=re.S)
base_pp = m.group(0)
h = h.replace(base_pp, base_pp + base_pp.replace('id="0"', 'id="2"').replace('horizontal="JUSTIFY"', 'horizontal="CENTER"'))
h = re.sub(r'(<hh:paraProperties itemCnt=")\d+(")', r"\g<1>3\g<2>", h)
open(os.path.join(X, "Contents/header.xml"), "w", encoding="utf-8", newline="").write(h)

# ── section0.xml 재작성 ─────────────────────────────────────────────────────
s = open(os.path.join(X, "Contents/section0.xml"), encoding="utf-8").read()
# 법정 여백: 상·좌·우 20mm(5669), 하 10mm(2835)
s = re.sub(r'(<hp:margin[^>]*left=")\d+(")', r"\g<1>5669\g<2>", s, count=1)
s = re.sub(r'(<hp:margin[^>]*?right=")\d+(")', r"\g<1>5669\g<2>", s, count=1)
s = re.sub(r'(<hp:margin[^>]*?top=")\d+(")', r"\g<1>5669\g<2>", s, count=1)
s = re.sub(r'(<hp:margin[^>]*?bottom=")\d+(")', r"\g<1>2835\g<2>", s, count=1)

first_p = re.search(r'<hp:p .*?</hp:p>', s, flags=re.S).group(0)
prolog = s[: s.index(first_p)]

FID = [1800000000]
def field(name, guide):
    FID[0] += 1
    return (f'<hp:ctrl><hp:fieldBegin id="{FID[0]}" type="CLICK_HERE" name="{name}" editable="1">'
            f'<hp:parameters cnt="1" name=""><hp:stringParam name="Command">Clickhere:set:48:Direction:wstring:{len(guide)}:{guide} HelpState:wstring:0:  </hp:stringParam></hp:parameters>'
            f'</hp:fieldBegin></hp:ctrl><hp:ctrl><hp:fieldEnd beginIDRef="{FID[0]}"/></hp:ctrl>')

PID = [200]
def para(inner, ppr=0, cpr=0):
    PID[0] += 1
    return (f'<hp:p id="{PID[0]}" paraPrIDRef="{ppr}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="{cpr}">{inner}</hp:run></hp:p>')

def t(x):
    return f'<hp:t>{x}</hp:t>'

def cell(inner, col, row, w, hgt, colspan=1, rowspan=1, cpr=3, ppr=0, bfid=3):
    return (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="{bfid}">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'<hp:p id="0" paraPrIDRef="{ppr}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="{cpr}">{inner}</hp:run></hp:p></hp:subList>'
            f'<hp:cellAddr colAddr="{col}" rowAddr="{row}"/>'
            f'<hp:cellSpan colSpan="{colspan}" rowSpan="{rowspan}"/>'
            f'<hp:cellSz width="{w}" height="{hgt}"/>'
            f'<hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')

TID = [1900000000]
def table(rows_xml, rowcnt, colcnt, width, height, halign="LEFT", bfid=3, wrap="SQUARE", vrel="PARA", hrel="COLUMN", voff=0):
    TID[0] += 1
    return (f'<hp:tbl id="{TID[0]}" zOrder="0" numberingType="TABLE" textWrap="{wrap}" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="{rowcnt}" colCnt="{colcnt}" cellSpacing="0" borderFillIDRef="{bfid}" noAdjust="0">'
            f'<hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{height}" heightRelTo="ABSOLUTE" protect="0"/>'
            f'<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="1" holdAnchorAndSO="0" vertRelTo="{vrel}" horzRelTo="{hrel}" vertAlign="TOP" horzAlign="{halign}" vertOffset="{voff}" horzOffset="0"/>'
            f'<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
            f'<hp:inMargin left="141" right="141" top="141" bottom="141"/>'
            f'{rows_xml}</hp:tbl>')

def tr(cells):
    return "<hp:tr>" + "".join(cells) + "</hp:tr>"

body = []
# ── 상단: 등록표(좌) + 결재란(우) — 같은 문단에 두 표 앵커 ──
reg_rows = []
labels = [("생산등록번호", "생산등록번호"), ("등록일", "등록일"), ("결재일", "결재일"), ("공개구분", "공개구분")]
for r, (lab, fname) in enumerate(labels):
    reg_rows.append(tr([
        cell(t(lab), 0, r, 5800, 1300),
        cell(field(fname, " "), 1, r, 10200, 1300),
    ]))
reg_tbl = table("".join(reg_rows), 4, 2, 16000, 5200, halign="LEFT", vrel="PAPER", hrel="PAPER", voff=5669)

app_rows = [tr([
    cell(field("결재직위1", "직위"), 0, 0, 6200, 1400),
    cell(field("결재직위2", " "), 1, 0, 6200, 1400),
    cell(field("결재직위3", " "), 2, 0, 6200, 1400),
    cell(field("결재직위4", " "), 3, 0, 6200, 1400),
]), tr([
    cell(t(""), 0, 1, 6200, 4600),
    cell(t(""), 1, 1, 6200, 4600),
    cell(t(""), 2, 1, 6200, 4600),
    cell(t(""), 3, 1, 6200, 4600),
]), tr([
    cell(t("협조자"), 0, 2, 6200, 1400),
    cell(field("협조자", " "), 1, 2, 18600, 1400, colspan=3),
])]
app_tbl = table("".join(app_rows), 3, 4, 24800, 7400, halign="RIGHT", vrel="PAPER", hrel="PAPER", voff=5669)

body.append(para(reg_tbl + app_tbl + t(""), ppr=0, cpr=3))
for _ in range(6):
    body.append(para(t(""), ppr=0, cpr=0))

# ── 제목 박스 (중앙, 굵은 테두리 표 1×1) ──
title_tbl = table(tr([cell(field("제목", "(제        목)"), 0, 0, 40000, 6200, cpr=1, ppr=2)]), 1, 1, 40000, 6200, halign="CENTER", wrap="TOP_AND_BOTTOM")
body.append(para(title_tbl + t(""), ppr=2, cpr=0))
body.append(para(t(""), ppr=0, cpr=0))

# ── 요약설명 점선 박스 ──
sum_tbl = table(tr([cell(field("요약설명", "※필요한 경우 보고근거 및 보고내용을 요약하여 적을 수 있음"), 0, 0, 40000, 2400, cpr=3, ppr=2, bfid=4)]), 1, 1, 40000, 2400, halign="CENTER", bfid=4, wrap="TOP_AND_BOTTOM")
body.append(para(sum_tbl + t(""), ppr=2, cpr=0))
for _ in range(3):
    body.append(para(t(""), ppr=0, cpr=0))

# ── 작성일·작성기관 (중앙) ──
body.append(para(field("작성일", "작성일"), ppr=2, cpr=0))
body.append(para(t(""), ppr=0, cpr=0))
body.append(para(field("작성기관", "○○○○부(처·청·위원회 등) ○○○○국"), ppr=2, cpr=2))

first_clean = re.sub(r'<hp:t>.*?</hp:t>', '<hp:t></hp:t>', first_p, flags=re.S)
new_s = prolog + first_clean + "".join(body) + "</hs:sec>"
open(os.path.join(X, "Contents/section0.xml"), "w", encoding="utf-8", newline="").write(new_s)

if os.path.exists(OUT):
    os.remove(OUT)
with zipfile.ZipFile(OUT, "w") as z:
    z.write(os.path.join(X, "mimetype"), "mimetype", compress_type=zipfile.ZIP_STORED)
    for root, _, files in os.walk(X):
        for f in files:
            full = os.path.join(root, f)
            rel = os.path.relpath(full, X).replace("\\", "/")
            if rel == "mimetype":
                continue
            z.write(full, rel, compress_type=zipfile.ZIP_DEFLATED)
print("built:", OUT, os.path.getsize(OUT), "bytes")
