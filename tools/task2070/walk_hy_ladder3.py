"""hy_ladder3: 행높이(줄수) + P21/P27 셀 캐럿 마크 워크 (단일 한글 세션)."""
import sys
import time
import subprocess as sp
from pathlib import Path

from pyhwpx import Hwp

MM_TO_PX = 96.0 / 25.4
EM = 1400 * 96.0 / 7200.0
PAD = 2 * 141 * 96.0 / 7200.0

sys.stdout.reconfigure(encoding="utf-8")
src = Path("output/poc/task2070/hy_ladder3.hwpx").resolve()
labels = [l.split("\t")[1] for l in
          open("output/poc/task2070/hy_ladder3_labels.txt", encoding="utf-8")
          .read().splitlines()]

sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
time.sleep(2)
hwp = None
for attempt in range(5):
    try:
        hwp = Hwp(visible=False)
        break
    except Exception:
        time.sleep(3)
if hwp is None:
    sys.exit("한글 기동 실패")
try:
    hwp.open(str(src))
    # 1) 행높이: 셀명 점프로 각 행
    for i, lab in enumerate(labels):
        if not hwp.move_to_field(f"R{i}"):
            print(f"{i:2d} {lab:22s} move실패")
            continue
        h = float(hwp.get_row_height()) * MM_TO_PX
        print(f"{i:2d} {lab:22s} h={h:6.1f}px lines={round((h - PAD) / EM)}")
    # 2) P21/P27 셀 캐럿 마크
    for i in range(6):
        if not hwp.move_to_field(f"R{i}"):
            continue
        lst = hwp.GetPos()[0]
        hwp.SetPos(lst, 0, 0)
        hwp.MoveSelParaEnd()
        text = hwp.get_selected_text()
        hwp.Cancel()
        marks = []
        pos = 0
        for _ in range(30):
            hwp.SetPos(lst, 0, pos)
            hwp.MoveLineEnd()
            l2, p2, e2 = hwp.GetPos()
            if l2 != lst or p2 != 0:
                break
            if marks and e2 <= marks[-1]:
                pos = marks[-1] + 1
                if pos > len(text) + 2:
                    break
                continue
            marks.append(e2)
            pos = e2
            if e2 >= len(text):
                break
        print(f"{labels[i]}: marks={marks}")
finally:
    hwp.quit()
