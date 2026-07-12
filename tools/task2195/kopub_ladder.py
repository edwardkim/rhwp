"""폰트별 12pt/ls150% 3줄 문단 사다리 -> PDF -> 줄피치 실측용 문서 생성."""
import sys, time, subprocess as sp
from pathlib import Path
from pyhwpx import Hwp
sys.stdout.reconfigure(encoding="utf-8")
out = Path(sys.argv[1]).resolve()
FONTS = ["KoPub돋움체 Light", "KoPub돋움체 Medium", "함초롬바탕", "한양신명조", "휴먼명조", "맑은 고딕"]
TXT = "가나다라마바사아자차카타파하 " * 12  # 12pt에서 3줄 이상
sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
time.sleep(2)
hwp = Hwp(new=True, visible=False)
try:
    for i, f in enumerate(FONTS):
        hwp.insert_text(f"F{i}마커시작")
        hwp.BreakPara()
        hwp.set_font(FaceName=f, Height=12)
        # 줄간격 150%
        pset = hwp.HParameterSet.HParaShape
        hwp.HAction.GetDefault("ParagraphShape", pset.HSet)
        pset.LineSpacingType = hwp.LineSpacingMethod("Percent")
        pset.LineSpacing = 150
        hwp.HAction.Execute("ParagraphShape", pset.HSet)
        hwp.insert_text(TXT)
        hwp.BreakPara()
        hwp.set_font(FaceName="함초롬바탕", Height=10)
        hwp.HAction.GetDefault("ParagraphShape", pset.HSet)
        pset.LineSpacingType = hwp.LineSpacingMethod("Percent")
        pset.LineSpacing = 150
        hwp.HAction.Execute("ParagraphShape", pset.HSet)
    hwp.save_as(str(out), format="PDF")
    print("saved", out)
finally:
    hwp.quit()
    sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
