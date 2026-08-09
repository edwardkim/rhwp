"""레퍼런스 풀이 — 시각 무회귀 편집: edit set-cell 로 해당 칸만 갱신."""
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]
params = json.loads(os.environ["BENCH_PARAMS_JSON"])

subprocess.run(
    [
        RHWP, "edit", "set-cell", INP,
        "--table", str(params["table"]), "--row", str(params["row"]),
        "--col", str(params["col"]), "--text", params["newValue"],
        "-o", os.path.join(OUT, "updated.hwpx"), "--json",
    ],
    check=True, capture_output=True,
)
