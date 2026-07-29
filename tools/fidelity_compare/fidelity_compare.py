"""한컴 기준 PDF와 rhwp SVG를 페이지별로 비교하는 하네스."""

from __future__ import annotations

import argparse
import base64
import glob
import html
import math
import os
import re
import shutil
import subprocess
import sys
import unicodedata
import xml.etree.ElementTree as ET
from collections import Counter
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
PNG_W = 700


@dataclass(frozen=True)
class Fixture:
    name: str
    source_pattern: str
    reference_pattern: str
    reference_grade: str


# 배경 셸의 cp949 argv 인코딩과 NFC/NFD 차이를 피하려고 ASCII 글롭만 쓴다.
# 기준 PDF의 위치와 등급은 README의 표와 함께 유지한다.
REG = {
    "plan": Fixture(
        "plan",
        "samples/2022* *.hwp",
        "pdf/2022* *-2022.pdf",
        "기준 PDF: pdf/ 보존 한컴 2022 출력",
    ),
    "manual": Fixture(
        "manual",
        "samples/2025 *.hwpx",
        "pdf/2025 *-2024.pdf",
        "기준 PDF: pdf/ 보존 한컴 2024 출력",
    ),
    "bunjang": Fixture(
        "bunjang",
        "samples/21868765*.hwp",
        "samples/21868765*.pdf",
        "참고 PDF: samples/ 동반본(버전·provenance 별도 확인 필요)",
    ),
    "korexam": Fixture(
        "korexam",
        "samples/21_*.hwp",
        "pdf/21_*-2022.pdf",
        "기준 PDF: pdf/ 보존 한컴 2022 출력",
    ),
    "math": Fixture(
        "math",
        "samples/exam_math.hwp",
        "pdf/exam_math-2022.pdf",
        "기준 PDF: pdf/ 보존 한컴 2022 출력",
    ),
    "eng": Fixture(
        "eng",
        "samples/exam_eng.hwp",
        "pdf/exam_eng-2022.pdf",
        "기준 PDF: pdf/ 보존 한컴 2022 출력",
    ),
}


def resolve(repo: Path, pattern: str) -> Path:
    hits = sorted(glob.glob(str(repo / pattern)), key=lambda hit: (len(hit), hit))
    if not hits:
        raise FileNotFoundError(f"글롭 미해결: {pattern}")
    return Path(hits[0])


def _resolve_override(value: str) -> str | None:
    discovered = shutil.which(value)
    if discovered:
        return discovered
    expanded = Path(value).expanduser()
    if expanded.is_file():
        return str(expanded.resolve())
    return None


def find_rhwp(
    repo: Path = REPO,
    env: Mapping[str, str] = os.environ,
    os_name: str = os.name,
) -> str:
    override = env.get("RHWP_BIN")
    if override:
        resolved = _resolve_override(override)
        if resolved:
            return resolved
        raise FileNotFoundError(f"RHWP_BIN 실행 파일을 찾을 수 없습니다: {override}")

    binary = "rhwp.exe" if os_name == "nt" else "rhwp"
    for candidate in (
        repo / "target" / "release-test" / binary,
        repo / "target" / "release" / binary,
    ):
        if candidate.is_file():
            return str(candidate)

    discovered = shutil.which("rhwp")
    if discovered:
        return discovered
    raise FileNotFoundError(
        "rhwp 실행 파일을 찾을 수 없습니다. release-test/release 빌드 또는 RHWP_BIN을 지정하세요."
    )


