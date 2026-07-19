# -*- coding: utf-8 -*-
"""#2237/#2291 — 표별 쪽-행 범위 복원: 한글 PDF 행 앵커 vs rhwp 조각.

각 표(섹션/pi)의 행별 앵커 텍스트를 IR 에서 뽑아 한글 PDF 에서 **고유 매치**
(문서 전체 1회)만 채택해 쪽별 행 범위를 복원하고, rhwp dump-pages 의
PartialTable rows 와 대조한다. 목차·동형 반복 텍스트 오염은 고유성 검증으로
배제된다.

usage: python tools/task2287/row_anchor_map.py <hwp> <han_pdf> <sec:pi> [<sec:pi> ...]
       [--exe target/debug/rhwp.exe]
예: python tools/task2287/row_anchor_map.py samples/task2287/1342000_edu_curriculum_map.hwp \
      pdf/task2287/1342000_edu_curriculum_map-2022.pdf 2:0 2:1 3:0
(정답지 PDF 리포 동봉 — 한글 2022 COM 전체 415쪽, producer=Hancom PDF)
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

import fitz

sys.stdout.reconfigure(encoding="utf-8")


def default_exe() -> str:
    name = "rhwp.exe" if os.name == "nt" else "rhwp"
    return str(Path("target") / "debug" / name)


CELL_RE = re.compile(r'r=(\d+),c=(\d+) rs=\d+,cs=\d+ .* text="([^"]{6,})')


def table_anchors(exe: str, hwp: str, sec: int, pi: int) -> dict:
    out = subprocess.run(
        [exe, "dump", hwp, "-s", str(sec), "-p", str(pi)],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=300,
    )
    anchors = {}
    for ln in out.stdout.splitlines():
        m = CELL_RE.search(ln)
        if m:
            r, txt = int(m.group(1)), m.group(3)
            key = txt.replace(" ", "")[:14]
            if r not in anchors and len(key) >= 8:
                anchors[r] = key
    return anchors


def rhwp_fragments(exe: str, hwp: str) -> dict:
    """(sec, pi) -> [(page, row_start, row_end)]"""
    out = subprocess.run(
        [exe, "dump-pages", hwp],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900,
    )
    frags = {}
    cur = sec = None
    for ln in out.stdout.splitlines():
        m = re.match(r"=== 페이지 (\d+) \(global_idx=\d+, section=(\d+)", ln)
        if m:
            cur, sec = int(m.group(1)), int(m.group(2))
            continue
        m2 = re.search(r"PartialTable   pi=(\d+) ci=\d+  rows=(\d+)\.\.(\d+)", ln)
        if m2 and cur is not None:
            frags.setdefault((sec, int(m2.group(1))), []).append(
                (cur, int(m2.group(2)), int(m2.group(3)))
            )
        m3 = re.search(r"^\s*Table          pi=(\d+) ci=\d+  (\d+)x\d+", ln)
        if m3 and cur is not None:
            key = (sec, int(m3.group(1)))
            frags.setdefault(key, []).append((cur, 0, int(m3.group(2))))
    return frags


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("hwp")
    ap.add_argument("han_pdf")
    ap.add_argument("tables", nargs="+", help="sec:pi ...")
    ap.add_argument("--exe", default=default_exe())
    a = ap.parse_args()

    doc = fitz.open(a.han_pdf)
    page_texts = [doc[p].get_text().replace(" ", "").replace("\n", "") for p in range(doc.page_count)]
    frags = rhwp_fragments(a.exe, a.hwp)

    for spec in a.tables:
        sec, pi = (int(x) for x in spec.split(":"))
        anchors = table_anchors(a.exe, a.hwp, sec, pi)
        fr = frags.get((sec, pi), [])
        if not fr:
            print(f"== s{sec} pi{pi}: rhwp 조각 없음 ==")
            continue
        rh_pages = sorted({p for p, _, _ in fr})
        print(f"== s{sec} pi{pi} (rhwp p{rh_pages[0]}~p{rh_pages[-1]}, {len(rh_pages)}쪽) ==")
        # 앵커 고유 매치
        han_row_page = {}
        for r, key in sorted(anchors.items()):
            hits = [p + 1 for p, t in enumerate(page_texts) if key in t]
            if len(hits) == 1:
                han_row_page[r] = hits[0]
        if not han_row_page:
            print("  고유 앵커 없음")
            continue
        han_pages = sorted(set(han_row_page.values()))
        print(f"  한글: p{han_pages[0]}~p{han_pages[-1]} ({len(han_pages)}쪽 관측, 고유앵커 {len(han_row_page)}행)")
        # 쪽별 행수 비교표
        print("  row rhwp쪽 한글쪽")
        for r in sorted(han_row_page):
            rp = next((p for p, s0, e0 in fr if s0 <= r < e0), None)
            print(f"  {r:3d} {rp} {han_row_page[r]}")


if __name__ == "__main__":
    main()
