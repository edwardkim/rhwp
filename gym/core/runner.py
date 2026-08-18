"""[#4653] pack 인지 채점 엔진.

원칙(1부부터 이어지는 것):
1) 정답을 골든 파일로 박제하지 않는다 — 기대값은 채점 시점에 rhwp 로 라이브
   재계산한다.
2) 산출물 과제는 제출 파일을 rhwp 로 재검증한다.
3) 표준 라이브러리 전용, Windows/리눅스 경로 안전, 실패도 데이터.

pack 확장이 더한 것:
4) **점수는 pack 별로 보존한다** — 하나의 거대한 만점으로 합치면 어느 능력이
   모자란지 사라진다. profile 은 pack 을 고르는 도구이지 점수를 뭉치는
   도구가 아니다.
5) **부재는 실패가 아니다** — pack 이 요구하는 명령이 바이너리에 없으면
   `unavailable` 로 보고한다.
6) **점수에는 신원이 붙는다** — 실행 바이너리의 version·commit·capabilities
   digest 를 스코어카드에 남기고 pack 의 기준 실행과 대조한다.

예외 경로(#5260):
7) **침묵하지 않는다** — pack 로드·프로파일·과제 JSON·answer.json·CLI 실행이
   죽어도 채점 전체가 멈추지 않는다. 그 자리는 `status=error` 또는 과제
   `error`/`kind` 로 남긴다. 오류를 unavailable(명령 부재) 이나 0점으로
   위장하지 않는다.
8) **치명 예외는 삼키지 않는다** — KeyboardInterrupt·SystemExit·MemoryError·
   GeneratorExit 는 도구를 죽이는 것이 정직하다.
9) **식별자는 경로가 아니다** — pack id·profile id 의 `..` / 구분자 /
   절대경로는 `unsafe-id` 다. `{file:}`·`{sha256:}` 자리표시자도 상위
   디렉터리로 나가지 못한다.
"""

from __future__ import annotations

import io
import json
import os
import subprocess

from . import checks as check_registry
from . import schema as pack_schema

HERE = os.path.dirname(os.path.abspath(__file__))
GYM = os.path.dirname(HERE)
ROOT = os.path.dirname(GYM)
PACKS_DIR = os.path.join(GYM, "packs")
PROFILES_DIR = os.path.join(GYM, "profiles")

REPORT_KIND = "gymScorecard"
SCHEMA_VERSION = "2.0"
ADMISSION_KIND = "gymAdmission"
ADMISSION_SCHEMA = "1.0"

#: CLI stdout 머리. 예전 계약은 200 바이트가 아니라 200 글자다.
HEAD_LIMIT = 200
ERROR_HEAD_LIMIT = 160

#: 삼키면 안 되는 예외 — 도구를 죽이는 것이 정직하다.
FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

#: 운영 예외. BaseException 전부가 아니다.
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
    subprocess.SubprocessError,
)

#: pack 한 줄의 상태. scored 와 unavailable 사이에 error 를 끼워 넣지 않는다.
PACK_STATUSES = ("scored", "unavailable", "error")

#: 입장 판정. allow 는 packsScored >= 1 과만 같다.
ADMISSION_VERDICTS = ("allow", "deny")

#: 종료 코드. 새 코드를 만들지 않는다 — 0=만점, 3=그 외.
EXIT_PERFECT = 0
EXIT_IMPERFECT = 3

#: 예외 kind 카탈로그. 문서·시험이 같은 표를 본다.
EXCEPTION_KINDS = (
    "missing-bin",
    "missing-submit",
    "missing-pack",
    "missing-profile",
    "missing-file",
    "missing-tasks-dir",
    "missing-input",
    "malformed-json",
    "malformed-answer",
    "malformed-task",
    "malformed-check",
    "malformed-pack",
    "malformed-profile",
    "malformed-cmd",
    "unsafe-id",
    "permission",
    "os-error",
    "decode-error",
    "value-error",
    "type-error",
    "timeout",
    "subprocess",
    "unknown-op",
    "bad-expect-exits",
    "cli-exit",
    "envelope-parse",
    "path-eval",
    "empty-checks",
    "empty-agent",
    "write-error",
    "unexpected",
)

EXCEPTION_KIND_HELP = {
    "missing-bin": "rhwp 실행 파일이 없거나 CreateProcess 가 찾지 못함",
    "missing-submit": "과제 제출 폴더가 없음",
    "missing-pack": "pack.json 이 없거나 pack 폴더가 없음",
    "missing-profile": "profiles/<id>.json 이 없음",
    "missing-file": "제출 파일·해시 대상이 없음",
    "missing-tasks-dir": "pack 의 tasks/ 디렉터리가 없음",
    "missing-input": "과제에 input 이 없는데 {input} 자리표시자를 씀",
    "malformed-json": "JSON 파싱 실패",
    "malformed-answer": "answer.json 이 객체가 아니거나 깨짐",
    "malformed-task": "과제 JSON 이 객체가 아니거나 필수 키가 없음",
    "malformed-check": "체크가 객체가 아니거나 op 가 없음",
    "malformed-pack": "pack.json 이 객체가 아니거나 필수 키가 없음",
    "malformed-profile": "프로파일이 객체가 아니거나 packs 가 목록이 아님",
    "malformed-cmd": "cmd 가 문자열 목록이 아님",
    "unsafe-id": "pack/profile/파일 자리에 경로 구분자나 .. 가 있음",
    "permission": "읽기·쓰기·실행 권한 없음",
    "os-error": "그 밖의 OSError",
    "decode-error": "UTF-8 디코드 실패",
    "value-error": "값 형태가 기대와 다름",
    "type-error": "타입 불일치",
    "timeout": "대기 한도를 넘김",
    "subprocess": "자식 프로세스 기동·통신 실패",
    "unknown-op": "체크 연산자가 레지스트리에 없음",
    "bad-expect-exits": "expect_exits 가 비지 않은 정수 목록이 아님",
    "cli-exit": "rhwp 종료 코드가 허용 집합 밖",
    "envelope-parse": "stdout 이 JSON 봉투가 아님",
    "path-eval": "봉투 경로 평가(KeyError/IndexError/TypeError)",
    "empty-checks": "과제 checks 가 비어 통과할 칸이 없음",
    "empty-agent": "agent 이름이 비었음",
    "write-error": "scorecard/report/admission 기록 실패",
    "unexpected": "분류되지 않은 운영 예외",
}

