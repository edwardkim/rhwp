"""앵커 pi 로 표를 찾아 한글 per-row 높이(mm->px)를 걷는다. 인자: file pi [file pi ...]"""
import sys, time, subprocess as sp
from pathlib import Path
from pyhwpx import Hwp
sys.stdout.reconfigure(encoding="utf-8")
args = sys.argv[1:]
jobs = [(Path(args[i]).resolve(), int(args[i+1])) for i in range(0, len(args), 2)]
sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
time.sleep(2)
hwp = Hwp(new=True, visible=False)
try:
    for src, tgt in jobs:
        hwp.open(str(src))
        ctrl = hwp.HeadCtrl
        tbl = None
        while ctrl is not None:
            if ctrl.CtrlID == "tbl":
                pos = ctrl.GetAnchorPos(0)
                para = pos.Item("Para")
                if para == tgt:
                    tbl = ctrl
                    break
            ctrl = ctrl.Next
        if tbl is None:
            print(f"{src.name} pi={tgt}: 표 없음")
            hwp.clear(option=1)
            continue
        hwp.SetPosBySet(tbl.GetAnchorPos(0))
        hwp.FindCtrl()
        if not hwp.ShapeObjTableSelCell():
            print(f"{src.name} pi={tgt}: 셀 진입 실패")
            hwp.clear(option=1)
            continue
        n = int(hwp.get_row_num())
        hwp.Cancel()
        hs = []
        for _ in range(n):
            hs.append(round(float(hwp.get_row_height()) * 96.0 / 25.4, 1))
            if not hwp.TableLowerCell():
                break
        print(f"{src.name} pi={tgt} rows={n} px={hs} sum={round(sum(hs),1)}")
        hwp.clear(option=1)
finally:
    hwp.quit()
    sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
