# -*- coding: utf-8 -*-
"""#2279 서브픽셀 정렬 하니스 — rhwp export-pdf ↔ 한글 PDF 의 anchor 간격 대조.

양쪽 PDF 에서 (a) 텍스트 줄 baseline origin(정확 좌표), (b) 표 괘선 프레임을
추출해 내용 순서로 시퀀스 정렬 후, **같은 페이지 내 연속 anchor 간격**의
rhwp−한글 Δ를 산출한다. bbox 퍼짐/페이지 경계 차이와 무관하게 발산 구간을
0.1pt 해상도로 국소화한다.

usage: python subpixel_align_harness.py <rhwp_exe> <doc.hwpx> <hancom.pdf> [--max N]
"""
import difflib
import os
import re
import subprocess
import sys
import tempfile

import fitz

sys.stdout.reconfigure(encoding="utf-8")

FONTS = r"C:\Users\planet\rhwp\ttfs"


def norm_text(t):
    t = re.sub(r"\s+", "", t)
    t = t.replace("ᄋ", "ㅇ").replace("․", ".").replace("‧", ".")
    return t[:24]


def extract_anchors(pdf_path):
    """문서 순서의 anchor 리스트: (page, y, kind, key).

    kind: 'T'=텍스트 줄(baseline origin.y), 'B'=표 프레임 top, 'E'=표 프레임 bottom.
    key 는 시퀀스 정렬용 내용 서명.
    """
    d = fitz.open(pdf_path)
    anchors = []
    for pno in range(d.page_count):
        page_items = []
        # 텍스트 줄 (origin = baseline, 정확 좌표)
        for blk in d[pno].get_text("rawdict")["blocks"]:
            if blk["type"] != 0:
                continue
            for line in blk["lines"]:
                chars = [c for span in line["spans"] for c in span["chars"]]
                text = "".join(c["c"] for c in chars)
                key = norm_text(text)
                if len(key) < 2:
                    continue
                y = chars[0]["origin"][1]
                page_items.append((y, "T", key))
        # 표 프레임 (수평+수직 괘선 연결성분)
        segs = []
        for dr in d[pno].get_drawings():
            for item in dr["items"]:
                if item[0] == "l":
                    p1, p2 = item[1], item[2]
                    if abs(p1.y - p2.y) < 0.5 and abs(p1.x - p2.x) > 8:
                        segs.append((min(p1.x, p2.x), p1.y, max(p1.x, p2.x), p1.y))
                    elif abs(p1.x - p2.x) < 0.5 and abs(p1.y - p2.y) > 4:
                        segs.append((p1.x, min(p1.y, p2.y), p1.x, max(p1.y, p2.y)))
                elif item[0] == "re":
                    r = item[1]
                    if r.height < 2 and r.width > 8:
                        segs.append((r.x0, r.y0, r.x1, r.y0))
                    elif r.width < 2 and r.height > 4:
                        segs.append((r.x0, r.y0, r.x0, r.y1))
        parent = list(range(len(segs)))

        def find(a):
            while parent[a] != a:
                parent[a] = parent[parent[a]]
                a = parent[a]
            return a

        for i in range(len(segs)):
            for j in range(i + 1, len(segs)):
                si, sj = segs[i], segs[j]
                if not (si[2] < sj[0] - 3 or sj[2] < si[0] - 3
                        or si[3] < sj[1] - 3 or sj[3] < si[1] - 3):
                    ri, rj = find(i), find(j)
                    if ri != rj:
                        parent[ri] = rj
        comps = {}
        for i, sg in enumerate(segs):
            r = find(i)
            c = comps.setdefault(r, [1e9, 1e9, -1e9, -1e9, 0])
            c[0] = min(c[0], sg[0]); c[1] = min(c[1], sg[1])
            c[2] = max(c[2], sg[2]); c[3] = max(c[3], sg[3]); c[4] += 1
        for c in comps.values():
            w, h = c[2] - c[0], c[3] - c[1]
            if c[4] >= 3 and w > 60:
                key = f"[TBL w{int(round(w / 20) * 20)}]"
                page_items.append((c[1], "B", key))
                page_items.append((c[3], "E", key))
        page_items.sort()
        for y, kind, key in page_items:
            anchors.append((pno + 1, round(y, 2), kind, key))
    return anchors


def main():
    rhwp_exe, doc, hancom_pdf = sys.argv[1], sys.argv[2], sys.argv[3]
    max_rows = int(sys.argv[sys.argv.index("--max") + 1]) if "--max" in sys.argv else 9999
    tmp = tempfile.mkdtemp(prefix="subpix_")
    rhwp_pdf = os.path.join(tmp, "rhwp.pdf")
    r = subprocess.run(
        [rhwp_exe, "export-pdf", doc, "-o", rhwp_pdf, "--font-path", FONTS],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900,
    )
    if not os.path.exists(rhwp_pdf):
        print("export-pdf 실패:", r.stderr[:300])
        sys.exit(1)

    a = extract_anchors(rhwp_pdf)      # rhwp
    b = extract_anchors(hancom_pdf)    # 한글
    sm = difflib.SequenceMatcher(a=[x[3] for x in a], b=[x[3] for x in b], autojunk=False)
    pairs = []
    for blk in sm.get_matching_blocks():
        for k in range(blk.size):
            pairs.append((a[blk.a + k], b[blk.b + k]))
    print(f"rhwp anchors={len(a)} hancom={len(b)} matched={len(pairs)}")
    print("구간 Δ(간격 rhwp−한글, pt) — 같은 페이지 내 연속 매칭쌍만:")
    prev = None
    shown = 0
    for ra, hb in pairs:
        if prev is not None:
            (rp, ry, _, _), (hp, hy, _, _) = ra, hb
            (prp, pry, _, _), (php, phy, _, _) = prev
            if rp == prp and hp == php:
                d_r = ry - pry
                d_h = hy - phy
                delta = d_r - d_h
                flag = "  <<<" if abs(delta) > 1.5 else ""
                if abs(delta) > 0.3 or flag:
                    print(f"  r_p{rp} {pry:7.1f}→{ry:7.1f} ({d_r:6.1f}) | 한글_p{hp} "
                          f"{phy:7.1f}→{hy:7.1f} ({d_h:6.1f}) | Δ{delta:+6.2f} "
                          f"'{ra[3][:18]}'{flag}")
                    shown += 1
                    if shown >= max_rows:
                        return
        prev = (ra, hb)


if __name__ == "__main__":
    main()
