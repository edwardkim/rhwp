"""#1759 razor-thin 메트릭 드리프트 서베이 — 줄 pitch / 표 bbox 의 rhwp vs 한글 정량 대조.

쪽 경계 razor-thin mismatch(수 px 누적 차)의 지배 원인 분해용 측정 배치. 파일별로:
- 한글측: pyhwpx 로 PDF 생성(배치 1회 Hwp 인스턴스) → PyMuPDF 페이지별 텍스트 baseline
  (행 클러스터 median pitch, hangul_pdf_baseline 재사용) + `find_tables` 표 bbox(pt→px ×96/72).
- rhwp측: `export-svg` 텍스트 baseline + `export-render-tree` Table bbox(pi/rows/cols).
- 표 매칭: 같은 페이지에서 폭 유사(<10%) + 가로 중심 근접 + 세로 겹침 — 근사 매칭이며
  미달은 unmatched 로 정직 표기(object_visual_regression 컨벤션).

산출(-o DIR):
- pages.tsv  : file / page / hg_lines / rh_lines / hg_pitch / rh_pitch / pitch_delta
- tables.tsv : file / page / pi / rows x cols / rh_w / rh_h / hg_w / hg_h / dw / dh
- summary.tsv: file / pages(rh=hg) / matched_tables / unmatched / mean|max pitch_delta / mean|max dh

사용:
    python tools/metric_drift_survey.py --list files.txt -o output/poc/drift
    python tools/metric_drift_survey.py --files a.hwpx b.hwp -o output/poc/drift

요구: Windows + 한컴오피스 + pyhwpx + PyMuPDF. rhwp release 바이너리.
알려진 한계: 표 매칭은 근사(무테 결재란 등 한글 find_tables 미검출), PDF 배율 경고
(PageCount 불일치) 파일은 SKIP 표기.
"""
from __future__ import annotations

import argparse
import csv
import json
import statistics
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from hangul_pdf_baseline import median_pitch, rhwp_svg_baselines  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
RHWP = ROOT / "target" / "release" / ("rhwp.exe" if sys.platform == "win32" else "rhwp")
PT_TO_PX = 96.0 / 72.0


def hangul_pdf(hwp, src: Path, pdf_dir: Path):
    """한글 PDF 생성(캐시). (pdf_path, hwp_page_count) 반환."""
    dst = pdf_dir / (src.stem[:60] + "_hangul.pdf")
    hwp.open(str(src))
    pages = int(hwp.PageCount)
    if not dst.exists():
        hwp.save_as(str(dst), format="PDF")
    hwp.clear(option=1)
    return dst, pages


def hangul_page_metrics(pdf: Path):
    """페이지별 (baseline 목록 px, 표 bbox 목록 px)."""
    import fitz

    doc = fitz.open(str(pdf))
    pages = []
    for page in doc:
        ys = set()
        for block in page.get_text("dict")["blocks"]:
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    ys.add(round(span["origin"][1] * PT_TO_PX, 1))
        tables = []
        try:
            for t in page.find_tables():
                x0, y0, x1, y1 = t.bbox
                tables.append(
                    (x0 * PT_TO_PX, y0 * PT_TO_PX, (x1 - x0) * PT_TO_PX, (y1 - y0) * PT_TO_PX)
                )
        except Exception:  # noqa: BLE001 — find_tables 개별 실패 격리
            pass
        pages.append((sorted(ys), tables))
    return pages


def rhwp_render_tree_tables(src: Path):
    """페이지별 rhwp Table 노드 [(pi, rows, cols, x, y, w, h)] (render-tree)."""
    with tempfile.TemporaryDirectory() as td:
        proc = subprocess.run(
            [str(RHWP), "export-render-tree", str(src), "-o", td],
            capture_output=True, timeout=300,
        )
        if proc.returncode != 0:
            return None
        pages = []
        for f in sorted(Path(td).glob("render_tree_*.json")):
            tree = json.loads(f.read_text(encoding="utf-8"))
            tables = []

            def walk(n):
                ty = n.get("type") or ""
                if isinstance(ty, dict):
                    ty = next(iter(ty))
                bb = n.get("bbox") or {}
                if str(ty) == "Table" and bb.get("h"):
                    tables.append(
                        (n.get("pi"), n.get("rows"), n.get("cols"),
                         bb["x"], bb["y"], bb["w"], bb["h"])
                    )
                for c in n.get("children", []):
                    walk(c)

            walk(tree.get("root") or tree)
            pages.append(tables)
        return pages


