#!/usr/bin/env python
"""#3239 crop 폴백 회귀 검증 — imgDim 없는 200dpi 스캔 그림 확대·절단.

samples/issue3239/evaluation_form_200dpi_scan.hwp 를 export-png 으로 렌더해
수정 시점 레퍼런스(reference_page1.png)와 그레이스케일 픽셀 대조한다.

- 정상(적응 폴백): diff ≈ 0% (머신 간 AA 차 감안 임계 2%)
- 회귀(고정 75 HU/px 폴백): 스캔이 2.08배 확대·절단되어 diff ≈ 5.4%

사용: python tools/verify_issue3239.py [--exe <rhwp.exe>]  (기본: target/debug/rhwp.exe)
"""
import argparse
import glob
import os
import subprocess
import sys
import tempfile

import numpy as np
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SAMPLE = os.path.join(ROOT, "samples", "issue3239", "evaluation_form_200dpi_scan.hwp")
REFERENCE = os.path.join(ROOT, "samples", "issue3239", "reference_page1.png")
THRESHOLD_PCT = 2.0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", default=os.path.join(ROOT, "target", "debug", "rhwp.exe"))
    args = ap.parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run(
            [args.exe, "export-png", SAMPLE, "-o", tmp],
            check=True,
            capture_output=True,
            timeout=180,
        )
        pngs = sorted(glob.glob(os.path.join(tmp, "*.png")))
        if not pngs:
            print("FAIL: export-png 산출물 없음")
            return 1
        got = np.array(Image.open(pngs[0]).convert("L"), int)
        ref = np.array(Image.open(REFERENCE).convert("L"), int)
        if got.shape != ref.shape:
            print(f"FAIL: 크기 불일치 got={got.shape} ref={ref.shape}")
            return 1
        diff_pct = (np.abs(got - ref) > 40).mean() * 100
        verdict = "OK" if diff_pct < THRESHOLD_PCT else "FAIL"
        print(f"{verdict}: diff>{40} 픽셀 비율 {diff_pct:.2f}% (임계 {THRESHOLD_PCT}%)")
        return 0 if verdict == "OK" else 1


if __name__ == "__main__":
    sys.exit(main())
