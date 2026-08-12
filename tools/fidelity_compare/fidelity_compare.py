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


def configured_font_paths(env: Mapping[str, str] = os.environ) -> list[Path]:
    """Return valid font directories from the portable path-list environment.

    ``RHWP_FONT_PATH_DIR`` historically accepted one directory.  Supporting the
    platform path separator keeps that form valid while allowing a fidelity run
    to expose both the Hancom Install and All directories to Chrome.
    """
    raw = env.get("RHWP_FONT_PATH_DIR", "")
    return [
        Path(value).expanduser().resolve()
        for value in raw.split(os.pathsep)
        if value and Path(value).expanduser().is_dir()
    ]


def svg_font_export_option(env: Mapping[str, str] = os.environ) -> str:
    """Return the SVG font mode requested for a fidelity capture.

    The default preserves compact, portable local() SVG.  ``full`` is intended
    for an evidence run with proprietary font directories supplied: it embeds
    the selected Hancom-compatible face so sandboxed Chrome cannot silently
    replace it with a system font.
    """
    mode = env.get("RHWP_SVG_FONT_MODE", "style").lower()
    options = {
        "style": "--font-style",
        "subset": "--embed-fonts",
        "full": "--embed-fonts=full",
    }
    try:
        return options[mode]
    except KeyError as error:
        raise ValueError(
            "RHWP_SVG_FONT_MODE은 style, subset, full 중 하나여야 합니다."
        ) from error


def chrome_fontconfig_environment(
    work_dir: Path,
    env: Mapping[str, str] = os.environ,
    *,
    os_name: str = os.name,
    platform: str = sys.platform,
) -> dict[str, str] | None:
    """Create a per-run Linux fontconfig setup for Chrome local() fonts.

    ``rhwp export-svg --font-style`` emits local() aliases and does not embed
    proprietary fonts.  ``--font-path`` is enough for rhwp's own loaders but
    Chrome uses fontconfig independently.  On Linux, register the same supplied
    directories only for this capture process; macOS and Windows keep their
    native installed-font behavior unchanged.
    """
    font_dirs = configured_font_paths(env)
    if os_name == "nt" or platform == "darwin" or not font_dirs:
        return None

    config_dir = work_dir / "_fontconfig"
    config_dir.mkdir(parents=True, exist_ok=True)
    entries = "\n".join(
        f"  <dir>{html.escape(str(path), quote=True)}</dir>" for path in font_dirs
    )
    (config_dir / "fonts.conf").write_text(
        "<?xml version=\"1.0\"?>\n"
        "<!DOCTYPE fontconfig SYSTEM \"fonts.dtd\">\n"
        "<fontconfig>\n"
        "  <include ignore_missing=\"yes\">/etc/fonts/fonts.conf</include>\n"
        f"{entries}\n"
        "</fontconfig>\n",
        encoding="utf-8",
    )
    configured = dict(env)
    configured["FONTCONFIG_PATH"] = str(config_dir)
    configured["FONTCONFIG_FILE"] = "fonts.conf"
    return configured