SCORECARD_KEYS = (
    "kind",
    "schemaVersion",
    "profile",
    "runner",
    "total",
    "packs",
)

TOTAL_KEYS = (
    "score",
    "max",
    "packsScored",
    "packsUnavailable",
    "packsErrored",
)

OPTIONAL_SCORECARD_KEYS = (
    "agent",
    "exceptions",
    "exceptionCount",
    "binPath",
    "binMissing",
    "trusted",
)

TASK_SHAPE_KEYS = ("id", "tier", "title", "checks")

UNSAFE_ID_CHARS = ("/", "\\", ":", "\x00")


class ScoreRunnerError(Exception):
    """채점기가 접는 운영 예외. kind 는 EXCEPTION_KINDS 중 하나."""

    def __init__(self, kind, message, **extra):
        if kind not in EXCEPTION_KINDS:
            kind = "unexpected"
        super().__init__(message)
        self.kind = kind
        self.message = message
        self.extra = extra

    def as_row(self, where=""):
        return exception_row(self.kind, where=where, message=self.message,
                             extra=self.extra)


def is_fatal_exception(exc):
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def is_catchable_exception(exc):
    """운영 예외로 접을 수 있는가. 치명 예외는 아니다."""
    if exc is None or is_fatal_exception(exc):
        return False
    return isinstance(exc, CATCHABLE_EXCEPTIONS) or isinstance(exc, ScoreRunnerError)


def is_known_exception_kind(kind):
    return kind in EXCEPTION_KINDS


def is_known_pack_status(status):
    return status in PACK_STATUSES


def describe_exception_kind(kind):
    """kind 한 줄 설명. 모르는 값은 unexpected 설명."""
    if kind in EXCEPTION_KIND_HELP:
        return EXCEPTION_KIND_HELP[kind]
    return EXCEPTION_KIND_HELP["unexpected"]


def truncate_head(text, limit=HEAD_LIMIT):
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


def error_head(exc, limit=ERROR_HEAD_LIMIT):
    """예외 메시지 머리. 예외가 아니면 문자열로 접는다."""
    if exc is None:
        return ""
    try:
        text = str(exc)
    except Exception:
        text = type(exc).__name__
    return truncate_head(text, limit)


def exception_kind(exc, context="check"):
    """예외를 kind 로 접는다. 순수.

    context:
      - bin: 바이너리 실행. FileNotFound → missing-bin
      - pack: pack.json 로드. FileNotFound → missing-pack
      - profile: 프로파일 로드. FileNotFound → missing-profile
      - submit: 제출 폴더. FileNotFound → missing-submit
      - file: 제출 파일·해시. FileNotFound → missing-file
      - answer: answer.json. JSONDecodeError → malformed-answer
      - check: 체크 평가. KeyError → path-eval
      - write: 산출 기록. OSError → write-error
    """
    if isinstance(exc, ScoreRunnerError):
        return exc.kind if is_known_exception_kind(exc.kind) else "unexpected"
    if exc is None:
        return "unexpected"
    if isinstance(exc, FileNotFoundError):
        return {
            "bin": "missing-bin",
            "pack": "missing-pack",
            "profile": "missing-profile",
            "submit": "missing-submit",
            "file": "missing-file",
            "answer": "missing-file",
            "check": "missing-file",
            "write": "write-error",
        }.get(context, "missing-file")
    if isinstance(exc, PermissionError):
        return "permission" if context != "write" else "write-error"
    if isinstance(exc, json.JSONDecodeError):
        if context == "answer":
            return "malformed-answer"
        if context == "pack":
            return "malformed-pack"
        if context == "profile":
            return "malformed-profile"
        return "malformed-json"
    if isinstance(exc, UnicodeError):
        return "decode-error"
    if isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, subprocess.SubprocessError):
        return "subprocess"
    if isinstance(exc, AttributeError):
        return "type-error"
    if isinstance(exc, TypeError):
        return "path-eval" if context == "check" else "type-error"
    if isinstance(exc, (KeyError, IndexError)):
        return "path-eval" if context == "check" else "value-error"
    if isinstance(exc, ValueError):
        if context == "answer":
            return "malformed-answer"
        return "value-error"
    if isinstance(exc, OSError):
        return "write-error" if context == "write" else "os-error"
    if isinstance(exc, RuntimeError):
        return "unexpected"
    return "unexpected"


def exception_row(kind, where="", message="", extra=None):
    """예외 한 줄. 점수 집계를 뒤집지 않는다."""
    if not is_known_exception_kind(kind):
        kind = "unexpected"
    row = {
        "kind": kind,
        "where": where or "",
        "message": truncate_head(message, ERROR_HEAD_LIMIT),
    }
    if isinstance(extra, dict):
        for key, value in extra.items():
            if key in row:
                continue
            row[key] = value
    return row


def wrap_exception(exc, context="check", where=""):
    """운영 예외를 ScoreRunnerError 로. 치명 예외는 다시 던진다."""
    if is_fatal_exception(exc):
        raise exc
    if isinstance(exc, ScoreRunnerError):
        return exc
    kind = exception_kind(exc, context)
    return ScoreRunnerError(kind, error_head(exc), where=where,
                            type=type(exc).__name__)


