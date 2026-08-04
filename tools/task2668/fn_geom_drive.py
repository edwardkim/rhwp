# -*- coding: utf-8 -*-
"""#2668 실측 드라이버 — 워커를 문서당 별도 프로세스로 실행(COM 크래시/행 격리)."""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
ROOT = Path(r"C:\Users\planet\rhwp")
OUT = ROOT / "output/poc/task2668"
WORKER = str(OUT / "fn_geom_worker.py")
TIMEOUT = 900

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


def resolve(pre):
    if pre == "ISSUE1733":
        return ISSUE1733
    return PATHS.get(pre) or next((v for k, v in PATHS.items() if k.startswith(pre)), None)


def summarize(pre, grp, rec):
    rows = rec["rows"]
    fn = [r for r in rows if r.get("fn") and "over" in r]
    npn = sum(1 for r in rows if "pnum_bot" in r)
    if not fn:
        print(f"{grp} {pre}: rhwp={rec['rhwp_pages']} pdf={len(rows)} 각주쪽=0 "
              f"쪽번호쪽={npn}", flush=True)
        return
    ov = sorted(r["over"] for r in fn)
    band = sum(1 for o in ov if o > 1.0)
    ex = fn[len(fn) // 2]
    print(f"{grp} {pre}: rhwp={rec['rhwp_pages']} pdf={len(rows)} 각주쪽={len(fn)} "
          f"쪽번호쪽={npn} over중앙={ov[len(ov)//2]:+.1f} min={ov[0]:+.1f} "
          f"max={ov[-1]:+.1f} 밴드사용={band}/{len(fn)} "
          f"| p{ex['page']} bot='{ex.get('bot_txt','')}' sz={ex.get('fn_sz',0):.1f}",
          flush=True)


def main():
    for grp, lst in [("REG", REG), ("IMP", IMP)]:
        for pre in lst:
            g = resolve(pre)
            if not g or not os.path.exists(g):
                print(f"?MISSING {grp} {pre}", flush=True)
                continue
            dst = OUT / f"geom_{pre}.json"
            # COM 은 CO_E_SERVER_EXEC_FAILURE 로 간헐 실패한다 — 재시도 + 냉각.
            for attempt in range(3):
                if dst.exists():
                    break
                try:
                    p = subprocess.run([sys.executable, WORKER, g, str(dst)],
                                       capture_output=True, text=True,
                                       encoding="utf-8", errors="replace",
                                       timeout=TIMEOUT)
                except subprocess.TimeoutExpired:
                    print(f"?STALL {grp} {pre} (try{attempt})", flush=True)
                    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe", "/T"],
                                   capture_output=True)
                    time.sleep(20)
                    continue
                if not dst.exists():
                    print(f"?ERR {grp} {pre} (try{attempt}): "
                          f"{(p.stderr or '').strip()[-160:]}", flush=True)
                    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe", "/T"],
                                   capture_output=True)
                    time.sleep(20)
            if not dst.exists():
                continue
            summarize(pre, grp, json.load(open(dst, encoding="utf-8")))


if __name__ == "__main__":
    main()