def capture_with_chrome(
    chrome: str,
    source: Path,
    out_png: Path,
    width: int,
    height: int,
    *,
    attempts: int = 2,
    env: Mapping[str, str] | None = None,
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
            env=env,
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


def svg_to_png(
    svg_path: Path,
    out_png: Path,
    chrome: str,
    chrome_env: Mapping[str, str] | None = None,
) -> bool:
    if out_png.is_file() and out_png.stat().st_size > 0:
        return True
    head = svg_path.read_text(encoding="utf-8", errors="ignore")[:600]
    width_match = re.search(r'width="([0-9.]+)"', head)
    height_match = re.search(r'height="([0-9.]+)"', head)
    width = math.ceil(float(width_match.group(1))) + 2 if width_match else 810
    height = math.ceil(float(height_match.group(1))) + 2 if height_match else 1140
    return capture_with_chrome(chrome, svg_path, out_png, width, height, env=chrome_env)


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
    title: str,
    left: Path,
    right: Path,
    out_png: Path,
    work_dir: Path,
    chrome: str,
    chrome_env: Mapping[str, str] | None = None,
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
    return capture_with_chrome(
        chrome, html_path, out_png, 1440, 1040, env=chrome_env
    )


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


def _svg_viewport(root: ET.Element) -> tuple[float, float, float, float] | None:
    """Return the root SVG viewport when it is an axis-aligned numeric box."""
    view_box = root.get("viewBox")
    if view_box:
        try:
            x, y, width, height = (float(value) for value in view_box.replace(",", " ").split())
        except ValueError:
            return None
        if width > 0.0 and height > 0.0:
            return (x, y, x + width, y + height)
    width = _svg_float(root, "width")
    height = _svg_float(root, "height")
    if width is not None and height is not None and width > 0.0 and height > 0.0:
        return (0.0, 0.0, width, height)
    return None


def svg_visible_text(svg_path: Path) -> tuple[str, int]:
    """Extract text whose baseline band intersects the effective rhwp clip.

    `export-svg` deliberately retains descendants of earlier table fragments even
    when an ancestor `body-clip-*`/`cell-clip-*` makes them completely invisible.
    The ordinary text ledger preserves those source nodes for forensic work, but
    page-owner comparison needs the text the user can actually see.  This helper
    follows the axis-aligned clips emitted by rhwp and applies a conservative
    baseline band; unknown coordinates stay included rather than becoming a false
    negative.
    """
    root = ET.parse(svg_path).getroot()
    clip_rectangles = _svg_clip_rectangles(root)
    viewport = _svg_viewport(root)
    parts: list[str] = []
    excluded_chars = 0

    def text_is_visible(
        element: ET.Element, active_clip: tuple[float, float, float, float] | None
    ) -> bool:
        if active_clip is None:
            return False
        y = _svg_float(element, "y")
        font_size = _svg_float(element, "font-size")
        if y is None or font_size is None or font_size <= 0.0:
            return True
        # SVG `y` is a baseline.  The band deliberately over-approximates a
        # glyph so a partially visible line remains in the ledger.
        text_top = y - font_size
        text_bottom = y + font_size * 0.3
        return text_bottom > active_clip[1] and text_top < active_clip[3]

    def walk(
        element: ET.Element, active_clip: tuple[float, float, float, float] | None
    ) -> None:
        nonlocal excluded_chars
        tag = _svg_local_name(element)
        if tag in {"defs", "clipPath"}:
            return
        if element.get("display") == "none" or element.get("visibility") == "hidden":
            return
        clip_id = _clip_id_from_attr(element.get("clip-path"))
        if clip_id is not None and clip_id in clip_rectangles:
            active_clip = _intersect_svg_rectangles(active_clip, clip_rectangles[clip_id])
        if tag == "text":
            text = "".join(element.itertext())
            if text_is_visible(element, active_clip):
                parts.append(text)
            else:
                excluded_chars += sum(normalized_characters(text).values())
            return
        for child in element:
            walk(child, active_clip)

    walk(root, viewport)
    return "".join(parts), excluded_chars


def svg_glyph_risks(text: str) -> Counter[str]:
    """Return text glyphs that can become a visible tofu in a public font.

    SVG output is consumed by Chrome/Canvas where HWP-only fonts are generally
    unavailable.  Private-use code points therefore are not harmless text
    metadata: they are a direct missing-glyph candidate.  U+FFFD is reported
    separately because it means a decoder already lost the original glyph.

    This is intentionally a *candidate* ledger rather than a PDF text diff.
    A reference PDF may itself lack the HWP private font, while a raw PUA in
    rhwp SVG is independently actionable and was the cause of #2007's
    U+F02FB tofu bullet.
    """

    def is_private_use(ch: str) -> bool:
        code_point = ord(ch)
        return (
            0xE000 <= code_point <= 0xF8FF
            or 0xF0000 <= code_point <= 0xFFFFD
            or 0x100000 <= code_point <= 0x10FFFD
        )

    return Counter(ch for ch in text if is_private_use(ch) or ch == "\uFFFD")


def write_svg_glyph_risk_report(
    work_dir: Path, glyph_risks: Mapping[int, Counter[str]], requested_pages: Sequence[int]
) -> Path:
    """Write every raw PUA/replacement glyph candidate, including zero rows."""
    report_path = work_dir / "svg-glyph-risk-report.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write("page\trisk_count\tglyphs\tnote\n")
        for page_index in requested_pages:
            risks = glyph_risks.get(page_index)
            if risks is None:
                report.write(f"{page_index + 1}\t-\t-\tSVG 없음 — glyph 미검사\n")
                continue
            if not risks:
                report.write(f"{page_index + 1}\t0\t-\t-\n")
                continue
            report.write(
                f"{page_index + 1}\t{sum(risks.values())}\t"
                f"{counter_summary(risks)}\t"
                "raw PUA 또는 U+FFFD — 공개 글꼴에서 두부 후보\n"
            )
    return report_path


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


def visible_text_excess_candidates(
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
    clip_excluded_chars: Mapping[int, int],
) -> list[dict[str, object]]:
    """Find a page that visibly contains substantial text beyond its PDF peer.

    This is complementary to reciprocal adjacent-page matching.  When rhwp
    renders an entire reference page *and* source text belonging to later pages,
    no reciprocal difference exists: the later page can retain the same text
    again.  Require the PDF text to be almost wholly present so font substitution
    or an ordinary replacement does not become a page-owner candidate.
    """
    candidates: list[dict[str, object]] = []
    for page_index, (missing, extra) in sorted(page_differences.items()):
        missing_count = sum(missing.values())
        extra_count = sum(extra.values())
        if extra_count < 48 or missing_count > max(8, int(extra_count * 0.15)):
            continue
        candidates.append(
            {
                "page": page_index,
                "reference_only": missing_count,
                "visible_svg_only": extra_count,
                "clip_excluded_chars": clip_excluded_chars.get(page_index, 0),
            }
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
        svg_font_export_option(),
        "-p",
        str(page_index),
        "-o",
        str(svg_dir),
    ]
    for font_path in configured_font_paths():
        command.extend(["--font-path", str(font_path)])
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
    command = [
        rhwp,
        "export-svg",
        str(source),
        svg_font_export_option(),
        "--json",
        "-o",
        str(svg_dir),
    ]
    for font_path in configured_font_paths():
        command.extend(["--font-path", str(font_path)])
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


def visible_text_run_text(node: Mapping[str, object]) -> str | None:
    """Return painted TextRun text, excluding blank/private layout markers."""
    if node.get("type") != "TextRun":
        return None
    display = node.get("displayText")
    source = node.get("text")
    text = display if isinstance(display, str) else source
    if not isinstance(text, str) or not text.replace("\U000f081c", "").strip():
        return None
    return text


def table_cell_text_boundary_candidates(
    tree: Mapping[str, object],
    *,
    minimum_overflow_px: float = 2.0,
) -> list[dict[str, object]]:
    """Find painted line boxes and visible-ending natural width risks in Cells.

    A TextRun bbox is its natural measured width, not necessarily the final SVG
    glyph extent after stored spacing/justification.  A run whose overflowing
    edge is only trailing/leading whitespace is therefore ignored while a run
    ending in a visible character remains a review candidate.  A TextLine that
    itself crosses the Cell is reported directly. Descendant Cells are never
    charged to an outer Cell, and wholly detached retained continuation nodes
    are ignored.
    """
    candidates: list[dict[str, object]] = []

    def collect_owned_text_lines(
        cell: Mapping[str, object],
    ) -> list[
        tuple[
            tuple[float, float, float, float],
            list[tuple[tuple[float, float, float, float], str]],
            bool,
        ]
    ]:
        lines: list[
            tuple[
                tuple[float, float, float, float],
                list[tuple[tuple[float, float, float, float], str]],
                bool,
            ]
        ] = []

        def collect_runs(
            node: Mapping[str, object],
            runs: list[tuple[tuple[float, float, float, float], str]],
        ) -> None:
            if node.get("type") == "Cell":
                return
            text = visible_text_run_text(node)
            if text is not None:
                box = bbox_from_node(node)
                if box is not None and box[2] > 0.0 and box[3] > 0.0:
                    runs.append((box, text))
            children = node.get("children")
            if isinstance(children, list):
                for child in children:
                    if isinstance(child, Mapping):
                        collect_runs(child, runs)

        def walk(node: Mapping[str, object], *, is_owner: bool) -> None:
            if not is_owner and node.get("type") == "Cell":
                return
            if node.get("type") == "TextLine":
                line_box = bbox_from_node(node)
                if line_box is None or line_box[2] <= 0.0 or line_box[3] <= 0.0:
                    return
                runs: list[tuple[tuple[float, float, float, float], str]] = []
                collect_runs(node, runs)
                visible = bool(runs) or text_line_has_visible_paint(node)
                if visible:
                    lines.append((line_box, runs, visible))
                return
            children = node.get("children")
            if isinstance(children, list):
                for child in children:
                    if isinstance(child, Mapping):
                        walk(child, is_owner=False)

        walk(cell, is_owner=True)
        return lines

    def inspect_cell(
        cell: Mapping[str, object], table: Mapping[str, object] | None
    ) -> None:
        cell_box = bbox_from_node(cell)
        if cell_box is None or cell_box[2] <= 0.0 or cell_box[3] <= 0.0:
            return
        cell_left, cell_top, cell_width, cell_height = cell_box
        cell_right = cell_left + cell_width
        cell_bottom = cell_top + cell_height
        def overflow_for(
            box: tuple[float, float, float, float],
        ) -> dict[str, float]:
            box_left, box_top, box_width, box_height = box
            box_right = box_left + box_width
            box_bottom = box_top + box_height
            return {
                "left": max(0.0, cell_left - box_left),
                "right": max(0.0, box_right - cell_right),
                "top": max(0.0, cell_top - box_top),
                "bottom": max(0.0, box_bottom - cell_bottom),
            }

        def overlaps_cell(box: tuple[float, float, float, float]) -> bool:
            box_left, box_top, box_width, box_height = box
            return (
                min(cell_right, box_left + box_width) - max(cell_left, box_left) > 0.0
                and min(cell_bottom, box_top + box_height) - max(cell_top, box_top)
                > 0.0
            )

        def append_candidate(
            *,
            candidate_kind: str,
            node_type: str,
            text_box: tuple[float, float, float, float],
            line_box: tuple[float, float, float, float],
            edges: tuple[str, ...],
            overflow: Mapping[str, float],
            edge_clearance_px: float,
            text: str,
        ) -> None:
            candidates.append(
                {
                    "pi": table.get("pi") if table is not None else None,
                    "ci": table.get("ci") if table is not None else None,
                    "rows": table.get("rows") if table is not None else None,
                    "cols": table.get("cols") if table is not None else None,
                    "row": cell.get("row"),
                    "col": cell.get("col"),
                    "candidate_kind": candidate_kind,
                    "cell_bbox": tuple(round(value, 1) for value in cell_box),
                    "node_type": node_type,
                    "line_bbox": tuple(round(value, 1) for value in line_box),
                    "text_bbox": tuple(round(value, 1) for value in text_box),
                    "edges": edges,
                    "edge_clearance_px": round(edge_clearance_px, 1),
                    "overflow_left_px": round(overflow["left"], 1),
                    "overflow_right_px": round(overflow["right"], 1),
                    "overflow_top_px": round(overflow["top"], 1),
                    "overflow_bottom_px": round(overflow["bottom"], 1),
                    "max_overflow_px": round(max(overflow.values()), 1),
                    "text": text,
                }
            )

        for line_box, runs, _visible in collect_owned_text_lines(cell):
            if not overlaps_cell(line_box):
                continue
            line_overflow = overflow_for(line_box)
            line_edges = tuple(
                edge
                for edge in ("left", "right", "top", "bottom")
                if line_overflow[edge] >= minimum_overflow_px
            )
            if line_edges:
                append_candidate(
                    candidate_kind="line_boundary_overflow",
                    node_type="TextLine",
                    text_box=line_box,
                    line_box=line_box,
                    edges=line_edges,
                    overflow=line_overflow,
                    edge_clearance_px=0.0,
                    text="painted line",
                )
                continue

            line_left, _line_top, line_width, _line_height = line_box
            line_right = line_left + line_width
            horizontal_clearance = {
                "left": max(0.0, line_left - cell_left),
                "right": max(0.0, cell_right - line_right),
            }
            for text_box, text in runs:
                if not overlaps_cell(text_box):
                    continue
                overflow = overflow_for(text_box)
                edge_text = text.strip("\U000f081c")
                edges = tuple(
                    edge
                    for edge in ("left", "right")
                    if overflow[edge] >= minimum_overflow_px
                    and edge_text
                    and not (
                        edge == "left" and edge_text[0].isspace()
                        or edge == "right" and edge_text[-1].isspace()
                    )
                )
                if not edges:
                    continue
                append_candidate(
                    candidate_kind="natural_visible_width_risk",
                    node_type="TextRun",
                    text_box=text_box,
                    line_box=line_box,
                    edges=edges,
                    overflow=overflow,
                    edge_clearance_px=min(horizontal_clearance[edge] for edge in edges),
                    text=text,
                )

    def walk(
        node: Mapping[str, object],
        table: Mapping[str, object] | None = None,
        source_table: Mapping[str, object] | None = None,
    ) -> None:
        node_type = node.get("type")
        if node_type == "Table":
            table = node
            if node.get("pi") is not None or node.get("ci") is not None:
                source_table = node
        if node_type == "Cell":
            inspect_cell(node, source_table or table)
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, Mapping):
                    walk(child, table, source_table)

    walk(tree)
    return sorted(
        candidates,
        key=lambda candidate: (
            float(candidate["cell_bbox"][1]),
            float(candidate["cell_bbox"][0]),
            float(candidate["text_bbox"][1]),
            float(candidate["text_bbox"][0]),
            str(candidate["pi"]),
            str(candidate["ci"]),
        ),
    )


