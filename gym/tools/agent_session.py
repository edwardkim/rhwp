"""gym 에이전트 세션 트레이스 — 선언된 명령 열을 기록하고 경로를 채점한다.

## 왜 이 도구인가 (종점만 보면 경로가 사라진다)

gym 채점은 산출(종점)을 본다. `trajectory.py` 는 기준 풀이에서 마지막 외부 스텝을
잘라 그 스텝이 load-bearing 인지 감사한다. 둘 다 "에이전트가 실제로 어떤 명령
열을 어떤 순서로 밟았는가"는 파일로 남기지 않는다. 종점만 맞으면 우회·역순·여분
명령도 만점이 된다.

이 도구는 **선언된 세션**(명령 열 + 기대 종료)과 **기록된 트레이스**(JSONL)를
기계 대조한다. 채점 축은 명령 계열(`argv[0]`)·종료 코드·순서다. LLM-judge 도
골든 경로 문자열 비교도 쓰지 않는다.

재생(`score-replay`)은 rhwp 를 부르지 않는다 — 픽스처 JSONL 만으로 단위 시험이
가능하다. 기록(`record`)은 `--bin` 이 있을 때만 실행한다. 없으면 거절한다.
없는 바이너리를 가장해 트레이스를 위조하지 않는다.

## 세션 정의

    {
      "id": "inspect-then-export",
      "input": "samples/x.hwp",
      "subDir": "work",
      "steps": [
        {"run": ["info", "{input}", "--json"], "expectExit": 0},
        {"run": ["export-text", "{input}", "-o", "{sub:out.txt}"],
         "expectExit": 0, "expectPath": "{sub:out.txt}"}
      ]
    }

`{input}` 은 세션(또는 CLI) 입력 경로, `{sub:이름}` 은 작업 폴더 안 경로다.

## 트레이스 JSONL (한 줄 = 한 스텝)

    {"ts": "2026-08-18T00:00:00Z", "argv": ["info", "samples/x.hwp", "--json"],
     "exit": 0, "stdoutSha256": "<hex>", "ok": true}

## 사용

    python gym/tools/agent_session.py validate --session S.json
    python gym/tools/agent_session.py score-replay --session S.json --replay T.jsonl
    python gym/tools/agent_session.py record --session S.json --bin target/debug/rhwp --out T.jsonl
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)

SCHEMA_VERSION = "1.0"
REPORT_KIND = "gymAgentSession"
VALIDATE_KIND = "gymAgentSessionValidate"
RECORD_KIND = "gymAgentSessionRecord"

PLACEHOLDER_RE = re.compile(r"\{([^{}]+)\}")
HEX64_RE = re.compile(r"^[0-9a-fA-F]{64}$")
SUB_NAME_RE = re.compile(r"^[^\s{}]+$")

REASON_WRONG_COMMAND = "wrongCommand"
REASON_WRONG_ORDER = "wrongOrder"
REASON_WRONG_EXIT = "wrongExit"
REASON_EXTRA_STEP = "extraStep"
REASON_MISSING_STEP = "missingStep"
REASON_WRONG_PATH = "wrongPath"
REASON_BAD_TRACE = "badTrace"
REASON_BAD_SESSION = "badSession"

USAGE_EXIT = 2
FAIL_EXIT = 1
OK_EXIT = 0


class SessionError(ValueError):
    """세션·트레이스·기록 입력이 계약을 깨뜨렸을 때.

    하위 유형은 `code` 로 기계 분류한다. CLI 는 유형에 따라
    검증 리포트 / 채점 리포트 / stderr 거절 문구를 고른다.
    """

    code = "sessionError"
    exit_code = FAIL_EXIT

    def __init__(
        self,
        message: str,
        *,
        code: str | None = None,
        path: str | None = None,
        detail=None,
        line: int | None = None,
    ):
        super().__init__(message)
        if code is not None:
            self.code = code
        self.path = path
        self.detail = detail
        self.line = line

    def to_dict(self) -> dict:
        payload = {
            "type": type(self).__name__,
            "code": self.code,
            "message": str(self),
            "exitCode": int(self.exit_code),
        }
        if self.path is not None:
            payload["path"] = self.path
        if self.detail is not None:
            payload["detail"] = self.detail
        if self.line is not None:
            payload["line"] = self.line
        return payload


class RecordRefused(SessionError):
    """record 가 바이너리 없이 위조하려 할 때 — 실행하지 않고 거절."""

    code = "recordRefused"
    exit_code = USAGE_EXIT


class SessionFileError(SessionError):
    """세션 JSON 파일을 열 수 없다 (없음·디렉터리·권한)."""

    code = "sessionFile"


class SessionParseError(SessionError):
    """세션 파일이 UTF-8 JSON 이 아니다."""

    code = "sessionParse"


class SessionSchemaError(SessionError):
    """세션 JSON 은 열렸으나 스키마(id/steps/자리표)가 깨졌다."""

    code = "sessionSchema"


class TraceFileError(SessionError):
    """트레이스 JSONL 파일을 열 수 없다."""

    code = "traceFile"


class TraceParseError(SessionError):
    """트레이스 줄이 JSON 이 아니거나 이벤트가 없다."""

    code = "traceParse"


class TraceSchemaError(SessionError):
    """트레이스 이벤트 필드(ts/argv/exit/ok/stdoutSha256)가 깨졌다."""

    code = "traceSchema"


class ExecuteError(SessionError):
    """기록 실행기가 예외를 내거나 exit 을 반환하지 않았다."""

    code = "executeError"


class WriteError(SessionError):
    """트레이스 JSONL 쓰기가 실패했다."""

    code = "writeError"


class PlaceholderError(SessionError):
    """자리표 해석이 계약을 깨뜨렸다 (엄격 모드)."""

    code = "placeholderError"


ERROR_CODE_CATALOG = (
    ("sessionError", SessionError, "분류되지 않은 세션 계약 위반"),
    ("recordRefused", RecordRefused, "record 가 --bin 없이 위조하려 함"),
    ("sessionFile", SessionFileError, "세션 JSON 파일을 열 수 없음"),
    ("sessionParse", SessionParseError, "세션 파일이 UTF-8 JSON 이 아님"),
    ("sessionSchema", SessionSchemaError, "세션 스키마(id/steps/자리표) 위반"),
    ("traceFile", TraceFileError, "트레이스 JSONL 을 열 수 없음"),
    ("traceParse", TraceParseError, "트레이스 줄 파싱 실패 또는 이벤트 없음"),
    ("traceSchema", TraceSchemaError, "트레이스 이벤트 필드 위반"),
    ("executeError", ExecuteError, "실행기 예외 또는 exit 미반환"),
    ("writeError", WriteError, "트레이스 JSONL 쓰기 실패"),
    ("placeholderError", PlaceholderError, "자리표 엄격 해석 실패"),
)


def error_code_names() -> list[str]:
    return [row[0] for row in ERROR_CODE_CATALOG]


def error_class_for_code(code: str):
    for name, cls, _hint in ERROR_CODE_CATALOG:
        if name == code:
            return cls
    return SessionError


def classify_exception(exc: BaseException) -> str:
    """예외 → 채점 mismatch reason. record 거절은 채점 사유가 아니다."""
    if isinstance(exc, (SessionSchemaError, SessionParseError, SessionFileError)):
        return REASON_BAD_SESSION
    if isinstance(exc, RecordRefused):
        return REASON_BAD_SESSION
    if isinstance(exc, (TraceSchemaError, TraceParseError, TraceFileError)):
        return REASON_BAD_TRACE
    if isinstance(exc, SessionError):
        return REASON_BAD_TRACE
    return REASON_BAD_TRACE


def wrap_io_error(exc: OSError, path: str, *, reading: bool = True) -> SessionError:
    """OSError → 읽기면 SessionFileError, 쓰기면 WriteError.

    Windows 는 디렉터리를 open 하면 IsADirectoryError 대신 PermissionError 가
    나는 경우가 있어, 경로가 디렉터리면 그걸 우선한다.
    """
    cls: type[SessionError] = SessionFileError if reading else WriteError
    action = "읽기" if reading else "쓰기"
    if isinstance(exc, FileNotFoundError):
        return cls(f"파일을 찾을 수 없다: {path}", path=path)
    if isinstance(exc, IsADirectoryError) or os.path.isdir(path):
        return cls(f"경로가 디렉터리다: {path}", path=path)
    if isinstance(exc, PermissionError):
        return cls(f"파일 {action} 권한이 없다: {path}", path=path)
    return cls(f"파일 {action} 실패: {path}: {exc}", path=path)


def fail_score_report(
    reason: str,
    detail: str,
    session_id: str | None = None,
    declared: int = 0,
    observed: int = 0,
) -> dict:
    """로드/파싱 실패를 채점 리포트로 접는다. 바이너리는 부르지 않는다."""
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": False,
        "sessionId": session_id,
        "declared": declared,
        "observed": observed,
        "matched": 0,
        "orderOk": False,
        "steps": [],
        "extraSteps": [],
        "missingSteps": [],
        "mismatches": [{"reason": reason, "detail": detail}],
    }


class SessionContext:
    """자리표 해석에 쓰는 입력 문서·작업 폴더. 없으면 해당 자리표는 미해석."""

    def __init__(self, input_path: str | None = None, sub_dir: str | None = None):
        self.input_path = input_path
        self.sub_dir = sub_dir

    @classmethod
    def from_session(
        cls,
        session: dict,
        input_path: str | None = None,
        sub_dir: str | None = None,
    ) -> SessionContext:
        return cls(
            input_path if input_path is not None else session.get("input"),
            sub_dir if sub_dir is not None else session.get("subDir"),
        )


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_text(data: str | bytes | None) -> str | None:
    if data is None:
        return None
    if isinstance(data, str):
        data = data.encode("utf-8")
    return sha256_bytes(data)


def command_family(argv) -> str:
    """명령 계열 = argv[0]. 빈 argv 는 빈 문자열."""
    if not argv:
        return ""
    first = argv[0]
    if not isinstance(first, str):
        return ""
    return first


def find_placeholders(token: str) -> list[tuple[int, int, str]]:
    """토큰 안 `{…}` 자리표의 (시작, 끝, 본문) 목록. 중첩 없음."""
    if not isinstance(token, str):
        return []
    found = []
    for match in PLACEHOLDER_RE.finditer(token):
        found.append((match.start(), match.end(), match.group(1)))
    return found


def classify_placeholder(body: str) -> tuple[str, str | None]:
    """자리표 본문 → (종류, 이름). 종류: input / sub / unknown."""
    if body == "input":
        return "input", None
    if body.startswith("sub:"):
        return "sub", body[4:]
    return "unknown", body


def placeholder_issues(token: str, path: str) -> list[str]:
    issues = []
    if not isinstance(token, str):
        issues.append(f"{path}: 인자는 문자열이어야 한다")
        return issues
    if token.count("{") != token.count("}"):
        issues.append(f"{path}: 자리표 중괄호가 짝이 아니다: {token!r}")
    for _start, _end, body in find_placeholders(token):
        kind, name = classify_placeholder(body)
        if kind == "sub":
            if not name:
                issues.append(f"{path}: {{sub:}} 이름이 비었다")
            elif not SUB_NAME_RE.match(name):
                issues.append(f"{path}: {{sub:{name}}} 이름이 유효하지 않다")
        elif kind == "unknown":
            issues.append(f"{path}: 알 수 없는 자리표 {{{body}}}")
    if "{" in token and not find_placeholders(token) and "}" not in token:
        issues.append(f"{path}: 닫히지 않은 자리표: {token!r}")
    return issues


def resolve_token(
    token: str,
    context: SessionContext | None,
    create_parents: bool = False,
    strict: bool = False,
) -> str:
    """한 인자에서 `{input}` / `{sub:이름}` 을 치환. 모르는 토큰은 그대로 둔다.

    strict=True 이면 미해석 자리표·알 수 없는 자리표를 PlaceholderError 로 올린다.
    기본(False)은 재생 채점이 작업 폴더 없이 돌아가게 자리표를 남긴다.
    """
    if not isinstance(token, str) or "{" not in token:
        return token
    ctx = context or SessionContext()
    out = []
    cursor = 0
    for start, end, body in find_placeholders(token):
        out.append(token[cursor:start])
        kind, name = classify_placeholder(body)
        if kind == "input":
            if ctx.input_path is None:
                if strict:
                    raise PlaceholderError("strict: {input} 을 해석할 input 이 없다")
                out.append(token[start:end])
            else:
                out.append(ctx.input_path)
        elif kind == "sub":
            if ctx.sub_dir is None or not name:
                if strict:
                    raise PlaceholderError(f"strict: {{sub:{name or ''}}} 을 해석할 subDir 이 없다")
                out.append(token[start:end])
            else:
                path = os.path.join(ctx.sub_dir, name.replace("/", os.sep))
                if create_parents:
                    parent = os.path.dirname(path)
                    if parent:
                        try:
                            os.makedirs(parent, exist_ok=True)
                        except OSError as exc:
                            raise WriteError(
                                f"자리표 부모 디렉터리 생성 실패: {parent}: {exc}",
                                path=parent,
                            ) from exc
                out.append(path)
        else:
            if strict:
                raise PlaceholderError(f"strict: 알 수 없는 자리표 {{{body}}}")
            out.append(token[start:end])
        cursor = end
    out.append(token[cursor:])
    return "".join(out)


def resolve_argv(
    argv,
    context: SessionContext | None,
    create_parents: bool = False,
    strict: bool = False,
) -> list[str]:
    if argv is None:
        raise SessionSchemaError("run 인자가 없다")
    try:
        items = list(argv)
    except TypeError as exc:
        raise SessionSchemaError(f"run 인자가 순회 가능하지 않다: {exc}") from exc
    return [
        resolve_token(a, context, create_parents=create_parents, strict=strict)
        for a in items
    ]


def expected_ok(exit_code: int, expect_exit: int, path_ok: bool | None) -> bool:
    if exit_code != expect_exit:
        return False
    if path_ok is False:
        return False
    return True


def normalize_expect_exit(step: dict) -> int:
    raw = step.get("expectExit", 0)
    if raw is None:
        return 0
    return int(raw)


def declared_family(step: dict) -> str:
    return command_family(step.get("run") or [])


def validate_session(doc) -> list[str]:
    """세션 정의 구조 검사. 빈 목록이면 유효. 바이너리 불요."""
    issues: list[str] = []
    if not isinstance(doc, dict):
        return ["세션 정의는 객체여야 한다"]
    sid = doc.get("id")
    if not isinstance(sid, str) or not sid.strip():
        issues.append("id 가 비어 있다")
    if "input" in doc and doc["input"] is not None and not isinstance(doc["input"], str):
        issues.append("input 은 문자열이어야 한다")
    if "subDir" in doc and doc["subDir"] is not None and not isinstance(doc["subDir"], str):
        issues.append("subDir 은 문자열이어야 한다")
    steps = doc.get("steps")
    if not isinstance(steps, list):
        issues.append("steps 가 배열이 아니다")
        return issues
    if not steps:
        issues.append("steps 가 비어 있다")
        return issues
    for index, step in enumerate(steps):
        prefix = f"steps[{index}]"
        if not isinstance(step, dict):
            issues.append(f"{prefix}: 객체여야 한다")
            continue
        run = step.get("run")
        if not isinstance(run, list) or not run:
            issues.append(f"{prefix}.run: 비어 있지 않은 인자 배열이어야 한다")
        else:
            for arg_i, arg in enumerate(run):
                if not isinstance(arg, str) or arg == "":
                    issues.append(f"{prefix}.run[{arg_i}]: 비어 있지 않은 문자열이어야 한다")
                else:
                    issues.extend(placeholder_issues(arg, f"{prefix}.run[{arg_i}]"))
            if isinstance(run[0], str) and not run[0].strip():
                issues.append(f"{prefix}.run[0]: 명령 계열이 비어 있다")
        if "expectExit" in step and step["expectExit"] is not None:
            if not isinstance(step["expectExit"], int) or isinstance(step["expectExit"], bool):
                issues.append(f"{prefix}.expectExit: 정수여야 한다")
        if "expectPath" in step and step["expectPath"] is not None:
            if not isinstance(step["expectPath"], str) or not step["expectPath"]:
                issues.append(f"{prefix}.expectPath: 비어 있지 않은 문자열이어야 한다")
            else:
                issues.extend(placeholder_issues(step["expectPath"], f"{prefix}.expectPath"))
    return issues


def validate_trace_event(event, index: int) -> list[str]:
    prefix = f"trace[{index}]"
    if not isinstance(event, dict):
        return [f"{prefix}: 객체여야 한다"]
    issues = []
    ts = event.get("ts")
    if not isinstance(ts, str) or not ts.strip():
        issues.append(f"{prefix}.ts: 비어 있지 않은 문자열이어야 한다")
    argv = event.get("argv")
    if not isinstance(argv, list) or not argv:
        issues.append(f"{prefix}.argv: 비어 있지 않은 배열이어야 한다")
    else:
        for arg_i, arg in enumerate(argv):
            if not isinstance(arg, str):
                issues.append(f"{prefix}.argv[{arg_i}]: 문자열이어야 한다")
    if "exit" not in event:
        issues.append(f"{prefix}.exit: 필수")
    elif not isinstance(event["exit"], int) or isinstance(event["exit"], bool):
        issues.append(f"{prefix}.exit: 정수여야 한다")
    if "ok" not in event:
        issues.append(f"{prefix}.ok: 필수")
    elif not isinstance(event["ok"], bool):
        issues.append(f"{prefix}.ok: 불리언이어야 한다")
    digest = event.get("stdoutSha256")
    if digest is not None:
        if not isinstance(digest, str) or not HEX64_RE.match(digest):
            issues.append(f"{prefix}.stdoutSha256: 64자리 hex 여야 한다")
    return issues


def parse_trace_jsonl(text: str) -> list[dict]:
    """JSONL 본문 → 이벤트 목록. 빈 줄은 건너뛴다. 잘못된 줄은 TraceParseError."""
    events = []
    if text is None:
        raise TraceParseError("트레이스가 비어 있다")
    if not isinstance(text, str):
        raise TraceParseError(f"트레이스는 문자열이어야 한다: {type(text).__name__}")
    # UTF-8 BOM 이 첫 줄에 붙으면 JSON 파서가 거절한다. 한 번만 벗긴다.
    if text.startswith("\ufeff"):
        text = text.lstrip("\ufeff")
    for line_no, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line:
            continue
        try:
            event = json.loads(line)
        except ValueError as exc:
            raise TraceParseError(
                f"trace 줄 {line_no}: JSON 이 아니다: {exc}",
                line=line_no,
            ) from exc
        if not isinstance(event, dict):
            raise TraceParseError(
                f"trace 줄 {line_no}: 객체여야 한다 (배열/스칼라 금지)",
                line=line_no,
            )
        events.append(event)
    if not events:
        raise TraceParseError("트레이스에 이벤트가 없다")
    return events


def validate_trace(events) -> list[str]:
    if not isinstance(events, list):
        return ["트레이스는 이벤트 배열이어야 한다"]
    issues = []
    for index, event in enumerate(events):
        issues.extend(validate_trace_event(event, index))
    return issues


def load_json_file(path: str):
    """세션 JSON 로드. 없음/디렉터리/권한/UTF-8/JSON 을 유형별로 접는다."""
    if path is None or not str(path).strip():
        raise SessionFileError("세션 경로가 비어 있다", path=path)
    try:
        with open(path, encoding="utf-8") as fh:
            return json.load(fh)
    except FileNotFoundError as exc:
        raise wrap_io_error(exc, path, reading=True) from exc
    except IsADirectoryError as exc:
        raise wrap_io_error(exc, path, reading=True) from exc
    except PermissionError as exc:
        raise wrap_io_error(exc, path, reading=True) from exc
    except UnicodeDecodeError as exc:
        raise SessionParseError(f"UTF-8 이 아니다: {path}: {exc}", path=path) from exc
    except ValueError as exc:
        raise SessionParseError(f"JSON 파싱 실패: {path}: {exc}", path=path) from exc
    except OSError as exc:
        raise wrap_io_error(exc, path, reading=True) from exc


def load_text_file(path: str) -> str:
    """트레이스 JSONL 본문 로드. 없음/디렉터리/권한/UTF-8 을 유형별로 접는다."""
    if path is None or not str(path).strip():
        raise TraceFileError("트레이스 경로가 비어 있다", path=path)
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except FileNotFoundError as exc:
        raise TraceFileError(f"파일을 찾을 수 없다: {path}", path=path) from exc
    except UnicodeDecodeError as exc:
        raise TraceParseError(f"UTF-8 이 아니다: {path}: {exc}", path=path) from exc
    except OSError as exc:
        wrapped = wrap_io_error(exc, path, reading=True)
        raise TraceFileError(str(wrapped), path=path) from exc


def write_jsonl(path: str, events: list[dict]) -> None:
    if path is None or not str(path).strip():
        raise WriteError("트레이스 출력 경로가 비어 있다", path=path)
    if not isinstance(events, list):
        raise WriteError(f"이벤트 목록이 배열이 아니다: {type(events).__name__}", path=path)
    parent = os.path.dirname(path)
    try:
        if parent:
            os.makedirs(parent, exist_ok=True)
        lines = [json.dumps(ev, ensure_ascii=False, separators=(",", ":")) for ev in events]
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write("\n".join(lines))
            if lines:
                fh.write("\n")
    except TypeError as exc:
        raise WriteError(f"트레이스 직렬화 실패: {path}: {exc}", path=path) from exc
    except OSError as exc:
        raise wrap_io_error(exc, path, reading=False) from exc


def load_session_file(path: str) -> dict:
    doc = load_json_file(path)
    issues = validate_session(doc)
    if issues:
        raise SessionSchemaError(
            "세션 정의가 유효하지 않다: " + "; ".join(issues),
            path=path,
            detail=list(issues),
        )
    return doc


def load_trace_file(path: str) -> list[dict]:
    events = parse_trace_jsonl(load_text_file(path))
    issues = validate_trace(events)
    if issues:
        raise TraceSchemaError(
            "트레이스가 유효하지 않다: " + "; ".join(issues),
            path=path,
            detail=list(issues),
        )
    return events


def lcs_table(left: list[str], right: list[str]) -> list[list[int]]:
    n, m = len(left), len(right)
    table = [[0] * (m + 1) for _ in range(n + 1)]
    for i, a in enumerate(left):
        row = table[i]
        nxt = table[i + 1]
        for j, b in enumerate(right):
            if a == b:
                nxt[j + 1] = row[j] + 1
            else:
                nxt[j + 1] = row[j + 1] if row[j + 1] >= nxt[j] else nxt[j]
    return table


def lcs_ops(left: list[str], right: list[str]) -> list[tuple[str, int | None, int | None]]:
    """LCS 역추적 — match / del / ins. 인덱스는 원 수열 기준."""
    table = lcs_table(left, right)
    i, j = len(left), len(right)
    raw: list[tuple[str, int | None, int | None]] = []
    while i > 0 or j > 0:
        if i > 0 and j > 0 and left[i - 1] == right[j - 1]:
            raw.append(("match", i - 1, j - 1))
            i -= 1
            j -= 1
        elif j > 0 and (i == 0 or table[i][j - 1] >= table[i - 1][j]):
            raw.append(("ins", None, j - 1))
            j -= 1
        else:
            raw.append(("del", i - 1, None))
            i -= 1
    raw.reverse()
    return raw


def collapse_ops(ops: list[tuple[str, int | None, int | None]]) -> list[tuple[str, int | None, int | None]]:
    """인접 del+ins 를 sub(명령 교체)로 접어 여분+누락으로 오인하지 않게 한다."""
    out: list[tuple[str, int | None, int | None]] = []
    index = 0
    while index < len(ops):
        kind, dec_i, obs_i = ops[index]
        if (
            kind == "del"
            and index + 1 < len(ops)
            and ops[index + 1][0] == "ins"
        ):
            out.append(("sub", dec_i, ops[index + 1][2]))
            index += 2
            continue
        if (
            kind == "ins"
            and index + 1 < len(ops)
            and ops[index + 1][0] == "del"
        ):
            out.append(("sub", ops[index + 1][1], obs_i))
            index += 2
            continue
        out.append((kind, dec_i, obs_i))
        index += 1
    return out


def same_multiset(left: list[str], right: list[str]) -> bool:
    if len(left) != len(right):
        return False
    counts: dict[str, int] = {}
    for item in left:
        counts[item] = counts.get(item, 0) + 1
    for item in right:
        counts[item] = counts.get(item, 0) - 1
        if counts[item] < 0:
            return False
    return all(v == 0 for v in counts.values())


def classify_sequence(declared: list[str], observed: list[str]) -> list[str]:
    """선언 계열 vs 관측 계열 → 순서/여분/누락/교체 사유. 같으면 빈 목록."""
    if declared == observed:
        return []
    if not declared and observed:
        return [REASON_EXTRA_STEP]
    if declared and not observed:
        return [REASON_MISSING_STEP]
    if len(observed) > len(declared) and observed[: len(declared)] == declared:
        return [REASON_EXTRA_STEP]
    if len(observed) < len(declared) and declared[: len(observed)] == observed:
        return [REASON_MISSING_STEP]
    if same_multiset(declared, observed) and declared != observed:
        return [REASON_WRONG_ORDER]
    reasons = []
    for kind, _dec_i, _obs_i in collapse_ops(lcs_ops(declared, observed)):
        if kind == "ins" and REASON_EXTRA_STEP not in reasons:
            reasons.append(REASON_EXTRA_STEP)
        elif kind == "del" and REASON_MISSING_STEP not in reasons:
            reasons.append(REASON_MISSING_STEP)
        elif kind == "sub" and REASON_WRONG_COMMAND not in reasons:
            reasons.append(REASON_WRONG_COMMAND)
    if not reasons:
        reasons.append(REASON_WRONG_COMMAND)
    return reasons


def _path_exists(path: str | None) -> bool | None:
    if not path or "{" in path:
        return None
    return os.path.exists(path)


def compare_step(
    index: int,
    step: dict,
    event: dict | None,
    context: SessionContext | None,
    check_paths: bool = False,
) -> dict:
    """한 스텝의 계열·종료·순서 대조. event 가 없으면 누락.

    재생 채점의 합격 축은 계열·종료·순서다. expectPath 디스크 존재는
    `check_paths=True`(기록 직후 재검증)일 때만 합격에 넣는다. 픽스처
    JSONL 재생이 작업 폴더 부재로 실패하지 않게 한다.
    """
    family = declared_family(step)
    expect_exit = normalize_expect_exit(step)
    expect_path = step.get("expectPath")
    resolved_path = None
    if isinstance(expect_path, str):
        resolved_path = resolve_token(expect_path, context)
    row = {
        "index": index,
        "declaredFamily": family,
        "observedFamily": None,
        "familyOk": False,
        "declaredExit": expect_exit,
        "observedExit": None,
        "exitOk": False,
        "orderOk": False,
        "pathOk": None,
        "ok": False,
    }
    if event is None:
        return row
    observed_family = command_family(event.get("argv") or [])
    observed_exit = event.get("exit")
    family_ok = observed_family == family
    exit_ok = observed_exit == expect_exit
    path_ok = None
    if check_paths and resolved_path is not None:
        path_ok = _path_exists(resolved_path)
    row["observedFamily"] = observed_family
    row["observedExit"] = observed_exit
    row["familyOk"] = family_ok
    row["exitOk"] = bool(exit_ok)
    row["orderOk"] = family_ok
    row["pathOk"] = path_ok
    path_blocks = check_paths and path_ok is False
    row["ok"] = bool(family_ok and exit_ok and not path_blocks)
    return row


def score_session(
    session: dict,
    events: list[dict],
    context: SessionContext | None = None,
    check_paths: bool = False,
) -> dict:
    """선언 세션 vs 트레이스. 기본은 계열·종료·순서만(디스크 불요)."""
    session_issues = validate_session(session)
    if session_issues:
        return {
            "kind": REPORT_KIND,
            "schemaVersion": SCHEMA_VERSION,
            "ok": False,
            "sessionId": session.get("id") if isinstance(session, dict) else None,
            "declared": 0,
            "observed": 0,
            "matched": 0,
            "orderOk": False,
            "steps": [],
            "extraSteps": [],
            "missingSteps": [],
            "mismatches": [{"reason": REASON_BAD_SESSION, "detail": issue} for issue in session_issues],
        }
    trace_issues = validate_trace(events)
    if trace_issues:
        return {
            "kind": REPORT_KIND,
            "schemaVersion": SCHEMA_VERSION,
            "ok": False,
            "sessionId": session["id"],
            "declared": len(session["steps"]),
            "observed": len(events) if isinstance(events, list) else 0,
            "matched": 0,
            "orderOk": False,
            "steps": [],
            "extraSteps": [],
            "missingSteps": [],
            "mismatches": [{"reason": REASON_BAD_TRACE, "detail": issue} for issue in trace_issues],
        }

    steps = session["steps"]
    ctx = context or SessionContext.from_session(session)
    declared_fams = [declared_family(step) for step in steps]
    observed_fams = [command_family(ev.get("argv") or []) for ev in events]
    seq_reasons = classify_sequence(declared_fams, observed_fams)

    compared = []
    paired = min(len(steps), len(events))
    for index in range(paired):
        compared.append(compare_step(index, steps[index], events[index], ctx, check_paths))
    missing_rows = []
    for index in range(paired, len(steps)):
        row = compare_step(index, steps[index], None, ctx, check_paths)
        compared.append(row)
        missing_rows.append({
            "index": index,
            "family": declared_fams[index],
            "expectExit": normalize_expect_exit(steps[index]),
        })
    extra_rows = []
    for offset, event in enumerate(events[len(steps):]):
        extra_rows.append({
            "index": len(steps) + offset,
            "family": command_family(event.get("argv") or []),
            "exit": event.get("exit"),
            "argv": list(event.get("argv") or []),
        })

    mismatches: list[dict] = []
    for reason in seq_reasons:
        payload = {"reason": reason, "declared": list(declared_fams), "observed": list(observed_fams)}
        if reason == REASON_EXTRA_STEP:
            payload["extra"] = extra_rows
        if reason == REASON_MISSING_STEP:
            payload["missing"] = missing_rows
        mismatches.append(payload)

    # 순서·계열은 맞는데 종료 코드나 expectPath 만 틀린 경우.
    if REASON_WRONG_ORDER not in seq_reasons:
        for row in compared:
            if row["observedFamily"] is None:
                continue
            if row["familyOk"] and not row["exitOk"]:
                mismatches.append({
                    "reason": REASON_WRONG_EXIT,
                    "index": row["index"],
                    "declaredExit": row["declaredExit"],
                    "observedExit": row["observedExit"],
                })
            if check_paths and row["familyOk"] and row["pathOk"] is False:
                mismatches.append({
                    "reason": REASON_WRONG_PATH,
                    "index": row["index"],
                    "expectPath": steps[row["index"]].get("expectPath"),
                })

    order_ok = declared_fams == observed_fams
    matched = sum(1 for row in compared if row["ok"])
    ok = order_ok and not extra_rows and not missing_rows and not mismatches
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": ok,
        "sessionId": session["id"],
        "declared": len(steps),
        "observed": len(events),
        "matched": matched,
        "orderOk": order_ok,
        "steps": compared,
        "extraSteps": extra_rows,
        "missingSteps": missing_rows,
        "mismatches": mismatches,
    }


def build_trace_event(
    argv,
    exit_code: int,
    stdout: str | bytes | None = None,
    expect_exit: int = 0,
    path_ok: bool | None = None,
    ts: str | None = None,
) -> dict:
    event = {
        "ts": ts or utc_now(),
        "argv": [str(a) for a in list(argv)],
        "exit": int(exit_code),
        "ok": expected_ok(int(exit_code), int(expect_exit), path_ok),
    }
    digest = sha256_text(stdout)
    if digest is not None:
        event["stdoutSha256"] = digest
    return event


def require_record_bin(bin_path: str | None) -> str:
    """record 는 --bin 이 실재해야 한다. 없으면 거절(위조 금지)."""
    if bin_path is None or not str(bin_path).strip():
        raise RecordRefused(
            "record 모드는 --bin 이 필요합니다. 바이너리 없이 트레이스를 위조하지 않습니다."
        )
    path = str(bin_path).strip()
    if not os.path.isfile(path):
        raise RecordRefused(
            f"record 모드: 바이너리를 찾을 수 없습니다: {path} "
            "(없는 실행파일을 가장해 기록하지 않습니다)"
        )
    return path


def default_execute(bin_path: str, argv: list[str], cwd: str | None = None) -> dict:
    """실바이너리 한 번 실행. FileNotFound/권한/OS 오류는 ExecuteError."""
    if not bin_path:
        raise ExecuteError("실행 바이너리 경로가 비어 있다")
    try:
        proc = subprocess.run(
            [bin_path] + list(argv),
            cwd=cwd or REPO_ROOT,
            capture_output=True,
        )
    except FileNotFoundError as exc:
        raise ExecuteError(f"실행 파일을 찾을 수 없다: {bin_path}") from exc
    except PermissionError as exc:
        raise ExecuteError(f"실행 권한이 없다: {bin_path}") from exc
    except OSError as exc:
        raise ExecuteError(f"실행 실패: {bin_path}: {exc}") from exc
    except subprocess.SubprocessError as exc:
        raise ExecuteError(f"서브프로세스 오류: {bin_path}: {exc}") from exc
    return {"exit": proc.returncode, "stdout": proc.stdout}


def record_session(
    session: dict,
    bin_path: str | None,
    out_path: str | None = None,
    context: SessionContext | None = None,
    execute=None,
    clock=None,
) -> list[dict]:
    """세션을 실행해 JSONL 이벤트를 만든다. execute 가 없으면 실바이너리 필요."""
    issues = validate_session(session)
    if issues:
        raise SessionSchemaError(
            "세션 정의가 유효하지 않다: " + "; ".join(issues),
            detail=list(issues),
        )
    ctx = context or SessionContext.from_session(session)
    if execute is None:
        resolved_bin = require_record_bin(bin_path)
        execute = lambda _bin, argv: default_execute(resolved_bin, argv)
    elif bin_path is None or not str(bin_path).strip():
        # 주입 실행기여도 호출부가 --bin 없이 record 를 열면 거절한다.
        raise RecordRefused(
            "record 모드는 --bin 이 필요합니다. 바이너리 없이 트레이스를 위조하지 않습니다."
        )

    if ctx.sub_dir:
        try:
            os.makedirs(ctx.sub_dir, exist_ok=True)
        except OSError as exc:
            raise WriteError(
                f"작업 폴더를 만들 수 없다: {ctx.sub_dir}: {exc}",
                path=ctx.sub_dir,
            ) from exc

    events = []
    for step_index, step in enumerate(session["steps"]):
        argv = resolve_argv(step["run"], ctx, create_parents=True)
        try:
            result = execute(bin_path, argv)
        except SessionError:
            raise
        except Exception as exc:
            raise ExecuteError(
                f"실행기 예외 (steps[{step_index}] {argv[:3]}): {exc}"
            ) from exc
        if not isinstance(result, dict) or "exit" not in result:
            raise ExecuteError(f"실행기가 exit 을 반환하지 않았다: {argv[:3]}")
        try:
            exit_code = int(result["exit"])
        except (TypeError, ValueError) as exc:
            raise ExecuteError(
                f"실행기 exit 이 정수가 아니다: {result.get('exit')!r}"
            ) from exc
        path_ok = None
        expect_path = step.get("expectPath")
        if isinstance(expect_path, str) and expect_path:
            resolved = resolve_token(expect_path, ctx, create_parents=False)
            path_ok = os.path.exists(resolved)
        try:
            ts = clock() if clock else utc_now()
        except Exception as exc:
            raise SessionError(f"시계 콜백 예외: {exc}") from exc
        events.append(
            build_trace_event(
                argv,
                exit_code,
                stdout=result.get("stdout"),
                expect_exit=normalize_expect_exit(step),
                path_ok=path_ok,
                ts=ts,
            )
        )
    if out_path:
        write_jsonl(out_path, events)
    return events


def render_validate(issues: list[str], session_id: str | None = None) -> str:
    if not issues:
        label = session_id or "세션"
        return f"gym 에이전트 세션 검증: {label} — 유효"
    lines = [f"gym 에이전트 세션 검증: 위반 {len(issues)}건"]
    for issue in issues:
        lines.append(f"  - {issue}")
    return "\n".join(lines)


def render_score(report: dict) -> str:
    sid = report.get("sessionId") or "?"
    if report.get("ok"):
        head = (
            f"gym 에이전트 세션: {sid} — 통과 "
            f"({report.get('matched')}/{report.get('declared')} 스텝, 순서·계열·종료 일치)"
        )
    else:
        head = (
            f"gym 에이전트 세션: {sid} — 실패 "
            f"(일치 {report.get('matched')}/{report.get('declared')}, "
            f"관측 {report.get('observed')})"
        )
    lines = [head]
    for row in report.get("steps") or []:
        idx = row.get("index")
        if row.get("observedFamily") is None:
            lines.append(f"  [{idx}] 누락 — 기대 {row.get('declaredFamily')} exit={row.get('declaredExit')}")
            continue
        mark = "일치" if row.get("ok") else "불일치"
        detail = (
            f"기대 {row.get('declaredFamily')} exit={row.get('declaredExit')}, "
            f"관측 {row.get('observedFamily')} exit={row.get('observedExit')}"
        )
        lines.append(f"  [{idx}] {mark} — {detail}")
    for extra in report.get("extraSteps") or []:
        lines.append(
            f"  [+{extra.get('index')}] 여분 — {extra.get('family')} exit={extra.get('exit')}"
        )
    for mismatch in report.get("mismatches") or []:
        reason = mismatch.get("reason")
        if reason == REASON_WRONG_ORDER:
            lines.append(
                f"  순서 불일치: 선언 {mismatch.get('declared')} / 관측 {mismatch.get('observed')}"
            )
        elif reason == REASON_WRONG_COMMAND:
            lines.append(
                f"  명령 계열 불일치: 선언 {mismatch.get('declared')} / 관측 {mismatch.get('observed')}"
            )
        elif reason == REASON_WRONG_EXIT:
            lines.append(
                f"  [{mismatch.get('index')}] 종료 코드 불일치: "
                f"기대 {mismatch.get('declaredExit')} / 관측 {mismatch.get('observedExit')}"
            )
        elif reason == REASON_EXTRA_STEP:
            lines.append(f"  여분 스텝 {len(mismatch.get('extra') or [])}건")
        elif reason == REASON_MISSING_STEP:
            lines.append(f"  누락 스텝 {len(mismatch.get('missing') or [])}건")
        elif reason == REASON_WRONG_PATH:
            lines.append(f"  [{mismatch.get('index')}] 기대 경로 없음: {mismatch.get('expectPath')}")
        elif reason in (REASON_BAD_SESSION, REASON_BAD_TRACE):
            lines.append(f"  {mismatch.get('detail')}")
    return "\n".join(lines)


REASON_LABELS_KO = {
    REASON_WRONG_COMMAND: "명령 계열 불일치",
    REASON_WRONG_ORDER: "순서 불일치",
    REASON_WRONG_EXIT: "종료 코드 불일치",
    REASON_EXTRA_STEP: "여분 스텝",
    REASON_MISSING_STEP: "누락 스텝",
    REASON_WRONG_PATH: "기대 경로 없음",
    REASON_BAD_TRACE: "트레이스 계약 위반",
    REASON_BAD_SESSION: "세션 정의 계약 위반",
}


def reason_label_ko(reason: str) -> str:
    return REASON_LABELS_KO.get(reason, reason)


def render_error(exc: BaseException) -> str:
    """예외를 한 줄 한국어 진단으로 접는다. CLI stderr / 문서 예시용."""
    if isinstance(exc, RecordRefused):
        return f"기록 거절: {exc}"
    if isinstance(exc, SessionError):
        bits = [f"{exc.code}: {exc}"]
        if exc.path:
            bits.append(f"경로={exc.path}")
        if exc.line is not None:
            bits.append(f"줄={exc.line}")
        return " ".join(bits)
    return f"미분류 예외: {type(exc).__name__}: {exc}"


def validate_report(issues: list[str], session_id: str | None = None) -> dict:
    return {
        "kind": VALIDATE_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": len(issues) == 0,
        "sessionId": session_id,
        "issueCount": len(issues),
        "issues": list(issues),
    }


def emit(payload, as_json: bool, text: str) -> None:
    if as_json:
        sys.stdout.write(json.dumps(payload, ensure_ascii=False, indent=2) + "\n")
    else:
        sys.stdout.write(text + "\n")


def cmd_validate(args) -> int:
    try:
        doc = load_json_file(args.session)
    except SessionError as exc:
        emit(validate_report([str(exc)]), args.json, render_validate([str(exc)]))
        return FAIL_EXIT
    issues = validate_session(doc)
    sid = doc.get("id") if isinstance(doc, dict) else None
    emit(validate_report(issues, sid), args.json, render_validate(issues, sid))
    return OK_EXIT if not issues else FAIL_EXIT


def cmd_score_replay(args) -> int:
    """픽스처 JSONL 만으로 채점한다. rhwp 바이너리를 부르지 않는다."""
    session = None
    try:
        session = load_session_file(args.session)
        events = load_trace_file(args.replay)
    except SessionError as exc:
        sid = None
        if isinstance(session, dict):
            sid = session.get("id")
        report = fail_score_report(
            classify_exception(exc),
            str(exc),
            session_id=sid,
        )
        emit(report, args.json, render_score(report))
        return FAIL_EXIT
    ctx = SessionContext.from_session(session, input_path=args.input, sub_dir=args.sub_dir)
    report = score_session(session, events, ctx)
    emit(report, args.json, render_score(report))
    return OK_EXIT if report["ok"] else FAIL_EXIT


def cmd_record(args) -> int:
    try:
        require_record_bin(args.bin)
        session = load_session_file(args.session)
        ctx = SessionContext.from_session(session, input_path=args.input, sub_dir=args.sub_dir)
        events = record_session(session, args.bin, out_path=args.out, context=ctx)
    except RecordRefused as exc:
        sys.stderr.write(str(exc) + "\n")
        return USAGE_EXIT
    except SessionError as exc:
        sys.stderr.write(str(exc) + "\n")
        return getattr(exc, "exit_code", FAIL_EXIT)
    report = score_session(session, events, ctx)
    payload = {
        "kind": RECORD_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": report["ok"],
        "sessionId": session["id"],
        "out": args.out,
        "eventCount": len(events),
        "score": report,
    }
    emit(payload, args.json, render_score(report))
    return OK_EXIT if report["ok"] else FAIL_EXIT


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(
        description="gym 에이전트 세션 트레이스 — 선언된 명령 열을 기록·재생 채점"
    )
    sub = ap.add_subparsers(dest="cmd")

    p_val = sub.add_parser("validate", help="세션 정의 JSON 검증 (바이너리 불요)")
    p_val.add_argument("--session", required=True, help="세션 정의 JSON")
    p_val.add_argument("--json", action="store_true")
    p_val.set_defaults(func=cmd_validate)

    p_rep = sub.add_parser("score-replay", help="기록된 JSONL 을 바이너리 없이 채점")
    p_rep.add_argument("--session", required=True, help="세션 정의 JSON")
    p_rep.add_argument("--replay", required=True, help="트레이스 JSONL")
    p_rep.add_argument("--input", default=None, help="세션 input 덮어쓰기")
    p_rep.add_argument("--sub-dir", default=None, dest="sub_dir", help="작업 폴더")
    p_rep.add_argument("--json", action="store_true")
    p_rep.set_defaults(func=cmd_score_replay)

    p_rec = sub.add_parser("record", help="세션을 실행해 JSONL 기록 (--bin 필수)")
    p_rec.add_argument("--session", required=True, help="세션 정의 JSON")
    p_rec.add_argument("--bin", default=None, help="rhwp 실행 파일. 없으면 거절")
    p_rec.add_argument("--out", required=True, help="트레이스 JSONL 출력 경로")
    p_rec.add_argument("--input", default=None, help="세션 input 덮어쓰기")
    p_rec.add_argument("--sub-dir", default=None, dest="sub_dir", help="작업 폴더")
    p_rec.add_argument("--json", action="store_true")
    p_rec.set_defaults(func=cmd_record)

    return ap


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if not getattr(args, "cmd", None):
        parser.print_help()
        return USAGE_EXIT
    return args.func(args)


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
