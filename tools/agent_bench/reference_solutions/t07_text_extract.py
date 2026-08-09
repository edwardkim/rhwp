"""레퍼런스 풀이 — 본문 텍스트 추출: export-text --json 의 pages[].text 이어 붙이기."""
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]

proc = subprocess.run(
    [RHWP, "export-text", INP, "--json"], check=True, capture_output=True
)
envelope = json.loads(proc.stdout.decode("utf-8"))
joined = "\n".join(page.get("text", "") for page in envelope["pages"])
with open(os.path.join(OUT, "text.txt"), "w", encoding="utf-8") as fh:
    fh.write(joined)
