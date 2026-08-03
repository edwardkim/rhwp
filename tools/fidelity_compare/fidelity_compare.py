"""한컴 기준 PDF와 rhwp SVG를 페이지별로 비교하는 하네스."""

from __future__ import annotations

import argparse
import base64
import difflib
import glob
import html
import json
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


def normalized_text_sequence(text: str) -> str:
    """Keep normalized character order for page-owner movement candidates."""
    normalized = unicodedata.normalize("NFC", text)
    return "".join(character for character in normalized if not character.isspace())


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


def adjacent_text_owner_shift_candidates(
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
) -> list[dict[str, object]]:
    """Find large reciprocal text differences across adjacent physical pages.

    A text multiset cannot prove visual layout, but an SVG-only block on pN that
    is the same as a PDF-only block on pN+1 is a strong page-owner candidate.
    Keep this separate from ordinary per-page text loss: the directional pairing
    is useful for footnotes/captions that were placed one page too early or late.
    """

    candidates: list[dict[str, object]] = []

    def append_candidate(
        page_index: int,
        direction: str,
        source: Counter[str],
        target: Counter[str],
    ) -> None:
        shared = source & target
        shared_count = sum(shared.values())
        source_count = sum(source.values())
        target_count = sum(target.values())
        if (
            shared_count < 8
            or source_count == 0
            or target_count == 0
            or shared_count / source_count < 0.75
            or shared_count / target_count < 0.75
        ):
            return
        candidates.append(
            {
                "page": page_index,
                "next_page": page_index + 1,
                "direction": direction,
                "shared_count": shared_count,
                "source_coverage": shared_count / source_count,
                "target_coverage": shared_count / target_count,
                "shared": shared,
            }
        )

    for page_index in sorted(page_differences):
        next_differences = page_differences.get(page_index + 1)
        if next_differences is None:
            continue
        missing, extra = page_differences[page_index]
        next_missing, next_extra = next_differences
        append_candidate(
            page_index,
            "rhwp_earlier_than_reference",
            extra,
            next_missing,
        )
        append_candidate(
            page_index,
            "rhwp_later_than_reference",
            missing,
            next_extra,
        )
    return candidates


def unmatched_sequence_fragments(
    reference_text: str,
    rendered_text: str,
    *,
    reference_only: bool,
    min_chars: int = 16,
) -> list[str]:
    """Return substantial ordered text unique to one side of a page comparison.

    Counter deltas are intentionally order-independent, but repeated ordinary
    characters can hide a whole URL or citation that moved by one page.  This
    helper preserves order only for candidate extraction; it does not assert
    that a page is visually equivalent.
    """
    reference = normalized_text_sequence(reference_text)
    rendered = normalized_text_sequence(rendered_text)
    fragments: list[str] = []
    matcher = difflib.SequenceMatcher(None, reference, rendered, autojunk=False)
    for tag, reference_start, reference_end, rendered_start, rendered_end in matcher.get_opcodes():
        if tag == "equal":
            continue
        if reference_only:
            if tag not in {"delete", "replace"}:
                continue
            fragment = reference[reference_start:reference_end]
        else:
            if tag not in {"insert", "replace"}:
                continue
            fragment = rendered[rendered_start:rendered_end]
        if len(fragment) >= min_chars:
            fragments.append(fragment)

    # A large replace can contain a shorter unmatched run.  Keep only the
    # largest representative so the review queue remains concise.
    selected: list[str] = []
    for fragment in sorted(set(fragments), key=len, reverse=True):
        if not any(fragment in kept for kept in selected):
            selected.append(fragment)
    return selected


def adjacent_text_owner_sequence_candidates(
    page_text_layers: Mapping[int, tuple[str, str]],
) -> list[dict[str, object]]:
    """Find ordered text that has moved exactly one physical page.

    This complements the Counter-based reciprocal ledger.  A substantial
    sequence missing from PDF pN's SVG but present only in SVG pN+1 is an
    `rhwp_later_than_reference` candidate; the inverse is
    `rhwp_earlier_than_reference`.  It is deliberately candidate-only because
    PDF text extraction and identical repeated prose can still be ambiguous.
    """
    candidates: list[dict[str, object]] = []
    for page_index in sorted(page_text_layers):
        next_layers = page_text_layers.get(page_index + 1)
        if next_layers is None:
            continue
        reference, rendered = page_text_layers[page_index]
        next_reference, next_rendered = next_layers
        # The fragments below are whitespace-normalized because SVG/PDF text
        # layers often disagree only in a line break.  Normalize every side
        # before both target membership checks as well; comparing a normalized
        # URL/citation to raw next-page text silently misses that common case.
        reference_sequence = normalized_text_sequence(reference)
        rendered_sequence = normalized_text_sequence(rendered)
        next_reference_sequence = normalized_text_sequence(next_reference)
        next_rendered_sequence = normalized_text_sequence(next_rendered)
        for fragment in unmatched_sequence_fragments(
            reference_sequence, rendered_sequence, reference_only=True
        ):
            if (
                fragment in next_rendered_sequence
                and fragment not in next_reference_sequence
                # SequenceMatcher can choose a non-local alignment after an
                # intra-page reorder.  Do not label text that is still on the
                # current rhwp page as a physical-page owner move.
                and fragment not in rendered_sequence
            ):
                candidates.append(
                    {
                        "page": page_index,
                        "next_page": page_index + 1,
                        "direction": "rhwp_later_than_reference",
                        "chars": len(fragment),
                        "sequence": fragment,
                    }
                )
        for fragment in unmatched_sequence_fragments(
            reference_sequence, rendered_sequence, reference_only=False
        ):
            if (
                fragment in next_reference_sequence
                and fragment not in next_rendered_sequence
                and fragment not in reference_sequence
            ):
                candidates.append(
                    {
                        "page": page_index,
                        "next_page": page_index + 1,
                        "direction": "rhwp_earlier_than_reference",
                        "chars": len(fragment),
                        "sequence": fragment,
                    }
                )
    return candidates