def is_safe_id(name):
    """pack/profile 한 칸 이름. 구분자·절대경로·.. 금지."""
    if not isinstance(name, str) or not name:
        return False
    if name in (".", ".."):
        return False
    if any(ch in name for ch in UNSAFE_ID_CHARS):
        return False
    return all(ch.isalnum() or ch in "-_" for ch in name)


def is_safe_relpath(name):
    """제출 파일 상대경로. 상위 디렉터리와 절대경로를 거부한다."""
    if not isinstance(name, str) or not name:
        return False
    if os.path.isabs(name):
        return False
    if name.startswith("/") or name.startswith("\\"):
        return False
    normalized = name.replace("\\", "/")
    if ":" in normalized.split("/")[0]:
        return False
    parts = normalized.split("/")
    if any(part in ("", ".", "..") for part in parts):
        return False
    if any("\x00" in part for part in parts):
        return False
    return True


def unsafe_id_reason(name, label="id"):
    if not isinstance(name, str):
        return f"{label} 가 문자열이 아니다"
    if not name:
        return f"{label} 가 비었다"
    if not is_safe_id(name):
        return f"{label} 가 안전하지 않다: {name!r}"
    return ""


def require_safe_id(name, label="id"):
    reason = unsafe_id_reason(name, label)
    if reason:
        raise ScoreRunnerError("unsafe-id", reason, id=name, label=label)
    return name


def bin_looks_like_path(bin_path):
    """경로형 바이너리인가. 맨이름(rhwp)은 PATH 조회라 부재 단정 금지."""
    if not isinstance(bin_path, str) or not bin_path:
        return False
    if os.path.isabs(bin_path):
        return True
    sep = os.path.sep
    alt = os.path.altsep
    if sep in bin_path or (alt and alt in bin_path):
        return True
    return False


def bin_is_missing(bin_path):
    """경로형 바이너리가 디스크에 없는가. 맨이름은 False."""
    if not isinstance(bin_path, str) or not bin_path:
        return True
    if not bin_looks_like_path(bin_path):
        return False
    return not os.path.exists(bin_path)


def find_bin(cli_arg):
    """--bin > RHWP_BIN > target 기본값. 상대경로는 절대화한다 — Windows
    CreateProcess 는 자식 cwd 가 아니라 부모 cwd 기준으로 상대 실행파일을
    찾으므로, 절대화 없이는 WinError 2 로 전 과제가 무너진다(1호 주행 실측)."""
    cand = cli_arg or os.environ.get("RHWP_BIN")
    if cand:
        if not isinstance(cand, str):
            try:
                cand = str(cand)
            except Exception:
                return "rhwp"
        cand = cand.replace("/", os.sep)
        if os.path.isabs(cand):
            return cand
        for base in (os.getcwd(), ROOT):
            p = os.path.abspath(os.path.join(base, cand))
            if os.path.exists(p):
                return p
        return cand
    for rel in ("target/debug/rhwp.exe", "target/debug/rhwp",
                "target/release/rhwp.exe", "target/release/rhwp"):
        p = os.path.join(ROOT, rel.replace("/", os.sep))
        if os.path.exists(p):
            return p
    return "rhwp"


def coerce_str_list(value, where="cmd"):
    """문자열 목록으로. 아니면 ScoreRunnerError(malformed-cmd)."""
    if isinstance(value, tuple):
        value = list(value)
    if not isinstance(value, list):
        raise ScoreRunnerError("malformed-cmd", f"{where} 가 목록이 아니다",
                               value=type(value).__name__)
    out = []
    for i, item in enumerate(value):
        if not isinstance(item, str):
            raise ScoreRunnerError(
                "malformed-cmd",
                f"{where}[{i}] 가 문자열이 아니다",
                index=i,
            )
        out.append(item)
    return out


def prepare_cli(bin_path, args):
    """subprocess 인자 검증. 실행은 하지 않는다."""
    if not isinstance(bin_path, str) or not bin_path:
        raise ScoreRunnerError("missing-bin", "바이너리 경로가 비었다")
    argv = coerce_str_list(args, "args")
    return [bin_path] + argv


def decode_cli_stdout(raw):
    """stdout 바이트 → 텍스트. 깨진 바이트는 교체한다."""
    if raw is None:
        return ""
    if isinstance(raw, str):
        return raw
    try:
        return raw.decode("utf-8", errors="replace")
    except (AttributeError, TypeError, UnicodeError):
        try:
            return str(raw)
        except Exception:
            return ""


def parse_envelope(text):
    """stdout 원문을 JSON 객체로. 실패·비객체는 None."""
    if not isinstance(text, str) or not text.strip():
        return None
    try:
        env = json.loads(text)
    except ValueError:
        return None
    if not isinstance(env, dict):
        return None
    return env


def run_cli(bin_path, args):
    """rhwp 실행 → (exit, 봉투 json 또는 None, stdout 원문 머리).

    FileNotFoundError·PermissionError 는 예전처럼 던진다 — eval_check 가
    `파일 없음:` 접두로 접는다. 그 밖의 운영 예외는 ScoreRunnerError.
    치명 예외는 삼키지 않는다.
    """
    argv = prepare_cli(bin_path, args)
    try:
        proc = subprocess.run(argv, cwd=ROOT, capture_output=True)
    except FileNotFoundError:
        raise
    except PermissionError:
        raise
    except subprocess.TimeoutExpired as e:
        raise ScoreRunnerError("timeout", error_head(e), bin=bin_path)
    except subprocess.SubprocessError as e:
        raise ScoreRunnerError("subprocess", error_head(e), bin=bin_path)
    except OSError as e:
        raise ScoreRunnerError("os-error", error_head(e), bin=bin_path)
    out = decode_cli_stdout(proc.stdout)
    env = parse_envelope(out)
    return proc.returncode, env, truncate_head(out, HEAD_LIMIT)


