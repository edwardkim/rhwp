"""레퍼런스 풀이 — 개인정보 마스킹: edit redact 기본 kind(전체 축) 적용."""
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]

subprocess.run(
    [RHWP, "edit", "redact", INP, "-o", os.path.join(OUT, "redacted.hwp"), "--json"],
    check=True, capture_output=True,
)
