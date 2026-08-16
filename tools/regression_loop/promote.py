#!/usr/bin/env python3
"""현장 사건을 영구 회귀 게이트로 승격한다 — 한 번 고친 것이 두 번 깨지지 않게.

## 왜 있는가 (닫히지 않은 고리)

이 저장소는 결함을 **찾는** 장치가 여럿이다: `gym/tools/fuzz_corpus.py`(퍼즈 발견),
`tools/fde/triage.py`(고객 증상 트리아지), `tools/crash_minimizer.py`(최소 재현체).
고치는 경로도 있다. 그런데 **고친 뒤 그 사건이 영구 시험으로 남는 경로가 없다.**

지금의 회귀 스위트(`tests/security_corpus_regression.rs`·
`tests/convert_verify_corpus_ratchet.rs` 등)는 전부 사람이 도메인별로 손으로 쓴
것이다. 고객이 들고 온 사건 하나하나에 Rust 시험을 손으로 쓰는 사람은 없다.
그래서 사건은 처리되고 **잊힌다** — 같은 결함이 다시 들어와도 아무 게이트도
울리지 않는다.

이 도구는 그 고리를 닫는다: 사건 → (재현 확인) → 최소화 → 대장 등재 →
[수정] → (수정 확인) → **영구 게이트**. 대장은 "한때 깨졌던 것"의 목록이고
줄어들지 않는다.

## 정직 규칙 (유령 게이트를 만들지 않는다)

1. **재현되지 않는 사건은 승격하지 않는다.** 등재 전에 실제로 실패를
   재현시켜 본다. 재현 안 되면 exit 3 으로 거절한다 — 아무것도 지키지 않는
   게이트가 지키는 척하는 것이 최악이다.
2. **`open` 과 `guarded` 를 구별한다.** 등재 직후는 `open`(아직 안 고쳐짐)이고,
   게이트를 실패시키지 않는다. `--confirm-fixed` 로 **더는 재현되지 않음을
   확인해야만** `guarded` 가 되고, 그때부터 재발이 곧 회귀다.
3. 고객 문서는 대장에 싣지 않는다 — 최소화 산출물만 픽스처로 커밋하고,
   최소화가 불가능하면(HWP5 등) 시그니처와 절차만 남긴다.

## 사용

    # fde 티켓에서 승격 (권장 경로)
    python3 tools/regression_loop/promote.py --ticket ticket.json --bin <rhwp>

    # 직접 지정
    python3 tools/regression_loop/promote.py --doc bad.hwpx \
        --cmd "info {doc} --json" --bin <rhwp>

    # 수정이 머지된 뒤 — 더는 재현되지 않음을 확인하고 영구 게이트로 승격
    python3 tools/regression_loop/promote.py --confirm-fixed RG-3 --bin <rhwp>

    python3 tools/regression_loop/promote.py --list

종료 코드: 0 = 승격됨 / 1 = 실행 실패 / 2 = 입력 오류 /
3 = 판정(재현 안 됨 → 등재 거절, 또는 --confirm-fixed 인데 아직 재현됨).
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
LEDGER = REPO_ROOT / "tests" / "regression_corpus" / "ledger.json"
FIXTURES = REPO_ROOT / "tests" / "regression_corpus" / "fixtures"
MINIMIZER = REPO_ROOT / "tools" / "crash_minimizer.py"

PANIC_RE = re.compile(r"panicked at\s+([^\r\n:]+\.rs:\d+)")


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


def load_ledger() -> dict:
    if LEDGER.is_file():
        return json.loads(LEDGER.read_text(encoding="utf-8"))
    return {"schemaVersion": "1", "note": "한때 깨졌던 것들 — 줄어들지 않는다.", "entries": []}


def save_ledger(led: dict) -> None:
    LEDGER.parent.mkdir(parents=True, exist_ok=True)
    LEDGER.write_text(json.dumps(led, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")


def signature_of(bin_path: str, doc: Path, cmd_args: list[str], timeout: float):
    """실행해 실패 시그니처를 낸다. 실패가 아니면 None — triage.py 와 같은 판정 규칙."""
    cmd = [bin_path] + [a.replace("{doc}", str(doc)) for a in cmd_args]
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8",
                           errors="replace", timeout=timeout)
    except subprocess.TimeoutExpired:
        return ["timeout"]
    m = PANIC_RE.search(p.stderr or "")
    if m:
        return ["panic", m.group(1).replace("\\", "/")]
    if p.returncode < 0 or p.returncode >= 0xC0000000:
        return ["abort", p.returncode]
    return None


def minimize(doc: Path, bin_path: str, cmd_args: list[str], out: Path) -> Path | None:
    """HWPX 면 최소화한다. 실패하거나 대상이 아니면 None — 원본을 대신 싣지 않는다."""
    if doc.suffix.lower() != ".hwpx" or not MINIMIZER.is_file():
        return None
    out.parent.mkdir(parents=True, exist_ok=True)
    p = subprocess.run(
        [sys.executable, str(MINIMIZER), str(doc), "--bin", bin_path,
         "--cmd", " ".join(cmd_args), "-o", str(out)],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
    )
    return out if p.returncode == 0 and out.is_file() else None


def cmd_promote(args) -> int:
    bin_path = args.bin or shutil.which("rhwp")
    if not bin_path or not (Path(bin_path).is_file() or shutil.which(bin_path)):
        log("rhwp 바이너리를 찾을 수 없다 (--bin).")
        return 2

    if args.ticket:
        t = json.loads(Path(args.ticket).read_text(encoding="utf-8"))
        doc = Path(args.ticket).parent / Path(t["doc"]).name
        if not doc.is_file():
            doc = Path(t["doc"])
        failed = [s for s in t.get("steps", []) if "failureSignature" in s]
        if not failed:
            log("티켓에 실패 단계가 없다 — 승격할 사건이 아니다.")
            return 3
        cmd_args = failed[0]["command"].split()
        source = f"fde-ticket:{Path(args.ticket).name}"
    elif args.doc and args.cmd:
        doc, cmd_args, source = Path(args.doc), args.cmd.split(), args.source or "manual"
    else:
        log("--ticket 또는 (--doc 과 --cmd) 가 필요하다.")
        return 2

    if not doc.is_file():
        log(f"문서가 없다: {doc}")
        return 2

    # 정직 규칙 1 — 재현되지 않으면 등재하지 않는다.
    sig = signature_of(bin_path, doc, cmd_args, args.timeout)
    if sig is None:
        log("이 사건은 지금 재현되지 않는다 — 등재를 거절한다(유령 게이트 금지).")
        return 3
    log(f"재현 확인: {sig}")

    led = load_ledger()
    for e in led["entries"]:
        if e["signature"] == sig and e["command"] == " ".join(cmd_args):
            log(f"이미 대장에 있다: {e['id']} ({e['status']})")
            return 0

    rg_id = f"RG-{len(led['entries']) + 1}"
    fixture = minimize(doc, bin_path, cmd_args, FIXTURES / f"{rg_id}.hwpx")
    if fixture:
        # 최소화본이 같은 시그니처로 실패하는지 재확인 — 축소가 사건을 바꿨으면 못 쓴다.
        if signature_of(bin_path, fixture, cmd_args, args.timeout) != sig:
            log("최소화본이 다른 시그니처로 실패한다 — 픽스처를 버리고 절차만 남긴다.")
            fixture.unlink(missing_ok=True)
            fixture = None

    entry = {
        "id": rg_id,
        "status": "open",
        "signature": sig,
        "command": " ".join(cmd_args),
        "fixture": str(fixture.relative_to(REPO_ROOT)).replace("\\", "/") if fixture else None,
        "source": source,
        "note": None if fixture else "최소화 불가(HWPX 아님 또는 축소 실패) — 시그니처·절차만",
    }
    led["entries"].append(entry)
    save_ledger(led)
    log(f"승격: {rg_id} status=open"
        + (f", 픽스처 {entry['fixture']}" if fixture else " (픽스처 없음)"))
    log("수정이 머지되면 --confirm-fixed 로 guarded 로 올려라 — 그때부터 재발이 회귀다.")
    print(json.dumps(entry, ensure_ascii=False))
    return 0


def cmd_confirm_fixed(args) -> int:
    bin_path = args.bin or shutil.which("rhwp")
    if not bin_path:
        log("rhwp 바이너리를 찾을 수 없다 (--bin).")
        return 2
    led = load_ledger()
    entry = next((e for e in led["entries"] if e["id"] == args.confirm_fixed), None)
    if entry is None:
        log(f"대장에 없는 id: {args.confirm_fixed}")
        return 2
    if not entry.get("fixture"):
        log("픽스처가 없는 항목은 자동 확인할 수 없다 — 절차대로 수동 확인 후 대장을 직접 고쳐라.")
        return 2

    fixture = REPO_ROOT / entry["fixture"]
    sig = signature_of(bin_path, fixture, entry["command"].split(), args.timeout)
    if sig is not None:
        log(f"아직 재현된다({sig}) — guarded 로 올리지 않는다.")
        return 3
    entry["status"] = "guarded"
    save_ledger(led)
    log(f"{entry['id']} → guarded. 이제 재발하면 게이트가 실패한다.")
    return 0


def cmd_list(_args) -> int:
    led = load_ledger()
    if not led["entries"]:
        print("대장이 비어 있다.")
        return 0
    for e in led["entries"]:
        mark = "🛡" if e["status"] == "guarded" else "○"
        print(f"{mark} {e['id']:<7} {e['status']:<8} {str(e['signature']):<44} {e['source']}")
    guarded = sum(1 for e in led["entries"] if e["status"] == "guarded")
    print(f"\n총 {len(led['entries'])}건 · 영구 게이트 {guarded}건 · 미수정 {len(led['entries']) - guarded}건")
    return 0


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--ticket", help="tools/fde/triage.py 가 낸 티켓 JSON")
    ap.add_argument("--doc", help="재현 문서 (직접 지정)")
    ap.add_argument("--cmd", help='재현 명령 템플릿, 예: "info {doc} --json"')
    ap.add_argument("--source", help="사건 출처 표기 (기본: manual)")
    ap.add_argument("--bin", help="rhwp 바이너리")
    ap.add_argument("--timeout", type=float, default=30.0)
    ap.add_argument("--confirm-fixed", metavar="RG-N", help="수정 확인 후 guarded 로 승격")
    ap.add_argument("--list", action="store_true", help="대장 조회")
    args = ap.parse_args(argv)

    if args.list:
        return cmd_list(args)
    if args.confirm_fixed:
        return cmd_confirm_fixed(args)
    return cmd_promote(args)


if __name__ == "__main__":
    raise SystemExit(main())