def table_cell_text_overlap_candidates(
    tree: Mapping[str, object],
) -> list[dict[str, object]]:
    """Find physically overlapping painted TextLines owned by one table cell.

    Text-only PDF/SVG comparison cannot see a line duplicated at the same
    coordinates: all characters may still be present on the correct page.
    This render-tree rule therefore groups painted TextLine boxes by their
    owning TableCell (never across nested cells) and records a candidate when
    two substantial lines share both a material vertical band and most of the
    narrower line's horizontal extent.  It deliberately remains candidate-only
    because a document can intentionally layer text inside a drawing object.
    """
    minimum_horizontal_overlap_px = 24.0
    minimum_horizontal_overlap_ratio = 0.45
    minimum_vertical_overlap_px = 3.0
    minimum_vertical_overlap_ratio = 0.35
    candidates: list[dict[str, object]] = []

    def collect_owned_text_lines(cell: Mapping[str, object]) -> list[tuple[float, float, float, float]]:
        lines: list[tuple[float, float, float, float]] = []

        def walk(node: Mapping[str, object], *, is_owner: bool) -> None:
            if not is_owner and node.get("type") == "Cell":
                # Nested table cells own their own text; mixing them with the
                # outer cell would manufacture false overlaps.
                return
            if node.get("type") == "TextLine" and text_line_has_visible_paint(node):
                box = bbox_from_node(node)
                if box is not None and box[2] > 0.0 and box[3] > 0.0:
                    lines.append(box)
            children = node.get("children")
            if isinstance(children, list):
                for child in children:
                    if isinstance(child, Mapping):
                        walk(child, is_owner=False)

        walk(cell, is_owner=True)
        return lines

    def inspect_cell(
        cell: Mapping[str, object], table: Mapping[str, object] | None
    ) -> None:
        cell_box = bbox_from_node(cell)
        if cell_box is None:
            return
        lines = sorted(collect_owned_text_lines(cell), key=lambda box: (box[1], box[0]))
        overlaps: list[tuple[tuple[float, float, float, float], tuple[float, float, float, float], float, float]] = []
        for index, first in enumerate(lines):
            first_bottom = first[1] + first[3]
            for second in lines[index + 1 :]:
                if second[1] >= first_bottom - minimum_vertical_overlap_px:
                    break
                overlap_y = min(first_bottom, second[1] + second[3]) - max(first[1], second[1])
                overlap_x = min(first[0] + first[2], second[0] + second[2]) - max(first[0], second[0])
                if overlap_y < max(minimum_vertical_overlap_px, min(first[3], second[3]) * minimum_vertical_overlap_ratio):
                    continue
                if overlap_x < max(minimum_horizontal_overlap_px, min(first[2], second[2]) * minimum_horizontal_overlap_ratio):
                    continue
                overlaps.append((first, second, overlap_x, overlap_y))

        if not overlaps:
            return
        first, second, overlap_x, overlap_y = overlaps[0]
        candidates.append(
            {
                "pi": table.get("pi") if table is not None else None,
                "ci": table.get("ci") if table is not None else None,
                "rows": table.get("rows") if table is not None else None,
                "cols": table.get("cols") if table is not None else None,
                "row": cell.get("row"),
                "col": cell.get("col"),
                "cell_bbox": [round(value, 1) for value in cell_box],
                "overlap_pair_count": len(overlaps),
                "max_overlap_x_px": round(max(pair[2] for pair in overlaps), 1),
                "max_overlap_y_px": round(max(pair[3] for pair in overlaps), 1),
                "first_line_bbox": [round(value, 1) for value in first],
                "second_line_bbox": [round(value, 1) for value in second],
                "first_overlap_x_px": round(overlap_x, 1),
                "first_overlap_y_px": round(overlap_y, 1),
            }
        )

    def walk(
        node: Mapping[str, object],
        table: Mapping[str, object] | None = None,
        source_table: Mapping[str, object] | None = None,
    ) -> None:
        node_type = node.get("type")
        if node_type == "Table":
            table = node
            if node.get("pi") is not None or node.get("ci") is not None:
                source_table = node
        if node_type == "Cell":
            inspect_cell(node, source_table or table)
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, Mapping):
                    walk(child, table, source_table)

    walk(tree)
    return sorted(
        candidates,
        key=lambda candidate: (
            float(candidate["cell_bbox"][1]),
            float(candidate["cell_bbox"][0]),
            str(candidate["pi"]),
            str(candidate["ci"]),
        ),
    )


def square_wrap_text_overlap_candidates(
    tree: Mapping[str, object],
) -> list[dict[str, object]]:
    """Find Body text that crosses or loses clearance against a flow-wrapped image.

    Square/Tight/Through images reserve a text-flow band.  Three or more Body
    lines crossing at least half of a substantial image width is a strong
    physical-overlap candidate.  A separate edge-contact candidate catches the
    equally harmful case where three Body lines meet or only slightly enter the
    image edge, which is how a lost HWP outer margin appears in the render tree.
    BehindText/InFrontOfText may overlap intentionally and are excluded.  The
    render-tree JSON exposes the source wrap mode so this low-cost ledger works
    without PDF raster dependencies.
    """
    minimum_line_count = 3
    edge_contact_tolerance_px = 1.0
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
        edge_contacts: list[tuple[tuple[float, float, float, float], str, float]] = []
        for line in body_lines:
            lx, ly, lw, lh = line
            overlap_x = min(ix + iw, lx + lw) - max(ix, lx)
            overlap_y = min(iy + ih, ly + lh) - max(iy, ly)
            if overlap_x >= iw * 0.5 and overlap_y >= min(5.0, lh * 0.5):
                overlap_lines.append(line)
                continue
            if overlap_y < min(5.0, lh * 0.5):
                continue

            line_right = lx + lw
            image_right = ix + iw
            # A Body line originating left/right of the image may legitimately
            # approach its edge, but contact (or a shallow intrusion) over
            # three lines means the Square exclusion's outer clearance may
            # have vanished.  Large intrusions are reported by the stronger
            # physical-overlap branch above.
            if lx < ix and line_right >= ix - edge_contact_tolerance_px:
                edge_contacts.append((line, "left", ix - line_right))
            elif line_right > image_right and lx <= image_right + edge_contact_tolerance_px:
                edge_contacts.append((line, "right", lx - image_right))

        if len(overlap_lines) >= minimum_line_count:
            candidates.append(
                {
                    "pi": para_index,
                    "ci": control_index,
                    "text_wrap": text_wrap,
                    "candidate_kind": "physical_overlap",
                    "overlap_line_count": len(overlap_lines),
                    "image_bbox": [round(value, 1) for value in image],
                    "first_line_bbox": [round(value, 1) for value in overlap_lines[0]],
                    "last_line_bbox": [round(value, 1) for value in overlap_lines[-1]],
                }
            )
        elif len(edge_contacts) >= minimum_line_count:
            edges = {edge for _, edge, _ in edge_contacts}
            candidates.append(
                {
                    "pi": para_index,
                    "ci": control_index,
                    "text_wrap": text_wrap,
                    "candidate_kind": "edge_clearance_loss",
                    "edge": next(iter(edges)) if len(edges) == 1 else "mixed",
                    "edge_contact_line_count": len(edge_contacts),
                    "min_clearance_px": round(
                        min(clearance for _, _, clearance in edge_contacts), 1
                    ),
                    "image_bbox": [round(value, 1) for value in image],
                    "first_line_bbox": [
                        round(value, 1) for value in edge_contacts[0][0]
                    ],
                    "last_line_bbox": [
                        round(value, 1) for value in edge_contacts[-1][0]
                    ],
                }
            )
    return candidates


def deferred_square_picture_page_top_drift_candidates(
    tree: Mapping[str, object],
) -> list[dict[str, object]]:
    """Find a deferred Square picture that consumes a previous-page Y offset twice.

    A native HWP5 Square picture can be materialized as the first item of the
    next physical page while its successor text begins at the new page's body
    top.  Its frame must then also start at the body top.  If the frame begins
    materially lower, the source paragraph's positive vertical offset has
    leaked into the new owner page.  This is a candidate-only structural rule:
    it intentionally requires the characteristic first-column image and a
    side-wrap text line at the body top, so ordinary positioned pictures are
    not reported.
    """
    minimum_top_drift_px = 20.0
    candidates: list[dict[str, object]] = []

    def visible_text_lines(node: Mapping[str, object]) -> list[tuple[float, float, float, float]]:
        lines: list[tuple[float, float, float, float]] = []

        def walk(child: Mapping[str, object]) -> None:
            if child.get("type") == "TextLine" and text_line_has_visible_paint(child):
                box = bbox_from_node(child)
                if box is not None:
                    lines.append(box)
            nested = child.get("children")
            if isinstance(nested, list):
                for grandchild in nested:
                    if isinstance(grandchild, Mapping):
                        walk(grandchild)

        walk(node)
        return lines

    children = tree.get("children")
    if not isinstance(children, list):
        return candidates
    for body in children:
        if not isinstance(body, Mapping) or body.get("type") != "Body":
            continue
        body_box = bbox_from_node(body)
        body_children = body.get("children")
        if body_box is None or not isinstance(body_children, list):
            continue
        body_y = body_box[1]
        for column in body_children:
            if not isinstance(column, Mapping) or column.get("type") != "Column":
                continue
            column_children = column.get("children")
            if not isinstance(column_children, list):
                continue
            first_item = next(
                (child for child in column_children if isinstance(child, Mapping)),
                None,
            )
            if not isinstance(first_item, Mapping) or first_item.get("type") != "Image":
                continue
            if first_item.get("textWrap") != "Square":
                continue
            image_box = bbox_from_node(first_item)
            if image_box is None or image_box[2] < 80.0 or image_box[3] < 80.0:
                continue
            image_x, image_y, image_width, _ = image_box
            top_drift = image_y - body_y
            if top_drift < minimum_top_drift_px:
                continue
            first_wrap_line = next(
                (
                    line
                    for line in visible_text_lines(column)
                    if line[1] <= body_y + min(24.0, line[3] + 4.0)
                    and (line[0] + line[2] <= image_x + 1.0 or line[0] >= image_x + image_width - 1.0)
                ),
                None,
            )
            if first_wrap_line is None:
                continue
            candidates.append(
                {
                    "pi": first_item.get("pi"),
                    "ci": first_item.get("ci"),
                    "text_wrap": "Square",
                    "candidate_kind": "deferred_page_start_offset_drift",
                    "image_bbox": [round(value, 1) for value in image_box],
                    "body_top_y": round(body_y, 1),
                    "image_top_drift_px": round(top_drift, 1),
                    "first_wrap_line_bbox": [round(value, 1) for value in first_wrap_line],
                }
            )
    return candidates


