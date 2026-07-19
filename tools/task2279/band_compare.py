# -*- coding: utf-8 -*-
"""#2279 s8: 한글 PDF ↔ rhwp export-pdf 페이지 내 세로 밴드(행) 정렬 대조.

용법: python tools/task2279/band_compare.py <hancom.pdf> <rhwp.pdf> <page0based> [dpi]

각 페이지를 그레이스케일로 렌더해 잉크 행 밴드(y0,y1)를 추출, 순서대로 나란히
출력한다. dy0 열이 균일하면 페이지 내 ladder 정합, 특정 행 이후 dy 가 계단으로
성장하면 그 경계의 spacing 성분 누락/과다다 (36392557 sec1 분해에 사용:
pi42/43/44 sa 경계 +6.7px 계단 실측 → 합성 경계 sa 스냅 보존 규칙).
"""
import sys

import fitz
import numpy as np

sys.stdout.reconfigure(encoding="utf-8")


def rows(path, page, dpi=150, thresh=2):
    doc = fitz.open(path)
    pix = doc[page].get_pixmap(dpi=dpi, colorspace=fitz.csGRAY)
    img = np.frombuffer(pix.samples, dtype=np.uint8).reshape(pix.height, pix.width)
    ink = (img < 160).sum(axis=1)
    bands, inb, y0 = [], False, 0
    for y, v in enumerate(ink):
        if v > thresh and not inb:
            inb, y0 = True, y
        elif v <= thresh and inb:
            inb = False
            if y - y0 >= 2:
                bands.append((y0, y))
    return bands


def main():
    ref, tgt, page = sys.argv[1], sys.argv[2], int(sys.argv[3])
    dpi = int(sys.argv[4]) if len(sys.argv) > 4 else 150
    hb, rb = rows(ref, page, dpi), rows(tgt, page, dpi)
    print(f"page {page + 1}: ref {len(hb)} bands / tgt {len(rb)} bands (dpi {dpi})")
    for i in range(max(len(hb), len(rb))):
        h = hb[i] if i < len(hb) else None
        r = rb[i] if i < len(rb) else None
        dy = (h[0] - r[0]) if h and r else ""
        print(f"  {str(h):>14} | {str(r):>14} | dy0={dy}")


if __name__ == "__main__":
    main()