def resolve_placeholder(token, task, sub_dir):
    """자리표시자 하나. 해당 없으면 원문 그대로."""
    if token == "{input}":
        if not isinstance(task, dict) or "input" not in task:
            raise ScoreRunnerError("missing-input", "{input} 자리인데 과제에 input 이 없다")
        value = task["input"]
        if not isinstance(value, str) or not value:
            raise ScoreRunnerError("missing-input", "과제 input 이 비었다")
        return value
    if token.startswith("{file:") and token.endswith("}"):
        name = token[6:-1]
        if not is_safe_relpath(name):
            raise ScoreRunnerError("unsafe-id", f"파일 자리표시자가 안전하지 않다: {name!r}")
        return os.path.join(sub_dir, name)
    if token.startswith("{sha256:") and token.endswith("}"):
        name = token[8:-1]
        if not is_safe_relpath(name):
            raise ScoreRunnerError("unsafe-id", f"해시 자리표시자가 안전하지 않다: {name!r}")
        path = os.path.join(sub_dir, name)
        try:
            return check_registry.sha256_of(path)
        except FileNotFoundError:
            raise ScoreRunnerError("missing-file", f"파일 없음: {path}")
    return token


def resolve_args(cmd, task, sub_dir):
    out = []
    for a in coerce_str_list(cmd, "cmd"):
        out.append(resolve_placeholder(a, task, sub_dir))
    return out


class CheckContext:
    """연산자가 보는 세계 — 제출 폴더·과제·봉투."""

    def __init__(self, check, task, sub_dir, answer, envelope):
        self.check = check
        self.task = task
        self.sub_dir = sub_dir
        self.answer = answer
        self.envelope = envelope

    def sub_path(self, name):
        return os.path.join(self.sub_dir, name)

    def root_path(self, name):
        return os.path.join(ROOT, name)

    def dug(self):
        return check_registry.dig(self.envelope, self.check.get("path", ""))


def validate_expect_exits(raw, fallback=0):
    """expect_exits 또는 단일 expect_exit 를 정수 목록으로.

    잘못된 값은 (None, 오류문자열). 예전 계약: 목록이 아니거나 비었거나
    정수가 아닌 칸이 있으면 거부.
    """
    if raw is None:
        if type(fallback) is not int:
            return None, f"잘못된 expect_exits: {raw!r}"
        return [fallback], None
    if not isinstance(raw, list) or not raw or any(type(v) is not int for v in raw):
        return None, f"잘못된 expect_exits: {raw!r}"
    return raw, None


def check_name_of(check, op=None):
    if isinstance(check, dict):
        return check.get("name", op if op is not None else check.get("op"))
    return op


def failed_check(name, op, error, kind):
    if not is_known_exception_kind(kind):
        kind = "unexpected"
    return {"name": name, "op": op, "ok": False, "error": error, "kind": kind}


def eval_check(check, task, sub_dir, answer, bin_path):
    if not isinstance(check, dict):
        return failed_check(None, None, "체크가 객체가 아니다", "malformed-check")
    op = check.get("op")
    detail = {"name": check.get("name", op), "op": op, "ok": False}
    if not op:
        detail["error"] = "op 없음"
        detail["kind"] = "malformed-check"
        return detail
    entry = check_registry.REGISTRY.get(op)
    if entry is None:
        detail["error"] = f"미지 op: {op}"
        detail["kind"] = "unknown-op"
        return detail
    fn, uses_cli = entry
    try:
        envelope = None
        if uses_cli:
            cmd = check.get("cmd")
            args = resolve_args(cmd, task, sub_dir)
            code, envelope, head = run_cli(bin_path, args)
            expect_exits, expect_err = validate_expect_exits(
                check.get("expect_exits"), check.get("expect_exit", 0))
            if expect_err:
                detail["error"] = expect_err
                detail["kind"] = "bad-expect-exits"
                return detail
            if code not in expect_exits:
                detail["error"] = f"exit {code} (허용 {expect_exits}): {head}"
                detail["kind"] = "cli-exit"
                return detail
            if envelope is None:
                detail["error"] = f"봉투 파싱 실패: {head}"
                detail["kind"] = "envelope-parse"
                return detail
        detail.update(fn(CheckContext(check, task, sub_dir, answer, envelope)))
    except FileNotFoundError as e:
        detail["error"] = f"파일 없음: {e}"
        detail["kind"] = exception_kind(e, "bin" if uses_cli else "file")
    except PermissionError as e:
        detail["error"] = f"권한 없음: {e}"
        detail["kind"] = "permission"
    except ScoreRunnerError as e:
        detail["error"] = e.message
        detail["kind"] = e.kind
    except (KeyError, IndexError, TypeError) as e:
        detail["error"] = f"경로 평가 실패: {type(e).__name__} {e}"
        detail["kind"] = "path-eval"
    except UnicodeError as e:
        detail["error"] = f"디코드 실패: {e}"
        detail["kind"] = "decode-error"
    except CATCHABLE_EXCEPTIONS as e:
        if is_fatal_exception(e):
            raise
        detail["error"] = f"{type(e).__name__}: {e}"
        detail["kind"] = exception_kind(e, "check")
    return detail


def empty_task_result(task_id=None, tier=None, title=None, error="", kind=""):
    result = {
        "id": task_id,
        "tier": 0 if not isinstance(tier, int) else tier,
        "title": title if isinstance(title, str) else "",
        "pass": False,
        "checks": [],
    }
    if error:
        result["error"] = error
    if kind:
        result["kind"] = kind if is_known_exception_kind(kind) else "unexpected"
    return result


