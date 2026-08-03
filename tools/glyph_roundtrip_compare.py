#!/usr/bin/env python
"""#3837 — 왕복(HWP5→HWPX) 전후의 글리프를 좌표까지 대조한다.

`export-hwpx --verify` 의 IR 차이 31% 는 대부분 표현 차이라 렌더에 안 나타난다. 실제로
사용자가 겪는 것만 골라내려면 **렌더 글리프**를 봐야 한다. 문서를 SVG 로 전량 내보낸 뒤
`<text>` 마다 (x, y, font-family, font-size, 글자)를 뽑아 쪽 단위로 대조한다.

사용:
  python tools/glyph_roundtrip_compare.py SRC RT [--exe target/debug/rhwp.exe]
"""
from __future__ import annotations

import argparse
import glob
import os
import re
import subprocess
import sys
import tempfile
from collections import Counter

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

TEXT = re.compile(r"<text\b([^>]*)>(.*?)</text>", re.S)
ATTR = re.compile(r'([a-zA-Z-]+)="([^"]*)"')
PAGENUM = re.compile(r"(\d+)")


def render(exe: str, path: str, out_dir: str) -> dict[int, list[tuple]]:
    os.makedirs(out_dir, exist_ok=True)
    subprocess.run([exe, "export-svg", path, "-o", out_dir], capture_output=True,
                   text=True, encoding="utf-8", errors="replace", timeout=1800)
    # 쪽 키는 **파일 순서**로 만든다. 파일명에서 숫자를 뽑으면 단일 쪽 문서(쪽번호 접미사가
    # 없다)에서 문서 제목 속 숫자를 쪽번호로 오독해, 두 산출물의 쪽이 어긋나 전량 불일치로
    # 보인다(20160897 "[별지 제11호서식]" → 11쪽으로 오독).
    pages: dict[int, list[tuple]] = {}
    files = sorted(glob.glob(os.path.join(out_dir, "*.svg")),
                   key=lambda p: [int(t) if t.isdigit() else t
                                  for t in re.split(r"(\d+)", os.path.basename(p))])
    for pg, f in enumerate(files, start=1):
        svg = open(f, encoding="utf-8", errors="replace").read()
        res = []
        for m in TEXT.finditer(svg):
            a = dict(ATTR.findall(m.group(1)))
            body = re.sub(r"<[^>]*>", "", m.group(2))
            res.append((round(float(a.get("x", 0) or 0), 1),
                        round(float(a.get("y", 0) or 0), 1),
                        a.get("font-family", ""), a.get("font-size", ""), body))
        pages[pg] = res
    return pages


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("src")
    ap.add_argument("rt")
    ap.add_argument("--exe", default="target/debug/rhwp.exe")
    ap.add_argument("--show", type=int, default=8)
    a = ap.parse_args()
    exe = os.path.abspath(a.exe)

    with tempfile.TemporaryDirectory() as td:
        gs = render(exe, a.src, os.path.join(td, "src"))
        gr = render(exe, a.rt, os.path.join(td, "rt"))
    print(f"쪽수  원본 {len(gs)} · 왕복 {len(gr)}")

    total = diff = shown = 0
    kinds = Counter()
    bad_pages = Counter()
    for pg in sorted(set(gs) | set(gr)):
        s, r = gs.get(pg, []), gr.get(pg, [])
        total += max(len(s), len(r))
        for i in range(max(len(s), len(r))):
            x = s[i] if i < len(s) else None
            y = r[i] if i < len(r) else None
            if x == y:
                continue
            diff += 1
            bad_pages[pg] += 1
            if x and y:
                if x[4] != y[4]:
                    kinds["글자"] += 1
                elif x[2] != y[2] or x[3] != y[3]:
                    kinds["폰트"] += 1
                elif x[1] != y[1]:
                    kinds["y"] += 1
                else:
                    kinds["x"] += 1
            else:
                kinds["개수"] += 1
            if shown < a.show:
                print(f"  p{pg} #{i}\n    src {x}\n    rt  {y}")
                shown += 1
    pct = 100.0 * diff / total if total else 0
    print(f"글리프 {total} · 다름 {diff} ({pct:.2f}%)")
    print("종류:", dict(kinds))
    print("어긋난 쪽 상위:", bad_pages.most_common(6))
    return 0


if __name__ == "__main__":
    sys.exit(main())
