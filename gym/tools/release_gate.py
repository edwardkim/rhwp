"""[#4662/#5259] 릴리스 게이트 러너 — 회귀 도구를 파이프라인 판정으로 묶는다.

로컬에서도 CI 에서도 같은 판정을 낸다. 하는 일:

1. 두 바이너리로 릴리스 차등(#4661)을 돌려 분류를 얻는다.
2. 커밋된 리더보드가 있으면 해시 체인(#4659)을 검증한다.
3. 판정을 낸다 — **regression 만 차단**한다. surface-changed 는 리뷰 신호이지
   자동 차단이 아니다(도구는 '무엇이 바뀌었나'를 가리키지 '어느 쪽이 옳은가'를
   판정하지 않는다 — #4661 정직 조항).

판별 감사(discriminate.py) 실패는 차등 분류가 아니다. 약한 오라클은 벤치마크
자체의 결함이라 워크플로가 게이트보다 먼저 exit 1 로 막는다. 러너는 그 종료
코드를 받아도 regression(exit 3) 이나 surface-changed(exit 2) 로 위장하지 않는다.

## 종료 코드 (게이트 계약)

- 0 = pass   — 차등 stable(또는 검사 대상 없음) + 리더보드 무결
- 1 = fail   — 도구/전제 실패. 신 바이너리 부재, 차등 보고 손상, probe-failed,
               판별 감사 실패. 삼원 분류로 위장하지 않는다
- 2 = review — 차등 surface-changed. 표면이 바뀐 릴리스라 사람 판정 필요(차단 아님)
- 3 = block  — 차등 regression, 또는 리더보드 체인 파손

## GitHub 연동

`--github-summary` 를 주면 GITHUB_STEP_SUMMARY 에 마크다운 표를 쓴다. old 바이너리가
없으면(직전 태그 미빌드) 차등은 건너뛰고 리더보드 검증만 한다 — 부재를 실패로
위장하지 않는 결 그대로 skipped 로 보고한다. new 바이너리가 없으면 생략이 아니라
fail 이다. 비교 대상이 없는 것과 현재 릴리스가 없는 것은 다르다.

CLI 플래그는 그대로다. 새 pack 을 만들지 않는다.
"""

from __future__ import annotations

import argparse
import io
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))

REPORT_KIND = "gymReleaseGate"
SCHEMA_VERSION = "1.0"

#: 게이트가 내는 네 판정. fail 은 삼원(차등 분류)이 아니다.
VERDICTS = ("pass", "review", "block", "fail")
EXIT_BY_VERDICT = {"pass": 0, "review": 2, "block": 3, "fail": 1}

#: 차등 오라클의 삼원. probe-failed / skipped 는 이 튜플에 넣지 않는다.
DIFF_CLASSIFICATIONS = ("stable", "regression", "surface-changed")
DIFF_SKIPPED = "skipped"
DIFF_PROBE_FAILED = "probe-failed"

#: 게이트가 접는 이유 카탈로그. 시험·문서가 같은 표를 본다.
REASONS = (
    "stable",
    "skipped",
    "missing-old-bin",
    "missing-new-bin",
    "discriminate-fail",
    "trajectory-fail",
    "audit-fail",
    "probe-failed",
    "diff-report-missing",
    "diff-report-invalid",
    "diff-tool-error",
    "surface-changed",
    "regression",
    "leaderboard-broken",
    "leaderboard-error",
    "write-error",
    "unexpected",
)

#: 이유 → 판정. 우선순위는 decide_verdict 가 적용한다.
VERDICT_BY_REASON = {
    "stable": "pass",
    "skipped": "pass",
    "missing-old-bin": "pass",
    "missing-new-bin": "fail",
    "discriminate-fail": "fail",
    "trajectory-fail": "fail",
    "audit-fail": "fail",
    "probe-failed": "fail",
    "diff-report-missing": "fail",
    "diff-report-invalid": "fail",
    "diff-tool-error": "fail",
    "surface-changed": "review",
    "regression": "block",
    "leaderboard-broken": "block",
    "leaderboard-error": "fail",
    "write-error": "fail",
    "unexpected": "fail",
}

#: 워크플로가 게이트보다 먼저 돌리는 무결성 전제. 러너가 이 스크립트를
#: 고치지 않는다 — 종료 코드만 읽는다.
PREFLIGHT_TOOLS = ("discriminate.py", "trajectory.py")
PREFLIGHT_REASON_BY_TOOL = {
    "discriminate": "discriminate-fail",
    "discriminate.py": "discriminate-fail",
    "trajectory": "trajectory-fail",
    "trajectory.py": "trajectory-fail",
}

