# -*- coding: utf-8 -*-
"""#2279 pi78 분할 정책 판별 사다리 — 하단여백 변형 × 한글 COM PDF.

가설 (a) 말미 fit에 trailing ls 요구: 슬랙 +0.19pt(여백 -15HU~)부터 첫 줄 삽입.
가설 (b) 2줄 문단 orphan 회피: 통째(45pt)가 들어갈 때까지 이월 유지.
각 변형을 한글로 열어 PDF 저장 후, pi77('모든 재난현장') 페이지에
pi78 line1('긴급한 출동일수록')/line2('컨트롤 필요')가 삽입됐는지와
pi77 baseline·페이지수를 기록한다.
"""
import re, sys, time, zipfile, subprocess
from pathlib import Path
sys.stdout.reconfigure(encoding="utf-8")

import argparse
_ap = argparse.ArgumentParser()
_ap.add_argument("--src", required=True, help="대상 HWPX (기본 실험: 36399374)")
_ap.add_argument("--out", default="output/poc/pi78_ladder")
_a = _ap.parse_args()
SRC = _a.src
TMP = Path(_a.out)
TMP.mkdir(parents=True, exist_ok=True)
BOTTOMS = [3600, 3585, 3570, 3550, 3520, 3480, 3440, 3400, 3300, 3200,
           3000, 2800, 2600, 2400, 2200, 2000, 1800, 1600]

MARGIN_RE = re.compile(r'(<hp:margin[^>]*bottom=")(\d+)(")')

def build_variant(bottom):
    out = TMP / f"v{bottom}.hwpx"
    if out.exists():
        return out
    with zipfile.ZipFile(SRC) as zin:
        infos = zin.infolist()
        data = {i.filename: zin.read(i.filename) for i in infos}
    xml = data["Contents/section0.xml"].decode("utf-8")
    m = MARGIN_RE.search(xml)
    xml = xml[: m.start(2)] + str(bottom) + xml[m.end(2):]
    with zipfile.ZipFile(out, "w") as zo:
        for zi in infos:
            payload = xml.encode("utf-8") if zi.filename == "Contents/section0.xml" else data[zi.filename]
            zo.writestr(zi, payload, zi.compress_type)
    return out

WORKER = r"""
# -*- coding: utf-8 -*-
import sys
from pyhwpx import Hwp
src, pdf = sys.argv[1], sys.argv[2]
hwp = Hwp(visible=False)
try:
    hwp.open(src)
    hwp.save_as(pdf, format="PDF")
finally:
    try: hwp.quit()
    except Exception: pass
"""
(TMP / "worker.py").write_text(WORKER, encoding="utf-8")

def make_pdf(doc, pdf):
    for attempt in range(2):
        try:
            r = subprocess.run([sys.executable, str(TMP / "worker.py"), str(doc), str(pdf)],
                               capture_output=True, timeout=180)
            if Path(pdf).exists():
                return True
        except subprocess.TimeoutExpired:
            pass
        subprocess.run(["taskkill", "/IM", "Hwp.exe", "/F"], capture_output=True)
        time.sleep(5)
    return False

def analyze(pdf, bottom):
    import fitz
    d = fitz.open(str(pdf))
    body_bottom = 841.89 - bottom / 100.0
    for pno in range(d.page_count):
        lines = []
        for blk in d[pno].get_text("rawdict")["blocks"]:
            if blk["type"] != 0:
                continue
            for line in blk["lines"]:
                chars = [c for span in line["spans"] for c in span["chars"]]
                t = "".join(c["c"] for c in chars)
                if t.strip():
                    lines.append((chars[0]["origin"][1], t))
        lines.sort()
        joined = [t for _, t in lines]
        if any("모든 재난현장" in t for t in joined):
            bl77 = next(y for y, t in lines if "모든 재난현장" in t)
            l1 = any("긴급한 출동일수록" in t for t in joined)
            l2 = any("컨트롤 필요" in t for t in joined)
            # 같은 페이지 & pi77 아래쪽인지 확인
            if l1:
                yl1 = next(y for y, t in lines if "긴급한 출동일수록" in t)
                l1 = yl1 > bl77
            if l2:
                yl2 = next(y for y, t in lines if "컨트롤 필요" in t)
                l2 = yl2 > bl77
            return dict(page=pno + 1, pages=d.page_count, bl77=round(bl77, 2),
                        slack=round(body_bottom - bl77, 2), l1=l1, l2=l2)
    return dict(page=None, pages=d.page_count)

print("bottom\tpages\tp(pi77)\tbl77\tslack_pt\tline1\tline2", flush=True)
for b in BOTTOMS:
    doc = build_variant(b)
    pdf = TMP / f"v{b}.pdf"
    if not pdf.exists():
        ok = make_pdf(doc, pdf)
        if not ok:
            print(f"{b}\tERR", flush=True)
            continue
    r = analyze(pdf, b)
    print(f"{b}\t{r.get('pages')}\t{r.get('page')}\t{r.get('bl77')}\t{r.get('slack')}\t{r.get('l1')}\t{r.get('l2')}", flush=True)
