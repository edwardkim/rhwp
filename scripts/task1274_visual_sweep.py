#!/usr/bin/env python3
"""Task 1274 PDF/SVG visual sweep helper."""

from __future__ import annotations

import argparse
import html as html_lib
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


FRAME_OVERFLOW_PIXEL_LIMIT = 20
FRAME_OVERFLOW_EXTRA_PIXEL_LIMIT = 12
FRAME_OVERFLOW_TOLERATED_BLEED_PX = 12
FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX = 6
CONTENT_BOTTOM_DELTA_LIMIT_PX = 36.0
RED_MARKER_DRIFT_LIMIT_PX = 18.0
RED_MARKER_CLUSTER_GAP_PX = 8
RED_MARKER_TEXT_MIN_HEIGHT_PX = 6.0
RED_MARKER_TEXT_MAX_HEIGHT_PX = 24.0
RED_MARKER_TEXT_MIN_PIXELS = 30.0
QUESTION_MARKER_COUNT_DELTA_LIMIT = 2
QUESTION_MARKER_FLOW_COUNT_DELTA_LIMIT = 3
QUESTION_MARKER_FLOW_MAX_DRIFT_PX = 180.0
QUESTION_MARKER_FLOW_SMALL_RED_MAX_PX = 80.0
QUESTION_MARKER_FLOW_LINE_MEAN_PX = 80.0
QUESTION_MARKER_FLOW_LARGE_DRIFT_PX = 180.0
LINE_BAND_DRIFT_LIMIT_PX = 42.0
LINE_BAND_DRIFT_MEAN_LIMIT_PX = 60.0
LINE_BAND_DRIFT_P90_LIMIT_PX = 120.0
COLUMN_LINE_DRIFT_MEAN_LIMIT_PX = 42.0
COLUMN_LINE_DRIFT_P90_LIMIT_PX = 70.0
EQUATION_OVERLAP_LIMIT = 0.08
EQUATION_OVERLAP_MIN_PX = 4.0
EQUATION_FLOW_LINE_OVERLAP_TOLERANCE_PX = 8.0
TEXT_RUN_INK_HEIGHT_LIMIT_PX = 16.0
LINE_ORDER_OVERLAP_LIMIT = 0.65
LINE_ORDER_OVERLAP_MIN_PX = 4.0
QUESTION_TITLE_OVERLAP_MIN_PX = 3.0
FRAME_TAIL_LINE_OVERFLOW_MIN_PX = 4.0
COLUMN_X_OVERLAP_LIMIT = 0.55
QUESTION_MARKER_Y_DRIFT_LIMIT_PX = 42.0
DEFAULT_PIXEL_DIFF_THRESHOLD = 32
LARGE_INK_TILE_SIZE = 16
LARGE_INK_TILE_MIN_PIXELS = 20
LARGE_INK_REGION_MIN_WIDTH_PX = 72.0
LARGE_INK_REGION_MIN_HEIGHT_PX = 48.0
LARGE_INK_REGION_DRIFT_LIMIT_PX = 80.0
ENDNOTE_SEPARATOR_MIN_RUN_PX = 70
ENDNOTE_SEPARATOR_GAP_DRIFT_LIMIT_PX = 18.0
QUESTION_TITLE_RE = re.compile(r"^\s*문\s*(\d+)")
CHOICE_MARKER_ONLY_RE = re.compile(r"^[①-⑳]+$")
PAGE_NUMBER_FOOTER_RE = re.compile(r"^\s*-\s*\d+\s*-\s*$")
PAPER_SIZE_FOOTER_RE = re.compile(
    r"^\s*\d+(?:\.\d+)?mm[×x]\d+(?:\.\d+)?mm\[.+\]\s*$"
)
PDF_PAGE_RE = re.compile(r'<page\s+[^>]*width="([0-9.]+)"\s+height="([0-9.]+)"')
PDF_WORD_RE = re.compile(
    r'<word\s+[^>]*xMin="([0-9.]+)"\s+yMin="([0-9.]+)"\s+'
    r'xMax="([0-9.]+)"\s+yMax="([0-9.]+)"[^>]*>(.*?)</word>'
)
RENDER_TREE_INVISIBLE_TEXT = "\U000f081c"


def strip_render_tree_invisible_text(text: str) -> str:
    return "".join(ch for ch in text if ch not in RENDER_TREE_INVISIBLE_TEXT)


@dataclass(frozen=True)
class Target:
    key: str
    hwp: Path
    pdf: Path


TARGETS = {
    "2022-09": Target(
        "2022-09",
        Path("samples/3-09월_교육_통합_2022.hwp"),
        Path("pdf/3-09월_교육_통합_2022.pdf"),
    ),
    "2023-09": Target(
        "2023-09",
        Path("samples/3-09월_교육_통합_2023.hwp"),
        Path("pdf/3-09월_교육_통합_2023.pdf"),
    ),
    "2024-09-below20": Target(
        "2024-09-below20",
        Path("samples/3-09월_교육_통합_2024-구분선아래20.hwp"),
        Path("pdf/3-09월_교육_통합_2024-구분선아래20-2024.pdf"),
    ),
    "2024-09-between20": Target(
        "2024-09-between20",
        Path("samples/3-09월_교육_통합_2024-미주사이20.hwp"),
        Path("pdf/3-09월_교육_통합_2024-미주사이20-2024.pdf"),
    ),
    "2024-09-below20-above20": Target(
        "2024-09-below20-above20",
        Path("samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp"),
        Path("pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf"),
    ),
    "2022-10": Target(
        "2022-10",
        Path("samples/3-10월_교육_통합_2022.hwp"),
        Path("pdf/3-10월_교육_통합_2022.pdf"),
    ),
    "2022-11-practice": Target(
        "2022-11-practice",
        Path("samples/3-11월_실전_통합_2022.hwp"),
        Path("pdf/3-11월_실전_통합_2022.pdf"),
    ),
    "2024-11-practice-shape987": Target(
        "2024-11-practice-shape987",
        Path("samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.pdf"),
    ),
    "2024-11-practice-above0-between0-below0": Target(
        "2024-11-practice-above0-between0-below0",
        Path("samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf"),
    ),
    "2024-11-practice-above0-between7-below2": Target(
        "2024-11-practice-above0-between7-below2",
        Path("samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.pdf"),
    ),
    "2024-11-practice-above0-between7-below20": Target(
        "2024-11-practice-above0-between7-below20",
        Path("samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.pdf"),
    ),
    "2024-11-practice-above0-between20-below2": Target(
        "2024-11-practice-above0-between20-below2",
        Path("samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.pdf"),
    ),
    "2024-11-practice-above20-between0-below20": Target(
        "2024-11-practice-above20-between0-below20",
        Path("samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.pdf"),
    ),
    "2024-11-practice-above20-between7-below2": Target(
        "2024-11-practice-above20-between7-below2",
        Path("samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.pdf"),
    ),
    "2024-11-practice-no-separator-above20-between20-below20": Target(
        "2024-11-practice-no-separator-above20-between20-below20",
        Path("samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp"),
        Path("pdf/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.pdf"),
    ),
}


def run(
    cmd: list[str],
    *,
    cwd: Path,
    log_path: Path | None = None,
    verbose: bool = True,
) -> subprocess.CompletedProcess[str]:
    if verbose:
        print("+ " + " ".join(cmd), flush=True)
    proc = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    if log_path is not None:
        log_path.parent.mkdir(parents=True, exist_ok=True)
        log_path.write_text(proc.stdout + proc.stderr, encoding="utf-8")
    if proc.returncode != 0:
        if proc.stdout:
            print(proc.stdout, file=sys.stdout)
        if proc.stderr:
            print(proc.stderr, file=sys.stderr)
        raise SystemExit(proc.returncode)
    return proc


