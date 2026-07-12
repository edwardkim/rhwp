import sys, time, subprocess as sp
from pathlib import Path
from pyhwpx import Hwp
sys.stdout.reconfigure(encoding="utf-8")
src = Path(sys.argv[1]).resolve(); out = Path(sys.argv[2]).resolve()
sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
time.sleep(2)
hwp = Hwp(new=True, visible=False)
try:
    hwp.open(str(src))
    hwp.save_as(str(out), format="PDF")
    print("saved", out)
finally:
    hwp.quit()
    sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
