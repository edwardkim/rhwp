"""gym 손상-강건성 감사 — rhwp 가 적대적/손상 입력에 절대 패닉·행 하지 않는가.

## 왜 이 도구인가 (도구 강건성이 능력의 천장)

2026 프론티어(AgentHijack 등)는 에이전트가 **환경 손상**에 견디는지 잰다. gym 은
에이전트가 rhwp 를 몰아 능력을 낸다 — 그런데 rhwp 가 손상 문서에 **패닉**하면
에이전트가 아무리 유능해도 과제를 못 끝낸다. 도구의 적대적 강건성이 능력의 천장이다.

이 감사기는 코퍼스를 **결정적으로 손상**시켜(무작위 없음 — 재현 가능) rhwp 가 언제나
우아하게 실패/부분복구하는지 인증한다:

- **패닉**(exit 101 · 시그널/음수 코드 · 'panicked' · 스택 오버플로) → 실패.
- **행**(timeout) → 실패.
- 그 외(깨끗한 비-0 실패 · 경고 후 부분복구 · 정상 파싱) → 우아함(정상).

종점 무결성(#4808 판별력)·경로 무결성(#4810 트라젝토리)에 이은 세 번째 기둥 —
도구 자체의 손상 강건성. 이것이 다른 문서 벤치마크가 안 재는 축이다: 벤치마크가
자기 도구가 적대적 입력에 죽지 않음을 CI 로 인증한다.

## 사용

    python gym/tools/robustness.py --bin target/debug/rhwp            # 결정적 부분집합
    python gym/tools/robustness.py --bin target/debug/rhwp --limit 40 # 더 넓게
    python gym/tools/robustness.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, GYM_ROOT)

from core import runner  # noqa: E402


def deterministic_mutants(data: bytes):
    """결정적 손상 변형들 — (라벨, 바이트). 무작위 없음(재현 가능)."""
    n = len(data)
    if n == 0:
        # 빈 입력은 위치 기반 flip/절단을 할 수 없지만, 감사 자체가 예외를 내면 안 된다.
        return [("empty-to-nul", b"\0")]
    out = []
    for pct in (25, 50, 75, 95):                       # 절단
        out.append((f"truncate@{pct}%", data[:max(1, n * pct // 100)]))
    for pct in (10, 50, 90):                           # 바이트 플립
        pos = min(n - 1, n * pct // 100)
        b = bytearray(data)
        b[pos] ^= 0xFF
        out.append((f"flip@{pct}%", bytes(b)))
    b = bytearray(data)                                # 헤더 매직 파손
    for i in range(min(n, 512)):
        b[i] = 0
    out.append(("zero-header", bytes(b)))
    return out


def select_samples(samples_dir: str, limit: int):
    """정렬된 .hwp 를 결정적 stride 로 limit 개 뽑는다(형식·크기 다양성 확보)."""
    everything = sorted(f for f in os.listdir(samples_dir) if f.endswith(".hwp"))
    if not everything or limit <= 0:
        return everything[:max(0, limit)], len(everything)
    if len(everything) <= limit:
        return everything, len(everything)
    stride = len(everything) / limit
    picked = [everything[min(len(everything) - 1, int(i * stride))] for i in range(limit)]
    # stride 반올림 중복 제거(순서 유지)
    seen, uniq = set(), []
    for f in picked:
        if f not in seen:
            seen.add(f)
            uniq.append(f)
    return uniq, len(everything)


def is_panic(code, err: str) -> bool:
    """우아한 실패(비-0)와 패닉(크래시)을 가른다."""
    low = err.lower()
    if "panicked" in low or "stack overflow" in low or "core dumped" in low:
        return True
    if code is None:
        return False
    # POSIX subprocess는 signal 종료를 음수로 돌려준다. Windows NTSTATUS 기반
    # 크래시는 큰 양수로 오므로 상위 두 비트로 구분한다. 임의의 CLI 오류 코드
    # (예: 255)를 패닉으로 오판하지 않는다.
    windows_exception = code >= 0 and (code & 0xC0000000) == 0xC0000000
    return code == 101 or code < 0 or windows_exception


def probe(bin_path: str, path: str, timeout: int):
    """한 손상 파일을 파싱 시도 — (code, panicked, timed_out, head)."""
    try:
        p = subprocess.run([bin_path, "info", path, "--json"], cwd=REPO_ROOT,
                           capture_output=True, timeout=timeout)
        err = p.stderr.decode("utf-8", "replace") + p.stdout.decode("utf-8", "replace")
        return p.returncode, is_panic(p.returncode, err), False, err[:160]
    except subprocess.TimeoutExpired:
        return None, False, True, f"timeout {timeout}s"


def audit(bin_path: str, samples_dir: str, limit: int, timeout: int) -> dict:
    picked, total = select_samples(samples_dir, limit)
    panics, hangs = [], []
    checked = 0
    degraded = 0
    with tempfile.TemporaryDirectory() as work:
        mut_path = os.path.join(work, "mutant.hwp")
        for name in picked:
            with open(os.path.join(samples_dir, name), "rb") as fh:
                data = fh.read()
            for label, mut in deterministic_mutants(data):
                with open(mut_path, "wb") as fh:
                    fh.write(mut)
                code, panicked, timed_out, head = probe(bin_path, mut_path, timeout)
                checked += 1
                tag = f"{name}:{label}"
                if timed_out:
                    hangs.append(tag)
                elif panicked:
                    panics.append(f"{tag} (code {code}): {head}")
                elif code not in (0, None):
                    degraded += 1
    return {
        "kind": "gymRobustness",
        "schemaVersion": "1.0",
        "ok": len(panics) == 0 and len(hangs) == 0,
        "samplesTested": len(picked),
        "totalSamples": total,
        "mutantsChecked": checked,
        "gracefullyDegraded": degraded,
        "panics": panics,
        "hangs": hangs,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 손상-강건성 감사 — rhwp 패닉·행 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--limit", type=int, default=16, help="감사할 샘플 수(결정적 부분집합)")
    ap.add_argument("--timeout", type=int, default=20)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    if a.timeout <= 0:
        ap.error("--timeout은 양수여야 합니다")
    bin_path = runner.find_bin(a.bin)
    samples_dir = os.path.join(REPO_ROOT, "samples")
    report = audit(bin_path, samples_dir, a.limit, a.timeout)
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    elif report["ok"]:
        print(f"gym 손상-강건성 감사: 샘플 {report['samplesTested']}/{report['totalSamples']} × "
              f"손상 {report['mutantsChecked']}건 — 패닉 0 · 행 0 "
              f"(우아한 실패/부분복구 {report['gracefullyDegraded']})")
    else:
        print(f"gym 손상-강건성 감사: 패닉 {len(report['panics'])} · 행 {len(report['hangs'])} — "
              "rhwp 가 손상 입력에 죽는다:")
        for t in report["panics"] + report["hangs"]:
            print(f"  - {t}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
