"""gym pack 건강 감사 — 지시문·검사 이름·힌트·제출 형식의 품질 게이트.

## 왜 이 도구인가 (audit 가 못 보는 층)

`audit.py` 는 스키마·기준풀이 짝·과제 ID 전역 고유만 본다. 지시문이 비거나 한
줄짜리이고, 같은 과제 안에서 `check.name` 이 비거나 겹치고, 힌트가 답을 그대로
적어 두고, `submit.kind` 가 모르는 값이어도 그 감사는 통과한다. 운동장 품질이
조용히 내려간다.

이 도구는 그 다음 층이다. 바이너리·네트워크 없이 pack JSON 만 읽어:

- 빈/짧은 `instructions` (기본 20글자 미만)
- 과제 안 `check.name` 누락·중복
- 과제 `id`/`title` 의 앞뒤·내부 공백
- 기준풀이 파일은 있는데 `steps` 가 비었다
- 모르는 `submit.kind`
- 힌트가 답을 직접 노출하는지 (정답 JSON 덤프, `답은 N`, 검사 기대값 복붙)

을 pack 별로 보고한다. 기존 pack 을 고치지 않는다 — 도구만 추가한다.

## 종료 코드 (CI 자기시험과 게이트를 가른다)

현재 트리가 깨끗하지 않을 수 있고, 도구 자체를 CI 에서 돌릴 때는 리포트만
필요할 수 있다. 그래서:

    python gym/tools/pack_health.py            # 리포트, 이슈가 있어도 0
    python gym/tools/pack_health.py --json     # 같은 리포트 JSON, 기본 0
    python gym/tools/pack_health.py --strict   # 이슈가 있으면 1
    python gym/tools/pack_health.py --pack id  # 한 pack 만
    python gym/tools/pack_health.py --root DIR # gym/ 루트 (packs/ 를 담은 곳)

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


def split_hint(instructions: str) -> tuple[str, str | None]:
    """본문과 힌트 꼬리를 가른다. 마커가 없으면 힌트는 None."""
    if not isinstance(instructions, str):
        return "", None
    found_at = -1
    found_marker = ""
    for marker in HINT_MARKERS:
        idx = instructions.find(marker)
        if idx >= 0 and (found_at < 0 or idx < found_at):
            found_at = idx
            found_marker = marker
    if found_at < 0:
        return instructions, None
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
    return issues


def scan_identity(task: dict, filename: str, where: str) -> list[dict]:
    """id/title 공백·공란·파일명 불일치."""
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
    return issues


def scan_checks(task: dict, where: str, tid: str | None) -> list[dict]:
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
        names.append(raw_name)

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
    return issues


def _steps_are_vacuous(steps: list) -> bool:
    if not steps:
        return True
    for step in steps:
        if not isinstance(step, dict):
            continue
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
    return []


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
) -> list[dict]:
    where = f"tasks/{filename}"
    tid = task.get("id") if isinstance(task.get("id"), str) else None
    issues: list[dict] = []
    issues.extend(scan_identity(task, filename, where))
    issues.extend(scan_instructions(task, where, tid, min_chars))
    issues.extend(scan_checks(task, where, tid))
    issues.extend(scan_submit(task, where, tid))
    issues.extend(scan_hint(task, where, tid))
    if ref_path:
        issues.extend(scan_reference(ref_path, filename, tid))
    return issues


def scan_pack(
    pack_id: str,
    pack_dir: str,
    min_chars: int = MIN_INSTRUCTION_CHARS,
) -> dict:
    """pack 하나. 반환: {id, taskCount, issueCount, issues}."""
    issues: list[dict] = []
    manifest_path = os.path.join(pack_dir, "pack.json")
    if not os.path.isfile(manifest_path):
        issues.append(
            issue(CODE_MISSING_PACK, "pack.json 이 없다", where="pack.json")
        )
        return _pack_report(pack_id, 0, issues)

    _manifest, err = _load_json(manifest_path)
    if err:
        issues.append(
            issue(CODE_PARSE_ERROR, f"pack.json {err}", where="pack.json")
        )
        return _pack_report(pack_id, 0, issues)

    tasks_dir = os.path.join(pack_dir, "tasks")
    ref_dir = os.path.join(pack_dir, "reference")
    if not os.path.isdir(tasks_dir):
        issues.append(
            issue(CODE_MISSING_TASKS, "tasks/ 가 없다", where="tasks")
        )
        return _pack_report(pack_id, 0, issues)

    task_files = list_json_names(tasks_dir)
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
        issues.extend(scan_task(task, name, ref_path, min_chars))

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


def audit(
    packs_root: str,
    pack_ids: list[str] | None = None,
    min_chars: int = MIN_INSTRUCTION_CHARS,
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
            pack_rows.append(scan_pack(pack_id, pack_dir, min_chars=min_chars))

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
    lines = [f"gym pack 건강: {pack_n} pack · {task_n} 과제 · 이슈 {issue_n}건"]
    for pack in report.get("packs") or []:
        for item in pack.get("issues") or []:
            task = item.get("task")
            loc = f"{pack['id']}/{task}" if task else pack["id"]
            lines.append(f"  [{loc}] {item.get('code')}: {item.get('message')}")
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
    return ap


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    min_chars = args.min_instructions
    if min_chars < 1:
        sys.stderr.write("--min-instructions 는 1 이상이어야 한다\n")
        return 2
    report = audit(args.root, pack_ids=args.packs, min_chars=min_chars)
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