def layout_candidates(tree: Mapping[str, object]) -> tuple[int, int, int, int, int, int, int]:
    """(body↔각주, table↔footer, table/frame, image/frame, Square/text, deferred Square, cell/text) 후보 수."""
    page_bbox = bbox_from_node(tree)
    if page_bbox is None:
        return (0, 0, 0, 0, 0, 0, 0)
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
    deferred_square_page_top_drift = len(deferred_square_picture_page_top_drift_candidates(tree))
    table_cell_text_overlap = len(table_cell_text_overlap_candidates(tree))
    return (
        body_footnote_lines,
        table_footer,
        table_outside_frame,
        image_outside_frame,
        square_wrap_text_overlap,
        deferred_square_page_top_drift,
        table_cell_text_overlap,
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


def page_boundary_fidelity_candidates(
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
    page_text_layers: Mapping[int, tuple[str, str]],
    *,
    tree_dir: Path | None = None,
    requested_pages: Sequence[int] = (),
) -> list[dict[str, object]]:
    """Join adjacent-page owner signals into one review-priority ledger.

    The individual text, sequence, and table-fragment ledgers deliberately stay
    conservative and independent.  That made a real boundary failure easy to
    overlook during a broad sweep: an operator had to notice that three files
    described the same pN→pN+1 event.  This helper preserves those raw ledgers
    while producing one candidate per physical boundary.  A source table that
    also has reciprocal PDF/SVG text movement is promoted to the explicit
    ``table_fragment_text_owner_drift`` kind; an ordinary small owner move is
    still retained as ``text_owner_shift`` so short captions and labels are not
    hidden by a 16-character sequence threshold.

    The result remains a candidate: only the reference PDF determines whether
    the physical page owner is actually wrong.
    """
    by_boundary: dict[tuple[int, int, str], dict[str, object]] = {}

    def candidate_for(
        page_index: int, next_page: int, direction: str
    ) -> dict[str, object]:
        key = (page_index, next_page, direction)
        return by_boundary.setdefault(
            key,
            {
                "page": page_index,
                "next_page": next_page,
                "direction": direction,
                "counter_chars": 0,
                "sequence_chars": 0,
                "sequence": "",
                "table_fragments": [],
            },
        )

    for owner_shift in adjacent_text_owner_shift_candidates(page_differences):
        candidate = candidate_for(
            int(owner_shift["page"]),
            int(owner_shift["next_page"]),
            str(owner_shift["direction"]),
        )
        candidate["counter_chars"] = max(
            int(candidate["counter_chars"]), int(owner_shift["shared_count"])
        )

    for owner_shift in adjacent_text_owner_sequence_candidates(page_text_layers):
        candidate = candidate_for(
            int(owner_shift["page"]),
            int(owner_shift["next_page"]),
            str(owner_shift["direction"]),
        )
        sequence = str(owner_shift["sequence"])
        if len(sequence) > int(candidate["sequence_chars"]):
            candidate["sequence_chars"] = len(sequence)
            candidate["sequence"] = sequence

    if tree_dir is not None:
        requested = list(requested_pages)
        for table_candidate in table_fragment_candidates(
            tree_dir, requested, page_differences
        ):
            page_table = table_candidate["page_table"]
            next_page_table = table_candidate.get("next_page_table")
            if not isinstance(page_table, Mapping) or not isinstance(next_page_table, Mapping):
                continue
            page_index = int(page_table["page"])
            next_page = int(next_page_table["page"])
            for direction in ("rhwp_earlier_than_reference", "rhwp_later_than_reference"):
                candidate = by_boundary.get((page_index, next_page, direction))
                if candidate is None:
                    continue
                fragments = candidate["table_fragments"]
                assert isinstance(fragments, list)
                fragments.append(
                    {
                        "pi": page_table.get("pi"),
                        "ci": page_table.get("ci"),
                        "rows": page_table.get("rows"),
                        "cols": page_table.get("cols"),
                        "signals": list(table_candidate["signals"]),
                    }
                )

    candidates: list[dict[str, object]] = []
    for candidate in by_boundary.values():
        fragments = candidate["table_fragments"]
        assert isinstance(fragments, list)
        candidate["kind"] = (
            "table_fragment_text_owner_drift" if fragments else "text_owner_shift"
        )
        candidate["priority_chars"] = max(
            int(candidate["counter_chars"]), int(candidate["sequence_chars"])
        )
        candidates.append(candidate)

    return sorted(
        candidates,
        key=lambda candidate: (
            int(candidate["page"]),
            int(candidate["next_page"]),
            str(candidate["direction"]),
        ),
    )


def format_bbox(box: object) -> str:
    if not isinstance(box, (tuple, list)) or len(box) != 4:
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


FLOAT_OWNER_SHIFT_WRAPS = frozenset({"TopAndBottom", "Square", "Tight", "Through"})
FLOAT_OWNER_SHIFT_MAX_TOP_RATIO = 0.25
FLOAT_OWNER_SHIFT_MIN_DIMENSION = 80.0


def successor_top_float_records(
    tree: Mapping[str, object], page_index: int
) -> list[dict[str, object]]:
    """Collect substantial Body floats in the successor page's top quarter.

    A float alone does not establish a page-break defect.  The narrow geometry
    filter is intentionally used only to attach an explanation to an already
    observed PDF↔SVG text-owner shift.
    """
    page_box = bbox_from_node(tree)
    if page_box is None:
        return []
    _, page_y, _, page_height = page_box
    top_limit = page_y + page_height * FLOAT_OWNER_SHIFT_MAX_TOP_RATIO
    records: list[dict[str, object]] = []

    def walk(node: Mapping[str, object], region: str = "outside") -> None:
        node_type = node.get("type")
        if node_type in {"Body", "FootnoteArea", "Footer", "Header"}:
            region = str(node_type)
        box = bbox_from_node(node)
        if region == "Body" and node_type == "Image" and box is not None:
            text_wrap = node.get("textWrap")
            x, y, width, height = box
            if (
                text_wrap in FLOAT_OWNER_SHIFT_WRAPS
                and width >= FLOAT_OWNER_SHIFT_MIN_DIMENSION
                and height >= FLOAT_OWNER_SHIFT_MIN_DIMENSION
                and y <= top_limit
            ):
                records.append(
                    {
                        "page": page_index,
                        "pi": node.get("pi"),
                        "ci": node.get("ci"),
                        "text_wrap": str(text_wrap),
                        "bbox": box,
                        "top_ratio": (y - page_y) / page_height,
                    }
                )
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, Mapping):
                    walk(child, region)

    walk(tree)
    return records


def successor_float_owner_shift_candidates(
    tree_dir: Path,
    requested_pages: Sequence[int],
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
) -> list[dict[str, object]]:
    """Attach an upper successor-page float to an early rhwp text-owner shift.

    This does not re-detect text movement.  It narrows the existing reciprocal
    PDF/SVG candidate to the page-break pattern where rhwp kept a paragraph
    above a float while the reference continued that paragraph before the same
    successor-page float.
    """
    requested = set(requested_pages)
    candidates: list[dict[str, object]] = []
    for owner_shift in adjacent_text_owner_shift_candidates(page_differences):
        page_index = int(owner_shift["page"])
        next_page = int(owner_shift["next_page"])
        if owner_shift["direction"] != "rhwp_earlier_than_reference":
            continue
        if page_index not in requested and next_page not in requested:
            continue
        tree_path = tree_path_for_page(tree_dir, next_page)
        if tree_path is None:
            continue
        try:
            tree = json.loads(tree_path.read_text(encoding="utf-8"))
        except (OSError, ValueError, json.JSONDecodeError):
            continue
        if not isinstance(tree, Mapping):
            continue
        for float_record in successor_top_float_records(tree, next_page):
            candidates.append(
                {
                    **owner_shift,
                    "float": float_record,
                }
            )

    return sorted(
        candidates,
        key=lambda candidate: (
            int(candidate["page"]),
            int(candidate["next_page"]),
            str(candidate["float"]["pi"]),
            str(candidate["float"]["ci"]),
        ),
    )


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
            "image_outside_frame\tsquare_wrap_text_overlap\t"
            "deferred_square_page_top_drift\ttable_cell_text_overlap\tnote\n"
        )
        for page_index in requested_pages:
            tree_path = tree_path_for_page(tree_dir, page_index)
            if tree_path is None:
                missing_pages.append(page_index)
                report.write(f"{page_index + 1}\t0\t0\t0\t0\t0\t0\t0\trender tree 없음\n")
                continue
            try:
                tree = json.loads(tree_path.read_text(encoding="utf-8"))
                if not isinstance(tree, Mapping):
                    raise ValueError("root가 object가 아님")
                candidates = layout_candidates(tree)
            except (OSError, ValueError, json.JSONDecodeError) as error:
                missing_pages.append(page_index)
                report.write(f"{page_index + 1}\t0\t0\t0\t0\t0\t0\t0\trender tree 읽기 실패: {error}\n")
                continue
            report.write(
                f"{page_index + 1}\t{candidates[0]}\t{candidates[1]}\t"
                f"{candidates[2]}\t{candidates[3]}\t{candidates[4]}\t{candidates[5]}\t"
                f"{candidates[6]}\t-\n"
            )
    return missing_pages


def write_table_cell_text_overlap_ledger(
    work_dir: Path,
    tree_dir: Path,
    requested_pages: Sequence[int],
) -> Path:
    """Write table-cell TextLine overlap candidates for visual PDF review."""
    report_path = work_dir / "table-cell-text-overlap-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tpi\tci\trows\tcols\trow\tcol\tcell_bbox\toverlap_pairs\t"
            "max_overlap_x_px\tmax_overlap_y_px\tfirst_line_bbox\tsecond_line_bbox\t"
            "note\n"
        )
        for page_index in requested_pages:
            tree_path = tree_path_for_page(tree_dir, page_index)
            if tree_path is None:
                continue
            try:
                tree = json.loads(tree_path.read_text(encoding="utf-8"))
            except (OSError, ValueError, json.JSONDecodeError):
                continue
            if not isinstance(tree, Mapping):
                continue
            for candidate in table_cell_text_overlap_candidates(tree):
                report.write(
                    f"{page_index + 1}\t{format_number(candidate['pi'])}\t"
                    f"{format_number(candidate['ci'])}\t{format_number(candidate['rows'])}\t"
                    f"{format_number(candidate['cols'])}\t{format_number(candidate['row'])}\t"
                    f"{format_number(candidate['col'])}\t{format_bbox(candidate['cell_bbox'])}\t"
                    f"{candidate['overlap_pair_count']}\t{candidate['max_overlap_x_px']:.1f}\t"
                    f"{candidate['max_overlap_y_px']:.1f}\t"
                    f"{format_bbox(candidate['first_line_bbox'])}\t"
                    f"{format_bbox(candidate['second_line_bbox'])}\t"
                    "candidate only; two painted TextLine bands overlap within one TableCell\n"
                )
    return report_path


