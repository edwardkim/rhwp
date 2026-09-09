"""gym 능력 리포트 — 한 바이너리/에이전트의 HWP 능력을 종합 스코어카드로 합친다.

## 왜 이 도구인가 (표준 계기)

지금까지 gym 의 조각은 흩어져 있었다: 점수(score.py)는 "얼마나 잘하나", 커버리지
(coverage.py)는 "무엇을 잴 수 있나", runner 신원은 "어느 바이너리가 냈나". 각각을
따로 봐서는 한 에이전트의 능력을 한 장으로 비교할 수 없다.

이 리포트는 넷을 **하나의 계기**로 합친다:

- **커버리지** — 에이전트-대면 능력 중 gym 이 잴 수 있는 비율(측정 폭).
- **정확도** — 전 pack 통과 점수(측정된 것 중 얼마나 통과).
- **축별 능력 프로파일** — 조사·편집·검증·보안·자동화 등 어느 차원이 강한가.
- **runner 신원** — 이 점수를 낸 바이너리(재현 기준). 다른 바이너리로 다시 돌리면
  같은 계기로 비교된다.

이것이 다른 에이전트가 자기 능력을 재고 겨루는 **표준 계기**다 — 커버리지·정확도를
뭉뚱그리지 않고(각각 다른 것을 잰다) 한 장에 정직하게 담는다.

예외 세 자리(이슈 #5275)는 스택이 아니라 kind 로 남긴다.

- **없는 스코어카드** — `missing-scorecard`. 채점 산출이 없는데 리포트를 합성하지 않는다.
- **깨진 JSON** — `malformed-json`. 배열·잘린 파일·디코드 실패를 점수로 위장하지 않는다.
- **미가용 pack** — `unavailable-pack`. 명령 부재는 0점이 아니다. 축 합산에서 빼고
  `packsUnavailable` 과 예외 칸에 같이 남긴다.

카탈로그·봉투 계약은 `gym/docs/certify_report.md` 가 정본이다. 작업 기록은
`mydocs/working/archives/gym_certify_report.md`. 시험은 `scripts/tests/test_gym_report.py`.

새 CLI 플래그는 없다. `--bin` `--scorecard` `--coverage` `--json` `--out` 만 쓴다.

## 사용

    python gym/report.py --bin target/debug/rhwp              # 전 pack 채점+커버리지→카드
    python gym/report.py --bin target/debug/rhwp --json       # 기계용 JSON
    python gym/report.py --scorecard sc.json --coverage cov.json  # 이미 있는 산출로 합성
    python gym/report.py --bin target/debug/rhwp --out report.md
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(HERE)

REPORT_KIND = "gymCapabilityReport"
REPORT_SCHEMA = "1.0"
SCORECARD_KIND = "gymScorecard"
DEFAULT_REPORT_AGENT = "_report"

# 종료 코드. 0=합성 성공, 2=도구 실패(없는 스코어카드·깨진 JSON·인자 부족).
EXIT_OK = 0
EXIT_TOOL_FAILED = 2

# 새 플래그 없음. 시험이 argparse 옵션 집합을 이 튜플과 대조한다.
REPORT_CLI_FLAGS = ("--bin", "--scorecard", "--coverage", "--json", "--out")

PACK_STATUSES = ("scored", "unavailable", "error")
PACK_STATUS_HELP = {
    "scored": "로드 성공, 요구 명령 충족. 축 합산·총점에 들어간다.",
    "unavailable": "요구 명령이 바이너리에 없다. 부재는 실패가 아니다.",
    "error": "로드·식별자·JSON·권한 실패. 도구 실패는 부재가 아니다.",
}

# JSON 보고 고정 키. 예전 칸을 유지하고 예외 칸을 뒤에 붙인다.
REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "agent",
    "runner",
    "accuracy",
    "coverage",
    "axisProfile",
    "packsScored",
    "packsUnavailable",
    "packsErrored",
    "exceptions",
    "exceptionCount",
    "trusted",
)

REPORT_LIST_KEYS = ("axisProfile", "packsUnavailable", "packsErrored", "exceptions")
REPORT_INT_KEYS = ("packsScored", "exceptionCount")
REPORT_BOOL_KEYS = ("trusted",)
ACCURACY_KEYS = ("score", "max", "percent")
COVERAGE_KEYS = ("percent", "covered", "agentFacingTotal", "uncoveredByCategory")
AXIS_ROW_KEYS = ("axis", "score", "max", "packs", "percent")
EXCEPTION_RECORD_KEYS = ("kind", "message", "where", "path", "pack", "role")

# 예외 kind 카탈로그. 문서·시험이 같은 표를 본다. 새 CLI 를 만들지 않는다.
EXCEPTION_KINDS = (
    "missing-scorecard",
    "missing-coverage",
    "missing-bin",
    "missing-report-arg",
    "malformed-json",
    "malformed-scorecard",
    "malformed-coverage",
    "malformed-pack-row",
    "unavailable-pack",
    "empty-packs",
    "empty-total",
    "permission",
    "os-error",
    "decode-error",
    "type-error",
    "value-error",
    "write-error",
    "report-tool-failed",
    "unexpected",
)

EXCEPTION_KIND_HELP = {
    "missing-scorecard": (
        "스코어카드 파일이 없다. --scorecard 경로가 비었거나, --bin 모드에서 "
        "score.py 가 submissions/_report/scorecard.json 을 남기지 않았다. "
        "없는 산출을 0점으로 합성하지 않는다."
    ),
    "missing-coverage": (
        "커버리지 JSON 파일이 없다. --coverage 를 줬으면 파일이 있어야 한다. "
        "--bin 모드에서 coverage.py 가 없거나 실패한 것은 선택적이라 이 kind 가 아니다."
    ),
    "missing-bin": (
        "경로형 바이너리가 없거나 --bin 값이 비었다. 채점 하위 도구가 시작되지 않는다."
    ),
    "missing-report-arg": (
        "--bin 도 (--scorecard + --coverage) 도 없다. 예전 필수 인자 계약 그대로 "
        "종료 코드 2. 새 플래그를 요구하지 않는다."
    ),
    "malformed-json": (
        "파일이 UTF-8 JSON 이 아니다. 잘린 객체, 트레일링 콤마, 빈 파일, BOM 만 "
        "있는 자리. 파싱 실패를 빈 점수로 바꾸지 않는다."
    ),
    "malformed-scorecard": (
        "스코어카드가 JSON 객체(dict)가 아니다. 배열·문자열·숫자는 카드가 아니다."
    ),
    "malformed-coverage": (
        "커버리지 산출이 JSON 객체가 아니다. 배열이면 카테고리 집계를 할 수 없다."
    ),
    "malformed-pack-row": (
        "packs 목록의 한 칸이 객체가 아니다. 그 칸만 건너뛰고 나머지는 합성한다."
    ),
    "unavailable-pack": (
        "pack status=unavailable. 요구 명령이 바이너리에 없다. 축 프로파일과 "
        "총점에 넣지 않고 packsUnavailable 과 예외 칸에 남긴다. 종료 코드는 0."
    ),
    "empty-packs": (
        "스코어카드 packs 가 없거나 빈 목록이다. 정확도 0/0, 축 프로파일 빈 목록."
    ),
    "empty-total": (
        "스코어카드 total 이 없거나 max 가 0 이다. 정확도 percent 는 0. "
        "측정된 pack 이 없다는 뜻이지 만점이 아니다."
    ),
    "permission": "스코어카드·커버리지·산출 파일을 읽을 권한 또는 쓸 권한이 없다.",
    "os-error": "그 밖의 OSError. 디스크·경로·잠금. 점수로 접지 않는다.",
    "decode-error": "파일이 UTF-8 이 아니다. 디코드 실패는 malformed-json 보다 앞선다.",
    "type-error": "값 타입이 계약과 다르다. bool 을 int 로 세지 않는다.",
    "value-error": "값은 있는데 형태가 틀렸다. 빈 경로, 음수 아닌데 문자인 점수 등.",
    "write-error": "--out 경로에 카드를 쓰지 못했다. stdout 대체가 아니면 실패.",
    "report-tool-failed": (
        "하위 도구(build_baseline.py 또는 score.py)가 비-0 으로 끝났다. "
        "리포트는 그 산출을 지어내지 않는다."
    ),
    "unexpected": "분류되지 않은 운영 예외. 치명 예외는 여기로 접지 않는다.",
}

FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

CATCHABLE_EXCEPTIONS = (
    FileNotFoundError,
    PermissionError,
    IsADirectoryError,
    NotADirectoryError,
    UnicodeError,
    json.JSONDecodeError,
    ValueError,
    TypeError,
    KeyError,
    IndexError,
    AttributeError,
    OSError,
)

ROLE_MISSING_KIND = {
    "scorecard": "missing-scorecard",
    "coverage": "missing-coverage",
    "bin": "missing-bin",
}

ROLE_MALFORMED_KIND = {
    "scorecard": "malformed-scorecard",
    "coverage": "malformed-coverage",
}

INFORMATIONAL_KINDS = frozenset({"unavailable-pack", "empty-packs", "empty-total"})

USAGE_MESSAGE = "필수: --bin <경로> 또는 (--scorecard + --coverage)"


class ReportError(Exception):
    """리포트 도구가 접는 운영 예외. kind 는 EXCEPTION_KINDS 중 하나."""

    def __init__(self, kind: str, message: str, **extra: object) -> None:
        if kind not in EXCEPTION_KINDS:
            kind = "unexpected"
        self.kind = kind
        self.message = message
        self.extra = extra
        super().__init__(message)

    def as_record(self) -> dict:
        return exception_record(self.kind, self.message, **self.extra)


def is_json_object(value: object) -> bool:
    """JSON 객체인가. bool 은 int 의 하위 타입이라 dict 만 인정한다."""
    return isinstance(value, dict) and not isinstance(value, bool)


def is_json_array(value: object) -> bool:
    return isinstance(value, list)


def is_known_exception_kind(kind: object) -> bool:
    return isinstance(kind, str) and kind in EXCEPTION_KINDS


def is_fatal_exception(exc: BaseException) -> bool:
    return isinstance(exc, FATAL_EXCEPTIONS)


def is_catchable_exception(exc: BaseException) -> bool:
    if is_fatal_exception(exc):
        return False
    return isinstance(exc, CATCHABLE_EXCEPTIONS) or isinstance(exc, ReportError)


def describe_exception_kind(kind: str) -> str:
    if kind in EXCEPTION_KIND_HELP:
        return EXCEPTION_KIND_HELP[kind]
    return EXCEPTION_KIND_HELP["unexpected"]


def exception_record(kind: str, message: str, **extra: object) -> dict:
    """예외 한 줄. 없는 칸은 생략한다. 기계가 같은 키로 읽는다."""
    rec: dict = {"kind": kind if is_known_exception_kind(kind) else "unexpected",
                 "message": message}
    for key in ("where", "path", "pack", "role"):
        if extra.get(key) not in (None, ""):
            rec[key] = extra[key]
    return rec


def as_int(value: object, default: int = 0) -> int:
    """점수·개수. bool 과 문자열은 기본값. 음수는 그대로 둔다(위조 탐지용)."""
    if type(value) is bool:
        return default
    if isinstance(value, int):
        return value
    if isinstance(value, float) and value.is_integer():
        return int(value)
    return default


def percent_of(score: int, maximum: int) -> int:
    """정수 나눗셈. max 가 0 이면 0. 예전 카드와 같은 공식이다."""
    if maximum:
        return 100 * score // maximum
    return 0


def axis_label(axis: str) -> str:
    """축 라벨 — 괄호 앞의 능력 차원(예: '편집 (표 좌표 지정)' → '편집')."""
    return (axis or "미분류").split(" (")[0].strip() or "미분류"


def pack_status(pack: object) -> str | None:
    if not is_json_object(pack):
        return None
    status = pack.get("status")
    if status in PACK_STATUSES:
        return status
    return None


def is_scored_pack(pack: object) -> bool:
    return pack_status(pack) == "scored"


def is_unavailable_pack(pack: object) -> bool:
    return pack_status(pack) == "unavailable"


def is_error_pack(pack: object) -> bool:
    return pack_status(pack) == "error"


def pack_id_of(pack: object) -> str:
    if not is_json_object(pack):
        return "?"
    pid = pack.get("id")
    if isinstance(pid, str) and pid.strip():
        return pid
    return "?"


def role_missing_kind(role: str) -> str:
    return ROLE_MISSING_KIND.get(role, "missing-scorecard")


def role_malformed_kind(role: str) -> str:
    return ROLE_MALFORMED_KIND.get(role, "malformed-json")


def classify_os_error(exc: BaseException, *, role: str = "scorecard") -> str:
    """FileNotFound 는 역할에 따라 갈린다. 없는 스코어카드를 없는 커버리지로 부르지 않는다."""
    if isinstance(exc, ReportError):
        return exc.kind
    if isinstance(exc, json.JSONDecodeError):
        return "malformed-json"
    if isinstance(exc, UnicodeError):
        return "decode-error"
    if isinstance(exc, PermissionError):
        return "permission"
    if isinstance(exc, FileNotFoundError):
        return role_missing_kind(role)
    if isinstance(exc, IsADirectoryError):
        return role_malformed_kind(role)
    if isinstance(exc, TypeError):
        return "type-error"
    if isinstance(exc, ValueError):
        return "value-error"
    if isinstance(exc, OSError):
        return "os-error"
    return "unexpected"


def error_head(exc: BaseException, limit: int = 240) -> str:
    text = str(exc).strip() or type(exc).__name__
    if len(text) > limit:
        return text[:limit]
    return text


def wrap_exception(exc: BaseException, *, role: str = "scorecard",
                   where: str = "", path: str = "") -> ReportError:
    if isinstance(exc, ReportError):
        return exc
    if is_fatal_exception(exc):
        raise exc
    kind = classify_os_error(exc, role=role)
    return ReportError(kind, error_head(exc), role=role, where=where, path=path)


def load_text(path: str, *, role: str) -> str:
    """UTF-8 본문. 없으면 역할별 missing-*, 디코드 실패는 decode-error."""
    if not isinstance(path, str) or not path.strip():
        raise ReportError(role_missing_kind(role), f"{role} 경로가 비었다",
                          role=role, path=path)
    if os.path.isdir(path):
        raise ReportError(role_malformed_kind(role),
                          f"{role} 경로가 디렉터리다: {path}",
                          role=role, path=path, where="load_text")
    if not os.path.isfile(path):
        raise ReportError(role_missing_kind(role),
                          f"{role} 파일이 없다: {path}",
                          role=role, path=path, where="load_text")
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except PermissionError as e:
        raise ReportError("permission", f"{role} 권한 없음: {error_head(e)}",
                          role=role, path=path)
    except UnicodeError as e:
        raise ReportError("decode-error", f"{role} UTF-8 디코드 실패: {error_head(e)}",
                          role=role, path=path)
    except OSError as e:
        raise ReportError("os-error", f"{role} 읽기 실패: {error_head(e)}",
                          role=role, path=path)


def parse_json_text(text: str, *, role: str) -> object:
    if text is None:
        raise ReportError("malformed-json", f"{role} 본문이 없다", role=role)
    try:
        return json.loads(text)
    except json.JSONDecodeError as e:
        raise ReportError("malformed-json",
                          f"{role} JSON 파싱 실패: {error_head(e)}",
                          role=role, where=f"line {e.lineno}")


def load_json_object(path: str, *, role: str) -> dict:
    """파일 → JSON 객체. 배열·스칼라는 역할별 malformed-*."""
    raw = load_text(path, role=role)
    data = parse_json_text(raw, role=role)
    if not is_json_object(data):
        raise ReportError(role_malformed_kind(role),
                          f"{role} 가 JSON 객체가 아니다",
                          role=role, path=path)
    return data


def write_text(path: str, text: str) -> None:
    try:
        parent = os.path.dirname(path)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
    except PermissionError as e:
        raise ReportError("write-error", f"산출 권한 없음: {error_head(e)}", path=path)
    except OSError as e:
        raise ReportError("write-error", f"산출 기록 실패: {error_head(e)}", path=path)


def scorecard_path_for_agent(agent: str = DEFAULT_REPORT_AGENT) -> str:
    return os.path.join(HERE, "submissions", agent, "scorecard.json")


def iter_pack_rows(scorecard: object) -> list[object]:
    if not is_json_object(scorecard):
        return []
    packs = scorecard.get("packs")
    if not is_json_array(packs):
        return []
    return list(packs)


def collect_unavailable_packs(scorecard: object) -> list[str]:
    ids: list[str] = []
    for pack in iter_pack_rows(scorecard):
        if is_unavailable_pack(pack):
            ids.append(pack_id_of(pack))
    return ids


def collect_error_packs(scorecard: object) -> list[str]:
    ids: list[str] = []
    for pack in iter_pack_rows(scorecard):
        if is_error_pack(pack):
            ids.append(pack_id_of(pack))
    return ids


def collect_scored_packs(scorecard: object) -> list[dict]:
    rows: list[dict] = []
    for pack in iter_pack_rows(scorecard):
        if is_scored_pack(pack) and is_json_object(pack):
            rows.append(pack)
    return rows


def validate_scorecard(scorecard: object) -> list[dict]:
    """스코어카드 뼈대. 없어도 compile 이 죽지 않게 예외 목록만 낸다."""
    notes: list[dict] = []
    if not is_json_object(scorecard):
        notes.append(exception_record(
            "malformed-scorecard", "스코어카드가 JSON 객체가 아니다",
            where="compile_report",
        ))
        return notes
    packs = scorecard.get("packs")
    if packs is None:
        notes.append(exception_record(
            "empty-packs", "스코어카드에 packs 칸이 없다",
            where="scorecard.packs",
        ))
    elif not is_json_array(packs):
        notes.append(exception_record(
            "malformed-scorecard", "packs 가 목록이 아니다",
            where="scorecard.packs",
        ))
    elif len(packs) == 0:
        notes.append(exception_record(
            "empty-packs", "스코어카드 packs 가 비었다",
            where="scorecard.packs",
        ))
    else:
        for index, pack in enumerate(packs):
            if not is_json_object(pack):
                notes.append(exception_record(
                    "malformed-pack-row",
                    f"packs[{index}] 가 객체가 아니다",
                    where=f"scorecard.packs[{index}]",
                ))
                continue
            if is_unavailable_pack(pack):
                notes.append(exception_record(
                    "unavailable-pack",
                    f"pack {pack_id_of(pack)} 는 요구 명령 부재로 채점되지 않았다",
                    pack=pack_id_of(pack),
                    where="scorecard.packs",
                ))
    total = scorecard.get("total")
    if total is None:
        notes.append(exception_record(
            "empty-total", "스코어카드에 total 칸이 없다",
            where="scorecard.total",
        ))
    elif not is_json_object(total):
        notes.append(exception_record(
            "malformed-scorecard", "total 이 객체가 아니다",
            where="scorecard.total",
        ))
    elif as_int(total.get("max"), 0) == 0:
        notes.append(exception_record(
            "empty-total", "total.max 가 0 이다 — 측정된 만점이 없다",
            where="scorecard.total.max",
        ))
    return notes


def validate_coverage(coverage: object) -> list[dict]:
    """커버리지는 선택적. 빈 객체는 예외가 아니다. 배열만 거부한다."""
    if coverage in (None, {}):
        return []
    if not is_json_object(coverage):
        return [exception_record(
            "malformed-coverage", "커버리지가 JSON 객체가 아니다",
            where="coverage",
        )]
    return []


def compile_axis_profile(packs: list[object]) -> list[dict]:
    """scored pack 만 축 라벨로 합산. unavailable·error·비객체는 빠진다."""
    by_axis: dict[str, dict] = {}
    for pack in packs:
        if not is_scored_pack(pack) or not is_json_object(pack):
            continue
        label = axis_label(pack.get("axis", "") if isinstance(pack.get("axis"), str) else "")
        acc = by_axis.setdefault(label, {
            "axis": label, "score": 0, "max": 0, "packs": 0,
        })
        acc["score"] += as_int(pack.get("score"), 0)
        acc["max"] += as_int(pack.get("max"), 0)
        acc["packs"] += 1
    for acc in by_axis.values():
        acc["percent"] = percent_of(acc["score"], acc["max"])
    return sorted(by_axis.values(), key=lambda a: (-a["percent"], a["axis"]))


def accuracy_from_total(total: object) -> dict:
    if not is_json_object(total):
        return {"score": 0, "max": 0, "percent": 0}
    score = as_int(total.get("score"), 0)
    maximum = as_int(total.get("max"), 0)
    return {"score": score, "max": maximum, "percent": percent_of(score, maximum)}


def coverage_block(coverage: object) -> dict:
    if not is_json_object(coverage):
        return {
            "percent": None,
            "covered": None,
            "agentFacingTotal": None,
            "uncoveredByCategory": {},
        }
    uncovered = coverage.get("uncoveredByCategory", {})
    if not is_json_object(uncovered):
        uncovered = {}
    return {
        "percent": coverage.get("coveragePercent"),
        "covered": coverage.get("covered"),
        "agentFacingTotal": coverage.get("agentFacingTotal"),
        "uncoveredByCategory": uncovered,
    }


def is_informational_kind(kind: str) -> bool:
    return kind in INFORMATIONAL_KINDS


def structural_exceptions(exceptions: list[dict]) -> list[dict]:
    return [e for e in exceptions if not is_informational_kind(e.get("kind", ""))]


def compile_report(scorecard: dict, coverage: dict) -> dict:
    """순수 합성 — 바이너리·파일 접근 없음(가드가 픽스처로 시험 가능).

    scorecard: score.py 산출(kind gymScorecard). coverage: coverage.py 산출.
    깨진 입력은 던지지 않고 exceptions 칸에 남긴다. 예전 칸(accuracy·coverage·
    axisProfile·packsUnavailable)의 계산식은 그대로다.
    """
    notes = validate_scorecard(scorecard)
    notes.extend(validate_coverage(coverage))
    if not is_json_object(scorecard):
        scorecard = {}
    if not is_json_object(coverage):
        coverage = {}

    packs = iter_pack_rows(scorecard)
    total = scorecard.get("total") if is_json_object(scorecard.get("total")) else {}
    axis_profile = compile_axis_profile(packs)
    unavailable = collect_unavailable_packs(scorecard)
    errored = collect_error_packs(scorecard)
    trusted = len(structural_exceptions(notes)) == 0

    return {
        "kind": REPORT_KIND,
        "schemaVersion": REPORT_SCHEMA,
        "agent": scorecard.get("agent") if is_json_object(scorecard) else None,
        "runner": scorecard.get("runner") if is_json_object(scorecard) else None,
        # 두 축을 뭉뚱그리지 않는다 — 정확도(측정된 것 통과율)와 커버리지(측정 폭).
        "accuracy": accuracy_from_total(total),
        "coverage": coverage_block(coverage),
        "axisProfile": axis_profile,
        "packsScored": as_int(total.get("packsScored"), 0) if is_json_object(total) else 0,
        "packsUnavailable": unavailable,
        "packsErrored": errored,
        "exceptions": notes,
        "exceptionCount": len(notes),
        "trusted": trusted,
    }


def render_exceptions(exceptions: object) -> list[str]:
    if not is_json_array(exceptions) or not exceptions:
        return []
    lines = ["", "## 예외 경로", ""]
    for item in exceptions:
        if not is_json_object(item):
            continue
        kind = item.get("kind") or "unexpected"
        message = item.get("message") or ""
        pack = item.get("pack")
        suffix = f" (pack {pack})" if pack else ""
        lines.append(f"- `{kind}`{suffix}: {message}")
    return lines


def render_card(report: dict) -> str:
    if not is_json_object(report):
        report = {}
    acc = report.get("accuracy") if is_json_object(report.get("accuracy")) else {}
    cov = report.get("coverage") if is_json_object(report.get("coverage")) else {}
    score = as_int(acc.get("score"), 0)
    maximum = as_int(acc.get("max"), 0)
    pct = as_int(acc.get("percent"), 0)
    lines = [
        "# gym 능력 스코어카드",
        "",
        f"- **정확도** (측정된 것 통과): {score}/{maximum} ({pct}%)",
    ]
    if cov.get("percent") is not None:
        lines.append(
            f"- **커버리지** (에이전트-대면 측정 폭): {cov['covered']}/{cov['agentFacingTotal']}"
            f" ({cov['percent']}%)"
        )
    lines.append(f"- **채점 pack**: {report.get('packsScored', 0)}")
    unavailable = report.get("packsUnavailable") or []
    if unavailable:
        labels = [str(x) for x in unavailable]
        lines.append(f"- **미가용 pack**(명령 부재): {', '.join(labels)}")
    errored = report.get("packsErrored") or []
    if errored:
        lines.append(f"- **오류 pack**(도구 실패): {', '.join(str(x) for x in errored)}")
    r = report.get("runner") or {}
    if is_json_object(r) and r:
        lines.append(
            f"- **runner**: v{r.get('rhwpVersion')} · {str(r.get('rhwpCommit'))[:12]}"
        )
    if report.get("trusted") is False:
        lines.append("- **신뢰**: 구조 예외가 있어 이 카드만으로 능력을 단정하지 않는다")
    lines += ["", "## 축별 능력 프로파일", "", "| 축 | 점수 | % |", "|---|---|---|"]
    for a in report.get("axisProfile") or []:
        if not is_json_object(a):
            continue
        lines.append(f"| {a.get('axis')} | {a.get('score')}/{a.get('max')} | {a.get('percent')}% |")
    uncovered = cov.get("uncoveredByCategory") or {}
    if is_json_object(uncovered) and uncovered:
        flat = [n for names in uncovered.values() if is_json_array(names) for n in names]
        lines += ["", f"## 미측정 능력 (다음 성장 방향, {len(flat)}개)", "",
                  "`" + "` · `".join(str(n) for n in flat) + "`"]
    lines += render_exceptions(report.get("exceptions"))
    return "\n".join(lines) + "\n"


def dump_report_json(report: dict) -> str:
    return json.dumps(report, ensure_ascii=False, indent=2) + "\n"


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(description="gym 능력 종합 스코어카드")
    ap.add_argument("--bin", help="rhwp 바이너리 — 전 pack 채점+커버리지를 돌린다")
    ap.add_argument("--scorecard", help="score.py 스코어카드 JSON(이미 있으면)")
    ap.add_argument("--coverage", help="coverage.py --json 산출(이미 있으면)")
    ap.add_argument("--json", action="store_true", help="카드 대신 JSON")
    ap.add_argument("--out", help="출력 파일(생략 시 stdout)")
    return ap


def cli_flag_names(parser: argparse.ArgumentParser | None = None) -> tuple[str, ...]:
    ap = parser or build_parser()
    names: list[str] = []
    for action in ap._actions:
        for opt in action.option_strings:
            if opt.startswith("--") and opt not in ("--help",):
                names.append(opt)
    return tuple(names)


def _run(tool_argv: list[str]) -> None:
    # 하위 도구(build_baseline·score)의 진행 로그는 stderr 로 넘긴다 — report.py 의
    # stdout 은 카드/JSON 전용으로 순수하게 둬, --json 을 기계가 그대로 파싱할 수 있게.
    out = subprocess.run([sys.executable, *tool_argv], cwd=REPO_ROOT, capture_output=True)
    sys.stderr.write(out.stdout.decode("utf-8", "replace"))
    sys.stderr.write(out.stderr.decode("utf-8", "replace"))
    if out.returncode != 0:
        name = os.path.basename(str(tool_argv[0]))
        raise ReportError(
            "report-tool-failed",
            f"하위 도구 실패: {name}",
            where=name,
        )


def _from_bin(bin_path: str) -> tuple[dict, dict]:
    """--bin 모드: 전 pack 채점 + (있으면) 커버리지를 실제로 돌려 산출을 읽는다.

    커버리지 측정기(coverage.py)는 선택적이다 — 없으면 정확도·축 프로파일만 낸다.
    스코어카드 부재·깨진 JSON 은 선택적이 아니다. missing-scorecard / malformed-json.
    """
    if not isinstance(bin_path, str) or not bin_path.strip():
        raise ReportError("missing-bin", "바이너리 경로가 비었다", role="bin")
    _run([os.path.join(HERE, "tools", "build_baseline.py"), "--agent", DEFAULT_REPORT_AGENT,
          "--bin", bin_path])
    _run([os.path.join(HERE, "score.py"), "--agent", DEFAULT_REPORT_AGENT, "--bin", bin_path])
    scorecard_path = scorecard_path_for_agent(DEFAULT_REPORT_AGENT)
    if not os.path.isfile(scorecard_path):
        raise ReportError(
            "missing-scorecard",
            f"score.py 가 scorecard.json 을 남기지 않았다: {scorecard_path}",
            path=scorecard_path, role="scorecard", where="submissions/_report",
        )
    scorecard = load_json_object(scorecard_path, role="scorecard")
    coverage: dict = {}
    cov_tool = os.path.join(HERE, "tools", "coverage.py")
    if os.path.isfile(cov_tool):
        try:
            cov_raw = subprocess.run(
                [sys.executable, cov_tool, "--bin", bin_path, "--json"],
                cwd=REPO_ROOT, capture_output=True,
            ).stdout
            parsed = json.loads(cov_raw)
            if is_json_object(parsed):
                coverage = parsed
        except (ValueError, OSError):
            coverage = {}
    return scorecard, coverage


def load_inputs(args: argparse.Namespace) -> tuple[dict, dict]:
    """CLI 입력을 스코어카드·커버리지로. 예외는 ReportError."""
    if args.scorecard and args.coverage:
        return (
            load_json_object(args.scorecard, role="scorecard"),
            load_json_object(args.coverage, role="coverage"),
        )
    if args.bin:
        return _from_bin(args.bin)
    raise ReportError("missing-report-arg", USAGE_MESSAGE)


def emit_output(report: dict, *, as_json: bool, out_path: str | None) -> int:
    text = dump_report_json(report) if as_json else render_card(report)
    if out_path:
        write_text(out_path, text)
        print(f"작성: {out_path}")
    else:
        sys.stdout.write(text)
    return EXIT_OK


def main(argv: list[str] | None = None) -> int:
    ap = build_parser()
    a = ap.parse_args(argv)
    try:
        scorecard, coverage = load_inputs(a)
        report = compile_report(scorecard, coverage)
        return emit_output(report, as_json=a.json, out_path=a.out)
    except ReportError as e:
        print(f"{e.kind}: {e}", file=sys.stderr)
        return EXIT_TOOL_FAILED
    except CATCHABLE_EXCEPTIONS as e:
        wrapped = wrap_exception(e, role="scorecard", where="main")
        print(f"{wrapped.kind}: {wrapped}", file=sys.stderr)
        return EXIT_TOOL_FAILED


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
