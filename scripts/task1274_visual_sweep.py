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
CONTENT_BOTTOM_DELTA_LIMIT_PX = 36.0
RED_MARKER_DRIFT_LIMIT_PX = 18.0
LINE_BAND_DRIFT_LIMIT_PX = 42.0
LINE_BAND_DRIFT_MEAN_LIMIT_PX = 60.0
LINE_BAND_DRIFT_P90_LIMIT_PX = 120.0
COLUMN_LINE_DRIFT_MEAN_LIMIT_PX = 42.0
COLUMN_LINE_DRIFT_P90_LIMIT_PX = 70.0
EQUATION_OVERLAP_LIMIT = 0.08
LINE_ORDER_OVERLAP_LIMIT = 0.65
LINE_ORDER_OVERLAP_MIN_PX = 4.0
FRAME_TAIL_LINE_OVERFLOW_MIN_PX = 4.0
COLUMN_X_OVERLAP_LIMIT = 0.55
QUESTION_MARKER_Y_DRIFT_LIMIT_PX = 42.0
QUESTION_TITLE_RE = re.compile(r"^\s*문\s*(\d+)")
CHOICE_MARKER_ONLY_RE = re.compile(r"^[①-⑳]+$")
PDF_PAGE_RE = re.compile(r'<page\s+[^>]*width="([0-9.]+)"\s+height="([0-9.]+)"')
PDF_WORD_RE = re.compile(
    r'<word\s+[^>]*xMin="([0-9.]+)"\s+yMin="([0-9.]+)"\s+'
    r'xMax="([0-9.]+)"\s+yMax="([0-9.]+)"[^>]*>(.*?)</word>'
)


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


def page_num(path: Path) -> int:
    matches = re.findall(r"(\d+)", path.stem)
    if not matches:
        raise ValueError(f"페이지 번호를 찾을 수 없습니다: {path}")
    return int(matches[-1])


def ensure_tools() -> None:
    missing = [tool for tool in ("rsvg-convert", "pdftoppm", "pdftotext") if shutil.which(tool) is None]
    if missing:
        raise SystemExit("필수 도구가 없습니다: " + ", ".join(missing))


