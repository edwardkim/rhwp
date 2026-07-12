#!/usr/bin/env python
"""#2156 문자폭 프로브 실행 — 한글 COM + rhwp 행높이에서 유효 문자폭 역산.

ls=100% 에서 행높이 = n x em + pad (양쪽 모델 일치) -> 줄수 n 역산 ->
w ∈ [(n-1) x inner/k, n x inner/k). k 사다리 구간 교집합으로 클래스별
유효 폭을 좁히고 한글/rhwp 를 대조한다.
"""
import csv
import os
import re
import subprocess
import sys
import time
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(r"C:\Users\planet\rhwp")
SRC = ROOT / "output" / "poc" / "task2156" / "width_ladder.hwpx"
MANIFEST = ROOT / "output" / "poc" / "task2156" / "width_ladder_manifest.tsv"
EXE = ROOT / "target" / "release" / "rhwp.exe"

MM_TO_PX = 96.0 / 25.4
PAD_V_PX = 2 * 141 / 7200 * 96  # 상하 패딩 3.76
PAD_H_PX = 2 * 141 / 7200 * 96  # 좌우 패딩
CELL_W_PX = 40000 / 7200 * 96   # 533.3
INNER_PX = CELL_W_PX - PAD_H_PX
EM_PX = 13.333  # 10pt


def hangul_rows() -> list[float] | None:
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
    try:
        hwp.open(str(SRC))
        ctrl = hwp.HeadCtrl
        tbl = None
        while ctrl is not None:
            if ctrl.CtrlID == "tbl":
                tbl = ctrl
                break
            ctrl = ctrl.Next
        hwp.SetPosBySet(tbl.GetAnchorPos(0))
        hwp.FindCtrl()
        if not hwp.ShapeObjTableSelCell():
            return None
        n = int(hwp.get_row_num())
        hwp.Cancel()
        heights = []
        for _ in range(n):
            heights.append(float(hwp.get_row_height()) * MM_TO_PX)
            if not hwp.TableLowerCell():
                break
        hwp.clear(option=1)
        return heights
    finally:
        try:
            hwp.quit()
        except Exception:
            pass
        sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)


def rhwp_rows() -> list[float] | None:
    env = dict(os.environ, RHWP_TABLE_DRIFT="1")
    r = subprocess.run([str(EXE), "dump-pages", str(SRC)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", env=env,
                       timeout=300)
    m = re.search(r"cut_rows=\[([^\]]*)\]", r.stdout + r.stderr)
    if not m:
        return None
    return [float(x) for x in m.group(1).split(",") if x.strip()]


def lines_from_height(h_px: float, includes_pad: bool) -> int:
    inner = h_px - (PAD_V_PX if includes_pad else 0.0)
    return max(1, round(inner / EM_PX))


def main() -> int:
    rows = []
    with open(MANIFEST, encoding="utf-8") as f:
        for rec in csv.DictReader(f, delimiter="\t"):
            rows.append((int(rec["row"]), rec["class"], int(rec["k"])))

    hg = hangul_rows()
    if hg is None:
        print("한글 COM 실패", file=sys.stderr)
        return 2
    rh = rhwp_rows()

    print(f"inner={INNER_PX:.2f}px  em={EM_PX}px  한글행={len(hg)}  rhwp행={len(rh) if rh else 'n/a'}")
    print(f"{'class':>8} {'k':>5} {'한글n':>5} {'rhwp n':>6} {'한글 w구간(px)':>22} {'rhwp w구간(px)':>22}")

    # 클래스별 구간 교집합
    bounds: dict[str, dict[str, list[float]]] = {}
    for row, cname, k in rows:
        h_n = lines_from_height(hg[row], True) if row < len(hg) else None
        r_n = lines_from_height(rh[row], True) if rh and row < len(rh) else None
        h_iv = ((h_n - 1) * INNER_PX / k, h_n * INNER_PX / k) if h_n else None
        r_iv = ((r_n - 1) * INNER_PX / k, r_n * INNER_PX / k) if r_n else None
        b = bounds.setdefault(cname, {"h": [0.0, 1e9], "r": [0.0, 1e9]})
        if h_iv:
            b["h"][0] = max(b["h"][0], h_iv[0])
            b["h"][1] = min(b["h"][1], h_iv[1])
        if r_iv:
            b["r"][0] = max(b["r"][0], r_iv[0])
            b["r"][1] = min(b["r"][1], r_iv[1])
        print(f"{cname:>8} {k:>5} {h_n if h_n else '-':>5} {r_n if r_n else '-':>6} "
              f"{f'[{h_iv[0]:.3f},{h_iv[1]:.3f})' if h_iv else '-':>22} "
              f"{f'[{r_iv[0]:.3f},{r_iv[1]:.3f})' if r_iv else '-':>22}")

    print("\n=== 클래스별 유효 폭 (구간 교집합 중앙) ===")
    print(f"{'class':>8} {'한글 w':>10} {'rhwp w':>10} {'차(r-h)':>9} {'비율':>7}")
    for cname, b in bounds.items():
        hw = (b["h"][0] + b["h"][1]) / 2 if b["h"][1] < 1e9 else float("nan")
        rw = (b["r"][0] + b["r"][1]) / 2 if b["r"][1] < 1e9 else float("nan")
        print(f"{cname:>8} {hw:>10.3f} {rw:>10.3f} {rw - hw:>+9.3f} {rw / hw:>7.3f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
