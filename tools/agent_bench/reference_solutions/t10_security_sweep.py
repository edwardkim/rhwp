"""레퍼런스 풀이 — 보안 스윕: 문서별 inspect 3축의 clean 판정을 대장으로 만든다."""
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
OUT = os.environ["BENCH_OUT_DIR"]
inputs = json.loads(os.environ["BENCH_INPUTS_JSON"])
AXES = {"hiddenText": "hidden-text", "unicode": "unicode", "injection": "injection"}

files = []
for path in inputs:
    row = {"file": os.path.basename(path)}
    for key, axis in AXES.items():
        proc = subprocess.run(
            [RHWP, "inspect", axis, path, "--json"], check=True, capture_output=True
        )
        row[key] = json.loads(proc.stdout.decode("utf-8"))["clean"]
    files.append(row)
with open(os.path.join(OUT, "verdict.json"), "w", encoding="utf-8") as fh:
    json.dump({"files": files}, fh, ensure_ascii=False)