def task_shape_error(task):
    """과제 뼈대가 채점에 쓰일 수 없으면 이유. 쓸 수 있으면 빈 문자열."""
    if not isinstance(task, dict):
        return "과제가 객체가 아니다"
    missing = [key for key in TASK_SHAPE_KEYS if key not in task]
    if missing:
        return "필수 키 없음: " + ", ".join(missing)
    if not isinstance(task.get("id"), str) or not task.get("id"):
        return "id 가 비었다"
    if not isinstance(task.get("title"), str):
        return "title 이 문자열이 아니다"
    if type(task.get("tier")) is not int:
        return "tier 가 정수가 아니다"
    checks = task.get("checks")
    if not isinstance(checks, list):
        return "checks 가 목록이 아니다"
    return ""


def load_json_value(path):
    """JSON 값 하나. 실패는 예외."""
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def load_json_object(path, kind_if_not_object="malformed-json"):
    value = load_json_value(path)
    if not isinstance(value, dict):
        raise ScoreRunnerError(kind_if_not_object,
                               f"{os.path.basename(path)} 이 객체가 아니다")
    return value


def read_answer_json(ans_path):
    """answer.json → dict. 없거나 객체 아니면 예외."""
    try:
        value = load_json_value(ans_path)
    except FileNotFoundError:
        raise
    except json.JSONDecodeError as e:
        raise ScoreRunnerError("malformed-answer", f"answer.json 파싱 실패: {e}")
    except ValueError as e:
        raise ScoreRunnerError("malformed-answer", f"answer.json 파싱 실패: {e}")
    except UnicodeError as e:
        raise ScoreRunnerError("decode-error", f"answer.json 디코드 실패: {e}")
    except PermissionError as e:
        raise ScoreRunnerError("permission", f"answer.json 권한 없음: {e}")
    except OSError as e:
        raise ScoreRunnerError("os-error", f"answer.json 읽기 실패: {e}")
    if not isinstance(value, dict):
        raise ScoreRunnerError("malformed-answer", "answer.json 이 객체가 아니다")
    return value


def score_task(task, sub_root, bin_path):
    shape = task_shape_error(task)
    if shape:
        ident = task.get("id") if isinstance(task, dict) else None
        title = task.get("title") if isinstance(task, dict) else None
        tier = task.get("tier") if isinstance(task, dict) else None
        return empty_task_result(ident, tier, title, shape, "malformed-task")
    sub_dir = os.path.join(sub_root, task["id"])
    result = {"id": task["id"], "tier": task["tier"], "title": task["title"],
              "pass": False, "checks": []}
    if not os.path.isdir(sub_dir):
        result["error"] = "제출 폴더 없음"
        result["kind"] = "missing-submit"
        return result
    answer = {}
    ans_path = os.path.join(sub_dir, "answer.json")
    if os.path.exists(ans_path):
        try:
            answer = read_answer_json(ans_path)
        except ScoreRunnerError as e:
            result["error"] = e.message
            result["kind"] = e.kind
            return result
        except ValueError as e:
            result["error"] = f"answer.json 파싱 실패: {e}"
            result["kind"] = "malformed-answer"
            return result
    checks = task["checks"]
    if not checks:
        result["error"] = "checks 가 비었다"
        result["kind"] = "empty-checks"
        return result
    for check in checks:
        result["checks"].append(eval_check(check, task, sub_dir, answer, bin_path))
    result["pass"] = bool(result["checks"]) and all(c.get("ok") for c in result["checks"])
    return result


def load_pack(pack_id):
    require_safe_id(pack_id, "pack")
    pack_dir = os.path.join(PACKS_DIR, pack_id)
    pack_json = os.path.join(pack_dir, "pack.json")
    if not os.path.isfile(pack_json):
        raise ScoreRunnerError("missing-pack", f"pack.json 이 없다: {pack_id}")
    try:
        manifest = load_json_object(pack_json, "malformed-pack")
    except FileNotFoundError:
        raise ScoreRunnerError("missing-pack", f"pack.json 이 없다: {pack_id}")
    except ScoreRunnerError:
        raise
    except json.JSONDecodeError as e:
        raise ScoreRunnerError("malformed-pack", f"pack.json 파싱 실패: {e}")
    except ValueError as e:
        raise ScoreRunnerError("malformed-pack", f"pack.json 파싱 실패: {e}")
    except PermissionError as e:
        raise ScoreRunnerError("permission", f"pack.json 권한 없음: {e}")
    except OSError as e:
        raise ScoreRunnerError("os-error", f"pack.json 읽기 실패: {e}")
    if not manifest.get("title"):
        raise ScoreRunnerError("malformed-pack", f"{pack_id}: title 이 비었다")
    if not manifest.get("axis"):
        raise ScoreRunnerError("malformed-pack", f"{pack_id}: axis 가 비었다")
    tasks_dir = os.path.join(pack_dir, "tasks")
    if not os.path.isdir(tasks_dir):
        raise ScoreRunnerError("missing-tasks-dir", f"{pack_id}: tasks/ 가 없다")
    tasks = []
    try:
        names = sorted(os.listdir(tasks_dir))
    except PermissionError as e:
        raise ScoreRunnerError("permission", f"{pack_id}: tasks/ 목록 실패: {e}")
    except OSError as e:
        raise ScoreRunnerError("os-error", f"{pack_id}: tasks/ 목록 실패: {e}")
    for name in names:
        if not name.endswith(".json"):
            continue
        path = os.path.join(tasks_dir, name)
        try:
            task = load_json_value(path)
        except json.JSONDecodeError as e:
            raise ScoreRunnerError("malformed-task", f"{pack_id}/{name} 파싱 실패: {e}")
        except ValueError as e:
            raise ScoreRunnerError("malformed-task", f"{pack_id}/{name} 파싱 실패: {e}")
        except PermissionError as e:
            raise ScoreRunnerError("permission", f"{pack_id}/{name} 권한 없음: {e}")
        except OSError as e:
            raise ScoreRunnerError("os-error", f"{pack_id}/{name} 읽기 실패: {e}")
        if not isinstance(task, dict):
            raise ScoreRunnerError("malformed-task", f"{pack_id}/{name} 이 객체가 아니다")
        tasks.append(task)
    return manifest, tasks