#: 차등 보고에서 게이트가 읽는 키. 없어도 도구를 죽이지 않는다.
DIFF_REPORT_KEYS = (
    "classification",
    "divergences",
    "surfaceChanged",
    "tasksCompared",
)

#: 판정 봉투 고정 키.
VERDICT_KEYS = (
    "kind",
    "schemaVersion",
    "diff",
    "leaderboard",
    "verdict",
    "exit",
    "reason",
    "reasons",
    "ok",
    "reviewRequired",
    "blocked",
    "failed",
    "old",
    "new",
    "preflight",
    "errors",
)

REASON_TEXT = {
    "stable": "명령 표면과 관측이 같다",
    "skipped": "구 바이너리 없음 — 차등 생략(직전 태그 미빌드)",
    "missing-old-bin": "구 바이너리 없음 — 차등 생략(직전 태그 미빌드)",
    "missing-new-bin": "신 바이너리가 없다 — 비교 대상이 아니라 현재 릴리스 부재",
    "discriminate-fail": "판별 감사가 약한 오라클을 신고했다 — 차등 분류로 위장하지 않는다",
    "trajectory-fail": "트라젝토리 감사가 연극 과제를 신고했다 — 차등 분류로 위장하지 않는다",
    "audit-fail": "무결성 전제 감사가 실패했다",
    "probe-failed": "차등이 표면을 재지 못했다 — stable/regression/surface-changed 로 위장하지 않는다",
    "diff-report-missing": "차등 보고 파일이 없다",
    "diff-report-invalid": "차등 보고를 읽지 못했다",
    "diff-tool-error": "차등 도구가 보고 없이 실패했다",
    "surface-changed": "명령 표면이 달라 사람 판정이 필요하다 — 자동 차단이 아니다",
    "regression": "명령 표면은 같고 관측이 갈렸다 — 순수 동작 변화",
    "leaderboard-broken": "리더보드 해시 체인이 파손됐다",
    "leaderboard-error": "리더보드 검증 도구가 예외로 죽었다",
    "write-error": "판정 봉투를 쓰지 못했다",
    "unexpected": "카탈로그 밖 실패",
}

#: 삼키면 안 되는 예외 — 사용자가 끊었는데 안정 보고를 내면 거짓말이다.
FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

CATCHABLE_EXCEPTIONS = (
    FileNotFoundError,
    PermissionError,
    TimeoutError,
    subprocess.TimeoutExpired,
    UnicodeError,
    ValueError,
    TypeError,
    KeyError,
    IndexError,
    AttributeError,
    OSError,
    json.JSONDecodeError,
    RuntimeError,
)

ERROR_HEAD_LIMIT = 160

EXCEPTION_KIND_BY_TYPE = {
    FileNotFoundError: "missing-bin",
    PermissionError: "permission",
    TimeoutError: "timeout",
    subprocess.TimeoutExpired: "timeout",
    UnicodeError: "decode-error",
    UnicodeDecodeError: "decode-error",
    UnicodeEncodeError: "decode-error",
    ValueError: "value-error",
    TypeError: "type-error",
    KeyError: "key-error",
    IndexError: "index-error",
    AttributeError: "attr-error",
    OSError: "os-error",
    json.JSONDecodeError: "invalid-json",
    RuntimeError: "runtime-error",
}


def is_fatal_exception(exc):
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def truncate_head(text, limit=ERROR_HEAD_LIMIT):
    """오류 머리. None/비문자는 빈 문자열. 한도는 0 이하면 빈 값."""
    if text is None:
        return ""
    if not isinstance(text, str):
        try:
            text = str(text)
        except FATAL_EXCEPTIONS:
            raise
        except Exception:
            return ""
    if limit is None or limit <= 0:
        return ""
    if len(text) <= limit:
        return text
    return text[:limit]


def exception_kind(exc, context="gate"):
    """예외를 이유/오류 kind 로 접는다. 순수.

    context:
      - bin: 바이너리 존재 확인. FileNotFound → missing-bin
      - diff-report: 차등 JSON. JSONDecodeError → invalid-json
      - write: 판정 쓰기
      - tool: 서브프로세스
      - gate: 그 외
    """
    if exc is None:
        return "unexpected"
    if isinstance(exc, subprocess.TimeoutExpired) or isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, FileNotFoundError):
        if context == "diff-report":
            return "diff-report-missing"
        if context == "bin":
            return "missing-bin"
        return "missing-bin"
    if isinstance(exc, PermissionError):
        return "permission"
    if isinstance(exc, UnicodeError):
        return "decode-error"
    if isinstance(exc, json.JSONDecodeError):
        return "invalid-json"
    if isinstance(exc, (KeyError, IndexError, AttributeError)):
        return "type-error" if isinstance(exc, AttributeError) else (
            "key-error" if isinstance(exc, KeyError) else "index-error"
        )
    if isinstance(exc, TypeError):
        return "type-error"
    if isinstance(exc, ValueError):
        return "value-error"
    if isinstance(exc, OSError):
        return "os-error"
    if isinstance(exc, RuntimeError):
        return "runtime-error"
    return "unexpected"


