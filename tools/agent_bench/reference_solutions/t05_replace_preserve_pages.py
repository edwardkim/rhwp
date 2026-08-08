"""레퍼런스 풀이 — 쪽수 보존 일괄 치환: edit replace-text (동일 길이 문자열)."""
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]
params = json.loads(os.environ["BENCH_PARAMS_JSON"])

subprocess.run(
    [
        RHWP, "edit", "replace-text", INP,
        "--find", params["find"], "--replace", params["replace"],
        "-o", os.path.join(OUT, "edited.hwp"), "--json",
    ],
    check=True, capture_output=True,
)