def discover_packs():
    if not os.path.isdir(PACKS_DIR):
        return []
    try:
        names = os.listdir(PACKS_DIR)
    except OSError:
        return []
    found = []
    for name in names:
        if not is_safe_id(name):
            continue
        if os.path.isfile(os.path.join(PACKS_DIR, name, "pack.json")):
            found.append(name)
    return sorted(found)


def load_profile(profile_id):
    require_safe_id(profile_id, "profile")
    path = os.path.join(PROFILES_DIR, f"{profile_id}.json")
    if not os.path.isfile(path):
        raise ScoreRunnerError("missing-profile", f"프로파일이 없다: {profile_id}")
    try:
        profile = load_json_object(path, "malformed-profile")
    except FileNotFoundError:
        raise ScoreRunnerError("missing-profile", f"프로파일이 없다: {profile_id}")
    except ScoreRunnerError:
        raise
    except json.JSONDecodeError as e:
        raise ScoreRunnerError("malformed-profile", f"프로파일 파싱 실패: {e}")
    except ValueError as e:
        raise ScoreRunnerError("malformed-profile", f"프로파일 파싱 실패: {e}")
    except PermissionError as e:
        raise ScoreRunnerError("permission", f"프로파일 권한 없음: {e}")
    except OSError as e:
        raise ScoreRunnerError("os-error", f"프로파일 읽기 실패: {e}")
    packs = profile.get("packs")
    if not isinstance(packs, list) or not packs:
        raise ScoreRunnerError("malformed-profile", f"{profile_id}: packs 가 비었다")
    for item in packs:
        if not is_safe_id(item):
            raise ScoreRunnerError("unsafe-id", f"{profile_id}: 안전하지 않은 pack {item!r}")
    return profile


def safe_known_commands(bin_path):
    """capabilities 조회. 실패는 None — 예전 parse 실패와 같다."""
    try:
        return pack_schema.known_commands(bin_path)
    except CATCHABLE_EXCEPTIONS:
        return None


def safe_runner_identity(bin_path):
    """실행 신원. 바이너리 부재여도 빈 칸으로 카드를 만든다."""
    try:
        ident = pack_schema.runner_identity(bin_path, ROOT)
    except CATCHABLE_EXCEPTIONS as e:
        return {
            "rhwpVersion": "",
            "rhwpCommit": "",
            "capabilitiesSha256": "",
            "kind": exception_kind(e, "bin"),
            "error": error_head(e),
        }
    if not isinstance(ident, dict):
        return {"rhwpVersion": "", "rhwpCommit": "", "capabilitiesSha256": ""}
    for key in ("rhwpVersion", "rhwpCommit", "capabilitiesSha256"):
        if key not in ident or ident[key] is None:
            ident[key] = ""
    return ident


def error_pack_entry(pack_id, exc, where="pack"):
    if isinstance(exc, ScoreRunnerError):
        kind = exc.kind
        message = exc.message
    else:
        kind = exception_kind(exc, "pack")
        message = error_head(exc)
    return {
        "id": pack_id,
        "title": "",
        "axis": "",
        "max": 0,
        "taskCount": 0,
        "status": "error",
        "score": None,
        "passed": 0,
        "tasks": [],
        "kind": kind,
        "error": message,
        "where": where,
    }


def task_tier(task):
    if isinstance(task, dict) and type(task.get("tier")) is int:
        return task["tier"]
    return 0


def score_pack(pack_id, sub_root, bin_path, available):
    """pack 하나 채점 — 요구 명령이 없으면 unavailable(0점 아님).

    로드 실패는 status=error. 점수는 None. unavailable 로 부르지 않는다.
    """
    try:
        manifest, tasks = load_pack(pack_id)
    except FATAL_EXCEPTIONS:
        raise
    except (ScoreRunnerError,) + CATCHABLE_EXCEPTIONS as e:
        return error_pack_entry(pack_id, e, "load_pack")
    maximum = sum(task_tier(t) for t in tasks)
    missing = []
    if available is not None:
        commands = manifest.get("requires", {}).get("commands", [])
        if not isinstance(commands, list):
            commands = []
        missing = [c for c in commands if c not in available]
    entry = {"id": pack_id, "title": manifest.get("title", ""),
             "axis": manifest.get("axis", ""),
             "max": maximum, "taskCount": len(tasks)}
    if missing:
        # 부재를 실패로 위장하지 않는다 — 오래된 바이너리에게 "0점"은 거짓말이다.
        entry.update({"status": "unavailable", "missingCommands": missing,
                      "score": None, "passed": 0, "tasks": []})
        return entry
    pack_sub = os.path.join(sub_root, pack_id)
    root_for_tasks = pack_sub if os.path.isdir(pack_sub) else sub_root
    results = []
    for t in tasks:
        try:
            results.append(score_task(t, root_for_tasks, bin_path))
        except FATAL_EXCEPTIONS:
            raise
        except CATCHABLE_EXCEPTIONS as e:
            ident = t.get("id") if isinstance(t, dict) else None
            results.append(empty_task_result(
                ident,
                task_tier(t) if isinstance(t, dict) else None,
                t.get("title") if isinstance(t, dict) else None,
                error_head(e),
                exception_kind(e, "check"),
            ))
    entry.update({"status": "scored",
                  "score": sum(r.get("tier", 0) for r in results if r.get("pass")),
                  "passed": sum(1 for r in results if r.get("pass")),
                  "tasks": results})
    return entry


