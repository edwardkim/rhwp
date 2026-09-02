#!/usr/bin/env python3
"""회귀 게이트 — 한때 고쳐진 결함이 다시 살아났는지 판정한다.

## 왜 있는가

[`promote.py`](promote.py) 가 쌓은 대장(`tests/regression_corpus/ledger.json`)은
"한때 깨졌던 것"의 목록이다. 이 게이트는 그중 **고쳐진 것으로 확인된**(`guarded`)
항목을 전부 다시 돌려, 하나라도 원래 시그니처로 되살아났으면 실패한다.

이것이 고리의 마지막 마디다. 발견(fuzz_corpus)·트리아지(fde)·축소(crash_minimizer)·
승격(promote)까지 왔어도, 재발을 잡는 상시 게이트가 없으면 같은 결함이 조용히
돌아온다.

## 판정 규칙 (정직하게)

- `guarded` 항목이 **원래 시그니처로** 다시 실패 → 회귀. exit 3.
- `guarded` 항목이 **다른 시그니처로** 실패 → 회귀로 세되 별도 표기한다.
  같은 자리는 아니지만 고쳐진 것이 다시 깨진 건 사실이다.
- `open` 항목(아직 안 고쳐진 사건)은 재현돼도 **게이트를 실패시키지 않는다.**
  알려진 미수정 결함으로 CI 를 막는 것은 게이트를 무의미하게 만든다 — 대신
  보고에는 항상 싣는다.
- 픽스처가 없는 항목(최소화 불가)은 자동 판정 대상이 아니다. `skipped` 로
  세고 보고에 남긴다 — **조용히 빠뜨리지 않는다.**

## 사용

    python3 tools/regression_loop/gate.py --bin target/release/rhwp
    python3 tools/regression_loop/gate.py --bin <rhwp> --json

종료 코드: 0 = 회귀 없음 / 1 = 실행 실패 / 2 = 입력 오류 / 3 = 회귀 발견(판정).
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
PANIC_RE = re.compile(r"panicked at\s+([^\r\n:]+\.rs:\d+)")


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


def signature_of(bin_path: str, doc: Path, cmd_args: list[str], timeout: float):
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


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--bin", help="rhwp 바이너리 (기본: RHWP_BIN → PATH)")
    ap.add_argument("--timeout", type=float, default=30.0)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    if not LEDGER.is_file():
        log(f"대장이 없다 — 승격된 사건이 아직 없다: {LEDGER.relative_to(REPO_ROOT)}")
        return 0
    import os
    bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
    if not bin_path or not (Path(bin_path).is_file() or shutil.which(bin_path)):
        log("rhwp 바이너리를 찾을 수 없다 (--bin / RHWP_BIN / PATH).")
        return 2

    led = json.loads(LEDGER.read_text(encoding="utf-8"))
    regressions, still_open, skipped, held = [], [], [], []

    for e in led.get("entries", []):
        if not e.get("fixture"):
            skipped.append({**e, "why": "픽스처 없음 — 수동 절차 항목"})
            continue
        fixture = REPO_ROOT / e["fixture"]
        if not fixture.is_file():
            skipped.append({**e, "why": f"픽스처 파일 없음: {e['fixture']}"})
            continue
        sig = signature_of(bin_path, fixture, e["command"].split(), args.timeout)
        if e["status"] == "guarded":
            if sig is None:
                held.append(e["id"])
            else:
                regressions.append({
                    "id": e["id"], "expected": e["signature"], "observed": sig,
                    "sameSignature": sig == e["signature"], "command": e["command"],
                })
        elif sig is not None:
            still_open.append(e["id"])

    report = {
        "ledger": str(LEDGER.relative_to(REPO_ROOT)).replace("\\", "/"),
        "guardedHeld": held,
        "regressions": regressions,
        "stillOpen": still_open,
        "skipped": [{"id": s["id"], "why": s["why"]} for s in skipped],
    }

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=1))
    else:
        print(f"회귀 게이트 — 영구 게이트 {len(held)}건 유지, 회귀 {len(regressions)}건")
        for r in regressions:
            same = "같은 자리" if r["sameSignature"] else f"다른 자리(원래 {r['expected']})"
            print(f"  ✗ {r['id']} 회귀 — {r['observed']} [{same}]  `{r['command']}`")
        if still_open:
            print(f"  ○ 미수정 재현(게이트 미실패): {', '.join(still_open)}")
        for s in skipped:
            print(f"  – 건너뜀 {s['id']}: {s['why']}")

    return 3 if regressions else 0


if __name__ == "__main__":
    raise SystemExit(main())