def exception_record(exc, context="gate", path=""):
    """예외를 오류 한 줄로 접는다. 여기서 예외를 다시 올리지 않는다."""
    return {
        "context": context,
        "kind": exception_kind(exc, context=context),
        "error": type(exc).__name__ if exc is not None else "NoneType",
        "head": truncate_head(str(exc) if exc is not None else ""),
        "path": path or "",
    }


def normalize_tool_name(name):
    """discriminate.py / discriminate / gym/tools/discriminate.py → discriminate."""
    if not name:
        return ""
    base = os.path.basename(str(name).replace("\\", "/"))
    lower = base.lower()
    if lower.endswith(".py"):
        lower = lower[:-3]
    return lower


def reason_for_audit(tool, code):
    """전제 감사 종료 코드를 이유 코드로 접는다. 0 이면 None. 순수.

    discriminate 실패는 regression 이 아니다. 오라클이 약한 것이지 바이너리
    관측이 갈린 것이 아니다. 게이트는 이 값을 fail(1) 로만 묶는다.
    """
    try:
        code_n = int(code)
    except (TypeError, ValueError):
        return "audit-fail"
    if code_n == 0:
        return None
    name = normalize_tool_name(tool)
    if name == "discriminate":
        return "discriminate-fail"
    if name == "trajectory":
        return "trajectory-fail"
    return "audit-fail"


def fold_preflight(preflight):
    """전제 감사 입력을 정규화한다. 순수.

    받는 것:
      - None / [] / {} → 전제 없음
      - {tool, exit} 한 줄
      - [{tool, exit}, ...]
      - {audits: [...]} / {ok: False, reason: "..."}

    내는 것: {ok, audits, reasons, failed}
    """
    empty = {"ok": True, "audits": [], "reasons": [], "failed": False}
    if preflight is None:
        return empty
    rows = []
    if isinstance(preflight, dict):
        if "audits" in preflight and isinstance(preflight.get("audits"), list):
            rows = list(preflight["audits"])
        elif "tool" in preflight or "exit" in preflight:
            rows = [preflight]
        elif preflight.get("ok") is False:
            reason = preflight.get("reason") or "audit-fail"
            if reason not in REASONS:
                reason = "audit-fail"
            return {
                "ok": False,
                "audits": [dict(preflight)],
                "reasons": [reason],
                "failed": True,
            }
        else:
            return empty
    elif isinstance(preflight, (list, tuple)):
        rows = list(preflight)
    else:
        return {
            "ok": False,
            "audits": [{"tool": "?", "exit": 1, "raw": truncate_head(str(preflight), 80)}],
            "reasons": ["audit-fail"],
            "failed": True,
        }

    audits = []
    reasons = []
    for row in rows:
        if not isinstance(row, dict):
            reasons.append("audit-fail")
            audits.append({"tool": "?", "exit": 1, "ok": False})
            continue
        tool = row.get("tool") or row.get("name") or ""
        code = row.get("exit")
        if code is None:
            code = 0 if row.get("ok") else 1
        reason = reason_for_audit(tool, code)
        item = {
            "tool": normalize_tool_name(tool) or tool or "?",
            "exit": code,
            "ok": reason is None,
        }
        if reason:
            item["reason"] = reason
            reasons.append(reason)
        audits.append(item)
    return {
        "ok": not reasons,
        "audits": audits,
        "reasons": reasons,
        "failed": bool(reasons),
    }


def map_diff_classification(classification, surface_changed=None, divergences=None):
    """차등 분류를 게이트 이유로 접는다. 순수.

    surface-changed 는 관측 분기 유무와 무관하게 review 다.
    regression 은 표면이 같을 때만 block 이다.
    probe-failed / 알 수 없는 값은 fail 이지 pass 가 아니다.
    """
    cls = classification
    if cls is None:
        return "diff-report-invalid"
    if not isinstance(cls, str):
        return "diff-report-invalid"
    cls = cls.strip()
    if cls == DIFF_SKIPPED:
        return "missing-old-bin"
    if cls == DIFF_PROBE_FAILED:
        return "probe-failed"
    if cls == "surface-changed":
        return "surface-changed"
    if cls == "regression":
        return "regression"
    if cls == "stable":
        return "stable"
    if cls == "":
        return "diff-report-invalid"
    return "unexpected"


