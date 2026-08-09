"""레퍼런스 풀이 — HWP→HWPX 변환: export-hwpx."""
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]

subprocess.run(
    [RHWP, "export-hwpx", INP, os.path.join(OUT, "converted.hwpx")],
    check=True, capture_output=True,
)