def render_target(root: Path, target: Target, out_root: Path, rhwp_bin: str, dpi: int) -> dict[str, object]:
    print(f"== {target.key} ==", flush=True)
    hwp = root / target.hwp
    pdf = root / target.pdf
    if not hwp.exists():
        raise SystemExit(f"HWP 파일이 없습니다: {hwp}")
    if not pdf.exists():
        raise SystemExit(f"PDF 파일이 없습니다: {pdf}")

    base = out_root / target.key
    svg_dir = base / "svg"
    rhwp_png_dir = base / "rhwp_png"
    pdf_png_dir = base / "pdf_png"
    compare_dir = base / "compare"
    analysis_dir = base / "analysis"
    tree_dir = base / "render_tree"
    pdf_bbox_html = base / "pdf_bbox.html"
    clean_dir(svg_dir)
    clean_dir(rhwp_png_dir)
    clean_dir(pdf_png_dir)
    clean_dir(compare_dir)
    clean_dir(analysis_dir)
    clean_dir(tree_dir)

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

    svg_paths = sorted(svg_dir.glob("*.svg"), key=page_num)
    tree_paths = sorted(tree_dir.glob("*.json"), key=page_num)
    print(f"SVG pages: {len(svg_paths)}", flush=True)
    for svg in svg_paths:
        png = rhwp_png_dir / f"rhwp_{page_num(svg):03d}.png"
        run(["rsvg-convert", "-f", "png", "-o", str(png), str(svg)], cwd=root, verbose=False)

    rhwp_pngs = sorted(rhwp_png_dir.glob("*.png"), key=page_num)
    pdf_pngs = sorted(pdf_png_dir.glob("*.png"), key=page_num)
    pdf_question_markers = extract_pdf_question_markers(pdf_bbox_html, pdf_pngs)
    print(f"PDF pages: {len(pdf_pngs)}", flush=True)
    compare_pages = make_compares(rhwp_pngs, pdf_pngs, compare_dir, target.key)
    contact = make_contact_sheet(compare_pages, base / "contact_sheet.png")
    visual_analysis = analyze_pages(
        rhwp_pngs,
        pdf_pngs,
        svg_paths,
        tree_paths,
        analysis_dir,
        target.key,
        pdf_question_markers,
    )

    log_text = export_log.read_text(encoding="utf-8") if export_log.exists() else ""
    overflow_lines = [
        line
        for line in log_text.splitlines()
        if "LAYOUT_OVERFLOW" in line or "overflow" in line.lower()
    ]
    manifest = {
        "key": target.key,
        "hwp": str(target.hwp),
        "pdf": str(target.pdf),
        "svg_pages": len(svg_paths),
        "render_tree_pages": len(tree_paths),
        "pdf_pages": len(pdf_pngs),
        "compare_pages": len(compare_pages),
        "pdf_question_markers": len(pdf_question_markers),
        "overflow_lines": overflow_lines,
        "contact_sheet": str(contact.relative_to(root)),
        "analysis_dir": str(analysis_dir.relative_to(root)),
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


def is_red_marker_pixel(pixel: tuple[int, int, int]) -> bool:
    r, g, b = pixel
    return r > 170 and g < 120 and b < 120 and r - max(g, b) > 45


def detect_frame(image: Image.Image) -> tuple[int, int, int, int]:
    rgb = image.convert("RGB")
    w, h = rgb.size
    px = rgb.load()

    row_counts = []
    for y in range(h):
        count = 0
        for x in range(w):
            if is_dark_pixel(px[x, y]):
                count += 1
        row_counts.append(count)

    col_counts = []
    for x in range(w):
        count = 0
        for y in range(h):
            if is_dark_pixel(px[x, y]):
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
            visual_empty = include_visual_empty and not text.strip() and bbox[3] >= 20.0
            if text.strip() or visual_empty:
                lines.append(
                    {
                        "path": path,
                        "bbox": bbox,
                        "pi": node.get("pi"),
                        "text": text[:96] if text.strip() else "[VISUAL]",
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


def collect_render_tree_question_markers(tree_paths: list[Path], rhwp_pngs: list[Path]) -> list[dict[str, object]]:
    markers: list[dict[str, object]] = []
    for page_index, tree_path in enumerate(tree_paths):
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
                    "page": page_index + 1,
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


def render_tree_equation_overlap_candidates(tree_path: Path) -> list[dict[str, object]]:
    tree = load_render_tree(tree_path)
    if tree is None:
        return []

    equations: list[dict[str, object]] = []
    text_runs: list[dict[str, object]] = []

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
            if isinstance(text, str) and text.strip():
                text_runs.append(
                    {
                        "path": path,
                        "bbox": bbox,
                        "line_path": current_line_path,
                        "line_text": next_line_text,
                        "pi": node.get("pi"),
                        "text": text[:32],
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
            ratio = bbox_overlap_ratio(eq_box, text_box)
            overlap_width, overlap_height = bbox_overlap_size(eq_box, text_box)
            if is_equation_overlap_noise(equation, text_run, ratio):
                continue
            if ratio >= EQUATION_OVERLAP_LIMIT and overlap_width >= 3.0 and overlap_height >= 2.5:
                candidates.append(
                    {
                        "equation_index": eq_idx,
                        "text_index": text_idx,
                        "overlap_ratio": round(ratio, 3),
                        "overlap_width_px": round(overlap_width, 1),
                        "overlap_height_px": round(overlap_height, 1),
                        "equation_path": equation["path"],
                        "text_path": text_run["path"],
                        "text_pi": text_run.get("pi"),
                        "text": text_run.get("text"),
                        "equation_line_pi": equation.get("line_pi"),
                        "equation_line_text": equation.get("line_text"),
                        "text_line_text": text_run.get("line_text"),
                        "equation_bbox": [round(v, 1) for v in eq_box],
                        "text_bbox": [round(v, 1) for v in text_box],
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
        if ratio >= 0.05:
            candidates.append(
                {
                    "title_index": index,
                    "next_index": index + 1,
                    "overlap_ratio": round(ratio, 3),
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
        stripped = text.replace("\ufffc", "").strip()
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

        if marker_is_stable and bottom_is_close and (small_bottom_bleed or equation_line_height_bleed):
            suppressed.append({**item, "suppressed_reason": "small_visual_tail_bleed"})
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

    rhwp_red = row_bands(
        rhwp,
        frame=rhwp_frame,
        predicate=is_red_marker_pixel,
        min_pixels_per_row=3,
        gap=2,
    )
    pdf_red = row_bands(
        pdf,
        frame=pdf_frame,
        predicate=is_red_marker_pixel,
        min_pixels_per_row=3,
        gap=2,
    )
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
    equation_overlaps = render_tree_equation_overlap_candidates(tree_path)
    question_title_overlaps = render_tree_question_title_overlap_candidates(tree_path)
    line_order_overlaps = render_tree_line_order_overlap_candidates(tree_path)
    frame_tail_overflows = render_tree_frame_tail_candidates(tree_path, rhwp_frame)

    rhwp_out_pixels = rhwp_out[4] if rhwp_out else 0
    pdf_out_pixels = pdf_out[4] if pdf_out else 0
    rhwp_out_max_y = rhwp_out[3] if rhwp_out else None
    pdf_out_max_y = pdf_out[3] if pdf_out else None
    rhwp_bottom = rhwp_inside[3] if rhwp_inside else None
    pdf_bottom = pdf_inside[3] if pdf_inside else None
    content_bottom_delta = None
    if rhwp_bottom is not None and pdf_bottom is not None:
        content_bottom_delta = round(float(rhwp_bottom - pdf_bottom), 1)
    frame_tail_overflows, suppressed_frame_tail_overflows = suppress_tolerated_frame_tail_candidates(
        frame_tail_overflows,
        rhwp_out_pixels=rhwp_out_pixels,
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
    tolerated_rhwp_frame_bleed = (
        rhwp_out_extent is not None
        and 0 < rhwp_out_extent <= FRAME_OVERFLOW_TOLERATED_BLEED_PX
        and (content_bottom_delta is None or abs(content_bottom_delta) < CONTENT_BOTTOM_DELTA_LIMIT_PX)
    )

    if (
        rhwp_out_pixels > max(FRAME_OVERFLOW_PIXEL_LIMIT, pdf_out_pixels + FRAME_OVERFLOW_EXTRA_PIXEL_LIMIT)
        and not tolerated_rhwp_frame_bleed
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
    if equation_overlaps:
        flags.append("equation_text_overlap")
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
        "content_bottom_delta_px": content_bottom_delta,
        "red_marker_drift": red_drift,
        "line_band_drift": line_drift,
        "column_line_band_drift": column_line_drifts,
        "column_line_band_drift_candidates": column_line_drift_candidates,
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
) -> dict[str, object]:
    page_count = min(len(rhwp_pngs), len(pdf_pngs), len(svg_paths), len(tree_paths))
    rhwp_question_markers = collect_render_tree_question_markers(tree_paths[:page_count], rhwp_pngs[:page_count])
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
            index,
            question_marker_drifts_by_page.get(index + 1, []),
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
        "line_band_drift_pages": [page["page"] for page in flagged_pages if "line_band_drift" in page["flags"]],
        "column_line_band_drift_pages": [
            page["page"] for page in flagged_pages if "column_line_band_drift" in page["flags"]
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
        f"line={summary['line_band_drift_pages']} column={summary['column_line_band_drift_pages']} "
        f"eq={summary['equation_text_overlap_pages']} "
        f"title={summary['question_title_text_overlap_pages']} "
        f"order={summary['line_order_overlap_pages']} "
        f"tail={summary['render_tree_frame_tail_overflow_pages']} "
        f"question={summary['question_marker_drift_pages']}",
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


def make_compares(rhwp_pngs: list[Path], pdf_pngs: list[Path], out_dir: Path, key: str) -> list[Path]:
    count = min(len(rhwp_pngs), len(pdf_pngs))
    font = label_font()
    pages: list[Path] = []
    for index in range(count):
        rhwp = Image.open(rhwp_pngs[index]).convert("RGB")
        pdf = Image.open(pdf_pngs[index]).convert("RGB")
        width = max(rhwp.width, pdf.width)
        height = max(rhwp.height, pdf.height)
        label_h = 30
        gutter = 16
        canvas = Image.new("RGB", (width * 2 + gutter, height + label_h), "white")
        draw = ImageDraw.Draw(canvas)
        draw.text((8, 5), f"{key} p{index + 1:03d} rhwp", fill=(20, 20, 20), font=font)
        draw.text((width + gutter + 8, 5), f"{key} p{index + 1:03d} pdf", fill=(20, 20, 20), font=font)
        canvas.paste(rhwp, (0, label_h))
        canvas.paste(pdf, (width + gutter, label_h))
        out = out_dir / f"compare_{index + 1:03d}.png"
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


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", choices=[*TARGETS.keys(), "all"], default="all")
    parser.add_argument("--out", default="output/task1274")
    parser.add_argument("--rhwp-bin", default="target/debug/rhwp")
    parser.add_argument("--dpi", type=int, default=96)
    args = parser.parse_args()

    root = Path.cwd()
    ensure_tools()
    selected = TARGETS.values() if args.target == "all" else [TARGETS[args.target]]
    out_root = root / args.out
    out_root.mkdir(parents=True, exist_ok=True)
    manifests = [render_target(root, target, out_root, args.rhwp_bin, args.dpi) for target in selected]
    summary_path = out_root / "summary.json"
    summary_path.write_text(json.dumps(manifests, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"summary: {summary_path}")


if __name__ == "__main__":
    main()