def surface_wins_over_regression(classification, surface_changed, divergences):
    """정직 조항 — 표면이 바뀌면 관측 분기를 회귀로 부르지 않는다. 순수.

    차등 오라클이 이미 이 규칙을 지키지만, 게이트가 보고를 다시 읽을 때도
    같은 규칙을 적용한다. 오라클이 실수로 regression + surfaceChanged=True 를
    내도 게이트는 review 다.
    """
    if classification == "surface-changed":
        return "surface-changed"
    if surface_changed and classification == "regression":
        return "surface-changed"
    if surface_changed and classification == "stable":
        return "surface-changed"
    return map_diff_classification(classification, surface_changed, divergences)


def decide_verdict(diff_reason, board_ok, preflight_reasons=None):
    """이유들을 하나의 판정으로 접는다. 순수.

    우선순위(앞이 이긴다):
      1. 전제 감사 실패 (discriminate-fail 등) → fail
      2. 신 바이너리 부재·도구 실패 → fail
      3. regression 또는 리더보드 파손 → block
      4. surface-changed → review
      5. 그 외(stable / skipped / missing-old) → pass

    리더보드 파손은 표면 변경보다 앞선다. 원장 무결은 사람 리뷰로 넘기지 않는다.
    """
    reasons = []
    if preflight_reasons:
        for r in preflight_reasons:
            if r:
                reasons.append(r)
    if diff_reason:
        reasons.append(diff_reason)
    if board_ok is False:
        reasons.append("leaderboard-broken")

    rank = {
        "discriminate-fail": 0,
        "trajectory-fail": 0,
        "audit-fail": 0,
        "missing-new-bin": 1,
        "probe-failed": 1,
        "diff-report-missing": 1,
        "diff-report-invalid": 1,
        "diff-tool-error": 1,
        "leaderboard-error": 1,
        "write-error": 1,
        "unexpected": 1,
        "regression": 2,
        "leaderboard-broken": 2,
        "surface-changed": 3,
        "stable": 4,
        "skipped": 4,
        "missing-old-bin": 4,
    }
    if not reasons:
        reasons = ["stable"]
    winner = min(reasons, key=lambda r: rank.get(r, 1))
    verdict = VERDICT_BY_REASON.get(winner, "fail")
    return {
        "verdict": verdict,
        "exit": EXIT_BY_VERDICT[verdict],
        "reason": winner,
        "reasons": list(reasons),
        "ok": verdict == "pass",
        "reviewRequired": verdict == "review",
        "blocked": verdict == "block",
        "failed": verdict == "fail",
    }


def empty_bin_record(role, given=None, status="missing", reason="empty"):
    return {
        "role": role,
        "given": given,
        "resolved": None,
        "status": status,
        "reason": reason,
    }


def find_bin_safe(path):
    """runner.find_bin 을 예외 없이 접는다."""
    try:
        found = runner.find_bin(path)
        return found, None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return path, exception_record(exc, context="bin", path=str(path) if path else "")
    except Exception as exc:
        return path, exception_record(exc, context="bin", path=str(path) if path else "")


def path_exists_safe(path):
    """os.path.exists 를 예외 없이 접는다. 모르면 없는 것으로 본다."""
    if not path:
        return False
    try:
        return os.path.exists(path)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS:
        return False
    except Exception:
        return False


def resolve_bin_record(path, role):
    """바이너리 한 쪽의 존재 기록. 예외로 도구를 죽이지 않는다."""
    if path is None:
        return empty_bin_record(role, given=None, status="omitted", reason="empty")
    if isinstance(path, str) and not path.strip():
        return empty_bin_record(role, given=path, status="omitted", reason="empty")
    resolved, err = find_bin_safe(path)
    if err:
        rec = empty_bin_record(role, given=path, status="error", reason=err.get("kind", "os-error"))
        rec["resolved"] = resolved
        rec["error"] = err
        return rec
    rec = {
        "role": role,
        "given": path,
        "resolved": resolved,
        "status": "present" if path_exists_safe(resolved) else "missing",
        "reason": "ok" if path_exists_safe(resolved) else "not-found",
    }
    return rec


def run_tool(script, args):
    """gym/tools 의 다른 러너를 서브프로세스로 부른다 — (exit, stdout)."""
    proc = subprocess.run([sys.executable, os.path.join(HERE, script)] + args,
                          cwd=runner.ROOT, capture_output=True)
    return proc.returncode, proc.stdout.decode("utf-8", errors="replace")


def run_tool_safe(script, args):
    """run_tool 예외를 접는다. (exit, stdout, error_or_None)."""
    try:
        code, out = run_tool(script, args)
        return code, out, None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return 1, "", exception_record(exc, context="tool", path=script)
    except Exception as exc:
        return 1, "", exception_record(exc, context="tool", path=script)