def normalize_pack_ids(pack_ids):
    """호출자가 준 pack 목록. None/빈 값은 None(탐색 신호)."""
    if pack_ids is None:
        return None
    if isinstance(pack_ids, str):
        pack_ids = [pack_ids]
    if not isinstance(pack_ids, (list, tuple)):
        raise ScoreRunnerError("value-error", "pack_ids 가 목록이 아니다")
    out = []
    for item in pack_ids:
        if not isinstance(item, str) or not item:
            raise ScoreRunnerError("unsafe-id", f"빈 pack id: {item!r}")
        require_safe_id(item, "pack")
        out.append(item)
    return out


def empty_scorecard(profile_id=None, bin_path="", runner=None, exceptions=None):
    rows = list(exceptions or [])
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "profile": profile_id,
        "runner": runner or {"rhwpVersion": "", "rhwpCommit": "",
                             "capabilitiesSha256": ""},
        "total": {"score": 0, "max": 0, "packsScored": 0,
                  "packsUnavailable": 0, "packsErrored": 0,
                  "exceptionCount": len(rows)},
        "packs": [],
        "exceptions": rows,
        "exceptionCount": len(rows),
        "binPath": bin_path or "",
        "binMissing": bin_is_missing(bin_path) if bin_path else False,
        "trusted": len(rows) == 0,
    }


def attach_card_counts(card):
    """total·예외 집계를 packs 와 맞춘다. 카드를 그 자리에서 고친다."""
    packs = card.get("packs") if isinstance(card.get("packs"), list) else []
    scored = [p for p in packs if p.get("status") == "scored"]
    unavailable = [p for p in packs if p.get("status") == "unavailable"]
    errored = [p for p in packs if p.get("status") == "error"]
    exceptions = card.get("exceptions") if isinstance(card.get("exceptions"), list) else []
    total = card.get("total") if isinstance(card.get("total"), dict) else {}
    total["score"] = sum(p.get("score") or 0 for p in scored)
    total["max"] = sum(p.get("max") or 0 for p in scored)
    total["packsScored"] = len(scored)
    total["packsUnavailable"] = len(unavailable)
    total["packsErrored"] = len(errored)
    total["exceptionCount"] = len(exceptions)
    card["total"] = total
    card["exceptionCount"] = len(exceptions)
    card["trusted"] = len(exceptions) == 0 and len(errored) == 0
    return card


