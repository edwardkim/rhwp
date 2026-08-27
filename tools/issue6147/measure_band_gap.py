"""[#6147] 자리차지 밴드 아래 첫 본문 문단의 세로 위치를 rhwp 와 한글 PDF 로 견준다.

밴드 하단 탐지 휴리스틱은 SVG 와 PDF 에서 서로 다르게 걸린다. 그래서 대신 **첫 본문
문단의 글자 내용을 키로** 양쪽에서 같은 줄을 찾아 그 baseline 을 견준다.

    python tools/issue6147/measure_band_gap.py <문서> [--oracle <한글 PDF>]

한글 PDF 는 `tools/hwp_oracle_pdf.ps1` 로 만든다. 문서 `settings.xml` 의
`PrintMethod` 가 4/6 이면 한글이 모아찍기로 내보내 정답지가 오염되므로, 사본에서 0 으로
낮춰 다시 뜬 PDF 를 써야 한다(156765780 실측).
"""

from __future__ import annotations

import argparse
import glob
import io
import os
import re
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PX_PER_PT = 96.0 / 72.0


def rhwp_exe() -> str:
    for profile in ("release", "debug"):
        path = os.path.join(REPO, "target", profile, "rhwp.exe")
        if os.path.exists(path):
            return path
    sys.exit("rhwp 바이너리를 찾지 못했다 — cargo build --bin rhwp")


def body_key(path: str) -> str | None:
    """dump-pages 1쪽에서 밴드 뒤 첫 FullParagraph 의 글자 앞머리."""
    out = subprocess.run(
        [rhwp_exe(), "dump-pages", path, "-p", "0"], capture_output=True
    ).stdout.decode("utf-8", "replace")
    seen_table = False
    for line in out.splitlines():
        if line.strip().startswith("Table"):
            seen_table = True
            continue
        matched = re.search(r'FullParagraph\s+pi=\d+.*?"(.+)"\s*$', line)
        if seen_table and matched:
            text = re.sub(r"\s+", "", matched.group(1))
            if len(text) >= 6:
                return text[:6]
    return None


def rhwp_baseline(path: str, key: str) -> float | None:
    out = tempfile.mkdtemp(prefix="issue6147-")
    subprocess.run(
        [rhwp_exe(), "export-svg", path, "-p", "0", "-o", out], capture_output=True
    )
    files = sorted(glob.glob(os.path.join(out, "*.svg")))
    if not files:
        return None
    svg = io.open(files[0], encoding="utf-8").read()
    lines: dict[float, list[str]] = {}
    for run in re.finditer(r'<text[^>]*\by="([-\d.]+)"[^>]*>(.*?)</text>', svg, re.S):
        lines.setdefault(round(float(run.group(1)), 2), []).append(
            re.sub(r"<[^>]*>", "", run.group(2))
        )
    for y in sorted(lines):
        if key in re.sub(r"\s+", "", "".join(lines[y])):
            return y
    return None


def oracle_baseline(pdf: str, key: str) -> float | None:
    import fitz  # PyMuPDF

    doc = fitz.open(pdf)
    producer = doc.metadata.get("producer") or ""
    if "Hancom" not in producer:
        doc.close()
        sys.exit(f"한컴 PDF 가 아니다 (producer={producer!r}) — 정답지로 쓸 수 없다")
    page = doc[0]
    if page.rect.width > page.rect.height:
        doc.close()
        sys.exit("정답지가 가로 방향이다 — settings.xml PrintMethod 모아찍기 오염을 먼저 걷어라")
    lines: dict[float, list[tuple[float, str]]] = {}
    for block in page.get_text("dict")["blocks"]:
        for line in block.get("lines", []):
            for span in line["spans"]:
                lines.setdefault(round(span["bbox"][3] * PX_PER_PT, 1), []).append(
                    (span["bbox"][0], span["text"])
                )
    doc.close()
    for base in sorted(lines):
        joined = re.sub(r"\s+", "", "".join(text for _, text in sorted(lines[base])))
        if key in joined:
            return base
    return None


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("document")
    parser.add_argument("--oracle", help="한글이 내보낸 PDF")
    args = parser.parse_args()

    key = body_key(args.document)
    if not key:
        sys.exit("밴드 뒤 첫 본문 문단을 찾지 못했다")
    rhwp = rhwp_baseline(args.document, key)
    print(f'키="{key}"  rhwp baseline={rhwp}')
    if args.oracle:
        oracle = oracle_baseline(args.oracle, key)
        print(f"한글 baseline={oracle}")
        if rhwp is not None and oracle is not None:
            print(f"편차={rhwp - oracle:+.1f}px")


main()