def load_json_safe(path, context="diff-report"):
    """JSON 파일을 예외 없이 읽는다. (obj, error_or_None)."""
    if not path:
        return None, {"kind": "diff-report-missing", "error": "empty-path", "head": "", "path": ""}
    try:
        if not os.path.exists(path):
            return None, {
                "kind": "diff-report-missing",
                "error": "FileNotFoundError",
                "head": path,
                "path": path,
            }
        with io.open(path, encoding="utf-8") as fh:
            return json.load(fh), None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return None, exception_record(exc, context=context, path=path)
    except Exception as exc:
        return None, exception_record(exc, context=context, path=path)


def remove_safe(path):
    """임시 보고 삭제. 실패해도 판정을 바꾸지 않는다."""
    if not path:
        return None
    try:
        if os.path.exists(path):
            os.remove(path)
        return None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return exception_record(exc, context="write", path=path)
    except Exception as exc:
        return exception_record(exc, context="write", path=path)


def extract_diff_fields(report):
    """차등 보고에서 게이트가 쓰는 칸만 꺼낸다. 순수.

    키가 없어도 도구를 죽이지 않는다. 분류가 없으면 invalid.
    """
    if not isinstance(report, dict):
        return None, {"kind": "diff-report-invalid", "error": "not-dict", "head": "", "path": ""}
    classification = report.get("classification")
    fields = {
        "classification": classification,
        "divergences": report.get("divergences"),
        "surfaceChanged": report.get("surfaceChanged"),
        "tasksCompared": report.get("tasksCompared"),
    }
    if "classificationReason" in report:
        fields["classificationReason"] = report["classificationReason"]
    if "probeFailed" in report:
        fields["probeFailed"] = report["probeFailed"]
    if "exit" in report:
        fields["toolExit"] = report["exit"]
    if classification is None:
        return fields, {"kind": "diff-report-invalid", "error": "no-classification", "head": "", "path": ""}
    return fields, None


def skipped_diff(reason="missing-old-bin"):
    return {
        "classification": DIFF_SKIPPED,
        "reason": REASON_TEXT.get(reason, REASON_TEXT["skipped"]),
        "reasonCode": reason,
    }


def failed_diff(reason, extra=None):
    row = {
        "classification": reason if reason == DIFF_PROBE_FAILED else "unavailable",
        "reason": REASON_TEXT.get(reason, reason),
        "reasonCode": reason,
    }
    if extra:
        row.update(extra)
    return row


def ledger_path():
    return os.path.join(runner.GYM, "leaderboard", "ledger.ndjson")


def run_release_diff(old_bin, new_bin, agent, packs):
    """차등 도구를 부르고 보고를 읽는다. 예외로 게이트를 죽이지 않는다."""
    out = os.path.join(runner.GYM, "release-gate-diff.json")
    args = ["--old", str(old_bin), "--new", str(new_bin), "--agent", agent or "claude-fable-5",
            "-o", out]
    for p in packs or []:
        args += ["--pack", p]
    code, _stdout, tool_err = run_tool_safe("release_diff.py", args)
    report, load_err = load_json_safe(out, context="diff-report")
    remove_safe(out)
    if tool_err and report is None:
        return failed_diff("diff-tool-error", {"toolError": tool_err, "toolExit": code}), "diff-tool-error"
    if load_err and report is None:
        kind = load_err.get("kind")
        reason = "diff-report-missing" if kind in ("diff-report-missing", "missing-bin") else "diff-report-invalid"
        return failed_diff(reason, {"loadError": load_err, "toolExit": code}), reason
    fields, field_err = extract_diff_fields(report)
    if field_err:
        return failed_diff("diff-report-invalid", {"loadError": field_err, "toolExit": code}), "diff-report-invalid"
    reason = surface_wins_over_regression(
        fields.get("classification"),
        fields.get("surfaceChanged"),
        fields.get("divergences"),
    )
    fields["reasonCode"] = reason
    fields["toolExit"] = code
    return fields, reason


def run_leaderboard(new_bin):
    """리더보드 verify. 예외로 게이트를 죽이지 않는다."""
    code, out, err = run_tool_safe("leaderboard.py", ["--bin", new_bin, "verify"])
    if err:
        return {"ok": False, "exit": code, "error": err, "reasonCode": "leaderboard-error"}
    return {"ok": code == 0, "exit": code, "reasonCode": None if code == 0 else "leaderboard-broken"}


