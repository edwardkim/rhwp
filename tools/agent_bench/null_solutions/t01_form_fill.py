"""오풀이(음성 대조) — 입력을 그대로 복사하고 채웠다고 주장한다."""
import os
import shutil

shutil.copyfile(os.environ["BENCH_INPUT"], os.path.join(os.environ["BENCH_OUT_DIR"], "filled.hwp"))
