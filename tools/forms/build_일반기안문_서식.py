# 일반기안문(별지 제1호서식) 표준 서식 HWPX 빌더 — seed.hwpx 를 기반으로 XML 재작성
import re, zipfile, shutil, os, io, sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

BASE = os.path.dirname(os.path.abspath(__file__))
X = os.path.join(BASE, "x")
OUT = os.path.join(BASE, "일반기안문_서식.hwpx")

# ── 1) header.xml 패치 ────────────────────────────────────────────────────────
h = open(os.path.join(X, "Contents/header.xml"), encoding="utf-8").read()

# (a) borderFill 1 을 무테두리로 (#3355 — 이 브랜치 기반엔 미반영이라 XML 에서 정정)
def fix_first_borderfill(m):
    s = m.group(0)
    return s.replace('type="SOLID"', 'type="NONE"')
h = re.sub(r'<hh:borderFill[^>]*id="1".*?</hh:borderFill>', fix_first_borderfill, h, count=1, flags=re.S)

# (b) borderFill 추가: 3=하단선만, 4=회색 배경(선 없음)
extra_bf = '''<hh:borderFill id="3"><hh:slash type="NONE" Crooked="0" isCounter="0"/><hh:backSlash type="NONE" Crooked="0" isCounter="0"/><hh:leftBorder type="NONE" width="0.12 mm" color="#000000"/><hh:rightBorder type="NONE" width="0.12 mm" color="#000000"/><hh:topBorder type="NONE" width="0.12 mm" color="#000000"/><hh:bottomBorder type="SOLID" width="0.4 mm" color="#000000"/><hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/></hh:borderFill><hh:borderFill id="4"><hh:slash type="NONE" Crooked="0" isCounter="0"/><hh:backSlash type="NONE" Crooked="0" isCounter="0"/><hh:leftBorder type="NONE" width="0.1 mm" color="#000000"/><hh:rightBorder type="NONE" width="0.1 mm" color="#000000"/><hh:topBorder type="NONE" width="0.1 mm" color="#000000"/><hh:bottomBorder type="NONE" width="0.1 mm" color="#000000"/><hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/><hc:fillBrush><hc:winBrush faceColor="#D9D9D9" hatchColor="#999999" alpha="0"/></hc:fillBrush></hh:borderFill>'''
h = h.replace("</hh:borderFills>", extra_bf + "</hh:borderFills>")
h = re.sub(r'(<hh:borderFills itemCnt=")\d+(")', r"\g<1>4\g<2>", h)

# (c) charPr 추가: 1=기관명(2200 bold), 2=발신명의(1600 bold), 3=결문(850)
m = re.search(r'<hh:charPr id="0".*?</hh:charPr>', h, flags=re.S)
base_cp = m.group(0)
def variant(cp_id, height, bold):
    v = base_cp.replace('id="0"', f'id="{cp_id}"').replace('height="1000"', f'height="{height}"')
    if bold:
        # OWPML 스키마 순서: offset 뒤·underline 앞에 bold 가 와야 한다
        v = re.sub(r"(<hh:offset[^>]*/>)", "\1<hh:bold/>", v, count=1)
    return v
h = h.replace(base_cp, base_cp + variant(1, 2200, True) + variant(2, 1600, True) + variant(3, 850, False))
h = re.sub(r'(<hh:charProperties itemCnt=")\d+(")', r"\g<1>4\g<2>", h)

# (d) paraPr 추가: 2=중앙정렬, 3=제목행(하단 굵은선), 4=회색 바(높이 작게)
m = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', h, flags=re.S)
base_pp = m.group(0)
def pvariant(pp_id, align=None, border=None):
    v = base_pp.replace('id="0"', f'id="{pp_id}"')
    if align:
        v = v.replace('horizontal="JUSTIFY"', f'horizontal="{align}"')
    if border is not None:
        # 기본 paraPr 의 border 참조를 교체 (borderFillIDRef 속성이 있으면 바꾸고 없으면 border 요소 추가)
        if "hh:border " in v or "borderFillIDRef" in v:
            v = re.sub(r'(<hh:border[^>]*borderFillIDRef=")\d+(")', rf"\g<1>{border}\g<2>", v)
        else:
            v = v.replace("</hh:paraPr>", f'<hh:border borderFillIDRef="{border}" offsetLeft="0" offsetRight="0" offsetTop="0" offsetBottom="140" connect="0" ignoreMargin="0"/></hh:paraPr>')
    return v
h = h.replace(base_pp, base_pp + pvariant(2, align="CENTER") + pvariant(3, border=3) + pvariant(4, border=4))
h = re.sub(r'(<hh:paraProperties itemCnt=")\d+(")', r"\g<1>5\g<2>", h)