def counter_summary(counter: Counter[str]) -> str:
    def label(character: str, count: int) -> str:
        escaped = character.encode("unicode_escape").decode("ascii")
        return f"U+{ord(character):04X}:{escaped}×{count}"

    return ",".join(
        label(character, counter[character]) for character in sorted(counter)
    )


def parse_page_index(value: str, option: str, parser: argparse.ArgumentParser) -> int:
    try:
        page_index = int(value)
    except ValueError:
        parser.error(f"{option}은 정수여야 합니다: {value}")
    if page_index < 0:
        parser.error(f"{option}은 0 이상이어야 합니다: {value}")
    return page_index


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="한컴 기준 PDF와 rhwp SVG를 페이지별로 비교합니다."
    )
    parser.add_argument(
        "positionals",
        nargs="+",
        metavar="ARG",
        help="등록 fixture는 <키> <시작쪽> <끝쪽>, direct pair는 <시작쪽> <끝쪽>",
    )
    parser.add_argument(
        "--source",
        type=Path,
        help="등록 fixture 대신 직접 비교할 HWP/HWPX 입력",
    )
    parser.add_argument(
        "--reference-pdf",
        type=Path,
        help="직접 비교할 기준 PDF (--source, --label과 함께 사용)",
    )
    parser.add_argument(
        "--label",
        help="direct pair 산출물/provenance 식별 ASCII label",
    )
    parser.add_argument(
        "--reference-grade",
        default="사용자 지정 기준 PDF (provenance는 출력 파일 참조)",
        help="direct pair 기준 PDF의 등급/출처 설명",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        help="산출 디렉터리. 생략하면 output/fidelity/<키> 사용",
    )
    parser.add_argument(
        "--text-only",
        action="store_true",
        help="Chrome/PNG/sheet 없이 PDF text와 SVG text 후보 원장만 생성",
    )
    parser.add_argument(
        "--export-all-svg",
        action="store_true",
        help="선택 범위와 관계없이 export-svg를 한 번 실행해 SVG cache 전체를 생성",
    )
    parser.add_argument(
        "--layout-ledger",
        action="store_true",
        help="render tree를 한 번 export해 body/각주·표/footer 충돌 후보 원장을 생성",
    )
    args = parser.parse_args(argv)

    direct_values = (args.source, args.reference_pdf, args.label)
    is_direct = any(value is not None for value in direct_values)
    if is_direct:
        if not all(value is not None for value in direct_values):
            parser.error("direct pair에는 --source, --reference-pdf, --label을 모두 지정해야 합니다.")
        if len(args.positionals) != 2:
            parser.error("direct pair positional은 <시작쪽> <끝쪽> 두 개여야 합니다.")
        args.key = None
        args.start_page = parse_page_index(args.positionals[0], "시작쪽", parser)
        args.end_page = parse_page_index(args.positionals[1], "끝쪽", parser)
    else:
        if args.reference_grade != "사용자 지정 기준 PDF (provenance는 출력 파일 참조)":
            parser.error("--reference-grade는 direct pair에서만 사용할 수 있습니다.")
        if len(args.positionals) != 3:
            parser.error("등록 fixture positional은 <키> <시작쪽> <끝쪽> 세 개여야 합니다.")
        key, start_page, end_page = args.positionals
        if key not in REG:
            parser.error(f"등록되지 않은 문서 키: {key} (선택: {', '.join(sorted(REG))})")
        args.key = key
        args.start_page = parse_page_index(start_page, "시작쪽", parser)
        args.end_page = parse_page_index(end_page, "끝쪽", parser)

    if args.end_page < args.start_page:
        parser.error("끝 쪽이 시작 쪽보다 작을 수 없습니다.")
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


def render_all_svg(rhwp: str, source: Path, svg_dir: Path) -> bool:
    """한 번의 export로 전체 SVG cache를 만들고 raw manifest도 보관한다."""
    command = [rhwp, "export-svg", str(source), "--json", "-o", str(svg_dir)]
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
            f"rhwp 전체 SVG 렌더 실패 (exit {result.returncode})\n{detail}",
            file=sys.stderr,
        )
        return False
    (svg_dir / "export-svg-manifest.json").write_text(result.stdout, encoding="utf-8")
    return True


