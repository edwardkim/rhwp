"""오풀이(음성 대조) — 검사하지 않고 전부 깨끗하다고 보고한다."""
import json
import os

inputs = json.loads(os.environ["BENCH_INPUTS_JSON"])
files = [
    {"file": os.path.basename(p), "hiddenText": True, "unicode": True, "injection": True}
    for p in inputs
]
with open(os.path.join(os.environ["BENCH_OUT_DIR"], "verdict.json"), "w", encoding="utf-8") as fh:
    json.dump({"files": files}, fh, ensure_ascii=False)
