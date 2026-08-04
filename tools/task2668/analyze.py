# -*- coding: utf-8 -*-
"""#2668 최종 집계 — '한글이 각주를 빈 꼬리말 밴드에 넣는가' 판정.

판정량: over = fn_bot - body_bot  (각주 블록 바닥 - 본문영역 바닥)
  over <= 0  : 각주가 본문영역 안에 있다 (밴드 미사용)
  over  > 0  : 각주가 밴드로 내려갔다 (현행 #2627 모델의 전제)

오검출 배제(물리 상한): over > margin_bottom(밴드 높이) 인 페이지는
밴드 아래 영역이라 물리적으로 각주일 수 없다 → 구분선 오검출(표 괘선 등)로 제외.
"""
import json
import re
import statistics
import subprocess
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
ROOT = Path(r"C:\Users\planet\rhwp")
EXE = str(ROOT / "target/debug/rhwp.exe")
OUT = ROOT / "output/poc/task2668"
HU2PX = 96.0 / 7200.0
RE_PAPER = re.compile(r"구역(\d+) 용지: (\d+)×(\d+) HWPUNIT.*?여백: 좌(\d+) 우(\d+) 상(\d+) 하(\d+)\)")

REG = {"1130000-200400010", "1170000-200500007", "1192000-202400015",
       "1320000-200900012", "1320000-201400002", "1741000-202000012",
       "1430000-200700032", "ISSUE1733"}


def band_px(path):
    t = subprocess.run([EXE, "info", path], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=300).stdout
    secs = RE_PAPER.findall(t)
    return max((int(s[6]) for s in secs), default=0) * HU2PX


def main():
    print(f"{'군':4} {'문서':22} {'rhwp':>5} {'한글':>5} {'Δ':>4} {'각주쪽':>6} "
          f"{'유효':>5} {'밴드px':>7} {'over중앙':>9} {'over최대':>9} {'밴드사용':>8}")
    tot_valid = tot_band = 0
    for f in sorted(OUT.glob("geom_*.json")):
        pre = f.stem[5:]
        rec = json.load(open(f, encoding="utf-8"))
        grp = "REG" if pre in REG else "IMP"
        rows = [r for r in rec["rows"] if r.get("fn") and "over" in r]
        bp = band_px(rec["path"])
        # 유효성 2중 필터
        #  (a) 물리 상한: over > 밴드 높이 = 밴드 아래 → 각주일 수 없음(구분선 오검출)
        #  (b) 글꼴: 각주는 본문보다 작다. fn_sz(구분선 아래 최대 글꼴)가 11pt 초과면
        #      본문/표 텍스트를 각주로 오인한 페이지다.
        valid = [r for r in rows
                 if r["over"] <= bp + 2.0 and r.get("fn_sz", 99) <= 11.0]
        if not valid:
            print(f"{grp:4} {pre:22} {rec['rhwp_pages']:5d} {len(rec['rows']):5d} "
                  f"{len(rec['rows'])-rec['rhwp_pages']:+4d} {len(rows):6d} "
                  f"{0:5d} {bp:7.1f}  (유효 각주쪽 없음)")
            continue
        ov = sorted(r["over"] for r in valid)
        nb = sum(1 for o in ov if o > 1.0)
        tot_valid += len(valid)
        tot_band += nb
        print(f"{grp:4} {pre:22} {rec['rhwp_pages']:5d} {len(rec['rows']):5d} "
              f"{len(rec['rows'])-rec['rhwp_pages']:+4d} {len(rows):6d} "
              f"{len(valid):5d} {bp:7.1f} {statistics.median(ov):+9.1f} "
              f"{ov[-1]:+9.1f} {nb:4d}/{len(valid):<4d}")
    print(f"\n합계: 유효 각주쪽 {tot_valid} 중 밴드사용 {tot_band}")


if __name__ == "__main__":
    main()