open(os.path.join(X, "Contents/header.xml"), "w", encoding="utf-8", newline="") .write(h)

# ── 2) section0.xml 재작성 ────────────────────────────────────────────────────
s = open(os.path.join(X, "Contents/section0.xml"), encoding="utf-8").read()
# seed 의 첫 문단(secPr 포함)에서 섹션 껍데기를 보존한다
first_p = re.search(r'<hp:p .*?</hp:p>', s, flags=re.S).group(0)
prolog = s[: s.index(first_p)]
epilog = "</hs:sec>"

FID = [1700000000]
def field(name, guide):
    FID[0] += 1
    n = len(guide)
    return (f'<hp:ctrl><hp:fieldBegin id="{FID[0]}" type="CLICK_HERE" name="{name}" editable="1">'
            f'<hp:parameters cnt="1" name=""><hp:stringParam name="Command">Clickhere:set:48:Direction:wstring:{n}:{guide} HelpState:wstring:0:  </hp:stringParam></hp:parameters>'
            f'</hp:fieldBegin></hp:ctrl><hp:ctrl><hp:fieldEnd beginIDRef="{FID[0]}"/></hp:ctrl>')

PID = [100]
def para(inner, ppr=0, cpr=0):
    PID[0] += 1
    return (f'<hp:p id="{PID[0]}" paraPrIDRef="{ppr}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="{cpr}">{inner}</hp:run></hp:p>')

def t(x):
    return f'<hp:t>{x}</hp:t>'

body = []
# 두문
body.append(para(field("행정기관명", "행정기관명"), ppr=2, cpr=1))
body.append(para(t(""), ppr=0, cpr=0))
body.append(para(t("수신  ") + field("수신자", "수신자(참조)"), ppr=0, cpr=0))
body.append(para(t("(경유)  ") + field("경유", "경유 기관"), ppr=0, cpr=0))
# 제목행 — 하단 굵은선
body.append(para(t("제목  ") + field("제목", "제목을 입력하세요"), ppr=3, cpr=0))
body.append(para(t(""), ppr=0, cpr=0))
# 본문
body.append(para(field("본문", "내용을 입력하세요"), ppr=0, cpr=0))
body.append(para(t(""), ppr=0, cpr=0))
body.append(para(t("붙임  ") + field("붙임", "붙임을 입력하세요") + t("  1부.  끝."), ppr=0, cpr=0))
for _ in range(3):
    body.append(para(t(""), ppr=0, cpr=0))
# 결문
body.append(para(field("발신명의", "발신명의"), ppr=2, cpr=2))
body.append(para(t("수신자  ") + field("수신자명단", "수신자 명단"), ppr=0, cpr=3))
body.append(para(t(""), ppr=4, cpr=3))  # 회색 바
body.append(para(t("기안자 ") + field("기안자", "직위(직급) 서명") + t("   검토자 ") + field("검토자", "직위(직급) 서명") + t("   결재권자 ") + field("결재권자", "직위(직급) 서명"), ppr=0, cpr=3))
body.append(para(t("협조자 ") + field("협조자", "직위(직급) 서명"), ppr=0, cpr=3))
body.append(para(t("시행  ") + field("시행번호", "처리과명-일련번호") + t(" (") + field("시행일", "시행일") + t(")    접수  ") + field("접수번호", "처리과명-일련번호") + t(" (") + field("접수일", "접수일") + t(")"), ppr=0, cpr=3))
body.append(para(t("우 ") + field("우편번호", "우편번호") + t("  ") + field("주소", "도로명주소") + t("    / ") + field("홈페이지", "홈페이지 주소"), ppr=0, cpr=3))
body.append(para(t("전화번호 (") + field("전화번호", "전화번호") + t(")  팩스번호 (") + field("팩스번호", "팩스번호") + t(")  / ") + field("전자우편", "공무원의 전자우편주소") + t(" / ") + field("공개구분", "공개 구분"), ppr=0, cpr=3))

# 첫 문단은 seed 것을 유지하되 텍스트 run 을 비운다 (secPr/colPr 보존)
first_clean = re.sub(r'<hp:t>.*?</hp:t>', '<hp:t></hp:t>', first_p, flags=re.S)
new_s = prolog + first_clean + "".join(body) + epilog
open(os.path.join(X, "Contents/section0.xml"), "w", encoding="utf-8", newline="").write(new_s)

# ── 3) 재압축 (mimetype 무압축 선두) ──────────────────────────────────────────
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
