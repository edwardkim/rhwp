#!/usr/bin/env python3
"""문서별 PI→쪽 지도의 해시를 뽑는다 (COM 불필요).

두 바이너리의 해시를 비교하면 **PI 판정이 바뀔 수 있는 문서**만 골라낼 수 있다.
해시가 같으면 그 문서의 PI 오라클 결과는 이전 실행 그대로다.

사용: python tools/pi_map_hash.py <chunks_dir> <out_tsv> --exe <rhwp> [--jobs N]

측정하지 못한 문서는 "변경 없음"으로 취급하면 안 된다. 하나라도 실패하면 기존 TSV를
바꾸지 않고 비영(0이 아닌) 종료한다. 성공 TSV의 ``doc`` 열에는 chunk에 적힌 원래 경로를
보존해 동명 파일도 구분한다.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import os
import re
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


PG = re.compile(r"=== 페이지 (\d+) \(global_idx=\d+, section=(\d+),")
PI = re.compile(r"\bpi=(\d+)")


def positive_int(raw: str) -> int:
    """argparse용 0 초과 정수."""
    try:
        value = int(raw)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"정수가 아닙니다: {raw}") from error
    if value <= 0:
        raise argparse.ArgumentTypeError(f"0보다 커야 합니다: {raw}")
    return value


def load_files(chunks_dir: Path) -> list[str]:
    """chunk 파일의 경로를 입력 순서대로 중복 제거해 읽는다."""
    if not chunks_dir.is_dir():
        raise ValueError(f"chunk 디렉터리가 아닙니다: {chunks_dir}")
    chunks = sorted(chunks_dir.glob("chunk_*.txt"))
    if not chunks:
        raise ValueError(f"chunk_*.txt 파일이 없습니다: {chunks_dir}")

    files: list[str] = []
    seen: set[str] = set()
    for chunk in chunks:
        for line in chunk.read_text(encoding="utf-8").splitlines():
            if not line.strip() or line in seen:
                continue
            seen.add(line)
            files.append(line)
    if not files:
        raise ValueError(f"측정할 문서 경로가 없습니다: {chunks_dir}")
    return files


def run_one(exe: str, path: str, timeout: int) -> tuple[str | None, int | None, str | None]:
    """문서 하나의 PI→쪽 지도를 반환하거나 실패 사유를 반환한다."""
    try:
        process = subprocess.run(
            [exe, "dump-pages", path],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            stdin=subprocess.DEVNULL,
        )
    except subprocess.TimeoutExpired:
        return None, None, f"{timeout}초 시간 초과"
    except OSError as error:
        return None, None, f"실행 실패: {error}"

    if process.returncode != 0:
        detail = next((line.strip() for line in process.stderr.splitlines() if line.strip()), "")
        suffix = f" ({detail})" if detail else ""
        return None, None, f"dump-pages 종료 코드 {process.returncode}{suffix}"

    page: int | None = None
    section: int | None = None
    seen: dict[tuple[int, int], int] = {}
    pages = 0
    for line in process.stdout.splitlines():
        match = PG.search(line)
        if match:
            page = int(match.group(1))
            section = int(match.group(2))
            pages = max(pages, page)
            continue
        if page is None or section is None:
            continue
        for raw_pi in PI.findall(line):
            key = (section, int(raw_pi))
            seen.setdefault(key, page)

    if pages == 0:
        return None, None, "dump-pages 출력에 페이지 지도가 없습니다"
    blob = ";".join(f"{section}.{pi}={page}" for (section, pi), page in sorted(seen.items()))
    return hashlib.sha1(blob.encode()).hexdigest()[:16], pages, None


def collect_results(
    exe: str, files: list[str], jobs: int, timeout: int
) -> tuple[list[tuple[str, str, int]], list[tuple[str, str]]]:
    """전체 문서의 측정을 끝낸 뒤 성공 결과와 실패 결과를 분리한다."""
    results: list[tuple[str, str, int]] = []
    failures: list[tuple[str, str]] = []
    with ThreadPoolExecutor(max_workers=jobs) as executor:
        futures = [(path, executor.submit(run_one, exe, path, timeout)) for path in files]
        for path, future in futures:
            try:
                digest, pages, error = future.result()
            except Exception as unexpected:  # worker 예외도 성공 TSV 생성 전에 중단한다.
                failures.append((path, f"예상하지 못한 측정 예외: {unexpected}"))
                continue
            if error is not None or digest is None or pages is None:
                failures.append((path, error or "불완전한 측정 결과"))
                continue
            results.append((path, digest, pages))
    return results, failures


def write_results(out_tsv: Path, results: list[tuple[str, str, int]]) -> None:
    """성공한 전체 결과만 임시 파일을 거쳐 원자적으로 교체한다."""
    if not out_tsv.parent.is_dir():
        raise ValueError(f"출력 디렉터리가 없습니다: {out_tsv.parent}")
    fd, temporary = tempfile.mkstemp(prefix=f".{out_tsv.name}.", dir=out_tsv.parent, text=True)
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="") as handle:
            writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
            writer.writerow(("doc", "pi_hash", "pages"))
            writer.writerows(results)
        os.replace(temporary, out_tsv)
    except Exception:
        Path(temporary).unlink(missing_ok=True)
        raise


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("chunks_dir", type=Path)
    parser.add_argument("out_tsv", type=Path)
    parser.add_argument("--exe", required=True)
    parser.add_argument("--jobs", type=positive_int, default=12)
    parser.add_argument("--timeout", type=positive_int, default=300)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        files = load_files(args.chunks_dir)
    except ValueError as error:
        print(f"오류: {error}", file=sys.stderr)
        return 2

    print(f"문서 {len(files)}건", flush=True)
    started = time.monotonic()
    results, failures = collect_results(args.exe, files, args.jobs, args.timeout)
    if failures:
        print(f"오류: {len(failures)}건의 PI 지도를 만들지 못했습니다. 기존 TSV는 유지합니다.", file=sys.stderr)
        for path, error in failures[:20]:
            print(f"  - {path}: {error}", file=sys.stderr)
        if len(failures) > 20:
            print(f"  - 그 외 {len(failures) - 20}건", file=sys.stderr)
        return 1

    try:
        write_results(args.out_tsv, results)
    except ValueError as error:
        print(f"오류: {error}", file=sys.stderr)
        return 2
    print(f"=== 완료 {(time.monotonic() - started) / 60:.1f}m ===", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
