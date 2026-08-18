"""gym 정합 감사 — 모든 pack 이 "그 방식"(해결 가능·고유·정합)을 지키는지 전수 검사.

## 왜 이 도구인가 (강제된 표준)

gym 이 자라고 기여자가 늘수록, 새 pack 이 조용히 규약을 어길 수 있다: 과제에 기준
풀이가 없거나(=해결 가능성 미선언), 과제 ID 가 다른 pack 과 충돌하거나, 스키마를
벗어나거나. 개별 검증(`schema.validate_pack`/`validate_task`)은 pack 하나·과제 하나만
본다 — **전 저장소에 걸친 정합**(과제↔기준 짝·과제 ID 전역 고유·고아 기준풀이)은
아무도 안 본다. 그 틈으로 정합이 무너진다.

이 감사기가 그 전수 정합을 강제한다. gym 에 기여하는 모든 에이전트의 pack 은 이걸
통과해야 한다 — 벗어날 수 없되 감옥이 아니라 **품질 관문**이다(규칙은 열려 있고,
검사하는 것은 해결 가능성·고유성·정합 같은 품질이다). 바이너리 없이 순수 파일 검사라
CI 에서 상시 돈다.

## 검사하는 것 (그 방식)

- **스키마 정합** — `schema.validate_pack` + `validate_task`(바이너리 불요, 명령 존재
  검사만 러너에 위임).
- **해결 가능성 선언** — 모든 과제에 짝 기준풀이(`tasks/X.json` ↔ `reference/X.json`,
  id 일치). 기준풀이 없는 과제는 "풀 수 있다" 는 근거가 없다.
- **고아 기준풀이 없음** — 기준풀이는 반드시 과제를 가진다.
- **과제 ID 전역 고유** — pack 간 ID 충돌 금지(리더보드·집계가 ID 로 과제를 가른다).
- **pack 안 ID 중복·파일명 불일치** — 같은 pack 의 두 파일이 같은 id 를 쓰거나,
  `X01.json` 이 `id=X02` 를 들고 있으면 집계가 파일 이름으로 과제를 가른다는
  가정이 깨진다.
- **예외를 위장하지 않음** — 없는 packs 루트, 읽기 실패, 객체가 아닌 JSON 은
  도구가 죽지 않고 코드로 접힌다. 못 읽은 것을 "정합 0건" 으로 부르지 않는다.

## 사용

    python gym/tools/audit.py           # 전 pack 감사, 문제 있으면 exit 1
    python gym/tools/audit.py --json

새 플래그는 없다. `--pack` / `--out` / `--strict` 를 열지 않는다. 전 pack 을
도는 것이 이 감사의 점이다. 한 pack 만 봐서 전역 ID 충돌이 없다고 말하면
거짓말이다.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
sys.path.insert(0, GYM_ROOT)

from core import schema  # noqa: E402  (gym/core)

REPORT_KIND = "gymAudit"
SCHEMA_VERSION = "1.0"

#: 삼키면 안 되는 예외 — 사용자가 끊었는데 정합 0건이라고 쓰면 거짓말이다.
FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

#: 도구가 접는 위반 코드. 시험·문서가 같은 표를 본다.
ISSUE_CODES = (
    "missing-packs-root",
    "packs-not-dir",
    "unlistable-packs",
    "empty-packs-root",
    "missing-pack-json",
    "pack-json-parse",
    "pack-json-not-object",
    "pack-json-unreadable",
    "bad-schema",
    "missing-tasks-dir",
    "tasks-not-dir",
    "unlistable-tasks",
    "missing-reference-dir",
    "reference-not-dir",
    "unlistable-reference",
    "empty-pack",
    "task-parse",
    "task-not-object",
    "task-unreadable",
    "task-empty-id",
    "task-filename-id-mismatch",
    "task-id-duplicate-in-pack",
    "missing-reference",
    "reference-parse",
    "reference-not-object",
    "reference-unreadable",
    "reference-id-mismatch",
    "orphan-reference",
    "task-id-collision",
    "unexpected",
)

#: 코드 → 가족. 가족이 늘면 문서 표와 시험을 같이 고친다.
ISSUE_FAMILY = {
    "missing-packs-root": "root",
    "packs-not-dir": "root",
    "unlistable-packs": "root",
    "empty-packs-root": "root",
    "missing-pack-json": "manifest",
    "pack-json-parse": "manifest",
    "pack-json-not-object": "manifest",
    "pack-json-unreadable": "manifest",
    "bad-schema": "schema",
    "missing-tasks-dir": "layout",
    "tasks-not-dir": "layout",
    "unlistable-tasks": "layout",
    "missing-reference-dir": "layout",
    "reference-not-dir": "layout",
    "unlistable-reference": "layout",
    "empty-pack": "layout",
    "task-parse": "task",
    "task-not-object": "task",
    "task-unreadable": "task",
    "task-empty-id": "identity",
    "task-filename-id-mismatch": "identity",
    "task-id-duplicate-in-pack": "identity",
    "missing-reference": "pairing",
    "reference-parse": "pairing",
    "reference-not-object": "pairing",
    "reference-unreadable": "pairing",
    "reference-id-mismatch": "pairing",
    "orphan-reference": "pairing",
    "task-id-collision": "identity",
    "unexpected": "tool",
}

ISSUE_FAMILIES = ("root", "manifest", "schema", "layout", "task", "pairing", "identity", "tool")

#: 사람 메시지 기본값. 자리표시가 있으면 format_issue_message 가 채운다.
ISSUE_TEXT = {
    "missing-packs-root": "packs 루트가 없다 — 정합 0건으로 위장하지 않는다",
    "packs-not-dir": "packs 가 디렉터리가 아니다",
    "unlistable-packs": "packs 디렉터리를 읽을 수 없다",
    "empty-packs-root": "packs 아래에 pack 폴더가 없다",
    "missing-pack-json": "pack.json 이 없다",
    "pack-json-parse": "pack.json 파싱 실패",
    "pack-json-not-object": "pack.json 이 객체가 아니다",
    "pack-json-unreadable": "pack.json 을 읽을 수 없다",
    "bad-schema": "스키마 위반",
    "missing-tasks-dir": "tasks 디렉터리가 없다",
    "tasks-not-dir": "tasks 가 디렉터리가 아니다",
    "unlistable-tasks": "tasks 디렉터리를 읽을 수 없다",
    "missing-reference-dir": "reference 디렉터리가 없다",
    "reference-not-dir": "reference 가 디렉터리가 아니다",
    "unlistable-reference": "reference 디렉터리를 읽을 수 없다",
    "empty-pack": "과제가 없다 — 빈 pack 은 해결 가능성을 선언할 수 없다",
    "task-parse": "과제 JSON 파싱 실패",
    "task-not-object": "과제 JSON 이 객체가 아니다",
    "task-unreadable": "과제 파일을 읽을 수 없다",
    "task-empty-id": "과제 id 가 비었다",
    "task-filename-id-mismatch": "과제 id 가 파일 이름과 다르다",
    "task-id-duplicate-in-pack": "pack 안 과제 ID 가 여러 파일에 있다",
    "missing-reference": "짝 기준풀이가 없다 — 해결 가능성 미선언",
    "reference-parse": "기준풀이 JSON 파싱 실패",
    "reference-not-object": "기준풀이 JSON 이 객체가 아니다",
    "reference-unreadable": "기준풀이 파일을 읽을 수 없다",
    "reference-id-mismatch": "기준풀이 id 가 과제 id 와 다르다",
    "orphan-reference": "고아 기준풀이 — 짝 과제가 없다",
    "task-id-collision": "과제 ID 가 여러 pack 에 있다",
    "unexpected": "카탈로그 밖 실패",
}

#: 종료 코드. 도구 자리 실패는 정합 위반(1)과 구별한다.
EXIT_OK = 0
EXIT_VIOLATION = 1
EXIT_TOOL = 2

REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "packCount",
    "taskCount",
    "referenceCount",
    "packs",
    "okPacks",
    "emptyPacks",
    "taskIdCollisions",
    "issueCount",
    "issues",
    "issueCountsByCode",
    "issueCountsByFamily",
    "toolErrors",
    "missingPacksRoot",
    "toolFailed",
    "exit",
)

ISSUE_RECORD_KEYS = ("code", "pack", "path", "message", "family")

ERROR_HEAD_LIMIT = 160

CATCHABLE_EXCEPTIONS = (
    FileNotFoundError,
    NotADirectoryError,
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

JSON_SUFFIX = ".json"


def is_fatal_exception(exc) -> bool:
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def truncate_head(text, limit=ERROR_HEAD_LIMIT) -> str:
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


def exception_kind(exc, context="json") -> str:
    """예외를 위반/도구 코드로 접는다. 순수.

    context:
      - packs-root: packs 디렉터리 자체
      - listdir: 디렉터리 나열
      - json / pack-json / task / reference: JSON 읽기
      - schema: schema.validate_* 예외
      - audit: 그 외
    """
    if exc is None:
        return "unexpected"
    if isinstance(exc, json.JSONDecodeError):
        if context == "pack-json":
            return "pack-json-parse"
        if context == "task":
            return "task-parse"
        if context == "reference":
            return "reference-parse"
        return "pack-json-parse" if context == "json" else "unexpected"
    if isinstance(exc, UnicodeError):
        if context == "pack-json":
            return "pack-json-unreadable"
        if context == "task":
            return "task-unreadable"
        if context == "reference":
            return "reference-unreadable"
        return "pack-json-unreadable"
    if isinstance(exc, PermissionError):
        if context == "packs-root":
            return "unlistable-packs"
        if context == "listdir-tasks":
            return "unlistable-tasks"
        if context == "listdir-reference":
            return "unlistable-reference"
        if context == "pack-json":
            return "pack-json-unreadable"
        if context == "task":
            return "task-unreadable"
        if context == "reference":
            return "reference-unreadable"
        return "unlistable-packs"
    if isinstance(exc, FileNotFoundError):
        if context == "packs-root":
            return "missing-packs-root"
        if context == "pack-json":
            return "missing-pack-json"
        if context == "listdir-tasks":
            return "missing-tasks-dir"
        if context == "listdir-reference":
            return "missing-reference-dir"
        return "missing-packs-root"
    if isinstance(exc, NotADirectoryError):
        if context == "packs-root":
            return "packs-not-dir"
        if context == "listdir-tasks":
            return "tasks-not-dir"
        if context == "listdir-reference":
            return "reference-not-dir"
        return "packs-not-dir"
    if isinstance(exc, IsADirectoryError):
        if context == "pack-json":
            return "pack-json-unreadable"
        if context == "task":
            return "task-unreadable"
        if context == "reference":
            return "reference-unreadable"
        return "unexpected"
    if isinstance(exc, (TypeError, AttributeError, KeyError, IndexError)):
        if context == "schema":
            return "bad-schema"
        if context == "pack-json":
            return "pack-json-not-object"
        if context == "task":
            return "task-not-object"
        if context == "reference":
            return "reference-not-object"
        return "unexpected"
    if isinstance(exc, ValueError):
        if context == "pack-json":
            return "pack-json-parse"
        if context == "task":
            return "task-parse"
        if context == "reference":
            return "reference-parse"
        return "unexpected"
    if isinstance(exc, OSError):
        if context == "packs-root":
            return "unlistable-packs"
        if context == "listdir-tasks":
            return "unlistable-tasks"
        if context == "listdir-reference":
            return "unlistable-reference"
        if context == "pack-json":
            return "pack-json-unreadable"
        if context == "task":
            return "task-unreadable"
        if context == "reference":
            return "reference-unreadable"
        return "unexpected"
    if context == "schema":
        return "bad-schema"
    return "unexpected"


def exception_record(exc, context="json", path="") -> dict:
    """예외를 오류 한 줄로 접는다. 여기서 예외를 다시 올리지 않는다."""
    return {
        "context": context or "",
        "kind": exception_kind(exc, context=context),
        "error": type(exc).__name__ if exc is not None else "NoneType",
        "head": truncate_head(str(exc) if exc is not None else ""),
        "path": path or "",
    }


def issue_family(code: str) -> str:
    """위반 코드의 가족. 모르는 코드는 tool."""
    if not code:
        return "tool"
    return ISSUE_FAMILY.get(str(code), "tool")


def issue_codes() -> tuple:
    """카탈로그 코드 튜플. 시험이 이 순서를 고정한다."""
    return ISSUE_CODES


def catalog_ids() -> tuple:
    """issue_codes 별칭 — 다른 도구의 catalog_ids 와 같은 이름."""
    return ISSUE_CODES


def is_known_code(code: str) -> bool:
    return code in ISSUE_CODES


def is_blocking_code(code: str) -> bool:
    """지금 카탈로그의 모든 코드는 차단이다. 경고 등급을 숨기지 않는다."""
    return is_known_code(code) or code == "unexpected"


def format_issue_message(code: str, **kwargs) -> str:
    """카탈로그 기본 문구 + 선택 자리. 순수."""
    base = ISSUE_TEXT.get(code, ISSUE_TEXT["unexpected"])
    if not kwargs:
        return base
    extra = kwargs.get("detail")
    if extra:
        return f"{base}: {extra}"
    return base


def make_issue(code: str, pack: str = "", path: str = "", message: str = "", **extra) -> dict:
    """구조화 위반 한 줄. 모르는 코드는 unexpected 로 접는다."""
    raw = code if is_known_code(code) else "unexpected"
    rec = {
        "code": raw,
        "pack": pack or "",
        "path": path or "",
        "message": message or format_issue_message(raw),
        "family": issue_family(raw),
    }
    for key, value in extra.items():
        if key in rec:
            continue
        rec[key] = value
    return rec


def posix_rel(*parts) -> str:
    """보고용 상대 경로. 백슬래시를 섞지 않는다."""
    cleaned = [str(p).replace("\\", "/").strip("/") for p in parts if p]
    return "/".join(cleaned)


def is_json_name(name) -> bool:
    """과제·기준풀이 파일로 치는 이름인가. 소문자 .json 만 (원 계약)."""
    if not isinstance(name, str) or not name:
        return False
    if name in (".", ".."):
        return False
    return name.endswith(JSON_SUFFIX) and not name.startswith(".")


def stem_of(name: str) -> str:
    """`X01.json` → `X01`. 접미가 없으면 이름 그대로."""
    if not isinstance(name, str):
        return ""
    if name.endswith(JSON_SUFFIX):
        return name[: -len(JSON_SUFFIX)]
    return name


def json_filename(task_id: str) -> str:
    """id → 짝 파일 이름. 빈 id 는 빈 문자열."""
    if not task_id or not isinstance(task_id, str):
        return ""
    return f"{task_id}{JSON_SUFFIX}"


def as_text(value) -> str:
    if value is None:
        return ""
    if isinstance(value, str):
        return value
    try:
        return str(value)
    except FATAL_EXCEPTIONS:
        raise
    except Exception:
        return ""


def is_nonempty_str(value) -> bool:
    return isinstance(value, str) and bool(value)


def distinct_preserve(items):
    """중복을 제거하되 처음 본 순서를 지킨다."""
    seen = []
    for item in items or []:
        if item not in seen:
            seen.append(item)
    return seen


def list_dir_safe(path, context="listdir"):
    """디렉터리 나열. 실패해도 예외를 올리지 않는다.

    반환: (names: list[str], error: dict|None)
    """
    if not path:
        return [], exception_record(FileNotFoundError("empty-path"), context=context, path=path or "")
    try:
        return sorted(os.listdir(path)), None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return [], exception_record(exc, context=context, path=path)
    except Exception as exc:
        return [], exception_record(exc, context=context, path=path)


def path_kind(path: str) -> str:
    """missing / file / dir / other. 예외는 missing 으로 접지 않고 other."""
    if not path:
        return "missing"
    try:
        if os.path.isdir(path):
            return "dir"
        if os.path.isfile(path):
            return "file"
        if os.path.lexists(path):
            return "other"
        return "missing"
    except FATAL_EXCEPTIONS:
        raise
    except Exception:
        return "other"


def load_json_safe(path, context="json"):
    """JSON 객체 읽기. 예외를 올리지 않는다.

    반환: (obj, error). obj 는 파싱 성공 시의 값(객체 아니어도 그대로).
    """
    if not path:
        return None, exception_record(FileNotFoundError("empty-path"), context=context, path="")
    try:
        with open(path, encoding="utf-8") as fh:
            return json.load(fh), None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return None, exception_record(exc, context=context, path=path)
    except Exception as exc:
        return None, exception_record(exc, context=context, path=path)


def load_object(path, context="json"):
    """JSON 객체를 요구한다. 리스트/숫자는 not-object.

    반환: (obj: dict|None, error: dict|None)
    """
    obj, err = load_json_safe(path, context=context)
    if err is not None:
        return None, err
    if not isinstance(obj, dict):
        kind = {
            "pack-json": "pack-json-not-object",
            "task": "task-not-object",
            "reference": "reference-not-object",
        }.get(context, "unexpected")
        return None, {
            "context": context,
            "kind": kind,
            "error": type(obj).__name__ if obj is not None else "NoneType",
            "head": truncate_head(f"JSON 이 객체가 아니다: {type(obj).__name__}"),
            "path": path or "",
        }
    return obj, None


def _legacy_load(path):
    """원 `_load` — 시험·외부 호출 호환. 예외를 그대로 올린다."""
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def _load(path):
    return _legacy_load(path)


def json_names_from(names):
    """이름 목록에서 .json 만, 정렬된 튜플."""
    if not names:
        return []
    return sorted(n for n in names if is_json_name(n))


def pair_names(task_names, ref_names):
    """파일 이름 짝짓기. 순수.

    반환: {paired, missing_refs, orphans}
    """
    tasks = set(task_names or [])
    refs = set(ref_names or [])
    return {
        "paired": sorted(tasks & refs),
        "missing_refs": sorted(tasks - refs),
        "orphans": sorted(refs - tasks),
    }


def detect_in_pack_duplicates(id_to_files: dict) -> dict:
    """같은 pack 에서 한 id 가 여러 파일에 있으면 그 맵만 남긴다. 순수."""
    out = {}
    for tid, files in (id_to_files or {}).items():
        uniq = distinct_preserve(files)
        if tid and len(uniq) > 1:
            out[tid] = uniq
    return out


def detect_global_collisions(task_id_owners: dict) -> dict:
    """서로 다른 pack 이 같은 id 를 쓰면 충돌. 같은 pack 의 이중 등록은 제외.

    순수. 값 순서는 처음 본 pack 순.
    """
    collisions = {}
    for tid, owners in (task_id_owners or {}).items():
        distinct = distinct_preserve(owners)
        if tid and len(distinct) > 1:
            collisions[tid] = distinct
    return collisions


def classify_schema_message(message: str) -> str:
    """schema.validate_* 가 남긴 한글 문장을 세부 태그로 본다. 코드는 항상 bad-schema."""
    text = as_text(message)
    if "kind" in text and "아니다" in text:
        return "kind"
    if "schemaVersion" in text:
        return "schemaVersion"
    if "폴더 이름" in text or "pack id" in text:
        return "pack-id"
    if "title" in text and "비었" in text:
        return "title"
    if "axis" in text and "비었" in text:
        return "axis"
    if "requires.commands" in text:
        return "requires"
    if "runner." in text:
        return "runner"
    if "필수 키 없음" in text:
        return "task-required"
    if "tier" in text:
        return "tier"
    if "checks 가 비었" in text:
        return "checks-empty"
    if "미등록 연산자" in text:
        return "unknown-op"
    if "전역 훑기" in text:
        return "global-scan"
    if "cmd 가 필요" in text:
        return "missing-cmd"
    if "cmd 가 있다" in text:
        return "unexpected-cmd"
    return "other"


def run_validate_pack(manifest, pack_dir) -> list:
    """schema.validate_pack 을 감싼다. 예외는 메시지로 접는다."""
    messages: list[str] = []
    try:
        schema.validate_pack(manifest, pack_dir, messages)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        messages.append(f"schema.validate_pack 예외: {exc}")
    except Exception as exc:
        messages.append(f"schema.validate_pack 예외: {exc}")
    return messages


def run_validate_task(task, manifest) -> list:
    """schema.validate_task. known_commands=None — 명령 존재는 러너 몫."""
    messages: list[str] = []
    try:
        schema.validate_task(task, manifest, None, messages)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        messages.append(f"schema.validate_task 예외: {exc}")
    except Exception as exc:
        messages.append(f"schema.validate_task 예외: {exc}")
    return messages


def pack_issue_line(code: str, **kwargs) -> str:
    """packs[].issues 에 넣는 한글 한 줄. 원 계약 문구를 우선한다."""
    name = kwargs.get("name", "")
    tid = kwargs.get("tid")
    rid = kwargs.get("rid")
    detail = kwargs.get("detail", "")
    files = kwargs.get("files") or []
    owners = kwargs.get("owners") or []
    if code == "missing-pack-json":
        return "pack.json 이 없다"
    if code == "pack-json-parse":
        return f"pack.json 파싱 실패: {detail}" if detail else "pack.json 파싱 실패"
    if code == "pack-json-not-object":
        return "pack.json 이 객체가 아니다"
    if code == "pack-json-unreadable":
        return f"pack.json 을 읽을 수 없다: {detail}" if detail else "pack.json 을 읽을 수 없다"
    if code == "bad-schema":
        return detail or "스키마 위반"
    if code == "empty-pack":
        return "과제가 없다 — 빈 pack 은 해결 가능성을 선언할 수 없다"
    if code == "unlistable-tasks":
        return f"tasks 디렉터리를 읽을 수 없다: {detail}" if detail else "tasks 디렉터리를 읽을 수 없다"
    if code == "unlistable-reference":
        return f"reference 디렉터리를 읽을 수 없다: {detail}" if detail else "reference 디렉터리를 읽을 수 없다"
    if code == "tasks-not-dir":
        return "tasks 가 디렉터리가 아니다"
    if code == "reference-not-dir":
        return "reference 가 디렉터리가 아니다"
    if code == "task-parse":
        return f"tasks/{name} 파싱 실패: {detail}" if detail else f"tasks/{name} 파싱 실패"
    if code == "task-not-object":
        return f"tasks/{name} 이 객체가 아니다"
    if code == "task-unreadable":
        return f"tasks/{name} 을 읽을 수 없다: {detail}" if detail else f"tasks/{name} 을 읽을 수 없다"
    if code == "task-empty-id":
        return f"과제 {name} 의 id 가 비었다"
    if code == "task-filename-id-mismatch":
        return f"과제 {name} 의 id({tid}) 가 파일 이름과 다르다"
    if code == "task-id-duplicate-in-pack":
        joined = ", ".join(files) if files else name
        return f"pack 안 과제 ID '{tid}' 가 여러 파일에 있다: {joined}"
    if code == "missing-reference":
        return f"과제 {name} 에 짝 기준풀이(reference/{name})가 없다 — 해결 가능성 미선언"
    if code == "reference-parse":
        return f"reference/{name} 파싱 실패: {detail}" if detail else f"reference/{name} 파싱 실패"
    if code == "reference-not-object":
        return f"reference/{name} 이 객체가 아니다"
    if code == "reference-unreadable":
        return f"reference/{name} 을 읽을 수 없다: {detail}" if detail else f"reference/{name} 을 읽을 수 없다"
    if code == "reference-id-mismatch":
        return f"reference/{name} 의 id({rid}) 가 과제 id({tid}) 와 다르다"
    if code == "orphan-reference":
        return f"고아 기준풀이 reference/{name} — 짝 과제(tasks/{name})가 없다"
    if code == "task-id-collision":
        return f"과제 ID '{tid}' 충돌: {', '.join(owners)}"
    if code == "missing-tasks-dir":
        return "tasks 디렉터리가 없다"
    if code == "missing-reference-dir":
        return "reference 디렉터리가 없다"
    if detail:
        return f"{format_issue_message(code)}: {detail}"
    return format_issue_message(code)


def append_issue(pack_issues, structured, code, pack, path="", **kwargs):
    """한글 줄 + 구조화 줄을 동시에 남긴다."""
    line = pack_issue_line(code, **kwargs)
    pack_issues.append(line)
    rec = make_issue(code, pack=pack, path=path, message=line)
    tag = kwargs.get("schema_tag")
    if tag:
        rec["schemaTag"] = tag
    tid = kwargs.get("tid")
    if tid:
        rec["taskId"] = tid
    structured.append(rec)
    return rec


def empty_report() -> dict:
    """빈 정합 봉투. 키가 빠지면 시험이 막는다."""
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": True,
        "packCount": 0,
        "taskCount": 0,
        "referenceCount": 0,
        "packs": [],
        "okPacks": [],
        "emptyPacks": [],
        "taskIdCollisions": {},
        "issueCount": 0,
        "issues": [],
        "issueCountsByCode": {},
        "issueCountsByFamily": {},
        "toolErrors": [],
        "missingPacksRoot": False,
        "toolFailed": False,
        "exit": EXIT_OK,
    }


def count_by(items, key):
    """구조화 이슈에서 key 별 건수. 순수."""
    counts = {}
    for item in items or []:
        if not isinstance(item, dict):
            continue
        value = item.get(key) or "unexpected"
        counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def resolve_exit(report: dict) -> int:
    """봉투 → 종료 코드. 도구 실패가 위반보다 앞선다. 순수."""
    if not isinstance(report, dict):
        return EXIT_TOOL
    if report.get("toolFailed") or report.get("missingPacksRoot"):
        return EXIT_TOOL
    if report.get("ok"):
        return EXIT_OK
    return EXIT_VIOLATION


def validate_report(report) -> list:
    """봉투 계약. 빈 리스트면 통과. 순수 — 예외를 올리지 않는다."""
    problems = []
    if not isinstance(report, dict):
        return ["report 가 객체가 아니다"]
    for key in REPORT_KEYS:
        if key not in report:
            problems.append(f"키 없음: {key}")
    if report.get("kind") != REPORT_KIND:
        problems.append(f"kind 가 {REPORT_KIND} 가 아니다")
    if report.get("schemaVersion") != SCHEMA_VERSION:
        problems.append(f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다")
    if not isinstance(report.get("ok"), bool):
        problems.append("ok 가 bool 이 아니다")
    if not isinstance(report.get("packCount"), int):
        problems.append("packCount 가 int 가 아니다")
    if not isinstance(report.get("packs"), list):
        problems.append("packs 가 list 가 아니다")
    if not isinstance(report.get("taskIdCollisions"), dict):
        problems.append("taskIdCollisions 가 dict 가 아니다")
    if not isinstance(report.get("issues"), list):
        problems.append("issues 가 list 가 아니다")
    if not isinstance(report.get("toolErrors"), list):
        problems.append("toolErrors 가 list 가 아니다")
    issue_count = report.get("issueCount")
    if isinstance(issue_count, int) and isinstance(report.get("issues"), list):
        # issueCount 는 원 계약대로 packs[].issues + 전역 충돌 수.
        # 구조화 issues 길이와 같을 수도, 전역 충돌이 packs 밖에만 있을 수도 있다.
        if issue_count < 0:
            problems.append("issueCount 가 음수다")
    expected_ok = (report.get("issueCount") == 0) and (not report.get("toolFailed"))
    if isinstance(report.get("ok"), bool) and isinstance(report.get("issueCount"), int):
        if report["ok"] != expected_ok:
            problems.append("ok 가 issueCount/toolFailed 와 어긋난다")
    expected_exit = resolve_exit(report)
    if report.get("exit") not in (None, expected_exit) and report.get("exit") != expected_exit:
        problems.append("exit 가 ok/toolFailed 와 어긋난다")
    return problems


def format_human_report(report: dict) -> str:
    """사람용 요약. JSON 이 아닌 stdout 경로."""
    if not isinstance(report, dict):
        return "gym 정합 감사: 보고가 손상됐다\n"
    if report.get("toolFailed") or report.get("missingPacksRoot"):
        heads = []
        for err in report.get("toolErrors") or []:
            if isinstance(err, dict):
                heads.append(err.get("head") or err.get("kind") or "도구 실패")
        if not heads:
            for issue in report.get("issues") or []:
                if isinstance(issue, dict):
                    heads.append(issue.get("message") or issue.get("code") or "")
        detail = heads[0] if heads else "도구 실패"
        return f"gym 정합 감사: 도구 실패 — {detail}\n"
    if report.get("ok"):
        return f"gym 정합 감사: {report.get('packCount', 0)} pack 전부 통과 — 위반 0\n"
    lines = [f"gym 정합 감사: 위반 {report.get('issueCount', 0)}건"]
    for pack in report.get("packs") or []:
        pid = pack.get("id") if isinstance(pack, dict) else "?"
        for issue in (pack.get("issues") if isinstance(pack, dict) else []) or []:
            lines.append(f"  [{pid}] {issue}")
    collisions = report.get("taskIdCollisions") or {}
    if isinstance(collisions, dict):
        for tid in sorted(collisions):
            owners = collisions[tid]
            if isinstance(owners, (list, tuple)):
                joined = ", ".join(as_text(o) for o in owners)
            else:
                joined = as_text(owners)
            lines.append(f"  [전역] 과제 ID '{tid}' 충돌: {joined}")
    return "\n".join(lines) + "\n"


def format_json_report(report: dict) -> str:
    """UTF-8 · BOM 없음 · ensure_ascii=False. 끝에 개행."""
    return json.dumps(report, ensure_ascii=False, indent=2) + "\n"


def parse_args(argv=None):
    """CLI. `--json` 만. 새 플래그를 열지 않는다."""
    ap = argparse.ArgumentParser(description="gym 전 pack 정합 감사 (해결가능·고유·정합)")
    ap.add_argument("--json", action="store_true")
    return ap.parse_args(argv)


def _note_tool_error(tool_errors, structured, code, path="", detail=""):
    rec = exception_record(None, context="audit", path=path)
    rec["kind"] = code
    rec["head"] = detail or format_issue_message(code)
    rec["error"] = "AuditToolError"
    tool_errors.append(rec)
    structured.append(make_issue(code, pack="", path=path, message=rec["head"]))


def audit_one_pack(pack_id: str, pack_dir: str, task_id_owners: dict) -> dict:
    """pack 하나. 예외를 올리지 않는다(치명 제외).

    반환: {id, issues, structured, taskCount, referenceCount, empty}
    """
    issues: list[str] = []
    structured: list[dict] = []
    task_count = 0
    ref_count = 0
    id_to_files: dict[str, list[str]] = {}

    manifest_path = os.path.join(pack_dir, "pack.json")
    kind = path_kind(manifest_path)
    if kind == "missing":
        append_issue(issues, structured, "missing-pack-json", pack_id, path=posix_rel(pack_id, "pack.json"))
        return {
            "id": pack_id,
            "issues": issues,
            "structured": structured,
            "taskCount": 0,
            "referenceCount": 0,
            "empty": False,
        }
    if kind != "file":
        append_issue(
            issues, structured, "pack-json-unreadable", pack_id,
            path=posix_rel(pack_id, "pack.json"),
            detail=f"종류={kind}",
        )
        return {
            "id": pack_id,
            "issues": issues,
            "structured": structured,
            "taskCount": 0,
            "referenceCount": 0,
            "empty": False,
        }

    manifest, err = load_object(manifest_path, context="pack-json")
    if err is not None:
        code = err.get("kind") if is_known_code(err.get("kind")) else "pack-json-parse"
        append_issue(
            issues, structured, code, pack_id,
            path=posix_rel(pack_id, "pack.json"),
            detail=err.get("head") or err.get("error") or "",
        )
        return {
            "id": pack_id,
            "issues": issues,
            "structured": structured,
            "taskCount": 0,
            "referenceCount": 0,
            "empty": False,
        }

    for msg in run_validate_pack(manifest, pack_dir):
        append_issue(
            issues, structured, "bad-schema", pack_id,
            path=posix_rel(pack_id, "pack.json"),
            detail=msg,
            schema_tag=classify_schema_message(msg),
        )

    tasks_dir = os.path.join(pack_dir, "tasks")
    ref_dir = os.path.join(pack_dir, "reference")
    task_files: list[str] = []
    ref_files: list[str] = []

    tasks_kind = path_kind(tasks_dir)
    tasks_listed = False
    if tasks_kind == "dir":
        names, lerr = list_dir_safe(tasks_dir, context="listdir-tasks")
        if lerr is not None:
            append_issue(
                issues, structured, "unlistable-tasks", pack_id,
                path=posix_rel(pack_id, "tasks"),
                detail=lerr.get("head") or lerr.get("error") or "",
            )
        else:
            task_files = json_names_from(names)
            tasks_listed = True
    elif tasks_kind == "missing":
        # 원 계약: 없는 tasks 는 빈 목록. 과제가 없으면 아래에서 empty-pack.
        task_files = []
        tasks_listed = True
    else:
        append_issue(
            issues, structured, "tasks-not-dir", pack_id,
            path=posix_rel(pack_id, "tasks"),
        )

    refs_kind = path_kind(ref_dir)
    if refs_kind == "dir":
        names, lerr = list_dir_safe(ref_dir, context="listdir-reference")
        if lerr is not None:
            append_issue(
                issues, structured, "unlistable-reference", pack_id,
                path=posix_rel(pack_id, "reference"),
                detail=lerr.get("head") or lerr.get("error") or "",
            )
        else:
            ref_files = json_names_from(names)
            ref_count = len(ref_files)
    elif refs_kind == "missing":
        ref_files = []
    else:
        append_issue(
            issues, structured, "reference-not-dir", pack_id,
            path=posix_rel(pack_id, "reference"),
        )

    ref_set = set(ref_files)
    task_count = len(task_files)

    for name in task_files:
        task_path = os.path.join(tasks_dir, name)
        task, terr = load_object(task_path, context="task")
        if terr is not None:
            code = terr.get("kind") if is_known_code(terr.get("kind")) else "task-parse"
            append_issue(
                issues, structured, code, pack_id,
                path=posix_rel(pack_id, "tasks", name),
                name=name,
                detail=terr.get("head") or terr.get("error") or "",
            )
            if name not in ref_set:
                append_issue(
                    issues, structured, "missing-reference", pack_id,
                    path=posix_rel(pack_id, "reference", name),
                    name=name,
                )
            continue

        for msg in run_validate_task(task, manifest):
            append_issue(
                issues, structured, "bad-schema", pack_id,
                path=posix_rel(pack_id, "tasks", name),
                detail=msg,
                schema_tag=classify_schema_message(msg),
            )

        tid = task.get("id")
        expected = stem_of(name)
        if not is_nonempty_str(tid):
            append_issue(
                issues, structured, "task-empty-id", pack_id,
                path=posix_rel(pack_id, "tasks", name),
                name=name,
            )
            tid = None
        else:
            id_to_files.setdefault(tid, []).append(name)
            task_id_owners.setdefault(tid, []).append(pack_id)
            if tid != expected:
                append_issue(
                    issues, structured, "task-filename-id-mismatch", pack_id,
                    path=posix_rel(pack_id, "tasks", name),
                    name=name,
                    tid=tid,
                )

        if name not in ref_set:
            append_issue(
                issues, structured, "missing-reference", pack_id,
                path=posix_rel(pack_id, "reference", name),
                name=name,
            )
            continue

        ref, rerr = load_object(os.path.join(ref_dir, name), context="reference")
        if rerr is not None:
            code = rerr.get("kind") if is_known_code(rerr.get("kind")) else "reference-parse"
            append_issue(
                issues, structured, code, pack_id,
                path=posix_rel(pack_id, "reference", name),
                name=name,
                detail=rerr.get("head") or rerr.get("error") or "",
            )
            continue
        rid = ref.get("id")
        if rid != tid:
            append_issue(
                issues, structured, "reference-id-mismatch", pack_id,
                path=posix_rel(pack_id, "reference", name),
                name=name,
                tid=tid,
                rid=rid,
            )

    pairing = pair_names(task_files, ref_files)
    for name in pairing["orphans"]:
        append_issue(
            issues, structured, "orphan-reference", pack_id,
            path=posix_rel(pack_id, "reference", name),
            name=name,
        )

    duplicates = detect_in_pack_duplicates(id_to_files)
    for tid, files in sorted(duplicates.items()):
        append_issue(
            issues, structured, "task-id-duplicate-in-pack", pack_id,
            path=posix_rel(pack_id, "tasks"),
            tid=tid,
            files=files,
        )

    empty = bool(tasks_listed and task_count == 0)
    if empty:
        append_issue(
            issues, structured, "empty-pack", pack_id,
            path=posix_rel(pack_id),
        )

    return {
        "id": pack_id,
        "issues": issues,
        "structured": structured,
        "taskCount": task_count,
        "referenceCount": ref_count,
        "empty": empty,
    }


def _coerce_root(packs_root) -> str:
    if packs_root is None:
        return ""
    try:
        return os.fspath(packs_root)
    except FATAL_EXCEPTIONS:
        raise
    except Exception:
        return str(packs_root)


def audit(packs_root: str) -> dict:
    """전 pack 정합 감사 — 순수 파일 검사(바이너리·네트워크 없음).

    packs_root: `gym/packs` 를 담은 디렉토리(보통 gym/).
    반환: 원 계약 키(ok, packs, taskIdCollisions, issueCount, packCount) +
    구조화 이슈·도구 실패 자리.
    """
    report = empty_report()
    root = _coerce_root(packs_root)
    packs_dir = os.path.join(root, "packs") if root else "packs"
    task_id_owners: dict[str, list[str]] = {}
    pack_reports = []
    structured: list[dict] = []
    tool_errors: list[dict] = []
    task_total = 0
    ref_total = 0
    empty_packs: list[str] = []

    root_kind = path_kind(packs_dir)
    if root_kind == "missing":
        report["missingPacksRoot"] = True
        report["toolFailed"] = True
        _note_tool_error(tool_errors, structured, "missing-packs-root", path=packs_dir)
        report["issues"] = structured
        report["toolErrors"] = tool_errors
        report["issueCount"] = len(structured)
        report["ok"] = False
        report["issueCountsByCode"] = count_by(structured, "code")
        report["issueCountsByFamily"] = count_by(structured, "family")
        report["exit"] = resolve_exit(report)
        return report
    if root_kind != "dir":
        report["toolFailed"] = True
        code = "packs-not-dir"
        _note_tool_error(tool_errors, structured, code, path=packs_dir, detail=f"종류={root_kind}")
        report["issues"] = structured
        report["toolErrors"] = tool_errors
        report["issueCount"] = len(structured)
        report["ok"] = False
        report["issueCountsByCode"] = count_by(structured, "code")
        report["issueCountsByFamily"] = count_by(structured, "family")
        report["exit"] = resolve_exit(report)
        return report

    names, lerr = list_dir_safe(packs_dir, context="packs-root")
    if lerr is not None:
        report["toolFailed"] = True
        code = lerr.get("kind") if is_known_code(lerr.get("kind")) else "unlistable-packs"
        _note_tool_error(
            tool_errors, structured, code, path=packs_dir,
            detail=lerr.get("head") or lerr.get("error") or "",
        )
        report["issues"] = structured
        report["toolErrors"] = tool_errors
        report["issueCount"] = len(structured)
        report["ok"] = False
        report["issueCountsByCode"] = count_by(structured, "code")
        report["issueCountsByFamily"] = count_by(structured, "family")
        report["exit"] = resolve_exit(report)
        return report

    pack_ids = [name for name in names if path_kind(os.path.join(packs_dir, name)) == "dir"]
    if not pack_ids:
        # 원 계약: 빈 packs 는 위반 0 · packCount 0. 다만 사실을 기록한다.
        report["issues"] = structured
        report["toolErrors"] = tool_errors
        report["ok"] = True
        report["exit"] = EXIT_OK
        return report

    for pack_id in pack_ids:
        pack_dir = os.path.join(packs_dir, pack_id)
        try:
            one = audit_one_pack(pack_id, pack_dir, task_id_owners)
        except FATAL_EXCEPTIONS:
            raise
        except CATCHABLE_EXCEPTIONS as exc:
            rec = exception_record(exc, context="audit", path=pack_dir)
            tool_errors.append(rec)
            one = {
                "id": pack_id,
                "issues": [f"pack 감사 예외: {rec['head']}"],
                "structured": [make_issue("unexpected", pack=pack_id, path=posix_rel(pack_id), message=rec["head"])],
                "taskCount": 0,
                "referenceCount": 0,
                "empty": False,
            }
        pack_reports.append({"id": one["id"], "issues": list(one["issues"])})
        structured.extend(one["structured"])
        task_total += int(one.get("taskCount") or 0)
        ref_total += int(one.get("referenceCount") or 0)
        if one.get("empty"):
            empty_packs.append(pack_id)

    collisions = detect_global_collisions(task_id_owners)
    for tid, owners in sorted(collisions.items()):
        line = pack_issue_line("task-id-collision", tid=tid, owners=owners)
        structured.append(make_issue(
            "task-id-collision",
            pack="",
            path="",
            message=line,
            taskId=tid,
            owners=list(owners),
        ))

    issue_count = sum(len(p["issues"]) for p in pack_reports) + len(collisions)
    ok_packs = [p["id"] for p in pack_reports if not p["issues"]]
    dirty = [p for p in pack_reports if p["issues"]]
    tool_failed = bool(tool_errors)
    report.update({
        "ok": issue_count == 0 and not tool_failed,
        "packCount": len(pack_reports),
        "taskCount": task_total,
        "referenceCount": ref_total,
        "packs": dirty,
        "okPacks": ok_packs,
        "emptyPacks": empty_packs,
        "taskIdCollisions": collisions,
        "issueCount": issue_count,
        "issues": structured,
        "issueCountsByCode": count_by(structured, "code"),
        "issueCountsByFamily": count_by(structured, "family"),
        "toolErrors": tool_errors,
        "missingPacksRoot": False,
        "toolFailed": tool_failed,
    })
    report["exit"] = resolve_exit(report)
    return report


def main(argv=None) -> int:
    a = parse_args(argv)
    try:
        report = audit(GYM_ROOT)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        report = empty_report()
        rec = exception_record(exc, context="audit", path=GYM_ROOT)
        report["toolFailed"] = True
        report["ok"] = False
        report["toolErrors"] = [rec]
        report["issues"] = [make_issue("unexpected", message=rec["head"])]
        report["issueCount"] = 1
        report["issueCountsByCode"] = {"unexpected": 1}
        report["issueCountsByFamily"] = {"tool": 1}
        report["exit"] = EXIT_TOOL
    if a.json:
        sys.stdout.write(format_json_report(report))
    else:
        sys.stdout.write(format_human_report(report))
    return int(report.get("exit", resolve_exit(report)))


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
