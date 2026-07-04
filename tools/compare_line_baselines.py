"""줄 baseline 대조 — rhwp export-pdf vs 한글 PDF 의 실제 글자 기준선(Δ) 시퀀스.

Task #1827 판별 도구. line **bbox top** 비교는 임베드 폰트의 ascender 메트릭 차이
(예: rhwp Palatino 0.732 vs 한컴 서브셋 1.0)로 ±2~3pt 의 가짜 상수 오프셋을 만든다.
실제 잉크 위치는 span **origin(baseline)** 으로 비교해야 한다 — 본 도구는 텍스트
유사도로 줄을 짝지어 Δbaseline 시퀀스·통계·스텝(±1pt 초과 급변)을 출력한다.

사용:
    rhwp export-pdf sample.hwp -o out.pdf
    python tools/compare_line_baselines.py out.pdf reference-2024.pdf [--page N] [--max-abs 2.0]

- --page: 1-기반 (생략 시 공통 페이지 전체 요약)
- --max-abs: |Δbaseline| 중앙값 게이트 (초과 시 exit 1)
의존: pymupdf.
"""
from __future__ import annotations

import argparse
import difflib
import statistics
import sys

import fitz  # pymupdf


def lines(path: str, page: int):
    d = fitz.open(path)
    out = []
    for b in d[page].get_text("dict")["blocks"]:
        for l in b.get("lines", []):
            spans = l.get("spans", [])
            txt = "".join(s["text"] for s in spans).strip()
            if txt and spans:
                out.append((spans[0]["origin"][1], txt))
    out.sort()
    return out


def match_key(t: str) -> str:
    """짝짓기 키 — 공백 제거 후 앞 24자.

    한컴 PDF 는 수식을 PUA(U+E000~U+F8FF) 글리프로 임베드하므로 키에서 제거한다.
    남기면 인접한 유사 줄(예: '종료' vs '시작')과 짝지어져 한 줄 피치만큼의
    가짜 Δbaseline 스텝(−21~−23pt)을 만든다 (#1829 오탐 원인).
    """
    return "".join(ch for ch in t if not "\ue000" <= ch <= "\uf8ff").replace(" ", "")[:24]


def pair_deltas(a, b):
    deltas = []
    bi = 0
    for ya, ta in a:
        ka = match_key(ta)
        if not ka:
            continue
        for j in range(bi, min(bi + 6, len(b))):
            yb, tb = b[j]
            kb = match_key(tb)
            if kb and difflib.SequenceMatcher(None, ka, kb).ratio() > 0.7:
                deltas.append((ya - yb, ya, ta[:24]))
                bi = j + 1
                break
    return deltas


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("rhwp_pdf")
    ap.add_argument("ref_pdf")
    ap.add_argument("--page", type=int, default=None, help="1-기반 페이지")
    ap.add_argument("--max-abs", type=float, default=None, help="|Δ| 중앙값 게이트(pt)")
    a = ap.parse_args()

    ra, rb = fitz.open(a.rhwp_pdf), fitz.open(a.ref_pdf)
    pages = [a.page - 1] if a.page else list(range(min(ra.page_count, rb.page_count)))
    fail = False
    for p in pages:
        ds = pair_deltas(lines(a.rhwp_pdf, p), lines(a.ref_pdf, p))
        if not ds:
            print(f"p{p + 1}: 짝지은 줄 없음")
            continue
        vals = [d for d, _, _ in ds]
        med = statistics.median(vals)
        mark = ""
        if a.max_abs is not None and abs(med) > a.max_abs:
            mark = "  << GATE FAIL"
            fail = True
        print(
            f"p{p + 1}: n={len(ds)} Δbaseline median={med:+.2f}pt "
            f"min={min(vals):+.2f} max={max(vals):+.2f}{mark}"
        )
        prev = None
        for d, y, t in ds:
            if prev is not None and abs(d - prev) > 1.0:
                print(f"    step {prev:+.1f}→{d:+.1f}pt at y={y:.0f} {t!r}")
            prev = d
    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
