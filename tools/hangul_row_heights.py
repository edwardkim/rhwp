"""#1658 한글 표 행높이 추출(COM) + rhwp cut_row_h 대조.

cut↔render↔한글 fidelity 의 **한글 행높이 기준**을 COM 으로 직접 추출한다(PDF 배율 차단 우회).
핵심: 캐럿이 표 밖이면 셀 진입 실패 → HeadCtrl 순회로 표(tbl) 찾아 anchor 진입.

방법:
  HeadCtrl 순회 → CtrlID=="tbl" → GetAnchorPos(0) → SetPosBySet → FindCtrl →
  ShapeObjTableSelCell → get_row_num/get_row_height(mm) → TableLowerCell 행 순회.
한글 row_h(mm) × 96/25.4 = px 로 rhwp cut_row_h(px, RHWP_TABLE_DRIFT) 와 비교.

사용:
  python tools/hangul_row_heights.py <file.hwp> [--exe <rhwp>] [--table-index N]
요구: Windows + 한컴 + pyhwpx. rhwp release 바이너리(--exe).
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

MM_TO_PX = 96.0 / 25.4


def hangul_row_heights(src: Path, table_index: int = 0) -> list[float] | None:
    """한글 표(table_index 번째 tbl)의 per-row 높이(mm)."""
    import subprocess as sp

    from pyhwpx import Hwp

    sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    hwp = Hwp(new=True, visible=False)
    hwp.open(str(src))
    # HeadCtrl 순회로 table_index 번째 tbl 찾기
    ctrl = hwp.HeadCtrl
    seen = 0
    tbl = None
    while ctrl is not None:
        if ctrl.CtrlID == "tbl":
            if seen == table_index:
                tbl = ctrl
                break
            seen += 1
        ctrl = ctrl.Next
    if tbl is None:
        hwp.quit()
        sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
        return None
    hwp.SetPosBySet(tbl.GetAnchorPos(0))
    hwp.FindCtrl()
    if not hwp.ShapeObjTableSelCell():
        hwp.quit()
        sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
        return None
    n = int(hwp.get_row_num())
    hwp.Cancel()
    heights: list[float] = []
    for _ in range(n):
        heights.append(float(hwp.get_row_height()))
        if not hwp.TableLowerCell():
            break
    hwp.clear(option=1)
    hwp.quit()
    sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    return heights


def rhwp_cut_rows(src: Path, exe: str) -> list[float] | None:
    """RHWP_TABLE_DRIFT 의 cut_rows(px) 첫 표."""
    import os

    env = dict(os.environ, RHWP_TABLE_DRIFT="1")
    r = subprocess.run([exe, "dump-pages", str(src)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", env=env, timeout=180)
    m = re.search(r"cut_rows=\[([^\]]*)\]", r.stdout + r.stderr)
    if not m:
        return None
    return [float(x) for x in m.group(1).split(",") if x.strip()]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("src", type=Path)
    ap.add_argument("--exe", default="C:/Users/planet/rhwp/target/release/rhwp.exe"
                    if sys.platform == "win32" else "target/release/rhwp")
    ap.add_argument("--table-index", type=int, default=0)
    a = ap.parse_args()

    hg_mm = hangul_row_heights(a.src, a.table_index)
    if hg_mm is None:
        print("한글 표 추출 실패", file=sys.stderr)
        return 2
    hg_px = [round(h * MM_TO_PX, 1) for h in hg_mm]
    rh = rhwp_cut_rows(a.src, a.exe)
    print(f"한글 행수={len(hg_px)}  rhwp cut_rows={len(rh) if rh else 'n/a'}")
    if rh is None:
        for i, h in enumerate(hg_px):
            print(f"  r{i}: 한글={h}px")
        return 0
    print(f"{'row':>4} {'한글_px':>9} {'rhwp_px':>9} {'diff':>8}")
    n = max(len(hg_px), len(rh))
    tot = 0.0
    for i in range(n):
        h = hg_px[i] if i < len(hg_px) else None
        r = rh[i] if i < len(rh) else None
        d = (r - h) if (h is not None and r is not None) else None
        if d is not None:
            tot += d
        print(f"{i:>4} {(f'{h:.1f}' if h is not None else '-'):>9} "
              f"{(f'{r:.1f}' if r is not None else '-'):>9} "
              f"{(f'{d:+.1f}' if d is not None else '-'):>8}")
    print(f"누적 diff(rhwp-한글) = {tot:+.1f}px")
    return 0


if __name__ == "__main__":
    sys.exit(main())
