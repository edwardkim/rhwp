"""오풀이(음성 대조) — 아무것도 안 바꾸면 렌더는 당연히 같다. 편집 반영 검사가 잡는다."""
import os
import shutil

shutil.copyfile(os.environ["BENCH_INPUT"], os.path.join(os.environ["BENCH_OUT_DIR"], "updated.hwpx"))
