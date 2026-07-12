"""hy_ladder4: 5행 P21 변형 캐럿 마크 워크."""
import sys
import time
import subprocess as sp
from pathlib import Path

from pyhwpx import Hwp

sys.stdout.reconfigure(encoding="utf-8")
src = Path("output/poc/task2070/hy_ladder4.hwpx").resolve()
labels = [l.split("\t")[1] for l in
          open("output/poc/task2070/hy_ladder4_labels.txt", encoding="utf-8")
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
    for i, lab in enumerate(labels):
        if not hwp.move_to_field(f"R{i}"):
            print(f"{lab}: move실패")
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
        print(f"{lab}: marks={marks}")
    print("실문서 oracle: [22, 39, 58, 75, 95, 105]")
finally:
    hwp.quit()
