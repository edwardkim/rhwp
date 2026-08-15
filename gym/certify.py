"""gym 능력 인증서 — 능력 점수를 재현 가능하게 봉인해 위조 불가능한 신뢰 원본으로.

## 왜 이 도구인가 (증명 가능한 능력)

점수를 재는 것과 그 점수가 **진짜**임을 증명하는 것은 다르다. 누가 "내 에이전트가
gym 을 만점 통과했다"고 주장해도, 그게 몰래 축소한 벤치마크나 다른 바이너리로 낸
것이면 거짓이다. 리포트(report.py)만으로는 그 위조를 막지 못한다.

이 인증서가 막는다:

- **벤치마크 지문** — 전 pack 정의(pack.json·tasks·reference)의 sha256. 인증서가
  '무엇을' 재고 채점했는지 못박는다. 벤치마크를 몰래 줄이면 지문이 바뀌어 들킨다.
- **바이너리 신원** — `capabilitiesSha256`. '어느 바이너리로' 냈는지 못박는다.
- **재현 = 증명** — 같은 바이너리 + 같은 벤치마크면 누구나 같은 점수를 재현한다.
  `--verify` 가 다시 돌려 인증서와 대조한다: 재현되면 진짜, 아니면 위조.

암호 서명이 아니라 **결정론적 재현**이 증명 원리다 — reproducible-build attestation 과
같은 계열이라 키 관리 없이 누구나 검증할 수 있다.

## 사용

    python gym/certify.py --bin target/debug/rhwp --out cert.json      # 인증서 발급
    python gym/certify.py --verify cert.json --bin target/debug/rhwp    # 재현 검증(exit 0/1)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = HERE
REPO_ROOT = os.path.dirname(GYM_ROOT)

CERT_KIND = "gymCapabilityCertificate"
CERT_SCHEMA = "1.0"


def benchmark_fingerprint(gym_root: str) -> str:
    """실제 측정 입력·프로토콜의 결정론적 sha256.

    과제 선언만이 아니라 pack asset, 채점기, 기준 풀이 조립기, 리포트·커버리지
    코드도 결과를 바꾼다. 이 중 하나라도 바뀌면 같은 점수라도 다른 benchmark
    certification으로 취급해야 한다.
    """
    entries = []

    def add_file(path: str) -> None:
        if os.path.isfile(path):
            rel = os.path.relpath(path, gym_root).replace(os.sep, "/")
            with open(path, "rb") as fh:
                entries.append((rel, fh.read()))

    def add_tree(rel_dir: str) -> None:
        root = os.path.join(gym_root, rel_dir)
        if not os.path.isdir(root):
            return
        for current, dirs, files in os.walk(root):
            dirs[:] = sorted(d for d in dirs if d != "__pycache__")
            for name in sorted(files):
                if not name.endswith(".pyc"):
                    add_file(os.path.join(current, name))

    # `packs`는 과제 선언뿐 아니라 그 과제가 실제로 읽는 asset도 포함한다.
    # 나머지는 report.py가 점수를 재는 코드 경로와 인증서 판정 자체다.
    for rel_dir in ("packs", "core", "profiles", "tools"):
        add_tree(rel_dir)
    for name in ("score.py", "report.py", "certify.py"):
        add_file(os.path.join(gym_root, name))
    h = hashlib.sha256()
    for rel, data in sorted(entries):
        h.update(rel.encode("utf-8"))
        h.update(b"\0")
        h.update(hashlib.sha256(data).digest())
    return h.hexdigest()


def reproducible_core(report: dict, fingerprint: str) -> dict:
    """인증서에서 재현으로 대조하는 필드 — 변동 메타(git commit·시각·agent 이름)는 뺀다."""
    runner = report.get("runner") or {}
    cov = report.get("coverage") or {}
    return {
        "benchmarkFingerprint": fingerprint,
        "capabilitiesSha256": runner.get("capabilitiesSha256"),
        "accuracy": report.get("accuracy"),
        "coverage": {k: cov.get(k) for k in ("percent", "covered", "agentFacingTotal")},
        "axisProfile": report.get("axisProfile"),
    }


def _run_report(bin_path: str) -> dict:
    out = subprocess.run(
        [sys.executable, os.path.join(GYM_ROOT, "report.py"), "--bin", bin_path, "--json"],
        cwd=REPO_ROOT, capture_output=True,
    )
    if out.returncode != 0:
        raise RuntimeError(f"report.py 실패: {out.stderr.decode('utf-8', 'replace')[:300]}")
    return json.loads(out.stdout)


def certify(bin_path: str, measured_at: str | None = None) -> dict:
    report = _run_report(bin_path)
    fp = benchmark_fingerprint(GYM_ROOT)
    cert = {
        "kind": CERT_KIND,
        "schemaVersion": CERT_SCHEMA,
        "benchmarkFingerprint": fp,
        "report": report,
        "proof": "reproduce: 같은 bin + 같은 pack 정의로 --verify 하면 core 가 일치한다",
    }
    if measured_at:
        cert["certifiedAt"] = measured_at
    return cert


def verify(cert: dict, bin_path: str) -> tuple[bool, list[str]]:
    """인증서를 재발급해 재현 core 를 대조한다 — 위조·환경 변화를 잡는다."""
    if cert.get("kind") != CERT_KIND:
        return False, [f"kind 가 {CERT_KIND} 가 아니다: {cert.get('kind')}"]
    claimed = reproducible_core(cert.get("report", {}), cert.get("benchmarkFingerprint", ""))
    fresh_report = _run_report(bin_path)
    fresh = reproducible_core(fresh_report, benchmark_fingerprint(GYM_ROOT))
    diffs = []
    if claimed["benchmarkFingerprint"] != fresh["benchmarkFingerprint"]:
        diffs.append("벤치마크 지문 불일치 — pack 정의가 인증 시점과 다르다(축소·변조 가능)")
    if claimed["capabilitiesSha256"] != fresh["capabilitiesSha256"]:
        diffs.append("바이너리 신원(capabilitiesSha256) 불일치 — 다른 바이너리다")
    if claimed["accuracy"] != fresh["accuracy"]:
        diffs.append(f"정확도 불일치: 인증 {claimed['accuracy']} vs 재현 {fresh['accuracy']}")
    if claimed["coverage"] != fresh["coverage"]:
        diffs.append(f"커버리지 불일치: 인증 {claimed['coverage']} vs 재현 {fresh['coverage']}")
    if claimed["axisProfile"] != fresh["axisProfile"]:
        diffs.append("축별 프로파일 불일치")
    return (len(diffs) == 0), diffs


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 능력 인증서 — 발급/재현 검증")
    ap.add_argument("--bin", required=True, help="rhwp 바이너리")
    ap.add_argument("--verify", help="검증할 인증서 JSON")
    ap.add_argument("--out", help="발급 인증서 출력 파일(생략 시 stdout)")
    ap.add_argument("--at", help="certifiedAt 메타(재현 core 에 미포함)")
    a = ap.parse_args()

    if a.verify:
        with open(a.verify, encoding="utf-8") as fh:
            cert = json.load(fh)
        ok, diffs = verify(cert, a.bin)
        if ok:
            print("✅ 인증서 재현 검증 통과 — 벤치마크·바이너리·전 점수가 재현된다(진짜)")
            return 0
        print("❌ 인증서 재현 검증 실패:")
        for d in diffs:
            print(f"  - {d}")
        return 1

    cert = certify(a.bin, a.at)
    text = json.dumps(cert, ensure_ascii=False, indent=2) + "\n"
    if a.out:
        with open(a.out, "w", encoding="utf-8") as fh:
            fh.write(text)
        r = cert["report"]
        print(f"발급: {a.out} · 정확도 {r['accuracy']['percent']}% · 지문 {cert['benchmarkFingerprint'][:12]}")
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
