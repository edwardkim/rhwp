#!/usr/bin/env python3
"""자립형 visual oracle — 시스템 래스터라이저(rsvg-convert/pdftoppm) 없이 동작.

rhwp 쪽:  `rhwp export-png`(native-skia) 네이티브 래스터
PDF 쪽:   PyMuPDF(fitz) 로 직접 래스터 (poppler 불필요)
비교:     PIL + numpy 로 pixel/ink match + overlay (task1274_visual_sweep.py 시맨틱 정합)

Windows/Git-Bash 환경(패키지 매니저 없음)에서 시각 판정을 가능케 한다.
scripts/task1274_visual_sweep.py 의 대체가 아니라, 오라클 도구 부재 환경용 경량 경로.

사용:
  python scripts/visual_oracle_native.py \
      --hwp samples/task1749/saved_bounds_cumulative_page_break.hwpx \
      --pdf samples/task1749/saved_bounds_cumulative_page_break-2024.pdf \
      --pages 4,5 --dpi 96 --out output/poc/oracle

출력(--out):
  rhwp_png/page_00N.png, pdf_png/page_00N.png, overlay/overlay_00N.png,
  review/review_00N.png, metrics.json
페이지 번호는 사용자가 PDF 뷰어에서 보는 1-based.
"""
import argparse
import json
import os
import subprocess
import sys

import fitz  # PyMuPDF
import numpy as np
from PIL import Image, ImageDraw, ImageFont

PIXEL_DIFF_THRESHOLD = 32  # task1274 기본과 동일
INK_WHITE_CUTOFF = 245     # 이 값 이상(모든 채널)이면 배경(잉크 아님)


def rhwp_bin():
    for p in ("target/release/rhwp.exe", "target/release/rhwp",
              "target/debug/rhwp.exe", "target/debug/rhwp"):
        if os.path.exists(p):
            return os.path.abspath(p)
    sys.exit("rhwp 바이너리를 찾을 수 없습니다 (cargo build --release --features native-skia)")


def parse_pages(spec):
    out = []
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            a, b = part.split("-")
            out.extend(range(int(a), int(b) + 1))
        else:
            out.append(int(part))
    return sorted(set(out))


def export_rhwp_png(binary, hwp, page0, dpi, out_path, font_paths):
    """rhwp export-png 으로 단일 페이지 PNG 생성. page0 은 0-based."""
    tmp_dir = os.path.join(os.path.dirname(out_path), "_rhwp_tmp")
    os.makedirs(tmp_dir, exist_ok=True)
    cmd = [binary, "export-png", hwp, "-p", str(page0), "-o", tmp_dir, "--dpi", str(dpi)]
    for fp in font_paths or []:
        cmd += ["--font-path", fp]
    res = subprocess.run(cmd, capture_output=True, text=True,
                         encoding="utf-8", errors="replace")
    if res.returncode != 0:
        sys.exit(f"export-png 실패(p{page0}):\n{res.stderr}")
    # rhwp 는 page_00N.png 또는 <stem>_00N.png 형태로 낼 수 있음 — 가장 최근 png 채택
    cand = [os.path.join(tmp_dir, f) for f in os.listdir(tmp_dir) if f.lower().endswith(".png")]
    if not cand:
        sys.exit(f"export-png 산출 PNG 없음(p{page0}) in {tmp_dir}\n{res.stdout}")
    # 파일명에 (page0+1) 또는 page0 이 들어간 것을 우선
    want = [c for c in cand if f"{page0 + 1:03d}" in os.path.basename(c)
            or f"{page0 + 1}" in os.path.basename(c)]
    src = (want or cand)[0]
    Image.open(src).convert("RGB").save(out_path)
    for c in cand:
        try:
            os.remove(c)
        except OSError:
            pass
    return out_path


def render_pdf_png(pdf_doc, page_idx0, dpi, out_path):
    """PyMuPDF 로 PDF 페이지(0-based) 를 dpi 로 래스터."""
    page = pdf_doc[page_idx0]
    zoom = dpi / 72.0
    pix = page.get_pixmap(matrix=fitz.Matrix(zoom, zoom), alpha=False)
    img = Image.frombytes("RGB", (pix.width, pix.height), pix.samples)
    img.save(out_path)
    return out_path


def to_common_canvas(img_a, img_b):
    """두 이미지를 좌상단 정렬로 공통 캔버스(흰 배경)에 패딩."""
    w = max(img_a.width, img_b.width)
    h = max(img_a.height, img_b.height)
    ca = Image.new("RGB", (w, h), (255, 255, 255))
    cb = Image.new("RGB", (w, h), (255, 255, 255))
    ca.paste(img_a, (0, 0))
    cb.paste(img_b, (0, 0))
    return ca, cb


