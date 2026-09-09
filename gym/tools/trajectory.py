"""gym 트라젝토리 필요성 감사 — 다단계 과제의 마지막 스텝이 정말 load-bearing 인가.

## 왜 이 도구인가 (종점만 채점하는 프론티어의 사각)

2026 에이전트 평가의 합의: **종점만 보면 안 된다**. 에이전트가 옳은 결과에 낭비·
위험·우회 경로로 도달해도 종점만 채점하면 만점이다(프로덕션 실패). 그래서 프론티어
프레임워크들은 트라젝토리(결정 경로)를 채점한다 — 다만 대부분 **LLM-judge** 아니면
**골든 경로**로. 둘 다 취약하다(judge 불안정, 골든 취성).

gym 도 종점-오라클이라 같은 사각을 가진다. 다단계 과제가 "N 스텝을 하라"고
광고해도, 채점이 마지막 스텝의 산출을 실제로 요구하지 않으면 그 과제는 **연극**이다
— 에이전트는 N-1 스텝만 하고도 만점을 받는다.

이 감사기는 골든 경로도 judge 도 없이 그 연극을 잡는다: 각 다단계 과제에서
**마지막 외부 의미 스텝을 빼고**(부분 트라젝토리) 기준 풀이를 재조립해 채점한다.
trailing `answer`·`keyring_from`은 제출을 모으는 내부 단계이므로 남겨야 마지막 실제
에이전트 동작이 load-bearing인지 판별할 수 있다.

- 부분 트라젝토리가 **통과** → 마지막 스텝(=선언된 최종 산출물)이 채점에 무의미.
  트라젝토리 연극이다. 리포트한다.
- 부분 트라젝토리가 **실패**(빌드 실패 포함) → 마지막 스텝이 load-bearing. 정상.

이것이 #4808 판별력 감사(종점: "산출이 입력과 다른가")를 **경로**로 민 것이다:
종점의 무편집 거부 → 경로의 무의미-스텝 거부. 모든 선언된 스텝이 결과를 바꿔야 한다.

## 예외 경로 (침묵하지 않는다)

탐색이 과제를 건너뛰는 자리는 연극 판정이 아니다. 예전에는 기준풀이 부재·빈
스텝·수집 전용 tail·바이너리 부재가 `continue` 로 사라졌다. 그러면 "연극 0"이
탐색을 못 한 자리까지 덮는다. 이제 그 네 자리는 예외 목록으로 남긴다.

- `missing-reference` — 과제 JSON 은 있는데 짝 기준풀이가 없다.
- `empty-steps` — 기준풀이에 `steps` 가 없거나 빈 목록이다.
- `collection-only-tail` — 스텝이 2개 이상인데 전부 `answer`/`keyring_from` 이다.
  마지막 외부 의미 스텝을 고를 수 없다.
- `missing-bin` — 조립·채점이 `FileNotFoundError` 로 죽었다. 이걸 load-bearing
  으로 부르면 없는 바이너리가 전 과제를 정상으로 위장한다.

단스텝 과제(T01 같은 `answer` 한 줄)는 예외가 아니다. 트라젝토리가 아니므로
건너뛴다. 마지막 스텝 load-bearing 판정 자체는 바꾸지 않는다.

보고 봉투: `kind=gymTrajectoryNecessity`, `schemaVersion=1.0`. `ok` 는 연극
0건과만 같다. 도구 실패는 `exit=1`·`trusted=false` 로 가린다.

## 사용

    python gym/tools/trajectory.py --bin target/debug/rhwp        # 전 다단계 과제 감사
    python gym/tools/trajectory.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, REPO_ROOT)

from gym.core import runner  # noqa: E402

# build_baseline 을 모듈로 실어 기준 풀이 조립기를 재사용한다(부분 트라젝토리도
# 같은 조립기로 만들어 채점 경로를 동일하게 유지).
_spec = importlib.util.spec_from_file_location("gym_build_baseline", os.path.join(HERE, "build_baseline.py"))
baseline = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(baseline)

COLLECTION_STEP_KEYS = frozenset({"answer", "keyring_from"})

REPORT_KIND = "gymTrajectoryNecessity"
SCHEMA_VERSION = "1.0"

#: 탐색·조립에서 접는 예외 kind. 시험과 문서가 같은 표를 본다.
EXCEPTION_KINDS = (
    "missing-reference",
    "empty-steps",
    "collection-only-tail",
    "missing-bin",
    "malformed-json",
    "malformed-task",
    "malformed-reference",
    "permission",
    "timeout",
    "os-error",
    "decode-error",
    "value-error",
    "type-error",
    "unexpected",
)

#: 기준풀이 스텝 열의 분류. 단스텝은 예외가 아니다.
STEP_LABELS = (
    "multi",
    "single-step",
    "empty-steps",
    "collection-only-tail",
    "malformed-reference",
)

#: JSON 보고 고정 키. 분류가 성공한 봉투의 최소 집합.
REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "taskCount",
    "loadBearing",
    "theater",
)

#: 부가 키. 있어도 연극·load-bearing 집계를 뒤집지 않는다.
OPTIONAL_REPORT_KEYS = (
    "exceptions",
    "exceptionCount",
    "skipped",
    "skipCount",
    "results",
    "trusted",
    "toolFailed",
    "toolErrors",
    "exit",
    "missingBin",
    "binPath",
)

#: 종료 코드. 0=연극 없음·도구 신뢰, 1=연극 또는 도구 실패.
EXIT_OK = 0
EXIT_FAILED = 1

#: 오류 메시지 머리 길이.
HEAD_LIMIT = 80
ERROR_HEAD_LIMIT = 160

#: 삼키면 안 되는 예외 — 도구를 죽이는 것이 정직하다.
FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

#: 탐색·조립에서 잡는 예외. BaseException 전부가 아니다.
CATCHABLE_EXCEPTIONS = (
    FileNotFoundError,
    PermissionError,
    TimeoutError,
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

#: 예외 → kind. FileNotFound 는 항상 missing-bin (조립·채점 자리).
EXCEPTION_KIND_BY_TYPE = {
    FileNotFoundError: "missing-bin",
    PermissionError: "permission",
    TimeoutError: "timeout",
    UnicodeError: "decode-error",
    UnicodeDecodeError: "decode-error",
    UnicodeEncodeError: "decode-error",
    json.JSONDecodeError: "malformed-json",
    ValueError: "value-error",
    TypeError: "type-error",
    KeyError: "value-error",
    IndexError: "value-error",
    AttributeError: "type-error",
    OSError: "os-error",
    RuntimeError: "unexpected",
}


def is_step_mapping(step) -> bool:
    """한 스텝이 키를 가진 매핑인가. 순수."""
    return isinstance(step, dict)


def step_keys(step) -> list[str]:
    """스텝 키를 정렬해 돌려준다. 매핑이 아니면 빈 목록."""
    if not is_step_mapping(step):
        return []
    return sorted(str(key) for key in step.keys())


def step_kind_label(step) -> str:
    """리포트용 스텝 종류. 예전 계약: 정렬 키를 `/` 로 잇는다."""
    keys = step_keys(step)
    return "/".join(keys) if keys else "empty"


def is_collection_step(step) -> bool:
    """수집 전용 스텝인가. answer/keyring_from 키가 하나라도 있으면 수집."""
    if not is_step_mapping(step):
        return False
    return bool(COLLECTION_STEP_KEYS.intersection(step))


def has_meaningful_key(step) -> bool:
    """외부 의미 스텝인가. 수집 키가 없는 매핑."""
    if not is_step_mapping(step):
        return False
    return not is_collection_step(step)


def last_meaningful_step_index(steps: list[dict]) -> int | None:
    """수집 전용 tail을 건너뛴 마지막 외부 의미 기준 풀이 step 위치."""
    if not isinstance(steps, list):
        return None
    for index in range(len(steps) - 1, -1, -1):
        if not is_step_mapping(steps[index]):
            continue
        if not COLLECTION_STEP_KEYS.intersection(steps[index]):
            return index
    return None


def truncate_steps(steps: list, removed_index: int) -> list:
    """removed_index 한 칸만 뺀 부분 트라젝토리. 원본을 바꾸지 않는다."""
    if not isinstance(steps, list):
        return []
    if not isinstance(removed_index, int):
        return list(steps)
    if removed_index < 0 or removed_index >= len(steps):
        return list(steps)
    return list(steps[:removed_index]) + list(steps[removed_index + 1:])


def truncate_reference(reference: dict, removed_index: int) -> dict:
    """기준풀이 사본에서 한 스텝만 뺀다. 원본 dict 를 바꾸지 않는다."""
    if not isinstance(reference, dict):
        return {"steps": []}
    truncated = dict(reference)
    truncated["steps"] = truncate_steps(reference.get("steps") or [], removed_index)
    return truncated


def normalize_steps(raw) -> list | None:
    """steps 필드를 목록으로. 없거나 목록이 아니면 None (빈 목록과 구분)."""
    if raw is None:
        return None
    if isinstance(raw, list):
        return raw
    return None


def steps_of_reference(reference) -> list | None:
    """기준풀이에서 steps 를 꺼낸다. 사전이 아니면 None."""
    if not isinstance(reference, dict):
        return None
    return normalize_steps(reference.get("steps"))


def classify_steps(steps) -> str:
    """스텝 열 분류. 순수.

    - empty-steps: None 또는 빈 목록
    - malformed-reference: 목록이 아님
    - collection-only-tail: 2개 이상이고 의미 스텝이 없음
    - single-step: 길이 1 (수집 전용이어도 단스텝 — T01)
    - multi: 길이 ≥2 이고 의미 스텝이 있음
    """
    if steps is None:
        return "empty-steps"
    if not isinstance(steps, list):
        return "malformed-reference"
    if len(steps) == 0:
        return "empty-steps"
    meaningful = last_meaningful_step_index(steps)
    if meaningful is None:
        if len(steps) >= 2:
            return "collection-only-tail"
        return "single-step"
    if len(steps) < 2:
        return "single-step"
    return "multi"


def classify_reference(reference) -> str:
    """기준풀이 JSON 분류. 순수.

    steps 키가 없거나 null 이면 empty-steps. 목록이 아닌 값이면
    malformed-reference. 빈 목록과 객체 steps 를 같은 칸에 두지 않는다.
    """
    if not isinstance(reference, dict):
        return "malformed-reference"
    if "steps" not in reference:
        return "empty-steps"
    raw = reference.get("steps")
    if raw is None:
        return "empty-steps"
    if not isinstance(raw, list):
        return "malformed-reference"
    return classify_steps(raw)


def is_audit_candidate(label: str) -> bool:
    """부분 트라젝토리 감사 대상인가. multi 만."""
    return label == "multi"


def is_skip_label(label: str) -> bool:
    """단스텝 건너뛰기인가. 예외가 아니다."""
    return label == "single-step"


def is_exception_label(label: str) -> bool:
    """탐색 예외 라벨인가."""
    return label in {
        "missing-reference",
        "empty-steps",
        "collection-only-tail",
        "malformed-json",
        "malformed-task",
        "malformed-reference",
        "missing-bin",
    }


def is_fatal_exception(exc) -> bool:
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def is_known_exception_kind(kind) -> bool:
    return kind in EXCEPTION_KINDS


def exception_kind(exc, context="audit") -> str:
    """예외를 kind 로 접는다. 순수.

    context:
      - audit: 조립·채점. FileNotFound → missing-bin.
      - load: JSON 읽기. JSONDecodeError → malformed-json.
      - scan: 디렉터리 탐색. OSError → os-error.
    """
    if exc is None:
        return "unexpected"
    if isinstance(exc, FileNotFoundError):
        return "missing-bin"
    if isinstance(exc, PermissionError):
        return "permission"
    if isinstance(exc, json.JSONDecodeError):
        if context == "load":
            return "malformed-json"
        return "value-error"
    if isinstance(exc, UnicodeError):
        return "decode-error"
    if isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, AttributeError):
        return "type-error"
    if isinstance(exc, TypeError):
        return "type-error"
    if isinstance(exc, (KeyError, IndexError)):
        return "value-error"
    if isinstance(exc, ValueError):
        return "value-error"
    if isinstance(exc, OSError):
        return "os-error"
    if isinstance(exc, RuntimeError):
        # 조립기가 부분 트라젝토리를 못 만든 자리는 load-bearing 이다.
        # kind 는 예외 목록이 아니라 그 판정으로 간다.
        return "unexpected"
    return "unexpected"


def is_missing_bin_exception(exc) -> bool:
    """조립·채점 실패를 load-bearing 으로 부르면 안 되는 자리인가."""
    return isinstance(exc, FileNotFoundError)


def truncate_head(text, limit=HEAD_LIMIT) -> str:
    """출력 머리. None/비문자는 빈 문자열. 한도는 0 이하면 빈 값."""
    if text is None:
        return ""
    if not isinstance(text, str):
        try:
            text = str(text)
        except Exception:
            return ""
    try:
        n = int(limit)
    except (TypeError, ValueError):
        n = HEAD_LIMIT
    if n <= 0:
        return ""
    return text[:n]


def exception_row(kind, pack="", task="", path="", head="", extra=None) -> dict:
    """예외 한 줄. 연극 집계를 뒤집지 않는다."""
    if not is_known_exception_kind(kind):
        kind = "unexpected"
    row = {
        "kind": kind,
        "pack": pack or "",
        "task": task or "",
        "path": path or "",
        "head": truncate_head(head, ERROR_HEAD_LIMIT),
    }
    if extra:
        row.update(extra)
    return row


def exception_from_exc(exc, context="audit", pack="", task="", path="") -> dict:
    """예외 객체를 예외 행으로."""
    kind = exception_kind(exc, context=context)
    return exception_row(
        kind,
        pack=pack,
        task=task,
        path=path,
        head=str(exc) if exc is not None else "",
        extra={"error": type(exc).__name__ if exc is not None else "NoneType"},
    )


def verdict_from_score(result) -> bool:
    """채점 봉투에서 load-bearing 인가.

    pass=True → 부분 트라젝토리 통과 → 연극 → load-bearing False.
    pass 가 없거나 거짓 → 실패 → load-bearing True.
    """
    if not isinstance(result, dict):
        return True
    return not result.get("pass")


def verdict_from_build_error(exc) -> str:
    """조립·채점 예외의 판정 자리.

    - missing-bin: 바이너리 부재. load-bearing 으로 부르지 않는다.
    - load-bearing: 부분 트라젝토리가 유효 제출을 못 만듦 (정상).
    - fatal: 다시 올려야 한다.
    """
    if is_fatal_exception(exc):
        return "fatal"
    if is_missing_bin_exception(exc):
        return "missing-bin"
    return "load-bearing"


def make_theater_line(pack_id: str, task_id: str, removed_kind: str, step_count: int) -> str:
    """연극 한 줄. 기존 계약 문구를 유지한다."""
    return (
        f"{pack_id}/{task_id} (마지막 실제 스텝 {removed_kind}을 빼도 통과 — "
        f"{step_count}→{step_count - 1})"
    )


def format_exception_line(row: dict) -> str:
    """사람용 예외 한 줄."""
    if not isinstance(row, dict):
        return "예외: (형식 오류)"
    kind = row.get("kind") or "unexpected"
    pack = row.get("pack") or ""
    task = row.get("task") or ""
    loc = f"{pack}/{task}".strip("/") if (pack or task) else (row.get("path") or "")
    head = row.get("head") or ""
    if loc and head:
        return f"{kind}: {loc} — {head}"
    if loc:
        return f"{kind}: {loc}"
    if head:
        return f"{kind}: {head}"
    return f"{kind}"


def make_result_row(pack_id: str, task_id: str, load_bearing: bool,
                    step_count: int, removed_kind: str) -> dict:
    """감사 결과 한 줄. 기존 키를 유지한다."""
    return {
        "pack": pack_id,
        "task": task_id,
        "loadBearing": bool(load_bearing),
        "steps": int(step_count),
        "removedStep": removed_kind,
    }


def make_skip_row(pack_id: str, task_id: str, reason: str, step_count=0) -> dict:
    return {
        "pack": pack_id,
        "task": task_id,
        "reason": reason,
        "steps": int(step_count) if isinstance(step_count, int) else 0,
    }


def task_label(pack_id: str, task_id: str) -> str:
    return f"{pack_id}/{task_id}"


def report_ok(theater, missing_bin=False) -> bool:
    """ok 는 연극 0건과만 같다. missing-bin 은 뒤집지 않는다."""
    del missing_bin
    return len(theater) == 0


def report_exit(theater, missing_bin=False, tool_failed=False) -> int:
    """연극 또는 도구 실패면 1, 아니면 0."""
    if theater or missing_bin or tool_failed:
        return EXIT_FAILED
    return EXIT_OK


def report_trusted(exceptions, tool_failed=False, missing_bin=False) -> bool:
    """탐색·조립을 끝까지 믿어도 되는가.

    예외가 하나라도 있거나 도구가 실패하면 거짓. 단스텝 건너뛰기는 예외가 아니다.
    """
    if tool_failed or missing_bin:
        return False
    if exceptions:
        return False
    return True


def empty_report() -> dict:
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": True,
        "taskCount": 0,
        "loadBearing": 0,
        "theater": [],
        "exceptions": [],
        "exceptionCount": 0,
        "skipped": [],
        "skipCount": 0,
        "results": [],
        "trusted": True,
        "toolFailed": False,
        "toolErrors": [],
        "exit": EXIT_OK,
        "missingBin": False,
        "binPath": "",
    }


def attach_report_counts(report: dict) -> dict:
    """집계 키를 결과·예외 목록에서 다시 센다. 원본을 바꾼다."""
    results = report.get("results") or []
    theater = report.get("theater") or []
    exceptions = report.get("exceptions") or []
    skipped = report.get("skipped") or []
    tool_errors = report.get("toolErrors") or []
    missing_bin = bool(report.get("missingBin"))
    if not missing_bin:
        missing_bin = any(
            isinstance(row, dict) and row.get("kind") == "missing-bin"
            for row in list(exceptions) + list(tool_errors)
        )
    tool_failed = bool(report.get("toolFailed")) or bool(tool_errors) or missing_bin
    report["taskCount"] = len(results)
    report["loadBearing"] = sum(1 for row in results if isinstance(row, dict) and row.get("loadBearing"))
    report["exceptionCount"] = len(exceptions)
    report["skipCount"] = len(skipped)
    report["ok"] = report_ok(theater, missing_bin=missing_bin)
    report["missingBin"] = missing_bin
    report["toolFailed"] = tool_failed
    report["trusted"] = report_trusted(exceptions, tool_failed=tool_failed, missing_bin=missing_bin)
    report["exit"] = report_exit(theater, missing_bin=missing_bin, tool_failed=tool_failed)
    return report


def validate_report(report) -> list[str]:
    """보고 봉투 정직 계약. 문제 목록. 순수."""
    issues = []
    if not isinstance(report, dict):
        return ["report 가 dict 가 아니다"]
    for key in REPORT_KEYS:
        if key not in report:
            issues.append(f"필수 키 없음: {key}")
    if report.get("kind") != REPORT_KIND:
        issues.append(f"kind 가 {REPORT_KIND} 가 아니다")
    if report.get("schemaVersion") != SCHEMA_VERSION:
        issues.append(f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다")
    theater = report.get("theater")
    if not isinstance(theater, list):
        issues.append("theater 가 list 가 아니다")
        theater = []
    results = report.get("results") if isinstance(report.get("results"), list) else None
    if results is not None:
        counted = sum(1 for row in results if isinstance(row, dict) and row.get("loadBearing"))
        if report.get("taskCount") != len(results):
            issues.append("taskCount 가 results 길이와 다르다")
        if report.get("loadBearing") != counted:
            issues.append("loadBearing 집계가 results 와 다르다")
        theater_from_results = sum(
            1 for row in results if isinstance(row, dict) and not row.get("loadBearing")
        )
        if theater_from_results != len(theater):
            issues.append("theater 건수가 loadBearing=false 결과와 다르다")
    expected_ok = len(theater) == 0
    if report.get("ok") is not expected_ok:
        issues.append("ok 는 연극 0건과만 같아야 한다")
    exceptions = report.get("exceptions")
    if exceptions is not None:
        if not isinstance(exceptions, list):
            issues.append("exceptions 가 list 가 아니다")
        else:
            if report.get("exceptionCount") not in (None, len(exceptions)):
                issues.append("exceptionCount 가 exceptions 길이와 다르다")
            for row in exceptions:
                if not isinstance(row, dict) or not is_known_exception_kind(row.get("kind")):
                    issues.append("예외 행의 kind 가 카탈로그 밖이다")
                    break
    if report.get("missingBin") and report.get("exit") != EXIT_FAILED:
        issues.append("missing-bin 인데 exit 가 1 이 아니다")
    if theater and report.get("exit") not in (None, EXIT_FAILED):
        issues.append("연극이 있는데 exit 가 1 이 아니다")
    if report.get("trusted") and (exceptions or report.get("missingBin") or report.get("toolFailed")):
        issues.append("예외/도구실패가 있는데 trusted 가 참이다")
    return issues


def safe_listdir(path):
    """os.listdir. 실패면 (None, 예외)."""
    try:
        return sorted(os.listdir(path)), None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return None, exc
    except Exception as exc:
        return None, exc


def safe_isdir(path) -> bool:
    try:
        return os.path.isdir(path)
    except FATAL_EXCEPTIONS:
        raise
    except Exception:
        return False


def safe_isfile(path) -> bool:
    try:
        return os.path.isfile(path)
    except FATAL_EXCEPTIONS:
        raise
    except Exception:
        return False


def safe_load_json(path):
    """JSON 파일. 성공 (obj, None), 실패 (None, 예외)."""
    try:
        with open(path, encoding="utf-8") as fh:
            return json.load(fh), None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return None, exc
    except Exception as exc:
        return None, exc


def bin_looks_present(bin_path) -> bool:
    """경로가 실제 파일로 보이는가. 빈 값·상대 명령 이름은 거짓이 아니다.

    시험은 조립기를 목킹하고 `bin` 같은 더미를 넘긴다. 그 자리를 여기서
    missing-bin 으로 부르면 핵심 경로가 바이너리를 요구한다. 존재 검사는
    경로에 구분자가 있거나 확장자가 있을 때만 한다.
    """
    if not bin_path or not isinstance(bin_path, str):
        return False
    if os.path.sep in bin_path or "/" in bin_path or bin_path.lower().endswith(".exe"):
        return safe_isfile(bin_path)
    return True


def resolve_bin_safe(bin_arg):
    """runner.find_bin 을 예외 없이 접는다. (path, error)."""
    try:
        path = runner.find_bin(bin_arg)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return None, exc
    except Exception as exc:
        return None, exc
    if path and not bin_looks_present(path) and (os.path.sep in str(path) or "/" in str(path)):
        return path, FileNotFoundError(path)
    return path, None


def iter_pack_ids(gym_root: str):
    """pack 디렉터리 이름. 결정적 정렬. 실패면 빈 목록과 예외."""
    packs_dir = os.path.join(gym_root, "packs")
    if not safe_isdir(packs_dir):
        return [], None
    names, err = safe_listdir(packs_dir)
    if err is not None:
        return [], err
    out = []
    for name in names:
        if safe_isdir(os.path.join(packs_dir, name)):
            out.append(name)
    return out, None


def iter_json_names(directory: str):
    """디렉터리의 *.json 이름. 없으면 빈 목록."""
    if not safe_isdir(directory):
        return []
    names, err = safe_listdir(directory)
    if err is not None:
        return []
    return [name for name in names if isinstance(name, str) and name.endswith(".json")]


def task_id_of(task, fallback: str) -> str:
    """과제 id. 없거나 비문자면 파일 줄기."""
    if isinstance(task, dict):
        tid = task.get("id")
        if isinstance(tid, str) and tid:
            return tid
    if fallback.endswith(".json"):
        return fallback[:-5]
    return fallback or ""


def scan_task_pair(pack_id: str, name: str, task_path: str, ref_path: str) -> dict:
    """과제·기준풀이 한 쌍을 분류한 레코드."""
    rec = {
        "pack": pack_id,
        "name": name,
        "taskPath": task_path,
        "refPath": ref_path,
        "label": "missing-reference",
        "task": None,
        "reference": None,
        "stepCount": 0,
        "exception": None,
    }
    task, task_err = safe_load_json(task_path)
    if task_err is not None:
        rec["label"] = "malformed-task" if not isinstance(task_err, json.JSONDecodeError) else "malformed-json"
        rec["exception"] = exception_from_exc(
            task_err, context="load", pack=pack_id, task=name, path=task_path
        )
        rec["exception"]["kind"] = rec["label"] if rec["label"] in EXCEPTION_KINDS else rec["exception"]["kind"]
        return rec
    if not isinstance(task, dict):
        rec["label"] = "malformed-task"
        rec["exception"] = exception_row("malformed-task", pack=pack_id, task=name, path=task_path,
                                         head="과제 JSON 이 객체가 아니다")
        return rec
    rec["task"] = task
    tid = task_id_of(task, name)
    if not safe_isfile(ref_path):
        rec["label"] = "missing-reference"
        rec["exception"] = exception_row(
            "missing-reference",
            pack=pack_id,
            task=tid,
            path=ref_path,
            head="짝 기준풀이가 없다",
        )
        return rec
    reference, ref_err = safe_load_json(ref_path)
    if ref_err is not None:
        rec["label"] = "malformed-json" if isinstance(ref_err, json.JSONDecodeError) else "malformed-reference"
        rec["exception"] = exception_from_exc(
            ref_err, context="load", pack=pack_id, task=tid, path=ref_path
        )
        rec["exception"]["kind"] = rec["label"]
        return rec
    if not isinstance(reference, dict):
        rec["label"] = "malformed-reference"
        rec["exception"] = exception_row(
            "malformed-reference", pack=pack_id, task=tid, path=ref_path,
            head="기준풀이 JSON 이 객체가 아니다",
        )
        return rec
    rec["reference"] = reference
    steps = steps_of_reference(reference)
    rec["stepCount"] = len(steps) if isinstance(steps, list) else 0
    rec["label"] = classify_reference(reference)
    if rec["label"] == "empty-steps":
        rec["exception"] = exception_row(
            "empty-steps", pack=pack_id, task=tid, path=ref_path,
            head="기준풀이 steps 가 비어 있다",
        )
    elif rec["label"] == "collection-only-tail":
        rec["exception"] = exception_row(
            "collection-only-tail", pack=pack_id, task=tid, path=ref_path,
            head="의미 스텝 없이 수집 전용 tail 뿐이다",
        )
    elif rec["label"] == "malformed-reference":
        rec["exception"] = exception_row(
            "malformed-reference", pack=pack_id, task=tid, path=ref_path,
            head="기준풀이 steps 가 목록이 아니다",
        )
    return rec


def scan_gym(gym_root: str) -> tuple[list[dict], list[dict]]:
    """gym/packs 를 결정적으로 훑는다. (레코드, 도구오류)."""
    records: list[dict] = []
    tool_errors: list[dict] = []
    pack_ids, err = iter_pack_ids(gym_root)
    if err is not None:
        tool_errors.append(exception_from_exc(err, context="scan", path=os.path.join(gym_root, "packs")))
        return records, tool_errors
    packs_dir = os.path.join(gym_root, "packs")
    for pack_id in pack_ids:
        tasks_dir = os.path.join(packs_dir, pack_id, "tasks")
        ref_dir = os.path.join(packs_dir, pack_id, "reference")
        if not safe_isdir(tasks_dir):
            continue
        for name in iter_json_names(tasks_dir):
            rec = scan_task_pair(
                pack_id,
                name,
                os.path.join(tasks_dir, name),
                os.path.join(ref_dir, name),
            )
            records.append(rec)
    return records, tool_errors


def multi_step_tasks(gym_root: str):
    """(pack_id, task, reference) 중 reference 가 ≥2 스텝인 것만."""
    records, _tool = scan_gym(gym_root)
    for rec in records:
        reference = rec.get("reference")
        task = rec.get("task")
        if not isinstance(reference, dict) or not isinstance(task, dict):
            continue
        steps = reference.get("steps", [])
        if isinstance(steps, list) and len(steps) >= 2:
            yield rec["pack"], task, reference


def audit_one(bin_path: str, pack_id: str, task: dict, reference: dict, work_root: str) -> dict:
    """다단계 과제 하나. 마지막 의미 스텝을 빼고 채점한다.

    반환:
      - result: 기존 결과 행
      - theater: 연극 문구 또는 None
      - exception: missing-bin 행 또는 None
      - fatal: 다시 올릴 예외 또는 None
    """
    out = {"result": None, "theater": None, "exception": None, "fatal": None}
    steps = reference.get("steps") if isinstance(reference, dict) else None
    if not isinstance(steps, list):
        tid = task_id_of(task, "")
        out["exception"] = exception_row(
            "malformed-reference", pack=pack_id, task=tid,
            head="기준풀이 steps 가 목록이 아니다",
        )
        return out
    removed_index = last_meaningful_step_index(steps)
    if removed_index is None:
        tid = task_id_of(task, "")
        if len(steps) >= 2:
            out["exception"] = exception_row(
                "collection-only-tail", pack=pack_id, task=tid,
                head="의미 스텝 없이 수집 전용 tail 뿐이다",
            )
        return out
    truncated = truncate_reference(reference, removed_index)
    tid = task_id_of(task, "")
    load_bearing = True
    try:
        baseline.build_task(bin_path, pack_id, task, truncated, work_root)
        # 종점 check만 바로 실행하면 check 명령이 누락된 최종 산출물을 다시
        # 만들어 버리거나, exit 3 오류 봉투의 일부 값만 보고 통과할 수 있다.
        # 기준 풀이 검증과 같은 순서로 submit.files의 부재를 먼저 검사한 뒤
        # 실제 채점을 해야 "마지막 스텝이 선언 산출을 만들었다"는 경로 의미가
        # 보존된다.
        inspected = baseline.inspect_built_task(
            bin_path,
            pack_id,
            task,
            work_root,
            truncated,
        )
        load_bearing = not bool(inspected.get("ok"))
    except FATAL_EXCEPTIONS as exc:
        out["fatal"] = exc
        return out
    except Exception as exc:
        place = verdict_from_build_error(exc)
        if place == "missing-bin":
            out["exception"] = exception_from_exc(
                exc, context="audit", pack=pack_id, task=tid,
            )
            out["exception"]["kind"] = "missing-bin"
            return out
        load_bearing = True
    removed_kind = step_kind_label(steps[removed_index])
    row = make_result_row(pack_id, tid, load_bearing, len(steps), removed_kind)
    out["result"] = row
    if not load_bearing:
        out["theater"] = make_theater_line(pack_id, tid, removed_kind, len(steps))
    return out


def audit(bin_path: str, gym_root: str, work_root: str) -> dict:
    results = []
    theater = []
    exceptions = []
    skipped = []
    records, tool_errors = scan_gym(gym_root)
    missing_bin = False
    for rec in records:
        label = rec.get("label")
        pack_id = rec.get("pack") or ""
        task = rec.get("task")
        tid = task_id_of(task, rec.get("name") or "")
        if label == "single-step":
            skipped.append(make_skip_row(pack_id, tid, "single-step", rec.get("stepCount") or 0))
            continue
        if is_exception_label(label):
            row = rec.get("exception") or exception_row(label, pack=pack_id, task=tid)
            exceptions.append(row)
            continue
        if label != "multi":
            continue
        if missing_bin:
            # 없는 바이너리로 나머지를 load-bearing 으로 채우지 않는다.
            continue
        one = audit_one(bin_path, pack_id, task, rec["reference"], work_root)
        if one.get("fatal") is not None:
            raise one["fatal"]
        if one.get("exception") is not None:
            exceptions.append(one["exception"])
            if one["exception"].get("kind") == "missing-bin":
                missing_bin = True
            continue
        if one.get("result") is not None:
            results.append(one["result"])
        if one.get("theater"):
            theater.append(one["theater"])
    report = empty_report()
    report["results"] = results
    report["theater"] = theater
    report["exceptions"] = exceptions
    report["skipped"] = skipped
    report["toolErrors"] = [
        exception_from_exc(e, context="scan") if not isinstance(e, dict) else e
        for e in tool_errors
    ]
    report["missingBin"] = missing_bin
    report["binPath"] = bin_path if isinstance(bin_path, str) else ""
    return attach_report_counts(report)


def render_text_report(report: dict) -> str:
    """사람용 본문. JSON 과 같은 집계를 쓴다."""
    if not isinstance(report, dict):
        return "gym 트라젝토리 필요성 감사: 보고 봉투가 아니다\n"
    lines = []
    if report.get("ok") and not report.get("missingBin") and not report.get("toolFailed"):
        lines.append(
            f"gym 트라젝토리 필요성 감사: {report.get('taskCount', 0)} 다단계 과제 전부 "
            "마지막 스텝이 load-bearing — 연극 0"
        )
    elif not report.get("ok"):
        theater = report.get("theater") or []
        lines.append(
            f"gym 트라젝토리 필요성 감사: 연극(무의미한 마지막 스텝) {len(theater)}건 — "
            "부분 트라젝토리가 통과한다:"
        )
        for item in theater:
            lines.append(f"  - {item}")
    else:
        lines.append(
            f"gym 트라젝토리 필요성 감사: 연극 0 · 도구 실패 "
            f"(예외 {report.get('exceptionCount', 0)}건, trusted="
            f"{str(bool(report.get('trusted'))).lower()})"
        )
    exceptions = report.get("exceptions") or []
    if exceptions:
        lines.append(f"예외 경로 {len(exceptions)}건:")
        for row in exceptions:
            lines.append(f"  - {format_exception_line(row)}")
    return "\n".join(lines) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 트라젝토리 필요성 감사 — 무의미한 마지막 스텝(연극) 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    bin_path, bin_err = resolve_bin_safe(a.bin)
    work_root = os.path.join(GYM_ROOT, "submissions", "_trajectory_audit")
    shutil.rmtree(work_root, ignore_errors=True)
    if bin_err is not None and isinstance(bin_err, FileNotFoundError):
        report = empty_report()
        report["exceptions"] = [exception_from_exc(bin_err, context="audit", path=str(a.bin))]
        report["missingBin"] = True
        report["binPath"] = str(a.bin)
        attach_report_counts(report)
    else:
        resolved = bin_path or runner.find_bin(a.bin)
        report = audit(resolved, GYM_ROOT, work_root)
        report["binPath"] = resolved if isinstance(resolved, str) else ""
        attach_report_counts(report)
    shutil.rmtree(work_root, ignore_errors=True)
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    else:
        sys.stdout.write(render_text_report(report))
    return int(report.get("exit", EXIT_OK if report.get("ok") else EXIT_FAILED))


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