def render_all_render_tree(rhwp: str, source: Path, tree_dir: Path) -> bool:
    command = [rhwp, "export-render-tree", str(source), "-o", str(tree_dir)]
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
            f"rhwp 전체 render tree export 실패 (exit {result.returncode})\n{detail}",
            file=sys.stderr,
        )
        return False
    (tree_dir / "export-render-tree.log").write_text(result.stderr, encoding="utf-8")
    return True


def bbox_from_node(node: Mapping[str, object]) -> tuple[float, float, float, float] | None:
    bbox = node.get("bbox")
    if not isinstance(bbox, Mapping):
        return None
    try:
        return (
            float(bbox["x"]),
            float(bbox["y"]),
            float(bbox["w"]),
            float(bbox["h"]),
        )
    except (KeyError, TypeError, ValueError):
        return None


def text_line_has_visible_paint(node: Mapping[str, object]) -> bool:
    """Keep blank render-tree guide lines out of wrap-overlap candidates.

    Some HWP5 Square-wrap paragraphs retain zero-height/empty guide lines with
    a full-column `TextLine` bbox. They are layout aids, not painted Body text,
    so counting them would inflate a candidate. Older/minimal trees without
    children retain the conservative legacy behavior because their paint detail
    is unavailable.
    """
    children = node.get("children")
    if not isinstance(children, list) or not children:
        return True

    found_text_run = False
    visible = False

    def walk(child: Mapping[str, object]) -> None:
        nonlocal found_text_run, visible
        node_type = child.get("type")
        if node_type in {"Equation", "FnMarker"}:
            visible = True
        elif node_type == "TextRun":
            found_text_run = True
            display = child.get("displayText")
            source = child.get("text")
            text = display if isinstance(display, str) else source
            if isinstance(text, str) and text.replace("\U000f081c", "").strip():
                visible = True
        nested = child.get("children")
        if isinstance(nested, list):
            for grandchild in nested:
                if isinstance(grandchild, Mapping):
                    walk(grandchild)

    for child in children:
        if isinstance(child, Mapping):
            walk(child)
    return visible or not found_text_run


def square_wrap_text_overlap_candidates(
    tree: Mapping[str, object],
) -> list[dict[str, object]]:
    """Find Body text crossing a flow-wrapped image's physical box.

    Square/Tight/Through images reserve a text-flow band.  Three or more Body
    lines crossing at least half of a substantial image width is therefore a
    strong layout candidate, unlike BehindText/InFrontOfText which may overlap
    intentionally.  The render-tree JSON exposes the source wrap mode so this
    low-cost ledger works without PDF raster dependencies.
    """
    body_lines: list[tuple[float, float, float, float]] = []
    images: list[tuple[tuple[float, float, float, float], object, object, str]] = []

    def walk(node: Mapping[str, object], region: str = "outside") -> None:
        node_type = node.get("type")
        if node_type in {"Body", "FootnoteArea", "Footer", "Header"}:
            region = str(node_type)
        box = bbox_from_node(node)
        if region == "Body" and box is not None:
            if node_type == "TextLine":
                if text_line_has_visible_paint(node):
                    body_lines.append(box)
            elif node_type == "Image":
                text_wrap = node.get("textWrap")
                if text_wrap in {"Square", "Tight", "Through"}:
                    images.append((box, node.get("pi"), node.get("ci"), str(text_wrap)))
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, Mapping):
                    walk(child, region)

    walk(tree)
    candidates: list[dict[str, object]] = []
    for image, para_index, control_index, text_wrap in images:
        ix, iy, iw, ih = image
        if iw < 80.0 or ih < 80.0:
            continue
        overlap_lines: list[tuple[float, float, float, float]] = []
        for line in body_lines:
            lx, ly, lw, lh = line
            overlap_x = min(ix + iw, lx + lw) - max(ix, lx)
            overlap_y = min(iy + ih, ly + lh) - max(iy, ly)
            if overlap_x >= iw * 0.5 and overlap_y >= min(5.0, lh * 0.5):
                overlap_lines.append(line)
        if len(overlap_lines) >= 3:
            candidates.append(
                {
                    "pi": para_index,
                    "ci": control_index,
                    "text_wrap": text_wrap,
                    "overlap_line_count": len(overlap_lines),
                    "image_bbox": [round(value, 1) for value in image],
                    "first_line_bbox": [round(value, 1) for value in overlap_lines[0]],
                    "last_line_bbox": [round(value, 1) for value in overlap_lines[-1]],
                }
            )
    return candidates