def compute_metrics(rhwp_img, pdf_img):
    a = np.asarray(rhwp_img, dtype=np.int16)   # rhwp
    b = np.asarray(pdf_img, dtype=np.int16)    # pdf
    diff = np.abs(a - b).max(axis=2)           # per-pixel 최대 채널차
    total = diff.size
    diff_mask = diff > PIXEL_DIFF_THRESHOLD
    pixel_match = 100.0 * (1.0 - diff_mask.sum() / total)

    a_ink = (a.max(axis=2) < INK_WHITE_CUTOFF)  # rhwp 잉크
    b_ink = (b.max(axis=2) < INK_WHITE_CUTOFF)  # pdf 잉크
    ink_union = a_ink | b_ink
    ink_union_n = int(ink_union.sum())
    ink_diff = np.logical_and(ink_union, diff_mask)
    if ink_union_n > 0:
        ink_match = 100.0 * (1.0 - ink_diff.sum() / ink_union_n)
        proxy = ink_match
    else:
        ink_match = 100.0
        proxy = pixel_match

    return {
        "pixel_match_percent": round(float(pixel_match), 4),
        "ink_match_percent": round(float(ink_match), 4),
        "visual_accuracy_proxy_percent": round(float(proxy), 4),
        "ink_union_pixels": ink_union_n,
        "diff_pixels": int(diff_mask.sum()),
    }, (a_ink, b_ink, diff_mask)


def build_overlay(rhwp_img, pdf_img, masks):
    a_ink, b_ink, diff_mask = masks
    h, w = diff_mask.shape
    out = np.full((h, w, 3), 235, dtype=np.uint8)  # 옅은 회색 배경
    both_ink = a_ink & b_ink
    only_a = a_ink & ~b_ink            # rhwp 만: 빨강
    only_b = b_ink & ~a_ink            # pdf 만: 파랑
    both_diff = both_ink & diff_mask   # 양쪽 잉크 위치차: 주황
    both_same = both_ink & ~diff_mask  # 일치: 진회색
    out[only_a] = (220, 40, 40)
    out[only_b] = (40, 80, 220)
    out[both_diff] = (240, 150, 30)
    out[both_same] = (90, 90, 90)
    return Image.fromarray(out, "RGB")


def build_review(rhwp_img, pdf_img, overlay_img, caption):
    pad = 12
    w = rhwp_img.width + pdf_img.width + overlay_img.width + pad * 4
    h = max(rhwp_img.height, pdf_img.height, overlay_img.height) + 60
    canvas = Image.new("RGB", (w, h), (255, 255, 255))
    x = pad
    for img, label in ((rhwp_img, "rhwp"), (pdf_img, "pdf(oracle)"), (overlay_img, "overlay")):
        canvas.paste(img, (x, 30))
        d = ImageDraw.Draw(canvas)
        d.text((x, 8), label, fill=(0, 0, 0))
        x += img.width + pad
    d = ImageDraw.Draw(canvas)
    d.text((pad, h - 24), caption, fill=(0, 0, 0))
    return canvas


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--hwp", required=True)
    ap.add_argument("--pdf", required=True)
    ap.add_argument("--pages", required=True, help="1-based, 예: 4,5 또는 3-6")
    ap.add_argument("--dpi", type=float, default=96.0)
    ap.add_argument("--out", default="output/poc/oracle")
    ap.add_argument("--font-path", action="append", default=[])
    args = ap.parse_args()

    binary = rhwp_bin()
    pages = parse_pages(args.pages)
    for sub in ("rhwp_png", "pdf_png", "overlay", "review"):
        os.makedirs(os.path.join(args.out, sub), exist_ok=True)

    pdf_doc = fitz.open(args.pdf)
    n_pdf = pdf_doc.page_count
    results = []
    for p in pages:  # 1-based
        if p < 1 or p > n_pdf:
            print(f"[skip] page {p} (PDF pages=1..{n_pdf})")
            continue
        page0 = p - 1
        rhwp_png = os.path.join(args.out, "rhwp_png", f"page_{p:03d}.png")
        pdf_png = os.path.join(args.out, "pdf_png", f"page_{p:03d}.png")
        export_rhwp_png(binary, args.hwp, page0, args.dpi, rhwp_png, args.font_path)
        render_pdf_png(pdf_doc, page0, args.dpi, pdf_png)

        ra = Image.open(rhwp_png).convert("RGB")
        pb = Image.open(pdf_png).convert("RGB")
        ca, cb = to_common_canvas(ra, pb)
        metrics, masks = compute_metrics(ca, cb)
        overlay = build_overlay(ca, cb, masks)
        overlay_png = os.path.join(args.out, "overlay", f"overlay_{p:03d}.png")
        overlay.save(overlay_png)

        cap = (f"page {p}  pixel_match={metrics['pixel_match_percent']:.2f}%  "
               f"ink_match={metrics['ink_match_percent']:.2f}%  "
               f"proxy={metrics['visual_accuracy_proxy_percent']:.2f}%")
        review = build_review(ca, cb, overlay, cap)
        review_png = os.path.join(args.out, "review", f"review_{p:03d}.png")
        review.save(review_png)

        row = {"page": p, "rhwp_png": rhwp_png, "pdf_png": pdf_png,
               "overlay_png": overlay_png, "review_png": review_png, **metrics}
        results.append(row)
        print(f"[page {p}] {cap}")

    # 임시 폴더 정리
    tmp = os.path.join(args.out, "rhwp_png", "_rhwp_tmp")
    if os.path.isdir(tmp):
        try:
            os.rmdir(tmp)
        except OSError:
            pass
    with open(os.path.join(args.out, "metrics.json"), "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)
    print(f"metrics: {os.path.join(args.out, 'metrics.json')}")


if __name__ == "__main__":
    main()
