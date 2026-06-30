"""#1658 한글 PDF baseline 인프라 — 한글 권위 줄 위치(baseline) 추출 + rhwp render 대조.

cut↔render↔한글 3자 줄높이 fidelity 작업의 **기준(한글)** 을 제공한다. 한글이 그린 각 텍스트 줄의
baseline Y 를 권위 PDF 에서 추출하고, rhwp export-svg 의 baseline 과 줄 pitch(줄간격)를 비교한다.

- 한글 baseline: pyhwpx 로 PDF 생성(또는 기존 PDF) → PyMuPDF(fitz) span origin.y. pt→px(×96/72).
- rhwp baseline: `rhwp export-svg` → `<text y=...>`.
- 출력(--compare): 페이지별 한글/rhwp 줄수·median pitch·pitch diff. fidelity drift 정량화.

사용:
    # 한글 baseline 만 (PDF 자동 생성)
    python tools/hangul_pdf_baseline.py <file.hwp> [--pdf existing.pdf] [-o out.tsv]
    # rhwp render 와 줄 pitch 대조
    python tools/hangul_pdf_baseline.py <file.hwp> --compare --exe <rhwp>
요구: Windows + 한컴 + pyhwpx + PyMuPDF.
"""
from __future__ import annotations

import argparse
import re
import statistics
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path

PT_TO_PX = 96.0 / 72.0
SVG_NS = "{http://www.w3.org/2000/svg}"


def hangul_pdf_baselines(src: Path, pdf: Path | None) -> list[list[float]]:
    """페이지별 한글 텍스트 baseline Y (px @96dpi). pdf 없으면 pyhwpx 로 생성."""
    import fitz

    expect_pages: int | None = None
    if pdf is None:
        from pyhwpx import Hwp

        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
        hwp = Hwp(new=True, visible=False)
        hwp.open(str(src))
        expect_pages = int(hwp.PageCount)
        tmp = Path(tempfile.gettempdir()) / (src.stem + "_hangul.pdf")
        hwp.save_as(str(tmp), format="PDF")
        hwp.clear(option=1)
        hwp.quit()
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
        pdf = tmp

    doc = fitz.open(str(pdf))
    if expect_pages is not None and doc.page_count != expect_pages:
        print(
            f"경고: 생성 PDF {doc.page_count}쪽 != 편집기 PageCount {expect_pages}쪽. "
            f"인쇄 설정(맞춰찍기/배율)으로 비권위 레이아웃일 수 있음 — pdf/ 권위 PDF 사용 권장.",
            file=sys.stderr,
        )
    pages: list[list[float]] = []
    for page in doc:
        ys: set[float] = set()
        for block in page.get_text("dict")["blocks"]:
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    ys.add(round(span["origin"][1] * PT_TO_PX, 1))
        pages.append(sorted(ys))
    return pages


def rhwp_svg_baselines(src: Path, exe: str) -> list[list[float]]:
    """페이지별 rhwp 텍스트 baseline Y (px)."""
    with tempfile.TemporaryDirectory() as td:
        subprocess.run([exe, "export-svg", str(src), "-o", td],
                       capture_output=True, timeout=180)
        pages: list[list[float]] = []
        for svg in sorted(Path(td).glob("*.svg")):
            root = ET.parse(svg).getroot()
            ys = {round(float(t.get("y")), 1) for t in root.iter(f"{SVG_NS}text") if t.get("y")}
            pages.append(sorted(ys))
        return pages


def _cluster_rows(baselines: list[float], tol: float = 3.0) -> list[float]:
    """근접 baseline(같은 행의 다단 셀)을 한 행으로 병합 → 행 중심 목록.

    다단 표는 한 행의 여러 셀이 거의 같은 Y 에 baseline 을 가져, flatten 하면 가짜 작은 간격이
    생긴다. tol(px) 이내를 한 클러스터로 묶어 행 단위 pitch 를 정확히 한다.
    """
    if not baselines:
        return []
    bl = sorted(baselines)
    rows = [[bl[0]]]
    for y in bl[1:]:
        if y - rows[-1][-1] <= tol:
            rows[-1].append(y)
        else:
            rows.append([y])
    return [statistics.mean(r) for r in rows]


def median_pitch(baselines: list[float]) -> float | None:
    """행 클러스터 후 연속 행 간격의 median (줄 pitch). 다단 셀 오염 제거."""
    rows = _cluster_rows(baselines)
    if len(rows) < 2:
        return None
    diffs = [b - a for a, b in zip(rows, rows[1:]) if 5.0 < (b - a) < 60.0]
    return statistics.median(diffs) if diffs else None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("src", type=Path)
    ap.add_argument("--pdf", type=Path, default=None, help="기존 한글 PDF(없으면 pyhwpx 생성)")
    ap.add_argument("--compare", action="store_true", help="rhwp render 와 줄 pitch 대조")
    ap.add_argument("--exe", default="C:/Users/planet/rhwp/target/release/rhwp.exe"
                    if sys.platform == "win32" else "target/release/rhwp")
    ap.add_argument("-o", "--out", type=Path, default=None)
    a = ap.parse_args()

    hg = hangul_pdf_baselines(a.src, a.pdf)
    print(f"한글: {len(hg)}쪽")

    if not a.compare:
        rows = [("page", "n_lines", "median_pitch", "first", "last")]
        for i, bl in enumerate(hg):
            mp = median_pitch(bl)
            rows.append((str(i + 1), str(len(bl)),
                         f"{mp:.2f}" if mp else "-",
                         f"{bl[0]:.1f}" if bl else "-",
                         f"{bl[-1]:.1f}" if bl else "-"))
        for r in rows:
            print("\t".join(r))
        if a.out:
            a.out.write_text("\n".join("\t".join(r) for r in rows), encoding="utf-8")
        return 0

    rh = rhwp_svg_baselines(a.src, a.exe)
    print(f"rhwp: {len(rh)}쪽")
    print(f"{'page':>4} {'hg_lines':>8} {'rh_lines':>8} {'hg_pitch':>9} {'rh_pitch':>9} {'pitch_d':>8}")
    n = max(len(hg), len(rh))
    for i in range(n):
        hb = hg[i] if i < len(hg) else []
        rb = rh[i] if i < len(rh) else []
        hp = median_pitch(hb)
        rp = median_pitch(rb)
        d = (rp - hp) if (hp and rp) else None
        print(f"{i+1:>4} {len(hb):>8} {len(rb):>8} "
              f"{(f'{hp:.2f}' if hp else '-'):>9} {(f'{rp:.2f}' if rp else '-'):>9} "
              f"{(f'{d:+.2f}' if d is not None else '-'):>8}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
