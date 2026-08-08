"""레퍼런스 풀이 — 표 CSV 왕복: table-to-csv → 칸 수정 → csv-to-table."""
import csv
import json
import os
import subprocess

RHWP = os.environ["RHWP_BIN"]
INP = os.environ["BENCH_INPUT"]
OUT = os.environ["BENCH_OUT_DIR"]
params = json.loads(os.environ["BENCH_PARAMS_JSON"])
table = str(params["table"])
csv_path = os.path.join(OUT, "_roundtrip.csv")

subprocess.run(
    [RHWP, "table-to-csv", INP, "--table", table, "-o", csv_path, "--json"],
    check=True, capture_output=True,
)
with open(csv_path, newline="", encoding="utf-8-sig") as fh:
    rows = [row for row in csv.reader(fh)]
rows[params["row"]][params["col"]] = params["newValue"]
with open(csv_path, "w", newline="", encoding="utf-8") as fh:
    csv.writer(fh).writerows(rows)
subprocess.run(
    [
        RHWP, "csv-to-table", INP, "--csv", csv_path, "--table", table,
        "-o", os.path.join(OUT, "updated.hwpx"), "--json",
    ],
    check=True, capture_output=True,
)
os.remove(csv_path)
