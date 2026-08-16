#!/usr/bin/env python3
"""검증 실패 → 진단 → 수리 → 재검증 반복 루프 — 기존 CLI 판정 primitive를 그대로 부린다.

## 왜 있는가 (닫히지 않은 고리)

이 저장소는 이미 실패를 **데이터**로 돌려주는 판정 도구가 여럿이다: `verify`
(`--expect-*` 게이트), `edit <서브커맨드> --verify`(저장 직후 재파싱 자기검증),
`render-diff`(라운드트립 기하 변위), `ir-diff`(구조 차이). 전부 `--json` 이면
종료 코드와 구조화된 실패 사유를 낸다. 그런데 **"검증 실패 → 원인 진단 → 수정
시도 → 재검증"을 자동으로 반복하는 오케스트레이션이 없다** — 실패가 나면 매번
사람/에이전트가 원인을 읽고 다음 시도를 손으로 판단해야 했다.

이 도구는 그 오케스트레이션만 맡는다. 판정 primitive를 재구현하지 않고 그대로
서브프로세스로 부른다 — 이 도구가 하는 일은 순전히 **locate(실행) → diagnose
(JSON 파싱·분류) → repair(재시도 인자 생성) → re-verify(다시 실행)** 뿐이다.

## 안전장치 (절대 무한루프 없음)

1. **`--max-attempts`** — 진단 실행 횟수의 하드 캡. 다 쓰면 즉시 사람에게 인계한다.
2. **진전 판정(progress)** — 이번 시도의 실패 판정 시그니처(종류+핵심 값)가 바로
   전 시도와 **똑같으면** 더 시도하지 않는다. 수리를 적용했는데도 증상이 그대로면
   이 수리 전략은 이 사건에 안 듣는다는 뜻이지, 운이 나빴을 뿐이 아니다.
3. **loop detection** — 위 진전 판정과 같은 메커니즘이다: "같은 실패가 반복된다"와
   "진전이 없다"는 이 도구에서 하나의 불변식(시그니처 재발)으로 정의된다. 재발을
   감지한 그 즉시(다음 수리를 한 번 더 시도하지 않고) 중단한다.
4. **수리 전략이 없으면 첫 시도에서 바로 인계** — 진단된 실패 종류에 등록된 수리
   함수가 없으면(`REPAIRS` 에 없음) 시도 횟수를 소모하지 않고 즉시 멈춘다. 모르는
   실패를 아는 척 재시도하지 않는다.

모든 시도는 `attempts[]` 로 남고(`--json`/`--log`), 사후에 "무엇을 왜 멈췄는지"를
그대로 재구성할 수 있다.

## 진단 종류와 수리 전략

`diagnose()` 는 4개 판정 명령(`verify`/`edit --verify`/`render-diff`/`ir-diff`)과
`run`(계획 실행, CAS 전제조건)의 JSON 봉투를 읽어 실패를 종류(`category`)로 분류한다.
분류는 전부 하되, **실제 수리는 2종만 구현**하고 나머지는 진단만 하고 사람에게
넘기는 확장 지점으로 둔다(`REPAIRS` 에 핸들러를 추가하면 확장된다):

- `casStalePrecondition` (`run` 이 `invalid[].code == "preconditionFailed"` 로
  거절 — 계획 수립 후 입력 문서가 바뀜): **구현됨** — 계획 사본에
  `preconditions.inputSha256` 을 방금 관측한 `actual` 값으로 재기록해 재시도한다.
  원본 계획 파일은 건드리지 않는다(수리 산출물은 `--work-dir` 아래 사본).
- `renderDisplacementOver` (`render-diff` 라운드트립이 임계 초과 `OVER`):
  **구현됨** — `--via hwpx`/`--via hwp` 를 서로 바꿔 다른 라운드트립 경로로
  재시도한다(같은 값을 두 번 시도하지 않는다).
- 그 외(`expectMismatch`·`editReparseMismatch`·`irStructuralDiff`·
  `renderStructMismatch`·`renderPairMismatch`·`planInvalid`·`usageError`·
  `runtimeError`·`unparseable`·`timeout`): 진단만 하고 첫 시도에서 바로 인계한다.

## 사용

    python tools/repair_loop/loop.py --bin target/release/rhwp \\
        --max-attempts 5 --json -- run plan.json --json

    python tools/repair_loop/loop.py --bin <rhwp> --json -- \\
        render-diff samples/x.hwp --json

`--` 뒤는 rhwp 서브커맨드 인자를 그대로 붙인다(반드시 `--json` 포함 — 판정을
구조화된 JSON 으로만 신뢰한다). 검사 종류는 첫 인자로 자동 판별하되(`--kind` 로
override 가능), `edit ... --verify` 는 `edit` 뒤에 `--verify` 가 있어야 인식한다.

종료 코드: 0 = 최종 재검증 통과 / 1 = 실행 오류(바이너리 없음·JSON 파싱 불가·
타임아웃) / 2 = 이 도구 자체의 사용법 오류 / 3 = 검증 실패로 사람에게 인계
(`summary.haltReason` 으로 사유 구분: `noRepairStrategy`·`noProgress`·
`maxAttemptsReached`).
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable, Optional

REPO_ROOT = Path(__file__).resolve().parents[2]


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


# ---------------------------------------------------------------------------
# locate — rhwp 서브커맨드를 그대로 실행한다
# ---------------------------------------------------------------------------

def _bin_argv(bin_path: str) -> list[str]:
    """`--bin` 은 보통 실행파일 경로 하나지만, 시험용으로 `<파이썬> <스텁.py>` 같은
    다중 토큰 접두어도 허용한다(공백 포함 경로는 직접 따옴표로 감싼다)."""
    return shlex.split(bin_path, posix=(os.name != "nt"))


def run_check(bin_path: str, check_args: list[str], timeout: float) -> dict:
    """검사 명령을 실행해 (종료 코드·JSON 봉투) 를 얻는다. 판정은 여기서 내리지
    않는다 — diagnose() 가 이 결과만 보고 분류한다."""
    cmd = _bin_argv(bin_path) + check_args
    try:
        p = subprocess.run(
            cmd, capture_output=True, text=True, encoding="utf-8",
            errors="replace", timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return {"returncode": None, "envelope": None, "stdout": "", "stderr": "timeout",
                "timedOut": True}
    except OSError as e:
        return {"returncode": None, "envelope": None, "stdout": "", "stderr": str(e),
                "timedOut": False, "spawnError": True}
    envelope = None
    if "--json" in check_args:
        try:
            envelope = json.loads(p.stdout)
        except json.JSONDecodeError:
            envelope = None
    return {"returncode": p.returncode, "envelope": envelope, "stdout": p.stdout,
            "stderr": p.stderr, "timedOut": False}


def detect_kind(check_args: list[str]) -> str:
    if not check_args:
        return "unknown"
    head = check_args[0]
    if head in ("run", "verify", "render-diff", "ir-diff"):
        return head
    if head == "edit" and "--verify" in check_args:
        return "edit-verify"
    return "unknown"


# ---------------------------------------------------------------------------
# diagnose — JSON 봉투를 실패 "종류"로 분류한다
# ---------------------------------------------------------------------------

def _diag(category: str, detail: dict, signature: tuple) -> dict:
    return {"category": category, "detail": detail, "signature": list(signature)}


def _diagnose_verify(rc: int, env: dict) -> dict:
    if rc == 3 and env.get("verdict") == "fail":
        failing = sorted(e.get("kind") for e in env.get("expectations", []) if not e.get("pass"))
        return _diag("expectMismatch",
                     {"failingKinds": failing, "failCount": env.get("failCount")},
                     ("expectMismatch", tuple(failing)))
    if rc == 2:
        return _diag("usageError", {"envelope": env}, ("usageError",))
    return _diag("runtimeError", {"returncode": rc}, ("runtimeError", rc))


def _diagnose_edit_verify(rc: int, env: dict) -> dict:
    verify = env.get("verify") or {}
    if rc == 3 and verify.get("identical") is False:
        return _diag("editReparseMismatch",
                     {"diffCount": verify.get("diffCount"), "reparseError": verify.get("reparseError")},
                     ("editReparseMismatch", verify.get("diffCount")))
    if rc == 2:
        return _diag("usageError", {"envelope": env}, ("usageError",))
    return _diag("runtimeError", {"returncode": rc}, ("runtimeError", rc))


def _diagnose_render_diff(rc: int, env: dict) -> dict:
    mode, status, via = env.get("mode"), env.get("status"), env.get("via")
    if env.get("regression") is True:
        if mode == "roundtrip" and status == "OVER":
            return _diag("renderDisplacementOver",
                         {"via": via, "maxDisp": env.get("maxDisp"), "threshold": env.get("threshold")},
                         ("renderDisplacementOver", via))
        if mode == "roundtrip" and status == "STRUCT_MISMATCH":
            return _diag("renderStructMismatch", {"via": via}, ("renderStructMismatch", via))
        if mode == "pair":
            return _diag("renderPairMismatch", {"status": status}, ("renderPairMismatch", status))
        return _diag("renderRegression", {"status": status, "mode": mode},
                     ("renderRegression", status, mode))
    if rc == 2:
        return _diag("usageError", {"envelope": env}, ("usageError",))
    return _diag("runtimeError", {"returncode": rc}, ("runtimeError", rc))


def _diagnose_ir_diff(rc: int, env: dict) -> dict:
    if rc == 3 and env.get("identical") is False:
        return _diag("irStructuralDiff",
                     {"diffCount": env.get("diffCount"), "categories": env.get("categories")},
                     ("irStructuralDiff", env.get("diffCount")))
    if rc == 2:
        return _diag("usageError", {"envelope": env}, ("usageError",))
    return _diag("runtimeError", {"returncode": rc}, ("runtimeError", rc))


def _diagnose_run(rc: int, env: dict) -> dict:
    invalid = env.get("invalid") or []
    if rc == 2 and invalid:
        first = invalid[0]
        if first.get("code") == "preconditionFailed":
            return _diag("casStalePrecondition",
                         {"expected": first.get("expected"), "actual": first.get("actual"),
                          "reason": first.get("reason")},
                         ("casStalePrecondition", first.get("expected"), first.get("actual")))
        return _diag("planInvalid", {"invalid": invalid},
                     ("planInvalid", tuple(i.get("code") for i in invalid)))
    if rc == 2:
        return _diag("usageError", {"envelope": env}, ("usageError",))
    return _diag("runtimeError", {"returncode": rc}, ("runtimeError", rc))


_DIAGNOSERS: dict[str, Callable[[int, dict], dict]] = {
    "verify": _diagnose_verify,
    "edit-verify": _diagnose_edit_verify,
    "render-diff": _diagnose_render_diff,
    "ir-diff": _diagnose_ir_diff,
    "run": _diagnose_run,
}


def diagnose(kind: str, result: dict) -> dict:
    if result.get("timedOut"):
        return _diag("timeout", {"stderr": (result.get("stderr") or "")[:500]}, ("timeout",))
    if result.get("spawnError"):
        return _diag("spawnError", {"stderr": (result.get("stderr") or "")[:500]}, ("spawnError",))
    rc = result["returncode"]
    if rc == 0:
        return _diag("pass", {}, ("pass",))
    env = result["envelope"]
    if env is None:
        return _diag("unparseable", {"returncode": rc, "stderrHead": (result.get("stderr") or "")[:500]},
                     ("unparseable", rc))
    diagnoser = _DIAGNOSERS.get(kind)
    if diagnoser is None:
        return _diag("unknownKind", {"returncode": rc}, ("unknownKind", rc))
    return diagnoser(rc, env)


# ---------------------------------------------------------------------------
# repair — 진단된 실패 종류에 대응하는 재시도 인자를 만든다 (2종만 구현)
# ---------------------------------------------------------------------------

class RepairResult:
    __slots__ = ("new_args", "note")

    def __init__(self, new_args: list[str], note: str):
        self.new_args = new_args
        self.note = note


def _repair_cas_stale_precondition(ctx: dict) -> Optional[RepairResult]:
    """CAS 전제조건 불일치 — 계획 사본에 최신 입력 해시를 채워 재계획한다.

    `rhwp run` 은 계획서의 `preconditions.inputSha256` 이 실제 입력과 다르면
    실행 0·저장 0 으로 거절하고 봉투에 `expected`/`actual` 을 낸다(#3905 CAS).
    이 수리는 그 `actual`(방금 관측한 진짜 해시)을 계획 사본에 반영해 재시도한다
    — 원본 계획 파일은 그대로 두고, `--work-dir` 아래 새 사본을 쓴다."""
    args = ctx["check_args"]
    actual = ctx["diagnosis"]["detail"].get("actual")
    if not actual:
        return None

    plan_text: Optional[str] = None
    mode: Optional[str] = None
    idx: Optional[int] = None
    for i, a in enumerate(args):
        if a == "--plan-json" and i + 1 < len(args):
            plan_text, mode, idx = args[i + 1], "inline", i + 1
            break
    if plan_text is None:
        for i, a in enumerate(args):
            if i == 0 or a.startswith("--"):
                continue
            candidate = Path(a)
            if not candidate.is_absolute():
                candidate = (REPO_ROOT / a) if not Path(a).is_file() else Path(a)
            if candidate.is_file():
                plan_text = candidate.read_text(encoding="utf-8")
                mode, idx = "path", i
                break
    if plan_text is None:
        return None

    try:
        plan = json.loads(plan_text)
    except json.JSONDecodeError:
        return None
    if not isinstance(plan.get("preconditions"), dict):
        return None

    plan["preconditions"]["inputSha256"] = actual
    new_text = json.dumps(plan, ensure_ascii=False)
    new_args = list(args)

    if mode == "inline":
        new_args[idx] = new_text
        note = f"--plan-json 인라인의 preconditions.inputSha256 을 최신 입력 해시({actual[:12]}…)로 재기록"
    else:
        work_dir: Path = ctx["work_dir"]
        work_dir.mkdir(parents=True, exist_ok=True)
        out_path = work_dir / f"plan.repair{ctx['attempt']}.json"
        out_path.write_text(new_text + "\n", encoding="utf-8")
        new_args[idx] = str(out_path)
        note = (f"계획 사본 {out_path.name} 에 preconditions.inputSha256 을 "
                f"최신 입력 해시({actual[:12]}…)로 재기록 — 원본 계획 파일은 건드리지 않음")
    return RepairResult(new_args, note)


def _repair_render_via_toggle(ctx: dict) -> Optional[RepairResult]:
    """render-diff 라운드트립 변위 초과 — 다른 라운드트립 경로(--via)로 재시도한다.

    `--via hwpx`(기본)와 `--via hwp` 는 서로 다른 저장 경로를 거치는 자기
    라운드트립이다. 한쪽 경로에서만 임계를 넘는 변위가 관측됐다면 다른 경로로
    다시 재 본다 — 같은 값을 두 번 제안하지 않는다(양쪽 다 실패하면 수리 불가로
    인계)."""
    args = ctx["check_args"]
    tried: set = ctx["state"].setdefault("triedVia", set())

    current = "hwpx"
    via_idx: Optional[int] = None
    for i, a in enumerate(args):
        if a == "--via" and i + 1 < len(args):
            current, via_idx = args[i + 1], i + 1
            break
    tried.add(current)

    candidate = "hwp" if current == "hwpx" else "hwpx"
    if candidate in tried:
        return None

    new_args = list(args)
    if via_idx is not None:
        new_args[via_idx] = candidate
    else:
        new_args = new_args + ["--via", candidate]
    return RepairResult(new_args, f"--via {current} → {candidate} 로 재시도(라운드트립 경로 전환)")


REPAIRS: dict[str, Callable[[dict], Optional[RepairResult]]] = {
    "casStalePrecondition": _repair_cas_stale_precondition,
    "renderDisplacementOver": _repair_render_via_toggle,
}


# ---------------------------------------------------------------------------
# 오케스트레이션 — locate → diagnose → repair → re-verify, 안전장치 3종 포함
# ---------------------------------------------------------------------------

def orchestrate(bin_path: str, check_args: list[str], kind: str, max_attempts: int,
                 timeout: float, work_dir: Path) -> dict:
    attempts: list[dict] = []
    state: dict = {}
    previous_signature: Optional[tuple] = None
    current_args = list(check_args)
    outcome: Optional[str] = None
    halt_reason: Optional[str] = None

    for attempt in range(1, max_attempts + 1):
        result = run_check(bin_path, current_args, timeout)
        diagnosis = diagnose(kind, result)
        record = {
            "attempt": attempt,
            "args": current_args,
            "returncode": result.get("returncode"),
            "category": diagnosis["category"],
            "detail": diagnosis["detail"],
            "repairApplied": None,
        }

        if diagnosis["category"] == "pass":
            attempts.append(record)
            outcome = "pass"
            break

        signature = tuple(diagnosis["signature"])
        if previous_signature is not None and signature == previous_signature:
            # 진전 판정이자 loop detection: 수리를 적용했는데도(또는 아무것도
            # 못 했는데도) 바로 전과 시그니처가 같다 — 더 돌리지 않는다.
            attempts.append(record)
            outcome, halt_reason = "handoff", "noProgress"
            break
        previous_signature = signature

        handler = REPAIRS.get(diagnosis["category"])
        repair = handler({"check_args": current_args, "diagnosis": diagnosis,
                          "work_dir": work_dir, "attempt": attempt, "state": state}) \
            if handler else None
        if repair is None:
            attempts.append(record)
            outcome, halt_reason = "handoff", "noRepairStrategy"
            break

        record["repairApplied"] = repair.note
        attempts.append(record)
        current_args = repair.new_args
    else:
        outcome, halt_reason = "handoff", "maxAttemptsReached"

    return {
        "schemaVersion": "1.0",
        "kind": kind,
        "bin": bin_path,
        "maxAttempts": max_attempts,
        "attempts": attempts,
        "outcome": outcome,
        "haltReason": halt_reason,
    }


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def split_args(argv: list[str]) -> tuple[list[str], list[str]]:
    if "--" in argv:
        i = argv.index("--")
        return argv[:i], argv[i + 1:]
    return argv, []


def _print_human(summary: dict) -> None:
    outcome_kr = "통과" if summary["outcome"] == "pass" else "인계"
    suffix = f" ({summary['haltReason']})" if summary.get("haltReason") else ""
    print(f"repair_loop[{summary['kind']}] — 시도 {len(summary['attempts'])}/{summary['maxAttempts']}, "
          f"결과: {outcome_kr}{suffix}")
    for a in summary["attempts"]:
        if a["category"] == "pass":
            mark = "✓"
        elif a["repairApplied"]:
            mark = "↻"
        else:
            mark = "✗"
        print(f"  {mark} 시도 {a['attempt']}: {a['category']}"
              + (f" — {a['repairApplied']}" if a["repairApplied"] else ""))


def main(argv: Optional[list[str]] = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    raw = sys.argv[1:] if argv is None else argv
    own_argv, check_args = split_args(raw)

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--bin", help="rhwp 바이너리 (기본: RHWP_BIN → PATH)")
    ap.add_argument("--kind", choices=["auto", "verify", "edit-verify", "render-diff", "ir-diff", "run"],
                     default="auto")
    ap.add_argument("--max-attempts", type=int, default=5)
    ap.add_argument("--timeout", type=float, default=30.0)
    ap.add_argument("--work-dir", help="수리 산출물(재기록된 계획 등)을 쓸 디렉터리 (기본: 임시 디렉터리)")
    ap.add_argument("--log", help="시도별 NDJSON 로그를 이어 쓸 파일 (선택)")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(own_argv)

    if not check_args:
        log("검사할 rhwp 서브커맨드가 없습니다 — `--` 뒤에 그대로 붙이세요, "
            "예: -- verify samples/x.hwp --expect-pages 3 --json")
        return 2
    if "--json" not in check_args:
        log("검사 명령에 --json 이 없습니다 — 판정은 구조화된 JSON 만 신뢰합니다.")
        return 2

    bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
    if not bin_path:
        log("rhwp 바이너리를 찾을 수 없습니다 (--bin / RHWP_BIN / PATH).")
        return 2
    probe = _bin_argv(bin_path)
    if not (Path(probe[0]).is_file() or shutil.which(probe[0])):
        log(f"rhwp 바이너리를 찾을 수 없습니다: {probe[0]}")
        return 2

    kind = args.kind if args.kind != "auto" else detect_kind(check_args)
    if kind == "unknown":
        log(f"검사 명령 종류를 판별할 수 없습니다: {' '.join(check_args)} — --kind 로 명시하세요.")
        return 2

    cleanup_tmp = None
    if args.work_dir:
        work_dir = Path(args.work_dir)
    else:
        cleanup_tmp = tempfile.mkdtemp(prefix="repair_loop_")
        work_dir = Path(cleanup_tmp)

    summary = orchestrate(bin_path, check_args, kind, args.max_attempts, args.timeout, work_dir)

    if args.log:
        log_path = Path(args.log)
        log_path.parent.mkdir(parents=True, exist_ok=True)
        with log_path.open("a", encoding="utf-8") as fh:
            for a in summary["attempts"]:
                fh.write(json.dumps({"kind": kind, **a}, ensure_ascii=False) + "\n")

    last_category = summary["attempts"][-1]["category"] if summary["attempts"] else None
    if summary["outcome"] == "pass":
        exit_code = 0
    elif last_category in ("timeout", "unparseable", "spawnError"):
        exit_code = 1
    else:
        exit_code = 3

    summary["workDir"] = str(work_dir)
    summary["exitCode"] = exit_code

    if args.json:
        print(json.dumps(summary, ensure_ascii=False, indent=1))
    else:
        _print_human(summary)

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