def write_table_cell_text_boundary_ledger(
    work_dir: Path,
    tree_dir: Path,
    requested_pages: Sequence[int],
) -> Path:
    """Write visible text boxes that cross their owning Cell by at least 2px."""
    report_path = work_dir / "table-cell-text-boundary-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tpi\tci\trows\tcols\trow\tcol\tcandidate_kind\tcell_bbox\t"
            "node_type\tline_bbox\ttext_bbox\tedges\tedge_clearance_px\t"
            "max_overflow_px\tleft_px\tright_px\ttop_px\tbottom_px\ttext\tnote\n"
        )
        for page_index in requested_pages:
            tree_path = tree_path_for_page(tree_dir, page_index)
            if tree_path is None:
                continue
            try:
                tree = json.loads(tree_path.read_text(encoding="utf-8"))
            except (OSError, ValueError, json.JSONDecodeError):
                continue
            if not isinstance(tree, Mapping):
                continue
            for candidate in table_cell_text_boundary_candidates(tree):
                edges = candidate["edges"]
                assert isinstance(edges, tuple)
                text = str(candidate["text"]).replace("\t", " ").replace("\n", " ")
                report.write(
                    f"{page_index + 1}\t{format_number(candidate['pi'])}\t"
                    f"{format_number(candidate['ci'])}\t{format_number(candidate['rows'])}\t"
                    f"{format_number(candidate['cols'])}\t{format_number(candidate['row'])}\t"
                    f"{format_number(candidate['col'])}\t{candidate['candidate_kind']}\t"
                    f"{format_bbox(candidate['cell_bbox'])}\t{candidate['node_type']}\t"
                    f"{format_bbox(candidate['line_bbox'])}\t"
                    f"{format_bbox(candidate['text_bbox'])}\t{','.join(edges)}\t"
                    f"{float(candidate['edge_clearance_px']):.1f}\t"
                    f"{float(candidate['max_overflow_px']):.1f}\t"
                    f"{float(candidate['overflow_left_px']):.1f}\t"
                    f"{float(candidate['overflow_right_px']):.1f}\t"
                    f"{float(candidate['overflow_top_px']):.1f}\t"
                    f"{float(candidate['overflow_bottom_px']):.1f}\t{text}\t"
                    "candidate only; line crosses Cell or visible-ending natural width exceeds it\n"
                )
    return report_path


def _svg_local_name(element: ET.Element) -> str:
    return element.tag.rsplit("}", 1)[-1]


def _svg_float(element: ET.Element, name: str) -> float | None:
    value = element.get(name)
    if value is None:
        return None
    try:
        return float(value)
    except ValueError:
        return None


def _svg_clip_rectangles(root: ET.Element) -> dict[str, tuple[float, float, float, float]]:
    """Read axis-aligned SVG clipPath rectangles used by rhwp page/cell clips."""
    clips: dict[str, tuple[float, float, float, float]] = {}
    for element in root.iter():
        if _svg_local_name(element) != "clipPath":
            continue
        clip_id = element.get("id")
        if not clip_id:
            continue
        rect = next(
            (child for child in element if _svg_local_name(child) == "rect"), None
        )
        if rect is None:
            continue
        x = _svg_float(rect, "x")
        y = _svg_float(rect, "y")
        width = _svg_float(rect, "width")
        height = _svg_float(rect, "height")
        if None in {x, y, width, height} or width is None or height is None:
            continue
        if width <= 0.0 or height <= 0.0:
            continue
        assert x is not None and y is not None
        clips[clip_id] = (x, y, x + width, y + height)
    return clips


def _intersect_svg_rectangles(
    first: tuple[float, float, float, float] | None,
    second: tuple[float, float, float, float],
) -> tuple[float, float, float, float] | None:
    if first is None:
        return second
    x0 = max(first[0], second[0])
    y0 = max(first[1], second[1])
    x1 = min(first[2], second[2])
    y1 = min(first[3], second[3])
    if x1 <= x0 or y1 <= y0:
        return None
    return (x0, y0, x1, y1)


def _clip_id_from_attr(value: str | None) -> str | None:
    if not value:
        return None
    match = re.fullmatch(r"\s*url\(#([^)]+)\)\s*", value)
    return match.group(1) if match else None


def svg_text_band_clip_candidates(
    svg_path: Path,
    *,
    minimum_clipped_px: float = 2.0,
) -> list[dict[str, object]]:
    """Find visible glyph bands partially cut by explicit SVG clips.

    The vertical glyph approximation intentionally matches
    :func:`svg_visible_text`.  Only a *partial* intersection is reported:
    wholly excluded stale continuation text is ignored, as is text whose x
    baseline does not touch the effective clip.  Transformed text is skipped
    because its axis-aligned local coordinates cannot be compared safely.
    """
    root = ET.parse(svg_path).getroot()
    clip_rectangles = _svg_clip_rectangles(root)
    candidates: list[dict[str, object]] = []

    def walk(
        element: ET.Element,
        active_clip: tuple[float, float, float, float] | None,
        active_clip_ids: tuple[str, ...],
        *,
        fully_clipped: bool,
        transformed: bool,
        hidden: bool,
    ) -> None:
        tag = _svg_local_name(element)
        if tag in {"defs", "clipPath"}:
            return
        next_hidden = hidden or element.get("display") == "none" or element.get(
            "visibility"
        ) == "hidden"
        opacity = element.get("opacity")
        if opacity is not None:
            try:
                next_hidden = next_hidden or float(opacity) <= 0.0
            except ValueError:
                pass
        next_transformed = transformed or bool(element.get("transform"))
        next_clip = active_clip
        next_clip_ids = active_clip_ids
        next_fully_clipped = fully_clipped
        clip_id = _clip_id_from_attr(element.get("clip-path"))
        if clip_id is not None and clip_id in clip_rectangles:
            next_clip_ids = (*next_clip_ids, clip_id)
            if not next_fully_clipped:
                intersection = _intersect_svg_rectangles(
                    next_clip, clip_rectangles[clip_id]
                )
                next_fully_clipped = next_clip is not None and intersection is None
                next_clip = intersection

        if tag == "text":
            text = "".join(element.itertext())
            visible_text = text.replace("\U000f081c", "").strip()
            if (
                next_hidden
                or next_transformed
                or next_fully_clipped
                or not next_clip_ids
                or next_clip is None
                or not visible_text
                or element.get("fill") == "none"
            ):
                return
            x = _svg_float(element, "x")
            y = _svg_float(element, "y")
            font_size = _svg_float(element, "font-size")
            if x is None or y is None or font_size is None or font_size <= 0.0:
                return
            # Requiring the baseline x to meet the clip avoids flagging a
            # spatially unrelated retained text node with the same y band.
            if x < next_clip[0] - 0.01 or x > next_clip[2] + 0.01:
                return
            # The visibility ledger uses a deliberately generous 1.0em/0.3em
            # band so it never drops potentially painted text.  That envelope
            # is too wide for *partial clip* detection: Korean/Latin ink is
            # normally within roughly 0.8em above and 0.2em below the baseline.
            # Keep this a candidate heuristic; browser/PDF raster remains the
            # authority for exact font-specific outlines.
            text_top = y - font_size * 0.8
            text_bottom = y + font_size * 0.2
            visible_top = max(text_top, next_clip[1])
            visible_bottom = min(text_bottom, next_clip[3])
            visible_height = visible_bottom - visible_top
            band_height = text_bottom - text_top
            if visible_height <= 0.0 or visible_height >= band_height - 0.01:
                return
            clipped_top = max(0.0, next_clip[1] - text_top)
            clipped_bottom = max(0.0, text_bottom - next_clip[3])
            edges = tuple(
                edge
                for edge, amount in (
                    ("top", clipped_top),
                    ("bottom", clipped_bottom),
                )
                if amount >= minimum_clipped_px
            )
            if not edges:
                return
            candidates.append(
                {
                    "text": text,
                    "x": round(x, 1),
                    "baseline_y": round(y, 1),
                    "font_size": round(font_size, 1),
                    "band_top": round(text_top, 1),
                    "band_bottom": round(text_bottom, 1),
                    "edges": edges,
                    "clipped_top_px": round(clipped_top, 1),
                    "clipped_bottom_px": round(clipped_bottom, 1),
                    "visible_height_ratio": round(visible_height / band_height, 3),
                    "clip_ids": next_clip_ids,
                    "clip_rect": tuple(round(value, 1) for value in next_clip),
                }
            )
            return

        for child in element:
            walk(
                child,
                next_clip,
                next_clip_ids,
                fully_clipped=next_fully_clipped,
                transformed=next_transformed,
                hidden=next_hidden,
            )

    walk(
        root,
        None,
        (),
        fully_clipped=False,
        transformed=False,
        hidden=False,
    )
    return sorted(
        candidates,
        key=lambda candidate: (
            float(candidate["baseline_y"]),
            float(candidate["x"]),
            str(candidate["text"]),
        ),
    )


def svg_vertical_lines_with_clips(
    svg_path: Path,
) -> list[dict[str, object]]:
    """Collect painted vertical SVG lines and their effective rhwp clip intersection.

    This is deliberately structural rather than a generic pixel threshold.  A
    border can be emitted into the SVG and still be completely invisible when
    a body or TableCell clip rectangle excludes its stroke.  That exact class
    of error is lost by text ledgers and may be drowned out in a page pixel
    score, while the SVG retains enough geometry to identify it deterministically.
    """
    root = ET.parse(svg_path).getroot()
    clip_rectangles = _svg_clip_rectangles(root)
    lines: list[dict[str, object]] = []

    def walk(
        element: ET.Element,
        active_clip: tuple[float, float, float, float] | None,
        active_clip_ids: tuple[str, ...],
    ) -> None:
        tag = _svg_local_name(element)
        if tag in {"defs", "clipPath"}:
            return
        clip_id = _clip_id_from_attr(element.get("clip-path"))
        next_clip = active_clip
        next_clip_ids = active_clip_ids
        if clip_id is not None and clip_id in clip_rectangles:
            next_clip = _intersect_svg_rectangles(next_clip, clip_rectangles[clip_id])
            next_clip_ids = (*next_clip_ids, clip_id)

        if tag == "line":
            x1 = _svg_float(element, "x1")
            y1 = _svg_float(element, "y1")
            x2 = _svg_float(element, "x2")
            y2 = _svg_float(element, "y2")
            stroke_width = _svg_float(element, "stroke-width") or 1.0
            if None not in {x1, y1, x2, y2}:
                assert x1 is not None and y1 is not None and x2 is not None and y2 is not None
                if abs(x1 - x2) <= 0.01 and abs(y1 - y2) >= 1.0:
                    half_stroke = stroke_width / 2.0
                    lines.append(
                        {
                            "x": x1,
                            "y0": min(y1, y2),
                            "y1": max(y1, y2),
                            "stroke_width": stroke_width,
                            "paint_left": x1 - half_stroke,
                            "paint_right": x1 + half_stroke,
                            "clip": next_clip,
                            "clip_ids": next_clip_ids,
                        }
                    )

        for child in element:
            walk(child, next_clip, next_clip_ids)

    walk(root, None, ())
    return lines