def layout_candidates(tree: Mapping[str, object]) -> tuple[int, int, int, int, int]:
    """(body↔각주, table↔footer, table/frame, image/frame, square-wrap/text) 후보 수."""
    page_bbox = bbox_from_node(tree)
    if page_bbox is None:
        return (0, 0, 0, 0, 0)
    _, _, page_width, page_height = page_bbox
    footnote_tops: list[float] = []
    footer_tops: list[float] = []
    body_lines: list[tuple[float, float, float, float]] = []
    body_tables: list[tuple[float, float, float, float]] = []
    body_images: list[tuple[float, float, float, float]] = []

    def walk(node: Mapping[str, object], region: str = "outside") -> None:
        node_type = node.get("type")
        if node_type in {"Body", "FootnoteArea", "Footer", "Header"}:
            region = str(node_type)
        box = bbox_from_node(node)
        if node_type == "FootnoteArea" and box is not None:
            footnote_tops.append(box[1])
        elif node_type == "Footer" and box is not None:
            footer_tops.append(box[1])
        elif region == "Body" and box is not None:
            if node_type == "TextLine":
                body_lines.append(box)
            elif node_type == "Table":
                body_tables.append(box)
            elif node_type == "Image":
                body_images.append(box)
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, Mapping):
                    walk(child, region)

    walk(tree)
    footnote_top = min(footnote_tops, default=None)
    footer_top = min(footer_tops, default=None)
    # 1px는 stroke/float 반올림 noise로 보고 무시한다. 이 신호는 candidate-only다.
    body_footnote_lines = sum(
        line[1] + line[3] > footnote_top + 1.0
        for line in body_lines
    ) if footnote_top is not None else 0
    table_footer = sum(
        table[1] + table[3] > footer_top + 1.0
        for table in body_tables
    ) if footer_top is not None else 0

    def outside_page(box: tuple[float, float, float, float]) -> bool:
        x, y, width, height = box
        return x < -1.0 or y < -1.0 or x + width > page_width + 1.0 or y + height > page_height + 1.0

    table_outside_frame = sum(outside_page(table) for table in body_tables)
    image_outside_frame = sum(outside_page(image) for image in body_images)
    square_wrap_text_overlap = len(square_wrap_text_overlap_candidates(tree))
    return (
        body_footnote_lines,
        table_footer,
        table_outside_frame,
        image_outside_frame,
        square_wrap_text_overlap,
    )


TABLE_MATERIAL_TEXT_DELTA_CHARS = 24
TABLE_BOTTOM_NEAR_PAGE_RATIO = 0.85


def page_text_delta_chars(
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
    page_index: int,
) -> int | None:
    """Return a page's PDF/SVG character delta when that comparison exists."""
    differences = page_differences.get(page_index)
    if differences is None:
        return None
    missing, extra = differences
    return sum(missing.values()) + sum(extra.values())


def body_table_records(
    tree: Mapping[str, object],
    page_index: int,
    text_delta_chars: int | None,
) -> list[dict[str, object]]:
    """Collect Body-table geometry signals from one render-tree page.

    The render tree has no Hancom PDF table-row ownership information.  These
    records therefore preserve only rhwp's table identity/geometry and the
    local PDF/SVG text-delta *signal* for a later visual comparison.
    """
    page_box = bbox_from_node(tree)
    footer_tops: list[float] = []
    tables: list[tuple[Mapping[str, object], tuple[float, float, float, float]]] = []

    def walk(node: Mapping[str, object], region: str = "outside") -> None:
        node_type = node.get("type")
        if node_type in {"Body", "FootnoteArea", "Footer", "Header"}:
            region = str(node_type)
        box = bbox_from_node(node)
        if node_type == "Footer" and box is not None:
            footer_tops.append(box[1])
        elif region == "Body" and node_type == "Table" and box is not None:
            tables.append((node, box))
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, Mapping):
                    walk(child, region)

    walk(tree)
    footer_top = min(footer_tops, default=None)
    records: list[dict[str, object]] = []
    for node, box in tables:
        x, y, width, height = box
        table_bottom = y + height
        footer_overlap = footer_top is not None and table_bottom > footer_top + 1.0
        outside_frame = False
        bottom_gap: float | None = None
        bottom_near = False
        if page_box is not None:
            page_x, page_y, page_width, page_height = page_box
            page_right = page_x + page_width
            page_bottom = page_y + page_height
            outside_frame = (
                x < page_x - 1.0
                or y < page_y - 1.0
                or x + width > page_right + 1.0
                or table_bottom > page_bottom + 1.0
            )
            bottom_gap = page_bottom - table_bottom
            bottom_near = table_bottom >= page_y + page_height * TABLE_BOTTOM_NEAR_PAGE_RATIO
        records.append(
            {
                "page": page_index,
                "pi": node.get("pi"),
                "ci": node.get("ci"),
                "rows": node.get("rows"),
                "cols": node.get("cols"),
                "bbox": box,
                "footer_overlap": footer_overlap,
                "outside_frame": outside_frame,
                "bottom_gap": bottom_gap,
                "bottom_near": bottom_near,
                "text_delta_chars": text_delta_chars,
            }
        )
    return records


def table_identity(record: Mapping[str, object]) -> tuple[str, str] | None:
    """Return stable source identity only when both Table coordinates exist."""
    para_index = record.get("pi")
    control_index = record.get("ci")
    if para_index is None or control_index is None:
        return None
    return (str(para_index), str(control_index))


