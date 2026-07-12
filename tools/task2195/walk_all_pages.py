"""한글 전체 본문 pi -> current_page 덤프."""
import sys, time, subprocess as sp
from pathlib import Path
from pyhwpx import Hwp
sys.stdout.reconfigure(encoding="utf-8")
src = Path(sys.argv[1]).resolve()
out = Path(sys.argv[2])
sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
time.sleep(2)
hwp = None
for _ in range(5):
    try:
        hwp = Hwp(visible=False); break
    except Exception: time.sleep(3)
if hwp is None: sys.exit("한글 기동 실패")
try:
    hwp.open(str(src))
    rows = []
    pi = 0
    while hwp.SetPos(0, pi, 0):
        rows.append(f"{pi}\t{hwp.current_page}")
        pi += 1
        if pi > 5000: break
    out.write_text("\n".join(rows), encoding="utf-8")
    print(f"walked {pi} paras -> {out}")
finally:
    hwp.quit()
