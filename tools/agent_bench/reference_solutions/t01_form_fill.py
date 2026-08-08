"""레퍼런스 풀이 — 서식 누름틀 채우기: fields 확인 없이 fill-fields 한 방."""
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]
params = json.loads(os.environ["BENCH_PARAMS_JSON"])

subprocess.run(
    [
        RHWP, "edit", "fill-fields", INP,
        "--data", json.dumps(params["fields"], ensure_ascii=False),
        "-o", os.path.join(OUT, "filled.hwp"), "--json",
    ],
    check=True, capture_output=True,
)
