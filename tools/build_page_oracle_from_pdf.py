"""저장소 안 `pdf/` 한글 정답지로 render_page_gate 픽스처를 만든다.

`tools/build_page_oracle.py` 는 한글 COM(`pyhwpx`)으로 정답지를 **수집**한다 —
Windows + 한컴오피스가 있어야 하고, 대상도 저장소 밖 코퍼스다. 그래서 기여자와 CI 는
`tools/render_page_gate.py` 를 돌릴 수 없다(기존 픽스처의 `rel` 이
`C:/Users/planet/hwpdocs` 기준이다).

이 스크립트는 **이미 저장소에 커밋된 자산만** 쓴다 — `samples/` 문서와 그 짝인
`pdf/` 한글 출력. 정답지 쪽수를 한 번 떠서 TSV 로 굳혀 두면, 게이트 자체는 아무 의존성
없이 누구나 돌릴 수 있다(기존 `render_page_oracle_1658.tsv` 와 같은 분업이다 —
생성기만 의존성을 지고 픽스처는 값만 담는다).

짝짓기 규칙: `pdf/<이름>-<연도>[-<변종>].pdf` → `samples/**/<이름>.hwp|.hwpx`.
한 이름에 여러 정답지가 있으면 먼저 만난 하나만 쓰고 나머지는 건너뛴다.

사용:
    python tools/build_page_oracle_from_pdf.py --exe target/release-test/rhwp.exe \\
        -o tests/fixtures/render_page_samples.tsv

이후 게이트:
    python tools/render_page_gate.py --root . \\
        --fixture tests/fixtures/render_page_samples.tsv --exe target/release-test/rhwp.exe

요구: `pypdfium2` (생성 시에만). 산출 TSV 를 읽는 쪽은 의존성이 없다.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

PAGES_RE = re.compile(r"페이지 수:\s*(\d+)")
# pdf 파일명 꼬리의 판본 표시를 벗긴다 — `-2022`, `-20260814`, `-2010-kopub`, `-2020-no-ttf`.
VARIANT_RE = re.compile(r"-((?:19|20)\d{2})(\d{4})?(-[A-Za-z0-9\-]+)?$")
# `-p025-p035` 처럼 **쪽 구간만 뽑은 발췌본**은 전체 문서 정답지가 아니다.
# (실측: 이 발췌본은 11쪽인데 원본은 415쪽 — 짝지으면 델타가 +402 로 튄다.)
EXCERPT_RE = re.compile(r"-p\d+-p\d+", re.I)


def git_pdf_paths(rev: str) -> list[str]:
    out = subprocess.run(
        ["git", "ls-tree", "-r", rev, "pdf/", "--name-only"],
        capture_output=True,
        check=True,
    ).stdout.decode("utf-8", "replace")
    paths = []
    for line in out.splitlines():
        p = line.strip().strip('"')
        if p.lower().endswith(".pdf"):
            paths.append(p)
    return paths


def sample_index(root: Path) -> dict[str, str]:
    """`samples/` 문서를 확장자 뗀 이름으로 색인한다. 같은 이름이면 경로가 짧은 쪽."""
    index: dict[str, str] = {}
    base = root / "samples"
    for dirpath, _dirnames, filenames in os.walk(base):
        for name in filenames:
            if not name.lower().endswith((".hwp", ".hwpx")):
                continue
            stem = os.path.splitext(name)[0]
            rel = os.path.relpath(os.path.join(dirpath, name), root).replace("\\", "/")
            prev = index.get(stem)
            if prev is None or rel.count("/") < prev.count("/"):
                index[stem] = rel
    return index


def pdf_page_count(rev: str, path: str, scratch: Path) -> int | None:
    """정답지 PDF 의 쪽수. 순수 stdlib 파싱은 이 저장소 PDF 대부분(객체 스트림)에서
    실패하므로 pypdfium2 로 읽는다 — 생성 시점에만 필요하다."""
    import pypdfium2 as pdfium  # 지연 import: 이 함수를 안 쓰면 의존성도 없다

    blob = subprocess.run(
        ["git", "show", f"{rev}:{path}"], capture_output=True
    ).stdout
    if not blob:
        return None
    tmp = scratch / "oracle.pdf"
    tmp.write_bytes(blob)
    try:
        doc = pdfium.PdfDocument(str(tmp))
        try:
            return len(doc)
        finally:
            doc.close()
    except Exception:
        return None


def resolve_exe(exe: str) -> str:
    """Windows 의 `subprocess` 는 `a/b/c.exe` 같은 정규화 안 된 상대 경로를 못 찾는다
    (파일이 있어도 WinError 2). 실제 경로로 굳혀서 넘긴다."""
    p = Path(exe)
    if p.exists():
        return str(p.resolve())
    return exe  # PATH 에 있는 이름이면 그대로


def rhwp_page_count(exe: str, doc: Path, timeout: int) -> int | None:
    try:
        out = subprocess.run(
            [exe, "info", str(doc)], capture_output=True, timeout=timeout
        ).stdout.decode("utf-8", "replace")
    except Exception as err:  # noqa: BLE001 — 원인을 삼키지 않고 알린다
        print(f"    rhwp info 예외: {type(err).__name__}: {err}", file=sys.stderr)
        return None
    m = PAGES_RE.search(out)
    return int(m.group(1)) if m else None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path("."))
    ap.add_argument("--rev", default="HEAD", help="pdf/ 를 읽을 Git ref")
    ap.add_argument("--exe", default="target/release-test/rhwp.exe")
    ap.add_argument("--timeout", type=int, default=300)
    ap.add_argument("-o", "--out", type=Path, required=True)
    args = ap.parse_args()

    root = args.root.resolve()
    exe = resolve_exe(args.exe)
    samples = sample_index(root)
    pdfs = git_pdf_paths(args.rev)

    # stem 하나에 정답지가 여럿이면 **가장 담백한 판본**을 고른다 —
    # 연도 뒤 꼬리표(`-kopub`·`-no-ttf`·`-print`)가 없는 쪽, 그다음 최신 연도.
    # 꼬리표가 붙은 것도 전체 문서이긴 하나(폰트·렌더 변종), 기준선은 하나로 고정한다.
    best: dict[str, tuple[tuple[int, int], str]] = {}
    for pdf in pdfs:
        base = os.path.splitext(os.path.basename(pdf))[0]
        if EXCERPT_RE.search(base):
            continue
        m = VARIANT_RE.search(base)
        stem = base[: m.start()] if m else base
        if stem not in samples:
            continue
        year = int(m.group(1)) if m else 0
        plain = 0 if (m and m.group(3)) else 1  # 꼬리표 없으면 우선
        rank = (plain, year)
        prev = best.get(stem)
        if prev is None or rank > prev[0]:
            best[stem] = (rank, pdf)

    pairs = sorted((samples[stem], pdf) for stem, (_rank, pdf) in best.items())

    rows = []
    with tempfile.TemporaryDirectory() as td:
        scratch = Path(td)
        for rel, pdf in pairs:
            oracle = pdf_page_count(args.rev, pdf, scratch)
            if oracle is None:
                print(f"  건너뜀(정답지 못 읽음): {pdf}", file=sys.stderr)
                continue
            mine = rhwp_page_count(exe, root / rel, args.timeout)
            if mine is None:
                print(f"  건너뜀(rhwp info 실패): {rel}", file=sys.stderr)
                continue
            rows.append((rel, oracle, mine, mine - oracle))

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8", newline="\n") as f:
        f.write("rel\thangul_pages\trhwp_pages_baseline\tdelta\n")
        for rel, oracle, mine, delta in rows:
            f.write(f"{rel}\t{oracle}\t{mine}\t{delta}\n")

    off = [r for r in rows if r[3] != 0]
    print(f"짝 {len(rows)}건 기록 → {args.out}")
    print(f"쪽수 불일치 {len(off)}건")
    for rel, oracle, mine, delta in sorted(off, key=lambda r: -abs(r[3]))[:20]:
        print(f"  rhwp {mine:4d} vs 한글 {oracle:4d} ({delta:+d})  {rel}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