def svg_horizontal_lines_with_clips(
    svg_path: Path,
) -> list[dict[str, object]]:
    """Collect painted horizontal SVG lines and their effective rhwp clip.

    This is the horizontal counterpart to :func:`svg_vertical_lines_with_clips`.
    Continued tables are especially sensitive here: an outer frame can exist
    in the render tree while a page/cell clip removes half (or all) of its
    horizontal stroke at the physical page boundary.
    """
    root = ET.parse(svg_path).getroot()
    clip_rectangles = _svg_clip_rectangles(root)
    lines: list[dict[str, object]] = []

    def walk(
        element: ET.Element,
        active_clip: tuple[float, float, float, float] | None,
        active_clip_ids: tuple[str, ...],
    ) -> None:
        tag = _svg_local_name(element)
        if tag in {"defs", "clipPath"}:
            return
        clip_id = _clip_id_from_attr(element.get("clip-path"))
        next_clip = active_clip
        next_clip_ids = active_clip_ids
        if clip_id is not None and clip_id in clip_rectangles:
            next_clip = _intersect_svg_rectangles(next_clip, clip_rectangles[clip_id])
            next_clip_ids = (*next_clip_ids, clip_id)

        if tag == "line":
            x1 = _svg_float(element, "x1")
            y1 = _svg_float(element, "y1")
            x2 = _svg_float(element, "x2")
            y2 = _svg_float(element, "y2")
            stroke_width = _svg_float(element, "stroke-width") or 1.0
            if None not in {x1, y1, x2, y2}:
                assert x1 is not None and y1 is not None and x2 is not None and y2 is not None
                if abs(y1 - y2) <= 0.01 and abs(x1 - x2) >= 1.0:
                    half_stroke = stroke_width / 2.0
                    lines.append(
                        {
                            "x0": min(x1, x2),
                            "x1": max(x1, x2),
                            "y": y1,
                            "stroke_width": stroke_width,
                            "paint_top": y1 - half_stroke,
                            "paint_bottom": y1 + half_stroke,
                            "clip": next_clip,
                            "clip_ids": next_clip_ids,
                        }
                    )

        for child in element:
            walk(child, next_clip, next_clip_ids)

    walk(root, None, ())
    return lines


def table_records(tree: Mapping[str, object]) -> list[dict[str, object]]:
    """Return all render-tree table boxes used to identify SVG outer edges."""
    records: list[dict[str, object]] = []

    def walk(node: Mapping[str, object]) -> None:
        box = bbox_from_node(node)
        if node.get("type") == "Table" and box is not None:
            table_x, table_y, table_width, table_height = box
            own_outer_vertical_lines: list[tuple[str, float, float, float, float]] = []
            own_horizontal_lines: list[tuple[float, float, float, float]] = []
            children = node.get("children")
            if isinstance(children, list):
                for child in children:
                    if not isinstance(child, Mapping) or child.get("type") != "Line":
                        continue
                    line_box = bbox_from_node(child)
                    if line_box is None:
                        continue
                    line_x, line_y, line_width, line_height = line_box
                    if line_height > max(1.0, line_width):
                        edge = (
                            "left"
                            if abs(line_x - table_x) <= 0.2
                            else "right"
                            if abs(line_x - (table_x + table_width)) <= 0.2
                            else None
                        )
                        if edge is not None:
                            own_outer_vertical_lines.append(
                                (edge, line_x, line_y, line_y + line_height, line_width)
                            )
                    elif line_width >= max(1.0, line_height):
                        own_horizontal_lines.append(
                            (line_x, line_x + line_width, line_y, line_height)
                        )
            records.append(
                {
                    "pi": node.get("pi"),
                    "ci": node.get("ci"),
                    "rows": node.get("rows"),
                    "cols": node.get("cols"),
                    "bbox": box,
                    "own_outer_vertical_lines": own_outer_vertical_lines,
                    "own_horizontal_lines": own_horizontal_lines,
                }
            )
        children = node.get("children")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, Mapping):
                    walk(child)

    walk(tree)
    return records


def svg_table_border_clip_candidates(
    svg_path: Path,
    tree: Mapping[str, object],
) -> list[dict[str, object]]:
    """Find table outer vertical borders emitted wholly outside a parent clip.

    A candidate requires both signals: a render-tree Table outer edge and an
    SVG vertical stroke on that edge whose horizontal paint interval is almost
    entirely excluded by an ancestor clip.  This avoids treating ordinary
    clipped drawings as table-border defects.  It is candidate-only: PDFs can
    legitimately omit a source border, so the generated ledger never declares
    a visual failure on its own.
    """
    candidates: list[dict[str, object]] = []
    for line in svg_vertical_lines_with_clips(svg_path):
        clip = line["clip"]
        if not isinstance(clip, tuple):
            continue
        paint_left = float(line["paint_left"])
        paint_right = float(line["paint_right"])
        visible_width = max(0.0, min(paint_right, clip[2]) - max(paint_left, clip[0]))
        visible_width_ratio = visible_width / max(paint_right - paint_left, 0.01)
        if visible_width_ratio > 0.2:
            continue

        x = float(line["x"])
        line_y0 = float(line["y0"])
        line_y1 = float(line["y1"])
        stroke_width = float(line["stroke_width"])
        for table in table_records(tree):
            table_box = table["bbox"]
            assert isinstance(table_box, tuple)
            _, table_y, _, table_height = table_box
            own_outer_lines = table["own_outer_vertical_lines"]
            assert isinstance(own_outer_lines, list)
            matching_edges = [
                edge
                for edge, expected_x, expected_y0, expected_y1, expected_width in own_outer_lines
                if abs(x - expected_x)
                <= max(0.25, expected_width / 2.0 + 0.2)
                and max(0.0, min(line_y1, expected_y1) - max(line_y0, expected_y0))
                >= min(16.0, table_height * 0.2)
            ]
            if not matching_edges:
                continue
            overlap_height = max(
                0.0,
                min(line_y1, table_y + table_height) - max(line_y0, table_y),
            )
            if overlap_height < min(16.0, table_height * 0.2):
                continue
            for edge in sorted(set(matching_edges)):
                candidates.append(
                    {
                        "pi": table.get("pi"),
                        "ci": table.get("ci"),
                        "rows": table.get("rows"),
                        "cols": table.get("cols"),
                        "bbox": table_box,
                        "edge": edge,
                        "line_x": x,
                        "line_y0": line_y0,
                        "line_y1": line_y1,
                        "stroke_width": stroke_width,
                        "visible_width_ratio": visible_width_ratio,
                        "clip_ids": line["clip_ids"],
                        "clip_rect": clip,
                    }
                )

    return sorted(
        candidates,
        key=lambda candidate: (
            float(candidate["bbox"][1]),
            float(candidate["bbox"][0]),
            str(candidate["edge"]),
            float(candidate["line_y0"]),
        ),
    )


def _horizontal_line_matches_table_record(
    line: Mapping[str, object],
    table: Mapping[str, object],
) -> bool:
    """Require an SVG line to correspond to a direct render-tree table line."""
    expected_lines = table["own_horizontal_lines"]
    assert isinstance(expected_lines, list)
    for expected_x0, expected_x1, expected_y, expected_height in expected_lines:
        tolerance = max(0.25, float(expected_height) / 2.0 + 0.2)
        if (
            abs(float(line["x0"]) - float(expected_x0)) <= tolerance
            and abs(float(line["x1"]) - float(expected_x1)) <= tolerance
            and abs(float(line["y"]) - float(expected_y)) <= tolerance
        ):
            return True
    return False


def _table_has_paint_safe_horizontal_frame(
    table: Mapping[str, object],
    all_lines: Sequence[Mapping[str, object]],
    clip: tuple[float, float, float, float],
    edge: str,
) -> bool:
    """Return whether the table has a direct frame stroke wholly inside `clip`.

    Continued tables retain their original off-page border as a source node
    after a correct physical frame has been reconstructed.  Treating that
    hidden source node as a live defect would make the ledger permanently
    noisy, so a paint-safe direct sibling at the same page edge resolves it.
    """
    for line in all_lines:
        line_clip = line["clip"]
        if not isinstance(line_clip, tuple) or not _horizontal_line_matches_table_record(line, table):
            continue
        paint_top = float(line["paint_top"])
        paint_bottom = float(line["paint_bottom"])
        if edge == "top":
            if (
                paint_top >= clip[1] - 0.01
                and float(line["y"]) - clip[1] <= 6.0
                and paint_bottom <= clip[3] + 0.01
            ):
                return True
        elif (
            paint_bottom <= clip[3] + 0.01
            and clip[3] - float(line["y"]) <= 6.0
            and paint_top >= clip[1] - 0.01
        ):
            return True
    return False