def match_tables(rh, hg):
    """폭 유사 + 가로 중심 근접 + 세로 겹침 그리디 매칭 → [(rh, hg|None)]."""
    used = set()
    out = []
    for r in rh:
        _, _, _, rx, ry, rw, rh_h = r
        best, best_d = None, 1e9
        for i, (hx, hy, hw, hh) in enumerate(hg):
            if i in used:
                continue
            if abs(hw - rw) > 0.10 * max(rw, hw):
                continue
            if abs((hx + hw / 2) - (rx + rw / 2)) > 25.0:
                continue
            if ry + rh_h < hy or hy + hh < ry:  # 세로 겹침 없음
                continue
            d = abs(hy - ry) + abs(hh - rh_h)
            if d < best_d:
                best, best_d = i, d
        if best is not None:
            used.add(best)
            out.append((r, hg[best]))
        else:
            out.append((r, None))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--files", nargs="+", type=Path)
    g.add_argument("--list", type=Path, help="파일 경로 목록 텍스트(줄당 1개)")
    ap.add_argument("-o", "--out", type=Path, required=True)
    a = ap.parse_args()

    if not RHWP.exists():
        print(f"오류: rhwp 바이너리 없음 {RHWP}", file=sys.stderr)
        return 2
    try:
        from pyhwpx import Hwp
        import fitz  # noqa: F401
    except ImportError as e:
        print(f"오류: 의존성 미설치 — {e}", file=sys.stderr)
        return 2

    files = a.files or [
        Path(line.strip()) for line in a.list.read_text(encoding="utf-8").splitlines() if line.strip()
    ]
    a.out.mkdir(parents=True, exist_ok=True)
    pdf_dir = a.out / "pdf"
    pdf_dir.mkdir(exist_ok=True)

    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    hwp = Hwp(new=True, visible=False)

    fp = open(a.out / "pages.tsv", "w", encoding="utf-8", newline="")
    ft = open(a.out / "tables.tsv", "w", encoding="utf-8", newline="")
    fs = open(a.out / "summary.tsv", "w", encoding="utf-8", newline="")
    wp = csv.writer(fp, delimiter="\t")
    wt = csv.writer(ft, delimiter="\t")
    ws = csv.writer(fs, delimiter="\t")
    wp.writerow(["file", "page", "hg_lines", "rh_lines", "hg_pitch", "rh_pitch", "pitch_delta"])
    wt.writerow(["file", "page", "pi", "size", "rh_w", "rh_h", "hg_w", "hg_h", "dw", "dh", "dy"])
    ws.writerow(["file", "status", "rh_pages", "hg_pages", "n_tables", "matched",
                 "mean_pitch_d", "max_pitch_d", "mean_dh", "max_dh"])

    for f in files:
        name = f.stem[:70]
        try:
            pdf, hg_count = hangul_pdf(hwp, f, pdf_dir)
            hg_pages = hangul_page_metrics(pdf)
        except Exception as e:  # noqa: BLE001 — 파일별 격리
            ws.writerow([name, f"ERR_HWP:{type(e).__name__}", "", "", "", "", "", "", "", ""])
            fs.flush()
            try:
                subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
                hwp = Hwp(new=True, visible=False)
            except Exception:
                pass
            continue
        if len(hg_pages) != hg_count:
            ws.writerow([name, "SKIP_PDF_SCALE", "", hg_count, "", "", "", "", "", ""])
            fs.flush()
            continue
        rh_base = rhwp_svg_baselines(f, str(RHWP))
        rh_tables = rhwp_render_tree_tables(f)
        if rh_tables is None or len(rh_base) != len(hg_pages):
            ws.writerow([name, "SKIP_PAGES_DIFFER", len(rh_base), len(hg_pages),
                         "", "", "", "", "", ""])
            fs.flush()
            continue

        pitch_ds, dhs, n_tab, n_match = [], [], 0, 0
        for i, ((hg_ys, hg_tabs), rh_ys) in enumerate(zip(hg_pages, rh_base)):
            hp = median_pitch(hg_ys)
            rp = median_pitch(rh_ys)
            d = (rp - hp) if (hp and rp) else None
            if d is not None:
                pitch_ds.append(d)
            wp.writerow([name, i + 1, len(hg_ys), len(rh_ys),
                         f"{hp:.2f}" if hp else "-", f"{rp:.2f}" if rp else "-",
                         f"{d:+.2f}" if d is not None else "-"])
            page_rh_tabs = rh_tables[i] if i < len(rh_tables) else []
            for r, h in match_tables(page_rh_tabs, hg_tabs):
                pi, rows, cols, _, ry, rw, rh_h = r
                n_tab += 1
                if h is None:
                    wt.writerow([name, i + 1, pi, f"{rows}x{cols}",
                                 f"{rw:.1f}", f"{rh_h:.1f}", "-", "-", "-", "-", "-"])
                    continue
                n_match += 1
                _, hy, hw, hh = h
                dhs.append(rh_h - hh)
                wt.writerow([name, i + 1, pi, f"{rows}x{cols}", f"{rw:.1f}", f"{rh_h:.1f}",
                             f"{hw:.1f}", f"{hh:.1f}", f"{rw - hw:+.1f}", f"{rh_h - hh:+.1f}",
                             f"{ry - hy:+.1f}"])
        ws.writerow([name, "OK", len(rh_base), hg_count, n_tab, n_match,
                     f"{statistics.mean(pitch_ds):+.2f}" if pitch_ds else "-",
                     f"{max(pitch_ds, key=abs):+.2f}" if pitch_ds else "-",
                     f"{statistics.mean(dhs):+.2f}" if dhs else "-",
                     f"{max(dhs, key=abs):+.2f}" if dhs else "-"])
        for h in (fp, ft, fs):
            h.flush()
        print(f"  {name}: pages={len(rh_base)} tables={n_match}/{n_tab} "
              f"pitch_d={statistics.mean(pitch_ds):+.2f}" if pitch_ds else f"  {name}: done",
              file=sys.stderr)

    for h in (fp, ft, fs):
        h.close()
    try:
        hwp.quit()
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    except Exception:
        pass
    print(f"→ {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
