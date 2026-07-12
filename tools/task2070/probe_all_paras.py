"""개정안{{1}} 셀 전 문단 per-para 줄끝 마크 (원본 .hwp 기준)."""
from __future__ import annotations

import sys
import time
import subprocess as sp
from pathlib import Path

from pyhwpx import Hwp

sys.stdout.reconfigure(encoding="utf-8")
src = Path(sys.argv[1] if len(sys.argv) > 1 else "samples/80168_regulatory_analysis.hwp").resolve()
field = sys.argv[2] if len(sys.argv) > 2 else "개정안{{1}}"

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
    if not hwp.move_to_field(field):
        sys.exit("move 실패")
    lst = hwp.GetPos()[0]
    for tgt in range(60):
        # 문단 존재 확인 + 전문
        if not hwp.SetPos(lst, tgt, 0):
            break
        cur = hwp.GetPos()
        if cur[0] != lst or cur[1] != tgt:
            break
        hwp.MoveSelParaEnd()
        text = hwp.get_selected_text()
        hwp.Cancel()
        marks = []
        pos = 0
        for _ in range(40):
            hwp.SetPos(lst, tgt, pos)
            hwp.MoveLineEnd()
            l2, p2, e2 = hwp.GetPos()
            if l2 != lst or p2 != tgt:
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
        print(f"pidx={tgt}\tlen={len(text)}\tlines={len(marks)}\tmarks={marks}\ttext={text[:24]!r}")
finally:
    hwp.quit()
