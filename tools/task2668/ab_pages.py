# -*- coding: utf-8 -*-
"""#2668 A/B — 밴드 회수 ON(현행 devel) vs OFF(RHWP_FB_OFF) 쪽수 vs 한글 정답.

한글 정답 쪽수는 geom_*.json 의 PDF 쪽수(한글 COM 저장 PDF 실측).
밴드 회수가 '물리적으로 맞아서' 이기는지, 아니면 다른 오차를 상쇄하는 fudge 인지
쪽수 수준에서 정량화한다.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
ROOT = Path(r"C:\Users\planet\rhwp")
EXE = str(ROOT / "target/debug/rhwp.exe")
OUT = ROOT / "output/poc/task2668"
REG = {"1130000-200400010", "1170000-200500007", "1192000-202400015",
       "1320000-200900012", "1320000-201400002", "1741000-202000012",
       "1430000-200700032", "ISSUE1733"}
RE_PG = re.compile(r"페이지 수: (\d+)")


def pages(path, off):
    env = dict(os.environ)
    if off:
        env["RHWP_FB_OFF"] = "1"
    else:
        env.pop("RHWP_FB_OFF", None)
    t = subprocess.run([EXE, "info", path], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=1800,
                       env=env).stdout
    m = RE_PG.search(t)
    return int(m.group(1)) if m else None


def main():
    print(f"{'군':4} {'문서':22} {'한글':>5} {'ON':>5} {'OFF':>5} "
          f"{'ΔON':>5} {'ΔOFF':>6}  판정")
    agg = {"ON": 0, "OFF": 0}
    for f in sorted(OUT.glob("geom_*.json")):
        pre = f.stem[5:]
        rec = json.load(open(f, encoding="utf-8"))
        truth = len(rec["rows"])
        grp = "REG" if pre in REG else "IMP"
        on, off = pages(rec["path"], False), pages(rec["path"], True)
        if on is None or off is None:
            print(f"{grp:4} {pre:22} ?")
            continue
        d_on, d_off = on - truth, off - truth
        agg["ON"] += abs(d_on)
        agg["OFF"] += abs(d_off)
        if abs(d_off) < abs(d_on):
            verd = "OFF 우세(회수가 해로움)"
        elif abs(d_off) > abs(d_on):
            verd = "ON 우세(회수가 이로움)"
        else:
            verd = "동률"
        print(f"{grp:4} {pre:22} {truth:5d} {on:5d} {off:5d} "
              f"{d_on:+5d} {d_off:+6d}  {verd}")
    print(f"\n|Δ| 합계: ON={agg['ON']}  OFF={agg['OFF']}")


if __name__ == "__main__":
    main()
