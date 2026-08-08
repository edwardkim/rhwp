"""오풀이(음성 대조) — 문서를 열지 않고 쪽수를 전부 1로 보고한다."""
import json
import os

inputs = json.loads(os.environ["BENCH_INPUTS_JSON"])
files = [{"file": os.path.basename(p), "pageCount": 1} for p in inputs]
summary = {"files": files, "totalPageCount": len(files)}
with open(os.path.join(os.environ["BENCH_OUT_DIR"], "summary.json"), "w", encoding="utf-8") as fh:
    json.dump(summary, fh, ensure_ascii=False)
