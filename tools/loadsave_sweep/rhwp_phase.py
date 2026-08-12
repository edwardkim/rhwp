#!/usr/bin/env python3
"""Phase A — rhwp 변환 매트릭스 실행기 (COM 불필요, 병렬 가능).

master.tsv 의 각 문서에 대해 rhwp 저장 경로 두 갈래를 실행한다:

    hwp  입력:  convert → <docid>.h2h.hwp     export-hwpx → <docid>.h2x.hwpx
    hwpx 입력:  convert → <docid>.x2h.hwp     export-hwpx → <docid>.x2x.hwpx

경로 매트릭스가 곧 진단이다 — 한 입력의 두 산출이 모두 나쁘면 불러오기(parse) 쪽,
한쪽만 나쁘면 그 저장 축 쪽이다.

`--verify`/`--verify-pages` 를 함께 걸어 rhwp 의 자기검증 판정(exit 3/4)도 데이터로
수확한다. exit 3/4 는 산출물을 남기므로 Phase B 측정 대상에 포함된다.

산출:
    <out>/conv/<docid>.<route>.hwp|hwpx   변환 산출물
    <out>/phase_a.ndjson                  저널 (재실행 시 완료 건 재개-생략)
    <out>/oracle_tasks.tsv                Phase B 작업 목록 (key \t abspath, 문서별 orig→routes 순)

사용:
    python rhwp_phase.py --master master.tsv --out D:\\sweep\\s1 --rhwp <rhwp.exe> [--jobs 6]
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import hashlib
import json
import subprocess
import sys
import threading
import time
from pathlib import Path

EXIT_STATUS = {0: "OK", 3: "VERIFY_DIFF", 4: "PAGE_DIFF"}


def read_master(path: Path) -> list[dict]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        docid, fmt, src = line.split("\t", 2)
        rows.append({"docid": docid, "format": fmt, "src": src})
    return rows


def routes_for(fmt: str) -> list[tuple[str, str, str]]:
    # (route, subcommand, 출력 확장자)
    if fmt == "hwp":
        return [("h2h", "convert", ".hwp"), ("h2x", "export-hwpx", ".hwpx")]
    return [("x2h", "convert", ".hwp"), ("x2x", "export-hwpx", ".hwpx")]


def run_one(rhwp: str, doc: dict, route: str, sub: str, ext: str,
            conv_dir: Path, timeout: int, verify: bool) -> dict:
    out_path = conv_dir / f"{doc['docid']}.{route}{ext}"
    cmd = [rhwp, sub, doc["src"], str(out_path)]
    if verify:
        cmd += ["--verify", "--verify-pages"]
    t0 = time.monotonic()
    rec = {"docid": doc["docid"], "format": doc["format"], "route": route,
           "src": doc["src"], "output": str(out_path)}
    try:
        p = subprocess.run(cmd, capture_output=True, timeout=timeout)
        rec["exit"] = p.returncode
        rec["status"] = EXIT_STATUS.get(p.returncode, "FAIL")
        tail = p.stderr.decode("utf-8", "replace").strip().splitlines()[-3:]
        if rec["status"] != "OK" and tail:
            rec["stderr"] = " | ".join(tail)[:500]
    except subprocess.TimeoutExpired:
        rec["exit"] = -1
        rec["status"] = "TIMEOUT"
    except OSError as e:
        rec["exit"] = -1
        rec["status"] = "SPAWN_FAIL"
        rec["stderr"] = str(e)
    rec["ms"] = int((time.monotonic() - t0) * 1000)
    rec["bytes"] = out_path.stat().st_size if out_path.is_file() else 0
    # exit 3/4 라도 산출물이 있으면 Phase B 가 측정한다. 산출물 없는 "성공"은 없다.
    if rec["status"] in ("OK", "VERIFY_DIFF", "PAGE_DIFF") and rec["bytes"] == 0:
        rec["status"] = "FAIL"
        rec.setdefault("stderr", "exit ok but output missing/empty")
    return rec


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--master", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--rhwp", required=True)
    ap.add_argument("--jobs", type=int, default=6)
    ap.add_argument("--timeout", type=int, default=180, help="한 변환 호출의 초 상한")
    ap.add_argument("--no-verify", action="store_true",
                    help="--verify/--verify-pages 자기검증 생략 (속도 우선)")
    args = ap.parse_args()

    out_dir = Path(args.out)
    conv_dir = out_dir / "conv"
    conv_dir.mkdir(parents=True, exist_ok=True)
    journal_path = out_dir / "phase_a.ndjson"

    rhwp = str(Path(args.rhwp).resolve())
    exe_sha = hashlib.sha256(Path(rhwp).read_bytes()).hexdigest()

    docs = read_master(Path(args.master))

    # 재개: 저널에 이미 있고 산출물 상태가 저널과 일치하는 (docid, route) 는 건너뛴다.
    done: dict[tuple[str, str], dict] = {}
    if journal_path.is_file():
        for line in journal_path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("kind") == "meta":
                if r.get("exeSha") != exe_sha:
                    print(f"[phase-a] 경고: 저널의 exe({r.get('exeSha','?')[:12]})와 현재 "
                          f"exe({exe_sha[:12]})가 다르다 — 섞인 저널은 판정 출처를 오염시킨다. "
                          f"새 --out 디렉터리를 쓰거나 저널을 지우고 전체 재실행할 것.",
                          file=sys.stderr)
                    return 2
                continue
            done[(r["docid"], r["route"])] = r

    lock = threading.Lock()
    journal = journal_path.open("a", encoding="utf-8", newline="\n")
    if not done:
        journal.write(json.dumps({"kind": "meta", "exe": rhwp, "exeSha": exe_sha,
                                  "master": str(args.master)}, ensure_ascii=False) + "\n")
        journal.flush()

    tasks = []
    for doc in docs:
        for route, sub, ext in routes_for(doc["format"]):
            key = (doc["docid"], route)
            prev = done.get(key)
            if prev and (prev["status"] not in ("OK", "VERIFY_DIFF", "PAGE_DIFF")
                         or Path(prev["output"]).is_file()):
                continue
            tasks.append((doc, route, sub, ext))

    total = len(tasks)
    print(f"[phase-a] {len(docs)} docs, {total} conversions to run "
          f"({len(done)} already journaled), jobs={args.jobs}")

    n_done = 0
    t0 = time.monotonic()

    def work(t):
        doc, route, sub, ext = t
        return run_one(rhwp, doc, route, sub, ext, conv_dir, args.timeout, not args.no_verify)

    with cf.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for rec in ex.map(work, tasks):
            with lock:
                journal.write(json.dumps(rec, ensure_ascii=False) + "\n")
                journal.flush()
                done[(rec["docid"], rec["route"])] = rec
                n_done += 1
                if n_done % 200 == 0:
                    rate = n_done / max(time.monotonic() - t0, 1e-9) * 60
                    print(f"[phase-a] {n_done}/{total} ({rate:.0f}/min)")
    journal.close()

    # Phase B 작업 목록 — 문서별로 orig 다음에 그 문서의 산출물이 오도록 묶는다.
    # (COM 워커의 시간 지역성 + 웨이브 단위 삭제·재개를 쉽게 한다)
    n_meas, n_conv_fail = 0, 0
    tasks_path = out_dir / "oracle_tasks.tsv"
    with tasks_path.open("w", encoding="utf-8", newline="\n") as f:
        for doc in docs:
            f.write(f"{doc['docid']}.orig\t{doc['src']}\n")
            n_meas += 1
            for route, _sub, _ext in routes_for(doc["format"]):
                rec = done.get((doc["docid"], route))
                if rec and rec["status"] in ("OK", "VERIFY_DIFF", "PAGE_DIFF"):
                    f.write(f"{doc['docid']}.{route}\t{rec['output']}\n")
                    n_meas += 1
                else:
                    n_conv_fail += 1
    print(f"[phase-a] done. oracle tasks: {n_meas} opens, convert failures: {n_conv_fail}")
    print(f"[phase-a] journal: {journal_path}")
    print(f"[phase-a] tasks:   {tasks_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
