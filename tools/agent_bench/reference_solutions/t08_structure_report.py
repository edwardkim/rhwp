"""레퍼런스 풀이 — 구조 파악: export-structure --json 봉투에서 답을 뽑는다."""
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]

proc = subprocess.run(
    [RHWP, "export-structure", INP, "--json"], check=True, capture_output=True
)
envelope = json.loads(proc.stdout.decode("utf-8"))
answers = {
    "mode": envelope["mode"],
    "nodeCount": envelope["nodeCount"],
    "topLevelCount": len(envelope["structure"]["roots"]),
}
with open(os.path.join(OUT, "answers.json"), "w", encoding="utf-8") as fh:
    json.dump(answers, fh, ensure_ascii=False)
