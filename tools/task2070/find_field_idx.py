"""개정안{{i}} 순차 점검 — 각 셀 첫 문단 머리 텍스트 출력."""
import sys
import time
import subprocess as sp
from pathlib import Path

from pyhwpx import Hwp

sys.stdout.reconfigure(encoding="utf-8")
src = Path("samples/80168_regulatory_analysis.hwp").resolve()

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
    for i in range(14):
        if not hwp.move_to_field(f"개정안{{{{{i}}}}}"):
            print(f"{i}: 없음")
            break
        lst = hwp.GetPos()[0]
        hwp.SetPos(lst, 0, 0)
        hwp.MoveSelParaEnd()
        t = hwp.get_selected_text()
        hwp.Cancel()
        print(f"{i}: {t[:26]!r}")
finally:
    hwp.quit()
