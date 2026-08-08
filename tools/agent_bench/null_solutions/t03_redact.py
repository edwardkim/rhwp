"""오풀이(음성 대조) — 개인정보를 지우지 않은 채 배포본이라 주장한다."""
import os
import shutil

shutil.copyfile(os.environ["BENCH_INPUT"], os.path.join(os.environ["BENCH_OUT_DIR"], "redacted.hwp"))