def table_record_signals(record: Mapping[str, object], prefix: str) -> list[str]:
    """Return candidate signals; none of them is a PDF row-owner verdict."""
    signals: list[str] = []
    if record.get("footer_overlap") is True:
        signals.append(f"{prefix}_table_footer")
    if record.get("outside_frame") is True:
        signals.append(f"{prefix}_table_outside_frame")
    delta = record.get("text_delta_chars")
    if (
        record.get("bottom_near") is True
        and isinstance(delta, int)
        and delta >= TABLE_MATERIAL_TEXT_DELTA_CHARS
    ):
        signals.append(f"{prefix}_bottom_near_material_text_delta")
    return signals


def table_fragment_candidates(
    tree_dir: Path,
    requested_pages: Sequence[int],
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
) -> list[dict[str, object]]:
    """Triages adjacent Body-table fragments and local table-risk signals.

    A same `(pi, ci)` table on physical pN/pN+1 establishes only that rhwp
    fragmented one source table.  The companion PDF/SVG text delta and footer/
    frame geometry narrow review priority; neither can assert which PDF row
    belongs to which physical page.
    """
    requested = set(requested_pages)
    page_paths: dict[int, Path] = {}
    pattern = re.compile(r"_([0-9]+)\.json$")
    for path in tree_dir.glob("*.json"):
        match = pattern.search(path.name)
        if match is not None:
            page_paths[int(match.group(1)) - 1] = path

    # The render tree is global, but a narrow compare should not deserialize a
    # 200-page document merely to decide whether pN's table continues on pN+1.
    # Include both neighbors so a requested pN also reports a pN-1→pN join.
    inspected_pages = {
        neighbor
        for page_index in requested
        for neighbor in (page_index - 1, page_index, page_index + 1)
        if neighbor >= 0
    }
    records_by_page: dict[int, list[dict[str, object]]] = {}
    for page_index in sorted(inspected_pages):
        path = page_paths.get(page_index)
        if path is None:
            continue
        try:
            tree = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError, json.JSONDecodeError):
            continue
        if not isinstance(tree, Mapping):
            continue
        records_by_page[page_index] = body_table_records(
            tree,
            page_index,
            page_text_delta_chars(page_differences, page_index),
        )

    candidates: list[dict[str, object]] = []
    paired_records: set[int] = set()
    for page_index, records in sorted(records_by_page.items()):
        next_records = records_by_page.get(page_index + 1, [])
        next_by_identity: dict[tuple[str, str], list[dict[str, object]]] = {}
        for record in next_records:
            identity = table_identity(record)
            if identity is not None:
                next_by_identity.setdefault(identity, []).append(record)
        for record in records:
            identity = table_identity(record)
            if identity is None:
                continue
            for next_record in next_by_identity.get(identity, []):
                if page_index not in requested and page_index + 1 not in requested:
                    continue
                paired_records.add(id(record))
                paired_records.add(id(next_record))
                signals = ["same_pi_ci_adjacent_fragment"]
                signals.extend(table_record_signals(record, "page"))
                signals.extend(table_record_signals(next_record, "next_page"))
                candidates.append(
                    {
                        "page_table": record,
                        "next_page_table": next_record,
                        "signals": signals,
                    }
                )

    for page_index in sorted(requested):
        for record in records_by_page.get(page_index, []):
            if id(record) in paired_records:
                continue
            signals = table_record_signals(record, "page")
            if signals:
                candidates.append(
                    {
                        "page_table": record,
                        "next_page_table": None,
                        "signals": signals,
                    }
                )

    def candidate_sort_key(candidate: Mapping[str, object]) -> tuple[int, int, str, str]:
        page_table = candidate["page_table"]
        assert isinstance(page_table, Mapping)
        next_page_table = candidate.get("next_page_table")
        next_page = (
            int(next_page_table["page"])
            if isinstance(next_page_table, Mapping)
            else -1
        )
        return (
            int(page_table["page"]),
            next_page,
            str(page_table.get("pi", "")),
            str(page_table.get("ci", "")),
        )

    return sorted(candidates, key=candidate_sort_key)


def format_bbox(box: object) -> str:
    if not isinstance(box, tuple) or len(box) != 4:
        return "-"
    return ",".join(f"{float(value):.1f}" for value in box)


def format_number(value: object) -> str:
    if value is None:
        return "-"
    if isinstance(value, float):
        return f"{value:.1f}"
    return str(value)


