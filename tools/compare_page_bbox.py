"""페이지 dark-pixel bbox 비교 — rhwp export-pdf 산출물 vs 한글 편집기 PDF 정답지.

Task #1748 검증 도구. 두 PDF 를 같은 DPI 로 래스터화해 페이지별 잉크 bbox
(top/bottom) 를 비교한다. 쪽 경계 over-fill(하단 잉크 초과) 회귀 검증용.

사용:
    rhwp export-pdf samples/table_scattered_header_rowbreak.hwp -o out.pdf
    python tools/compare_page_bbox.py out.pdf pdf/table_scattered_header_rowbreak-2024.pdf --pages 5,6

    # Task #1748 게이트 (p6 Δbot ≤ 5px; p5 는 기존 행 컷 소차 −6 이라 게이트 제외):
    python tools/compare_page_bbox.py out.pdf pdf/table_scattered_header_rowbreak-2024.pdf \
        --pages 6 --max-dbot 5

의존: pymupdf, Pillow  (pip install pymupdf pillow)
- --pages 는 1-기반 페이지 번호(콤마 구분). 생략 시 공통 페이지 전체.
- --max-dbot N 지정 시 |Δbot| > N 인 페이지가 있으면 exit 1 (게이트 모드).
"""
from __future__ import annotations

import argparse
import io
import sys

import fitz  # pymupdf
from PIL import Image


def page_bbox(doc: "fitz.Document", page_idx: int, dpi: int, thresh: int):
    """페이지의 dark-pixel (top, bottom) y 좌표. 잉크 없으면 (None, None)."""
    pix = doc[page_idx].get_pixmap(dpi=dpi)
    img = Image.open(io.BytesIO(pix.tobytes("png"))).convert("L")
    w, h = img.size
    px = img.load()
    top = bot = None
    for y in range(h):
        if any(px[x, y] < thresh for x in range(w)):
            if top is None:
                top = y
            bot = y
    return top, bot


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("rhwp_pdf", help="rhwp export-pdf 산출 PDF")
    ap.add_argument("hancom_pdf", help="한글 편집기 PDF 정답지")
    ap.add_argument("--pages", default=None, help="1-기반 페이지 번호 콤마 구분 (예: 5,6)")
    ap.add_argument("--dpi", type=int, default=96)
    ap.add_argument("--thresh", type=int, default=160, help="dark-pixel 임계 (grayscale)")
    ap.add_argument("--max-dbot", type=float, default=None, help="|Δbot| 게이트 (px, 초과 시 exit 1)")
    a = ap.parse_args()

    rd = fitz.open(a.rhwp_pdf)
    hd = fitz.open(a.hancom_pdf)
    print(f"pages: rhwp={rd.page_count} hancom={hd.page_count}")
    if a.pages:
        idxs = [int(p) - 1 for p in a.pages.split(",")]
    else:
        idxs = list(range(min(rd.page_count, hd.page_count)))

    fail = False
    for i in idxs:
        if i >= rd.page_count or i >= hd.page_count:
            print(f"p{i + 1}: (범위 밖)")
            continue
        ht, hb = page_bbox(hd, i, a.dpi, a.thresh)
        rt, rb = page_bbox(rd, i, a.dpi, a.thresh)
        if None in (ht, hb, rt, rb):
            print(f"p{i + 1}: hancom={ht}/{hb} rhwp={rt}/{rb} (빈 페이지)")
            continue
        dt, db = rt - ht, rb - hb
        mark = ""
        if a.max_dbot is not None and abs(db) > a.max_dbot:
            mark = "  << GATE FAIL"
            fail = True
        print(f"p{i + 1}: hancom top/bot={ht}/{hb}  rhwp top/bot={rt}/{rb}  dTop={dt:+d} dBot={db:+d}{mark}")

    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
