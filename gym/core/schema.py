"""[#4653][#5279] pack·task·profile 스키마와 재현성 선언.

## pack manifest (`packs/<id>/pack.json`)

```json
{
  "schemaVersion": "1.0",
  "kind": "gymPack",
  "id": "table-editing",
  "title": "표 편집",
  "axis": "편집 (좌표 지정)",
  "requires": { "commands": ["export-tables", "edit", "table-to-csv"] },
  "runner": { "rhwpVersion": "0.8.3", "rhwpCommit": "…", "capabilitiesSha256": "…" }
}
```

`requires.commands` 는 이 pack 을 채점하려면 바이너리에 있어야 하는 명령이다.
없으면 **0점이 아니라 `unavailable`** 로 보고한다 — 부재를 실패로 위장하지
않는 것이 이 저장소의 결이고, pack 이 늘어날수록 이 구분이 중요해진다
(오래된 바이너리로 신규 pack 을 돌린 사람에게 "너는 0점"이라고 말하면 거짓말이다).

`runner` 는 **기준 실행의 신원**이다. 점수는 바이너리마다 달라질 수 있으므로
"이 점수가 어느 바이너리에서 났는가"를 pack 과 스코어카드 양쪽에 남긴다.

## 검증 API (예전 계약)

    validate_pack(manifest, pack_dir, errors)
    validate_task(task, pack, known_commands, errors)
    validate_profile(profile, pack_ids, errors)

`errors` 는 문자열 목록이다. `audit.py` 와 `test_gym_packs` 가 그 줄을 소비한다.
한 줄의 모양은 예전과 같다: `"<where>: <message>"`.

#5279 가 더한 것:

1. **객체가 아니면 죽지 않는다.** manifest·과제·프로파일·checks 항목·requires·
   runner·submit 이 목록이나 문자열이면 AttributeError 대신 칸을 남긴다.
2. **필수 키·공란·타입을 가른다.** `필수 키 없음` 은 키가 없을 때,
   `가 비었다` 는 키가 있는데 값이 공란일 때, `객체가 아니다` 는 타입이
   틀릴 때다. 예전에는 이 셋이 한 줄로 뭉개지거나 예외로 죽었다.
3. **tier 는 진짜 정수다.** `bool` 은 `int` 의 하위형이라 `True` 가 1 로
   통과했다. 이제 거부한다. 0·6·`"1"`·`1.0`·`None` 도 예전 메시지 그대로
   거부한다.
4. **미등록 연산자는 계속 거절한다.** `REGISTRY` 는 `checks.py` 가 소유한다.
   이 모듈은 읽기만 한다. 키를 더하거나 빼지 않는다.
5. **프로파일이 없는 pack 을 가리키면 `없는 pack 참조`.** 예전 메시지
   그대로다. packs 가 문자열이거나 중복이거나 경로이면 추가로 칸을 남긴다.
6. **구조화 칸(`IssueList`)은 선택이다.** 평범한 `list` 를 넘기면 예전처럼
   문자열만 쌓인다. `IssueList` 를 넘기면 `kind` 도 남는다.

새 CLI 는 없다. 새 연산자도 없다. pack JSON 도 바꾸지 않는다.
정본 규약은 `gym/docs/schema.md`, 작업 기록은 `mydocs/working/gym_schema.md`.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess

PACK_KIND = "gymPack"
PROFILE_KIND = "gymProfile"
SCHEMA_VERSION = "1.0"

#: 과제 파일의 필수 키.
TASK_REQUIRED = ("id", "tier", "title", "input", "instructions", "submit", "checks")

#: pack manifest 의 필수 키. 값이 공란인지는 키마다 따로 본다.
PACK_REQUIRED = ("schemaVersion", "kind", "id", "title", "axis", "requires", "runner")

#: 프로파일의 필수 키.
PROFILE_REQUIRED = ("schemaVersion", "kind", "id", "title", "packs")

#: 기준 실행 신원 키. 길이와 hex 형태는 키마다 다르다.
RUNNER_KEYS = ("rhwpVersion", "rhwpCommit", "capabilitiesSha256")

#: 제출 종류. README 의 세 칸과 같다.
SUBMIT_KINDS = ("answer", "artifact", "pair")

#: 편집 과제 — 전역 훑기 연산자를 금지한다(#4600 재발 방지).
EDITING_AXES = ("편집", "보안")

#: 난도 티어 1~5. 놀이공원 은유에서 어트랙션의 키 제한과 같다 —
#: 1=입문(누구나·부모님도), 2=초급, 3=중급, 4=고급, 5=보스(사다리 완주 급).
#: 상한을 3→5 로 넓힌 이유: 한쪽에는 비전문가도 성공하는 유아용 놀이기구를,
#: 다른 쪽에는 한 단계만 틀려도 최종 판정이 막히는 보스 어트랙션을 둔다(#4664).
TIER_MIN, TIER_MAX = 1, 5
TIER_NAMES = {1: "입문", 2: "초급", 3: "중급", 4: "고급", 5: "보스"}

#: pack·task·profile id 가 경로가 되지 않게. `table-editing`, `TB01`, `T10` 허용.
SAFE_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
COMMIT_RE = re.compile(r"^[0-9a-fA-F]{40}$")
SHA256_RE = re.compile(r"^[0-9a-fA-F]{64}$")

#: 예전 메시지 조각. 시험이 이 문자열을 고정한다 — 바꾸면 audit 소비자가 깨진다.
MSG_PACK_KIND = f"kind 가 {PACK_KIND} 가 아니다"
MSG_PACK_SCHEMA = f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다"
MSG_REQUIRES_EMPTY = "requires.commands 가 비었다 — 요구 capability 선언은 필수"
MSG_TIER = f"tier 는 {TIER_MIN}~{TIER_MAX} 정수 (1=입문 … 5=보스)"
MSG_CHECKS_EMPTY = "checks 가 비었다"
MSG_PROFILE_KIND = f"kind 가 {PROFILE_KIND} 가 아니다"
MSG_PACKS_EMPTY = "packs 가 비었다"
MSG_MISSING_KEY_PREFIX = "필수 키 없음: "
MSG_UNKNOWN_OP_PREFIX = "미등록 연산자: "
MSG_MISSING_PACK_PREFIX = "없는 pack 참조: "
MSG_GLOBAL_SCAN_NEEDLE = "전역 훑기 연산자"
MSG_CMD_NEEDED_SUFFIX = " 는 cmd 가 필요하다"
MSG_CMD_UNEXPECTED_NEEDLE = "는 CLI 를 부르지 않는데 cmd 가 있다"
MSG_UNKNOWN_COMMAND_PREFIX = "CLI 에 없는 명령: "
MSG_RUNNER_EMPTY_NEEDLE = "가 비었다 — 기준 실행 신원 선언은 필수"
MSG_PACK_ID_MISMATCH_NEEDLE = "가 폴더 이름과 다르다"

#: 스키마 칸 kind 카탈로그. 문서·시험이 같은 표를 본다.
SCHEMA_ISSUE_KINDS = (
    "missing-key",
    "empty-field",
    "bad-type",
    "bad-kind",
    "bad-schema-version",
    "bad-tier",
    "bad-id",
    "pack-id-mismatch",
    "unknown-op",
    "unknown-submit-kind",
    "empty-checks",
    "malformed-check",
    "malformed-cmd",
    "malformed-submit",
    "malformed-requires",
    "malformed-runner",
    "malformed-object",
    "missing-cmd",
    "unexpected-cmd",
    "unknown-command",
    "global-scan-forbidden",
    "profile-missing-pack",
    "empty-packs",
    "duplicate-pack",
    "unsafe-id",
    "bad-runner-identity",
    "not-a-mapping",
    "not-a-list",
    "duplicate-check-name",
    "empty-commands",
    "unexpected",
)

SCHEMA_ISSUE_HELP = {
    "missing-key": "필수 키가 객체에 없다",
    "empty-field": "키는 있는데 값이 공란이거나 falsy 다",
    "bad-type": "값의 파이썬 타입이 계약과 다르다",
    "bad-kind": "kind 가 gymPack / gymProfile 이 아니다",
    "bad-schema-version": "schemaVersion 이 1.0 이 아니다",
    "bad-tier": "tier 가 1~5 정수가 아니다 (bool 포함)",
    "bad-id": "id 가 비었거나 허용 문자가 아니다",
    "pack-id-mismatch": "pack.id 가 폴더 이름과 다르다",
    "unknown-op": "checks[].op 가 REGISTRY 에 없다",
    "unknown-submit-kind": "submit.kind 가 answer/artifact/pair 가 아니다",
    "empty-checks": "checks 가 비어 통과할 칸이 없다",
    "malformed-check": "checks 항목이 객체가 아니거나 이름/op 가 없다",
    "malformed-cmd": "cmd 가 비지 않은 문자열 목록이 아니다",
    "malformed-submit": "submit 이 객체가 아니거나 files 가 깨졌다",
    "malformed-requires": "requires 가 객체가 아니거나 commands 가 목록이 아니다",
    "malformed-runner": "runner 가 객체가 아니다",
    "malformed-object": "루트 값이 객체가 아니다",
    "missing-cmd": "needs_cli 연산자인데 cmd 가 없다",
    "unexpected-cmd": "파일 연산자인데 cmd 가 있다",
    "unknown-command": "cmd[0] 이 알려진 CLI 명령이 아니다",
    "global-scan-forbidden": "편집·보안 축에서 전역 훑기 연산자를 썼다",
    "profile-missing-pack": "프로파일이 없는 pack id 를 가리킨다",
    "empty-packs": "프로파일 packs 가 비었다",
    "duplicate-pack": "프로파일 packs 에 같은 id 가 두 번 있다",
    "unsafe-id": "id 에 경로 구분자나 .. 가 있다",
    "bad-runner-identity": "runner 신원의 길이·hex 가 틀렸다",
    "not-a-mapping": "객체여야 할 자리가 객체가 아니다",
    "not-a-list": "목록이어야 할 자리가 목록이 아니다",
    "duplicate-check-name": "한 과제 안에서 check.name 이 겹친다",
    "empty-commands": "requires.commands 항목이 공란이다",
    "unexpected": "분류되지 않은 스키마 위반",
}

#: 연산자별 권장 필드. 스키마 기본 검증은 강제하지 않는다 — lint_check_fields 용.
#: REGISTRY 키만 나열하고, 새 키를 여기서 만들어 등록하지 않는다.
CHECK_FIELD_HINTS = {
    "same_hash": ("files",),
    "differs_from_input": ("file",),
    "file_exists": ("file",),
    "files_differ": ("files",),
    "xml_root_eq": ("file", "value"),
    "json_value_eq": ("file", "value"),
    "csv_cell_eq": ("file", "row", "col", "value"),
    "text_file_envelope_eq": ("file",),
    "utf8_bom": ("file",),
    "json_len_eq": ("file", "value"),
    "csv_row_count_eq": ("file", "value"),
    "ndjson_count_eq": ("file", "value"),
    "ndjson_field_eq": ("file", "row", "value"),
    "json_keys_contain": ("file", "keys"),
    "text_line_eq": ("file", "line", "value"),
    "json_type_eq": ("file", "value"),
    "json_len_ge": ("file", "value"),
    "json_array_item_eq": ("file", "index", "value"),
    "csv_col_count_eq": ("file", "row", "value"),
    "csv_header_eq": ("file", "values"),
    "csv_row_eq": ("file", "row", "values"),
    "ndjson_keys_contain": ("file", "row", "keys"),
    "ndjson_len_eq": ("file", "row", "value"),
    "text_line_count_eq": ("file", "value"),
    "text_line_contains": ("file", "line", "value"),
    "answer_eq": ("answer",),
    "len_answer_eq": ("answer",),
    "len_ge": ("value",),
    "value_eq": ("value",),
    "value_ge": ("value",),
    "value_in": ("values",),
    "deep_contains": ("value",),
    "not_contains": ("value",),
    "cell_text_eq": ("table", "row", "col", "value"),
}

#: 문서·시험에서 재사용하는 최소 유효 뼈대. 저장소 pack 을 바꾸지 않는다.
MINIMAL_RUNNER = {
    "rhwpVersion": "0.0.0",
    "rhwpCommit": "c" * 40,
    "capabilitiesSha256": "a" * 64,
}

MINIMAL_PACK = {
    "schemaVersion": SCHEMA_VERSION,
    "kind": PACK_KIND,
    "id": "demo-pack",
    "title": "데모",
    "axis": "시험",
    "requires": {"commands": ["info"]},
    "runner": dict(MINIMAL_RUNNER),
}

MINIMAL_TASK = {
    "id": "D01",
    "tier": 1,
    "title": "데모 과제",
    "input": "samples/x.hwp",
    "instructions": "제출하라",
    "submit": {"kind": "answer"},
    "checks": [{"name": "존재", "op": "file_exists", "file": "answer.json"}],
}

MINIMAL_PROFILE = {
    "schemaVersion": SCHEMA_VERSION,
    "kind": PROFILE_KIND,
    "id": "demo",
    "title": "데모 코스",
    "packs": ["demo-pack"],
}

MINIMAL_CHECK_FILE = {"name": "존재", "op": "file_exists", "file": "answer.json"}
MINIMAL_CHECK_CLI = {
    "name": "쪽수",
    "op": "answer_eq",
    "answer": "pages",
    "cmd": ["info", "{input}", "--json"],
    "path": "pageCount",
}


class SchemaIssue:
    """한 칸의 구조화 기록. 텍스트 줄은 as_text() 가 예전 계약을 지킨다."""

    __slots__ = ("kind", "where", "message", "field", "got")

    def __init__(self, kind, where, message, field=None, got=None):
        self.kind = kind if kind in SCHEMA_ISSUE_KINDS else "unexpected"
        self.where = where
        self.message = message
        self.field = field
        self.got = _preview(got)

    def as_text(self):
        return f"{self.where}: {self.message}"

    def as_dict(self):
        payload = {
            "kind": self.kind,
            "where": self.where,
            "message": self.message,
        }
        if self.field is not None:
            payload["field"] = self.field
        if self.got is not None:
            payload["got"] = self.got
        return payload


class IssueList(list):
    """문자열 목록 + 구조화 칸. 평범한 list 자리에 넣어도 append 계약이 산다."""

    def __init__(self, *args):
        super().__init__(*args)
        self.structured = []

    def append_issue(self, kind, where, message, field=None, got=None):
        issue = SchemaIssue(kind, where, message, field=field, got=got)
        self.structured.append(issue)
        self.append(issue.as_text())
        return issue

    def kinds(self):
        return [item.kind for item in self.structured]

    def has_kind(self, kind):
        return any(item.kind == kind for item in self.structured)

    def of_kind(self, kind):
        return [item for item in self.structured if item.kind == kind]

    def fields_of(self, kind):
        return [item.field for item in self.of_kind(kind)]

    def as_dicts(self):
        return [item.as_dict() for item in self.structured]


def _preview(value):
    if value is None:
        return None
    if isinstance(value, (str, int, float, bool)):
        text = str(value)
        return text if len(text) <= 80 else text[:77] + "..."
    if isinstance(value, type):
        return value.__name__
    return type(value).__name__


def is_known_issue_kind(kind):
    return kind in SCHEMA_ISSUE_KINDS


def describe_issue_kind(kind):
    return SCHEMA_ISSUE_HELP.get(kind, SCHEMA_ISSUE_HELP["unexpected"])


def is_valid_tier(value):
    """1~5 정수. bool 은 거부한다 — True 가 1 로 통과하면 입문 과제가 생긴다."""
    if isinstance(value, bool) or not isinstance(value, int):
        return False
    return TIER_MIN <= value <= TIER_MAX


def is_safe_id(value):
    if not isinstance(value, str) or not value:
        return False
    if value != value.strip():
        return False
    if value in (".", ".."):
        return False
    if any(sep in value for sep in ("/", "\\", ":")):
        return False
    return bool(SAFE_ID_RE.match(value))


def is_commit_hex(value):
    return isinstance(value, str) and bool(COMMIT_RE.match(value))


def is_sha256_hex(value):
    return isinstance(value, str) and bool(SHA256_RE.match(value))


def is_nonempty_str(value):
    return isinstance(value, str) and bool(value.strip())


def is_mapping(value):
    return isinstance(value, dict)


def is_str_list(value):
    return isinstance(value, list) and all(isinstance(item, str) for item in value)


def is_nonempty_str_list(value):
    return is_str_list(value) and bool(value) and all(item.strip() for item in value)


def is_known_submit_kind(value):
    return value in SUBMIT_KINDS


def is_editing_axis(axis):
    if not isinstance(axis, str):
        return False
    return any(axis.startswith(prefix) for prefix in EDITING_AXES)


def check_registry():
    """checks.REGISTRY 의 읽기 전용 손잡이. 이 모듈은 등록부를 고치지 않는다."""
    from . import checks as registry
    return registry


def registered_ops():
    return frozenset(check_registry().REGISTRY)


def is_registered_op(op):
    return op in check_registry().REGISTRY


def is_global_scan_op(op):
    return op in check_registry().GLOBAL_SCAN_OPS


def op_needs_cli(op):
    return check_registry().needs_cli(op)


def lint_check_fields(check):
    """연산자가 권장하는 필드가 빠졌는지. 기본 validate_task 는 이걸 강제하지 않는다."""
    if not is_mapping(check):
        return ("<not-a-mapping>",)
    op = check.get("op")
    hints = CHECK_FIELD_HINTS.get(op)
    if not hints:
        return ()
    return tuple(field for field in hints if field not in check)


def _fail(errors, where, message, kind="unexpected", field=None, got=None):
    """errors 가 IssueList 면 kind 를 남기고, 아니면 예전 문자열만 남긴다."""
    if errors is None:
        return
    append_issue = getattr(errors, "append_issue", None)
    if callable(append_issue):
        append_issue(kind, where, message, field=field, got=got)
        return
    errors.append(f"{where}: {message}")


def collect_pack(manifest, pack_dir):
    issues = IssueList()
    validate_pack(manifest, pack_dir, issues)
    return issues


def collect_task(task, pack, known_commands=None):
    issues = IssueList()
    validate_task(task, pack, known_commands, issues)
    return issues


def collect_profile(profile, pack_ids):
    issues = IssueList()
    validate_profile(profile, pack_ids, issues)
    return issues


def _axis_of(task, pack):
    if is_mapping(task) and is_nonempty_str(task.get("axis")):
        return task["axis"]
    if is_mapping(pack) and is_nonempty_str(pack.get("axis")):
        return pack["axis"]
    return ""


def _task_where(pack, task):
    pack_id = pack.get("id") if is_mapping(pack) else None
    task_id = task.get("id") if is_mapping(task) else None
    return f"{pack_id}/{task_id}"


def _profile_where(profile):
    ident = profile.get("id") if is_mapping(profile) else None
    return f"profiles/{ident}"


def validate_pack(manifest, pack_dir, errors):
    where = os.path.basename(pack_dir) if pack_dir else "<pack>"
    if not is_mapping(manifest):
        _fail(errors, where, "pack.json 이 객체가 아니다",
              kind="not-a-mapping", field="manifest", got=type(manifest))
        return

    if manifest.get("kind") != PACK_KIND:
        _fail(errors, where, MSG_PACK_KIND,
              kind="bad-kind", field="kind", got=manifest.get("kind"))
    if manifest.get("schemaVersion") != SCHEMA_VERSION:
        _fail(errors, where, MSG_PACK_SCHEMA,
              kind="bad-schema-version", field="schemaVersion",
              got=manifest.get("schemaVersion"))

    pack_id = manifest.get("id")
    if pack_id != where:
        _fail(errors, where, f"pack id({pack_id}) 가 폴더 이름과 다르다",
              kind="pack-id-mismatch", field="id", got=pack_id)
    if pack_id is not None and not is_safe_id(pack_id):
        _fail(errors, where, f"pack id({pack_id}) 가 안전하지 않다",
              kind="unsafe-id", field="id", got=pack_id)

    for key in ("title", "axis"):
        value = manifest.get(key)
        if not value:
            _fail(errors, where, f"{key} 가 비었다",
                  kind="empty-field", field=key, got=value)
        elif not isinstance(value, str):
            _fail(errors, where, f"{key} 가 문자열이 아니다",
                  kind="bad-type", field=key, got=type(value))

    _validate_requires(manifest.get("requires", {}), where, errors)
    _validate_runner(manifest.get("runner", {}), where, errors)


def _validate_requires(requires, where, errors):
    if not is_mapping(requires):
        _fail(errors, where, "requires 가 객체가 아니다",
              kind="malformed-requires", field="requires", got=type(requires))
        return
    commands = requires.get("commands")
    if not isinstance(commands, list) or not commands:
        _fail(errors, where, MSG_REQUIRES_EMPTY,
              kind="empty-commands", field="requires.commands", got=commands)
        return
    bad = [item for item in commands if not is_nonempty_str(item)]
    if bad:
        _fail(errors, where, "requires.commands 항목이 빈 문자열이다",
              kind="empty-commands", field="requires.commands", got=bad[0])


def _validate_runner(runner, where, errors):
    if not is_mapping(runner):
        _fail(errors, where, "runner 가 객체가 아니다",
              kind="malformed-runner", field="runner", got=type(runner))
        return
    for key in RUNNER_KEYS:
        value = runner.get(key)
        if not value:
            _fail(errors, where, f"runner.{key} 가 비었다 — 기준 실행 신원 선언은 필수",
                  kind="empty-field", field=f"runner.{key}", got=value)
            continue
        if not isinstance(value, str):
            _fail(errors, where, f"runner.{key} 가 문자열이 아니다",
                  kind="bad-type", field=f"runner.{key}", got=type(value))
            continue
        if key == "rhwpCommit" and not is_commit_hex(value):
            _fail(errors, where, "runner.rhwpCommit 이 40자리 hex 가 아니다",
                  kind="bad-runner-identity", field="runner.rhwpCommit", got=value)
        if key == "capabilitiesSha256" and not is_sha256_hex(value):
            _fail(errors, where, "runner.capabilitiesSha256 이 64자리 hex 가 아니다",
                  kind="bad-runner-identity", field="runner.capabilitiesSha256",
                  got=value)


def validate_task(task, pack, known_commands, errors):
    if not is_mapping(pack):
        pack = {}
    if not is_mapping(task):
        _fail(errors, _task_where(pack, {}), "과제가 객체가 아니다",
              kind="not-a-mapping", field="task", got=type(task))
        return

    where = _task_where(pack, task)
    for key in TASK_REQUIRED:
        if key not in task:
            _fail(errors, where, f"{MSG_MISSING_KEY_PREFIX}{key}",
                  kind="missing-key", field=key)

    task_id = task.get("id")
    if "id" in task:
        if not is_nonempty_str(task_id):
            _fail(errors, where, "id 가 비었다",
                  kind="empty-field", field="id", got=task_id)
        elif not is_safe_id(task_id):
            _fail(errors, where, f"id({task_id}) 가 안전하지 않다",
                  kind="unsafe-id", field="id", got=task_id)

    for key in ("title", "input", "instructions"):
        if key not in task:
            continue
        value = task.get(key)
        if not isinstance(value, str):
            _fail(errors, where, f"{key} 는 문자열이어야 한다",
                  kind="bad-type", field=key, got=type(value))
        elif not value.strip():
            _fail(errors, where, f"{key} 가 비었다",
                  kind="empty-field", field=key, got=value)

    if "tier" in task and not is_valid_tier(task.get("tier")):
        _fail(errors, where, MSG_TIER,
              kind="bad-tier", field="tier", got=task.get("tier"))
    elif "tier" not in task:
        # 필수 키 없음과 별도로, 예전 코드도 빠진 tier 를 범위 밖으로 한 번 더 남겼다.
        _fail(errors, where, MSG_TIER,
              kind="bad-tier", field="tier", got=None)

    if "submit" in task:
        _validate_submit(task.get("submit"), where, errors)

    checks = task.get("checks")
    if "checks" not in task:
        _fail(errors, where, MSG_CHECKS_EMPTY,
              kind="empty-checks", field="checks", got=None)
    elif not isinstance(checks, list):
        _fail(errors, where, "checks 가 목록이 아니다",
              kind="not-a-list", field="checks", got=type(checks))
    elif not checks:
        _fail(errors, where, MSG_CHECKS_EMPTY,
              kind="empty-checks", field="checks", got=checks)
    else:
        _validate_checks(checks, task, pack, known_commands, where, errors)


def _validate_submit(submit, where, errors):
    if not is_mapping(submit):
        _fail(errors, where, "submit 이 객체가 아니다",
              kind="malformed-submit", field="submit", got=type(submit))
        return
    kind = submit.get("kind")
    if not kind:
        _fail(errors, where, "submit.kind 가 비었다",
              kind="empty-field", field="submit.kind", got=kind)
    elif not is_known_submit_kind(kind):
        _fail(errors, where, f"submit.kind 가 알려진 값이 아니다: {kind}",
              kind="unknown-submit-kind", field="submit.kind", got=kind)
    files = submit.get("files")
    if files is None:
        return
    if not isinstance(files, list):
        _fail(errors, where, "submit.files 가 목록이 아니다",
              kind="malformed-submit", field="submit.files", got=type(files))
        return
    if not files:
        _fail(errors, where, "submit.files 가 비었다",
              kind="empty-field", field="submit.files", got=files)
        return
    if not is_nonempty_str_list(files):
        _fail(errors, where, "submit.files 항목이 빈 문자열이다",
              kind="malformed-submit", field="submit.files")


def _validate_checks(checks, task, pack, known_commands, where, errors):
    registry = check_registry()
    editing = is_editing_axis(_axis_of(task, pack))
    seen_names = []
    for index, check in enumerate(checks):
        _validate_one_check(
            check, index, editing, known_commands, registry, where, errors, seen_names)


def _validate_one_check(check, index, editing, known_commands, registry, where, errors,
                        seen_names):
    label = f"checks[{index}]"
    if not is_mapping(check):
        _fail(errors, where, f"{label} 가 객체가 아니다",
              kind="malformed-check", field=label, got=type(check))
        return

    name = check.get("name")
    if not is_nonempty_str(name):
        _fail(errors, where, f"{label} 이름 없음",
              kind="malformed-check", field=f"{label}.name", got=name)
    elif name in seen_names:
        _fail(errors, where, f"{label} 이름 중복: {name}",
              kind="duplicate-check-name", field=f"{label}.name", got=name)
    else:
        seen_names.append(name)

    op = check.get("op")
    if op not in registry.REGISTRY:
        _fail(errors, where, f"{MSG_UNKNOWN_OP_PREFIX}{op}",
              kind="unknown-op", field=f"{label}.op", got=op)
        return
    if editing and op in registry.GLOBAL_SCAN_OPS and not check.get("allowGlobalScan"):
        _fail(errors, where,
              f"편집 과제에 전역 훑기 연산자({op}) — 좌표를 지목하는 연산자를 쓰거나 "
              "allowGlobalScan 으로 사유를 명시하라(#4600)",
              kind="global-scan-forbidden", field=f"{label}.op", got=op)

    cmd = check.get("cmd")
    if registry.needs_cli(op):
        if not cmd:
            _fail(errors, where, f"{op}{MSG_CMD_NEEDED_SUFFIX}",
                  kind="missing-cmd", field=f"{label}.cmd", got=cmd)
            return
        if not is_nonempty_str_list(cmd):
            _fail(errors, where, f"{op} 의 cmd 가 문자열 목록이 아니다",
                  kind="malformed-cmd", field=f"{label}.cmd", got=type(cmd))
            return
        if known_commands is not None and cmd[0] not in known_commands:
            _fail(errors, where, f"{MSG_UNKNOWN_COMMAND_PREFIX}{cmd[0]}",
                  kind="unknown-command", field=f"{label}.cmd", got=cmd[0])
    elif cmd:
        _fail(errors, where, f"{op} 는 CLI 를 부르지 않는데 cmd 가 있다",
              kind="unexpected-cmd", field=f"{label}.cmd", got=cmd)


def validate_profile(profile, pack_ids, errors):
    if not is_mapping(profile):
        _fail(errors, "profiles/<unknown>", "프로파일이 객체가 아니다",
              kind="not-a-mapping", field="profile", got=type(profile))
        return

    where = _profile_where(profile)
    if profile.get("kind") != PROFILE_KIND:
        _fail(errors, where, MSG_PROFILE_KIND,
              kind="bad-kind", field="kind", got=profile.get("kind"))
    if "schemaVersion" in profile and profile.get("schemaVersion") != SCHEMA_VERSION:
        _fail(errors, where, f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다",
              kind="bad-schema-version", field="schemaVersion",
              got=profile.get("schemaVersion"))

    ident = profile.get("id")
    if ident is not None and not is_safe_id(ident):
        _fail(errors, where, f"profile id({ident}) 가 안전하지 않다",
              kind="unsafe-id", field="id", got=ident)
    if "title" in profile and not is_nonempty_str(profile.get("title")):
        _fail(errors, where, "title 가 비었다",
              kind="empty-field", field="title", got=profile.get("title"))

    packs = profile.get("packs")
    if not packs:
        _fail(errors, where, MSG_PACKS_EMPTY,
              kind="empty-packs", field="packs", got=packs)
        return
    if not isinstance(packs, list):
        _fail(errors, where, "packs 가 목록이 아니다",
              kind="not-a-list", field="packs", got=type(packs))
        return

    known = set(pack_ids) if pack_ids is not None else set()
    seen = set()
    for pid in packs:
        if not is_nonempty_str(pid):
            _fail(errors, where, "packs 항목이 빈 문자열이다",
                  kind="empty-field", field="packs", got=pid)
            continue
        if not is_safe_id(pid):
            _fail(errors, where, f"packs 항목({pid}) 이 안전하지 않다",
                  kind="unsafe-id", field="packs", got=pid)
        if pid in seen:
            _fail(errors, where, f"packs 항목 중복: {pid}",
                  kind="duplicate-pack", field="packs", got=pid)
        else:
            seen.add(pid)
        if pack_ids is not None and pid not in known:
            _fail(errors, where, f"{MSG_MISSING_PACK_PREFIX}{pid}",
                  kind="profile-missing-pack", field="packs", got=pid)


def parse_capabilities_payload(raw):
    """capabilities stdout 을 객체로. 깨지면 None. 예외를 밖으로 던지지 않는다."""
    if raw is None:
        return None
    if isinstance(raw, str):
        text = raw
    else:
        try:
            text = raw.decode("utf-8")
        except (UnicodeError, AttributeError, TypeError):
            return None
    try:
        payload = json.loads(text)
    except ValueError:
        return None
    return payload if is_mapping(payload) else None


def parse_command_names(raw):
    payload = parse_capabilities_payload(raw)
    if payload is None:
        return None
    commands = payload.get("commands")
    if not isinstance(commands, list):
        return None
    names = set()
    for item in commands:
        if is_mapping(item) and is_nonempty_str(item.get("name")):
            names.add(item["name"])
    return names


def parse_capabilities_version(raw):
    payload = parse_capabilities_payload(raw)
    if payload is None:
        return ""
    version = payload.get("version", "")
    return version if isinstance(version, str) else ""


def capabilities_digest(bin_path):
    """`rhwp capabilities` 원문의 sha256 — 명령 표면의 지문."""
    if not bin_path:
        raise ValueError("bin_path 가 비었다")
    proc = subprocess.run([bin_path, "capabilities"], capture_output=True)
    raw = proc.stdout or b""
    return hashlib.sha256(raw).hexdigest(), raw


def known_commands(bin_path):
    """capabilities 의 명령 이름 집합. 파싱 실패는 None — 명령 검사를 건너뛴다.

    TypeError·UnicodeError 도 같은 None 이다. 예전에는 ValueError·KeyError 만
    잡아서 목록이 깨지면 채점 전체가 죽었다. 빈 집합과 None 은 다르다: 빈
    집합은 '명령을 읽었고 하나도 없다', None 은 '읽지 못했다'.
    """
    _, raw = capabilities_digest(bin_path)
    try:
        return {c["name"] for c in json.loads(raw.decode("utf-8"))["commands"]}
    except (ValueError, KeyError, TypeError, UnicodeError, AttributeError):
        return None


def try_known_commands(bin_path):
    """known_commands 와 같되 바이너리 부재는 None. 채점기가 삼킬 자리를 가른다."""
    try:
        return known_commands(bin_path)
    except OSError:
        return None


def git_head(repo_root):
    try:
        proc = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repo_root,
            capture_output=True,
        )
    except OSError:
        return ""
    text = (proc.stdout or b"").decode("utf-8", errors="replace").strip()
    return text if is_commit_hex(text) or text else text


def runner_identity(bin_path, repo_root):
    """실행 시점 신원 — pack 의 `runner` 선언과 대조할 값."""
    digest, raw = capabilities_digest(bin_path)
    version = ""
    try:
        version = json.loads(raw.decode("utf-8")).get("version", "")
    except (ValueError, TypeError, UnicodeError, AttributeError):
        version = parse_capabilities_version(raw)
    if not isinstance(version, str):
        version = ""
    commit = git_head(repo_root)
    return {
        "rhwpVersion": version,
        "rhwpCommit": commit,
        "capabilitiesSha256": digest,
    }


def load_json_mapping(path, errors=None, where=None):
    """JSON 객체를 읽는다. 실패하면 (None, 이유) 이고 errors 에도 남긴다."""
    label = where or os.path.basename(path)
    try:
        with open(path, encoding="utf-8") as fh:
            payload = json.load(fh)
    except FileNotFoundError:
        _fail(errors, label, "파일이 없다", kind="missing-key", field="path", got=path)
        return None
    except (OSError, UnicodeError) as exc:
        _fail(errors, label, f"읽기 실패: {exc}",
              kind="unexpected", field="path", got=path)
        return None
    except ValueError as exc:
        _fail(errors, label, f"JSON 파싱 실패: {exc}",
              kind="malformed-object", field="path", got=path)
        return None
    if not is_mapping(payload):
        _fail(errors, label, "JSON 루트가 객체가 아니다",
              kind="not-a-mapping", field="root", got=type(payload))
        return None
    return payload


def discover_pack_ids(packs_dir):
    if not os.path.isdir(packs_dir):
        return []
    found = []
    for name in sorted(os.listdir(packs_dir)):
        if os.path.isfile(os.path.join(packs_dir, name, "pack.json")):
            found.append(name)
    return found


def iter_task_paths(pack_dir):
    tasks_dir = os.path.join(pack_dir, "tasks")
    if not os.path.isdir(tasks_dir):
        return
    for name in sorted(os.listdir(tasks_dir)):
        if name.endswith(".json"):
            yield os.path.join(tasks_dir, name)


def iter_profile_paths(profiles_dir):
    if not os.path.isdir(profiles_dir):
        return
    for name in sorted(os.listdir(profiles_dir)):
        if name.endswith(".json"):
            yield os.path.join(profiles_dir, name)


def validate_gym_tree(gym_root, errors=None, known_commands=None):
    """gym/ 아래 pack·과제·프로파일을 전수 검증. audit.py 를 바꾸지 않는 읽기 경로."""
    if errors is None:
        errors = IssueList()
    packs_dir = os.path.join(gym_root, "packs")
    profiles_dir = os.path.join(gym_root, "profiles")
    pack_ids = discover_pack_ids(packs_dir)
    for pid in pack_ids:
        pack_dir = os.path.join(packs_dir, pid)
        manifest = load_json_mapping(
            os.path.join(pack_dir, "pack.json"), errors, pid)
        if manifest is None:
            continue
        validate_pack(manifest, pack_dir, errors)
        for task_path in iter_task_paths(pack_dir):
            task = load_json_mapping(
                task_path, errors, f"{pid}/{os.path.basename(task_path)}")
            if task is None:
                continue
            validate_task(task, manifest, known_commands, errors)
    known = set(pack_ids)
    for profile_path in iter_profile_paths(profiles_dir):
        profile = load_json_mapping(
            profile_path, errors, f"profiles/{os.path.basename(profile_path)}")
        if profile is None:
            continue
        validate_profile(profile, known, errors)
    return errors


def clone_minimal_pack(**overrides):
    body = json.loads(json.dumps(MINIMAL_PACK))
    body.update(overrides)
    return body


def clone_minimal_task(**overrides):
    body = json.loads(json.dumps(MINIMAL_TASK))
    body.update(overrides)
    return body


def clone_minimal_profile(**overrides):
    body = json.loads(json.dumps(MINIMAL_PROFILE))
    body.update(overrides)
    return body


def format_issue_catalog():
    """문서·시험이 같은 표를 그리기 위한 (kind, help) 목록."""
    return [(kind, SCHEMA_ISSUE_HELP[kind]) for kind in SCHEMA_ISSUE_KINDS]


def issue_kind_count():
    return len(SCHEMA_ISSUE_KINDS)