def write_table_fragment_ledger(
    work_dir: Path,
    tree_dir: Path,
    requested_pages: Sequence[int],
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
) -> None:
    """Write bounded table-fragment review candidates for `--layout-ledger`."""
    report_path = work_dir / "table-fragment-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tnext_page\tpi\tci\trows\tcols\tnext_rows\tnext_cols\t"
            "bbox\tnext_bbox\tsignals\tpage_text_delta_chars\t"
            "next_page_text_delta_chars\tpage_bottom_gap_px\t"
            "next_page_bottom_gap_px\tnote\n"
        )
        for candidate in table_fragment_candidates(
            tree_dir, requested_pages, page_differences
        ):
            page_table = candidate["page_table"]
            assert isinstance(page_table, Mapping)
            next_page_table = candidate.get("next_page_table")
            assert next_page_table is None or isinstance(next_page_table, Mapping)
            report.write(
                f"{int(page_table['page']) + 1}\t"
                f"{int(next_page_table['page']) + 1 if next_page_table is not None else '-'}\t"
                f"{format_number(page_table.get('pi'))}\t"
                f"{format_number(page_table.get('ci'))}\t"
                f"{format_number(page_table.get('rows'))}\t"
                f"{format_number(page_table.get('cols'))}\t"
                f"{format_number(next_page_table.get('rows')) if next_page_table is not None else '-'}\t"
                f"{format_number(next_page_table.get('cols')) if next_page_table is not None else '-'}\t"
                f"{format_bbox(page_table.get('bbox'))}\t"
                f"{format_bbox(next_page_table.get('bbox')) if next_page_table is not None else '-'}\t"
                f"{'|'.join(str(signal) for signal in candidate['signals'])}\t"
                f"{format_number(page_table.get('text_delta_chars'))}\t"
                f"{format_number(next_page_table.get('text_delta_chars')) if next_page_table is not None else '-'}\t"
                f"{format_number(page_table.get('bottom_gap'))}\t"
                f"{format_number(next_page_table.get('bottom_gap')) if next_page_table is not None else '-'}\t"
                "candidate only; does not assert PDF table row owner or fragment correctness\n"
            )


def tree_path_for_page(tree_dir: Path, page_index: int) -> Path | None:
    matches = sorted(tree_dir.glob(f"*_{page_index + 1:03}.json"))
    return matches[0] if matches else None


def numbered_page_count(directory: Path, suffix: str) -> int:
    """Count renderer page files while ignoring manifests and auxiliary files."""
    pattern = re.compile(rf"_([0-9]+){re.escape(suffix)}$")
    return len(
        {
            int(match.group(1))
            for path in directory.glob(f"*{suffix}")
            if (match := pattern.search(path.name)) is not None
        }
    )


def write_layout_ledger(
    work_dir: Path,
    tree_dir: Path,
    requested_pages: Sequence[int],
) -> list[int]:
    """requested page마다 geometry 후보를 기록하고 tree 누락 index를 반환한다."""
    report_path = work_dir / "layout-candidates.tsv"
    missing_pages: list[int] = []
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tbody_footnote_lines\ttable_footer\ttable_outside_frame\t"
            "image_outside_frame\tsquare_wrap_text_overlap\tnote\n"
        )
        for page_index in requested_pages:
            tree_path = tree_path_for_page(tree_dir, page_index)
            if tree_path is None:
                missing_pages.append(page_index)
                report.write(f"{page_index + 1}\t0\t0\t0\t0\t0\trender tree 없음\n")
                continue
            try:
                tree = json.loads(tree_path.read_text(encoding="utf-8"))
                if not isinstance(tree, Mapping):
                    raise ValueError("root가 object가 아님")
                candidates = layout_candidates(tree)
            except (OSError, ValueError, json.JSONDecodeError) as error:
                missing_pages.append(page_index)
                report.write(f"{page_index + 1}\t0\t0\t0\t0\t0\trender tree 읽기 실패: {error}\n")
                continue
            report.write(
                f"{page_index + 1}\t{candidates[0]}\t{candidates[1]}\t"
                f"{candidates[2]}\t{candidates[3]}\t{candidates[4]}\t-\n"
            )
    return missing_pages


def write_text_owner_shift_ledger(
    work_dir: Path,
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
) -> None:
    """Write adjacent-page text owner candidates without treating them as verdicts."""
    report_path = work_dir / "text-owner-shift-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tnext_page\tdirection\tshared_chars\tsource_coverage\t"
            "target_coverage\tshared_chars_detail\tnote\n"
        )
        for candidate in adjacent_text_owner_shift_candidates(page_differences):
            shared = candidate["shared"]
            assert isinstance(shared, Counter)
            report.write(
                f"{int(candidate['page']) + 1}\t{int(candidate['next_page']) + 1}\t"
                f"{candidate['direction']}\t{candidate['shared_count']}\t"
                f"{float(candidate['source_coverage']):.3f}\t"
                f"{float(candidate['target_coverage']):.3f}\t"
                f"{counter_summary(shared)}\tcandidate only; PDF visual owner review required\n"
            )


def write_text_owner_sequence_ledger(
    work_dir: Path,
    page_text_layers: Mapping[int, tuple[str, str]],
) -> None:
    """Write ordered adjacent-page owner candidates not visible to Counters."""
    report_path = work_dir / "text-owner-sequence-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write("page\tnext_page\tdirection\tsequence_chars\tsequence\tnote\n")
        for candidate in adjacent_text_owner_sequence_candidates(page_text_layers):
            sequence = str(candidate["sequence"]).replace("\t", " ").replace("\n", " ")
            report.write(
                f"{int(candidate['page']) + 1}\t{int(candidate['next_page']) + 1}\t"
                f"{candidate['direction']}\t{candidate['chars']}\t{sequence}\t"
                "candidate only; PDF visual owner review required\n"
            )


