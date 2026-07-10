"""#2150 한글 표 행높이 오라클 v2 — 0열 병합 세그먼트 정렬.

v1(hangul_row_heights.py) 의 TableLowerCell 걷기는 0열을 따라 내려가므로
병합 셀(rowspan>1)이 한 항목으로 잡혀 rhwp per-grid-row cut 과 개수가 어긋난다
(21761835: 한글 31 vs cut 78). v2 는 rhwp dump 의 0열 셀 (row, rowspan, 선언h)
목록으로 세그먼트를 구성해 한글 걷기 순서와 1:1 정렬하고, 세그먼트별로
sum(cut_rows[row..row+rs]) 를 비교한다.

출력: 세그먼트별 한글_px / rhwp_cut_sum / 선언_sum / diff. 전 행 오라클 정렬로
"선언==한글" 가정을 표 전체에서 검증할 수 있다.

사용:
  python tools/hangul_row_heights2.py <file.hwp> [--exe <rhwp>] [--table-index N]
요구: Windows + 한컴 + pyhwpx. rhwp release 바이너리(--exe).
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

MM_TO_PX = 96.0 / 25.4
HU_TO_PX = 96.0 / 7200.0

sys.stdout.reconfigure(encoding="utf-8")


def hangul_col0_heights(src: Path, table_index: int = 0) -> list[float] | None:
    """한글 표 0열 걷기 세그먼트 높이(mm) — v1 과 동일 메커니즘.

    taskkill 직후 CreateObject 레이스(-2146959355 '서버 실행 실패') 대비
    백오프 재시도 (verify_pi_page_vs_hangul.py fresh_hwp 선례).
    """
    import time
    import subprocess as sp

    from pyhwpx import Hwp

    sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    time.sleep(2)
    hwp = None
    for attempt in range(5):
        try:
            hwp = Hwp(new=True, visible=False)
            break
        except Exception:
            time.sleep(2 + attempt * 2)
    if hwp is None:
        return None
    hwp.open(str(src))
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


def col0_segments(src: Path, exe: str, table_index: int = 0) -> list[tuple[int, int, float]]:
    """rhwp dump 에서 0열 셀 (row, rowspan, 선언높이px) — dump 순서(행 오름차순)."""
    r = subprocess.run([exe, "dump", str(src)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=180)
    segs: list[tuple[int, int, float]] = []
    ti = -1
    for line in r.stdout.splitlines():
        if "표:" in line and "행×" in line:
            ti += 1
        if ti != table_index:
            continue
        m = re.search(r"셀\[\d+\] r=(\d+),c=(\d+) rs=(\d+),cs=\d+ h=(\d+)", line)
        if m and int(m.group(2)) == 0:
            segs.append((int(m.group(1)), int(m.group(3)), int(m.group(4)) * HU_TO_PX))
    segs.sort(key=lambda s: s[0])
    return segs


def rhwp_cut_rows(src: Path, exe: str) -> list[float] | None:
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

    hg_mm = hangul_col0_heights(a.src, a.table_index)
    if hg_mm is None:
        print("한글 표 추출 실패", file=sys.stderr)
        return 2
    hg_px = [h * MM_TO_PX for h in hg_mm]
    segs = col0_segments(a.src, a.exe, a.table_index)
    cut = rhwp_cut_rows(a.src, a.exe)
    if cut is None:
        print("rhwp cut_rows 추출 실패", file=sys.stderr)
        return 2
    print(f"한글 세그먼트={len(hg_px)}  0열 셀={len(segs)}  cut rows={len(cut)}")
    print(f"{'seg':>4} {'rows':>8} {'한글_px':>9} {'cut합':>9} {'선언합':>9} "
          f"{'cutΔ한글':>9} {'선언Δ한글':>9}")
    tot_c = tot_d = 0.0
    n = min(len(hg_px), len(segs))
    for i in range(n):
        row, rs, decl = segs[i]
        csum = sum(cut[row:row + rs]) if row + rs <= len(cut) else float("nan")
        h = hg_px[i]
        dc = csum - h
        dd = decl - h
        tot_c += dc if dc == dc else 0.0
        tot_d += dd
        mark = "  <<<" if abs(dc) >= 1.0 else ""
        print(f"{i:>4} {f'{row}+{rs}':>8} {h:>9.1f} {csum:>9.1f} {decl:>9.1f} "
              f"{dc:>+9.1f} {dd:>+9.1f}{mark}")
    if len(hg_px) != len(segs):
        print(f"주의: 세그먼트 개수 불일치 (한글 {len(hg_px)} vs 0열 {len(segs)})")
    print(f"누적: cutΔ한글={tot_c:+.1f}px  선언Δ한글={tot_d:+.1f}px")
    return 0


if __name__ == "__main__":
    sys.exit(main())