def empty_verdict():
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "diff": None,
        "leaderboard": None,
        "verdict": "fail",
        "exit": 1,
        "reason": "unexpected",
        "reasons": [],
        "ok": False,
        "reviewRequired": False,
        "blocked": False,
        "failed": True,
        "old": empty_bin_record("old"),
        "new": empty_bin_record("new"),
        "preflight": fold_preflight(None),
        "errors": [],
    }


def attach_decision(verdict, decision):
    verdict.update(decision)
    return verdict


def gate(old_bin, new_bin, agent, packs, verify_board, preflight=None):
    """게이트 본체. 기존 위치 인자는 그대로다. preflight 는 선택.

    preflight 는 CLI 플래그가 아니다. 워크플로가 이미 discriminate.py 를
    게이트보다 먼저 돌리므로 main() 은 이 인자를 쓰지 않는다. 시험과
    프로그래매틱 호출이 판별 실패를 차등 분류로 위장하지 않는지 고정한다.
    """
    verdict = empty_verdict()
    errors = []
    pre = fold_preflight(preflight)
    verdict["preflight"] = pre

    old_rec = resolve_bin_record(old_bin, "old")
    new_rec = resolve_bin_record(new_bin, "new")
    verdict["old"] = old_rec
    verdict["new"] = new_rec

    if new_rec.get("status") != "present":
        verdict["diff"] = skipped_diff("missing-new-bin")
        verdict["diff"]["classification"] = "unavailable"
        verdict["leaderboard"] = {"ok": None, "reason": REASON_TEXT["missing-new-bin"]}
        if new_rec.get("error"):
            errors.append(new_rec["error"])
        verdict["errors"] = errors
        return attach_decision(verdict, decide_verdict(
            "missing-new-bin", None, pre["reasons"]))

    if pre["failed"]:
        # 전제가 무너지면 차등을 돌려도 오라클을 믿을 수 없다. 그래도 신
        # 바이너리는 있으므로 리더보드만 선택적으로 본다 — 파손이면 이유도
        # 남긴다. 판정은 전제 실패(fail)가 이긴다.
        if old_rec.get("status") == "present":
            verdict["diff"] = skipped_diff(pre["reasons"][0] if pre["reasons"] else "audit-fail")
            verdict["diff"]["classification"] = "unavailable"
        else:
            verdict["diff"] = skipped_diff("missing-old-bin")
        if verify_board and path_exists_safe(ledger_path()):
            # --bin 은 호출자가 준 new_bin 그대로. 해석 경로로 바꾸면
            # 기존 계약(선택한 실행 파일로 원장 검증)이 깨진다.
            board = run_leaderboard(new_bin)
            verdict["leaderboard"] = board
            board_ok = board.get("ok")
        else:
            verdict["leaderboard"] = {"ok": None, "reason": "커밋된 리더보드 없음 — 검증 생략"}
            board_ok = None
        verdict["errors"] = errors
        extra = list(pre["reasons"])
        if (verdict.get("leaderboard") or {}).get("reasonCode") == "leaderboard-error":
            extra.append("leaderboard-error")
            board_ok = None
        return attach_decision(verdict, decide_verdict(
            verdict["diff"].get("reasonCode"), board_ok, extra))

    if old_rec.get("status") == "present":
        fields, reason = run_release_diff(old_bin, new_bin, agent, packs)
        verdict["diff"] = fields
    else:
        reason = "missing-old-bin"
        verdict["diff"] = skipped_diff("missing-old-bin")

    if verify_board and path_exists_safe(ledger_path()):
        board = run_leaderboard(new_bin)
        verdict["leaderboard"] = board
        board_ok = board.get("ok")
        if board.get("reasonCode") == "leaderboard-error":
            errors.append(board.get("error"))
    else:
        verdict["leaderboard"] = {"ok": None, "reason": "커밋된 리더보드 없음 — 검증 생략"}
        board_ok = None

    verdict["errors"] = errors
    extra = list(pre["reasons"])
    if (verdict.get("leaderboard") or {}).get("reasonCode") == "leaderboard-error":
        extra.append("leaderboard-error")
        board_ok = None
    return attach_decision(verdict, decide_verdict(reason, board_ok, extra))