def write_page_count_ledger(
    work_dir: Path,
    *,
    reference_page_count: int,
    full_svg_page_count: int | None,
    full_render_tree_page_count: int | None,
) -> None:
    """Expose global page-count drift without conflating it with run completion."""
    report_path = work_dir / "page-count-ledger.tsv"

    def cell(value: int | None) -> str:
        return str(value) if value is not None else "-"

    def delta(value: int | None) -> str:
        return str(value - reference_page_count) if value is not None else "-"

    with report_path.open("w", encoding="utf-8") as report:
        report.write("measure\tpages\tdelta_from_reference\tscope\tnote\n")
        report.write(
            f"reference_pdf\t{reference_page_count}\t0\tfull PDF\tcomparison baseline\n"
        )
        report.write(
            f"rhwp_svg\t{cell(full_svg_page_count)}\t{delta(full_svg_page_count)}\t"
            f"{'full export' if full_svg_page_count is not None else 'not counted'}\t"
            "page-count difference is a candidate, not a global-break fix\n"
        )
        report.write(
            f"rhwp_render_tree\t{cell(full_render_tree_page_count)}\t"
            f"{delta(full_render_tree_page_count)}\t"
            f"{'full render tree' if full_render_tree_page_count is not None else 'not run'}\t"
            "page-count difference is a candidate, not a global-break fix\n"
        )


def write_run_state(
    work_dir: Path,
    *,
    requested_pages: Sequence[int],
    completed_pages: Sequence[int],
    missing_pages: Sequence[int],
    text_only: bool,
) -> None:
    def page_list(pages: Sequence[int]) -> str:
        return ",".join(str(page + 1) for page in pages) or "-"

    (work_dir / "run-state.tsv").write_text(
        "field\tvalue\n"
        f"mode\t{'text-only' if text_only else 'pixel-and-text'}\n"
        f"requested_pages_1based\t{page_list(requested_pages)}\n"
        f"completed_pages_1based\t{page_list(completed_pages)}\n"
        f"missing_pages_1based\t{page_list(missing_pages)}\n"
        f"run_state\t{'complete' if not missing_pages else 'incomplete'}\n",
        encoding="utf-8",
    )


