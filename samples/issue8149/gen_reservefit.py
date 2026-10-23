#!/usr/bin/env python3
"""#8149 쪽 바닥 첫 조각 아래 예약량을 2.7px 단위로 재는 HWPX 생성기.

  python3 gen_reservefit.py samples/issue1549_multipositive_float_tables.hwpx reservefit.hwpx FILL

gen_rowfit.py 의 표(1열 30행, 행 1500HU)를 같은 채움 줄 FILL 개 뒤에 두고 바깥 아래 여백만
0..3000HU(200HU 간격)로 바꾼다. 아래 캡션 있음/없음 두 계열. 한/글 저장·Print 뒤 첫 조각 행 수가
한 행 줄어드는 여백 문턱에서 `행 하단 + 예약 = 본문 바닥` 을 풀어 예약 구성을 정한다.
"""
import importlib.util
import os
import sys

spec = importlib.util.spec_from_file_location(
    "gen_rowfit", os.path.join(os.path.dirname(os.path.abspath(__file__)), "gen_rowfit.py"))
rowfit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rowfit)

FILL = int(sys.argv[3])
CASES = [(cap, ob) for cap in (True, False) for ob in range(0, 3001, 200)]


def body():
    out = []
    for i, (cap, ob) in enumerate(CASES):
        label = f"R{i + 1:02d}"
        out.append(rowfit.para(f"{label} 캡션={'있음' if cap else '없음'} 아래여백={ob} 행높이=1500", page_break=i > 0))
        out.extend(rowfit.para(f"{label} 채움 {n}") for n in range(1, FILL + 1))
        out.append(rowfit.host(rowfit.table(label, cap, ob, 1500)))
    out.append(rowfit.para("끝", page_break=True))
    return out


rowfit.body = body
sys.argv = sys.argv[:3]
rowfit.main()