def svg_table_horizontal_border_clip_candidates(
    svg_path: Path,
    tree: Mapping[str, object],
) -> list[dict[str, object]]:
    """Find table frame strokes that a vertical clip removes or halves.

    The vertical-border ledger intentionally cannot see page-top/page-bottom
    frame losses.  This companion checks direct Table horizontal `Line`s near
    an effective clip's top or bottom and reports only strokes with <80% of
    their height remaining and no paint-safe sibling frame.  A six-pixel
    boundary band captures native HWP border rounding while excluding old
    off-page source lines and tiny residual fragments.
    """
    all_lines = svg_horizontal_lines_with_clips(svg_path)
    candidates: list[dict[str, object]] = []

    def append_candidate(candidate: dict[str, object]) -> None:
        """Keep one candidate per table edge and effective clip."""
        key = (
            tuple(candidate["bbox"]),
            str(candidate["edge"]),
            tuple(candidate["clip_ids"]),
            tuple(candidate["clip_rect"]),
        )
        for existing in candidates:
            existing_key = (
                tuple(existing["bbox"]),
                str(existing["edge"]),
                tuple(existing["clip_ids"]),
                tuple(existing["clip_rect"]),
            )
            if existing_key == key:
                return
        candidates.append(candidate)

    for line in all_lines:
        clip = line["clip"]
        if not isinstance(clip, tuple):
            continue
        paint_top = float(line["paint_top"])
        paint_bottom = float(line["paint_bottom"])
        visible_height = max(0.0, min(paint_bottom, clip[3]) - max(paint_top, clip[1]))
        visible_height_ratio = visible_height / max(paint_bottom - paint_top, 0.01)
        if visible_height_ratio >= 0.8:
            continue

        y = float(line["y"])
        top_distance = abs(y - clip[1])
        bottom_distance = abs(y - clip[3])
        if min(top_distance, bottom_distance) > 6.0:
            continue
        edge = "top" if top_distance <= bottom_distance else "bottom"

        for table in table_records(tree):
            table_box = table["bbox"]
            assert isinstance(table_box, tuple)
            table_x, table_y, table_width, table_height = table_box
            fragment_height = max(
                0.0,
                min(table_y + table_height, clip[3]) - max(table_y, clip[1]),
            )
            if fragment_height < 6.0 or not _horizontal_line_matches_table_record(line, table):
                continue
            if _table_has_paint_safe_horizontal_frame(table, all_lines, clip, edge):
                continue
            append_candidate(
                {
                    "pi": table.get("pi"),
                    "ci": table.get("ci"),
                    "rows": table.get("rows"),
                    "cols": table.get("cols"),
                    "bbox": table_box,
                    "edge": edge,
                    "line_x0": line["x0"],
                    "line_x1": line["x1"],
                    "line_y": y,
                    "stroke_width": line["stroke_width"],
                    "visible_height_ratio": visible_height_ratio,
                    "clip_ids": line["clip_ids"],
                    "clip_rect": clip,
                }
            )

    # A continuation can also lose its physical frame before any source line
    # approaches the clip edge: p10/p13 keep the table's real bottom border
    # on a later page, so the old near-edge-only pass saw no line to flag.
    # Once a direct-bordered table spans an effective clip, the active
    # physical fragment requires a paint-safe top and/or bottom sibling.
    for table in table_records(tree):
        table_box = table["bbox"]
        assert isinstance(table_box, tuple)
        table_x, table_y, table_width, table_height = table_box
        matching_lines = [
            line
            for line in all_lines
            if isinstance(line["clip"], tuple)
            and _horizontal_line_matches_table_record(line, table)
        ]
        if not matching_lines:
            continue
        for source_line in matching_lines:
            clip = source_line["clip"]
            assert isinstance(clip, tuple)
            fragment_height = max(
                0.0,
                min(table_y + table_height, clip[3]) - max(table_y, clip[1]),
            )
            if fragment_height < 6.0:
                continue
            for edge, continues in (
                ("top", table_y < clip[1] - 0.5),
                ("bottom", table_y + table_height > clip[3] + 0.5),
            ):
                if not continues or _table_has_paint_safe_horizontal_frame(
                    table, all_lines, clip, edge
                ):
                    continue
                frame_y = clip[1] if edge == "top" else clip[3]
                append_candidate(
                    {
                        "pi": table.get("pi"),
                        "ci": table.get("ci"),
                        "rows": table.get("rows"),
                        "cols": table.get("cols"),
                        "bbox": table_box,
                        "edge": edge,
                        "line_x0": table_x,
                        "line_x1": table_x + table_width,
                        "line_y": frame_y,
                        "stroke_width": source_line["stroke_width"],
                        "visible_height_ratio": 0.0,
                        "clip_ids": source_line["clip_ids"],
                        "clip_rect": clip,
                    }
                )

    return sorted(
        candidates,
        key=lambda candidate: (
            float(candidate["bbox"][1]),
            float(candidate["bbox"][0]),
            str(candidate["edge"]),
            float(candidate["line_y"]),
        ),
    )


def write_svg_text_band_clip_ledger(
    work_dir: Path,
    svg_dir: Path,
    requested_pages: Sequence[int],
) -> Path:
    """Write partially clipped visible SVG glyph-band candidates."""
    report_path = work_dir / "svg-text-band-clip-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tx\tbaseline_y\tfont_size\tband_top\tband_bottom\tedges\t"
            "clipped_top_px\tclipped_bottom_px\tvisible_height_ratio\tclip_ids\t"
            "clip_rect\ttext\tnote\n"
        )
        for page_index in requested_pages:
            svg_paths = list(svg_dir.glob(f"*_{page_index + 1:03}.svg"))
            if not svg_paths:
                continue
            try:
                candidates = svg_text_band_clip_candidates(svg_paths[0])
            except (ET.ParseError, OSError, ValueError):
                continue
            for candidate in candidates:
                edges = candidate["edges"]
                clip_ids = candidate["clip_ids"]
                assert isinstance(edges, tuple)
                assert isinstance(clip_ids, tuple)
                text = str(candidate["text"]).replace("\t", " ").replace("\n", " ")
                report.write(
                    f"{page_index + 1}\t{float(candidate['x']):.1f}\t"
                    f"{float(candidate['baseline_y']):.1f}\t"
                    f"{float(candidate['font_size']):.1f}\t"
                    f"{float(candidate['band_top']):.1f}\t"
                    f"{float(candidate['band_bottom']):.1f}\t{','.join(edges)}\t"
                    f"{float(candidate['clipped_top_px']):.1f}\t"
                    f"{float(candidate['clipped_bottom_px']):.1f}\t"
                    f"{float(candidate['visible_height_ratio']):.3f}\t"
                    f"{','.join(clip_ids)}\t{format_bbox(candidate['clip_rect'])}\t"
                    f"{text}\tcandidate only; visible glyph band is partially cut by SVG clip\n"
                )
    return report_path


def write_svg_table_border_clip_ledger(
    work_dir: Path,
    svg_dir: Path,
    tree_dir: Path,
    requested_pages: Sequence[int],
) -> None:
    """Write structural candidates for table outer strokes hidden by SVG clips."""
    report_path = work_dir / "svg-table-border-clip-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tpi\tci\trows\tcols\tedge\ttable_bbox\tline_x\tline_y0\tline_y1\t"
            "stroke_width\tvisible_width_ratio\tclip_ids\tclip_rect\tnote\n"
        )
        for page_index in requested_pages:
            svg_paths = list(svg_dir.glob(f"*_{page_index + 1:03}.svg"))
            tree_path = tree_path_for_page(tree_dir, page_index)
            if not svg_paths or tree_path is None:
                report.write(
                    f"{page_index + 1}\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t"
                    "SVG 또는 render tree 없음\n"
                )
                continue
            try:
                tree = json.loads(tree_path.read_text(encoding="utf-8"))
                if not isinstance(tree, Mapping):
                    raise ValueError("render tree root가 object가 아님")
                candidates = svg_table_border_clip_candidates(svg_paths[0], tree)
            except (ET.ParseError, OSError, ValueError, json.JSONDecodeError) as error:
                report.write(
                    f"{page_index + 1}\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t"
                    f"SVG/tree 읽기 실패: {error}\n"
                )
                continue
            if not candidates:
                report.write(
                    f"{page_index + 1}\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t"
                    "-\n"
                )
                continue
            for candidate in candidates:
                clip_ids = candidate["clip_ids"]
                assert isinstance(clip_ids, tuple)
                report.write(
                    f"{page_index + 1}\t{format_number(candidate['pi'])}\t"
                    f"{format_number(candidate['ci'])}\t{format_number(candidate['rows'])}\t"
                    f"{format_number(candidate['cols'])}\t{candidate['edge']}\t"
                    f"{format_bbox(candidate['bbox'])}\t{float(candidate['line_x']):.1f}\t"
                    f"{float(candidate['line_y0']):.1f}\t{float(candidate['line_y1']):.1f}\t"
                    f"{float(candidate['stroke_width']):.1f}\t"
                    f"{float(candidate['visible_width_ratio']):.3f}\t"
                    f"{','.join(clip_ids) or '-'}\t{format_bbox(candidate['clip_rect'])}\t"
                    "candidate only; outer table stroke is emitted but hidden by SVG clip\n"
                )