def main(argv: Sequence[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    args = parse_args(argv)
    try:
        if args.key is None:
            source = args.source.expanduser().resolve()
            reference_pdf = args.reference_pdf.expanduser().resolve()
            if not source.is_file():
                raise FileNotFoundError(f"source 파일을 찾을 수 없습니다: {source}")
            if not reference_pdf.is_file():
                raise FileNotFoundError(f"reference PDF를 찾을 수 없습니다: {reference_pdf}")
            fixture = Fixture(
                args.label,
                str(source),
                str(reference_pdf),
                args.reference_grade,
            )
        else:
            fixture = REG[args.key]
            source = resolve(REPO, fixture.source_pattern)
            reference_pdf = resolve(REPO, fixture.reference_pattern)
        rhwp = find_rhwp()
        chrome = None if args.text_only else find_chrome()
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

    requested_pages = list(range(args.start_page, args.end_page + 1))
    if args.export_all_svg:
        if not render_all_svg(rhwp, source, svg_dir):
            write_run_state(
                work_dir,
                requested_pages=requested_pages,
                completed_pages=[],
                missing_pages=requested_pages,
                text_only=args.text_only,
            )
            return 1
    else:
        for page_index in requested_pages:
            render_svg(rhwp, source, svg_dir, page_index)

    layout_missing_pages: list[int] = []
    if args.layout_ledger:
        tree_dir = work_dir / "render_tree"
        tree_dir.mkdir(parents=True, exist_ok=True)
        if not render_all_render_tree(rhwp, source, tree_dir):
            write_run_state(
                work_dir,
                requested_pages=requested_pages,
                completed_pages=[],
                missing_pages=requested_pages,
                text_only=args.text_only,
            )
            return 1
        layout_missing_pages = write_layout_ledger(work_dir, tree_dir, requested_pages)

    pdf = None
    try:
        if args.text_only:
            from pypdf import PdfReader

            text_pdf = PdfReader(reference_pdf)
            reference_page_count = len(text_pdf.pages)

            def reference_text_for_page(page_index: int) -> str:
                return text_pdf.pages[page_index].extract_text() or ""

        else:
            import pypdfium2 as pdfium

            pdf = pdfium.PdfDocument(reference_pdf)
            reference_page_count = len(pdf)

            def reference_text_for_page(page_index: int) -> str:
                text_page = pdf[page_index].get_textpage()
                try:
                    return text_page.get_text_range()
                finally:
                    text_page.close()

    except ImportError:
        dependency = "pypdf" if args.text_only else "pypdfium2"
        print(
            f"{dependency}가 필요합니다: python -m pip install {dependency}", file=sys.stderr
        )
        return 2

    if args.end_page >= reference_page_count:
        print(
            f"요청 끝 쪽 {args.end_page}가 기준 PDF 마지막 index {reference_page_count - 1}를 넘습니다.",
            file=sys.stderr,
        )
        return 2

    full_svg_page_count = (
        numbered_page_count(svg_dir, ".svg") if args.export_all_svg else None
    )
    full_render_tree_page_count = (
        numbered_page_count(tree_dir, ".json") if args.layout_ledger else None
    )

    rows: list[tuple[int, float, str]] = []
    text_rows: list[tuple[int, int, int, str, str, str]] = []
    text_differences: dict[int, tuple[Counter[str], Counter[str]]] = {}
    text_layers: dict[int, tuple[str, str]] = {}
    completed_pages: list[int] = []
    missing_pages: list[int] = []
    for page_index in requested_pages:
        svg_files = list(svg_dir.glob(f"*_{page_index + 1:03}.svg"))
        if not svg_files:
            if not args.text_only:
                rows.append((page_index, -1.0, "rhwp SVG 없음"))
            text_rows.append((page_index, 0, 0, "", "", "rhwp SVG 없음"))
            missing_pages.append(page_index)
            continue

        svg_path = svg_files[0]

        try:
            reference_text = reference_text_for_page(page_index)
            rendered_text = svg_text(svg_path)
            missing, extra = compare_text_layers(reference_text, rendered_text)
            text_differences[page_index] = (missing, extra)
            text_layers[page_index] = (reference_text, rendered_text)
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

        if args.text_only:
            completed_pages.append(page_index)
            continue

        rendered_png = work_dir / f"r{page_index:03}.png"
        reference_png = work_dir / f"g{page_index:03}.png"
        comparison_png = work_dir / f"cmp-p{page_index:03}.png"
        assert chrome is not None
        assert pdf is not None
        page = pdf[page_index]
        svg_ok = svg_to_png(svg_path, rendered_png, chrome)
        pdf_to_png(page, reference_png)
        if not svg_ok or not (rendered_png.is_file() and reference_png.is_file()):
            rows.append((page_index, -1.0, "PNG 실패"))
            missing_pages.append(page_index)
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
        completed_pages.append(page_index)

    report_path = work_dir / "report.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write("page\tdiff%\tnote\n")
        if args.text_only:
            for page_index in requested_pages:
                note = "text-only" if page_index not in missing_pages else "rhwp SVG 없음"
                report.write(f"{page_index + 1}\tnot-run\t{note}\n")
        else:
            rows.sort(key=lambda row: -row[1])
            for page_index, score, note in rows:
                report.write(f"{page_index + 1}\t{score}\t{note}\n")

    text_report_path = work_dir / "text-report.tsv"
    with text_report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\treference_only\tsvg_only\treference_only_chars\tsvg_only_chars\tnote\n"
        )
        for page_index, missing_count, extra_count, missing, extra, note in text_rows:
            report.write(
                f"{page_index + 1}\t{missing_count}\t{extra_count}\t{missing}\t{extra}\t{note or '-'}\n"
            )

    write_text_owner_shift_ledger(work_dir, text_differences)
    write_text_owner_sequence_ledger(work_dir, text_layers)
    if args.layout_ledger:
        write_table_fragment_ledger(
            work_dir,
            tree_dir,
            requested_pages,
            text_differences,
        )
    write_page_count_ledger(
        work_dir,
        reference_page_count=reference_page_count,
        full_svg_page_count=full_svg_page_count,
        full_render_tree_page_count=full_render_tree_page_count,
    )

    all_missing_pages = sorted(set(missing_pages + layout_missing_pages))
    all_completed_pages = [
        page_index for page_index in completed_pages if page_index not in all_missing_pages
    ]
    write_run_state(
        work_dir,
        requested_pages=requested_pages,
        completed_pages=all_completed_pages,
        missing_pages=all_missing_pages,
        text_only=args.text_only,
    )

    print(f"기준 PDF: {reference_pdf}")
    print(f"등급: {fixture.reference_grade}")
    print(f"요청: {len(requested_pages)}쪽, 완료: {len(all_completed_pages)}쪽, 누락: {len(all_missing_pages)}쪽")
    if not args.text_only:
        print("diff 랭킹(top 8):")
        for page_index, score, note in rows[:8]:
            print(f"  p{page_index + 1}: {score}% {note}")
    print("pixel report:", report_path)
    print("text report:", text_report_path)
    print("text owner-shift candidates:", work_dir / "text-owner-shift-candidates.tsv")
    print("text owner-sequence candidates:", work_dir / "text-owner-sequence-candidates.tsv")
    print("page-count ledger:", work_dir / "page-count-ledger.tsv")
    if args.layout_ledger:
        print("layout ledger:", work_dir / "layout-candidates.tsv")
        print("table fragment candidates:", work_dir / "table-fragment-candidates.tsv")
    print("run state:", work_dir / "run-state.tsv")
    return 0 if not all_missing_pages else 1


if __name__ == "__main__":
    raise SystemExit(main())