def validate_verdict(verdict):
    """판정 봉투의 정직 계약. 문제 문자열 목록(비면 통과)."""
    issues = []
    if not isinstance(verdict, dict):
        return ["verdict 가 dict 가 아니다"]
    for key in ("kind", "schemaVersion", "diff", "leaderboard", "verdict", "exit"):
        if key not in verdict:
            issues.append(f"키 없음: {key}")
    if verdict.get("kind") != REPORT_KIND:
        issues.append(f"kind 가 {REPORT_KIND} 가 아니다")
    if verdict.get("schemaVersion") != SCHEMA_VERSION:
        issues.append(f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다")
    v = verdict.get("verdict")
    if v not in VERDICTS:
        issues.append(f"알 수 없는 verdict: {v!r}")
    else:
        if verdict.get("exit") != EXIT_BY_VERDICT[v]:
            issues.append(f"exit 가 {v} 계약과 다르다")
        if verdict.get("ok") is not None and verdict.get("ok") != (v == "pass"):
            issues.append("ok 는 pass 와만 같아야 한다")
        if verdict.get("reviewRequired") is not None and verdict.get("reviewRequired") != (v == "review"):
            issues.append("reviewRequired 는 review 와만 같아야 한다")
        if verdict.get("blocked") is not None and verdict.get("blocked") != (v == "block"):
            issues.append("blocked 는 block 과만 같아야 한다")
        if verdict.get("failed") is not None and verdict.get("failed") != (v == "fail"):
            issues.append("failed 는 fail 과만 같아야 한다")
    reason = verdict.get("reason")
    if reason is not None and reason not in REASONS:
        issues.append(f"알 수 없는 reason: {reason!r}")
    if v == "review" and reason and reason != "surface-changed":
        issues.append("review 인데 reason 이 surface-changed 가 아니다")
    if v == "block" and reason not in (None, "regression", "leaderboard-broken"):
        issues.append("block 인데 reason 이 regression/leaderboard-broken 이 아니다")
    if v == "pass" and reason in ("regression", "surface-changed", "discriminate-fail",
                                  "missing-new-bin", "probe-failed"):
        issues.append(f"pass 로 {reason} 을 위장하면 안 된다")
    if v == "fail" and reason == "regression":
        issues.append("regression 을 fail 로 위장하면 안 된다 — block 이다")
    if v == "fail" and reason == "surface-changed":
        issues.append("surface-changed 를 fail 로 위장하면 안 된다 — review 이다")
    if v == "block" and reason == "surface-changed":
        issues.append("surface-changed 를 block 으로 위장하면 안 된다")
    if v == "review" and reason == "regression":
        issues.append("regression 을 review 로 위장하면 안 된다")
    if v == "pass" and verdict.get("exit") not in (None, 0):
        issues.append("pass 의 exit 는 0 이어야 한다")
    if v == "review" and verdict.get("exit") not in (None, 2):
        issues.append("review 의 exit 는 2 이어야 한다")
    if v == "block" and verdict.get("exit") not in (None, 3):
        issues.append("block 의 exit 는 3 이어야 한다")
    if v == "fail" and verdict.get("exit") not in (None, 1):
        issues.append("fail 의 exit 는 1 이어야 한다")
    diff = verdict.get("diff")
    if isinstance(diff, dict):
        cls = diff.get("classification")
        if cls == "regression" and v == "review":
            issues.append("차등 regression 을 review 로 읽으면 안 된다")
        if cls == "surface-changed" and v == "block" and reason != "leaderboard-broken":
            issues.append("차등 surface-changed 를 block 으로 읽으면 안 된다")
        if cls == "probe-failed" and v == "pass":
            issues.append("probe-failed 를 pass 로 위장하면 안 된다")
        if cls == "probe-failed" and v in ("review", "block"):
            issues.append("probe-failed 를 삼원 판정으로 위장하면 안 된다")
    return issues


def write_verdict(verdict, path):
    """UTF-8 · BOM 없음 · LF. 같은 입력이면 바이트가 같다."""
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(verdict, ensure_ascii=False, indent=2))
        fh.write("\n")


def write_verdict_safe(verdict, path):
    """쓰기 예외를 접는다. 성공이면 None, 실패면 오류 기록."""
    try:
        write_verdict(verdict, path)
        return None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return exception_record(exc, context="write", path=path)
    except Exception as exc:
        return exception_record(exc, context="write", path=path)


