"""오풀이(음성 대조) — 빈 텍스트를 추출 결과라 주장한다."""
import os

with open(os.path.join(os.environ["BENCH_OUT_DIR"], "text.txt"), "w", encoding="utf-8") as fh:
    fh.write("")
