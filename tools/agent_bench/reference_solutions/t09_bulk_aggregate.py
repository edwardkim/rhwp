"""레퍼런스 풀이 — 대량 집계: 문서별 info --json 의 pageCount 를 모아 대장을 만든다."""
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
OUT = os.environ["BENCH_OUT_DIR"]
inputs = json.loads(os.environ["BENCH_INPUTS_JSON"])

files = []
for path in inputs:
    proc = subprocess.run([RHWP, "info", path, "--json"], check=True, capture_output=True)
    info = json.loads(proc.stdout.decode("utf-8"))
    files.append({"file": os.path.basename(path), "pageCount": info["pageCount"]})
summary = {"files": files, "totalPageCount": sum(f["pageCount"] for f in files)}
with open(os.path.join(OUT, "summary.json"), "w", encoding="utf-8") as fh:
    json.dump(summary, fh, ensure_ascii=False)