def validate_scorecard(card):
    """카드 뼈대. 문제 목록. 비면 통과."""
    errors = []
    if not isinstance(card, dict):
        return ["스코어카드가 객체가 아니다"]
    if card.get("kind") != REPORT_KIND:
        errors.append(f"kind 가 {REPORT_KIND} 가 아니다")
    if card.get("schemaVersion") != SCHEMA_VERSION:
        errors.append(f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다")
    for key in SCORECARD_KEYS:
        if key not in card:
            errors.append(f"필수 키 없음: {key}")
    total = card.get("total")
    if not isinstance(total, dict):
        errors.append("total 이 객체가 아니다")
    else:
        for key in TOTAL_KEYS:
            if key not in total:
                errors.append(f"total.{key} 없음")
    packs = card.get("packs")
    if not isinstance(packs, list):
        errors.append("packs 가 목록이 아니다")
    else:
        for i, pack in enumerate(packs):
            if not isinstance(pack, dict):
                errors.append(f"packs[{i}] 가 객체가 아니다")
                continue
            status = pack.get("status")
            if not is_known_pack_status(status):
                errors.append(f"packs[{i}].status 가 아니다: {status!r}")
    return errors


def score_all(sub_root, bin_path, pack_ids=None, profile_id=None):
    exceptions = []
    runner_ident = safe_runner_identity(bin_path)
    available = safe_known_commands(bin_path)
    if bin_is_missing(bin_path):
        exceptions.append(exception_row(
            "missing-bin", where="bin",
            message=f"경로형 바이너리가 없다: {bin_path}"))
    selected = None
    if profile_id:
        try:
            profile = load_profile(profile_id)
            selected = list(profile["packs"])
        except FATAL_EXCEPTIONS:
            raise
        except (ScoreRunnerError,) + CATCHABLE_EXCEPTIONS as e:
            err = wrap_exception(e, "profile", "profile")
            exceptions.append(err.as_row("profile"))
            card = empty_scorecard(profile_id, bin_path, runner_ident, exceptions)
            return attach_card_counts(card)
    if selected is None:
        try:
            selected = normalize_pack_ids(pack_ids)
        except FATAL_EXCEPTIONS:
            raise
        except (ScoreRunnerError,) + CATCHABLE_EXCEPTIONS as e:
            err = wrap_exception(e, "pack", "pack_ids")
            exceptions.append(err.as_row("pack_ids"))
            card = empty_scorecard(profile_id, bin_path, runner_ident, exceptions)
            return attach_card_counts(card)
    if not selected:
        try:
            selected = discover_packs()
        except CATCHABLE_EXCEPTIONS as e:
            exceptions.append(exception_row(
                exception_kind(e, "pack"), where="discover",
                message=error_head(e)))
            selected = []
    packs = []
    for pid in selected:
        try:
            packs.append(score_pack(pid, sub_root, bin_path, available))
        except FATAL_EXCEPTIONS:
            raise
        except CATCHABLE_EXCEPTIONS as e:
            packs.append(error_pack_entry(pid, e, "score_pack"))
    card = {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "profile": profile_id,
        "runner": runner_ident,
        "total": {},
        "packs": packs,
        "exceptions": exceptions,
        "binPath": bin_path or "",
        "binMissing": bin_is_missing(bin_path),
    }
    return attach_card_counts(card)


def admission_from_card(card, agent):
    """입장 봉투. packsScored >= 1 이면 allow. 만점과 무관."""
    total = card.get("total") if isinstance(card, dict) else None
    if not isinstance(total, dict):
        total = {}
    scored = total.get("packsScored") or 0
    try:
        scored = int(scored)
    except (TypeError, ValueError):
        scored = 0
    return {
        "schemaVersion": ADMISSION_SCHEMA,
        "kind": ADMISSION_KIND,
        "agent": agent,
        "verdict": "allow" if scored >= 1 else "deny",
        "packsScored": scored,
        "packsUnavailable": total.get("packsUnavailable") or 0,
        "packsErrored": total.get("packsErrored") or 0,
        "score": total.get("score") or 0,
        "max": total.get("max") or 0,
        "runner": card.get("runner") if isinstance(card, dict) else {},
    }


def pack_table_cell(pack):
    """리포트 표 한 줄."""
    pid = pack.get("id", "")
    axis = pack.get("axis", "")
    status = pack.get("status")
    if status == "unavailable":
        missing = pack.get("missingCommands") or []
        return f"| {pid} | {axis} | unavailable | 요구 명령 없음: {', '.join(missing)} |"
    if status == "error":
        err = pack.get("error") or pack.get("kind") or "error"
        return f"| {pid} | {axis} | error | {err} |"
    return (f"| {pid} | {axis} | **{pack.get('score')} / {pack.get('max')}** | "
            f"{pack.get('passed')}/{pack.get('taskCount')} 통과 |")


def task_detail_line(task):
    if not isinstance(task, dict):
        return "과제가 객체가 아니다"
    if "error" in task:
        return task["error"]
    checks = task.get("checks") or []
    return " · ".join(("O" if c.get("ok") else "X") + " " + str(c.get("name"))
                      for c in checks)


def render_report(card, agent):
    if not isinstance(card, dict):
        return f"# 짐 스코어카드 — {agent}\n\n스코어카드가 객체가 아니다"
    total = card.get("total") if isinstance(card.get("total"), dict) else {}
    score = total.get("score", 0)
    maximum = total.get("max", 0)
    scored = total.get("packsScored", 0)
    unavailable = total.get("packsUnavailable", 0)
    errored = total.get("packsErrored", 0)
    extra = ""
    if unavailable:
        extra += f" · {unavailable}개 unavailable"
    if errored:
        extra += f" · {errored}개 error"
    lines = [f"# 짐 스코어카드 — {agent}", "",
             f"**{score} / {maximum}** (pack {scored}개 채점{extra})", ""]
    r = card.get("runner") if isinstance(card.get("runner"), dict) else {}
    version = r.get("rhwpVersion") or ""
    commit = r.get("rhwpCommit") or ""
    digest = r.get("capabilitiesSha256") or ""
    lines += [f"실행 신원: rhwp {version} · commit `{commit[:12]}` "
              f"· capabilities `{digest[:12]}`", "",
              "| pack | 능력 축 | 점수 | 과제 |", "|---|---|---|---|"]
    packs = card.get("packs") if isinstance(card.get("packs"), list) else []
    for p in packs:
        if not isinstance(p, dict):
            continue
        lines.append(pack_table_cell(p))
    for p in packs:
        if not isinstance(p, dict) or p.get("status") != "scored":
            continue
        lines += ["", f"## {p.get('id')} — {p.get('title')}", "",
                  "| 과제 | 티어 | 판정 | 세부 |", "|---|---|---|---|"]
        for t in p.get("tasks") or []:
            if not isinstance(t, dict):
                continue
            det = task_detail_line(t)
            lines.append(f"| {t.get('id')} {t.get('title')} | {t.get('tier')} | "
                         f"{'통과' if t.get('pass') else '실패'} | {det} |")
    exceptions = card.get("exceptions") if isinstance(card.get("exceptions"), list) else []
    if exceptions:
        lines += ["", "## 예외", ""]
        for row in exceptions:
            if not isinstance(row, dict):
                continue
            lines.append(f"- `{row.get('kind')}` {row.get('where')}: {row.get('message')}")
    lines += ["", "채점기: gym/core/runner.py (라이브 오라클 · pack 별 점수 보존)"]
    return "\n".join(lines)


def console_pack_line(pack):
    pid = str(pack.get("id") or "")
    status = pack.get("status")
    if status == "unavailable":
        missing = pack.get("missingCommands") or []
        return f"  - {pid:<18} unavailable (없는 명령: {', '.join(missing)})"
    if status == "error":
        return f"  - {pid:<18} error ({pack.get('kind')}: {pack.get('error')})"
    return (f"  - {pid:<18} {pack.get('score')}/{pack.get('max')}  "
            f"({pack.get('passed')}/{pack.get('taskCount')} 과제)")


def format_console_summary(card, agent, card_path):
    total = card.get("total") if isinstance(card.get("total"), dict) else {}
    parts = [f"{agent}: {total.get('score')}/{total.get('max')}  "
             f"(pack {total.get('packsScored')} 채점"]
    if total.get("packsUnavailable"):
        parts.append(f", {total.get('packsUnavailable')} unavailable")
    if total.get("packsErrored"):
        parts.append(f", {total.get('packsErrored')} error")
    parts.append(f")  → {card_path}")
    lines = ["".join(parts)]
    for p in card.get("packs") or []:
        if isinstance(p, dict):
            lines.append(console_pack_line(p))
    return "\n".join(lines)


def exit_from_card(card):
    """만점이면 0, 아니면 3. 새 종료 코드를 만들지 않는다."""
    total = card.get("total") if isinstance(card, dict) else None
    if not isinstance(total, dict):
        return EXIT_IMPERFECT
    if total.get("score") == total.get("max") and total.get("packsErrored", 0) == 0:
        if total.get("max") or total.get("packsScored"):
            return EXIT_PERFECT
    return EXIT_IMPERFECT
