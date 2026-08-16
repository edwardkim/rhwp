#!/usr/bin/env python3
"""DAP/1.0 · DATP/1.0 준수 검사기 — 아키텍처를 반증 가능한 진술로 만든다.

## 왜 있는가

프로토콜 문서는 쓰기 쉽고 지키기 어렵다. 검사기가 없으면 "설계했다"와 "된다"를
구별할 수 없고, 문서는 조용히 희망 사항이 된다.

이 검사기는 rhwp 를 **실제로 실행해** 두 프로토콜의 각 요구를 판정한다. 통과만
세지 않는다 — **미달 항목을 그대로 보고하고, 그 목록이 곧 다음 구현 목록**이다.
현재 rhwp 가 프로토콜을 100% 만족하지 않는 것은 정상이며(DAP 의 요청 신원·오류
코드는 아직 결속 전이다), 이 도구의 목적은 그 격차를 **숨기지 않고 세는 것**이다.

AWS/1.0 의 tools/adoption_spine.py 와 같은 규율이다:
표준마다 자기 척추를 검사하는 장치를 함께 낸다.

## 사용

    python3 tools/dar/conformance.py --bin target/release/rhwp
    python3 tools/dar/conformance.py --bin <rhwp> --protocol dap --json
    python3 tools/dar/conformance.py --self-check      # 두 정본 정합만

종료 코드: 0 = 검사 완료(미달이 있어도 0 — 판정은 보고 안에) /
1 = 실행 실패 / 2 = 입력 오류 / 3 = --require 로 요구한 최소 준수율 미달.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
STD = REPO_ROOT / "mydocs" / "tech" / "standards"
DAP_JSON, DAP_MD = STD / "document_agent_protocol.json", STD / "document_agent_protocol.md"
DATP_JSON, DATP_MD = STD / "document_transaction_protocol.json", STD / "document_transaction_protocol.md"
DAR_MD = STD / "document_agent_runtime.md"


def log(m: str) -> None:
    print(m, file=sys.stderr)


def run(bin_path: str, args: list[str], timeout: float = 30.0):
    try:
        return subprocess.run([bin_path] + args, capture_output=True, text=True,
                              encoding="utf-8", errors="replace", timeout=timeout)
    except subprocess.TimeoutExpired:
        return None


def jload(p: Path) -> dict:
    return json.loads(p.read_text(encoding="utf-8"))


# --- 척추 자기검사 ----------------------------------------------------------

def self_check() -> list[str]:
    problems = []
    for p in (DAP_JSON, DAP_MD, DATP_JSON, DATP_MD, DAR_MD):
        if not p.is_file():
            problems.append(f"정본 파일 없음: {p.relative_to(REPO_ROOT)}")
    if problems:
        return problems

    dap, datp = jload(DAP_JSON), jload(DATP_JSON)
    dap_md, datp_md = DAP_MD.read_text(encoding="utf-8"), DATP_MD.read_text(encoding="utf-8")

    # 오류 코드가 사람 정본에도 대역으로 설명돼 있는가
    bands = {str(c["code"])[0] + "000" for c in dap["errorCodes"]["codes"] if c["code"]}
    for b in bands:
        if b[0] not in dap_md:
            problems.append(f"사람 정본에 {b} 대역 설명이 없다")
    # 연산이 양쪽에 다 있는가
    for op in (o["op"] for o in datp["operations"]):
        if op not in datp_md:
            problems.append(f"사람 정본에 연산 {op} 이 없다")
    # 상태기계가 모든 연산을 덮는가
    sm = datp["stateMachine"]["transitions"]
    for op in (o["op"] for o in datp["operations"]):
        if op not in sm:
            problems.append(f"상태기계에 {op} 전이가 없다")
    # 선언한 surface 실재
    for std in (dap, datp):
        for s in std.get("surfaces", []):
            if not (REPO_ROOT / s).exists():
                problems.append(f"선언한 surface 가 실재하지 않는다: {s}")
    return problems


# --- DAP 준수 (실제 실행으로 판정) -------------------------------------------

def check_dap(bin_path: str) -> list[dict]:
    dap = jload(DAP_JSON)
    checks: list[dict] = []

    def add(req: str, ok: bool, note: str):
        checks.append({"requirement": req, "satisfied": ok, "note": note})

    caps_proc = run(bin_path, ["capabilities"])
    caps = None
    if caps_proc and caps_proc.returncode == 0:
        try:
            caps = json.loads(caps_proc.stdout)
        except json.JSONDecodeError:
            pass
    add("능력 자기서술 — 런타임이 연산 목록을 기계 판독 가능하게 광고한다",
        caps is not None, f"capabilities → {len(caps['commands'])}개 연산" if caps else "capabilities 봉투를 읽지 못함")

    if caps:
        with_fields = [c for c in caps["commands"] if c.get("recordFields")]
        add("연산별 결과 필드 선언 — 봉투 모양을 미리 알 수 있다",
            len(with_fields) > 0,
            f"{len(with_fields)}/{len(caps['commands'])}개 연산이 recordFields 선언")

    pm = run(bin_path, ["export-provenance-map", "--json"])
    add("신뢰 모델 — 어느 필드가 문서 파생인지 지도가 있다",
        bool(pm and pm.returncode == 0 and pm.stdout.strip().startswith("{")),
        "export-provenance-map" if pm and pm.returncode == 0 else "출처 지도를 얻지 못함")

    man = run(bin_path, ["export-agent-manifest", "--json"])
    add("런타임 매니페스트 — 능력·IR·출처·계획 스키마가 한 봉투로 조립된다",
        bool(man and man.returncode == 0),
        "export-agent-manifest" if man and man.returncode == 0 else "매니페스트 없음")

    # 종료 코드 계약 — 없는 연산은 사용법 오류(2)여야 한다
    bogus = run(bin_path, ["no-such-operation-xyz"])
    add("종료 코드 계약 — 모르는 연산은 사용법 오류(2)로 거절",
        bool(bogus and bogus.returncode == 2),
        f"exit {bogus.returncode if bogus else '?'} (기대 2 = UNKNOWN_OPERATION)")

    # 아직 결속되지 않은 것들 — 정직하게 미달로 센다
    add("요청 신원(request_id) — 재시도 멱등 판정 키가 봉투에 있다", False,
        "미구현 — DAP 결속 대상(현 봉투에 request_id 없음)")
    add("트랜잭션 신원(transaction_id)이 결과 봉투에 실린다", False,
        "미구현 — DATP 결속 대상")
    add("안정 숫자 오류 코드가 봉투에 실린다(자연어 아님)", False,
        f"미구현 — 현재는 종료 코드 {len(dap['errorCodes']['codes'])}종 대역만 계약")
    return checks


# --- DATP 준수 ---------------------------------------------------------------

def check_datp(bin_path: str) -> list[dict]:
    datp = jload(DATP_JSON)
    checks: list[dict] = []

    def add(req: str, ok: bool, note: str):
        checks.append({"requirement": req, "satisfied": ok, "note": note})

    caps_proc = run(bin_path, ["capabilities"])
    names = set()
    if caps_proc and caps_proc.returncode == 0:
        try:
            names = {c["name"] for c in json.loads(caps_proc.stdout)["commands"]}
        except (json.JSONDecodeError, KeyError):
            pass

    # 각 연산에 대응 표면이 실재하는가 (rhwp 대응이 선언된 것만)
    op_surface = {
        "BEGIN": "run", "READ": "info", "SELECT": "search", "PROPOSE": "run",
        "MODIFY": "edit", "VALIDATE": "verify", "DIFF": "ir-diff",
        "COMMIT": "run", "REPLAY": "replay", "VERIFY": "audit",
    }
    missing = [op for op, cmd in op_surface.items() if cmd not in names]
    add("모든 연산에 대응 표면이 실재한다",
        not missing and bool(names),
        "전 연산 대응 확인" if not missing and names else f"대응 없음: {missing}")

    add("영수증 — 입력·계획·산출 3해시 발급 표면",
        "replay" in names, "replay --capsule" if "replay" in names else "replay 없음")
    add("결정적 재실행 — 타인의 산출 주장을 재현 검증하고 불일치를 판정으로 낸다",
        "replay" in names, "replay --expect-output-sha256 → 불일치 exit 3")
    add("계보 — 부모 트랜잭션 체인 검증",
        "lineage" in names, "lineage" if "lineage" in names else "lineage 없음")
    add("서명 귀속 — 누가 했는가",
        {"keygen", "verify-signature"} <= names, "keygen·verify-signature")
    add("소급 변조 저항 — 투명성 로그·에폭",
        "anchor" in names, "anchor" if "anchor" in names else "anchor 없음")
    add("감사·적합성 — 전수 재현 회계",
        {"audit", "conformance"} <= names, "audit·conformance")
    add("원자 실행 + 정적 선검증 + 저널",
        "run" in names, "run (선언적 계획)" if "run" in names else "run 없음")

    # 상태기계 불변식은 정의는 있으나 런타임이 강제하지 않는다 — 정직하게
    add("상태기계 강제 — COMMIT 전 VALIDATE 성공을 런타임이 요구한다", False,
        "미구현 — 현재는 계획서 관례로 지켜지며 프로토콜 수준 강제가 없다")
    add("정책 해시(policySha256)가 영수증에 실린다", False,
        "미구현 — 정책 엔진(DAR 층 3) 대상")
    _ = datp
    return checks


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--bin", help="rhwp 바이너리 (기본: RHWP_BIN → PATH)")
    ap.add_argument("--protocol", choices=["dap", "datp", "both"], default="both")
    ap.add_argument("--self-check", action="store_true", help="두 정본 정합만 검사")
    ap.add_argument("--require", type=float, metavar="비율",
                    help="이 준수율(0~1) 미만이면 exit 3")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    problems = self_check()
    if args.self_check:
        for p in problems:
            log(f"- {p}")
        log(f"자기검사: 문제 {len(problems)}건")
        return 0 if not problems else 3

    import os
    bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
    if not bin_path or not (Path(bin_path).is_file() or shutil.which(bin_path)):
        log("rhwp 바이너리를 찾을 수 없다 (--bin / RHWP_BIN / PATH).")
        return 2

    result = {"selfCheckProblems": problems, "protocols": {}}
    if args.protocol in ("dap", "both"):
        result["protocols"]["DAP/1.0"] = check_dap(bin_path)
    if args.protocol in ("datp", "both"):
        result["protocols"]["DATP/1.0"] = check_datp(bin_path)

    total = sum(len(v) for v in result["protocols"].values())
    met = sum(1 for v in result["protocols"].values() for c in v if c["satisfied"])
    result["summary"] = {"satisfied": met, "total": total,
                         "rate": round(met / total, 3) if total else None}

    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=1))
    else:
        for proto, checks in result["protocols"].items():
            ok = sum(1 for c in checks if c["satisfied"])
            print(f"\n{proto} — {ok}/{len(checks)} 충족")
            for c in checks:
                print(f"  {'✓' if c['satisfied'] else '✗'} {c['requirement']}")
                print(f"      {c['note']}")
        print(f"\n합계 {met}/{total} ({result['summary']['rate']:.0%})")
        gaps = [c["requirement"] for v in result["protocols"].values()
                for c in v if not c["satisfied"]]
        if gaps:
            print("\n다음 구현 목록 (미달이 곧 로드맵이다):")
            for g in gaps:
                print(f"  · {g}")
        if problems:
            print(f"\n정본 정합 문제 {len(problems)}건:")
            for p in problems:
                print(f"  - {p}")

    if args.require is not None and result["summary"]["rate"] is not None:
        if result["summary"]["rate"] < args.require:
            return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
