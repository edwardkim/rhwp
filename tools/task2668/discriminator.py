# -*- coding: utf-8 -*-
"""#2668 판별자 검정 — 회귀군 vs 개선군의 '꼬리말 위치 쪽 번호' 유무.

가설: 회귀군 = 꼬리말 밴드에 쪽 번호가 찍히는 구역(밴드 점유) → 회수하면 안 됨.
      개선군 = 밴드가 진짜 비어 있음 → 회수해야 함.
RHWP_DIAG_FBAND 로 구역별 판정을 직접 읽는다.
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

RE_D = re.compile(r"DIAG_FBAND sec=(\d+) footer_ctrl=(\w+) pnum_pos=(\S+) "
                  r"footer_pnum=(\w+) reclaim=(\w+)")


def resolve(pre):
    if pre == "ISSUE1733":
        return ISSUE1733
    return PATHS.get(pre) or next((v for k, v in PATHS.items() if k.startswith(pre)), None)


def main():
    env = dict(os.environ, RHWP_DIAG_FBAND="1")
    print(f"{'grp':4} {'doc':22} {'구역':>4} {'꼬리말':>6} {'쪽번호위치':>10} "
          f"{'꼬리말쪽번호':>12} {'밴드회수':>8}")
    for grp, lst in [("REG", REG), ("IMP", IMP)]:
        for pre in lst:
            g = resolve(pre)
            if not g or not os.path.exists(g):
                print(f"{grp:4} {pre:22} MISSING")
                continue
            try:
                p = subprocess.run([EXE, "dump-pages", g, "-p", "1"],
                                   capture_output=True, text=True, encoding="utf-8",
                                   errors="replace", timeout=1200, env=env)
            except subprocess.TimeoutExpired:
                print(f"{grp:4} {pre:22} TIMEOUT")
                continue
            ms = RE_D.findall(p.stderr)
            if not ms:
                print(f"{grp:4} {pre:22} NODIAG")
                continue
            seen = set()
            for sec, fc, pp, fp, rc in ms:
                key = (sec, fc, pp, fp, rc)
                if key in seen:
                    continue
                seen.add(key)
                tag = pre if sec == "0" else ""
                print(f"{grp:4} {tag:22} {sec:>4} {fc:>6} {pp:>10} {fp:>12} {rc:>8}")


if __name__ == "__main__":
    main()
