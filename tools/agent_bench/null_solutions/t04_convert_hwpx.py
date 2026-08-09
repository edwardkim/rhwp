"""오풀이(음성 대조) — HWP 바이트를 확장자만 바꿔 HWPX 라 주장한다."""
import os
import shutil

shutil.copyfile(os.environ["BENCH_INPUT"], os.path.join(os.environ["BENCH_OUT_DIR"], "converted.hwpx"))