def clean_dir(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def resolve_input_path(root: Path, path: Path) -> Path:
    return path if path.is_absolute() else root / path


def safe_rel_str(root: Path, path: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def safe_target_key(value: str) -> str:
    key = value.strip() or "custom"
    key = re.sub(r"[\\/:\s]+", "-", key)
    key = re.sub(r"[^0-9A-Za-z가-힣_.-]+", "-", key)
    key = re.sub(r"-+", "-", key).strip(".-")
    return key or "custom"


def parse_page_selection(page_values: list[int] | None, pages_values: list[str] | None) -> list[int] | None:
    selected: set[int] = set()
    for page in page_values or []:
        if page < 1:
            raise SystemExit("--page는 1 이상의 페이지 번호를 사용해야 합니다.")
        selected.add(page)

    for spec in pages_values or []:
        for part in spec.split(","):
            token = part.strip()
            if not token:
                continue
            if "-" in token:
                start_text, end_text = token.split("-", 1)
                try:
                    start = int(start_text.strip())
                    end = int(end_text.strip())
                except ValueError as exc:
                    raise SystemExit(f"--pages 범위를 해석할 수 없습니다: {token}") from exc
                if start < 1 or end < 1 or end < start:
                    raise SystemExit(f"--pages 범위가 올바르지 않습니다: {token}")
                selected.update(range(start, end + 1))
                continue
            try:
                page = int(token)
            except ValueError as exc:
                raise SystemExit(f"--pages 페이지 번호를 해석할 수 없습니다: {token}") from exc
            if page < 1:
                raise SystemExit("--pages는 1 이상의 페이지 번호를 사용해야 합니다.")
            selected.add(page)

    return sorted(selected) if selected else None


def filter_paths_by_pages(paths: list[Path], selected_pages: list[int] | None) -> list[Path]:
    if not selected_pages:
        return paths
    selected = set(selected_pages)
    return [path for path in paths if page_num(path) in selected]


def ensure_selected_pages_available(
    selected_pages: list[int],
    groups: dict[str, list[Path]],
) -> None:
    missing_by_group: dict[str, list[int]] = {}
    for group_name, paths in groups.items():
        available = {page_num(path) for path in paths}
        missing = [page for page in selected_pages if page not in available]
        if missing:
            missing_by_group[group_name] = missing
    if missing_by_group:
        details = ", ".join(
            f"{name}: {pages}" for name, pages in sorted(missing_by_group.items())
        )
        raise SystemExit(f"선택한 페이지의 산출물을 찾을 수 없습니다: {details}")


def use_singleton_page_fallback(
    selected_pages: list[int] | None,
    all_groups: dict[str, list[Path]],
    selected_groups: dict[str, list[Path]],
) -> bool:
    """단일 페이지 문서의 파일명 숫자가 문서번호일 때 선택 페이지 매칭을 보정한다."""
    if not selected_pages or len(selected_pages) != 1:
        return False
    if all(selected_groups.values()):
        return False
    return all(len(paths) == 1 for paths in all_groups.values())


def page_num(path: Path) -> int:
    matches = re.findall(r"(\d+)", path.stem)
    if not matches:
        return 1
    return int(matches[-1])


def ensure_tools() -> None:
    missing = [tool for tool in ("rsvg-convert", "pdftoppm", "pdftotext") if shutil.which(tool) is None]
    if missing:
        raise SystemExit("필수 도구가 없습니다: " + ", ".join(missing))


def load_note_shape(root: Path, hwp: Path, rhwp_bin: str, out_path: Path) -> dict[str, object]:
    proc = run([rhwp_bin, "dump-note-shape", str(hwp)], cwd=root)
    out_path.write_text(proc.stdout, encoding="utf-8")
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"미주 모양 JSON을 해석할 수 없습니다: {out_path}: {exc}") from exc


def compact_note_shape(note_shape: dict[str, object]) -> list[dict[str, object]]:
    compact: list[dict[str, object]] = []
    sections = note_shape.get("sections", [])
    if not isinstance(sections, list):
        return compact

    for section in sections:
        if not isinstance(section, dict):
            continue
        row: dict[str, object] = {"section": section.get("section")}
        for key, out_key in (("footnoteShape", "footnote"), ("endnoteShape", "endnote")):
            shape = section.get(key)
            if not isinstance(shape, dict):
                continue
            ui = shape.get("ui")
            raw = shape.get("raw")
            if not isinstance(ui, dict) or not isinstance(raw, dict):
                continue
            row[out_key] = {
                "separatorAboveMm": _note_mm(ui, "separatorAbove"),
                "separatorBelowMm": _note_mm(ui, "separatorBelow"),
                "betweenNotesMm": _note_mm(ui, "betweenNotes"),
                "separatorEnabled": bool(
                    raw.get("separatorLineType") and raw.get("separatorLineWidth")
                ),
                "separatorLengthMm": _note_mm(raw, "separatorLength"),
                "rawSeparatorMarginTopMm": _note_mm(raw, "separatorMarginTop"),
                "rawSeparatorMarginBottomMm": _note_mm(raw, "separatorMarginBottom"),
                "rawNoteSpacingMm": _note_mm(raw, "noteSpacing"),
                "rawUnknownMm": _note_mm(raw, "rawUnknown"),
            }
        compact.append(row)
    return compact


def _note_mm(shape_values: dict[str, object], key: str) -> float | None:
    value = shape_values.get(key)
    if not isinstance(value, dict):
        return None
    mm = value.get("mm")
    return mm if isinstance(mm, (int, float)) else None


def _note_hu(shape_values: dict[str, object], key: str) -> int | None:
    value = shape_values.get(key)
    if not isinstance(value, dict):
        return None
    hu = value.get("hu")
    return hu if isinstance(hu, int) else None


def first_endnote_shape(compact_shapes: list[dict[str, object]]) -> dict[str, object]:
    for section in compact_shapes:
        endnote = section.get("endnote")
        if isinstance(endnote, dict):
            return endnote
    return {}


def render_target(
    root: Path,
    target: Target,
    out_root: Path,
    rhwp_bin: str,
    dpi: int,
    pixel_diff_threshold: int,
    selected_pages: list[int] | None,
) -> dict[str, object]:
    print(f"== {target.key} ==", flush=True)
    hwp = resolve_input_path(root, target.hwp)
    pdf = resolve_input_path(root, target.pdf)
    if not hwp.exists():
        raise SystemExit(f"HWP 파일이 없습니다: {hwp}")
    if not pdf.exists():
        raise SystemExit(f"PDF 파일이 없습니다: {pdf}")

    base = out_root / safe_target_key(target.key)
    svg_dir = base / "svg"
    rhwp_png_dir = base / "rhwp_png"
    pdf_png_dir = base / "pdf_png"
    compare_dir = base / "compare"
    overlay_dir = base / "overlay"
    review_dir = base / "review"
    analysis_dir = base / "analysis"
    tree_dir = base / "render_tree"
    pdf_bbox_html = base / "pdf_bbox.html"
    clean_dir(svg_dir)
    clean_dir(rhwp_png_dir)
    clean_dir(pdf_png_dir)
    clean_dir(compare_dir)
    clean_dir(overlay_dir)
    clean_dir(review_dir)
    clean_dir(analysis_dir)
    clean_dir(tree_dir)

    note_shape_path = analysis_dir / "note_shape.json"
    note_shape = load_note_shape(root, hwp, rhwp_bin, note_shape_path)
    compact_shapes = compact_note_shape(note_shape)
    export_log = base / "export.log"
    run([rhwp_bin, "export-svg", str(hwp), "-o", str(svg_dir)], cwd=root, log_path=export_log)
    tree_log = base / "render_tree.log"
    run(
        [rhwp_bin, "export-render-tree", str(hwp), "-o", str(tree_dir)],
        cwd=root,
        log_path=tree_log,
    )

    pdf_prefix = pdf_png_dir / "pdf"
    run(["pdftoppm", "-r", str(dpi), "-png", str(pdf), str(pdf_prefix)], cwd=root)
    run(["pdftotext", "-bbox-layout", str(pdf), str(pdf_bbox_html)], cwd=root)

    all_svg_paths = sorted(svg_dir.glob("*.svg"), key=page_num)
    all_tree_paths = sorted(tree_dir.glob("*.json"), key=page_num)
    print(f"SVG pages: {len(all_svg_paths)}", flush=True)
    for svg in all_svg_paths:
        png = rhwp_png_dir / f"rhwp_{page_num(svg):03d}.png"
        run(["rsvg-convert", "-f", "png", "-o", str(png), str(svg)], cwd=root, verbose=False)

    all_rhwp_pngs = sorted(rhwp_png_dir.glob("*.png"), key=page_num)
    all_pdf_pngs = sorted(pdf_png_dir.glob("*.png"), key=page_num)
    pdf_question_markers = extract_pdf_question_markers(pdf_bbox_html, all_pdf_pngs)
    print(f"PDF pages: {len(all_pdf_pngs)}", flush=True)
    svg_paths = filter_paths_by_pages(all_svg_paths, selected_pages)
    tree_paths = filter_paths_by_pages(all_tree_paths, selected_pages)
    rhwp_pngs = filter_paths_by_pages(all_rhwp_pngs, selected_pages)
    pdf_pngs = filter_paths_by_pages(all_pdf_pngs, selected_pages)
    if selected_pages:
        print(f"Selected pages: {selected_pages}", flush=True)
        selected_groups = {
            "svg": svg_paths,
            "render_tree": tree_paths,
            "rhwp_png": rhwp_pngs,
            "pdf_png": pdf_pngs,
        }
        all_groups = {
            "svg": all_svg_paths,
            "render_tree": all_tree_paths,
            "rhwp_png": all_rhwp_pngs,
            "pdf_png": all_pdf_pngs,
        }
        if use_singleton_page_fallback(selected_pages, all_groups, selected_groups):
            print(
                (
                    "Selected page singleton fallback: 산출물 파일명 숫자가 선택 페이지와 "
                    "다르지만 모든 비교 그룹이 단일 페이지라 1:1로 매칭합니다."
                ),
                flush=True,
            )
            svg_paths = all_svg_paths
            tree_paths = all_tree_paths
            rhwp_pngs = all_rhwp_pngs
            pdf_pngs = all_pdf_pngs
        else:
            ensure_selected_pages_available(
                selected_pages,
                {
                    "svg": svg_paths,
                    "render_tree": tree_paths,
                    "rhwp_png": rhwp_pngs,
                    "pdf_png": pdf_pngs,
                },
            )
    compare_pages = make_compares(rhwp_pngs, pdf_pngs, compare_dir, target.key)
    overlay_result = make_overlay_compares(
        rhwp_pngs,
        pdf_pngs,
        overlay_dir,
        target.key,
        pixel_diff_threshold=pixel_diff_threshold,
    )
    contact = make_contact_sheet(compare_pages, base / "contact_sheet.png")
    overlay_contact = make_contact_sheet(
        overlay_result["pages"],
        base / "overlay_contact_sheet.png",
    ) if overlay_result["pages"] else None
    review_pages = make_review_panels(
        compare_pages,
        overlay_result["pages"],
        overlay_result["metrics"],
        review_dir,
    )
    review_contact = make_contact_sheet(
        review_pages,
        base / "review_contact_sheet.png",
    ) if review_pages else None
    visual_analysis = analyze_pages(
        rhwp_pngs,
        pdf_pngs,
        svg_paths,
        tree_paths,
        analysis_dir,
        target.key,
        pdf_question_markers,
        first_endnote_shape(compact_shapes),
    )

    log_text = export_log.read_text(encoding="utf-8") if export_log.exists() else ""
    overflow_lines = [
        line
        for line in log_text.splitlines()
        if "LAYOUT_OVERFLOW" in line or "overflow" in line.lower()
    ]
    manifest = {
        "key": target.key,
        "hwp": safe_rel_str(root, hwp),
        "pdf": safe_rel_str(root, pdf),
        "requested_pages": selected_pages,
        "exported_svg_pages": len(all_svg_paths),
        "exported_render_tree_pages": len(all_tree_paths),
        "exported_pdf_pages": len(all_pdf_pngs),
        "svg_pages": len(svg_paths),
        "render_tree_pages": len(tree_paths),
        "pdf_pages": len(pdf_pngs),
        "compare_pages": len(compare_pages),
        "overlay_pages": len(overlay_result["pages"]),
        "review_pages": len(review_pages),
        "pdf_question_markers": len(pdf_question_markers),
        "overflow_lines": overflow_lines,
        "contact_sheet": safe_rel_str(root, contact),
        "overlay_contact_sheet": safe_rel_str(root, overlay_contact)
        if overlay_contact is not None
        else None,
        "review_contact_sheet": safe_rel_str(root, review_contact)
        if review_contact is not None
        else None,
        "analysis_dir": safe_rel_str(root, analysis_dir),
        "overlay_dir": safe_rel_str(root, overlay_dir),
        "review_dir": safe_rel_str(root, review_dir),
        "note_shape": compact_shapes,
        "note_shape_json": safe_rel_str(root, note_shape_path),
        "overlay_metrics": overlay_result["summary"],
        "overlay_metrics_json": safe_rel_str(root, overlay_result["metrics_path"]),
        "visual_metrics": visual_analysis["summary"],
        "flagged_pages": visual_analysis["flagged_pages"],
    }
    (base / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return manifest


def is_content_pixel(pixel: tuple[int, int, int]) -> bool:
    r, g, b = pixel
    if r >= 244 and g >= 244 and b >= 244:
        return False
    return min(r, g, b) < 232 or max(r, g, b) - min(r, g, b) > 24


def is_dark_pixel(pixel: tuple[int, int, int]) -> bool:
    r, g, b = pixel
    return r < 110 and g < 110 and b < 110


def is_frame_line_pixel(pixel: tuple[int, int, int]) -> bool:
    r, g, b = pixel
    # rsvg-convert가 0.5px frame 선을 회색 antialias 픽셀로 내보내는 경우가 있다.
    return max(r, g, b) < 210 and max(r, g, b) - min(r, g, b) <= 12


def is_red_marker_pixel(pixel: tuple[int, int, int]) -> bool:
    r, g, b = pixel
    return r > 170 and g < 120 and b < 120 and r - max(g, b) > 45


def is_horizontal_rule_pixel(pixel: tuple[int, int, int]) -> bool:
    r, g, b = pixel
    if r >= 244 and g >= 244 and b >= 244:
        return False
    if is_red_marker_pixel(pixel):
        return False
    if r < 130 and g < 130 and b < 130:
        return True
    if g > 120 and r < 150 and b < 150 and g - max(r, b) > 20:
        return True
    return max(r, g, b) < 220 and max(r, g, b) - min(r, g, b) <= 18


def detect_frame(image: Image.Image) -> tuple[int, int, int, int]:
    rgb = image.convert("RGB")
    w, h = rgb.size
    px = rgb.load()

    row_counts = []
    for y in range(h):
        count = 0
        for x in range(w):
            if is_frame_line_pixel(px[x, y]):
                count += 1
        row_counts.append(count)

    col_counts = []
    for x in range(w):
        count = 0
        for y in range(h):
            if is_frame_line_pixel(px[x, y]):
                count += 1
        col_counts.append(count)

    top_candidates = [
        (count, y)
        for y, count in enumerate(row_counts[: max(1, h // 3)])
        if y > h * 0.03 and count > w * 0.45
    ]
    bottom_candidates = [
        (count, y)
        for y, count in enumerate(row_counts[int(h * 0.60) :], start=int(h * 0.60))
        if count > w * 0.45
    ]
    left_candidates = [
        (count, x)
        for x, count in enumerate(col_counts[: max(1, w // 3)])
        if x > w * 0.02 and count > h * 0.45
    ]
    right_candidates = [
        (count, x)
        for x, count in enumerate(col_counts[int(w * 0.60) :], start=int(w * 0.60))
        if count > h * 0.45
    ]

    top = max(top_candidates)[1] if top_candidates else round(h * 0.067)
    bottom = max(bottom_candidates, key=lambda item: item[1])[1] if bottom_candidates else round(h * 0.977)
    if bottom < h * 0.90:
        bottom = round(h * 0.977)
    left = max(left_candidates)[1] if left_candidates else round(w * 0.033)
    right = max(right_candidates)[1] if right_candidates else round(w * 0.967)
    return left, top, right, bottom


def horizontal_rule_candidates(
    image: Image.Image,
    *,
    frame: tuple[int, int, int, int],
    expected_length_px: float | None = None,
) -> list[dict[str, float]]:
    rgb = image.convert("RGB")
    px = rgb.load()
    left, top, right, bottom = frame
    frame_w = right - left
    if expected_length_px is not None and expected_length_px > 0.0:
        min_len = max(ENDNOTE_SEPARATOR_MIN_RUN_PX, int(expected_length_px * 0.55))
        max_len = max(min_len, int(expected_length_px * 1.35))
    else:
        min_len = ENDNOTE_SEPARATOR_MIN_RUN_PX
        max_len = max(min_len, int(frame_w * 0.58))
    raw_runs: list[dict[str, float]] = []
    y_start = max(0, top + 8)
    y_end = min(rgb.height - 1, bottom - 8)
    x_start = max(0, left + 4)
    x_end = min(rgb.width - 1, right - 4)

    for y in range(y_start, y_end + 1):
        x = x_start
        while x <= x_end:
            while x <= x_end and not is_horizontal_rule_pixel(px[x, y]):
                x += 1
            run_start = x
            while x <= x_end and is_horizontal_rule_pixel(px[x, y]):
                x += 1
            run_end = x - 1
            length = run_end - run_start + 1
            if min_len <= length <= max_len:
                raw_runs.append(
                    {
                        "x0": float(run_start),
                        "x1": float(run_end),
                        "y0": float(y),
                        "y1": float(y),
                        "length": float(length),
                    }
                )

    merged: list[dict[str, float]] = []
    for run in raw_runs:
        if (
            merged
            and run["y0"] - merged[-1]["y1"] <= 2
            and min(run["x1"], merged[-1]["x1"]) - max(run["x0"], merged[-1]["x0"]) >= min_len * 0.5
        ):
            band = merged[-1]
            band["x0"] = min(band["x0"], run["x0"])
            band["x1"] = max(band["x1"], run["x1"])
            band["y1"] = max(band["y1"], run["y1"])
            band["length"] = max(band["length"], run["length"])
        else:
            merged.append(dict(run))

    for band in merged:
        band["cy"] = round((band["y0"] + band["y1"]) / 2.0, 1)
        band["gap_height"] = round(band["y1"] - band["y0"] + 1.0, 1)
    return merged


def first_band_below(bands: list[dict[str, float]], y: float) -> dict[str, float] | None:
    for band in bands:
        if band["y0"] > y:
            return band
    return None


def endnote_separator_gap_measure(
    image: Image.Image,
    *,
    frame: tuple[int, int, int, int],
    expected_separator: bool,
    expected_length_px: float | None = None,
    candidates_override: list[dict[str, float]] | None = None,
    anchor_y: float | None = None,
) -> dict[str, object]:
    candidates = []
    if expected_separator:
        candidates = (
            candidates_override
            if candidates_override is not None
            else horizontal_rule_candidates(
                image,
                frame=frame,
                expected_length_px=expected_length_px,
            )
        )
    content_bands = row_bands(
        image,
        frame=frame,
        predicate=is_content_pixel,
        min_pixels_per_row=8,
        gap=2,
    )
    red_bands = row_bands(
        image,
        frame=frame,
        predicate=is_red_marker_pixel,
        min_pixels_per_row=3,
        gap=2,
    )
    if not expected_separator:
        return {
            "expected_separator": False,
            "candidate_count": len(candidates),
            "selected": None,
            "gap_px": None,
            "first_content_y": None,
            "first_marker_y": None,
            "candidates": [
                {key: round(value, 1) for key, value in candidate.items()}
                for candidate in candidates[-4:]
            ],
        }

    scored: list[tuple[float, dict[str, object]]] = []
    for candidate in candidates:
        if anchor_y is not None and abs(candidate["cy"] - anchor_y) > 90.0:
            continue
        content = first_band_below(content_bands, candidate["y1"] + 2.0)
        marker = first_band_below(red_bands, candidate["y1"] + 2.0)
        if content is None:
            continue
        marker_gap = marker["y0"] - candidate["y1"] if marker else None
        if marker_gap is None or marker_gap < 0 or marker_gap > 220:
            continue
        note_top_y = marker["y0"] if marker is not None else content["y0"]
        content_gap = note_top_y - candidate["y1"]
        # 첫 미주 marker 바로 위의 선을 우선한다. 같은 거리라면 더 아래쪽 후보가 separator일 가능성이 높다.
        length_score = 0.0
        if expected_length_px is not None and expected_length_px > 0.0:
            length_score = abs(candidate["length"] - expected_length_px) * 0.05
        anchor_score = abs(candidate["cy"] - anchor_y) * 0.2 if anchor_y is not None else 0.0
        score = marker_gap + length_score + anchor_score - candidate["y1"] * 0.001
        scored.append(
            (
                score,
                {
                    "candidate": candidate,
                    "content": content,
                    "marker": marker,
                    "note_top_y": note_top_y,
                    "gap_px": content_gap,
                    "marker_gap_px": marker_gap,
                },
            )
        )

    if not scored:
        return {
            "expected_separator": True,
            "candidate_count": len(candidates),
            "selected": None,
            "gap_px": None,
            "first_content_y": None,
            "first_marker_y": None,
            "candidates": [
                {key: round(value, 1) for key, value in candidate.items()}
                for candidate in candidates[-6:]
            ],
        }

    selected = min(scored, key=lambda item: item[0])[1]
    candidate = selected["candidate"]
    content = selected["content"]
    marker = selected["marker"]
    assert isinstance(candidate, dict)
    assert isinstance(content, dict)
    assert isinstance(marker, dict)
    return {
        "expected_separator": True,
        "candidate_count": len(candidates),
        "selected": {key: round(float(value), 1) for key, value in candidate.items()},
        "gap_px": round(float(selected["gap_px"]), 1),
        "marker_gap_px": round(float(selected["marker_gap_px"]), 1),
        "first_content_y": round(float(selected["note_top_y"]), 1),
        "first_marker_y": round(float(marker["y0"]), 1),
        "candidates": [
            {key: round(value, 1) for key, value in candidate.items()}
            for candidate in candidates[-6:]
        ],
    }


def lower_note_content_start(
    content_bands: list[dict[str, float]],
    red_bands: list[dict[str, float]],
    frame: tuple[int, int, int, int],
) -> dict[str, object]:
    left, top, right, bottom = frame
    lower_y = top + (bottom - top) * 0.45
    lower_content = [band for band in content_bands if band["y0"] >= lower_y]
    lower_red = [band for band in red_bands if band["y0"] >= lower_y]
    first_content = lower_content[0] if lower_content else None
    first_marker = lower_red[0] if lower_red else None
    start_candidates = [
        float(band["y0"])
        for band in (first_content, first_marker)
        if isinstance(band, dict)
    ]
    return {
        "frame_bottom_y": bottom,
        "content_band_count": len(content_bands),
        "marker_count": len(red_bands),
        "first_lower_content_y": round(float(first_content["y0"]), 1)
        if first_content is not None
        else None,
        "first_lower_marker_y": round(float(first_marker["y0"]), 1)
        if first_marker is not None
        else None,
        "content_start_y": round(min(start_candidates), 1) if start_candidates else None,
        "bottom_content_y": round(float(content_bands[-1]["y1"]), 1)
        if content_bands
        else None,
        "frame_width_px": right - left,
    }


def content_bounds(
    image: Image.Image,
    *,
    x_min: int,
    x_max: int,
    y_min: int,
    y_max: int,
) -> tuple[int, int, int, int, int] | None:
    rgb = image.convert("RGB")
    w, h = rgb.size
    px = rgb.load()
    x_min = max(0, x_min)
    x_max = min(w - 1, x_max)
    y_min = max(0, y_min)
    y_max = min(h - 1, y_max)
    found = False
    min_x = w
    min_y = h
    max_x = -1
    max_y = -1
    count = 0
    for y in range(y_min, y_max + 1):
        for x in range(x_min, x_max + 1):
            if is_content_pixel(px[x, y]):
                found = True
                count += 1
                min_x = min(min_x, x)
                min_y = min(min_y, y)
                max_x = max(max_x, x)
                max_y = max(max_y, y)
    if not found:
        return None
    return min_x, min_y, max_x, max_y, count


def row_bands(
    image: Image.Image,
    *,
    frame: tuple[int, int, int, int],
    predicate,
    min_pixels_per_row: int,
    gap: int = 2,
) -> list[dict[str, float]]:
    rgb = image.convert("RGB")
    px = rgb.load()
    left, top, right, bottom = frame
    rows: list[tuple[int, int, int, int]] = []
    for y in range(max(0, top + 2), min(rgb.height, bottom - 1)):
        xs = [x for x in range(max(0, left + 2), min(rgb.width, right - 1)) if predicate(px[x, y])]
        if len(xs) >= min_pixels_per_row:
            rows.append((y, min(xs), max(xs), len(xs)))
    bands: list[dict[str, float]] = []
    for y, min_x, max_x, count in rows:
        if not bands or y - bands[-1]["y1"] > gap:
            bands.append({"y0": y, "y1": y, "x0": min_x, "x1": max_x, "pixels": count})
        else:
            band = bands[-1]
            band["y1"] = y
            band["x0"] = min(band["x0"], min_x)
            band["x1"] = max(band["x1"], max_x)
            band["pixels"] += count
    for band in bands:
        band["cy"] = (band["y0"] + band["y1"]) / 2.0
    return bands


def cluster_marker_bands(
    bands: list[dict[str, float]],
    *,
    max_gap_px: int = RED_MARKER_CLUSTER_GAP_PX,
) -> list[dict[str, float]]:
    clusters: list[dict[str, float]] = []
    for band in bands:
        if not clusters or band["y0"] - clusters[-1]["y1"] > max_gap_px:
            clusters.append(dict(band))
            continue
        cluster = clusters[-1]
        cluster["y1"] = max(cluster["y1"], band["y1"])
        cluster["x0"] = min(cluster["x0"], band["x0"])
        cluster["x1"] = max(cluster["x1"], band["x1"])
        cluster["pixels"] += band["pixels"]
        cluster["cy"] = (cluster["y0"] + cluster["y1"]) / 2.0
    return clusters


def marker_text_bands(bands: list[dict[str, float]]) -> list[dict[str, float]]:
    filtered: list[dict[str, float]] = []
    for band in bands:
        height = band["y1"] - band["y0"] + 1.0
        if (
            RED_MARKER_TEXT_MIN_HEIGHT_PX <= height <= RED_MARKER_TEXT_MAX_HEIGHT_PX
            and band["pixels"] >= RED_MARKER_TEXT_MIN_PIXELS
        ):
            filtered.append(band)
    return filtered


def column_marker_text_bands(
    image: Image.Image,
    frame: tuple[int, int, int, int],
) -> list[dict[str, float]]:
    left, top, right, bottom = frame
    mid = int(round((left + right) / 2.0))
    markers: list[dict[str, float]] = []
    for column, (x0, x1) in enumerate(((left, mid), (mid, right))):
        if x1 - x0 < 24:
            continue
        raw = row_bands(
            image,
            frame=(x0, top, x1, bottom),
            predicate=is_red_marker_pixel,
            min_pixels_per_row=3,
            gap=2,
        )
        for band in marker_text_bands(cluster_marker_bands(raw)):
            item = dict(band)
            item["column"] = column
            markers.append(item)
    return markers


def compare_ordered_y(
    rhwp_bands: list[dict[str, float]],
    pdf_bands: list[dict[str, float]],
) -> dict[str, float | int | None]:
    count = min(len(rhwp_bands), len(pdf_bands))
    if count == 0:
        return {
            "rhwp_count": len(rhwp_bands),
            "pdf_count": len(pdf_bands),
            "paired": 0,
            "max_abs_delta_px": None,
            "mean_abs_delta_px": None,
        }
    deltas = [rhwp_bands[i]["cy"] - pdf_bands[i]["cy"] for i in range(count)]
    abs_deltas = [abs(delta) for delta in deltas]
    sorted_abs = sorted(abs_deltas)
    p90_index = min(len(sorted_abs) - 1, max(0, int(len(sorted_abs) * 0.9) - 1))
    return {
        "rhwp_count": len(rhwp_bands),
        "pdf_count": len(pdf_bands),
        "paired": count,
        "max_abs_delta_px": round(max(abs_deltas), 1),
        "p90_abs_delta_px": round(sorted_abs[p90_index], 1),
        "mean_abs_delta_px": round(sum(abs_deltas) / len(abs_deltas), 1),
    }

def column_frame(frame: tuple[int, int, int, int], column: int) -> tuple[int, int, int, int]:
    left, top, right, bottom = frame
    mid = (left + right) // 2
    if column == 0:
        return left, top, max(left, mid - 2), bottom
    return min(right, mid + 2), top, right, bottom


def column_line_band_drifts(
    rhwp: Image.Image,
    pdf: Image.Image,
    rhwp_frame: tuple[int, int, int, int],
    pdf_frame: tuple[int, int, int, int],
) -> list[dict[str, object]]:
    drifts: list[dict[str, object]] = []
    for column in (0, 1):
        rhwp_column_frame = column_frame(rhwp_frame, column)
        pdf_column_frame = column_frame(pdf_frame, column)
        rhwp_bands = row_bands(
            rhwp,
            frame=rhwp_column_frame,
            predicate=is_content_pixel,
            min_pixels_per_row=8,
            gap=2,
        )
        pdf_bands = row_bands(
            pdf,
            frame=pdf_column_frame,
            predicate=is_content_pixel,
            min_pixels_per_row=8,
            gap=2,
        )
        drift = compare_ordered_y(rhwp_bands, pdf_bands)
        drifts.append(
            {
                "column": column,
                "rhwp_frame": list(rhwp_column_frame),
                "pdf_frame": list(pdf_column_frame),
                "drift": drift,
                "rhwp_first_band": rhwp_bands[0] if rhwp_bands else None,
                "rhwp_last_band": rhwp_bands[-1] if rhwp_bands else None,
                "pdf_first_band": pdf_bands[0] if pdf_bands else None,
                "pdf_last_band": pdf_bands[-1] if pdf_bands else None,
            }
        )
    return drifts


def column_line_band_drift_candidates(drifts: list[dict[str, object]]) -> list[dict[str, object]]:
    candidates: list[dict[str, object]] = []
    for item in drifts:
        drift = item.get("drift")
        if not isinstance(drift, dict):
            continue
        mean = drift.get("mean_abs_delta_px")
        p90 = drift.get("p90_abs_delta_px")
        if not isinstance(mean, (int, float)) or not isinstance(p90, (int, float)):
            continue
        if mean >= COLUMN_LINE_DRIFT_MEAN_LIMIT_PX and p90 >= COLUMN_LINE_DRIFT_P90_LIMIT_PX:
            candidates.append(item)
    return candidates


def compare_adjacent_marker_gaps(
    rhwp_bands: list[dict[str, float]],
    pdf_bands: list[dict[str, float]],
    *,
    expected_between_notes_mm: object,
) -> dict[str, object]:
    def marker_y_by_column(bands: list[dict[str, float]]) -> list[list[float]]:
        columns = sorted(
            {int(band.get("column", 0)) for band in bands if isinstance(band, dict)}
        )
        if not columns:
            columns = [0]
        return [
            [
                round(float(band["cy"]), 1)
                for band in bands
                if int(band.get("column", 0)) == column
            ]
            for column in columns
        ]

    rhwp_y_by_column = marker_y_by_column(rhwp_bands)
    pdf_y_by_column = marker_y_by_column(pdf_bands)
    rhwp_y = [y for column in rhwp_y_by_column for y in column]
    pdf_y = [y for column in pdf_y_by_column for y in column]
    rhwp_gaps = [
        round(column[index + 1] - column[index], 1)
        for column in rhwp_y_by_column
        for index in range(max(0, len(column) - 1))
    ]
    pdf_gaps = [
        round(column[index + 1] - column[index], 1)
        for column in pdf_y_by_column
        for index in range(max(0, len(column) - 1))
    ]
    paired = min(len(rhwp_gaps), len(pdf_gaps))
    pairs = [
        {
            "prev_index": index,
            "next_index": index + 1,
            "rhwp_gap_px": rhwp_gaps[index],
            "pdf_gap_px": pdf_gaps[index],
            "delta_px": round(rhwp_gaps[index] - pdf_gaps[index], 1),
        }
        for index in range(paired)
    ]
    abs_deltas = [abs(float(pair["delta_px"])) for pair in pairs]
    expected_between_notes_px = mm_to_px(expected_between_notes_mm)
    return {
        "expected_between_notes_mm": expected_between_notes_mm
        if isinstance(expected_between_notes_mm, (int, float))
        else None,
        "expected_between_notes_px": round(expected_between_notes_px, 1)
        if expected_between_notes_px is not None
        else None,
        "rhwp_marker_count": len(rhwp_y),
        "pdf_marker_count": len(pdf_y),
        "paired_gap_count": paired,
        "max_abs_delta_px": round(max(abs_deltas), 1) if abs_deltas else None,
        "mean_abs_delta_px": round(sum(abs_deltas) / len(abs_deltas), 1)
        if abs_deltas
        else None,
        "rhwp_marker_y": rhwp_y,
        "pdf_marker_y": pdf_y,
        "rhwp_marker_y_by_column": rhwp_y_by_column,
        "pdf_marker_y_by_column": pdf_y_by_column,
        "rhwp_gaps_px": rhwp_gaps,
        "pdf_gaps_px": pdf_gaps,
        "pairs": pairs,
    }


def is_question_marker_flow_drift(
    red_drift: dict[str, float | int | None],
    line_drift: dict[str, float | int | None],
    large_region_drift: dict[str, object],
) -> bool:
    """문항 marker가 page/column 흐름 자체를 다르게 타는 강한 후보인지 판정한다."""
    rhwp_count = int(red_drift.get("rhwp_count") or 0)
    pdf_count = int(red_drift.get("pdf_count") or 0)
    count_delta = abs(rhwp_count - pdf_count)
    red_max = red_drift.get("max_abs_delta_px")
    line_mean = line_drift.get("mean_abs_delta_px")
    line_p90 = line_drift.get("p90_abs_delta_px")
    large_max = large_region_drift.get("max_abs_delta_px")
    large_count_delta = abs(
        int(large_region_drift.get("rhwp_count") or 0)
        - int(large_region_drift.get("pdf_count") or 0)
    )
    if count_delta < QUESTION_MARKER_FLOW_COUNT_DELTA_LIMIT:
        return False

    line_is_structural = (
        isinstance(line_mean, (int, float))
        and (
            line_mean >= QUESTION_MARKER_FLOW_LINE_MEAN_PX
            or (
                isinstance(line_p90, (int, float))
                and line_p90 >= LINE_BAND_DRIFT_P90_LIMIT_PX
            )
        )
    )
    large_is_structural = large_count_delta >= QUESTION_MARKER_COUNT_DELTA_LIMIT or (
        isinstance(large_max, (int, float))
        and large_max >= QUESTION_MARKER_FLOW_LARGE_DRIFT_PX
    )
    red_is_structural = count_delta >= QUESTION_MARKER_COUNT_DELTA_LIMIT or (
        isinstance(red_max, (int, float))
        and red_max >= QUESTION_MARKER_FLOW_MAX_DRIFT_PX
    )
    if (
        count_delta >= QUESTION_MARKER_COUNT_DELTA_LIMIT
        and isinstance(red_max, (int, float))
        and red_max < QUESTION_MARKER_FLOW_SMALL_RED_MAX_PX
        and not line_is_structural
    ):
        return False

    return red_is_structural and (line_is_structural or large_is_structural)


def large_ink_regions(
    image: Image.Image,
    *,
    frame: tuple[int, int, int, int],
) -> list[dict[str, float]]:
    left, top, right, bottom = frame
    tile = LARGE_INK_TILE_SIZE
    pixels = image.load()
    marked: set[tuple[int, int]] = set()
    grid_w = max(0, (right - left + tile - 1) // tile)
    grid_h = max(0, (bottom - top + tile - 1) // tile)

    for gy in range(grid_h):
        y0 = top + gy * tile
        y1 = min(bottom, y0 + tile)
        for gx in range(grid_w):
            x0 = left + gx * tile
            x1 = min(right, x0 + tile)
            count = 0
            for y in range(y0, y1):
                for x in range(x0, x1):
                    if is_content_pixel(pixels[x, y]):
                        count += 1
                        if count >= LARGE_INK_TILE_MIN_PIXELS:
                            marked.add((gx, gy))
                            break
                if (gx, gy) in marked:
                    break

    regions: list[dict[str, float]] = []
    seen: set[tuple[int, int]] = set()
    for start in sorted(marked, key=lambda item: (item[1], item[0])):
        if start in seen:
            continue
        stack = [start]
        seen.add(start)
        xs: list[int] = []
        ys: list[int] = []
        while stack:
            gx, gy = stack.pop()
            xs.append(gx)
            ys.append(gy)
            for nx, ny in ((gx - 1, gy), (gx + 1, gy), (gx, gy - 1), (gx, gy + 1)):
                neighbor = (nx, ny)
                if neighbor in marked and neighbor not in seen:
                    seen.add(neighbor)
                    stack.append(neighbor)

        x0 = left + min(xs) * tile
        y0 = top + min(ys) * tile
        x1 = min(right, left + (max(xs) + 1) * tile)
        y1 = min(bottom, top + (max(ys) + 1) * tile)
        width = float(x1 - x0)
        height = float(y1 - y0)
        frame_width = float(right - left)
        frame_height = float(bottom - top)
        if width < LARGE_INK_REGION_MIN_WIDTH_PX or height < LARGE_INK_REGION_MIN_HEIGHT_PX:
            continue
        if width >= frame_width * 0.85 and height >= frame_height * 0.80:
            continue
        if y1 <= top + 90:
            continue
        regions.append(
            {
                "x0": float(x0),
                "y0": float(y0),
                "x1": float(x1),
                "y1": float(y1),
                "w": width,
                "h": height,
                "cy": float((y0 + y1) / 2.0),
                "tiles": float(len(xs)),
            }
        )
    return regions


def compare_large_ink_regions(
    rhwp_regions: list[dict[str, float]],
    pdf_regions: list[dict[str, float]],
) -> dict[str, object]:
    count = min(len(rhwp_regions), len(pdf_regions))
    if count == 0:
        return {
            "rhwp_count": len(rhwp_regions),
            "pdf_count": len(pdf_regions),
            "paired": 0,
            "max_abs_delta_px": None,
            "mean_abs_delta_px": None,
            "rhwp_regions": rhwp_regions[:8],
            "pdf_regions": pdf_regions[:8],
        }

    deltas = [rhwp_regions[index]["cy"] - pdf_regions[index]["cy"] for index in range(count)]
    abs_deltas = [abs(delta) for delta in deltas]
    return {
        "rhwp_count": len(rhwp_regions),
        "pdf_count": len(pdf_regions),
        "paired": count,
        "deltas_px": [round(delta, 1) for delta in deltas],
        "max_abs_delta_px": round(max(abs_deltas), 1),
        "mean_abs_delta_px": round(sum(abs_deltas) / len(abs_deltas), 1),
        "rhwp_regions": [
            {key: round(value, 1) for key, value in region.items()}
            for region in rhwp_regions[:8]
        ],
        "pdf_regions": [
            {key: round(value, 1) for key, value in region.items()}
            for region in pdf_regions[:8]
        ],
    }


def bbox_overlap_ratio(a: tuple[float, float, float, float], b: tuple[float, float, float, float]) -> float:
    ax, ay, aw, ah = a
    bx, by, bw, bh = b
    x0 = max(ax, bx)
    y0 = max(ay, by)
    x1 = min(ax + aw, bx + bw)
    y1 = min(ay + ah, by + bh)
    if x1 <= x0 or y1 <= y0:
        return 0.0
    area = (x1 - x0) * (y1 - y0)
    return area / max(1.0, min(aw * ah, bw * bh))


def bbox_overlap_size(
    a: tuple[float, float, float, float],
    b: tuple[float, float, float, float],
) -> tuple[float, float]:
    ax, ay, aw, ah = a
    bx, by, bw, bh = b
    width = max(0.0, min(ax + aw, bx + bw) - max(ax, bx))
    height = max(0.0, min(ay + ah, by + bh) - max(ay, by))
    return width, height


def interval_overlap_ratio(a0: float, a1: float, b0: float, b1: float) -> float:
    overlap = min(a1, b1) - max(a0, b0)
    if overlap <= 0.0:
        return 0.0
    return overlap / max(1.0, min(a1 - a0, b1 - b0))


def bbox_x_overlap_ratio(a: tuple[float, float, float, float], b: tuple[float, float, float, float]) -> float:
    ax, _, aw, _ = a
    bx, _, bw, _ = b
    return interval_overlap_ratio(ax, ax + aw, bx, bx + bw)


def bbox_y_overlap_ratio(a: tuple[float, float, float, float], b: tuple[float, float, float, float]) -> float:
    _, ay, _, ah = a
    _, by, _, bh = b
    return interval_overlap_ratio(ay, ay + ah, by, by + bh)


def text_run_ink_bbox(box: tuple[float, float, float, float]) -> tuple[float, float, float, float]:
    x, y, w, h = box
    return (x, y, w, min(h, TEXT_RUN_INK_HEIGHT_LIMIT_PX))


def point_in_bbox(
    x: int,
    y: int,
    box: tuple[float, float, float, float],
    pad: float = 0.0,
) -> bool:
    bx, by, bw, bh = box
    return bx - pad <= x <= bx + bw + pad and by - pad <= y <= by + bh + pad


def image_ink_bbox(
    image: Image.Image | None,
    box: tuple[float, float, float, float],
    *,
    exclude_boxes: list[tuple[float, float, float, float]] | None = None,
) -> tuple[float, float, float, float] | None:
    if image is None:
        return None
    x, y, w, h = box
    left = max(0, int(x))
    top = max(0, int(y))
    right = min(image.width - 1, int(x + w + 0.999))
    bottom = min(image.height - 1, int(y + h + 0.999))
    if left > right or top > bottom:
        return None
    excludes = exclude_boxes or []
    min_x = min_y = 10**9
    max_x = max_y = -1
    pixels = image.load()
    for py in range(top, bottom + 1):
        for px in range(left, right + 1):
            if any(point_in_bbox(px, py, exclude, pad=1.0) for exclude in excludes):
                continue
            if is_content_pixel(pixels[px, py]):
                min_x = min(min_x, px)
                min_y = min(min_y, py)
                max_x = max(max_x, px)
                max_y = max(max_y, py)
    if max_x < min_x or max_y < min_y:
        return None
    return (float(min_x), float(min_y), float(max_x - min_x + 1), float(max_y - min_y + 1))


def path_parent_and_index(path: str) -> tuple[str, int] | None:
    if "/" not in path:
        return None
    parent, _, index_text = path.rpartition("/")
    try:
        return parent, int(index_text)
    except ValueError:
        return None


def is_adjacent_flow_equation_overlap(
    equation: dict[str, object],
    text_run: dict[str, object],
    overlap_px: float,
) -> bool:
    if overlap_px > EQUATION_FLOW_LINE_OVERLAP_TOLERANCE_PX:
        return False
    eq_line_path = equation.get("line_path")
    text_line_path = text_run.get("line_path")
    if not isinstance(eq_line_path, str) or not isinstance(text_line_path, str):
        return False
    eq_parent = path_parent_and_index(eq_line_path)
    text_parent = path_parent_and_index(text_line_path)
    if eq_parent is None or text_parent is None:
        return False
    if eq_parent[0] != text_parent[0] or eq_parent[1] != text_parent[1] + 1:
        return False
    eq_box = equation.get("bbox")
    text_box = text_run.get("bbox")
    if not isinstance(eq_box, tuple) or not isinstance(text_box, tuple):
        return False
    return eq_box[1] >= text_box[1]


def path_segments(path: object) -> list[str]:
    if not isinstance(path, str):
        return []
    return path.split("/")


def render_tree_sibling_box(
    node_boxes: dict[str, tuple[float, float, float, float]],
    segments: list[str],
    depth: int,
) -> tuple[str, tuple[float, float, float, float]] | None:
    if depth >= len(segments):
        return None
    sibling_path = "/".join(segments[: depth + 1])
    sibling_box = node_boxes.get(sibling_path)
    if sibling_box is None:
        return None
    return sibling_path, sibling_box


def is_column_sibling_boundary_false_positive(
    equation: dict[str, object],
    text_run: dict[str, object],
    node_boxes: dict[str, tuple[float, float, float, float]],
) -> bool:
    eq_segments = path_segments(equation.get("path"))
    text_segments = path_segments(text_run.get("path"))
    if len(eq_segments) < 4 or len(text_segments) < 4:
        return False
    common_len = 0
    for eq_part, text_part in zip(eq_segments, text_segments):
        if eq_part != text_part:
            break
        common_len += 1
    if common_len == 0 or common_len >= min(len(eq_segments), len(text_segments)):
        return False
    parent_path = "/".join(eq_segments[:common_len])
    parent_box = node_boxes.get(parent_path)
    eq_sibling = render_tree_sibling_box(node_boxes, eq_segments, common_len)
    text_sibling = render_tree_sibling_box(node_boxes, text_segments, common_len)
    if parent_box is None or eq_sibling is None or text_sibling is None:
        return False
    _, eq_col_box = eq_sibling
    _, text_col_box = text_sibling
    parent_h = parent_box[3]
    if parent_h <= 0.0:
        return False
    looks_like_columns = (
        eq_col_box[3] >= parent_h * 0.6
        and text_col_box[3] >= parent_h * 0.6
        and eq_col_box[2] >= 100.0
        and text_col_box[2] >= 100.0
        and abs(eq_col_box[1] - text_col_box[1]) <= 2.0
        and abs(eq_col_box[3] - text_col_box[3]) <= max(2.0, parent_h * 0.05)
    )
    if not looks_like_columns or eq_col_box[0] >= text_col_box[0]:
        return False
    eq_box = equation.get("bbox")
    text_box = text_run.get("bbox")
    if not isinstance(eq_box, tuple) or not isinstance(text_box, tuple):
        return False
    text_starts_column = abs(text_box[0] - text_col_box[0]) <= 2.0
    equation_bbox_crosses_boundary = eq_box[0] + eq_box[2] > text_col_box[0]
    return text_starts_column and equation_bbox_crosses_boundary


def render_tree_bbox(node: dict[str, object]) -> tuple[float, float, float, float] | None:
    bbox = node.get("bbox")
    if not isinstance(bbox, dict):
        return None
    try:
        x = float(bbox["x"])
        y = float(bbox["y"])
        w = float(bbox["w"])
        h = float(bbox["h"])
    except (KeyError, TypeError, ValueError):
        return None
    if w <= 0.0 or h <= 0.0:
        return None
    return x, y, w, h


def load_render_tree(tree_path: Path) -> dict[str, object] | None:
    if not tree_path.exists():
        return None
    try:
        tree = json.loads(tree_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None
    if isinstance(tree, dict) and isinstance(tree.get("tree"), dict):
        tree = tree["tree"]
    if not isinstance(tree, dict):
        return None
    return tree


def mm_to_px(mm: object, dpi: int = 96) -> float | None:
    if not isinstance(mm, (int, float)) or mm <= 0:
        return None
    return float(mm) / 25.4 * dpi


def render_tree_separator_candidates(
    tree_path: Path,
    *,
    frame: tuple[int, int, int, int],
    expected_length_px: float | None,
) -> list[dict[str, float]]:
    tree = load_render_tree(tree_path)
    if tree is None:
        return []
    left, top, right, bottom = frame
    candidates: list[dict[str, float]] = []

    def visit(node: dict[str, object]) -> None:
        bbox = render_tree_bbox(node)
        if bbox is not None and node.get("type") == "Line":
            x, y, w, h = bbox
            horizontal = w >= ENDNOTE_SEPARATOR_MIN_RUN_PX and h <= 5.0
            in_frame = left + 2 <= x <= right - 2 and top + 2 <= y <= bottom - 2
            length_matches = True
            if expected_length_px is not None and expected_length_px > 0.0:
                length_matches = expected_length_px * 0.55 <= w <= expected_length_px * 1.35
            if horizontal and in_frame and length_matches:
                candidates.append(
                    {
                        "x0": round(x, 1),
                        "x1": round(x + w, 1),
                        "y0": round(y, 1),
                        "y1": round(y + h, 1),
                        "length": round(w, 1),
                        "cy": round(y + h / 2.0, 1),
                        "gap_height": round(h, 1),
                    }
                )
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, dict):
                    visit(child)

    visit(tree)
    return candidates


def collect_render_tree_text_lines(
    tree: dict[str, object],
    *,
    include_visual_empty: bool = False,
) -> list[dict[str, object]]:
    lines: list[dict[str, object]] = []

    def line_text(node: dict[str, object]) -> str:
        parts: list[str] = []
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if not isinstance(child, dict):
                    continue
                if child.get("type") == "TextRun":
                    text = child.get("text")
                    if isinstance(text, str):
                        parts.append(text)
                elif child.get("type") == "Equation":
                    parts.append("[EQ]")
        return "".join(parts)

    def visit(node: dict[str, object], path: str) -> None:
        bbox = render_tree_bbox(node)
        if bbox is not None and node.get("type") == "TextLine":
            text = line_text(node)
            visible_text = strip_render_tree_invisible_text(text)
            visual_empty = include_visual_empty and not text.strip() and bbox[3] >= 20.0
            if visible_text.strip() or visual_empty:
                lines.append(
                    {
                        "path": path,
                        "bbox": bbox,
                        "pi": node.get("pi"),
                        "text": visible_text[:96] if visible_text.strip() else "[VISUAL]",
                    }
                )
        children = node.get("children")
        if isinstance(children, list):
            for index, child in enumerate(children):
                if isinstance(child, dict):
                    visit(child, f"{path}/{index}")

    visit(tree, "root")

    current_question: str | None = None
    current_question_text: str | None = None
    for line in lines:
        text = str(line.get("text", ""))
        match = QUESTION_TITLE_RE.match(text)
        if match:
            current_question = f"문{match.group(1)}"
            current_question_text = text
        line["question"] = current_question
        line["question_text"] = current_question_text
    return lines

def column_index(center_x: float, image_width: int) -> int:
    return 0 if center_x < image_width / 2.0 else 1


def extract_pdf_question_markers(pdf_bbox_html: Path, pdf_pngs: list[Path]) -> list[dict[str, object]]:
    if not pdf_bbox_html.exists():
        return []

    markers: list[dict[str, object]] = []
    page_index = -1
    page_width = 1.0
    page_height = 1.0
    image_width = 1
    image_height = 1
    try:
        lines = pdf_bbox_html.read_text(encoding="utf-8").splitlines()
    except OSError:
        return []

    for line in lines:
        page_match = PDF_PAGE_RE.search(line)
        if page_match:
            page_index += 1
            page_width = max(1.0, float(page_match.group(1)))
            page_height = max(1.0, float(page_match.group(2)))
            if page_index < len(pdf_pngs):
                with Image.open(pdf_pngs[page_index]) as image:
                    image_width, image_height = image.size
            continue

        word_match = PDF_WORD_RE.search(line)
        if not word_match or page_index < 0:
            continue
        text = html_lib.unescape(word_match.group(5)).strip()
        question_match = QUESTION_TITLE_RE.match(text)
        if not question_match:
            continue

        x0 = float(word_match.group(1)) * image_width / page_width
        y0 = float(word_match.group(2)) * image_height / page_height
        x1 = float(word_match.group(3)) * image_width / page_width
        y1 = float(word_match.group(4)) * image_height / page_height
        bbox = [round(x0, 1), round(y0, 1), round(x1 - x0, 1), round(y1 - y0, 1)]
        center_x = x0 + (x1 - x0) / 2.0
        markers.append(
            {
                "source": "pdf",
                "page": page_index + 1,
                "number": int(question_match.group(1)),
                "question": f"문{question_match.group(1)}",
                "text": text,
                "bbox": bbox,
                "column": column_index(center_x, image_width),
            }
        )
    return markers


def collect_render_tree_question_markers(
    tree_paths: list[Path],
    rhwp_pngs: list[Path],
    page_numbers: list[int] | None = None,
) -> list[dict[str, object]]:
    markers: list[dict[str, object]] = []
    for page_index, tree_path in enumerate(tree_paths):
        page_number = page_numbers[page_index] if page_numbers and page_index < len(page_numbers) else page_index + 1
        tree = load_render_tree(tree_path)
        if tree is None:
            continue
        image_width = 1
        if page_index < len(rhwp_pngs):
            with Image.open(rhwp_pngs[page_index]) as image:
                image_width = image.size[0]
        for line in collect_render_tree_text_lines(tree):
            text = str(line.get("text", ""))
            question_match = QUESTION_TITLE_RE.match(text)
            if not question_match:
                continue
            bbox = line.get("bbox")
            if not isinstance(bbox, tuple):
                continue
            x, _, w, _ = bbox
            markers.append(
                {
                    "source": "rhwp",
                    "page": page_number,
                    "number": int(question_match.group(1)),
                    "question": f"문{question_match.group(1)}",
                    "text": text[:96],
                    "pi": line.get("pi"),
                    "path": line.get("path"),
                    "bbox": [round(v, 1) for v in bbox],
                    "column": column_index(x + w / 2.0, image_width),
                }
            )
    return markers


def markers_by_question(markers: list[dict[str, object]]) -> dict[int, list[dict[str, object]]]:
    by_number: dict[int, list[dict[str, object]]] = {}
    for marker in markers:
        number = marker.get("number")
        if isinstance(number, int):
            by_number.setdefault(number, []).append(marker)
    return by_number


def marker_y(marker: dict[str, object]) -> float | None:
    bbox = marker.get("bbox")
    if not isinstance(bbox, list) or len(bbox) != 4:
        return None
    try:
        return float(bbox[1])
    except (TypeError, ValueError):
        return None


def marker_match_score(rhwp_marker: dict[str, object], pdf_marker: dict[str, object]) -> float:
    rhwp_page = int(rhwp_marker.get("page", 0))
    pdf_page = int(pdf_marker.get("page", 0))
    page_cost = abs(rhwp_page - pdf_page) * 2000.0
    column_cost = 350.0 if rhwp_marker.get("column") != pdf_marker.get("column") else 0.0
    rhwp_y = marker_y(rhwp_marker)
    pdf_y = marker_y(pdf_marker)
    y_cost = abs(rhwp_y - pdf_y) if rhwp_y is not None and pdf_y is not None else 500.0
    return page_cost + column_cost + y_cost


def build_question_marker_drifts(
    rhwp_markers: list[dict[str, object]],
    pdf_markers: list[dict[str, object]],
) -> dict[int, list[dict[str, object]]]:
    pdf_by_number = markers_by_question(pdf_markers)
    by_page: dict[int, list[dict[str, object]]] = {}

    for rhwp_marker in rhwp_markers:
        number = rhwp_marker.get("number")
        if not isinstance(number, int):
            continue
        pdf_candidates = pdf_by_number.get(number, [])
        pdf_marker = min(pdf_candidates, key=lambda item: marker_match_score(rhwp_marker, item)) if pdf_candidates else None
        reasons: list[str] = []
        y_delta: float | None = None
        page_delta: int | None = None

        if pdf_marker is None:
            reasons.append("missing_pdf_marker")
            page = int(rhwp_marker.get("page", 1))
        else:
            rhwp_page = int(rhwp_marker.get("page", 0))
            pdf_page = int(pdf_marker.get("page", 0))
            page_delta = rhwp_page - pdf_page
            if page_delta != 0:
                reasons.append("page_drift")
            if rhwp_marker.get("column") != pdf_marker.get("column"):
                reasons.append("column_drift")

            rhwp_bbox = rhwp_marker.get("bbox")
            pdf_bbox = pdf_marker.get("bbox")
            if isinstance(rhwp_bbox, list) and isinstance(pdf_bbox, list) and len(rhwp_bbox) == 4 and len(pdf_bbox) == 4:
                y_delta = float(rhwp_bbox[1]) - float(pdf_bbox[1])
                if abs(y_delta) >= QUESTION_MARKER_Y_DRIFT_LIMIT_PX:
                    reasons.append("y_drift")
            page = rhwp_page or pdf_page or 1

        if not reasons:
            continue

        candidate = {
            "number": number,
            "question": f"문{number}",
            "reasons": reasons,
            "page_delta": page_delta,
            "y_delta_px": round(y_delta, 1) if y_delta is not None else None,
            "rhwp_page": rhwp_marker.get("page"),
            "pdf_page": pdf_marker.get("page") if pdf_marker else None,
            "rhwp_column": rhwp_marker.get("column"),
            "pdf_column": pdf_marker.get("column") if pdf_marker else None,
            "rhwp_pi": rhwp_marker.get("pi"),
            "rhwp_text": rhwp_marker.get("text"),
            "pdf_text": pdf_marker.get("text") if pdf_marker else None,
            "rhwp_bbox": rhwp_marker.get("bbox"),
            "pdf_bbox": pdf_marker.get("bbox") if pdf_marker else None,
        }
        by_page.setdefault(page, []).append(candidate)

    for candidates in by_page.values():
        candidates.sort(
            key=lambda item: (
                abs(float(item["page_delta"] or 0)) * 1000.0,
                abs(float(item["y_delta_px"] or 0.0)),
            ),
            reverse=True,
        )
    return by_page


def render_tree_equation_overlap_candidates(
    tree_path: Path,
    rhwp_image_path: Path | None = None,
) -> list[dict[str, object]]:
    tree = load_render_tree(tree_path)
    if tree is None:
        return []
    rhwp_image = Image.open(rhwp_image_path).convert("RGB") if rhwp_image_path else None

    equations: list[dict[str, object]] = []
    text_runs: list[dict[str, object]] = []
    node_boxes: dict[str, tuple[float, float, float, float]] = {}

    def line_text(node: dict[str, object]) -> str:
        parts: list[str] = []
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if not isinstance(child, dict):
                    continue
                if child.get("type") == "TextRun":
                    text = child.get("text")
                    if isinstance(text, str):
                        parts.append(text)
                elif child.get("type") == "Equation":
                    parts.append("[EQ]")
        return "".join(parts)

    def is_equation_overlap_noise(
        equation: dict[str, object],
        text_run: dict[str, object],
        ratio: float,
    ) -> bool:
        eq_line_text = str(equation.get("line_text") or "")
        text_line_text = str(text_run.get("line_text") or "")
        text = str(text_run.get("text") or "")
        text_box = text_run["bbox"]
        assert isinstance(text_box, tuple)
        stripped = text.strip()
        if equation.get("line_pi") == text_run.get("pi"):
            return True
        if QUESTION_TITLE_RE.match(eq_line_text) or QUESTION_TITLE_RE.match(text_line_text):
            return True
        if "\ufffc" in text:
            return True
        if CHOICE_MARKER_ONLY_RE.match(stripped):
            return True
        if text_box[3] >= 20.0 and ratio < 0.12:
            return True
        return False

    def visit(
        node: dict[str, object],
        path: str,
        line_path: str | None = None,
        current_line_text: str = "",
        current_line_pi: object | None = None,
    ) -> None:
        bbox = render_tree_bbox(node)
        if bbox is not None:
            node_boxes[path] = bbox
        node_type = node.get("type")
        current_line_path = path if node_type == "TextLine" else line_path
        next_line_text = line_text(node) if node_type == "TextLine" else current_line_text
        next_line_pi = node.get("pi") if node_type == "TextLine" else current_line_pi
        if bbox is not None and node_type == "Equation":
            equations.append(
                {
                    "path": path,
                    "bbox": bbox,
                    "line_path": current_line_path,
                    "line_text": next_line_text,
                    "line_pi": next_line_pi,
                }
            )
        elif bbox is not None and node_type == "TextRun":
            text = node.get("text")
            if isinstance(text, str) and strip_render_tree_invisible_text(text).strip():
                text_runs.append(
                    {
                        "path": path,
                        "bbox": bbox,
                        "line_path": current_line_path,
                        "line_text": next_line_text,
                        "pi": node.get("pi"),
                        "line_pi": next_line_pi,
                        "text": strip_render_tree_invisible_text(text)[:32],
                    }
                )

        children = node.get("children")
        if isinstance(children, list):
            for index, child in enumerate(children):
                if isinstance(child, dict):
                    visit(child, f"{path}/{index}", current_line_path, next_line_text, next_line_pi)

    visit(tree, "root")

    candidates = []
    for eq_idx, equation in enumerate(equations):
        eq_box = equation["bbox"]
        assert isinstance(eq_box, tuple)
        for text_idx, text_run in enumerate(text_runs):
            text_box = text_run["bbox"]
            assert isinstance(text_box, tuple)
            if equation.get("line_path") == text_run.get("line_path"):
                continue
            # TextRun bbox는 줄 높이를 포함하므로, 바로 위 텍스트 줄의
            # 아래쪽 line box와 다음 수식 줄이 겹치는 정상 배치를 제외한다.
            if text_box[1] < eq_box[1] and (text_box[1] + text_box[3]) > eq_box[1]:
                continue
            adjusted_text_box = text_run_ink_bbox(text_box)
            coarse_ratio = bbox_overlap_ratio(eq_box, adjusted_text_box)
            coarse_overlap_px = min(
                eq_box[1] + eq_box[3],
                adjusted_text_box[1] + adjusted_text_box[3],
            ) - max(eq_box[1], adjusted_text_box[1])
            if is_equation_overlap_noise(equation, text_run, coarse_ratio):
                continue
            if coarse_ratio < EQUATION_OVERLAP_LIMIT or coarse_overlap_px < EQUATION_OVERLAP_MIN_PX:
                continue
            if is_column_sibling_boundary_false_positive(equation, text_run, node_boxes):
                continue
            text_exclude_boxes = [
                run["bbox"]
                for run in text_runs
                if isinstance(run.get("bbox"), tuple)
                and bbox_overlap_ratio(eq_box, run["bbox"]) > 0
            ]
            eq_ink_box = image_ink_bbox(
                rhwp_image,
                eq_box,
                exclude_boxes=text_exclude_boxes,
            )
            text_ink_box = image_ink_bbox(rhwp_image, adjusted_text_box)
            effective_eq_box = eq_ink_box or eq_box
            effective_text_box = text_ink_box or adjusted_text_box
            ratio = bbox_overlap_ratio(effective_eq_box, effective_text_box)
            overlap_width, overlap_height = bbox_overlap_size(effective_eq_box, effective_text_box)
            overlap_px = min(
                effective_eq_box[1] + effective_eq_box[3],
                effective_text_box[1] + effective_text_box[3],
            ) - max(effective_eq_box[1], effective_text_box[1])
            if is_adjacent_flow_equation_overlap(equation, text_run, overlap_px):
                continue
            if (
                ratio >= EQUATION_OVERLAP_LIMIT
                and overlap_width >= 3.0
                and overlap_height >= 2.5
                and overlap_px >= EQUATION_OVERLAP_MIN_PX
            ):
                candidates.append(
                    {
                        "equation_index": eq_idx,
                        "text_index": text_idx,
                        "overlap_ratio": round(ratio, 3),
                        "overlap_width_px": round(overlap_width, 1),
                        "overlap_height_px": round(overlap_height, 1),
                        "overlap_px": round(overlap_px, 1),
                        "equation_path": equation["path"],
                        "text_path": text_run["path"],
                        "text_pi": text_run.get("pi"),
                        "text": text_run.get("text"),
                        "equation_line_pi": equation.get("line_pi"),
                        "equation_line_text": equation.get("line_text"),
                        "text_line_text": text_run.get("line_text"),
                        "equation_bbox": [round(v, 1) for v in eq_box],
                        "text_bbox": [round(v, 1) for v in text_box],
                        "adjusted_text_bbox": [round(v, 1) for v in adjusted_text_box],
                        "equation_ink_bbox": [round(v, 1) for v in eq_ink_box]
                        if eq_ink_box
                        else None,
                        "text_ink_bbox": [round(v, 1) for v in text_ink_box]
                        if text_ink_box
                        else None,
                    }
                )
    candidates.sort(key=lambda item: item["overlap_ratio"], reverse=True)
    return candidates[:20]


def render_tree_question_title_overlap_candidates(tree_path: Path) -> list[dict[str, object]]:
    tree = load_render_tree(tree_path)
    if tree is None:
        return []

    lines = collect_render_tree_text_lines(tree)

    candidates: list[dict[str, object]] = []
    for index, title_line in enumerate(lines[:-1]):
        title_text = str(title_line.get("text", ""))
        if not QUESTION_TITLE_RE.match(title_text):
            continue
        next_line = lines[index + 1]
        title_box = title_line["bbox"]
        next_box = next_line["bbox"]
        assert isinstance(title_box, tuple)
        assert isinstance(next_box, tuple)
        ratio = bbox_overlap_ratio(title_box, next_box)
        _, overlap_height = bbox_overlap_size(title_box, next_box)
        if ratio >= 0.05 and overlap_height >= QUESTION_TITLE_OVERLAP_MIN_PX:
            candidates.append(
                {
                    "title_index": index,
                    "next_index": index + 1,
                    "overlap_ratio": round(ratio, 3),
                    "overlap_px": round(overlap_height, 1),
                    "title_path": title_line["path"],
                    "next_path": next_line["path"],
                    "title_pi": title_line.get("pi"),
                    "next_pi": next_line.get("pi"),
                    "title_text": title_text,
                    "next_text": next_line.get("text"),
                    "title_bbox": [round(v, 1) for v in title_box],
                    "next_bbox": [round(v, 1) for v in next_box],
                }
            )
    candidates.sort(key=lambda item: item["overlap_ratio"], reverse=True)
    return candidates[:20]


def render_tree_line_order_overlap_candidates(tree_path: Path) -> list[dict[str, object]]:
    tree = load_render_tree(tree_path)
    if tree is None:
        return []

    lines = collect_render_tree_text_lines(tree, include_visual_empty=True)
    candidates: list[dict[str, object]] = []
    for index, prev_line in enumerate(lines[:-1]):
        next_line = lines[index + 1]
        prev_box = prev_line["bbox"]
        next_box = next_line["bbox"]
        assert isinstance(prev_box, tuple)
        assert isinstance(next_box, tuple)
        prev_pi = prev_line.get("pi")
        next_pi = next_line.get("pi")
        if prev_pi is not None and prev_pi == next_pi:
            continue
        prev_text = str(prev_line.get("text") or "")
        next_text = str(next_line.get("text") or "")
        if prev_text == "[VISUAL]" and not QUESTION_TITLE_RE.match(next_text):
            continue
        if "[EQ]" in prev_text and QUESTION_TITLE_RE.match(next_text):
            continue
        if (
            next_text == "[VISUAL]"
            and prev_text != "[VISUAL]"
            and not QUESTION_TITLE_RE.match(prev_text)
            and next_box[1] < prev_box[1]
            and next_box[1] + next_box[3] <= prev_box[1] + prev_box[3] + LINE_ORDER_OVERLAP_MIN_PX
        ):
            continue
        if bbox_x_overlap_ratio(prev_box, next_box) < COLUMN_X_OVERLAP_LIMIT:
            continue
        px, py, pw, ph = prev_box
        nx, ny, nw, nh = next_box
        overlap_px = min(py + ph, ny + nh) - max(py, ny)
        if overlap_px < LINE_ORDER_OVERLAP_MIN_PX:
            continue
        y_ratio = bbox_y_overlap_ratio(prev_box, next_box)
        if y_ratio < LINE_ORDER_OVERLAP_LIMIT:
            continue
        candidates.append(
            {
                "prev_index": index,
                "next_index": index + 1,
                "question": next_line.get("question") or prev_line.get("question"),
                "question_text": next_line.get("question_text") or prev_line.get("question_text"),
                "overlap_ratio": round(y_ratio, 3),
                "overlap_px": round(overlap_px, 1),
                "y_delta": round(ny - py, 1),
                "prev_path": prev_line["path"],
                "next_path": next_line["path"],
                "prev_pi": prev_pi,
                "next_pi": next_pi,
                "prev_text": prev_line.get("text"),
                "next_text": next_line.get("text"),
                "prev_bbox": [round(v, 1) for v in prev_box],
                "next_bbox": [round(v, 1) for v in next_box],
            }
        )
    candidates.sort(key=lambda item: (item["overlap_ratio"], item["overlap_px"]), reverse=True)
    return candidates[:20]


def render_tree_frame_tail_candidates(
    tree_path: Path,
    frame: tuple[int, int, int, int],
) -> list[dict[str, object]]:
    tree = load_render_tree(tree_path)
    if tree is None:
        return []

    left, top, right, bottom = frame
    mid_x = (left + right) / 2.0
    candidates: list[dict[str, object]] = []
    for line in collect_render_tree_text_lines(tree):
        box = line["bbox"]
        assert isinstance(box, tuple)
        x, y, w, h = box
        if y < top or x + w < left + 2 or x > right - 2:
            continue
        overflow_px = y + h - bottom
        if overflow_px < FRAME_TAIL_LINE_OVERFLOW_MIN_PX:
            continue
        text = str(line.get("text") or "")
        stripped = strip_render_tree_invisible_text(text).replace("\ufffc", "").strip()
        if not stripped and "[EQ]" not in text:
            continue
        candidates.append(
            {
                "path": line["path"],
                "pi": line.get("pi"),
                "question": line.get("question"),
                "question_text": line.get("question_text"),
                "text": text[:96],
                "overflow_px": round(overflow_px, 1),
                "frame_bottom": bottom,
                "column": 0 if x + w / 2.0 < mid_x else 1,
                "bbox": [round(v, 1) for v in box],
            }
        )
    candidates.sort(key=lambda item: item["overflow_px"], reverse=True)
    return candidates[:20]


def suppress_tolerated_frame_tail_candidates(
    candidates: list[dict[str, object]],
    *,
    rhwp_out_pixels: int,
    rhwp_outside_frame_bleed_px: int,
    pdf_outside_frame_bleed_px: int,
    content_bottom_delta: float | None,
    question_marker_drifts: list[dict[str, object]],
) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    active: list[dict[str, object]] = []
    suppressed: list[dict[str, object]] = []

    marker_is_stable = not question_marker_drifts
    bottom_is_close = content_bottom_delta is None or abs(content_bottom_delta) < 16.0
    for item in candidates:
        overflow = float(item.get("overflow_px") or 0.0)
        bbox = item.get("bbox")
        text = str(item.get("text") or "")
        line_height = float(bbox[3]) if isinstance(bbox, list) and len(bbox) == 4 else 0.0
        small_bottom_bleed = overflow <= 6.0 and rhwp_out_pixels <= 300
        equation_line_height_bleed = overflow <= 12.0 and rhwp_out_pixels <= 10 and (
            "[EQ]" in text or line_height >= 20.0
        )
        actual_bottom_bleed_is_tolerated = (
            0 < rhwp_outside_frame_bleed_px <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
            and pdf_outside_frame_bleed_px <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
            and rhwp_out_pixels <= 300
        )
        actual_bottom_extent_is_tolerated = (
            0 < rhwp_outside_frame_bleed_px <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
            and pdf_outside_frame_bleed_px <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
        )
        equation_logical_box_bleed = (
            "[EQ]" in text
            and line_height > 0.0
            and overflow <= line_height + FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
            and actual_bottom_bleed_is_tolerated
            and (content_bottom_delta is None or abs(content_bottom_delta) < CONTENT_BOTTOM_DELTA_LIMIT_PX)
        )
        text_logical_box_bleed = (
            line_height > 0.0
            and overflow <= line_height
            and actual_bottom_extent_is_tolerated
            and (content_bottom_delta is None or abs(content_bottom_delta) < CONTENT_BOTTOM_DELTA_LIMIT_PX)
        )
        paper_size_footer_bleed = (
            PAPER_SIZE_FOOTER_RE.match(text) is not None
            and line_height > 0.0
            and overflow <= line_height + FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
            and abs(rhwp_outside_frame_bleed_px - pdf_outside_frame_bleed_px) <= 2
            and (content_bottom_delta is None or abs(content_bottom_delta) < 16.0)
        )
        page_number_footer_bleed = (
            PAGE_NUMBER_FOOTER_RE.match(text) is not None
            and line_height > 0.0
            and overflow <= 64.0
            and pdf_outside_frame_bleed_px >= rhwp_outside_frame_bleed_px - 2
            and (content_bottom_delta is None or abs(content_bottom_delta) < 16.0)
            and rhwp_out_pixels <= 128
        )

        if marker_is_stable and (
            (bottom_is_close and (small_bottom_bleed or equation_line_height_bleed))
            or equation_logical_box_bleed
            or text_logical_box_bleed
            or paper_size_footer_bleed
            or page_number_footer_bleed
        ):
            reason = "small_visual_tail_bleed"
            if paper_size_footer_bleed:
                reason = "paper_size_footer_bleed"
            elif page_number_footer_bleed:
                reason = "page_number_footer_bleed"
            suppressed.append({**item, "suppressed_reason": reason})
        else:
            active.append(item)

    return active, suppressed


def analyze_page(
    rhwp_path: Path,
    pdf_path: Path,
    svg_path: Path,
    tree_path: Path,
    analysis_dir: Path,
    key: str,
    page_index: int,
    question_marker_drifts: list[dict[str, object]],
    endnote_shape: dict[str, object],
) -> dict[str, object]:
    rhwp = Image.open(rhwp_path).convert("RGB")
    pdf = Image.open(pdf_path).convert("RGB")
    rhwp_frame = detect_frame(rhwp)
    pdf_frame = detect_frame(pdf)
    rl, rt, rr, rb = rhwp_frame
    pl, pt, pr, pb = pdf_frame

    rhwp_out = content_bounds(rhwp, x_min=rl + 2, x_max=rr - 2, y_min=rb + 3, y_max=rhwp.height - 1)
    pdf_out = content_bounds(pdf, x_min=pl + 2, x_max=pr - 2, y_min=pb + 3, y_max=pdf.height - 1)
    rhwp_inside = content_bounds(rhwp, x_min=rl + 2, x_max=rr - 2, y_min=rt + 2, y_max=rb - 2)
    pdf_inside = content_bounds(pdf, x_min=pl + 2, x_max=pr - 2, y_min=pt + 2, y_max=pb - 2)

    rhwp_red_raw = row_bands(
        rhwp,
        frame=rhwp_frame,
        predicate=is_red_marker_pixel,
        min_pixels_per_row=3,
        gap=2,
    )
    pdf_red_raw = row_bands(
        pdf,
        frame=pdf_frame,
        predicate=is_red_marker_pixel,
        min_pixels_per_row=3,
        gap=2,
    )
    rhwp_red_by_y = marker_text_bands(cluster_marker_bands(rhwp_red_raw))
    pdf_red_by_y = marker_text_bands(cluster_marker_bands(pdf_red_raw))
    rhwp_red = column_marker_text_bands(rhwp, rhwp_frame)
    pdf_red = column_marker_text_bands(pdf, pdf_frame)
    rhwp_bands = row_bands(
        rhwp,
        frame=rhwp_frame,
        predicate=is_content_pixel,
        min_pixels_per_row=8,
        gap=2,
    )
    pdf_bands = row_bands(
        pdf,
        frame=pdf_frame,
        predicate=is_content_pixel,
        min_pixels_per_row=8,
        gap=2,
    )
    red_drift = compare_ordered_y(rhwp_red, pdf_red)
    line_drift = compare_ordered_y(rhwp_bands, pdf_bands)
    column_line_drifts = column_line_band_drifts(rhwp, pdf, rhwp_frame, pdf_frame)
    column_line_drift_candidates = column_line_band_drift_candidates(column_line_drifts)
    large_region_drift = compare_large_ink_regions(
        large_ink_regions(rhwp, frame=rhwp_frame),
        large_ink_regions(pdf, frame=pdf_frame),
    )
    expected_separator = bool(endnote_shape.get("separatorEnabled"))
    endnote_shape_ui = {
        "separator_visible": expected_separator,
        "separator_above_mm": endnote_shape.get("separatorAboveMm"),
        "between_notes_mm": endnote_shape.get("betweenNotesMm"),
        "separator_below_mm": endnote_shape.get("separatorBelowMm"),
        "separator_length_mm": endnote_shape.get("separatorLengthMm"),
    }
    expected_separator_length_px = mm_to_px(endnote_shape.get("separatorLengthMm"))
    rhwp_separator_candidates = (
        render_tree_separator_candidates(
            tree_path,
            frame=rhwp_frame,
            expected_length_px=expected_separator_length_px,
        )
        if expected_separator
        else []
    )
    rhwp_separator_gap = endnote_separator_gap_measure(
        rhwp,
        frame=rhwp_frame,
        expected_separator=expected_separator,
        expected_length_px=expected_separator_length_px,
        candidates_override=rhwp_separator_candidates,
    )
    selected_rhwp_separator = rhwp_separator_gap.get("selected")
    separator_anchor_y = (
        selected_rhwp_separator.get("cy")
        if isinstance(selected_rhwp_separator, dict)
        and isinstance(selected_rhwp_separator.get("cy"), (int, float))
        else None
    )
    pdf_separator_gap = endnote_separator_gap_measure(
        pdf,
        frame=pdf_frame,
        expected_separator=expected_separator,
        expected_length_px=expected_separator_length_px,
        anchor_y=float(separator_anchor_y) if separator_anchor_y is not None else None,
    )
    no_separator_content_start = None
    if not expected_separator:
        no_separator_content_start = {
            "rhwp": lower_note_content_start(rhwp_bands, rhwp_red_by_y, rhwp_frame),
            "pdf": lower_note_content_start(pdf_bands, pdf_red_by_y, pdf_frame),
        }
    between_notes_marker_gap = compare_adjacent_marker_gaps(
        rhwp_red,
        pdf_red,
        expected_between_notes_mm=endnote_shape.get("betweenNotesMm"),
    )
    separator_gap_delta = None
    if isinstance(rhwp_separator_gap.get("gap_px"), (int, float)) and isinstance(
        pdf_separator_gap.get("gap_px"),
        (int, float),
    ):
        separator_gap_delta = round(
            float(rhwp_separator_gap["gap_px"]) - float(pdf_separator_gap["gap_px"]),
            1,
        )
    equation_overlaps = render_tree_equation_overlap_candidates(tree_path, rhwp_path)
    question_title_overlaps = render_tree_question_title_overlap_candidates(tree_path)
    line_order_overlaps = render_tree_line_order_overlap_candidates(tree_path)
    frame_tail_overflows = render_tree_frame_tail_candidates(tree_path, rhwp_frame)

    rhwp_out_pixels = rhwp_out[4] if rhwp_out else 0
    pdf_out_pixels = pdf_out[4] if pdf_out else 0
    rhwp_out_max_y = rhwp_out[3] if rhwp_out else None
    pdf_out_max_y = pdf_out[3] if pdf_out else None
    rhwp_outside_frame_bleed_px = (
        max(0, rhwp_out_max_y - rb) if rhwp_out_max_y is not None else 0
    )
    pdf_outside_frame_bleed_px = max(0, pdf_out_max_y - pb) if pdf_out_max_y is not None else 0
    rhwp_bottom = rhwp_inside[3] if rhwp_inside else None
    pdf_bottom = pdf_inside[3] if pdf_inside else None
    content_bottom_delta = None
    if rhwp_bottom is not None and pdf_bottom is not None:
        content_bottom_delta = round(float(rhwp_bottom - pdf_bottom), 1)
    frame_tail_overflows, suppressed_frame_tail_overflows = suppress_tolerated_frame_tail_candidates(
        frame_tail_overflows,
        rhwp_out_pixels=rhwp_out_pixels,
        rhwp_outside_frame_bleed_px=rhwp_outside_frame_bleed_px,
        pdf_outside_frame_bleed_px=pdf_outside_frame_bleed_px,
        content_bottom_delta=content_bottom_delta,
        question_marker_drifts=question_marker_drifts,
    )

    flags: list[str] = []
    rhwp_out_extent = None
    pdf_out_extent = None
    if rhwp_out_max_y is not None:
        rhwp_out_extent = int(rhwp_out_max_y - rb)
    if pdf_out_max_y is not None:
        pdf_out_extent = int(pdf_out_max_y - pb)
    paper_size_footer_frame_bleed = any(
        item.get("suppressed_reason") == "paper_size_footer_bleed"
        for item in suppressed_frame_tail_overflows
    ) and (
        abs(rhwp_outside_frame_bleed_px - pdf_outside_frame_bleed_px) <= 2
        and content_bottom_delta is not None
        and abs(content_bottom_delta) <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
        and rhwp_out_pixels <= pdf_out_pixels + 64
    )
    tolerated_rhwp_frame_bleed = (
        rhwp_out_extent is not None
        and 0 < rhwp_out_extent <= FRAME_OVERFLOW_TOLERATED_BLEED_PX
        and (content_bottom_delta is None or abs(content_bottom_delta) < CONTENT_BOTTOM_DELTA_LIMIT_PX)
    )
    content_bottom_matches = (
        content_bottom_delta is not None
        and abs(content_bottom_delta) <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
    )
    minor_rhwp_glyph_bleed = (
        rhwp_out_pixels > 0
        and rhwp_outside_frame_bleed_px <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
        and (
            pdf_outside_frame_bleed_px <= FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX
            or content_bottom_matches
        )
    )
    if (
        rhwp_out_pixels > max(FRAME_OVERFLOW_PIXEL_LIMIT, pdf_out_pixels + FRAME_OVERFLOW_EXTRA_PIXEL_LIMIT)
        and not tolerated_rhwp_frame_bleed
        and not minor_rhwp_glyph_bleed
        and not paper_size_footer_frame_bleed
    ):
        flags.append("frame_overflow_pixels")
    content_bottom_drift = content_bottom_delta is not None and abs(content_bottom_delta) >= CONTENT_BOTTOM_DELTA_LIMIT_PX
    red_counts_match = red_drift["rhwp_count"] == red_drift["pdf_count"]
    red_mean = red_drift["mean_abs_delta_px"]
    red_p90 = red_drift.get("p90_abs_delta_px")
    red_marker_drift_is_stable = (
        red_counts_match
        and red_drift["paired"] >= 2
        and red_mean is not None
        and red_p90 is not None
        and red_mean >= RED_MARKER_DRIFT_LIMIT_PX * 0.5
        and red_p90 >= RED_MARKER_DRIFT_LIMIT_PX
    )
    red_marker_drift = (
        red_drift["max_abs_delta_px"] is not None
        and red_drift["max_abs_delta_px"] >= RED_MARKER_DRIFT_LIMIT_PX
        and red_marker_drift_is_stable
    )
    line_mean = line_drift["mean_abs_delta_px"]
    line_p90 = line_drift.get("p90_abs_delta_px")
    line_band_drift = (
        line_mean is not None
        and (
            line_mean >= LINE_BAND_DRIFT_MEAN_LIMIT_PX
            or (
                line_p90 is not None
                and line_mean >= LINE_BAND_DRIFT_LIMIT_PX
                and line_p90 >= LINE_BAND_DRIFT_P90_LIMIT_PX
            )
        )
    )
    large_ink_region_drift = (
        (red_marker_drift or line_band_drift)
        and (
            large_region_drift["rhwp_count"] != large_region_drift["pdf_count"]
            or (
                large_region_drift["max_abs_delta_px"] is not None
                and large_region_drift["max_abs_delta_px"] >= LARGE_INK_REGION_DRIFT_LIMIT_PX
            )
        )
    )
    if is_question_marker_flow_drift(red_drift, line_drift, large_region_drift):
        flags.append("question_marker_flow_drift")
    if equation_overlaps:
        flags.append("equation_text_overlap")
    if (
        expected_separator
        and separator_gap_delta is not None
        and abs(separator_gap_delta) >= ENDNOTE_SEPARATOR_GAP_DRIFT_LIMIT_PX
    ):
        flags.append("endnote_separator_gap_drift")
    if question_title_overlaps:
        flags.append("question_title_text_overlap")
    if line_order_overlaps:
        flags.append("line_order_overlap")
    frame_tail_flow_overflow = bool(frame_tail_overflows and (column_line_drift_candidates or rhwp_out_pixels > 0))
    if frame_tail_flow_overflow:
        flags.append("render_tree_frame_tail_overflow")
    if question_marker_drifts:
        flags.append("question_marker_drift")
    semantic_flow_flags = bool(
        equation_overlaps
        or question_title_overlaps
        or line_order_overlaps
        or frame_tail_flow_overflow
        or question_marker_drifts
    )
    if content_bottom_drift and (rhwp_out_pixels > 0 or semantic_flow_flags):
        flags.append("content_bottom_drift")
    if red_marker_drift and question_marker_drifts:
        flags.append("red_marker_drift")
    if line_band_drift and semantic_flow_flags:
        flags.append("line_band_drift")
    if column_line_drift_candidates and semantic_flow_flags:
        flags.append("column_line_band_drift")
    if large_ink_region_drift and semantic_flow_flags:
        flags.append("large_ink_region_drift")

    annotated = None
    if flags:
        annotated_path = analysis_dir / f"annotated_{page_index + 1:03d}.png"
        annotated = make_annotation(
            rhwp,
            pdf,
            rhwp_frame,
            pdf_frame,
            rhwp_out,
            pdf_out,
            flags,
            key,
            page_index,
            annotated_path,
            {
                "equation_text_overlap": equation_overlaps,
                "question_title_text_overlap": question_title_overlaps,
                "line_order_overlap": line_order_overlaps,
                "render_tree_frame_tail_overflow": frame_tail_overflows,
                "question_marker_drift": question_marker_drifts,
                "column_line_band_drift": column_line_drift_candidates,
            },
        )

    return {
        "page": page_index + 1,
        "flags": flags,
        "rhwp_frame": list(rhwp_frame),
        "pdf_frame": list(pdf_frame),
        "rhwp_outside_frame_pixels": rhwp_out_pixels,
        "pdf_outside_frame_pixels": pdf_out_pixels,
        "rhwp_outside_frame_max_y": rhwp_out_max_y,
        "pdf_outside_frame_max_y": pdf_out_max_y,
        "rhwp_outside_frame_extent_px": rhwp_out_extent,
        "pdf_outside_frame_extent_px": pdf_out_extent,
        "frame_overflow_tolerated_bleed": tolerated_rhwp_frame_bleed,
        "paper_size_footer_frame_bleed": paper_size_footer_frame_bleed,
        "rhwp_outside_frame_bleed_px": rhwp_outside_frame_bleed_px,
        "pdf_outside_frame_bleed_px": pdf_outside_frame_bleed_px,
        "content_bottom_delta_px": content_bottom_delta,
        "red_marker_drift": red_drift,
        "line_band_drift": line_drift,
        "column_line_band_drift": column_line_drifts,
        "column_line_band_drift_candidates": column_line_drift_candidates,
        "large_ink_region_drift": large_region_drift,
        "endnote_shape_ui": endnote_shape_ui,
        "endnote_separator_gap": {
            "expected_separator": expected_separator,
            "separator_below_mm": endnote_shape.get("separatorBelowMm"),
            "separator_above_mm": endnote_shape.get("separatorAboveMm"),
            "between_notes_mm": endnote_shape.get("betweenNotesMm"),
            "separator_length_px": round(expected_separator_length_px, 1)
            if expected_separator_length_px is not None
            else None,
            "rhwp": rhwp_separator_gap,
            "pdf": pdf_separator_gap,
            "gap_delta_px": separator_gap_delta,
        },
        "endnote_no_separator_content_start": no_separator_content_start,
        "between_notes_marker_gap": between_notes_marker_gap,
        "svg": str(svg_path),
        "render_tree_json": str(tree_path),
        "equation_text_overlap_candidates": equation_overlaps,
        "question_title_text_overlap_candidates": question_title_overlaps,
        "line_order_overlap_candidates": line_order_overlaps,
        "render_tree_frame_tail_overflow_candidates": frame_tail_overflows,
        "render_tree_frame_tail_overflow_suppressed_candidates": suppressed_frame_tail_overflows,
        "question_marker_drift_candidates": question_marker_drifts,
        "annotated": str(annotated) if annotated else None,
    }


def make_annotation(
    rhwp: Image.Image,
    pdf: Image.Image,
    rhwp_frame: tuple[int, int, int, int],
    pdf_frame: tuple[int, int, int, int],
    rhwp_out: tuple[int, int, int, int, int] | None,
    pdf_out: tuple[int, int, int, int, int] | None,
    flags: list[str],
    key: str,
    page_index: int,
    out_path: Path,
    render_overlays: dict[str, list[dict[str, object]]] | None = None,
) -> Path:
    label_h = 40
    gutter = 16
    width = max(rhwp.width, pdf.width)
    height = max(rhwp.height, pdf.height)
    canvas = Image.new("RGB", (width * 2 + gutter, height + label_h), "white")
    canvas.paste(rhwp, (0, label_h))
    canvas.paste(pdf, (width + gutter, label_h))
    draw = ImageDraw.Draw(canvas)
    font = label_font()
    draw.text((8, 8), f"{key} p{page_index + 1:03d} rhwp flags={','.join(flags)}", fill=(180, 0, 0), font=font)
    draw.text((width + gutter + 8, 8), f"{key} p{page_index + 1:03d} pdf", fill=(20, 20, 20), font=font)
    for offset_x, frame, out in ((0, rhwp_frame, rhwp_out), (width + gutter, pdf_frame, pdf_out)):
        left, top, right, bottom = frame
        draw.rectangle(
            [offset_x + left, label_h + top, offset_x + right, label_h + bottom],
            outline=(0, 120, 255),
            width=2,
        )
        if out:
            x0, y0, x1, y1, _ = out
            draw.rectangle(
                [offset_x + x0, label_h + y0, offset_x + x1, label_h + y1],
                outline=(255, 0, 0),
                width=3,
            )
    if render_overlays:
        draw_render_tree_overlays(draw, label_h, render_overlays, width + gutter)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_path)
    return out_path


def draw_render_tree_overlays(
    draw: ImageDraw.ImageDraw,
    label_h: int,
    render_overlays: dict[str, list[dict[str, object]]],
    pdf_offset_x: int,
) -> None:
    font = label_font()

    for index, item in enumerate(render_overlays.get("column_line_band_drift", [])[:4]):
        drift = item.get("drift")
        if not isinstance(drift, dict):
            continue
        label = (
            f"column flow c{item.get('column')} "
            f"mean={drift.get('mean_abs_delta_px')} "
            f"p90={drift.get('p90_abs_delta_px')} "
            f"max={drift.get('max_abs_delta_px')}"
        )
        draw.text((18, label_h + 18 + index * 20), label, fill=(180, 0, 0), font=font)

    def draw_bbox(
        box: object,
        color: tuple[int, int, int],
        width: int = 3,
        *,
        offset_x: int = 0,
    ) -> tuple[float, float] | None:
        if not isinstance(box, list) or len(box) != 4:
            return None
        try:
            x, y, w, h = (float(v) for v in box)
        except (TypeError, ValueError):
            return None
        draw.rectangle(
            [offset_x + x, label_h + y, offset_x + x + w, label_h + y + h],
            outline=color,
            width=width,
        )
        return offset_x + x, label_h + y

    for item in render_overlays.get("question_marker_drift", [])[:8]:
        anchor = draw_bbox(item.get("rhwp_bbox"), (255, 0, 0), 3)
        draw_bbox(item.get("pdf_bbox"), (0, 140, 0), 3, offset_x=pdf_offset_x)
        if anchor is not None:
            x, y = anchor
            label = (
                f"{item.get('question')} "
                f"p {item.get('rhwp_page')} vs {item.get('pdf_page')} "
                f"dy={item.get('y_delta_px')} "
                f"{','.join(str(v) for v in item.get('reasons', []))}"
            )
            draw.text((x, max(label_h + 2, y - 18)), label, fill=(255, 0, 0), font=font)
    for item in render_overlays.get("line_order_overlap", [])[:6]:
        anchor = draw_bbox(item.get("prev_bbox"), (116, 59, 205), 3)
        draw_bbox(item.get("next_bbox"), (255, 128, 0), 3)
        if anchor is not None:
            x, y = anchor
            label = (
                f"line {item.get('question') or ''} "
                f"pi {item.get('prev_pi')}->{item.get('next_pi')} "
                f"r={item.get('overlap_ratio')}"
            )
            draw.text((x, max(label_h + 2, y - 18)), label, fill=(116, 59, 205), font=font)
    for item in render_overlays.get("render_tree_frame_tail_overflow", [])[:6]:
        anchor = draw_bbox(item.get("bbox"), (255, 0, 0), 3)
        if anchor is not None:
            x, y = anchor
            label = (
                f"frame tail pi {item.get('pi')} "
                f"c{item.get('column')} +{item.get('overflow_px')}px"
            )
            draw.text((x, max(label_h + 2, y - 18)), label, fill=(255, 0, 0), font=font)
    for item in render_overlays.get("equation_text_overlap", [])[:4]:
        anchor = draw_bbox(item.get("equation_bbox"), (255, 160, 0), 2)
        draw_bbox(item.get("text_bbox"), (220, 0, 160), 2)
        if anchor is not None:
            x, y = anchor
            label = f"eq/text pi {item.get('text_pi')} r={item.get('overlap_ratio')}"
            draw.text((x, max(label_h + 2, y - 18)), label, fill=(180, 80, 0), font=font)
    for item in render_overlays.get("question_title_text_overlap", [])[:4]:
        anchor = draw_bbox(item.get("title_bbox"), (0, 150, 180), 2)
        draw_bbox(item.get("next_bbox"), (220, 60, 0), 2)
        if anchor is not None:
            x, y = anchor
            label = f"title pi {item.get('title_pi')}->{item.get('next_pi')}"
            draw.text((x, max(label_h + 2, y - 18)), label, fill=(0, 120, 140), font=font)


def analyze_pages(
    rhwp_pngs: list[Path],
    pdf_pngs: list[Path],
    svg_paths: list[Path],
    tree_paths: list[Path],
    analysis_dir: Path,
    key: str,
    pdf_question_markers: list[dict[str, object]],
    endnote_shape: dict[str, object],
) -> dict[str, object]:
    page_count = min(len(rhwp_pngs), len(pdf_pngs), len(svg_paths), len(tree_paths))
    page_numbers = [page_num(path) for path in rhwp_pngs[:page_count]]
    rhwp_question_markers = collect_render_tree_question_markers(
        tree_paths[:page_count],
        rhwp_pngs[:page_count],
        page_numbers,
    )
    question_marker_drifts_by_page = build_question_marker_drifts(rhwp_question_markers, pdf_question_markers)

    question_flow_path = analysis_dir / "question_flow.json"
    question_flow_path.write_text(
        json.dumps(
            {
                "rhwp_question_markers": rhwp_question_markers,
                "pdf_question_markers": pdf_question_markers,
                "question_marker_drifts_by_page": question_marker_drifts_by_page,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    pages = [
        analyze_page(
            rhwp_pngs[index],
            pdf_pngs[index],
            svg_paths[index],
            tree_paths[index],
            analysis_dir,
            key,
            page_numbers[index] - 1,
            question_marker_drifts_by_page.get(page_numbers[index], []),
            endnote_shape,
        )
        for index in range(page_count)
    ]
    flagged_pages = [page for page in pages if page["flags"]]
    metrics_path = analysis_dir / "metrics.json"
    metrics_path.write_text(json.dumps(pages, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    summary = {
        "analyzed_pages": page_count,
        "flagged_page_count": len(flagged_pages),
        "frame_overflow_pages": [page["page"] for page in flagged_pages if "frame_overflow_pixels" in page["flags"]],
        "content_bottom_drift_pages": [page["page"] for page in flagged_pages if "content_bottom_drift" in page["flags"]],
        "red_marker_drift_pages": [page["page"] for page in flagged_pages if "red_marker_drift" in page["flags"]],
        "question_marker_flow_drift_pages": [
            page["page"] for page in flagged_pages if "question_marker_flow_drift" in page["flags"]
        ],
        "line_band_drift_pages": [page["page"] for page in flagged_pages if "line_band_drift" in page["flags"]],
        "column_line_band_drift_pages": [
            page["page"] for page in flagged_pages if "column_line_band_drift" in page["flags"]
        ],
        "large_ink_region_drift_pages": [
            page["page"] for page in flagged_pages if "large_ink_region_drift" in page["flags"]
        ],
        "endnote_separator_gap_drift_pages": [
            page["page"] for page in flagged_pages if "endnote_separator_gap_drift" in page["flags"]
        ],
        "endnote_separator_observed_pages": [
            page["page"]
            for page in pages
            if page["endnote_shape_ui"]["separator_visible"]
            and (
                page["endnote_separator_gap"]["rhwp"]["selected"]
                or page["endnote_separator_gap"]["pdf"]["selected"]
            )
        ],
        "endnote_separator_gap_pages": [
            page["page"]
            for page in pages
            if page["endnote_separator_gap"]["gap_delta_px"] is not None
        ],
        "endnote_no_separator_content_pages": [
            page["page"]
            for page in pages
            if not page["endnote_shape_ui"]["separator_visible"]
            and page["endnote_no_separator_content_start"] is not None
            and (
                page["endnote_no_separator_content_start"]["rhwp"]["content_start_y"] is not None
                or page["endnote_no_separator_content_start"]["pdf"]["content_start_y"] is not None
            )
        ],
        "between_notes_marker_gap_pages": [
            page["page"]
            for page in pages
            if page["between_notes_marker_gap"]["paired_gap_count"] > 0
        ],
        "equation_text_overlap_pages": [page["page"] for page in flagged_pages if "equation_text_overlap" in page["flags"]],
        "question_title_text_overlap_pages": [
            page["page"] for page in flagged_pages if "question_title_text_overlap" in page["flags"]
        ],
        "line_order_overlap_pages": [page["page"] for page in flagged_pages if "line_order_overlap" in page["flags"]],
        "render_tree_frame_tail_overflow_pages": [
            page["page"] for page in flagged_pages if "render_tree_frame_tail_overflow" in page["flags"]
        ],
        "question_marker_drift_pages": [
            page["page"] for page in flagged_pages if "question_marker_drift" in page["flags"]
        ],
        "metrics_json": str(metrics_path),
        "question_flow_json": str(question_flow_path),
    }
    flagged_path = analysis_dir / "flagged_pages.json"
    flagged_path.write_text(json.dumps(flagged_pages, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(
        f"analysis: {key} flagged={len(flagged_pages)}/{page_count} "
        f"frame={summary['frame_overflow_pages']} red={summary['red_marker_drift_pages']} "
        f"qflow={summary['question_marker_flow_drift_pages']} "
        f"line={summary['line_band_drift_pages']} column={summary['column_line_band_drift_pages']} "
        f"sep={summary['endnote_separator_gap_drift_pages']} "
        f"eq={summary['equation_text_overlap_pages']} "
        f"title={summary['question_title_text_overlap_pages']} "
        f"order={summary['line_order_overlap_pages']} "
        f"tail={summary['render_tree_frame_tail_overflow_pages']} "
        f"question={summary['question_marker_drift_pages']} "
        f"large={summary['large_ink_region_drift_pages']}",
        flush=True,
    )
    return {"summary": summary, "flagged_pages": flagged_pages}


def label_font() -> ImageFont.ImageFont:
    for font_path in (
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
    ):
        if Path(font_path).exists():
            return ImageFont.truetype(font_path, 18)
    return ImageFont.load_default()


def padded_pair(left_image: Image.Image, right_image: Image.Image) -> tuple[Image.Image, Image.Image]:
    width = max(left_image.width, right_image.width)
    height = max(left_image.height, right_image.height)
    left = Image.new("RGB", (width, height), "white")
    right = Image.new("RGB", (width, height), "white")
    left.paste(left_image.convert("RGB"), (0, 0))
    right.paste(right_image.convert("RGB"), (0, 0))
    return left, right


def overlay_color(
    rhwp_pixel: tuple[int, int, int],
    pdf_pixel: tuple[int, int, int],
    *,
    threshold: int,
) -> tuple[tuple[int, int, int], bool, bool, bool]:
    max_delta = max(abs(rhwp_pixel[channel] - pdf_pixel[channel]) for channel in range(3))
    rhwp_ink = is_content_pixel(rhwp_pixel)
    pdf_ink = is_content_pixel(pdf_pixel)
    union_ink = rhwp_ink or pdf_ink
    if max_delta <= threshold:
        avg = tuple((rhwp_pixel[channel] + pdf_pixel[channel]) // 2 for channel in range(3))
        gray = int(avg[0] * 0.299 + avg[1] * 0.587 + avg[2] * 0.114)
        return (gray, gray, gray), False, union_ink, False
    if rhwp_ink and not pdf_ink:
        return (255, 40, 40), True, union_ink, True
    if pdf_ink and not rhwp_ink:
        return (40, 100, 255), True, union_ink, True
    if rhwp_ink and pdf_ink:
        return (255, 150, 0), True, union_ink, True
    return (255, 190, 220), True, union_ink, False


def make_overlay_page(
    rhwp_path: Path,
    pdf_path: Path,
    out_path: Path,
    key: str,
    page_index: int,
    *,
    pixel_diff_threshold: int,
) -> dict[str, object]:
    rhwp_raw = Image.open(rhwp_path).convert("RGB")
    pdf_raw = Image.open(pdf_path).convert("RGB")
    rhwp, pdf = padded_pair(rhwp_raw, pdf_raw)
    width, height = rhwp.size
    overlay = Image.new("RGB", (width, height), "white")
    rhwp_px = rhwp.load()
    pdf_px = pdf.load()
    out_px = overlay.load()

    diff_pixels = 0
    ink_union_pixels = 0
    ink_diff_pixels = 0
    max_channel_delta = 0
    total_abs_delta = 0
    bbox_min_x = width
    bbox_min_y = height
    bbox_max_x = -1
    bbox_max_y = -1

    for y in range(height):
        for x in range(width):
            rhwp_pixel = rhwp_px[x, y]
            pdf_pixel = pdf_px[x, y]
            channel_deltas = [abs(rhwp_pixel[channel] - pdf_pixel[channel]) for channel in range(3)]
            max_delta = max(channel_deltas)
            max_channel_delta = max(max_channel_delta, max_delta)
            total_abs_delta += sum(channel_deltas)
            color, is_diff, union_ink, ink_diff = overlay_color(
                rhwp_pixel,
                pdf_pixel,
                threshold=pixel_diff_threshold,
            )
            out_px[x, y] = color
            if union_ink:
                ink_union_pixels += 1
            if is_diff:
                diff_pixels += 1
                bbox_min_x = min(bbox_min_x, x)
                bbox_min_y = min(bbox_min_y, y)
                bbox_max_x = max(bbox_max_x, x)
                bbox_max_y = max(bbox_max_y, y)
            if ink_diff:
                ink_diff_pixels += 1

    total_pixels = width * height
    diff_ratio = diff_pixels / total_pixels if total_pixels else 0.0
    ink_diff_ratio = ink_diff_pixels / ink_union_pixels if ink_union_pixels else 0.0
    pixel_match_percent = (1.0 - diff_ratio) * 100.0
    ink_match_percent = (1.0 - ink_diff_ratio) * 100.0 if ink_union_pixels else None
    visual_accuracy_proxy_percent = ink_match_percent if ink_match_percent is not None else pixel_match_percent
    diff_bbox = None
    if bbox_max_x >= 0:
        diff_bbox = [bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y]

    label_h = 42
    canvas = Image.new("RGB", (width, height + label_h), "white")
    canvas.paste(overlay, (0, label_h))
    draw = ImageDraw.Draw(canvas)
    font = label_font()
    ink_match_label = f"{ink_match_percent:.3f}%" if ink_match_percent is not None else "n/a"
    draw.text(
        (8, 6),
        (
            f"{key} p{page_index + 1:03d} overlay "
            f"pixel_match={pixel_match_percent:.3f}% "
            f"ink_match={ink_match_label} "
            f"diff={diff_pixels}/{total_pixels}"
        ),
        fill=(20, 20, 20),
        font=font,
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_path)

    return {
        "page": page_index + 1,
        "rhwp_png": str(rhwp_path),
        "pdf_png": str(pdf_path),
        "overlay_png": str(out_path),
        "width": width,
        "height": height,
        "pixel_diff_threshold": pixel_diff_threshold,
        "total_pixels": total_pixels,
        "diff_pixels": diff_pixels,
        "diff_ratio": round(diff_ratio, 8),
        "pixel_match_percent": round(pixel_match_percent, 5),
        "ink_union_pixels": ink_union_pixels,
        "ink_diff_pixels": ink_diff_pixels,
        "ink_diff_ratio": round(ink_diff_ratio, 8) if ink_union_pixels else None,
        "ink_match_percent": round(ink_match_percent, 5) if ink_match_percent is not None else None,
        "visual_accuracy_proxy_percent": round(visual_accuracy_proxy_percent, 5),
        "mean_abs_channel_delta": round(total_abs_delta / (total_pixels * 3), 3)
        if total_pixels
        else 0.0,
        "max_channel_delta": max_channel_delta,
        "diff_bbox": diff_bbox,
    }


def make_overlay_compares(
    rhwp_pngs: list[Path],
    pdf_pngs: list[Path],
    out_dir: Path,
    key: str,
    *,
    pixel_diff_threshold: int,
) -> dict[str, object]:
    count = min(len(rhwp_pngs), len(pdf_pngs))
    pages: list[Path] = []
    metrics: list[dict[str, object]] = []
    for index in range(count):
        page_number = page_num(rhwp_pngs[index])
        out = out_dir / f"overlay_{page_number:03d}.png"
        page_metrics = make_overlay_page(
            rhwp_pngs[index],
            pdf_pngs[index],
            out,
            key,
            page_number - 1,
            pixel_diff_threshold=pixel_diff_threshold,
        )
        pages.append(out)
        metrics.append(page_metrics)

    pixel_matches = [
        float(item["pixel_match_percent"])
        for item in metrics
        if isinstance(item.get("pixel_match_percent"), (int, float))
    ]
    ink_matches = [
        float(item["ink_match_percent"])
        for item in metrics
        if isinstance(item.get("ink_match_percent"), (int, float))
    ]
    proxy_matches = [
        float(item["visual_accuracy_proxy_percent"])
        for item in metrics
        if isinstance(item.get("visual_accuracy_proxy_percent"), (int, float))
    ]
    worst_pixel = min(pixel_matches) if pixel_matches else None
    worst_ink = min(ink_matches) if ink_matches else None
    worst_proxy = min(proxy_matches) if proxy_matches else None
    summary = {
        "compared_pages": count,
        "pixel_diff_threshold": pixel_diff_threshold,
        "average_pixel_match_percent": round(sum(pixel_matches) / len(pixel_matches), 5)
        if pixel_matches
        else None,
        "worst_pixel_match_percent": round(worst_pixel, 5)
        if worst_pixel is not None
        else None,
        "average_ink_match_percent": round(sum(ink_matches) / len(ink_matches), 5)
        if ink_matches
        else None,
        "worst_ink_match_percent": round(worst_ink, 5)
        if worst_ink is not None
        else None,
        "average_visual_accuracy_proxy_percent": round(sum(proxy_matches) / len(proxy_matches), 5)
        if proxy_matches
        else None,
        "worst_visual_accuracy_proxy_percent": round(worst_proxy, 5)
        if worst_proxy is not None
        else None,
        "worst_pages": [
            item["page"]
            for item in sorted(
                metrics,
                key=lambda row: float(row.get("visual_accuracy_proxy_percent", 100.0)),
            )[:10]
        ],
    }
    metrics_path = out_dir / "overlay_metrics.json"
    metrics_path.write_text(
        json.dumps({"summary": summary, "pages": metrics}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return {"pages": pages, "metrics": metrics, "metrics_path": metrics_path, "summary": summary}


def review_comment_line(metrics: dict[str, object] | None) -> str:
    percent = metrics.get("visual_accuracy_proxy_percent") if metrics else None
    if isinstance(percent, (int, float)):
        return f"코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 {percent:.2f}%."
    return "코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 확인 불가."


def make_review_panels(
    compare_pages: list[Path],
    overlay_pages: list[Path],
    overlay_metrics: list[dict[str, object]],
    out_dir: Path,
) -> list[Path]:
    overlays_by_page = {page_num(path): path for path in overlay_pages}
    metrics_by_page = {
        int(item["page"]): item
        for item in overlay_metrics
        if isinstance(item.get("page"), int)
    }
    review_pages: list[Path] = []
    gutter = 18
    footer_padding_x = 18
    footer_padding_y = 14
    font = label_font()
    for compare_path in compare_pages:
        page = page_num(compare_path)
        overlay_path = overlays_by_page.get(page)
        if overlay_path is None:
            continue
        compare = Image.open(compare_path).convert("RGB")
        overlay = Image.open(overlay_path).convert("RGB")
        width = compare.width + gutter + overlay.width
        image_height = max(compare.height, overlay.height)
        comment_line = review_comment_line(metrics_by_page.get(page))
        bbox = font.getbbox(comment_line)
        line_height = bbox[3] - bbox[1]
        overlay_footer_height = footer_padding_y * 2 + line_height
        height = max(image_height, overlay.height + overlay_footer_height)
        canvas = Image.new("RGB", (width, height), "white")
        canvas.paste(compare, (0, 0))
        overlay_x = compare.width + gutter
        canvas.paste(overlay, (overlay_x, 0))
        draw = ImageDraw.Draw(canvas)
        separator_y = overlay.height + 1
        draw.line(
            [(overlay_x, separator_y), (overlay_x + overlay.width, separator_y)],
            fill=(210, 210, 210),
            width=2,
        )
        draw.text(
            (overlay_x + footer_padding_x, overlay.height + footer_padding_y),
            comment_line,
            fill=(20, 20, 20),
            font=font,
        )
        out = out_dir / f"review_{page:03d}.png"
        out.parent.mkdir(parents=True, exist_ok=True)
        canvas.save(out)
        review_pages.append(out)
    return review_pages


def make_compares(rhwp_pngs: list[Path], pdf_pngs: list[Path], out_dir: Path, key: str) -> list[Path]:
    count = min(len(rhwp_pngs), len(pdf_pngs))
    font = label_font()
    pages: list[Path] = []
    for index in range(count):
        rhwp = Image.open(rhwp_pngs[index]).convert("RGB")
        pdf = Image.open(pdf_pngs[index]).convert("RGB")
        page_number = page_num(rhwp_pngs[index])
        width = max(rhwp.width, pdf.width)
        height = max(rhwp.height, pdf.height)
        label_h = 30
        gutter = 16
        canvas = Image.new("RGB", (width * 2 + gutter, height + label_h), "white")
        draw = ImageDraw.Draw(canvas)
        draw.text((8, 5), f"{key} p{page_number:03d} rhwp", fill=(20, 20, 20), font=font)
        draw.text((width + gutter + 8, 5), f"{key} p{page_number:03d} pdf", fill=(20, 20, 20), font=font)
        canvas.paste(rhwp, (0, label_h))
        canvas.paste(pdf, (width + gutter, label_h))
        out = out_dir / f"compare_{page_number:03d}.png"
        canvas.save(out)
        pages.append(out)
    return pages


def make_contact_sheet(compare_pages: list[Path], out_path: Path) -> Path:
    if not compare_pages:
        raise SystemExit("비교 PNG가 없습니다.")
    cols = 2
    thumb_w = 520
    gap = 14
    font = label_font()
    thumbs: list[Image.Image] = []
    for page in compare_pages:
        image = Image.open(page).convert("RGB")
        ratio = thumb_w / image.width
        thumb = image.resize((thumb_w, max(1, int(image.height * ratio))))
        labeled = Image.new("RGB", (thumb.width, thumb.height + 26), "white")
        labeled.paste(thumb, (0, 26))
        ImageDraw.Draw(labeled).text((4, 2), page.stem, fill=(20, 20, 20), font=font)
        thumbs.append(labeled)

    rows = (len(thumbs) + cols - 1) // cols
    row_h = max(t.height for t in thumbs)
    sheet = Image.new("RGB", (cols * thumb_w + (cols - 1) * gap, rows * row_h + (rows - 1) * gap), "white")
    for i, thumb in enumerate(thumbs):
        x = (i % cols) * (thumb_w + gap)
        y = (i // cols) * (row_h + gap)
        sheet.paste(thumb, (x, y))
    out_path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out_path)
    return out_path


def custom_targets_from_args(args: argparse.Namespace) -> list[Target]:
    targets: list[Target] = []
    if args.hwp or args.pdf:
        if not args.hwp or not args.pdf:
            raise SystemExit("--hwp와 --pdf는 함께 지정해야 합니다.")
        key = args.key or Path(args.hwp).stem
        targets.append(Target(safe_target_key(key), Path(args.hwp), Path(args.pdf)))

    for item in args.file_target or []:
        key, document_path, pdf_path = item
        targets.append(Target(safe_target_key(key), Path(document_path), Path(pdf_path)))
    return dedupe_target_keys(targets)


def dedupe_target_keys(targets: list[Target]) -> list[Target]:
    counts: dict[str, int] = {}
    deduped: list[Target] = []
    for target in targets:
        key = safe_target_key(target.key)
        counts[key] = counts.get(key, 0) + 1
        if counts[key] > 1:
            key = f"{key}-{counts[key]}"
        deduped.append(Target(key, target.hwp, target.pdf))
    return deduped


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--target",
        action="append",
        choices=[*TARGETS.keys(), "all"],
        help=(
            "검증할 preset target입니다. 여러 번 지정할 수 있습니다. "
            "target과 일반 파일 입력을 모두 생략하면 전체 preset을 실행합니다."
        ),
    )
    parser.add_argument("--out", default="output/task1274")
    parser.add_argument("--rhwp-bin", default="target/debug/rhwp")
    parser.add_argument("--dpi", type=int, default=96)
    parser.add_argument(
        "--page",
        action="append",
        type=int,
        help="비교할 1-based 페이지 번호입니다. 여러 번 지정할 수 있습니다.",
    )
    parser.add_argument(
        "--pages",
        action="append",
        help="비교할 1-based 페이지 목록/범위입니다. 예: 22 또는 43-46 또는 1,3,5-7",
    )
    parser.add_argument("--key", help="--hwp/--pdf로 넘긴 단일 외부 파일 target 이름입니다.")
    parser.add_argument("--hwp", help="임의 HWP/HWPX 파일 경로입니다. 절대 경로와 상대 경로를 모두 허용합니다.")
    parser.add_argument("--pdf", help="임의 기준 PDF 파일 경로입니다. 절대 경로와 상대 경로를 모두 허용합니다.")
    parser.add_argument(
        "--file-target",
        action="append",
        nargs=3,
        metavar=("KEY", "DOC", "PDF"),
        help="임의 파일 target을 추가합니다. 여러 번 지정할 수 있습니다.",
    )
    parser.add_argument(
        "--pixel-diff-threshold",
        type=int,
        default=DEFAULT_PIXEL_DIFF_THRESHOLD,
        help=(
            "PNG overlay diff에서 차이 픽셀로 볼 RGB 채널 최대 차이 임계값입니다 "
            f"(기본값: {DEFAULT_PIXEL_DIFF_THRESHOLD})."
        ),
    )
    args = parser.parse_args()
    if not 0 <= args.pixel_diff_threshold <= 255:
        raise SystemExit("--pixel-diff-threshold는 0 이상 255 이하로 지정해야 합니다.")
    selected_pages = parse_page_selection(args.page, args.pages)

    root = Path.cwd()
    ensure_tools()
    custom_targets = custom_targets_from_args(args)
    requested_targets = args.target
    if requested_targets and "all" in requested_targets:
        selected = list(TARGETS.values())
    elif requested_targets:
        selected = [TARGETS[target_key] for target_key in requested_targets]
    elif custom_targets:
        selected = []
    else:
        selected = list(TARGETS.values())
    selected = dedupe_target_keys([*selected, *custom_targets])
    out_root = root / args.out
    out_root.mkdir(parents=True, exist_ok=True)
    manifests = [
        render_target(
            root,
            target,
            out_root,
            args.rhwp_bin,
            args.dpi,
            args.pixel_diff_threshold,
            selected_pages,
        )
        for target in selected
    ]
    summary_path = out_root / "summary.json"
    summary_path.write_text(json.dumps(manifests, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"summary: {summary_path}")


if __name__ == "__main__":
    main()