def find_chrome(
    env: Mapping[str, str] = os.environ,
    os_name: str = os.name,
    platform: str = sys.platform,
) -> str:
    override = env.get("CHROME_BIN")
    if override:
        resolved = _resolve_override(override)
        if resolved:
            return resolved
        raise FileNotFoundError(f"CHROME_BIN 실행 파일을 찾을 수 없습니다: {override}")

    if os_name == "nt":
        names = ("chrome.exe", "chrome")
        roots = [
            env.get("PROGRAMFILES"),
            env.get("PROGRAMFILES(X86)"),
            env.get("LOCALAPPDATA"),
        ]
        candidates = [
            Path(root) / "Google" / "Chrome" / "Application" / "chrome.exe"
            for root in roots
            if root
        ]
    elif platform == "darwin":
        names = ("google-chrome", "chrome", "chromium")
        candidates = [
            Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            Path("/Applications/Chromium.app/Contents/MacOS/Chromium"),
            Path.home() / "Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        ]
    else:
        names = (
            "google-chrome",
            "google-chrome-stable",
            "chromium",
            "chromium-browser",
        )
        candidates = []

    for name in names:
        discovered = shutil.which(name)
        if discovered:
            return discovered
    for candidate in candidates:
        if candidate.is_file():
            return str(candidate)
    raise FileNotFoundError(
        "Chrome/Chromium을 찾을 수 없습니다. CHROME_BIN을 지정하세요."
    )


def capture_with_chrome(
    chrome: str,
    source: Path,
    out_png: Path,
    width: int,
    height: int,
    *,
    attempts: int = 2,
    run: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> bool:
    if out_png.is_file() and out_png.stat().st_size > 0:
        return True

    command = [
        chrome,
        "--headless=new",
        "--disable-gpu",
        f"--screenshot={out_png}",
        f"--window-size={width},{height}",
        "--hide-scrollbars",
        source.resolve().as_uri(),
    ]
    for attempt in range(1, attempts + 1):
        result = run(
            command,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        if result.returncode == 0 and out_png.is_file() and out_png.stat().st_size > 0:
            return True

        detail = (result.stderr or result.stdout or "출력 없음").strip()
        print(
            f"Chrome 캡처 실패 {attempt}/{attempts}: {source} -> {out_png} "
            f"(exit {result.returncode})\n{detail}",
            file=sys.stderr,
        )
        if out_png.exists():
            out_png.unlink()
    return False


def svg_to_png(svg_path: Path, out_png: Path, chrome: str) -> bool:
    if out_png.is_file() and out_png.stat().st_size > 0:
        return True
    head = svg_path.read_text(encoding="utf-8", errors="ignore")[:600]
    width_match = re.search(r'width="([0-9.]+)"', head)
    height_match = re.search(r'height="([0-9.]+)"', head)
    width = math.ceil(float(width_match.group(1))) + 2 if width_match else 810
    height = math.ceil(float(height_match.group(1))) + 2 if height_match else 1140
    return capture_with_chrome(chrome, svg_path, out_png, width, height)


def pdf_to_png(page: object, out_png: Path) -> None:
    if out_png.is_file() and out_png.stat().st_size > 0:
        return
    scale = PNG_W / page.get_size()[0]  # type: ignore[attr-defined]
    bitmap = page.render(scale=scale)  # type: ignore[attr-defined]
    bitmap.to_pil().save(out_png)


def diff_score(a_png: Path, b_png: Path) -> float:
    from PIL import Image, ImageChops

    a = Image.open(a_png).convert("L")
    b = Image.open(b_png).convert("L")
    height = min(a.height, b.height)
    a = a.resize((PNG_W, height))
    b = b.resize((PNG_W, height))
    difference = ImageChops.difference(a, b)
    histogram = difference.histogram()
    total = sum(histogram)
    changed = sum(histogram[16:])
    return round(100.0 * changed / total, 2)


def sheet(
    title: str, left: Path, right: Path, out_png: Path, work_dir: Path, chrome: str
) -> bool:
    def image_data(path: Path) -> str:
        return base64.b64encode(path.read_bytes()).decode()

    markup = (
        '<!doctype html><meta charset="utf-8"><style>body{margin:0;background:#eee;'
        "font-family:Malgun Gothic}.t{text-align:center;font-weight:700;padding:6px;font-size:15px}"
        ".r{display:flex;gap:8px;padding:0 8px}.c{flex:1}"
        ".l{text-align:center;font-size:12px;font-weight:600;padding:2px}"
        "img{width:100%;border:1px solid #aaa;background:#fff}</style>"
        f'<div class="t">{html.escape(title)}</div><div class="r">'
        '<div class="c"><div class="l" style="color:#1a56db">한컴 기준 PDF</div>'
        f'<img src="data:image/png;base64,{image_data(left)}"></div>'
        '<div class="c"><div class="l" style="color:#0e9f6e">rhwp 렌더</div>'
        f'<img src="data:image/png;base64,{image_data(right)}"></div></div>'
    )
    html_path = work_dir / "_s.html"
    html_path.write_text(markup, encoding="utf-8")
    return capture_with_chrome(chrome, html_path, out_png, 1440, 1040)


def normalized_characters(text: str) -> Counter[str]:
    normalized = unicodedata.normalize("NFC", text)
    return Counter(character for character in normalized if not character.isspace())


def svg_text(svg_path: Path) -> str:
    root = ET.parse(svg_path).getroot()
    parts: list[str] = []
    for element in root.iter():
        if element.tag.rsplit("}", 1)[-1] == "text":
            parts.extend(element.itertext())
    return "".join(parts)


def compare_text_layers(
    reference_text: str, rendered_text: str
) -> tuple[Counter[str], Counter[str]]:
    reference = normalized_characters(reference_text)
    rendered = normalized_characters(rendered_text)
    return reference - rendered, rendered - reference


def counter_summary(counter: Counter[str]) -> str:
    def label(character: str, count: int) -> str:
        escaped = character.encode("unicode_escape").decode("ascii")
        return f"U+{ord(character):04X}:{escaped}×{count}"

    return ",".join(
        label(character, counter[character]) for character in sorted(counter)
    )


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="한컴 기준 PDF와 rhwp SVG를 페이지별로 비교합니다."
    )
    parser.add_argument("key", choices=sorted(REG), help="등록된 문서 키")
    parser.add_argument("start_page", type=int, help="시작 쪽(0 기준)")
    parser.add_argument("end_page", type=int, help="끝 쪽(0 기준, 포함)")
    parser.add_argument(
        "--out-dir",
        type=Path,
        help="산출 디렉터리. 생략하면 output/fidelity/<키> 사용",
    )
    args = parser.parse_args(argv)
    if args.start_page < 0 or args.end_page < args.start_page:
        parser.error("쪽 범위는 0 이상이며 끝 쪽이 시작 쪽보다 작을 수 없습니다.")
    return args


def render_svg(rhwp: str, source: Path, svg_dir: Path, page_index: int) -> bool:
    if list(svg_dir.glob(f"*_{page_index + 1:03}.svg")):
        return True
    command = [
        rhwp,
        "export-svg",
        str(source),
        "-p",
        str(page_index),
        "-o",
        str(svg_dir),
    ]
    font_path = os.environ.get("RHWP_FONT_PATH_DIR")
    if font_path:
        command.extend(["--font-path", font_path])
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "출력 없음").strip()
        print(
            f"rhwp SVG 렌더 실패 p{page_index + 1} (exit {result.returncode})\n{detail}",
            file=sys.stderr,
        )
        return False
    return bool(list(svg_dir.glob(f"*_{page_index + 1:03}.svg")))


def main(argv: Sequence[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    args = parse_args(argv)
    fixture = REG[args.key]
    try:
        source = resolve(REPO, fixture.source_pattern)
        reference_pdf = resolve(REPO, fixture.reference_pattern)
        rhwp = find_rhwp()
        chrome = find_chrome()
    except FileNotFoundError as error:
        print(error, file=sys.stderr)
        return 2

    work_dir = (args.out_dir or REPO / "output" / "fidelity" / fixture.name).resolve()
    svg_dir = work_dir / "svg"
    work_dir.mkdir(parents=True, exist_ok=True)
    svg_dir.mkdir(parents=True, exist_ok=True)
    (work_dir / "provenance.tsv").write_text(
        "role\tpath\tgrade\n"
        f"source\t{source}\t원본 입력\n"
        f"reference_pdf\t{reference_pdf}\t{fixture.reference_grade}\n",
        encoding="utf-8",
    )

    for page_index in range(args.start_page, args.end_page + 1):
        render_svg(rhwp, source, svg_dir, page_index)

    try:
        import pypdfium2 as pdfium
    except ImportError:
        print(
            "pypdfium2가 필요합니다: python -m pip install pypdfium2", file=sys.stderr
        )
        return 2

    pdf = pdfium.PdfDocument(reference_pdf)
    if args.end_page >= len(pdf):
        print(
            f"요청 끝 쪽 {args.end_page}가 기준 PDF 마지막 index {len(pdf) - 1}를 넘습니다.",
            file=sys.stderr,
        )
        return 2

    rows: list[tuple[int, float, str]] = []
    text_rows: list[tuple[int, int, int, str, str, str]] = []
    for page_index in range(args.start_page, args.end_page + 1):
        svg_files = list(svg_dir.glob(f"*_{page_index + 1:03}.svg"))
        rendered_png = work_dir / f"r{page_index:03}.png"
        reference_png = work_dir / f"g{page_index:03}.png"
        comparison_png = work_dir / f"cmp-p{page_index:03}.png"
        if not svg_files:
            rows.append((page_index, -1.0, "rhwp SVG 없음"))
            text_rows.append((page_index, 0, 0, "", "", "rhwp SVG 없음"))
            continue

        page = pdf[page_index]
        svg_path = svg_files[0]
        svg_ok = svg_to_png(svg_path, rendered_png, chrome)
        pdf_to_png(page, reference_png)

        try:
            text_page = page.get_textpage()
            try:
                reference_text = text_page.get_text_range()
            finally:
                text_page.close()
            missing, extra = compare_text_layers(reference_text, svg_text(svg_path))
            text_rows.append(
                (
                    page_index,
                    sum(missing.values()),
                    sum(extra.values()),
                    counter_summary(missing),
                    counter_summary(extra),
                    "",
                )
            )
        except Exception as error:  # noqa: BLE001 - 선택적 텍스트 추출은 픽셀 대조를 막지 않는다.
            text_rows.append((page_index, 0, 0, "", "", f"텍스트층 추출 실패: {error}"))

        if not svg_ok or not (rendered_png.is_file() and reference_png.is_file()):
            rows.append((page_index, -1.0, "PNG 실패"))
            continue
        score = diff_score(reference_png, rendered_png)
        note = ""
        if not comparison_png.exists() and not sheet(
            f"{fixture.name} — p{page_index + 1} (diff {score}%)",
            reference_png,
            rendered_png,
            comparison_png,
            work_dir,
            chrome,
        ):
            note = "비교 시트 PNG 실패"
        rows.append((page_index, score, note))

    rows.sort(key=lambda row: -row[1])
    report_path = work_dir / "report.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write("page\tdiff%\tnote\n")
        for page_index, score, note in rows:
            report.write(f"{page_index + 1}\t{score}\t{note}\n")

    text_report_path = work_dir / "text-report.tsv"
    with text_report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\treference_only\tsvg_only\treference_only_chars\tsvg_only_chars\tnote\n"
        )
        for page_index, missing_count, extra_count, missing, extra, note in text_rows:
            report.write(
                f"{page_index + 1}\t{missing_count}\t{extra_count}\t{missing}\t{extra}\t{note}\n"
            )

    print(f"기준 PDF: {reference_pdf}")
    print(f"등급: {fixture.reference_grade}")
    print(f"완료: {args.end_page - args.start_page + 1}쪽. diff 랭킹(top 8):")
    for page_index, score, note in rows[:8]:
        print(f"  p{page_index + 1}: {score}% {note}")
    print("pixel report:", report_path)
    print("text report:", text_report_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
