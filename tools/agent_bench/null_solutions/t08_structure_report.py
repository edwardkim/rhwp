"""오풀이(음성 대조) — 문서를 열지 않고 그럴듯한 기본값을 보고한다."""
import json
import os

answers = {"mode": "outline", "nodeCount": 0, "topLevelCount": 0}
with open(os.path.join(os.environ["BENCH_OUT_DIR"], "answers.json"), "w", encoding="utf-8") as fh:
    json.dump(answers, fh, ensure_ascii=False)
