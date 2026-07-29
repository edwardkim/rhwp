# -*- coding: utf-8 -*-
"""#2279 한컴돋움/한컴바탕 메트릭 정합 — base vs head 픽셀 대조 (한글 PDF 기준).

지표: tools/task2279/visual_evidence.py 와 동일 (96dpi gray, 임계 이진화 일치율 + IoU).
"""
import subprocess
import sys
from pathlib import Path

import fitz
import numpy as np

sys.stdout.reconfigure(encoding="utf-8")

FONTS = Path(r"C:\Users\planet\rhwp\ttfs")


def render(exe: str, doc: Path, out_pdf: Path):
    if out_pdf.exists():
        out_pdf.unlink()
    r = subprocess.run(
        [exe, "export-pdf", str(doc), "-o", str(out_pdf), "--font-path", str(FONTS)],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900,
    )
    if not out_pdf.exists():
        raise RuntimeError(f"export-pdf 실패: {r.stderr[:300]}")


def page_gray(doc, i):
    pix = doc[i].get_pixmap(dpi=96, colorspace=fitz.csGRAY)
    return np.frombuffer(pix.samples, dtype=np.uint8).reshape(pix.height, pix.width)


def compare(ref, img):
    h = min(ref.shape[0], img.shape[0])
    w = min(ref.shape[1], img.shape[1])
    a, b = ref[:h, :w].astype(int), img[:h, :w].astype(int)
    pixel_match = 100.0 * float(((a > 200) == (b > 200)).mean())
    ink_a = (a <= 200)
    ink_b = (b <= 200)
    union = (ink_a | ink_b).sum()
    iou = 100.0 * float((ink_a & ink_b).sum() / union) if union else 100.0
    return pixel_match, iou


def main():
    base_exe, head_exe, out_dir = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    out_dir.mkdir(parents=True, exist_ok=True)
    hwpdocs = Path(r"C:\Users\planet\hwpdocs\samples")
    poc = Path(r"C:\Users\planet\rhwp\output\poc\task2246")
    pairs = [
        (hwpdocs / "문화본부 문화예술과" / "36398599_결재문서본문_「동대문구 찾아가는 전통시장」 ’25년 민간축제 정산 결과 보고.hwpx",
         poc / "36398599_h.pdf"),
        (hwpdocs / "문화본부 문화유산활용과" / "36398700_결재문서본문_공유재산 사용허가(임시무상 사용) 검토보고 2.hwpx",
         poc / "36398700_h.pdf"),
    ]
    for doc, ref_pdf in pairs:
        stem = doc.name.split("_")[0]
        ref = fitz.open(ref_pdf)
        print(f"== {stem} (한글 {ref.page_count}쪽)")
        for tag, exe in (("base", base_exe), ("head", head_exe)):
            out_pdf = out_dir / f"{stem}_{tag}.pdf"
            render(exe, doc, out_pdf)
            got = fitz.open(out_pdf)
            n = min(ref.page_count, got.page_count)
            scores = []
            for i in range(n):
                pm, iou = compare(page_gray(ref, i), page_gray(got, i))
                scores.append((pm, iou))
            avg_pm = sum(s[0] for s in scores) / n
            avg_iou = sum(s[1] for s in scores) / n
            per = " ".join(f"p{i}:{s[0]:.1f}/{s[1]:.1f}" for i, s in enumerate(scores))
            print(f"  {tag}: pages={got.page_count} avg_match={avg_pm:.2f} avg_iou={avg_iou:.2f}  [{per}]")


if __name__ == "__main__":
    main()
