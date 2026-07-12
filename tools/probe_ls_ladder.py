#!/usr/bin/env python
"""#2150 통제 프로브 - ls% 사다리 HWPX 생성 (A:1줄 / B:장문 다중줄 / C:2문단).

선언 셀높이 극소(288HU)의 1열 표에 ls% 사다리 문단을 넣는다. 한글이 열면
행높이 = 한글 fresh 콘텐츠 높이 + 패딩 -> tools/probe_ls_ladder.py 로 직독.

확정 공식 (2026-07-10): 한글 NO_LS fresh 셀 콘텐츠 높이 =
sum 줄박스(em=max_fs) + 연속 줄 사이 gap (ls-100%)x fs (문단 경계 포함),
셀 마지막 줄 뒤 trailing 없음. 1줄 셀은 ls% 완전 무시(em).
"""
import sys
import time
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

SRC = Path(r"C:\Users\planet\rhwp\output\poc\task2150\ls_ladder3.hwpx")
MM_TO_PX = 96.0 / 25.4
PAD_PX = 2 * 141 / 7200 * 96
FS_PX = 13.333
LS = [100, 130, 160, 200, 230, 300]
SERIES = ["A(1줄)", "B(장문)", "C(2문단)"]


def main() -> int:
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
        print("COM 기동 실패", file=sys.stderr)
        return 2
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
            print("셀 진입 실패", file=sys.stderr)
            return 2
        n = int(hwp.get_row_num())
        hwp.Cancel()
        heights = []
        for _ in range(n):
            heights.append(float(hwp.get_row_height()))
            if not hwp.TableLowerCell():
                break
        hwp.clear(option=1)
    finally:
        try:
            hwp.quit()
        except Exception:
            pass
        sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)

    print(f"행수={len(heights)}")
    print(f"{'series':>9} {'ls%':>5} {'row_px':>8} {'inner_px':>9} {'inner/fs':>9}")
    for i, mm in enumerate(heights):
        s = SERIES[i // len(LS)] if i // len(LS) < len(SERIES) else "?"
        ls = LS[i % len(LS)]
        row_px = mm * MM_TO_PX
        inner = row_px - PAD_PX
        print(f"{s:>9} {ls:>5} {row_px:>8.2f} {inner:>9.2f} {inner / FS_PX:>9.3f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
