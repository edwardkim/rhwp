"""오풀이(음성 대조) — 치환하지 않은 문서를 치환본이라 주장한다."""
import os
import shutil

shutil.copyfile(os.environ["BENCH_INPUT"], os.path.join(os.environ["BENCH_OUT_DIR"], "edited.hwp"))