def write_svg_table_horizontal_border_clip_ledger(
    work_dir: Path,
    svg_dir: Path,
    tree_dir: Path,
    requested_pages: Sequence[int],
) -> None:
    """Write candidates for clipped table top/bottom frames.

    Kept separate from the vertical ledger so callers can distinguish a lost
    right/left table edge from a page-break frame that is only half painted.
    """
    report_path = work_dir / "svg-table-horizontal-border-clip-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tpi\tci\trows\tcols\tedge\ttable_bbox\tline_x0\tline_x1\t"
            "line_y\tstroke_width\tvisible_height_ratio\tclip_ids\tclip_rect\tnote\n"
        )
        for page_index in requested_pages:
            svg_paths = list(svg_dir.glob(f"*_{page_index + 1:03}.svg"))
            tree_path = tree_path_for_page(tree_dir, page_index)
            if not svg_paths or tree_path is None:
                report.write(
                    f"{page_index + 1}\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t"
                    "SVG 또는 render tree 없음\n"
                )
                continue
            try:
                tree = json.loads(tree_path.read_text(encoding="utf-8"))
                if not isinstance(tree, Mapping):
                    raise ValueError("render tree root가 object가 아님")
                candidates = svg_table_horizontal_border_clip_candidates(svg_paths[0], tree)
            except (ET.ParseError, OSError, ValueError, json.JSONDecodeError) as error:
                report.write(
                    f"{page_index + 1}\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t"
                    f"SVG/tree 읽기 실패: {error}\n"
                )
                continue
            if not candidates:
                report.write(
                    f"{page_index + 1}\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\n"
                )
                continue
            for candidate in candidates:
                clip_ids = candidate["clip_ids"]
                assert isinstance(clip_ids, tuple)
                report.write(
                    f"{page_index + 1}\t{format_number(candidate['pi'])}\t"
                    f"{format_number(candidate['ci'])}\t{format_number(candidate['rows'])}\t"
                    f"{format_number(candidate['cols'])}\t{candidate['edge']}\t"
                    f"{format_bbox(candidate['bbox'])}\t{float(candidate['line_x0']):.1f}\t"
                    f"{float(candidate['line_x1']):.1f}\t{float(candidate['line_y']):.1f}\t"
                    f"{float(candidate['stroke_width']):.1f}\t"
                    f"{float(candidate['visible_height_ratio']):.3f}\t"
                    f"{','.join(clip_ids) or '-'}\t{format_bbox(candidate['clip_rect'])}\t"
                    "candidate only; table top/bottom frame stroke is clipped at a page boundary\n"
                )


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


def write_page_boundary_fidelity_ledger(
    work_dir: Path,
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
    page_text_layers: Mapping[int, tuple[str, str]],
    *,
    tree_dir: Path | None = None,
    requested_pages: Sequence[int] = (),
) -> Path:
    """Write one actionable review row per adjacent-page owner boundary."""
    report_path = work_dir / "page-boundary-fidelity-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tnext_page\tkind\tdirection\tcounter_chars\tsequence_chars\t"
            "sequence\ttable_fragments\tnote\n"
        )
        for candidate in page_boundary_fidelity_candidates(
            page_differences,
            page_text_layers,
            tree_dir=tree_dir,
            requested_pages=requested_pages,
        ):
            fragments = candidate["table_fragments"]
            assert isinstance(fragments, list)
            fragment_summary = ";".join(
                "pi={pi},ci={ci},rows={rows},cols={cols},signals={signals}".format(
                    pi=format_number(fragment.get("pi")),
                    ci=format_number(fragment.get("ci")),
                    rows=format_number(fragment.get("rows")),
                    cols=format_number(fragment.get("cols")),
                    signals="|".join(str(signal) for signal in fragment["signals"]),
                )
                for fragment in fragments
            ) or "-"
            sequence = str(candidate["sequence"]).replace("\t", " ").replace("\n", " ")
            report.write(
                f"{int(candidate['page']) + 1}\t{int(candidate['next_page']) + 1}\t"
                f"{candidate['kind']}\t{candidate['direction']}\t"
                f"{candidate['counter_chars']}\t{candidate['sequence_chars']}\t"
                f"{sequence or '-'}\t{fragment_summary}\t"
                "candidate only; PDF visual owner review required\n"
            )
    return report_path


def write_visible_text_excess_ledger(
    work_dir: Path,
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
    clip_excluded_chars: Mapping[int, int],
) -> None:
    """Write visible-only owner candidates without changing the raw text ledger."""
    report_path = work_dir / "visible-text-excess-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\treference_only\tvisible_svg_only\tclip_excluded_chars\tnote\n"
        )
        for candidate in visible_text_excess_candidates(
            page_differences, clip_excluded_chars
        ):
            report.write(
                f"{int(candidate['page']) + 1}\t{candidate['reference_only']}\t"
                f"{candidate['visible_svg_only']}\t{candidate['clip_excluded_chars']}\t"
                "candidate only; PDF text is preserved but visible rhwp text is substantially extra "
                "(possible early/duplicate page owner)\n"
            )


def write_successor_float_owner_shift_ledger(
    work_dir: Path,
    tree_dir: Path,
    requested_pages: Sequence[int],
    page_differences: Mapping[int, tuple[Counter[str], Counter[str]]],
) -> None:
    """Write high-signal owner shifts whose successor starts with a Body float."""
    report_path = work_dir / "float-owner-shift-candidates.tsv"
    with report_path.open("w", encoding="utf-8") as report:
        report.write(
            "page\tnext_page\tdirection\tshared_chars\tsource_coverage\t"
            "target_coverage\tpi\tci\ttext_wrap\tbbox\ttop_ratio\tnote\n"
        )
        for candidate in successor_float_owner_shift_candidates(
            tree_dir, requested_pages, page_differences
        ):
            float_record = candidate["float"]
            assert isinstance(float_record, Mapping)
            report.write(
                f"{int(candidate['page']) + 1}\t{int(candidate['next_page']) + 1}\t"
                f"{candidate['direction']}\t{candidate['shared_count']}\t"
                f"{float(candidate['source_coverage']):.3f}\t"
                f"{float(candidate['target_coverage']):.3f}\t"
                f"{format_number(float_record.get('pi'))}\t"
                f"{format_number(float_record.get('ci'))}\t"
                f"{float_record.get('text_wrap')}\t"
                f"{format_bbox(float_record.get('bbox'))}\t"
                f"{float(float_record['top_ratio']):.3f}\t"
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
    chrome_env = chrome_fontconfig_environment(work_dir)
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
    visible_text_differences: dict[int, tuple[Counter[str], Counter[str]]] = {}
    clip_excluded_text_chars: dict[int, int] = {}
    glyph_risks: dict[int, Counter[str]] = {}
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
            rendered_text = svg_text(svg_path)
            visible_rendered_text, clip_excluded_chars = svg_visible_text(svg_path)
            clip_excluded_text_chars[page_index] = clip_excluded_chars
            glyph_risks[page_index] = svg_glyph_risks(rendered_text)
        except Exception as error:  # noqa: BLE001 - glyph ledger도 SVG 파싱 실패를 남긴다.
            text_rows.append((page_index, 0, 0, "", "", f"SVG 텍스트층 추출 실패: {error}"))
        else:
            try:
                reference_text = reference_text_for_page(page_index)
                missing, extra = compare_text_layers(reference_text, rendered_text)
                text_differences[page_index] = (missing, extra)
                text_layers[page_index] = (reference_text, rendered_text)
                visible_text_differences[page_index] = compare_text_layers(
                    reference_text, visible_rendered_text
                )
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
            except Exception as error:  # noqa: BLE001 - PDF text 실패가 glyph 후보를 덮어쓰지 않는다.
                text_rows.append((page_index, 0, 0, "", "", f"기준 PDF 텍스트층 추출 실패: {error}"))

        if args.text_only:
            completed_pages.append(page_index)
            continue

        rendered_png = work_dir / f"r{page_index:03}.png"
        reference_png = work_dir / f"g{page_index:03}.png"
        comparison_png = work_dir / f"cmp-p{page_index:03}.png"
        assert chrome is not None
        assert pdf is not None
        page = pdf[page_index]
        svg_ok = svg_to_png(svg_path, rendered_png, chrome, chrome_env)
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
            chrome_env,
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
    boundary_fidelity_report_path = write_page_boundary_fidelity_ledger(
        work_dir,
        text_differences,
        text_layers,
        tree_dir=tree_dir if args.layout_ledger else None,
        requested_pages=requested_pages,
    )
    write_visible_text_excess_ledger(
        work_dir, visible_text_differences, clip_excluded_text_chars
    )
    glyph_risk_report_path = write_svg_glyph_risk_report(
        work_dir, glyph_risks, requested_pages
    )
    if args.layout_ledger:
        write_table_fragment_ledger(
            work_dir,
            tree_dir,
            requested_pages,
            text_differences,
        )
        write_table_cell_text_overlap_ledger(
            work_dir,
            tree_dir,
            requested_pages,
        )
        write_table_cell_text_boundary_ledger(
            work_dir,
            tree_dir,
            requested_pages,
        )
        write_svg_text_band_clip_ledger(
            work_dir,
            svg_dir,
            requested_pages,
        )
        write_svg_table_border_clip_ledger(
            work_dir,
            svg_dir,
            tree_dir,
            requested_pages,
        )
        write_svg_table_horizontal_border_clip_ledger(
            work_dir,
            svg_dir,
            tree_dir,
            requested_pages,
        )
        write_successor_float_owner_shift_ledger(
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
    print("SVG glyph-risk candidates:", glyph_risk_report_path)
    print("text owner-shift candidates:", work_dir / "text-owner-shift-candidates.tsv")
    print("text owner-sequence candidates:", work_dir / "text-owner-sequence-candidates.tsv")
    print("page-boundary fidelity candidates:", boundary_fidelity_report_path)
    print("visible text-excess candidates:", work_dir / "visible-text-excess-candidates.tsv")
    print("page-count ledger:", work_dir / "page-count-ledger.tsv")
    if args.layout_ledger:
        print("layout ledger:", work_dir / "layout-candidates.tsv")
        print("table fragment candidates:", work_dir / "table-fragment-candidates.tsv")
        print("table cell text-overlap candidates:", work_dir / "table-cell-text-overlap-candidates.tsv")
        print(
            "table cell text-boundary candidates:",
            work_dir / "table-cell-text-boundary-candidates.tsv",
        )
        print(
            "SVG text-band clip candidates:",
            work_dir / "svg-text-band-clip-candidates.tsv",
        )
        print(
            "SVG table-border clip candidates:",
            work_dir / "svg-table-border-clip-candidates.tsv",
        )
        print(
            "SVG table-horizontal-border clip candidates:",
            work_dir / "svg-table-horizontal-border-clip-candidates.tsv",
        )
        print(
            "float owner-shift candidates:",
            work_dir / "float-owner-shift-candidates.tsv",
        )
    print("run state:", work_dir / "run-state.tsv")
    return 0 if not all_missing_pages else 1


if __name__ == "__main__":
    raise SystemExit(main())
