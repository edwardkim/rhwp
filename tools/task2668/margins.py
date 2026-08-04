# -*- coding: utf-8 -*-
"""#2668: 회귀/개선군의 구역 여백 실측.

가설: footer_band_reclaim() 이 반환하는 layout.footer_area.height 는
model/page.rs 정의상 **margin_bottom** 이다(footer_area = [content_bottom,
page_h - margin_footer]). 실제 빈 꼬리말 밴드 높이는 **margin_footer** 다.
margin_footer == 0 인 문서는 회수할 밴드가 없는데도 margin_bottom 만큼
회수되어 본문이 과적재된다 → 회귀군.
"""
import os
import re
import subprocess
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
ROOT = Path(r"C:\Users\planet\rhwp")
EXE = str(ROOT / "target/debug/rhwp.exe")

PATHS = {}
for _l in open(ROOT / "output/poc/survey10k_r19_20260721/sample10000.txt", encoding="utf-8"):
    _l = _l.strip()
    if _l:
        PATHS[Path(_l).name] = _l

REG = ["1130000-200400010", "1170000-200500007", "1192000-202400015",
       "1320000-200900012", "1320000-201400002", "1741000-202000012",
       "1430000-200700032", "ISSUE1733"]
IMP = ["1290000-201800044", "1312000-201300049", "1420000-201600010",
       "1430000-201300008", "1430000-202000001", "1490000-200700072",
       "1490000-201900079", "156241785", "6460000-202500170", "B552462-201300011"]
ISSUE1733 = str(ROOT / "samples/task1725/text_footnote_tail_overpagination.hwp")

RE_PAPER = re.compile(r"구역(\d+) 용지: (\d+)×(\d+) HWPUNIT.*?여백: 좌(\d+) 우(\d+) 상(\d+) 하(\d+)\)")
RE_HF = re.compile(r"머리말여백=(\d+) 꼬리말여백=(\d+) 제본여백=(\d+)")
HU2PX = 96.0 / 7200.0


def resolve(pre):
    if pre == "ISSUE1733":
        return ISSUE1733
    return PATHS.get(pre) or next((v for k, v in PATHS.items() if k.startswith(pre)), None)


def main():
    print(f"{'group':5} {'doc':22} {'m_bottom':>9} {'m_footer':>9} {'band_px':>8} {'reclaim_px':>10}")
    for grp, lst in [("REG", REG), ("IMP", IMP)]:
        for pre in lst:
            g = resolve(pre)
            if not g or not os.path.exists(g):
                print(f"{grp:5} {pre:22} MISSING")
                continue
            t = subprocess.run([EXE, "info", g], capture_output=True, text=True,
                               encoding="utf-8", errors="replace", timeout=300).stdout
            secs = RE_PAPER.findall(t)
            hfs = RE_HF.findall(t)
            if not secs:
                print(f"{grp:5} {pre:22} NOPAPER")
                continue
            for i, s in enumerate(secs):
                mb = int(s[6])
                mf = int(hfs[i][1]) if i < len(hfs) else -1
                tag = pre if i == 0 else f"  ↳sec{i}"
                print(f"{grp:5} {tag:22} {mb:9d} {mf:9d} "
                      f"{mf * HU2PX:8.1f} {mb * HU2PX:10.1f}")


if __name__ == "__main__":
    main()
