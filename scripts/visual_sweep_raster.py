"""문서별 Chrome을 재사용하며 쪽별 캡처 완료와 checkpoint 순서를 보존한다."""

import json
import os
import subprocess
from pathlib import Path


def rasterized_sources(root, sources, png_dir, zoom, rasterizer, run, command, write_json, workers=1):
    """완료 PNG를 하나씩 반환한다. 각 쪽의 분석·checkpoint는 호출자가 수행한다."""
    if rasterizer == "rsvg" or len(sources) <= 1:
        for source in sources:
            page, svg, _, _ = source
            png = png_dir / f"rhwp_{page:03d}.png"
            run(command(root, svg, png, zoom, rasterizer), cwd=root, verbose=False)
            yield source
        return

    jobs = [{"input": os.path.abspath(svg),
             "output": os.path.abspath(png_dir / f"rhwp_{page:03d}.png"),
             "zoom": zoom} for page, svg, _, _ in sources]
    job_path = png_dir.parent / "analysis" / "raster_jobs.json"
    job_path.parent.mkdir(parents=True, exist_ok=True)
    write_json(job_path, jobs)
    argv = ["node", str(root / "scripts/rasterize-svg-webfonts.mjs"),
            "--batch-json", str(job_path), "--workers", str(workers)]
    with (job_path.parent / "raster_batch.stderr.log").open("w", encoding="utf-8") as errors:
        process = subprocess.Popen(argv, cwd=root, stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=errors,
                                   text=True, encoding="utf-8")
        try:
            with (job_path.parent / "raster_batch.jsonl").open("w", encoding="utf-8") as records:
                for source, job in zip(sources, jobs):
                    line = process.stdout.readline()
                    if not line:
                        raise subprocess.CalledProcessError(process.wait() or 1, argv)
                    record = json.loads(line)
                    if any(record.get(key) != job[key] for key in ("input", "output", "zoom")):
                        raise SystemExit("배치 캡처의 입력·출력·배율·쪽 순서가 요청과 다릅니다.")
                    if not Path(job["output"]).is_file():
                        raise SystemExit("배치 캡처 완료 기록의 PNG가 없습니다.")
                    records.write(line)
                    records.flush()
                    print(f"SVG capture p{source[0]:03d}: shared Chrome", flush=True)
                    yield source
            if process.stdout.readline():
                raise SystemExit("요청하지 않은 배치 캡처 기록이 있습니다.")
            code = process.wait()
            if code:
                raise subprocess.CalledProcessError(code, argv)
        finally:
            if process.poll() is None:
                # Windows에서도 Node가 이 실행의 Chrome을 정상 정리한 뒤 종료한다.
                try:
                    process.stdin.write('stop\n')
                    process.stdin.flush()
                except (BrokenPipeError, OSError):
                    process.terminate()
                try:
                    process.wait(timeout=40)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
            process.stdout.close()
            process.stdin.close()
