"""본문(list 0) 문단들의 한글 줄끝 마크 워크."""
from __future__ import annotations

import sys
import time
import subprocess as sp
from pathlib import Path

from pyhwpx import Hwp

sys.stdout.reconfigure(encoding="utf-8")
src = Path(sys.argv[1]).resolve()
pis = [int(x) for x in sys.argv[2].split(",")]

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
    for tgt in pis:
        if not hwp.SetPos(0, tgt, 0):
            print(f"pi={tgt}: SetPos 실패")
            continue
        hwp.MoveSelParaEnd()
        text = hwp.get_selected_text()
        hwp.Cancel()
        marks = []
        pos = 0
        for _ in range(40):
            hwp.SetPos(0, tgt, pos)
            hwp.MoveLineEnd()
            l2, p2, e2 = hwp.GetPos()
            if l2 != 0 or p2 != tgt:
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
        print(f"pi={tgt} len={len(text)} lines={len(marks)} marks={marks} t={text[:20]!r}")
finally:
    hwp.quit()
