#!/usr/bin/env python3
"""Task 1274 PDF/SVG visual sweep helper."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


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
    missing = [tool for tool in ("rsvg-convert", "pdftoppm") if shutil.which(tool) is None]
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
    clean_dir(svg_dir)
    clean_dir(rhwp_png_dir)
    clean_dir(pdf_png_dir)
    clean_dir(compare_dir)

    export_log = base / "export.log"
    run([rhwp_bin, "export-svg", str(hwp), "-o", str(svg_dir)], cwd=root, log_path=export_log)

    pdf_prefix = pdf_png_dir / "pdf"
    run(["pdftoppm", "-r", str(dpi), "-png", str(pdf), str(pdf_prefix)], cwd=root)

    svg_paths = sorted(svg_dir.glob("*.svg"), key=page_num)
    print(f"SVG pages: {len(svg_paths)}", flush=True)
    for svg in svg_paths:
        png = rhwp_png_dir / f"rhwp_{page_num(svg):03d}.png"
        run(["rsvg-convert", "-f", "png", "-o", str(png), str(svg)], cwd=root, verbose=False)

    rhwp_pngs = sorted(rhwp_png_dir.glob("*.png"), key=page_num)
    pdf_pngs = sorted(pdf_png_dir.glob("*.png"), key=page_num)
    print(f"PDF pages: {len(pdf_pngs)}", flush=True)
    compare_pages = make_compares(rhwp_pngs, pdf_pngs, compare_dir, target.key)
    contact = make_contact_sheet(compare_pages, base / "contact_sheet.png")

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
        "pdf_pages": len(pdf_pngs),
        "compare_pages": len(compare_pages),
        "overflow_lines": overflow_lines,
        "contact_sheet": str(contact.relative_to(root)),
    }
    (base / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return manifest


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
