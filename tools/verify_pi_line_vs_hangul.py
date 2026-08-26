# -*- coding: utf-8 -*-
"""드리프트 후보의 (쪽, 줄) COM 교차검증 — 백로그 확정/기각 도구.

`verify_ladder_drift.py`(PR #4526)가 플래그한 문단을 실제 한글에 물어 확정한다:
SetPos(0, para, 0) 뒤 `current_page`(쪽)와 `KeyIndicator` line(쪽 내 줄 번호)을 읽어
rhwp 렌더 트리의 (쪽, 본문 흐름 줄 서수)와 나란히 놓는다. 줄 번호의 절대 의미는 양쪽이
다를 수 있으므로(표 줄 계산 등) **요청 문단 사이의 상대 델타**로 판정하는 것을 권장한다.

주의: COM 규약(동시 실행 금지·문서당 열기 1회) — 배치 하니스와 같은 시간에 돌리지 말 것.
다구역 문서는 para 번호가 구역 누적이라 rhwp pi 와 다를 수 있다(v1 은 단일 구역 권장).

사용:
  python tools/verify_pi_line_vs_hangul.py <file.hwp> --pi 16,17 [--exe rhwp.exe]
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8", errors="backslashreplace")
sys.path.insert(0, str(Path(__file__).resolve().parent))

import verify_ladder_drift as drift  # 본문 흐름 줄 파서 재사용

REPO = Path(__file__).resolve().parents[1]


def rhwp_lines(exe: Path, path: Path):
    """pi → (쪽 번호, 쪽 내 본문 흐름 줄 서수 0-기반, 렌더 y)."""
    proc = subprocess.run(
        [str(exe), "dump-extents", str(path)], capture_output=True, timeout=180, check=False
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.decode("utf-8", "replace")[:200])
    pages = []
    stack = []
    for raw in proc.stdout.decode("utf-8", "replace").splitlines():
        if drift.PAGE_RE.match(raw):
            pages.append([])
            stack = []
            continue
        if not pages:
            continue
        node = drift.NODE_RE.match(raw)
        if not node:
            continue
        indent, ntype = len(node.group(1)), node.group(2)
        while stack and stack[-1][0] >= indent:
            stack.pop()
        m = drift.LINE_RE.match(raw)
        if m and not any(t in drift.FOREIGN for _, t in stack):
            h = float(m.group(3))
            if h > 0.5:
                pages[-1].append((float(m.group(2)), int(m.group(4)), int(m.group(5))))
        stack.append((indent, ntype))
    out = {}
    for pno, rows in enumerate(pages, 1):
        for ordinal, (y, pi, line) in enumerate(sorted(rows)):
            if line == 0 and pi not in out:
                out[pi] = (pno, ordinal, y)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("target")
    ap.add_argument("--pi", required=True, help="쉼표로 구분한 문단 번호")
    ap.add_argument("--exe", default=str(REPO / "target" / "debug" / "rhwp.exe"))
    args = ap.parse_args()

    pis = [int(x) for x in args.pi.split(",")]
    r = rhwp_lines(Path(args.exe), Path(args.target))

    from pyhwpx import Hwp

    hwp = Hwp(new=True, visible=False)
    rows = []
    try:
        hwp.open(str(Path(args.target).resolve()))
        for pi in pis:
            hwp.SetPos(0, pi, 0)
            page = hwp.current_page
            ki = hwp.KeyIndicator()
            # (성공, seccnt, secno, prnpageno, colno, line, pos, over, ctrlname)
            line = ki[5] if isinstance(ki, (list, tuple)) and len(ki) > 5 else None
            rows.append((pi, page, line))
    finally:
        try:
            hwp.quit()
        except Exception:  # noqa: BLE001
            pass

    print("pi\thwp(page,line)\trhwp(page,ordinal,y)")
    for pi, page, line in rows:
        rv = r.get(pi)
        rtxt = f"p{rv[0]} ord{rv[1]} y={rv[2]:.1f}" if rv else "-"
        print(f"{pi}\tp{page} line{line}\t{rtxt}")
    if len(rows) >= 2:
        print("-- 상대 델타(연속 쌍) --")
        for (pa, qa, la), (pb, qb, lb) in zip(rows, rows[1:]):
            ra, rb = r.get(pa), r.get(pb)
            hd = (lb - la) if (la is not None and lb is not None and qa == qb) else None
            rd = (rb[1] - ra[1]) if (ra and rb and ra[0] == rb[0]) else None
            verdict = "?" if hd is None or rd is None else ("일치" if hd == rd else f"어긋남 hwp={hd} rhwp={rd}")
            print(f"pi{pa}->pi{pb}: {verdict}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