def render_summary_lines(verdict):
    """GitHub step summary / 콘솔이 공유하는 표. 순수."""
    d = verdict.get("diff") or {}
    b = verdict.get("leaderboard") or {}
    pre = verdict.get("preflight") or {}
    v = verdict.get("verdict", "?")
    exit_code = verdict.get("exit", "?")
    reason = verdict.get("reason", "")
    lines = [
        "## 운동장 릴리스 게이트",
        "",
        f"**판정: {v}** (exit {exit_code})",
        "",
        "| 검사 | 결과 |",
        "|---|---|",
    ]
    diff_cell = d.get("classification", "?")
    if "divergences" in d:
        diff_cell += f" · 분기 {d.get('divergences')}"
    elif d.get("reason"):
        diff_cell += f" ({d.get('reason')})"
    lines.append(f"| 릴리스 차등 | {diff_cell} |")
    if b.get("ok") is True:
        board_cell = "무결"
    elif b.get("ok") is False:
        board_cell = "파손!!"
    else:
        board_cell = b.get("reason", "생략")
    lines.append(f"| 리더보드 체인 | {board_cell} |")
    new_rec = verdict.get("new") or {}
    old_rec = verdict.get("old") or {}
    lines.append(f"| 신 바이너리 | {new_rec.get('status', '?')} |")
    lines.append(f"| 구 바이너리 | {old_rec.get('status', '?')} |")
    if pre.get("audits"):
        audits = ", ".join(
            f"{a.get('tool')}={'ok' if a.get('ok') else a.get('reason', 'fail')}"
            for a in pre["audits"]
        )
        lines.append(f"| 전제 감사 | {audits} |")
    if reason:
        lines.append(f"| 이유 | {reason} — {REASON_TEXT.get(reason, '')} |")
    lines.append("")
    if d.get("classification") == "surface-changed" or reason == "surface-changed":
        lines.append("> surface-changed 는 **차단이 아니라 리뷰 신호**다 — 명령 표면이 "
                     "바뀐 릴리스라 관측 변화가 의도된 것일 수 있다. 사람이 판정한다.")
    if reason == "discriminate-fail":
        lines.append("> 판별 감사 실패는 **회귀가 아니다**. 일 안 한 제출이 만점을 받는 "
                     "약한 오라클이다. 차등 삼원으로 위장하지 않는다.")
    if reason == "missing-new-bin":
        lines.append("> 신 바이너리 부재는 차등 생략이 아니다. 구 바이너리 부재만 skipped 다.")
    if reason == "probe-failed":
        lines.append("> 표면을 모르면 분류하지 않는다. probe-failed 를 pass/review/block 으로 "
                     "부르지 않는다.")
    if reason == "regression":
        lines.append("> 표면이 같고 관측이 갈렸다. 어느 쪽이 한컴과 맞는지는 이 게이트가 "
                     "말하지 않는다. 차단만 한다.")
    return lines


def write_github_summary(verdict):
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not path:
        return None
    lines = render_summary_lines(verdict)
    try:
        with io.open(path, "a", encoding="utf-8") as fh:
            fh.write("\n".join(lines) + "\n")
        return None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return exception_record(exc, context="write", path=path)
    except Exception as exc:
        return exception_record(exc, context="write", path=path)


def print_console(verdict):
    d = verdict.get("diff") or {}
    print(f"릴리스 차등: {d.get('classification')}"
          + (f" · 분기 {d.get('divergences')}" if "divergences" in d else ""))
    b = verdict.get("leaderboard") or {}
    print("리더보드 체인: "
          + ("무결" if b.get("ok") else ("파손" if b.get("ok") is False else "생략")))
    if verdict.get("reason") and verdict.get("reason") not in ("stable", "skipped", "missing-old-bin"):
        print(f"이유: {verdict['reason']} — {REASON_TEXT.get(verdict['reason'], '')}")
    print(f"게이트 판정: [{verdict.get('verdict')}] (exit {verdict.get('exit')})")


def parse_args(argv=None):
    """기존 CLI 그대로. 새 플래그를 추가하지 않는다."""
    ap = argparse.ArgumentParser()
    ap.add_argument("--old", default=None, help="직전 릴리스 rhwp 바이너리(없으면 차등 생략)")
    ap.add_argument("--new", default=None, help="현재 rhwp 바이너리")
    ap.add_argument("--agent", default="claude-fable-5")
    ap.add_argument("--pack", action="append", default=None)
    ap.add_argument("--no-leaderboard", action="store_true", help="리더보드 검증 생략")
    ap.add_argument("--github-summary", action="store_true")
    ap.add_argument("-o", "--out", default=None)
    return ap.parse_args(argv)


def main(argv=None):
    a = parse_args(argv)
    new_bin, find_err = find_bin_safe(a.new)
    verdict = gate(a.old, new_bin, a.agent, a.pack, not a.no_leaderboard)
    if find_err and verdict.get("new", {}).get("status") != "present":
        verdict.setdefault("errors", []).append(find_err)

    if a.out:
        write_err = write_verdict_safe(verdict, a.out)
        if write_err:
            verdict.setdefault("errors", []).append(write_err)
            # 디스크가 가득 찼다고 회귀를 안정으로 바꾸지 않는다.
            # 이미 계산한 판정을 유지하고 오류만 남긴다.
            verdict["writeError"] = write_err
    if a.github_summary:
        write_github_summary(verdict)

    print_console(verdict)
    return verdict.get("exit", 1)


if __name__ == "__main__":
    sys.exit(main())
