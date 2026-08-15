"""gym 코퍼스 퍼징 발견 엔진 — 전 코퍼스 × 다명령 × 다변형을 병렬로 두들겨 rhwp 의
DoS(패닉·무한루프)를 **근본원인별로 클러스터링**한다.

## 왜 이 도구인가 (강건성 감사와의 분업)

`robustness.py`(#4814)는 릴리스 **게이트**다 — 바운드된 부분집합으로 "패닉·행 0"을
강제해 회귀를 막는다. 이 도구는 그 앞단의 **발견 엔진**이다 — 전 코퍼스를 여러 명령·
여러 손상으로 **exhaustive** 하게 두들겨 아직 안 고쳐진 DoS 를 찾아, 패닉을 **소스
위치(file:line)별로 묶어** "고쳐야 할 고유 버그 목록"을 낸다. 아무도 손으로 수백
문서를 수천 가지로 퍼징하지 않는다 — 에이전트가 이걸 돌려 rhwp 를 계속 경화한다.

- 패닉: stderr 의 `panicked at file:line` → 그 위치로 클러스터. 스택 오버플로·시그널·
  비-0 어보트도 별도 버킷.
- 무한루프: timeout → 샘플별 버킷.

## 사용

    python gym/tools/fuzz_corpus.py --bin target/debug/rhwp                    # 기본 명령·전 코퍼스
    python gym/tools/fuzz_corpus.py --bin <bin> --commands info,export-text    # 명령 지정
    python gym/tools/fuzz_corpus.py --bin <bin> --limit 40 --workers 8 --json  # 부분집합·기계용
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, GYM_ROOT)

from core import runner  # noqa: E402

DEFAULT_COMMANDS = ["info", "export-text", "export-structure", "export-render-tree"]
PANIC_RE = re.compile(r"panicked at ([^\n]+?:\d+)")


def deterministic_mutants(data: bytes):
    """결정적 손상 변형 — (라벨, 바이트). 무작위 없음(재현 가능)."""
    n = len(data)
    out = []
    for pct in (5, 25, 50, 75, 95):
        out.append((f"trunc{pct}", data[: max(1, n * pct // 100)]))
    for pct in (10, 30, 50, 70, 90):
        pos = min(n - 1, n * pct // 100)
        b = bytearray(data)
        b[pos] ^= 0xFF
        out.append((f"flip{pct}", bytes(b)))
    for pct in (10, 40, 70):  # 길이필드 추정 위치를 큰 값으로
        pos = min(n - 4, n * pct // 100)
        b = bytearray(data)
        b[pos:pos + 4] = b"\xff\xff\xff\x7f"
        out.append((f"biglen{pct}", bytes(b)))
    return out


def select_samples(samples_dir: str, limit: int):
    everything = sorted(
        f for f in os.listdir(samples_dir) if f.endswith((".hwp", ".hwpx", ".hml"))
    )
    if limit <= 0 or len(everything) <= limit:
        return everything, len(everything)
    stride = len(everything) / limit
    picked, seen = [], set()
    for i in range(limit):
        f = everything[min(len(everything) - 1, int(i * stride))]
        if f not in seen:
            seen.add(f)
            picked.append(f)
    return picked, len(everything)


def classify(code, err: str):
    """(kind, bucket) — kind in {panic, hang, None}. bucket 은 클러스터 키."""
    low = err.lower()
    m = PANIC_RE.search(err)
    if m:
        return "panic", m.group(1)
    if "stack overflow" in low:
        return "panic", "stack-overflow"
    if "panicked" in low or code == 101 or (code is not None and (code < 0 or code >= 132)):
        return "panic", f"code{code}"
    return None, None


def probe(bin_path, cmd, mut_path, timeout):
    args = [bin_path, cmd, mut_path]
    if cmd == "convert":
        args.append(mut_path + ".out.hwpx")
    try:
        p = subprocess.run(args, cwd=REPO_ROOT, capture_output=True, timeout=timeout)
        err = p.stderr.decode("utf-8", "replace") + p.stdout.decode("utf-8", "replace")
        return classify(p.returncode, err)
    except subprocess.TimeoutExpired:
        return "hang", cmd


def fuzz(bin_path, samples_dir, commands, limit, workers, timeout, work_dir):
    picked, total = select_samples(samples_dir, limit)
    jobs = []
    for i, name in enumerate(picked):
        data = Path(samples_dir, name).read_bytes()
        for label, mut in deterministic_mutants(data):
            jobs.append((i, name, label, mut))

    panic_clusters, hang_clusters = {}, {}
    checked = 0

    def run_one(job):
        idx, name, label, mut = job
        p = os.path.join(work_dir, f"m{idx}_{label}.hwp")
        Path(p).write_bytes(mut)
        try:
            results = []
            for cmd in commands:
                kind, bucket = probe(bin_path, cmd, p, timeout)
                if kind:
                    results.append((kind, bucket, f"{name}:{label}:{cmd}"))
            return results
        finally:
            try:
                os.remove(p)
            except OSError:
                pass

    with ThreadPoolExecutor(max_workers=workers) as ex:
        for fut in as_completed([ex.submit(run_one, j) for j in jobs]):
            checked += 1
            for kind, bucket, tag in fut.result():
                target = panic_clusters if kind == "panic" else hang_clusters
                target.setdefault(bucket, []).append(tag)

    panics = sorted(
        ({"location": loc, "count": len(c), "example": c[0]} for loc, c in panic_clusters.items()),
        key=lambda x: -x["count"],
    )
    hangs = sorted(
        (
            {
                "command": cmd,
                "count": len(c),
                "samples": sorted({t.split(":")[0] for t in c}),
                "example": c[0],
            }
            for cmd, c in hang_clusters.items()
        ),
        key=lambda x: -x["count"],
    )
    return {
        "kind": "gymFuzzCorpus",
        "schemaVersion": "1.0",
        "ok": not panics and not hangs,
        "samplesTested": len(picked),
        "totalSamples": total,
        "commands": commands,
        "mutantsPerSample": len(deterministic_mutants(b"x" * 4096)),
        "runsChecked": checked * len(commands),
        "distinctPanicSites": len(panics),
        "panicClusters": panics,
        "hangClusters": hangs,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 코퍼스 퍼징 발견 엔진 — DoS 를 근본원인별로 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--commands", default=",".join(DEFAULT_COMMANDS),
                    help="쉼표구분 rhwp 명령 (기본: %(default)s)")
    ap.add_argument("--limit", type=int, default=0, help="샘플 수(0=전수)")
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--timeout", type=int, default=10)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    bin_path = runner.find_bin(a.bin)
    commands = [c.strip() for c in a.commands.split(",") if c.strip()]
    import tempfile

    with tempfile.TemporaryDirectory() as work:
        report = fuzz(bin_path, os.path.join(REPO_ROOT, "samples"), commands,
                      a.limit, a.workers, a.timeout, work)
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    elif report["ok"]:
        print(f"코퍼스 퍼징: 샘플 {report['samplesTested']}/{report['totalSamples']} × "
              f"명령 {len(commands)} × {report['runsChecked']} 실행 — DoS 0")
    else:
        print(f"코퍼스 퍼징: 고유 패닉 {report['distinctPanicSites']}곳 · "
              f"행 클러스터 {len(report['hangClusters'])}개 — 고쳐야 할 DoS:")
        for p in report["panicClusters"]:
            print(f"  PANIC {p['location']}  ({p['count']}건)  예: {p['example']}")
        for h in report["hangClusters"]:
            print(f"  HANG  {h['command']}  ({h['count']}건, {len(h['samples'])}샘플)  예: {h['example']}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
