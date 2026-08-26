"""gym pack 건강 감사 — 지시문·검사 이름·힌트·제출 형식의 품질 게이트.

## 왜 이 도구인가 (audit 가 못 보는 층)

`audit.py` 는 스키마·기준풀이 짝·과제 ID 전역 고유만 본다. 지시문이 비거나 한
줄짜리이고, 같은 과제 안에서 `check.name` 이 비거나 겹치고, 힌트가 답을 그대로
적어 두고, `submit.kind` 가 모르는 값이어도 그 감사는 통과한다. 운동장 품질이
조용히 내려간다.

이 도구는 그 다음 층이다. 바이너리·네트워크 없이 pack JSON 만 읽어:

- 빈/짧은 `instructions` (기본 20글자 미만)
- 과제 안 `check.name` 누락·중복·앞뒤 공백
- 과제 `id`/`title` 의 앞뒤·내부 공백, 짧은 title
- 기준풀이 파일은 있는데 `steps` 가 비었다
- 모르는 `submit.kind`, 제출 파일 경로 위생
- 힌트가 답을 직접 노출하는지 (정답 JSON 덤프, `답은 N`, 검사 기대값 복붙)
- 힌트만 있고 본문이 비었거나, 마커만 있고 힌트가 빔, TODO 자리표
- pack.json 신원(kind·schema·id=폴더·title·axis·requires·runner)
- 과제 `tier`/`input` 범위·경로 위생
- 미지/`op` 누락, CLI 연산자 `cmd` 누락, 파일 연산자 `file`/`files` 누락
- 전역 훑기 연산자를 편집 축에서 사유 없이 씀
- 짝 reference 의 id 불일치, 빈 `run`/`answer`, 고아 reference, 빈 pack

을 pack 별로 보고한다. 기존 pack 을 고치지 않는다 — 도구만 추가한다.
실제 트리 과제를 실패로 뒤집지 않도록, 새 규칙은 픽스처로 고정하고
현재 gym/packs 가 이미 지키는 계약만 승격한다.

## 종료 코드 (CI 자기시험과 게이트를 가른다)

현재 트리가 깨끗하지 않을 수 있고, 도구 자체를 CI 에서 돌릴 때는 리포트만
필요할 수 있다. 그래서:

    python gym/tools/pack_health.py            # 리포트, 이슈가 있어도 0
    python gym/tools/pack_health.py --json     # 같은 리포트 JSON, 기본 0
    python gym/tools/pack_health.py --strict   # 이슈가 있으면 1
    python gym/tools/pack_health.py --pack id  # 한 pack 만
    python gym/tools/pack_health.py --root DIR # gym/ 루트 (packs/ 를 담은 곳)
    python gym/tools/pack_health.py --codes    # 이슈 코드 목록만
    python gym/tools/pack_health.py --exclude C  # 코드 C 를 집계에서 뺌

`--strict` 는 품질 관문이다. 기본은 관측이다.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)

KIND = "gymPackHealth"
SCHEMA_VERSION = "1.0"

# gym/README.md 「제출 형식」이 선언한 세 값. 새 kind 는 채점기가 모르면 제출이 증발한다.
KNOWN_SUBMIT_KINDS = frozenset({"answer", "artifact", "pair"})
MIN_INSTRUCTION_CHARS = 20
MIN_HINT_VALUE_CHARS = 4
MIN_TITLE_CHARS = 2
PACK_KIND = "gymPack"
PACK_SCHEMA_VERSION = "1.0"
RUNNER_KEYS = ("rhwpVersion", "rhwpCommit", "capabilitiesSha256")
EDITING_AXES = ("편집", "보안")

# core.checks 를 못 읽으면 쓰는 내장 목록. 현재 트리 REGISTRY 와 같다.
FALLBACK_FILE_OPS = frozenset(
    {
        "same_hash",
        "differs_from_input",
        "file_exists",
        "files_differ",
        "xml_root_eq",
        "json_value_eq",
        "csv_cell_eq",
        "utf8_bom",
    }
)
FALLBACK_CLI_OPS = frozenset(
    {
        "answer_eq",
        "len_answer_eq",
        "len_ge",
        "value_eq",
        "value_ge",
        "value_in",
        "deep_contains",
        "not_contains",
        "cell_text_eq",
    }
)
FALLBACK_GLOBAL_OPS = frozenset({"deep_contains", "not_contains"})
OPS_NEED_ANSWER = frozenset({"answer_eq", "len_answer_eq"})
OPS_NEED_VALUE = frozenset(
    {
        "value_eq",
        "value_ge",
        "value_in",
        "cell_text_eq",
        "xml_root_eq",
        "json_value_eq",
        "csv_cell_eq",
        "len_ge",
    }
)
OPS_NEED_FILE = frozenset(
    {
        "differs_from_input",
        "file_exists",
        "xml_root_eq",
        "json_value_eq",
        "csv_cell_eq",
        "utf8_bom",
    }
)
OPS_NEED_FILES = frozenset({"same_hash", "files_differ"})
OPS_NEED_CELL_COORD = frozenset({"cell_text_eq"})
OPS_NEED_CSV_COORD = frozenset({"csv_cell_eq"})

CODE_EMPTY_INSTRUCTIONS = "empty_instructions"
CODE_SHORT_INSTRUCTIONS = "short_instructions"
CODE_INSTRUCTIONS_TYPE = "instructions_type"
CODE_MISSING_CHECK_NAME = "missing_check_name"
CODE_EMPTY_CHECK_NAME = "empty_check_name"
CODE_DUPLICATE_CHECK_NAME = "duplicate_check_name"
CODE_TASK_ID_WHITESPACE = "task_id_whitespace"
CODE_TASK_TITLE_WHITESPACE = "task_title_whitespace"
CODE_EMPTY_TASK_ID = "empty_task_id"
CODE_EMPTY_TITLE = "empty_title"
CODE_ID_FILENAME_MISMATCH = "id_filename_mismatch"
CODE_DUPLICATE_TASK_ID = "duplicate_task_id"
CODE_EMPTY_REFERENCE_STEPS = "empty_reference_steps"
CODE_REFERENCE_STEPS_TYPE = "reference_steps_type"
CODE_UNKNOWN_SUBMIT_KIND = "unknown_submit_kind"
CODE_MISSING_SUBMIT = "missing_submit"
CODE_MISSING_SUBMIT_KIND = "missing_submit_kind"
CODE_SUBMIT_TYPE = "submit_type"
CODE_ARTIFACT_WITHOUT_FILES = "artifact_without_files"
CODE_PAIR_WITHOUT_FILES = "pair_without_files"
CODE_HINT_ANSWER_DUMP = "hint_answer_dump"
CODE_HINT_SPOILER = "hint_spoiler"
CODE_HINT_EMBEDS_VALUE = "hint_embeds_check_value"
CODE_CHECKS_TYPE = "checks_type"
CODE_EMPTY_CHECKS = "empty_checks"
CODE_CHECK_TYPE = "check_type"
CODE_PARSE_ERROR = "parse_error"
CODE_MISSING_PACK = "missing_pack_json"
CODE_MISSING_TASKS = "missing_tasks_dir"
CODE_PACK_TYPE = "pack_type"
CODE_PACK_KIND = "pack_kind"
CODE_PACK_SCHEMA_VERSION = "pack_schema_version"
CODE_PACK_ID_MISMATCH = "pack_id_mismatch"
CODE_PACK_EMPTY_TITLE = "pack_empty_title"
CODE_PACK_EMPTY_AXIS = "pack_empty_axis"
CODE_PACK_TITLE_WHITESPACE = "pack_title_whitespace"
CODE_PACK_AXIS_WHITESPACE = "pack_axis_whitespace"
CODE_PACK_MISSING_REQUIRES = "pack_missing_requires"
CODE_PACK_EMPTY_COMMANDS = "pack_empty_commands"
CODE_PACK_COMMAND_TYPE = "pack_command_type"
CODE_PACK_MISSING_RUNNER = "pack_missing_runner"
CODE_PACK_MISSING_RUNNER_FIELD = "pack_missing_runner_field"
CODE_EMPTY_PACK = "empty_pack"
CODE_ORPHAN_REFERENCE = "orphan_reference"
CODE_MISSING_TIER = "missing_tier"
CODE_TIER_TYPE = "tier_type"
CODE_TIER_RANGE = "tier_range"
CODE_MISSING_INPUT = "missing_input"
CODE_EMPTY_INPUT = "empty_input"
CODE_INPUT_TYPE = "input_type"
CODE_INPUT_WHITESPACE = "input_whitespace"
CODE_INPUT_ABSOLUTE = "input_absolute"
CODE_INPUT_BACKSLASH = "input_backslash"
CODE_INPUT_PARENT = "input_parent_traversal"
CODE_TITLE_TOO_SHORT = "title_too_short"
CODE_CHECK_NAME_WHITESPACE = "check_name_whitespace"
CODE_MISSING_CHECK_OP = "missing_check_op"
CODE_EMPTY_CHECK_OP = "empty_check_op"
CODE_UNKNOWN_CHECK_OP = "unknown_check_op"
CODE_CHECK_MISSING_ANSWER = "check_missing_answer"
CODE_CHECK_MISSING_VALUE = "check_missing_value"
CODE_CHECK_MISSING_FILE = "check_missing_file"
CODE_CHECK_MISSING_FILES = "check_missing_files"
CODE_CHECK_FILES_SHORT = "check_files_short"
CODE_CHECK_FILE_EMPTY = "check_file_empty"
CODE_CHECK_FILE_ABSOLUTE = "check_file_absolute"
CODE_CHECK_FILE_BACKSLASH = "check_file_backslash"
CODE_CHECK_MISSING_CMD = "check_missing_cmd"
CODE_CHECK_CMD_TYPE = "check_cmd_type"
CODE_CHECK_CMD_EMPTY = "check_cmd_empty"
CODE_CHECK_CMD_ITEM_TYPE = "check_cmd_item_type"
CODE_CHECK_UNEXPECTED_CMD = "check_unexpected_cmd"
CODE_CELL_MISSING_COORD = "cell_missing_coord"
CODE_CSV_MISSING_COORD = "csv_missing_coord"
CODE_GLOBAL_SCAN_UNDECLARED = "global_scan_undeclared"
CODE_SUBMIT_FILES_TYPE = "submit_files_type"
CODE_SUBMIT_FILE_EMPTY = "submit_file_empty"
CODE_SUBMIT_FILE_TYPE = "submit_file_type"
CODE_SUBMIT_FILE_WHITESPACE = "submit_file_whitespace"
CODE_SUBMIT_FILE_ABSOLUTE = "submit_file_absolute"
CODE_SUBMIT_FILE_BACKSLASH = "submit_file_backslash"
CODE_SUBMIT_FILE_DUPLICATE = "submit_file_duplicate"
CODE_INSTRUCTIONS_HINT_ONLY = "instructions_hint_only"
CODE_EMPTY_HINT = "empty_hint"
CODE_DUPLICATE_HINT_MARKER = "duplicate_hint_marker"
CODE_INSTRUCTIONS_TODO = "instructions_todo"
CODE_INSTRUCTIONS_CONTROL_CHAR = "instructions_control_char"
CODE_REFERENCE_ID_MISMATCH = "reference_id_mismatch"
CODE_REFERENCE_STEP_TYPE = "reference_step_type"
CODE_REFERENCE_RUN_EMPTY = "reference_run_empty"
CODE_REFERENCE_ANSWER_EMPTY = "reference_answer_empty"
CODE_REFERENCE_CMD_EMPTY = "reference_cmd_empty"

SEVERITY_ERROR = "error"
SEVERITY_WARNING = "warning"

HINT_MARKERS = ("힌트:", "힌트 :", "Hint:", "hint:")
SPOILER_RE = re.compile(
    r"(?:정답(?:은|이)?|답은|answer\s*(?:is|:))\s+.+\S",
    re.IGNORECASE,
)
BARE_ANSWER_RE = re.compile(r"^\s*(?:정답|답)?\s*[=:]?\s*-?\d+(?:\.\d+)?\s*$")
PLACEHOLDER_RE = re.compile(r"[<{\[][^>}\]]+[>}\]]")

# 너무 흔한 짧은 토큰은 힌트 복붙으로 치지 않는다(명령 이름·불린·한 글자).
COMMON_HINT_TOKENS = frozenset(
    {
        "ok",
        "true",
        "false",
        "allow",
        "deny",
        "info",
        "json",
        "hwp",
        "hwpx",
        "hwp3",
        "hml",
        "pdf",
        "html",
        "out",
        "ws",
        "in",
        "to",
        "on",
        "id",
    }
)

TODO_RE = re.compile(
    r"\b(?:TODO|FIXME|XXX|TBD)\b|여기를 채우|지시문을 작성|lorem ipsum",
    re.IGNORECASE,
)
CONTROL_CHAR_RE = re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f]")

# (code, default_severity, layer, 한 줄 설명) — --codes 와 문서가 같은 표를 본다.
ISSUE_CATALOG: tuple[tuple[str, str, str, str], ...] = (
    (CODE_EMPTY_INSTRUCTIONS, SEVERITY_ERROR, "지시", "instructions 키가 없거나 비었다"),
    (CODE_SHORT_INSTRUCTIONS, SEVERITY_ERROR, "지시", "instructions 가 최소 글자 미만이다"),
    (CODE_INSTRUCTIONS_TYPE, SEVERITY_ERROR, "지시", "instructions 가 문자열이 아니다"),
    (CODE_INSTRUCTIONS_HINT_ONLY, SEVERITY_ERROR, "지시", "힌트 마커 앞에 본문이 없다"),
    (CODE_EMPTY_HINT, SEVERITY_WARNING, "지시", "힌트 마커는 있는데 내용이 비었다"),
    (CODE_DUPLICATE_HINT_MARKER, SEVERITY_WARNING, "지시", "힌트 마커가 두 번 이상이다"),
    (CODE_INSTRUCTIONS_TODO, SEVERITY_ERROR, "지시", "TODO/FIXME 자리표가 남아 있다"),
    (CODE_INSTRUCTIONS_CONTROL_CHAR, SEVERITY_ERROR, "지시", "지시문에 제어 문자가 있다"),
    (CODE_MISSING_CHECK_NAME, SEVERITY_ERROR, "검사", "check.name 이 없다"),
    (CODE_EMPTY_CHECK_NAME, SEVERITY_ERROR, "검사", "check.name 이 비었다"),
    (CODE_DUPLICATE_CHECK_NAME, SEVERITY_ERROR, "검사", "같은 과제에서 check.name 이 겹친다"),
    (CODE_CHECK_NAME_WHITESPACE, SEVERITY_ERROR, "검사", "check.name 앞뒤에 공백이 있다"),
    (CODE_MISSING_CHECK_OP, SEVERITY_ERROR, "검사", "check.op 가 없다"),
    (CODE_EMPTY_CHECK_OP, SEVERITY_ERROR, "검사", "check.op 가 비었다"),
    (CODE_UNKNOWN_CHECK_OP, SEVERITY_ERROR, "검사", "등록되지 않은 연산자다"),
    (CODE_CHECK_MISSING_ANSWER, SEVERITY_ERROR, "검사", "answer_eq 계열에 answer 가 없다"),
    (CODE_CHECK_MISSING_VALUE, SEVERITY_ERROR, "검사", "값 비교 연산자에 value 가 없다"),
    (CODE_CHECK_MISSING_FILE, SEVERITY_ERROR, "검사", "파일 연산자에 file 이 없다"),
    (CODE_CHECK_MISSING_FILES, SEVERITY_ERROR, "검사", "해시 비교 연산자에 files 가 없다"),
    (CODE_CHECK_FILES_SHORT, SEVERITY_ERROR, "검사", "files 가 2개 미만이다"),
    (CODE_CHECK_FILE_EMPTY, SEVERITY_ERROR, "검사", "file/files 항목이 비었다"),
    (CODE_CHECK_FILE_ABSOLUTE, SEVERITY_ERROR, "검사", "file 경로가 절대 경로다"),
    (CODE_CHECK_FILE_BACKSLASH, SEVERITY_ERROR, "검사", "file 경로에 백슬래시가 있다"),
    (CODE_CHECK_MISSING_CMD, SEVERITY_ERROR, "검사", "CLI 연산자에 cmd 가 없다"),
    (CODE_CHECK_CMD_TYPE, SEVERITY_ERROR, "검사", "cmd 가 배열이 아니다"),
    (CODE_CHECK_CMD_EMPTY, SEVERITY_ERROR, "검사", "cmd 가 비었다"),
    (CODE_CHECK_CMD_ITEM_TYPE, SEVERITY_ERROR, "검사", "cmd 항목이 문자열이 아니다"),
    (CODE_CHECK_UNEXPECTED_CMD, SEVERITY_ERROR, "검사", "파일 연산자에 cmd 가 있다"),
    (CODE_CELL_MISSING_COORD, SEVERITY_ERROR, "검사", "cell_text_eq 에 table/row/col 이 없다"),
    (CODE_CSV_MISSING_COORD, SEVERITY_ERROR, "검사", "csv_cell_eq 에 row/col 이 없다"),
    (CODE_GLOBAL_SCAN_UNDECLARED, SEVERITY_ERROR, "검사", "편집 축에서 전역 훑기를 사유 없이 썼다"),
    (CODE_CHECKS_TYPE, SEVERITY_ERROR, "검사", "checks 가 배열이 아니다"),
    (CODE_EMPTY_CHECKS, SEVERITY_ERROR, "검사", "checks 가 없거나 비었다"),
    (CODE_CHECK_TYPE, SEVERITY_ERROR, "검사", "checks 항목이 객체가 아니다"),
    (CODE_TASK_ID_WHITESPACE, SEVERITY_ERROR, "신원", "과제 id 에 공백이 있다"),
    (CODE_TASK_TITLE_WHITESPACE, SEVERITY_ERROR, "신원", "과제 title 앞뒤에 공백이 있다"),
    (CODE_EMPTY_TASK_ID, SEVERITY_ERROR, "신원", "과제 id 가 비었다"),
    (CODE_EMPTY_TITLE, SEVERITY_ERROR, "신원", "과제 title 이 비었다"),
    (CODE_TITLE_TOO_SHORT, SEVERITY_ERROR, "신원", "과제 title 이 최소 글자 미만이다"),
    (CODE_ID_FILENAME_MISMATCH, SEVERITY_ERROR, "신원", "파일명과 과제 id 가 다르다"),
    (CODE_DUPLICATE_TASK_ID, SEVERITY_ERROR, "신원", "같은 pack 에서 과제 id 가 겹친다"),
    (CODE_MISSING_TIER, SEVERITY_ERROR, "신원", "tier 키가 없다"),
    (CODE_TIER_TYPE, SEVERITY_ERROR, "신원", "tier 가 정수가 아니다"),
    (CODE_TIER_RANGE, SEVERITY_ERROR, "신원", "tier 가 1~5 밖이다"),
    (CODE_MISSING_INPUT, SEVERITY_ERROR, "입력", "input 키가 없다"),
    (CODE_EMPTY_INPUT, SEVERITY_ERROR, "입력", "input 이 비었다"),
    (CODE_INPUT_TYPE, SEVERITY_ERROR, "입력", "input 이 문자열이 아니다"),
    (CODE_INPUT_WHITESPACE, SEVERITY_ERROR, "입력", "input 앞뒤에 공백이 있다"),
    (CODE_INPUT_ABSOLUTE, SEVERITY_ERROR, "입력", "input 이 절대 경로다"),
    (CODE_INPUT_BACKSLASH, SEVERITY_ERROR, "입력", "input 에 백슬래시가 있다"),
    (CODE_INPUT_PARENT, SEVERITY_ERROR, "입력", "input 이 상위 디렉터리를 가리킨다"),
    (CODE_EMPTY_REFERENCE_STEPS, SEVERITY_ERROR, "기준풀이", "reference 는 있는데 steps 가 비었다"),
    (CODE_REFERENCE_STEPS_TYPE, SEVERITY_ERROR, "기준풀이", "reference.steps 가 배열이 아니다"),
    (CODE_REFERENCE_ID_MISMATCH, SEVERITY_ERROR, "기준풀이", "reference.id 가 과제 id 와 다르다"),
    (CODE_REFERENCE_STEP_TYPE, SEVERITY_ERROR, "기준풀이", "steps 항목이 객체가 아니다"),
    (CODE_REFERENCE_RUN_EMPTY, SEVERITY_ERROR, "기준풀이", "steps.run 이 비었다"),
    (CODE_REFERENCE_ANSWER_EMPTY, SEVERITY_ERROR, "기준풀이", "steps.answer 가 비었다"),
    (CODE_REFERENCE_CMD_EMPTY, SEVERITY_ERROR, "기준풀이", "answer 항목의 cmd 가 비었다"),
    (CODE_ORPHAN_REFERENCE, SEVERITY_ERROR, "기준풀이", "짝 과제가 없는 reference 다"),
    (CODE_UNKNOWN_SUBMIT_KIND, SEVERITY_ERROR, "제출", "submit.kind 를 모른다"),
    (CODE_MISSING_SUBMIT, SEVERITY_ERROR, "제출", "submit 이 없다"),
    (CODE_MISSING_SUBMIT_KIND, SEVERITY_ERROR, "제출", "submit.kind 가 없다"),
    (CODE_SUBMIT_TYPE, SEVERITY_ERROR, "제출", "submit 이 객체가 아니다"),
    (CODE_ARTIFACT_WITHOUT_FILES, SEVERITY_WARNING, "제출", "artifact 인데 files 가 비었다"),
    (CODE_PAIR_WITHOUT_FILES, SEVERITY_WARNING, "제출", "pair 인데 files 가 2개 미만이다"),
    (CODE_SUBMIT_FILES_TYPE, SEVERITY_ERROR, "제출", "submit.files 가 배열이 아니다"),
    (CODE_SUBMIT_FILE_EMPTY, SEVERITY_ERROR, "제출", "submit.files 항목이 비었다"),
    (CODE_SUBMIT_FILE_TYPE, SEVERITY_ERROR, "제출", "submit.files 항목이 문자열이 아니다"),
    (CODE_SUBMIT_FILE_WHITESPACE, SEVERITY_ERROR, "제출", "submit.files 항목 앞뒤에 공백이 있다"),
    (CODE_SUBMIT_FILE_ABSOLUTE, SEVERITY_ERROR, "제출", "submit.files 가 절대 경로다"),
    (CODE_SUBMIT_FILE_BACKSLASH, SEVERITY_ERROR, "제출", "submit.files 에 백슬래시가 있다"),
    (CODE_SUBMIT_FILE_DUPLICATE, SEVERITY_ERROR, "제출", "submit.files 가 중복이다"),
    (CODE_HINT_ANSWER_DUMP, SEVERITY_ERROR, "힌트", "힌트에 정답 JSON 이 있다"),
    (CODE_HINT_SPOILER, SEVERITY_ERROR, "힌트", "힌트가 정답을 직접 적었다"),
    (CODE_HINT_EMBEDS_VALUE, SEVERITY_ERROR, "힌트", "힌트가 검사 기대값을 복붙했다"),
    (CODE_PARSE_ERROR, SEVERITY_ERROR, "구조", "JSON 파싱에 실패했다"),
    (CODE_MISSING_PACK, SEVERITY_ERROR, "구조", "pack.json 이 없다"),
    (CODE_MISSING_TASKS, SEVERITY_ERROR, "구조", "tasks/ 가 없다"),
    (CODE_PACK_TYPE, SEVERITY_ERROR, "매니페스트", "pack.json 이 객체가 아니다"),
    (CODE_PACK_KIND, SEVERITY_ERROR, "매니페스트", "kind 가 gymPack 이 아니다"),
    (CODE_PACK_SCHEMA_VERSION, SEVERITY_ERROR, "매니페스트", "schemaVersion 이 1.0 이 아니다"),
    (CODE_PACK_ID_MISMATCH, SEVERITY_ERROR, "매니페스트", "pack.id 가 폴더 이름과 다르다"),
    (CODE_PACK_EMPTY_TITLE, SEVERITY_ERROR, "매니페스트", "pack.title 이 비었다"),
    (CODE_PACK_EMPTY_AXIS, SEVERITY_ERROR, "매니페스트", "pack.axis 가 비었다"),
    (CODE_PACK_TITLE_WHITESPACE, SEVERITY_ERROR, "매니페스트", "pack.title 앞뒤에 공백이 있다"),
    (CODE_PACK_AXIS_WHITESPACE, SEVERITY_ERROR, "매니페스트", "pack.axis 앞뒤에 공백이 있다"),
    (CODE_PACK_MISSING_REQUIRES, SEVERITY_ERROR, "매니페스트", "requires.commands 가 없다"),
    (CODE_PACK_EMPTY_COMMANDS, SEVERITY_ERROR, "매니페스트", "requires.commands 가 비었다"),
    (CODE_PACK_COMMAND_TYPE, SEVERITY_ERROR, "매니페스트", "requires.commands 항목이 문자열이 아니다"),
    (CODE_PACK_MISSING_RUNNER, SEVERITY_ERROR, "매니페스트", "runner 가 없다"),
    (CODE_PACK_MISSING_RUNNER_FIELD, SEVERITY_ERROR, "매니페스트", "runner 신원 필드가 비었다"),
    (CODE_EMPTY_PACK, SEVERITY_WARNING, "구조", "tasks/ 에 과제가 없다"),
)

_OPS_CACHE: tuple[frozenset[str], frozenset[str], frozenset[str], frozenset[str]] | None = None


def issue(
    code: str,
    message: str,
    *,
    task: str | None = None,
    where: str = "",
    severity: str = SEVERITY_ERROR,
    extra: dict | None = None,
) -> dict:
    """한 이슈 — 리포트·시험이 같은 키를 본다."""
    row = {
        "code": code,
        "severity": severity,
        "task": task,
        "where": where,
        "message": message,
    }
    if extra:
        row.update(extra)
    return row


def _load_json(path: str):
    """(doc, error). 성공이면 error 는 None."""
    try:
        with open(path, encoding="utf-8") as fh:
            return json.load(fh), None
    except FileNotFoundError as exc:
        return None, f"파일 없음: {exc}"
    except (ValueError, OSError) as exc:
        return None, f"파싱 실패: {exc}"


def has_edge_whitespace(text: str) -> bool:
    return text != text.strip()


def has_any_whitespace(text: str) -> bool:
    return any(ch.isspace() for ch in text)


def instruction_length(text: str) -> int:
    """앞뒤 공백을 뺀 글자 수. 한글도 코드포인트 1글자."""
    return len(text.strip())


def _marker_is_parenthetical(text: str, idx: int) -> bool:
    """`(힌트: export-text)` 처럼 본문 안 괄호 힌트는 꼬리 마커가 아니다."""
    before = text[:idx]
    if before.rstrip().endswith("("):
        return True
    return before.count("(") > before.count(")")


def _marker_is_section(text: str, idx: int) -> bool:
    """문장 끝·줄바꿈·맨 앞의 힌트 꼬리만 센다."""
    if _marker_is_parenthetical(text, idx):
        return False
    if idx == 0:
        return True
    if text[idx - 1] in "\n\r":
        return True
    prefix = text[:idx].rstrip()
    return prefix.endswith((".", "。", "!", "?", "다", "라", "요"))


def iter_hint_markers(text: str) -> list[tuple[int, str, bool]]:
    """(index, marker, is_section) 출현 목록."""
    found: list[tuple[int, str, bool]] = []
    if not text:
        return found
    for marker in HINT_MARKERS:
        start = 0
        while True:
            idx = text.find(marker, start)
            if idx < 0:
                break
            found.append((idx, marker, _marker_is_section(text, idx)))
            start = idx + len(marker)
    found.sort(key=lambda row: row[0])
    return found


def split_hint(instructions: str) -> tuple[str, str | None]:
    """본문과 힌트 꼬리를 가른다. 마커가 없으면 힌트는 None.

    괄호 안 `(힌트: export-text)` 는 본문 안내로 보고, 문장 끝의 꼬리 마커를
    우선한다. 꼬리가 없으면 첫 마커로 후퇴한다.
    """
    if not isinstance(instructions, str):
        return "", None
    found = iter_hint_markers(instructions)
    if not found:
        return instructions, None
    section = [row for row in found if row[2]]
    chosen = section[0] if section else found[0]
    found_at, found_marker, _is_section = chosen
    body = instructions[:found_at]
    hint = instructions[found_at + len(found_marker) :]
    return body, hint


def extract_json_fragments(text: str) -> list[object]:
    """본문에서 `{…}` / `[…]` JSON 조각을 최대한 건진다. 깨진 중괄호는 건너뛴다."""
    if not text:
        return []
    found: list[object] = []
    n = len(text)
    i = 0
    while i < n:
        if text[i] not in "{[":
            i += 1
            continue
        opener = text[i]
        closer = "}" if opener == "{" else "]"
        depth = 0
        in_str = False
        escape = False
        j = i
        while j < n:
            ch = text[j]
            if in_str:
                if escape:
                    escape = False
                elif ch == "\\":
                    escape = True
                elif ch == '"':
                    in_str = False
            else:
                if ch == '"':
                    in_str = True
                elif ch == opener:
                    depth += 1
                elif ch == closer:
                    depth -= 1
                    if depth == 0:
                        blob = text[i : j + 1]
                        try:
                            found.append(json.loads(blob))
                        except ValueError:
                            pass
                        i = j + 1
                        break
            j += 1
        else:
            i += 1
    return found


def is_placeholder_value(value) -> bool:
    """`<수>`, `{input}` 처럼 자리를 비워 둔 값은 정답 노출이 아니다."""
    if not isinstance(value, str):
        return False
    stripped = value.strip()
    if not stripped:
        return False
    if PLACEHOLDER_RE.fullmatch(stripped):
        return True
    if stripped.startswith("<") and stripped.endswith(">"):
        return True
    return False


def fragment_looks_like_answer(obj) -> bool:
    """구체 스칼라만 가진 작은 객체/배열이면 정답 덤프로 본다.

    키가 `<필드이름>` 처럼 자리표이면 CLI 문법 예시이지 정답 봉투가 아니다.
    """
    if isinstance(obj, dict):
        if not obj or len(obj) > 8:
            return False
        if any(is_placeholder_value(str(key)) or PLACEHOLDER_RE.search(str(key)) for key in obj):
            return False
        scalars = 0
        for value in obj.values():
            if isinstance(value, (dict, list)):
                return False
            if is_placeholder_value(value):
                continue
            if isinstance(value, str) and not value.strip():
                continue
            if isinstance(value, str) and PLACEHOLDER_RE.search(value):
                continue
            scalars += 1
        return scalars >= 1
    if isinstance(obj, list):
        if not obj or len(obj) > 12:
            return False
        if all(is_placeholder_value(v) or v in ("...", "…") for v in obj):
            return False
        return all(isinstance(v, (str, int, float, bool)) or v is None for v in obj)
    return False


def token_appears_bare(text: str, token: str) -> bool:
    """토큰이 파일명·명령 일부(`export-hwpx`, `conv.hwpx`)가 아니라 단독으로 있나."""
    if not text or not token:
        return False
    if token.isascii() and any(ch.isalnum() for ch in token):
        pattern = r"(?<![A-Za-z0-9_.-])" + re.escape(token) + r"(?![A-Za-z0-9_.-])"
        return re.search(pattern, text) is not None
    return token in text


def check_expected_literals(check: dict) -> list[str]:
    """채점이 기대하는 구체 값 — 힌트에 그대로 있으면 답을 준 것이다."""
    out: list[str] = []
    for key in ("value", "expected"):
        raw = check.get(key)
        if isinstance(raw, bool):
            continue
        if isinstance(raw, (int, float)):
            # 0/1 은 쪽수·건수 힌트에 흔해 spoiler 로 치지 않는다.
            if isinstance(raw, int) and raw in (0, 1, -1):
                continue
            out.append(str(raw))
        elif isinstance(raw, str):
            token = raw.strip()
            if len(token) >= MIN_HINT_VALUE_CHARS and token.lower() not in COMMON_HINT_TOKENS:
                out.append(token)
    return out


def known_ops_bundle() -> tuple[frozenset[str], frozenset[str], frozenset[str], frozenset[str]]:
    """(all_ops, cli_ops, file_ops, global_ops). core.checks 를 우선한다."""
    global _OPS_CACHE
    if _OPS_CACHE is not None:
        return _OPS_CACHE
    all_ops = FALLBACK_FILE_OPS | FALLBACK_CLI_OPS
    cli_ops = FALLBACK_CLI_OPS
    file_ops = FALLBACK_FILE_OPS
    global_ops = FALLBACK_GLOBAL_OPS
    try:
        if GYM_ROOT not in sys.path:
            sys.path.insert(0, GYM_ROOT)
        from core import checks as check_registry  # type: ignore

        registry = getattr(check_registry, "REGISTRY", None)
        if isinstance(registry, dict) and registry:
            all_ops = frozenset(registry)
            cli_ops = frozenset(
                op for op, spec in registry.items() if isinstance(spec, tuple) and spec[1]
            )
            file_ops = all_ops - cli_ops
        scanned = getattr(check_registry, "GLOBAL_SCAN_OPS", None)
        if isinstance(scanned, (set, frozenset, list, tuple)) and scanned:
            global_ops = frozenset(scanned)
    except Exception:
        pass
    _OPS_CACHE = (all_ops, cli_ops, file_ops, global_ops)
    return _OPS_CACHE


def reset_ops_cache() -> None:
    """단위시험이 등록부를 갈아끼운 뒤 호출한다."""
    global _OPS_CACHE
    _OPS_CACHE = None


def catalog_codes() -> list[str]:
    return [row[0] for row in ISSUE_CATALOG]


def catalog_entry(code: str) -> tuple[str, str, str] | None:
    for item_code, severity, layer, summary in ISSUE_CATALOG:
        if item_code == code:
            return severity, layer, summary
    return None


def render_codes() -> str:
    """이슈 코드 목록 — `--codes` 가 그대로 찍는다."""
    lines = ["code\tseverity\tlayer\tsummary"]
    for code, severity, layer, summary in ISSUE_CATALOG:
        lines.append(f"{code}\t{severity}\t{layer}\t{summary}")
    return "\n".join(lines)


def is_absolute_path(path: str) -> bool:
    if not path:
        return False
    if path.startswith("/") or path.startswith("\\"):
        return True
    if len(path) >= 2 and path[0].isalpha() and path[1] == ":":
        return True
    return False


def has_backslash(path: str) -> bool:
    return "\\" in path


def has_parent_traversal(path: str) -> bool:
    parts = re.split(r"[\\/]+", path)
    return ".." in parts


def is_nonneg_int(value) -> bool:
    """좌표는 0 이상 정수. bool 은 int 하위형이라 거절한다."""
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def pack_axis_is_editing(axis: str | None) -> bool:
    if not isinstance(axis, str) or not axis:
        return False
    return any(axis.startswith(prefix) for prefix in EDITING_AXES)


def effective_axis(task: dict, pack_axis: str | None) -> str:
    raw = task.get("axis")
    if isinstance(raw, str) and raw.strip():
        return raw
    return pack_axis or ""


def count_hint_markers(text: str) -> int:
    """문장 끝 힌트 꼬리만 센다. 괄호 안 안내는 중복으로 치지 않는다."""
    return sum(1 for _idx, _marker, is_section in iter_hint_markers(text) if is_section)


def iter_pack_dirs(packs_root: str, pack_ids: list[str] | None = None) -> list[tuple[str, str]]:
    """(pack_id, pack_dir). packs_root 는 gym/ (packs/ 를 담은 곳)."""
    packs_dir = os.path.join(packs_root, "packs")
    if not os.path.isdir(packs_dir):
        return []
    names = sorted(os.listdir(packs_dir))
    wanted = set(pack_ids) if pack_ids else None
    rows = []
    for name in names:
        if wanted is not None and name not in wanted:
            continue
        path = os.path.join(packs_dir, name)
        if os.path.isdir(path):
            rows.append((name, path))
    return rows


def list_json_names(directory: str) -> list[str]:
    if not os.path.isdir(directory):
        return []
    return sorted(name for name in os.listdir(directory) if name.endswith(".json"))


def scan_instructions(task: dict, where: str, tid: str | None, min_chars: int) -> list[dict]:
    issues: list[dict] = []
    if "instructions" not in task:
        issues.append(
            issue(
                CODE_EMPTY_INSTRUCTIONS,
                "instructions 키가 없다",
                task=tid,
                where=where,
            )
        )
        return issues
    text = task.get("instructions")
    if not isinstance(text, str):
        issues.append(
            issue(
                CODE_INSTRUCTIONS_TYPE,
                f"instructions 가 문자열이 아니다 ({type(text).__name__})",
                task=tid,
                where=where,
            )
        )
        return issues
    length = instruction_length(text)
    if length == 0:
        issues.append(
            issue(
                CODE_EMPTY_INSTRUCTIONS,
                "instructions 가 비었다",
                task=tid,
                where=where,
            )
        )
    elif length < min_chars:
        issues.append(
            issue(
                CODE_SHORT_INSTRUCTIONS,
                f"instructions 가 {length}글자(< {min_chars})다",
                task=tid,
                where=where,
                extra={"length": length, "min": min_chars},
            )
        )
    issues.extend(scan_instruction_quality(text, where, tid))
    return issues


def scan_instruction_quality(text: str, where: str, tid: str | None) -> list[dict]:
    """길이·타입 다음 층 — 자리표·힌트 골격·제어 문자."""
    issues: list[dict] = []
    if CONTROL_CHAR_RE.search(text):
        issues.append(
            issue(
                CODE_INSTRUCTIONS_CONTROL_CHAR,
                "instructions 에 제어 문자가 있다",
                task=tid,
                where=where,
            )
        )
    if TODO_RE.search(text):
        issues.append(
            issue(
                CODE_INSTRUCTIONS_TODO,
                "instructions 에 TODO/FIXME 자리표가 남아 있다",
                task=tid,
                where=where,
            )
        )
    marker_count = count_hint_markers(text)
    if marker_count > 1:
        issues.append(
            issue(
                CODE_DUPLICATE_HINT_MARKER,
                f"힌트 마커가 {marker_count}번 나온다",
                task=tid,
                where=where,
                severity=SEVERITY_WARNING,
                extra={"count": marker_count},
            )
        )
    body, hint = split_hint(text)
    if hint is not None:
        if not body.strip():
            issues.append(
                issue(
                    CODE_INSTRUCTIONS_HINT_ONLY,
                    "힌트 마커 앞에 과제 본문이 없다",
                    task=tid,
                    where=where,
                )
            )
        if not hint.strip():
            issues.append(
                issue(
                    CODE_EMPTY_HINT,
                    "힌트 마커는 있는데 내용이 비었다",
                    task=tid,
                    where=where,
                    severity=SEVERITY_WARNING,
                )
            )
    return issues


def scan_identity(
    task: dict,
    filename: str,
    where: str,
    min_title: int = MIN_TITLE_CHARS,
) -> list[dict]:
    """id/title 공백·공란·파일명 불일치·짧은 title."""
    issues: list[dict] = []
    tid = task.get("id")
    title = task.get("title")
    shown = tid if isinstance(tid, str) and tid.strip() else None

    if not isinstance(tid, str) or not tid.strip():
        issues.append(
            issue(CODE_EMPTY_TASK_ID, "과제 id 가 비었다", task=shown, where=where)
        )
    elif has_edge_whitespace(tid) or has_any_whitespace(tid):
        issues.append(
            issue(
                CODE_TASK_ID_WHITESPACE,
                f"과제 id 에 공백이 있다: {tid!r}",
                task=tid.strip() or shown,
                where=where,
            )
        )
    elif filename.endswith(".json"):
        stem = filename[: -len(".json")]
        if stem != tid:
            issues.append(
                issue(
                    CODE_ID_FILENAME_MISMATCH,
                    f"파일명({stem}) 과 과제 id({tid}) 가 다르다",
                    task=tid,
                    where=where,
                )
            )

    if not isinstance(title, str) or not title.strip():
        issues.append(
            issue(CODE_EMPTY_TITLE, "과제 title 이 비었다", task=shown, where=where)
        )
    elif has_edge_whitespace(title):
        issues.append(
            issue(
                CODE_TASK_TITLE_WHITESPACE,
                f"과제 title 앞뒤에 공백이 있다: {title!r}",
                task=shown,
                where=where,
            )
        )
    elif len(title.strip()) < min_title:
        issues.append(
            issue(
                CODE_TITLE_TOO_SHORT,
                f"과제 title 이 {len(title.strip())}글자(< {min_title})다",
                task=shown,
                where=where,
                extra={"length": len(title.strip()), "min": min_title},
            )
        )
    return issues


def scan_tier(task: dict, where: str, tid: str | None) -> list[dict]:
    issues: list[dict] = []
    if "tier" not in task:
        issues.append(issue(CODE_MISSING_TIER, "tier 키가 없다", task=tid, where=where))
        return issues
    tier = task.get("tier")
    if isinstance(tier, bool) or not isinstance(tier, int):
        issues.append(
            issue(
                CODE_TIER_TYPE,
                f"tier 가 정수가 아니다 ({type(tier).__name__})",
                task=tid,
                where=where,
            )
        )
        return issues
    if tier < 1 or tier > 5:
        issues.append(
            issue(
                CODE_TIER_RANGE,
                f"tier 가 {tier} 이다 (허용 1~5)",
                task=tid,
                where=where,
                extra={"tier": tier},
            )
        )
    return issues


def scan_input(task: dict, where: str, tid: str | None) -> list[dict]:
    issues: list[dict] = []
    if "input" not in task:
        issues.append(issue(CODE_MISSING_INPUT, "input 키가 없다", task=tid, where=where))
        return issues
    raw = task.get("input")
    if not isinstance(raw, str):
        issues.append(
            issue(
                CODE_INPUT_TYPE,
                f"input 이 문자열이 아니다 ({type(raw).__name__})",
                task=tid,
                where=where,
            )
        )
        return issues
    if not raw.strip():
        issues.append(issue(CODE_EMPTY_INPUT, "input 이 비었다", task=tid, where=where))
        return issues
    if has_edge_whitespace(raw):
        issues.append(
            issue(
                CODE_INPUT_WHITESPACE,
                f"input 앞뒤에 공백이 있다: {raw!r}",
                task=tid,
                where=where,
            )
        )
    if is_absolute_path(raw):
        issues.append(
            issue(
                CODE_INPUT_ABSOLUTE,
                f"input 이 절대 경로다: {raw}",
                task=tid,
                where=where,
            )
        )
    if has_backslash(raw):
        issues.append(
            issue(
                CODE_INPUT_BACKSLASH,
                "input 경로에 백슬래시가 있다 — POSIX 상대 경로를 써라",
                task=tid,
                where=where,
            )
        )
    if has_parent_traversal(raw):
        issues.append(
            issue(
                CODE_INPUT_PARENT,
                "input 이 상위 디렉터리(..) 를 가리킨다",
                task=tid,
                where=where,
            )
        )
    return issues


def scan_checks(
    task: dict,
    where: str,
    tid: str | None,
    pack_axis: str | None = None,
) -> list[dict]:
    issues: list[dict] = []
    checks = task.get("checks")
    if checks is None:
        issues.append(
            issue(CODE_EMPTY_CHECKS, "checks 가 없다", task=tid, where=where)
        )
        return issues
    if not isinstance(checks, list):
        issues.append(
            issue(
                CODE_CHECKS_TYPE,
                f"checks 가 배열이 아니다 ({type(checks).__name__})",
                task=tid,
                where=where,
            )
        )
        return issues
    if not checks:
        issues.append(
            issue(CODE_EMPTY_CHECKS, "checks 가 비었다", task=tid, where=where)
        )
        return issues

    names: list[str] = []
    for index, check in enumerate(checks):
        prefix = f"{where}#checks[{index}]"
        if not isinstance(check, dict):
            issues.append(
                issue(
                    CODE_CHECK_TYPE,
                    f"checks[{index}] 가 객체가 아니다",
                    task=tid,
                    where=prefix,
                )
            )
            continue
        if "name" not in check or check.get("name") is None:
            issues.append(
                issue(
                    CODE_MISSING_CHECK_NAME,
                    f"checks[{index}] 에 name 이 없다",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
            continue
        raw_name = check.get("name")
        if not isinstance(raw_name, str):
            issues.append(
                issue(
                    CODE_MISSING_CHECK_NAME,
                    f"checks[{index}].name 이 문자열이 아니다 ({type(raw_name).__name__})",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
            continue
        if not raw_name.strip():
            issues.append(
                issue(
                    CODE_EMPTY_CHECK_NAME,
                    f"checks[{index}].name 이 비었다",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
            continue
        if has_edge_whitespace(raw_name):
            issues.append(
                issue(
                    CODE_CHECK_NAME_WHITESPACE,
                    f"checks[{index}].name 앞뒤에 공백이 있다: {raw_name!r}",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
        names.append(raw_name.strip())

    counts = Counter(names)
    for name, count in counts.items():
        if count > 1:
            issues.append(
                issue(
                    CODE_DUPLICATE_CHECK_NAME,
                    f"check.name {name!r} 이 이 과제에서 {count}번 나온다",
                    task=tid,
                    where=where,
                    extra={"name": name, "count": count},
                )
            )

    axis = effective_axis(task, pack_axis)
    for index, check in enumerate(checks):
        if isinstance(check, dict):
            issues.extend(
                scan_check_contract(
                    check,
                    f"{where}#checks[{index}]",
                    tid,
                    index,
                    axis,
                )
            )
    return issues


def scan_path_field(
    raw,
    *,
    code_empty: str,
    code_absolute: str,
    code_backslash: str,
    label: str,
    tid: str | None,
    where: str,
    extra: dict | None = None,
) -> list[dict]:
    """상대 POSIX 경로 위생. 문자열이 아니면 호출하지 않는다."""
    issues: list[dict] = []
    if not raw.strip():
        issues.append(
            issue(code_empty, f"{label} 이 비었다", task=tid, where=where, extra=extra)
        )
        return issues
    if is_absolute_path(raw):
        issues.append(
            issue(
                code_absolute,
                f"{label} 이 절대 경로다: {raw}",
                task=tid,
                where=where,
                extra=extra,
            )
        )
    if has_backslash(raw):
        issues.append(
            issue(
                code_backslash,
                f"{label} 에 백슬래시가 있다",
                task=tid,
                where=where,
                extra=extra,
            )
        )
    return issues


def scan_check_contract(
    check: dict,
    where: str,
    tid: str | None,
    index: int,
    axis: str,
) -> list[dict]:
    """op·필수 필드·cmd 계약. 이름은 scan_checks 가 이미 봤다."""
    issues: list[dict] = []
    extra = {"index": index}
    all_ops, cli_ops, file_ops, global_ops = known_ops_bundle()

    if "op" not in check or check.get("op") is None:
        issues.append(
            issue(CODE_MISSING_CHECK_OP, f"checks[{index}] 에 op 가 없다", task=tid, where=where, extra=extra)
        )
        return issues
    op = check.get("op")
    if not isinstance(op, str):
        issues.append(
            issue(
                CODE_MISSING_CHECK_OP,
                f"checks[{index}].op 이 문자열이 아니다 ({type(op).__name__})",
                task=tid,
                where=where,
                extra=extra,
            )
        )
        return issues
    if not op.strip():
        issues.append(
            issue(CODE_EMPTY_CHECK_OP, f"checks[{index}].op 이 비었다", task=tid, where=where, extra=extra)
        )
        return issues
    if op not in all_ops:
        issues.append(
            issue(
                CODE_UNKNOWN_CHECK_OP,
                f"checks[{index}].op {op!r} 는 등록되지 않았다",
                task=tid,
                where=where,
                extra={"index": index, "op": op},
            )
        )
        return issues

    if op in global_ops and pack_axis_is_editing(axis) and not check.get("allowGlobalScan"):
        issues.append(
            issue(
                CODE_GLOBAL_SCAN_UNDECLARED,
                f"편집/보안 축에서 {op} 를 쓰려면 allowGlobalScan 사유가 필요하다",
                task=tid,
                where=where,
                extra={"index": index, "op": op, "axis": axis},
            )
        )

    if op in OPS_NEED_ANSWER and "answer" not in check:
        issues.append(
            issue(
                CODE_CHECK_MISSING_ANSWER,
                f"{op} 에 answer 키가 없다",
                task=tid,
                where=where,
                extra={"index": index, "op": op},
            )
        )
    if op in OPS_NEED_VALUE and "value" not in check:
        issues.append(
            issue(
                CODE_CHECK_MISSING_VALUE,
                f"{op} 에 value 키가 없다",
                task=tid,
                where=where,
                extra={"index": index, "op": op},
            )
        )

    if op in OPS_NEED_FILE:
        raw_file = check.get("file")
        if not isinstance(raw_file, str):
            issues.append(
                issue(
                    CODE_CHECK_MISSING_FILE,
                    f"{op} 에 file 이 없다",
                    task=tid,
                    where=where,
                    extra={"index": index, "op": op},
                )
            )
        else:
            issues.extend(
                scan_path_field(
                    raw_file,
                    code_empty=CODE_CHECK_FILE_EMPTY,
                    code_absolute=CODE_CHECK_FILE_ABSOLUTE,
                    code_backslash=CODE_CHECK_FILE_BACKSLASH,
                    label="check.file",
                    tid=tid,
                    where=where,
                    extra={"index": index, "op": op},
                )
            )

    if op in OPS_NEED_FILES:
        files = check.get("files")
        if not isinstance(files, list):
            issues.append(
                issue(
                    CODE_CHECK_MISSING_FILES,
                    f"{op} 에 files 가 없다",
                    task=tid,
                    where=where,
                    extra={"index": index, "op": op},
                )
            )
        elif len(files) < 2:
            issues.append(
                issue(
                    CODE_CHECK_FILES_SHORT,
                    f"{op} 의 files 가 {len(files)}개다 (2개 필요)",
                    task=tid,
                    where=where,
                    extra={"index": index, "op": op, "count": len(files)},
                )
            )
        else:
            for f_index, item in enumerate(files):
                if not isinstance(item, str):
                    issues.append(
                        issue(
                            CODE_CHECK_FILE_EMPTY,
                            f"{op}.files[{f_index}] 가 문자열이 아니다",
                            task=tid,
                            where=where,
                            extra={"index": index, "op": op},
                        )
                    )
                    continue
                issues.extend(
                    scan_path_field(
                        item,
                        code_empty=CODE_CHECK_FILE_EMPTY,
                        code_absolute=CODE_CHECK_FILE_ABSOLUTE,
                        code_backslash=CODE_CHECK_FILE_BACKSLASH,
                        label=f"check.files[{f_index}]",
                        tid=tid,
                        where=where,
                        extra={"index": index, "op": op},
                    )
                )

    if op in OPS_NEED_CELL_COORD:
        missing = [key for key in ("table", "row", "col") if not is_nonneg_int(check.get(key))]
        if missing:
            issues.append(
                issue(
                    CODE_CELL_MISSING_COORD,
                    f"cell_text_eq 에 좌표 {', '.join(missing)} 가 없거나 정수가 아니다",
                    task=tid,
                    where=where,
                    extra={"index": index, "missing": missing},
                )
            )
    if op in OPS_NEED_CSV_COORD:
        missing = [key for key in ("row", "col") if not is_nonneg_int(check.get(key))]
        if missing:
            issues.append(
                issue(
                    CODE_CSV_MISSING_COORD,
                    f"csv_cell_eq 에 좌표 {', '.join(missing)} 가 없거나 정수가 아니다",
                    task=tid,
                    where=where,
                    extra={"index": index, "missing": missing},
                )
            )

    cmd = check.get("cmd")
    if op in cli_ops:
        if "cmd" not in check or cmd is None:
            issues.append(
                issue(
                    CODE_CHECK_MISSING_CMD,
                    f"CLI 연산자 {op} 에 cmd 가 없다",
                    task=tid,
                    where=where,
                    extra={"index": index, "op": op},
                )
            )
        elif not isinstance(cmd, list):
            issues.append(
                issue(
                    CODE_CHECK_CMD_TYPE,
                    f"{op}.cmd 가 배열이 아니다 ({type(cmd).__name__})",
                    task=tid,
                    where=where,
                    extra={"index": index, "op": op},
                )
            )
        elif not cmd:
            issues.append(
                issue(
                    CODE_CHECK_CMD_EMPTY,
                    f"{op}.cmd 가 비었다",
                    task=tid,
                    where=where,
                    extra={"index": index, "op": op},
                )
            )
        else:
            for c_index, item in enumerate(cmd):
                if not isinstance(item, str) or not item.strip():
                    issues.append(
                        issue(
                            CODE_CHECK_CMD_ITEM_TYPE,
                            f"{op}.cmd[{c_index}] 가 빈 값이거나 문자열이 아니다",
                            task=tid,
                            where=where,
                            extra={"index": index, "op": op, "cmdIndex": c_index},
                        )
                    )
                    break
    elif op in file_ops and cmd:
        issues.append(
            issue(
                CODE_CHECK_UNEXPECTED_CMD,
                f"파일 연산자 {op} 는 CLI 를 부르지 않는데 cmd 가 있다",
                task=tid,
                where=where,
                extra={"index": index, "op": op},
            )
        )
    return issues


def scan_submit(task: dict, where: str, tid: str | None) -> list[dict]:
    issues: list[dict] = []
    if "submit" not in task:
        issues.append(
            issue(CODE_MISSING_SUBMIT, "submit 이 없다", task=tid, where=where)
        )
        return issues
    submit = task.get("submit")
    if not isinstance(submit, dict):
        issues.append(
            issue(
                CODE_SUBMIT_TYPE,
                f"submit 이 객체가 아니다 ({type(submit).__name__})",
                task=tid,
                where=where,
            )
        )
        return issues
    if "kind" not in submit or submit.get("kind") is None:
        issues.append(
            issue(
                CODE_MISSING_SUBMIT_KIND,
                "submit.kind 가 없다",
                task=tid,
                where=where,
            )
        )
        return issues
    kind = submit.get("kind")
    if not isinstance(kind, str) or not kind.strip():
        issues.append(
            issue(
                CODE_UNKNOWN_SUBMIT_KIND,
                f"submit.kind 가 비었다: {kind!r}",
                task=tid,
                where=where,
                extra={"kind": kind},
            )
        )
        return issues
    if kind not in KNOWN_SUBMIT_KINDS:
        issues.append(
            issue(
                CODE_UNKNOWN_SUBMIT_KIND,
                f"submit.kind {kind!r} 는 모른다 (허용: {', '.join(sorted(KNOWN_SUBMIT_KINDS))})",
                task=tid,
                where=where,
                extra={"kind": kind},
            )
        )
        return issues

    files = submit.get("files")
    if kind == "artifact":
        if not isinstance(files, list) or not files:
            issues.append(
                issue(
                    CODE_ARTIFACT_WITHOUT_FILES,
                    "submit.kind=artifact 인데 files 가 비었다",
                    task=tid,
                    where=where,
                    severity=SEVERITY_WARNING,
                )
            )
    elif kind == "pair":
        if not isinstance(files, list) or len(files) < 2:
            issues.append(
                issue(
                    CODE_PAIR_WITHOUT_FILES,
                    "submit.kind=pair 인데 files 가 2개 미만이다",
                    task=tid,
                    where=where,
                    severity=SEVERITY_WARNING,
                )
            )
    if "files" in submit:
        issues.extend(scan_submit_files(files, where, tid))
    return issues


def scan_submit_files(files, where: str, tid: str | None) -> list[dict]:
    issues: list[dict] = []
    if not isinstance(files, list):
        issues.append(
            issue(
                CODE_SUBMIT_FILES_TYPE,
                f"submit.files 가 배열이 아니다 ({type(files).__name__})",
                task=tid,
                where=where,
            )
        )
        return issues
    seen: dict[str, int] = {}
    for index, item in enumerate(files):
        prefix = f"{where}#submit.files[{index}]"
        if not isinstance(item, str):
            issues.append(
                issue(
                    CODE_SUBMIT_FILE_TYPE,
                    f"submit.files[{index}] 가 문자열이 아니다",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
            continue
        if not item.strip():
            issues.append(
                issue(
                    CODE_SUBMIT_FILE_EMPTY,
                    f"submit.files[{index}] 가 비었다",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
            continue
        if has_edge_whitespace(item):
            issues.append(
                issue(
                    CODE_SUBMIT_FILE_WHITESPACE,
                    f"submit.files[{index}] 앞뒤에 공백이 있다: {item!r}",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
        if is_absolute_path(item):
            issues.append(
                issue(
                    CODE_SUBMIT_FILE_ABSOLUTE,
                    f"submit.files[{index}] 가 절대 경로다: {item}",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
        if has_backslash(item):
            issues.append(
                issue(
                    CODE_SUBMIT_FILE_BACKSLASH,
                    f"submit.files[{index}] 에 백슬래시가 있다",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
        key = item.strip()
        if key in seen:
            issues.append(
                issue(
                    CODE_SUBMIT_FILE_DUPLICATE,
                    f"submit.files 에 {key!r} 가 중복이다",
                    task=tid,
                    where=where,
                    extra={"file": key},
                )
            )
        else:
            seen[key] = index
    return issues


def _steps_are_vacuous(steps: list) -> bool:
    if not steps:
        return True
    for step in steps:
        if not isinstance(step, dict):
            return False
        if not step:
            continue
        if any(value not in (None, "", [], {}) for value in step.values()):
            return False
    return True


def scan_reference(ref_path: str, filename: str, tid: str | None) -> list[dict]:
    """짝 기준풀이가 있을 때만 호출. 없으면 이 도구의 소관이 아니다(audit 몫)."""
    where = f"reference/{filename}"
    if not os.path.isfile(ref_path):
        return []
    doc, err = _load_json(ref_path)
    if err:
        return [
            issue(
                CODE_PARSE_ERROR,
                f"reference/{filename} {err}",
                task=tid,
                where=where,
            )
        ]
    if not isinstance(doc, dict):
        return [
            issue(
                CODE_REFERENCE_STEPS_TYPE,
                f"reference/{filename} 이 객체가 아니다",
                task=tid,
                where=where,
            )
        ]
    if "steps" not in doc:
        return [
            issue(
                CODE_EMPTY_REFERENCE_STEPS,
                "reference 는 있는데 steps 키가 없다",
                task=tid,
                where=where,
            )
        ]
    steps = doc.get("steps")
    if steps is None:
        return [
            issue(
                CODE_EMPTY_REFERENCE_STEPS,
                "reference 는 있는데 steps 가 null 이다",
                task=tid,
                where=where,
            )
        ]
    if not isinstance(steps, list):
        return [
            issue(
                CODE_REFERENCE_STEPS_TYPE,
                f"reference.steps 가 배열이 아니다 ({type(steps).__name__})",
                task=tid,
                where=where,
            )
        ]
    if _steps_are_vacuous(steps):
        return [
            issue(
                CODE_EMPTY_REFERENCE_STEPS,
                "reference 는 있는데 steps 가 비었다",
                task=tid,
                where=where,
            )
        ]
    issues: list[dict] = []
    ref_id = doc.get("id")
    if isinstance(tid, str) and tid.strip() and ref_id is not None and ref_id != tid:
        issues.append(
            issue(
                CODE_REFERENCE_ID_MISMATCH,
                f"reference.id({ref_id!r}) 가 과제 id({tid!r}) 와 다르다",
                task=tid,
                where=where,
                extra={"referenceId": ref_id},
            )
        )
    issues.extend(scan_reference_steps(steps, where, tid))
    return issues


def scan_reference_steps(steps: list, where: str, tid: str | None) -> list[dict]:
    issues: list[dict] = []
    for index, step in enumerate(steps):
        prefix = f"{where}#steps[{index}]"
        if not isinstance(step, dict):
            issues.append(
                issue(
                    CODE_REFERENCE_STEP_TYPE,
                    f"steps[{index}] 가 객체가 아니다",
                    task=tid,
                    where=prefix,
                    extra={"index": index},
                )
            )
            continue
        if "run" in step:
            run = step.get("run")
            empty_run = run in (None, "", [], {})
            if isinstance(run, list) and not any(
                isinstance(item, str) and item.strip() for item in run
            ):
                empty_run = True
            if isinstance(run, str) and not run.strip():
                empty_run = True
            if empty_run:
                issues.append(
                    issue(
                        CODE_REFERENCE_RUN_EMPTY,
                        f"steps[{index}].run 이 비었다",
                        task=tid,
                        where=prefix,
                        extra={"index": index},
                    )
                )
        if "answer" in step:
            answer = step.get("answer")
            if not isinstance(answer, dict) or not answer:
                issues.append(
                    issue(
                        CODE_REFERENCE_ANSWER_EMPTY,
                        f"steps[{index}].answer 가 비었다",
                        task=tid,
                        where=prefix,
                        extra={"index": index},
                    )
                )
                continue
            for key, spec in answer.items():
                if not isinstance(spec, dict):
                    continue
                if "cmd" not in spec:
                    continue
                cmd = spec.get("cmd")
                if not isinstance(cmd, list) or not cmd:
                    issues.append(
                        issue(
                            CODE_REFERENCE_CMD_EMPTY,
                            f"steps[{index}].answer.{key}.cmd 가 비었다",
                            task=tid,
                            where=prefix,
                            extra={"index": index, "key": key},
                        )
                    )
    return issues


def scan_hint(task: dict, where: str, tid: str | None) -> list[dict]:
    """힌트가 답을 직접 적어 두면 과제가 측정하지 않는다."""
    text = task.get("instructions")
    if not isinstance(text, str) or not text.strip():
        return []
    body, hint = split_hint(text)
    if hint is None:
        return []
    issues: list[dict] = []
    stripped = hint.strip()
    if not stripped:
        return issues

    if BARE_ANSWER_RE.match(stripped) or SPOILER_RE.search(stripped):
        issues.append(
            issue(
                CODE_HINT_SPOILER,
                "힌트가 정답을 직접 적고 있다",
                task=tid,
                where=where,
                extra={"hint": stripped[:80]},
            )
        )

    for fragment in extract_json_fragments(hint):
        if fragment_looks_like_answer(fragment):
            issues.append(
                issue(
                    CODE_HINT_ANSWER_DUMP,
                    f"힌트에 정답 JSON 이 들어 있다: {json.dumps(fragment, ensure_ascii=False)[:80]}",
                    task=tid,
                    where=where,
                )
            )
            break

    literals: list[str] = []
    for check in task.get("checks") or []:
        if isinstance(check, dict):
            literals.extend(check_expected_literals(check))
    seen = set()
    for token in literals:
        if token in seen:
            continue
        seen.add(token)
        # 본문이 이미 시킨 값(채울 문구)을 힌트 CLI 예시에 반복하는 것은 유출이 아니다.
        if token_appears_bare(hint, token) and token not in body:
            issues.append(
                issue(
                    CODE_HINT_EMBEDS_VALUE,
                    f"힌트가 검사 기대값 {token!r} 을 그대로 담고 있다",
                    task=tid,
                    where=where,
                    extra={"value": token},
                )
            )
    return issues


def scan_task(
    task: dict,
    filename: str,
    ref_path: str | None,
    min_chars: int,
    pack_axis: str | None = None,
    min_title: int = MIN_TITLE_CHARS,
) -> list[dict]:
    where = f"tasks/{filename}"
    tid = task.get("id") if isinstance(task.get("id"), str) else None
    issues: list[dict] = []
    issues.extend(scan_identity(task, filename, where, min_title=min_title))
    issues.extend(scan_tier(task, where, tid))
    issues.extend(scan_input(task, where, tid))
    issues.extend(scan_instructions(task, where, tid, min_chars))
    issues.extend(scan_checks(task, where, tid, pack_axis=pack_axis))
    issues.extend(scan_submit(task, where, tid))
    issues.extend(scan_hint(task, where, tid))
    if ref_path:
        issues.extend(scan_reference(ref_path, filename, tid))
    return issues


def scan_manifest(manifest: dict, pack_id: str, pack_dir: str) -> list[dict]:
    """pack.json 신원 — audit/schema 와 같은 최소 계약, 이 봉투 코드로."""
    issues: list[dict] = []
    where = "pack.json"
    kind = manifest.get("kind")
    if kind != PACK_KIND:
        issues.append(
            issue(
                CODE_PACK_KIND,
                f"kind 가 {PACK_KIND} 가 아니다: {kind!r}",
                where=where,
                extra={"kind": kind},
            )
        )
    version = manifest.get("schemaVersion")
    if version != PACK_SCHEMA_VERSION:
        issues.append(
            issue(
                CODE_PACK_SCHEMA_VERSION,
                f"schemaVersion 이 {PACK_SCHEMA_VERSION} 이 아니다: {version!r}",
                where=where,
                extra={"schemaVersion": version},
            )
        )
    declared = manifest.get("id")
    folder = os.path.basename(os.path.normpath(pack_dir))
    if declared != pack_id or declared != folder:
        issues.append(
            issue(
                CODE_PACK_ID_MISMATCH,
                f"pack.id({declared!r}) 가 폴더({folder}) 와 다르다",
                where=where,
                extra={"id": declared, "folder": folder},
            )
        )
    title = manifest.get("title")
    if not isinstance(title, str) or not title.strip():
        issues.append(issue(CODE_PACK_EMPTY_TITLE, "pack.title 이 비었다", where=where))
    elif has_edge_whitespace(title):
        issues.append(
            issue(
                CODE_PACK_TITLE_WHITESPACE,
                f"pack.title 앞뒤에 공백이 있다: {title!r}",
                where=where,
            )
        )
    axis = manifest.get("axis")
    if not isinstance(axis, str) or not axis.strip():
        issues.append(issue(CODE_PACK_EMPTY_AXIS, "pack.axis 가 비었다", where=where))
    elif has_edge_whitespace(axis):
        issues.append(
            issue(
                CODE_PACK_AXIS_WHITESPACE,
                f"pack.axis 앞뒤에 공백이 있다: {axis!r}",
                where=where,
            )
        )
    requires = manifest.get("requires")
    commands = requires.get("commands") if isinstance(requires, dict) else None
    if not isinstance(requires, dict) or "commands" not in requires:
        issues.append(
            issue(CODE_PACK_MISSING_REQUIRES, "requires.commands 가 없다", where=where)
        )
    elif not isinstance(commands, list) or not commands:
        issues.append(
            issue(CODE_PACK_EMPTY_COMMANDS, "requires.commands 가 비었다", where=where)
        )
    elif any(not isinstance(item, str) or not item.strip() for item in commands):
        issues.append(
            issue(
                CODE_PACK_COMMAND_TYPE,
                "requires.commands 항목이 빈 값이거나 문자열이 아니다",
                where=where,
            )
        )
    runner = manifest.get("runner")
    if not isinstance(runner, dict) or not runner:
        issues.append(issue(CODE_PACK_MISSING_RUNNER, "runner 가 없다", where=where))
    else:
        missing = [key for key in RUNNER_KEYS if not runner.get(key)]
        if missing:
            issues.append(
                issue(
                    CODE_PACK_MISSING_RUNNER_FIELD,
                    f"runner 필드가 비었다: {', '.join(missing)}",
                    where=where,
                    extra={"missing": missing},
                )
            )
    return issues


def scan_orphan_references(tasks_dir: str, ref_dir: str) -> list[dict]:
    """짝 과제가 없는 reference/*.json — audit 과 같은 계약, 코드만 이 봉투."""
    if not os.path.isdir(ref_dir):
        return []
    task_names = set(list_json_names(tasks_dir)) if os.path.isdir(tasks_dir) else set()
    issues: list[dict] = []
    for name in list_json_names(ref_dir):
        if name not in task_names:
            issues.append(
                issue(
                    CODE_ORPHAN_REFERENCE,
                    f"고아 기준풀이 reference/{name} — 짝 과제(tasks/{name})가 없다",
                    where=f"reference/{name}",
                )
            )
    return issues


def scan_pack(
    pack_id: str,
    pack_dir: str,
    min_chars: int = MIN_INSTRUCTION_CHARS,
    min_title: int = MIN_TITLE_CHARS,
) -> dict:
    """pack 하나. 반환: {id, taskCount, issueCount, issues}."""
    issues: list[dict] = []
    manifest_path = os.path.join(pack_dir, "pack.json")
    if not os.path.isfile(manifest_path):
        issues.append(
            issue(CODE_MISSING_PACK, "pack.json 이 없다", where="pack.json")
        )
        return _pack_report(pack_id, 0, issues)

    manifest, err = _load_json(manifest_path)
    if err:
        issues.append(
            issue(CODE_PARSE_ERROR, f"pack.json {err}", where="pack.json")
        )
        return _pack_report(pack_id, 0, issues)
    if not isinstance(manifest, dict):
        issues.append(
            issue(
                CODE_PACK_TYPE,
                f"pack.json 이 객체가 아니다 ({type(manifest).__name__})",
                where="pack.json",
            )
        )
        return _pack_report(pack_id, 0, issues)
    issues.extend(scan_manifest(manifest, pack_id, pack_dir))
    pack_axis = manifest.get("axis") if isinstance(manifest.get("axis"), str) else None

    tasks_dir = os.path.join(pack_dir, "tasks")
    ref_dir = os.path.join(pack_dir, "reference")
    if not os.path.isdir(tasks_dir):
        issues.append(
            issue(CODE_MISSING_TASKS, "tasks/ 가 없다", where="tasks")
        )
        return _pack_report(pack_id, 0, issues)

    task_files = list_json_names(tasks_dir)
    if not task_files:
        issues.append(
            issue(
                CODE_EMPTY_PACK,
                "tasks/ 에 과제가 없다",
                where="tasks",
                severity=SEVERITY_WARNING,
            )
        )
    id_owners: dict[str, list[str]] = {}
    task_count = 0
    for name in task_files:
        path = os.path.join(tasks_dir, name)
        task, err = _load_json(path)
        if err:
            issues.append(
                issue(
                    CODE_PARSE_ERROR,
                    f"tasks/{name} {err}",
                    where=f"tasks/{name}",
                )
            )
            continue
        if not isinstance(task, dict):
            issues.append(
                issue(
                    CODE_PARSE_ERROR,
                    f"tasks/{name} 이 객체가 아니다",
                    where=f"tasks/{name}",
                )
            )
            continue
        task_count += 1
        tid = task.get("id")
        if isinstance(tid, str) and tid.strip():
            id_owners.setdefault(tid.strip(), []).append(name)
        ref_path = os.path.join(ref_dir, name)
        if not os.path.isfile(ref_path):
            ref_path = None
        issues.extend(
            scan_task(
                task,
                name,
                ref_path,
                min_chars,
                pack_axis=pack_axis,
                min_title=min_title,
            )
        )

    issues.extend(scan_orphan_references(tasks_dir, ref_dir))
    for tid, owners in sorted(id_owners.items()):
        if len(owners) > 1:
            issues.append(
                issue(
                    CODE_DUPLICATE_TASK_ID,
                    f"과제 id {tid!r} 가 이 pack 에서 {', '.join(owners)} 에 중복이다",
                    task=tid,
                    where="tasks",
                    extra={"files": list(owners)},
                )
            )
    return _pack_report(pack_id, task_count, issues)


def _pack_report(pack_id: str, task_count: int, issues: list[dict]) -> dict:
    issues_sorted = sorted(
        issues,
        key=lambda row: (
            row.get("where") or "",
            row.get("code") or "",
            row.get("message") or "",
        ),
    )
    return {
        "id": pack_id,
        "taskCount": task_count,
        "issueCount": len(issues_sorted),
        "issues": issues_sorted,
    }


def _filter_pack_issues(pack: dict, exclude: set[str]) -> dict:
    kept = [item for item in pack.get("issues") or [] if item.get("code") not in exclude]
    clone = dict(pack)
    clone["issues"] = kept
    clone["issueCount"] = len(kept)
    return clone


def audit(
    packs_root: str,
    pack_ids: list[str] | None = None,
    min_chars: int = MIN_INSTRUCTION_CHARS,
    min_title: int = MIN_TITLE_CHARS,
    exclude_codes: list[str] | None = None,
) -> dict:
    """전 pack(또는 지정 pack) 건강 감사. 순수 파일 검사.

    packs_root: `packs/` 를 담은 디렉토리(보통 gym/).
    """
    packs_dir = os.path.join(packs_root, "packs")
    pack_rows: list[dict] = []
    scan_error = None
    if not os.path.isdir(packs_dir):
        scan_error = f"packs/ 가 없다: {packs_dir}"
    else:
        found = iter_pack_dirs(packs_root, pack_ids)
        if pack_ids:
            have = {row[0] for row in found}
            for missing in pack_ids:
                if missing not in have:
                    pack_rows.append(
                        _pack_report(
                            missing,
                            0,
                            [
                                issue(
                                    CODE_MISSING_PACK,
                                    f"지정한 pack 이 없다: {missing}",
                                    where="packs",
                                )
                            ],
                        )
                    )
        for pack_id, pack_dir in found:
            pack_rows.append(
                scan_pack(
                    pack_id,
                    pack_dir,
                    min_chars=min_chars,
                    min_title=min_title,
                )
            )

    exclude = set(exclude_codes or [])
    if exclude:
        pack_rows = [_filter_pack_issues(pack, exclude) for pack in pack_rows]

    pack_rows.sort(key=lambda row: row["id"])
    all_issues = [item for pack in pack_rows for item in pack["issues"]]
    error_count = sum(1 for item in all_issues if item.get("severity") == SEVERITY_ERROR)
    warning_count = sum(1 for item in all_issues if item.get("severity") != SEVERITY_ERROR)
    codes: dict[str, int] = {}
    for item in all_issues:
        codes[item["code"]] = codes.get(item["code"], 0) + 1
    task_count = sum(pack["taskCount"] for pack in pack_rows)
    issue_count = len(all_issues)
    report = {
        "kind": KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": scan_error is None and issue_count == 0,
        "packCount": len(pack_rows),
        "taskCount": task_count,
        "issueCount": issue_count,
        "errorCount": error_count,
        "warningCount": warning_count,
        "codes": dict(sorted(codes.items())),
        "packs": pack_rows,
    }
    if scan_error:
        report["scanError"] = scan_error
    if exclude:
        report["excludedCodes"] = sorted(exclude)
    return report


def render_report(report: dict) -> str:
    pack_n = report.get("packCount", 0)
    task_n = report.get("taskCount", 0)
    issue_n = report.get("issueCount", 0)
    if report.get("scanError"):
        head = f"gym pack 건강: 스캔 실패 — {report['scanError']}"
        return head
    if report.get("ok"):
        return f"gym pack 건강: {pack_n} pack · {task_n} 과제 — 이슈 0"
    err_n = report.get("errorCount", 0)
    warn_n = report.get("warningCount", 0)
    lines = [
        f"gym pack 건강: {pack_n} pack · {task_n} 과제 · 이슈 {issue_n}건 "
        f"(error {err_n} · warning {warn_n})"
    ]
    for pack in report.get("packs") or []:
        for item in pack.get("issues") or []:
            task = item.get("task")
            loc = f"{pack['id']}/{task}" if task else pack["id"]
            sev = item.get("severity") or SEVERITY_ERROR
            lines.append(f"  [{loc}] {item.get('code')}: {item.get('message')} ({sev})")
    codes = report.get("codes") or {}
    if codes:
        joined = ", ".join(f"{code}={count}" for code, count in codes.items())
        lines.append(f"  코드 집계: {joined}")
    return "\n".join(lines)


def exit_status(report: dict, strict: bool) -> int:
    """기본 0. --strict 이거나 packs/ 자체를 못 읽으면 이슈 시 1."""
    if report.get("scanError"):
        return 1
    if strict and not report.get("ok"):
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(
        description="gym pack 건강 감사 (지시·검사 이름·힌트·제출 형식)"
    )
    ap.add_argument("--json", action="store_true", help="JSON 봉투로 출력")
    ap.add_argument(
        "--strict",
        action="store_true",
        help="이슈가 있으면 종료 코드 1 (기본은 리포트만 내고 0)",
    )
    ap.add_argument(
        "--pack",
        action="append",
        default=None,
        dest="packs",
        help="검사할 pack id (여러 번 지정 가능)",
    )
    ap.add_argument(
        "--root",
        default=GYM_ROOT,
        help="gym/ 루트 (packs/ 를 담은 디렉토리). 기본은 이 파일 기준 gym/",
    )
    ap.add_argument(
        "--min-instructions",
        type=int,
        default=MIN_INSTRUCTION_CHARS,
        dest="min_instructions",
        help=f"instructions 최소 글자(기본 {MIN_INSTRUCTION_CHARS})",
    )
    ap.add_argument(
        "--min-title",
        type=int,
        default=MIN_TITLE_CHARS,
        dest="min_title",
        help=f"과제 title 최소 글자(기본 {MIN_TITLE_CHARS})",
    )
    ap.add_argument(
        "--codes",
        action="store_true",
        dest="list_codes",
        help="이슈 코드 목록만 출력하고 종료한다",
    )
    ap.add_argument(
        "--exclude",
        action="append",
        default=None,
        dest="exclude_codes",
        help="집계에서 뺄 이슈 코드 (여러 번 지정 가능)",
    )
    return ap


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.list_codes:
        sys.stdout.write(render_codes() + "\n")
        return 0
    min_chars = args.min_instructions
    if min_chars < 1:
        sys.stderr.write("--min-instructions 는 1 이상이어야 한다\n")
        return 2
    min_title = args.min_title
    if min_title < 1:
        sys.stderr.write("--min-title 은 1 이상이어야 한다\n")
        return 2
    report = audit(
        args.root,
        pack_ids=args.packs,
        min_chars=min_chars,
        min_title=min_title,
        exclude_codes=args.exclude_codes,
    )
    if args.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    else:
        sys.stdout.write(render_report(report) + "\n")
    return exit_status(report, args.strict)


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
