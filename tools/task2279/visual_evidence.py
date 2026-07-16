# -*- coding: utf-8 -*-
"""PR #2284 visual sweep 증적 — base(devel) vs head(pr-task2279) vs 한글 기준 PDF.

리뷰 요구: "저자 실행 환경의 기준 PDF, 폰트 조건, 전후 sweep 산출물".
- 기준 PDF: pdf/issue1921/86712_regulatory_analysis-2024.pdf (Hancom PDF 1.3.0.550,
  리뷰와 동일 경로)
- 렌더: rhwp export-pdf --font-path ttfs (Windows + 한컴 폰트 설치 환경)
- 지표: pixel match % (리뷰 방식 근사 — 96dpi 래스터, 그레이스케일 임계 이진화
  후 일치 픽셀 비율) + 평균 절대차 유사도

usage: python visual_evidence.py <base_exe> <head_exe> <out_dir> [pages...]
"""
import subprocess
import sys
from pathlib import Path

import fitz
import numpy as np

sys.stdout.reconfigure(encoding="utf-8")

DOC = Path(r"C:\Users\planet\rhwp\samples\86712_regulatory_analysis.hwp")
REF = Path(r"C:\Users\planet\rhwp\pdf\issue1921\86712_regulatory_analysis-2024.pdf")
FONTS = Path(r"C:\Users\planet\rhwp\ttfs")


def render(exe: str, out_pdf: Path):
    if out_pdf.exists():
        out_pdf.unlink()
    r = subprocess.run(
        [exe, "export-pdf", str(DOC), "-o", str(out_pdf), "--font-path", str(FONTS)],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900,
    )
    if not out_pdf.exists():
        raise RuntimeError(f"export-pdf 실패: {r.stderr[:300]}")


def page_gray(doc: "fitz.Document", i: int) -> np.ndarray:
    pix = doc[i].get_pixmap(dpi=96, colorspace=fitz.csGRAY)
    return np.frombuffer(pix.samples, dtype=np.uint8).reshape(pix.height, pix.width)


def compare(ref: np.ndarray, img: np.ndarray):
    h = min(ref.shape[0], img.shape[0])
    w = min(ref.shape[1], img.shape[1])
    a, b = ref[:h, :w].astype(int), img[:h, :w].astype(int)
    pixel_match = 100.0 * float(((a > 200) == (b > 200)).mean())
    sim = 100.0 * (1.0 - np.abs(a - b).mean() / 255.0)
    ink_a = (a <= 200)
    ink_b = (b <= 200)
    union = (ink_a | ink_b).sum()
    iou = 100.0 * float((ink_a & ink_b).sum() / union) if union else 100.0
    return pixel_match, sim, iou


def main():
    base_exe, head_exe, out_dir = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    pages = [int(x) for x in sys.argv[4:]] or [9, 64]
    out_dir.mkdir(parents=True, exist_ok=True)
    base_pdf = out_dir / "86712_base.pdf"
    head_pdf = out_dir / "86712_head.pdf"
    render(base_exe, base_pdf)
    render(head_exe, head_pdf)
    ref = fitz.open(str(REF))
    bd = fitz.open(str(base_pdf))
    hd = fitz.open(str(head_pdf))
    print(f"ref={ref.page_count}p base={bd.page_count}p head={hd.page_count}p "
          f"(producer={ref.metadata.get('producer')})")
    print("page\tside\tpixel_match%\tsim%\tink_IoU%")
    for p in pages:
        rg = page_gray(ref, p)
        for name, d in (("base", bd), ("head", hd)):
            pm, sim, iou = compare(rg, page_gray(d, p))
            print(f"p{p + 1}\t{name}\t{pm:.5f}\t{sim:.3f}\t{iou:.3f}")
        # 증적 PNG: ref | base | head 가로 병치
        imgs = [rg, page_gray(bd, p), page_gray(hd, p)]
        hh = min(i.shape[0] for i in imgs)
        ww = min(i.shape[1] for i in imgs)
        strip = np.concatenate([i[:hh, :ww] for i in imgs], axis=1)
        import PIL.Image

        PIL.Image.fromarray(strip).save(out_dir / f"86712_p{p + 1}_ref_base_head.png")
    print(f"증적 PNG → {out_dir}")


if __name__ == "__main__":
    main()
