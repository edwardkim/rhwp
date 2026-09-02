"""[#4653] 기준 풀이 왕복 — 베이스라인 제출물을 기계적으로 생성한다.

## 왜 필요한가

과제를 손으로 늘리면 "돌아가지 않는 과제" 가 섞인다. pack 이 8개로 늘어나는
순간 그 위험은 8배가 된다. 그래서 pack 마다 `reference/<과제ID>.json` 에
**기준 풀이**를 두고, 이 스크립트가 그것을 실행해 제출물을 만든 뒤 곧바로
채점한다. 신규 과제는 이 왕복을 통과해야만 등재된다 — 즉 **저장소에 들어간
모든 과제는 풀 수 있음이 실측된 과제**다.

기준 풀이는 정답 노출이므로 `reference/` 로 분리해 명시한다(기존
`baselines/*/answer.json` 과 같은 성격이다). 과제를 푸는 에이전트는 이 폴더를
보지 않는 것이 규칙이고, 보더라도 측정되는 것은 "스스로 경로를 찾는 능력"이
아니게 될 뿐 채점은 정직하게 돌아간다.

## 기준 풀이 형식

```json
{
  "id": "TE01",
  "steps": [
    { "run": ["edit", "replace-text", "{input}", "--find", "규제",
              "--replace", "점검", "-o", "{sub:edited.hwp}", "--json"] },
    { "answer": { "remaining": { "cmd": ["search", "{sub:edited.hwp}", "--json", "--", "규제"],
                                 "path": "matchCount" } } }
  ]
}
```

- `run` — rhwp 를 실행한다. `{input}`(과제 입력)·`{sub:이름}`(제출 폴더 안 경로)
  자리표를 쓴다. `allowExits` 로 판정성 종료 코드를 허용한다.
- `answer` — 봉투에서 값을 길어 `answer.json` 에 합친다(라이브 재계산).
- `copy` — 과제 입력이나 자산을 제출 폴더로 복사한다.
- `write_json` — 부속 JSON(계획·정책)을 제출 폴더에 쓴다. 본문의 `{input}`
  · `{sub:}` 도 치환한다.
- `keyring_from` — 발급한 키의 공개키로 키링을 조립한다.

자리표 규약(#4664, #5273):

- 한 문자열의 `{sub:}` 는 **전부** 바꾼다. 첫 하나만 바꾸면 다세대
  계획서의 나머지 자리표가 리터럴로 남아 엉뚱한 파일이 생긴다.
- `{input}` 이 문자열 안에 박혀 있어도 바꾼다. 토큰 전체가 `{input}` 일
  때만 바꾸던 자리는 계획서 JSON 에서 입력을 놓친다.
- `{sub:}` 이름은 제출 폴더 안의 상대경로만 허용한다. 부모(`..`)·절대·
  드라이브·UNC·홈은 거부한다.
- 닫히지 않은 `{sub:` 는 그 과제만 실패로 접는다. `ValueError` 로
  전 왕복을 죽이지 않는다.

조립 뒤 두 자리(#5273):

- **부재 산출** — `submit.files` 가 선언한 파일이 제출 폴더에 없으면
  채점 전에 `missing-artifact` 로 보고한다. 생성 성공만으로 통과시키지
  않는다.
- **실패 보고** — 채점 봉투의 `pass` 가 참이 아니면 과제 ID 와 이유
  (`error` 또는 실패한 검사 이름)를 남긴다. 비-dict·키 없음은 통과로
  접지 않는다.

사용:
  python gym/tools/build_baseline.py --agent claude-fable-5 [--pack <id>] [--bin <경로>]

새 CLI 플래그·새 pack 은 없다. 시험은
`scripts/tests/test_gym_build_baseline.py` 와
`scripts/tests/test_gym_packs.py` 의 `BaselineResolveTests` 가 바이너리
없이 고정한다. 규약 문서는 `gym/docs/build_baseline.md`.
"""

from __future__ import annotations

import argparse
import io
import json
import os
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402
from gym.core.checks import dig  # noqa: E402

ROOT = runner.ROOT
PACKS_DIR = runner.PACKS_DIR

PLACEHOLDER_INPUT = "{input}"
PLACEHOLDER_SUB_PREFIX = "{sub:"
PLACEHOLDER_SUB_CLOSE = "}"

STEP_KINDS = ("run", "copy", "write_json", "keyring_from", "answer")
KNOWN_STEP_KINDS = frozenset(STEP_KINDS)

TOKEN_KINDS = (
    "literal",
    "exact-input",
    "exact-sub",
    "embedded-sub",
    "embedded-input",
    "mixed",
    "unclosed-sub",
    "not-str",
)

FAILURE_KINDS = (
    "ok",
    "missing-artifact",
    "failed-score",
    "build-error",
    "missing-reference",
    "malformed-reference",
    "unknown-step",
    "unsafe-sub",
    "unclosed-placeholder",
    "empty-steps",
)

UNSAFE_REL_REASONS = (
    "empty",
    "not-str",
    "absolute",
    "drive",
    "parent",
    "unc",
    "home",
)

REFERENCE_LABELS = (
    "ok",
    "empty-steps",
    "malformed-reference",
    "unknown-step",
)

COUNT_KEYS = (
    "built",
    "failed",
    "skipped",
    "missingArtifact",
    "failedScore",
    "buildError",
)

DEFAULT_ALLOW_EXITS = (0,)
ERROR_HEAD_LIMIT = 300
ANSWER_FILENAME = "answer.json"

FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

CATCHABLE_EXCEPTIONS = (
    RuntimeError,
    OSError,
    KeyError,
    IndexError,
    TypeError,
    ValueError,
    json.JSONDecodeError,
)

CLI_FLAGS = ("--agent", "--pack", "--bin")


def is_fatal_exception(exc) -> bool:
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def is_str(value) -> bool:
    return isinstance(value, str)


def as_text(value) -> str | None:
    if not is_str(value):
        return None
    return value


def placeholder_input() -> str:
    return PLACEHOLDER_INPUT


def placeholder_sub(name: str) -> str:
    return f"{PLACEHOLDER_SUB_PREFIX}{name}{PLACEHOLDER_SUB_CLOSE}"


def is_exact_input(token) -> bool:
    return token == PLACEHOLDER_INPUT


def is_exact_sub(token) -> bool:
    if not is_str(token):
        return False
    return token.startswith(PLACEHOLDER_SUB_PREFIX) and token.endswith(
        PLACEHOLDER_SUB_CLOSE) and token.count(PLACEHOLDER_SUB_PREFIX) == 1


def has_sub_placeholder(token) -> bool:
    return is_str(token) and PLACEHOLDER_SUB_PREFIX in token


def has_input_placeholder(token) -> bool:
    return is_str(token) and PLACEHOLDER_INPUT in token


def has_unclosed_sub(token) -> bool:
    """닫히지 않은 `{sub:` 가 남아 있는가. 순수."""
    if not is_str(token):
        return False
    rest = token
    while PLACEHOLDER_SUB_PREFIX in rest:
        _head, rest = rest.split(PLACEHOLDER_SUB_PREFIX, 1)
        if PLACEHOLDER_SUB_CLOSE not in rest:
            return True
        _name, rest = rest.split(PLACEHOLDER_SUB_CLOSE, 1)
    return False


def classify_token(token) -> str:
    """한 토큰의 자리표 종류. 문서·시험이 같은 표를 본다."""
    if not is_str(token):
        return "not-str"
    if has_unclosed_sub(token):
        return "unclosed-sub"
    if is_exact_input(token):
        return "exact-input"
    if is_exact_sub(token):
        return "exact-sub"
    has_sub = has_sub_placeholder(token)
    has_input = has_input_placeholder(token) and not is_exact_input(token)
    if has_sub and has_input:
        return "mixed"
    if has_sub:
        return "embedded-sub"
    if has_input:
        return "embedded-input"
    return "literal"


def iter_sub_placeholders(token):
    """문자열 안의 `{sub:이름}` 을 (이름, 시작, 끝) 으로 낸다. 닫히지 않으면 중단."""
    if not is_str(token):
        return
    index = 0
    while True:
        start = token.find(PLACEHOLDER_SUB_PREFIX, index)
        if start < 0:
            return
        close = token.find(PLACEHOLDER_SUB_CLOSE, start + len(PLACEHOLDER_SUB_PREFIX))
        if close < 0:
            return
        name = token[start + len(PLACEHOLDER_SUB_PREFIX):close]
        yield name, start, close + 1
        index = close + 1


def extract_sub_names(token) -> list[str]:
    """등장 순서를 유지한 `{sub:}` 이름. 중복도 남긴다."""
    return [name for name, _start, _end in iter_sub_placeholders(token)]


def unique_sub_names(token) -> list[str]:
    """등장 순서를 유지하되 중복은 한 번만."""
    out = []
    seen = set()
    for name in extract_sub_names(token):
        if name in seen:
            continue
        seen.add(name)
        out.append(name)
    return out


def extract_placeholders(token) -> list[dict]:
    """토큰 안의 자리표를 종류·이름과 함께 나열한다. 순수."""
    if not is_str(token):
        return []
    found = []
    index = 0
    while index < len(token):
        sub_at = token.find(PLACEHOLDER_SUB_PREFIX, index)
        input_at = token.find(PLACEHOLDER_INPUT, index)
        if sub_at < 0 and input_at < 0:
            break
        if input_at >= 0 and (sub_at < 0 or input_at < sub_at):
            found.append({"kind": "input", "name": "", "start": input_at,
                          "end": input_at + len(PLACEHOLDER_INPUT)})
            index = input_at + len(PLACEHOLDER_INPUT)
            continue
        close = token.find(PLACEHOLDER_SUB_CLOSE, sub_at + len(PLACEHOLDER_SUB_PREFIX))
        if close < 0:
            found.append({"kind": "unclosed-sub", "name": token[sub_at + len(PLACEHOLDER_SUB_PREFIX):],
                          "start": sub_at, "end": len(token)})
            break
        found.append({"kind": "sub",
                      "name": token[sub_at + len(PLACEHOLDER_SUB_PREFIX):close],
                      "start": sub_at, "end": close + 1})
        index = close + 1
    return found


def count_sub_placeholders(token) -> int:
    return len(extract_sub_names(token))


def count_input_placeholders(token) -> int:
    if not is_str(token):
        return 0
    return token.count(PLACEHOLDER_INPUT)


def remaining_placeholders(text) -> list[str]:
    """치환 뒤에 남은 자리표 조각. 비어 있어야 정상."""
    if not is_str(text):
        return []
    leftover = []
    if PLACEHOLDER_SUB_PREFIX in text:
        leftover.append(PLACEHOLDER_SUB_PREFIX)
    if PLACEHOLDER_INPUT in text:
        leftover.append(PLACEHOLDER_INPUT)
    return leftover


def has_unresolved_placeholder(text) -> bool:
    return bool(remaining_placeholders(text))


def _as_rel_text(rel) -> str | None:
    if not is_str(rel):
        return None
    text = rel.strip()
    return text if text else None


def normalize_rel(rel) -> str | None:
    """제출 상대경로를 `/` 로 정규화한다. 불안전하면 None.

    절대경로·드라이브·UNC·홈·부모(`..`)는 제출 폴더 밖으로 쓸 수 있으므로
    버린다. `.` 구간과 빈 구간만 접는다.
    """
    text = _as_rel_text(rel)
    if text is None:
        return None
    unified = text.replace("\\", "/")
    if unified.startswith("/") or unified.startswith("//"):
        return None
    if unified.startswith("~"):
        return None
    head = unified.split("/", 1)[0]
    if ":" in head:
        return None
    parts = []
    for part in unified.split("/"):
        if part in ("", "."):
            continue
        if part == "..":
            return None
        parts.append(part)
    if not parts:
        return None
    return "/".join(parts)


def unsafe_rel_reason(rel) -> str | None:
    """정규화가 거절한 이유를 카탈로그 단어로. 안전하면 None."""
    if not is_str(rel):
        return "not-str"
    text = rel.strip()
    if not text:
        return "empty"
    unified = text.replace("\\", "/")
    if unified.startswith("//") or unified.startswith("\\\\"):
        return "unc"
    if unified.startswith("/"):
        return "absolute"
    if unified.startswith("~"):
        return "home"
    head = unified.split("/", 1)[0]
    if ":" in head:
        return "drive"
    for part in unified.split("/"):
        if part == "..":
            return "parent"
    if normalize_rel(rel) is None:
        return "empty"
    return None


def is_safe_rel(rel) -> bool:
    return normalize_rel(rel) is not None


def is_safe_sub_name(name) -> bool:
    return is_safe_rel(name)


def require_safe_sub_name(name) -> str:
    """안전하면 정규화된 이름, 아니면 RuntimeError."""
    reason = unsafe_rel_reason(name)
    if reason is not None:
        raise RuntimeError(f"불안전 제출 경로 ({reason}): {name!r}")
    safe = normalize_rel(name)
    if safe is None:
        raise RuntimeError(f"불안전 제출 경로 (empty): {name!r}")
    return safe


def join_sub_path(sub_dir, name, *, mkdir=False) -> str:
    """정규화된 상대경로를 제출 폴더 아래에 붙인다."""
    safe = require_safe_sub_name(name)
    path = os.path.join(sub_dir, *safe.split("/"))
    if mkdir:
        ensure_parent_dir(path, sub_dir)
    return path


def escape_json_path(path: str) -> str:
    """계획서 JSON 문자열 안에 넣을 때 백슬래시를 두 번 쓴다."""
    return path.replace("\\", "\\\\")


def ensure_parent_dir(path, fallback=None) -> str:
    """산출 파일의 부모 폴더를 만든다. 부모가 없으면 fallback(보통 제출 폴더)."""
    parent = os.path.dirname(path)
    if not parent:
        parent = fallback or "."
    os.makedirs(parent, exist_ok=True)
    return parent


def task_input(task) -> str:
    if not isinstance(task, dict):
        raise RuntimeError("과제가 객체가 아니다")
    value = task.get("input")
    if not is_str(value) or not value:
        raise RuntimeError("과제 input 이 없다")
    return value


def task_id_of(task) -> str:
    if not isinstance(task, dict):
        return "?"
    value = task.get("id")
    return value if is_str(value) and value else "?"


def resolve_exact_input(task) -> str:
    return task_input(task)


def resolve_exact_sub(name, sub_dir, *, mkdir=True) -> str:
    return join_sub_path(sub_dir, name, mkdir=mkdir)


def replace_embedded_subs(token, sub_dir, *, mkdir=True, escape=True) -> str:
    """한 문자열의 `{sub:}` 를 전부 경로로 바꾼다. 닫히지 않으면 실패."""
    if not is_str(token):
        raise RuntimeError("자리표 토큰이 문자열이 아니다")
    if has_unclosed_sub(token):
        raise RuntimeError(f"닫히지 않은 {{sub:}} 자리표: {token[:80]}")
    out = []
    rest = token
    while PLACEHOLDER_SUB_PREFIX in rest:
        head, rest = rest.split(PLACEHOLDER_SUB_PREFIX, 1)
        if PLACEHOLDER_SUB_CLOSE not in rest:
            raise RuntimeError(f"닫히지 않은 {{sub:}} 자리표: {token[:80]}")
        name, rest = rest.split(PLACEHOLDER_SUB_CLOSE, 1)
        path = join_sub_path(sub_dir, name, mkdir=mkdir)
        out.append(head + (escape_json_path(path) if escape else path))
    out.append(rest)
    return "".join(out)


def replace_embedded_inputs(token, task) -> str:
    """한 문자열의 `{input}` 을 전부 바꾼다. 경로 구분자는 `/`."""
    if not is_str(token):
        raise RuntimeError("자리표 토큰이 문자열이 아니다")
    return token.replace(PLACEHOLDER_INPUT, task_input(task).replace("\\", "/"))


def resolve(token, task, sub_dir):
    """자리표를 제출 경로·입력 경로로 바꾼다.

    기존 계약:
    - 토큰 전체가 `{input}` 이면 과제 입력을 그대로 돌려준다(구분자는 원본).
    - 토큰 전체가 `{sub:이름}` 이면 제출 폴더 아래 경로를 만들고 부모를 만든다.
      이 자리의 백슬래시는 이스케이프하지 않는다.
    - 문자열 안의 `{sub:}` 는 전부 바꾸고, 계획서 JSON 을 위해 백슬래시를
      두 번 쓴다(#4664).

    보강(#5273):
    - 문자열 안의 `{input}` 도 바꾼다(혼합 토큰).
    - 닫히지 않은 `{sub:` · 불안전 이름은 RuntimeError.
    """
    if not is_str(token):
        return token
    kind = classify_token(token)
    if kind == "unclosed-sub":
        raise RuntimeError(f"닫히지 않은 {{sub:}} 자리표: {token[:80]}")
    if kind == "exact-input":
        return resolve_exact_input(task)
    if kind == "exact-sub":
        return resolve_exact_sub(token[len(PLACEHOLDER_SUB_PREFIX):-1], sub_dir, mkdir=True)
    if kind in ("embedded-sub", "mixed"):
        replaced = replace_embedded_subs(token, sub_dir, mkdir=True, escape=True)
        if PLACEHOLDER_INPUT in replaced:
            replaced = replace_embedded_inputs(replaced, task)
        return replaced
    if kind == "embedded-input":
        return replace_embedded_inputs(token, task)
    return token


def resolve_args(args, task, sub_dir) -> list:
    if not isinstance(args, (list, tuple)):
        raise RuntimeError("run 인자가 목록이 아니다")
    return [resolve(arg, task, sub_dir) for arg in args]


def resolve_text(text, task, sub_dir) -> str:
    """JSON 덤프처럼 자리표가 박힌 본문을 치환한다."""
    return resolve(text, task, sub_dir) if is_str(text) else text


def resolve_write_json_body(body, task, sub_dir):
    """write_json 본문의 `{input}` · `{sub:}` 를 바꾸고 객체로 되돌린다."""
    dumped = json.dumps(body)
    dumped = replace_embedded_inputs(dumped, task)
    if has_sub_placeholder(dumped):
        dumped = replace_embedded_subs(dumped, sub_dir, mkdir=True, escape=True)
    return json.loads(dumped)


def resolve_copy_source(spec_from, task, sub_dir) -> str:
    """copy.from 이 상대면 저장소 루트, 이미 절대면 그대로."""
    resolved = resolve(spec_from, task, sub_dir)
    if is_str(resolved) and os.path.isabs(resolved):
        return resolved
    return os.path.join(ROOT, resolved)


def is_step_mapping(step) -> bool:
    return isinstance(step, dict)


def step_keys(step) -> list[str]:
    if not is_step_mapping(step):
        return []
    return sorted(str(key) for key in step.keys() if key != "allowExits")


def step_kind(step) -> str | None:
    """한 스텝의 주 키. 알려진 키가 없으면 None."""
    if not is_step_mapping(step):
        return None
    for kind in STEP_KINDS:
        if kind in step:
            return kind
    return None


def is_known_step(step) -> bool:
    return step_kind(step) in KNOWN_STEP_KINDS


def classify_step(step) -> str:
    if not is_step_mapping(step):
        return "not-mapping"
    kind = step_kind(step)
    return kind if kind is not None else "unknown"


def normalize_steps(raw) -> list | None:
    if raw is None:
        return None
    if isinstance(raw, list):
        return raw
    return None


def steps_of_reference(reference) -> list | None:
    if not isinstance(reference, dict):
        return None
    return normalize_steps(reference.get("steps"))


def classify_reference(reference) -> str:
    steps = steps_of_reference(reference)
    if steps is None:
        return "malformed-reference"
    if not steps:
        return "empty-steps"
    for step in steps:
        if not is_known_step(step):
            return "unknown-step"
    return "ok"


def validate_step(step) -> list[str]:
    errors = []
    if not is_step_mapping(step):
        return ["스텝이 객체가 아니다"]
    kind = step_kind(step)
    if kind is None:
        return [f"알 수 없는 기준 풀이 단계 {list(step)}"]
    if kind == "run":
        if not isinstance(step.get("run"), list):
            errors.append("run 이 목록이 아니다")
    elif kind == "copy":
        spec = step.get("copy")
        if not isinstance(spec, dict):
            errors.append("copy 가 객체가 아니다")
        else:
            if "from" not in spec:
                errors.append("copy.from 이 없다")
            if "to" not in spec:
                errors.append("copy.to 가 없다")
    elif kind == "write_json":
        spec = step.get("write_json")
        if not isinstance(spec, dict):
            errors.append("write_json 이 객체가 아니다")
        else:
            if "to" not in spec:
                errors.append("write_json.to 가 없다")
            if "body" not in spec:
                errors.append("write_json.body 가 없다")
    elif kind == "keyring_from":
        spec = step.get("keyring_from")
        if not isinstance(spec, dict):
            errors.append("keyring_from 이 객체가 아니다")
        else:
            for key in ("key", "out", "keyId"):
                if key not in spec:
                    errors.append(f"keyring_from.{key} 가 없다")
    elif kind == "answer":
        spec = step.get("answer")
        if not isinstance(spec, dict):
            errors.append("answer 가 객체가 아니다")
    return errors


def validate_reference(reference) -> list[str]:
    if not isinstance(reference, dict):
        return ["기준 풀이가 객체가 아니다"]
    steps = steps_of_reference(reference)
    if steps is None:
        return ["steps 가 목록이 아니다"]
    if not steps:
        return ["steps 가 비었다"]
    errors = []
    for index, step in enumerate(steps):
        for item in validate_step(step):
            errors.append(f"steps[{index}]: {item}")
    return errors


def walk_strings(value):
    """중첩 구조 안의 문자열을 깊이 우선으로 낸다."""
    if is_str(value):
        yield value
        return
    if isinstance(value, dict):
        for item in value.values():
            yield from walk_strings(item)
        return
    if isinstance(value, (list, tuple)):
        for item in value:
            yield from walk_strings(item)


def collect_sub_names_from_token(token) -> list[str]:
    return extract_sub_names(token)


def collect_sub_names_from_step(step) -> list[str]:
    names = []
    for text in walk_strings(step):
        names.extend(extract_sub_names(text))
    return names


def collect_sub_names(reference) -> list[str]:
    """기준 풀이 전 스텝에서 등장하는 `{sub:}` 이름. 등장 순, 중복 제거."""
    steps = steps_of_reference(reference) or []
    out = []
    seen = set()
    for step in steps:
        for name in collect_sub_names_from_step(step):
            if name in seen:
                continue
            seen.add(name)
            out.append(name)
    return out


def submit_mapping(task) -> dict:
    if not isinstance(task, dict):
        return {}
    submit = task.get("submit")
    return submit if isinstance(submit, dict) else {}


def submit_kind(task) -> str:
    kind = submit_mapping(task).get("kind")
    return kind if is_str(kind) else ""


def submit_files(task) -> list[str]:
    """submit.files 중 상대경로로 쓸 수 있는 것만. 선언 순, 중복 제거."""
    raw = submit_mapping(task).get("files")
    if not isinstance(raw, list):
        return []
    out = []
    seen = set()
    for item in raw:
        safe = normalize_rel(item)
        if safe is None or safe in seen:
            continue
        seen.add(safe)
        out.append(safe)
    return out


def declared_artifacts(task, reference=None) -> list[str]:
    """과제가 요구하는 산출 파일. submit.files 가 있으면 그것만."""
    files = submit_files(task)
    if files:
        return list(files)
    if reference is None:
        return []
    # answer 전용 과제는 산출 파일이 없어도 된다. {sub:} 이름을 요구 목록으로
    # 승격하지 않는다 — 중간 산출(키, 임시 hwp)까지 제출 계약으로 오인한다.
    return []


def expected_artifacts(task, reference=None) -> list[str]:
    return declared_artifacts(task, reference)


def list_submission_files(sub_dir) -> list[str]:
    """제출 폴더 아래 파일의 상대경로(`/`). 없으면 빈 목록."""
    if not is_str(sub_dir) or not os.path.isdir(sub_dir):
        return []
    found = []
    for root, _dirs, files in os.walk(sub_dir):
        for name in files:
            full = os.path.join(root, name)
            rel = os.path.relpath(full, sub_dir).replace("\\", "/")
            found.append(rel)
    found.sort()
    return found


def missing_artifacts(sub_dir, expected) -> list[str]:
    """expected 중 파일이 아닌 것. 순서는 expected 순."""
    if not isinstance(expected, (list, tuple)):
        return []
    missing = []
    for name in expected:
        if not is_str(name) or not name:
            continue
        path = os.path.join(sub_dir, *name.replace("\\", "/").split("/"))
        if not os.path.isfile(path):
            missing.append(name)
    return missing


def artifact_status(sub_dir, expected) -> dict:
    missing = missing_artifacts(sub_dir, expected)
    return {
        "expected": list(expected) if isinstance(expected, (list, tuple)) else [],
        "present": [name for name in (expected or []) if name not in missing],
        "missing": missing,
        "ok": not missing,
    }


def format_task_failure(pack_id, task_id, reason) -> str:
    return f"{pack_id}/{task_id}: {reason}"


def format_missing_artifact(pack_id, task_id, missing) -> str:
    names = ", ".join(missing) if missing else "(없음)"
    return format_task_failure(pack_id, task_id, f"부재 산출: {names}")


def check_built_artifacts(sub_dir, task, reference=None) -> str | None:
    """선언된 산출이 없으면 `부재 산출:` 문구, 있으면 None. pack/task 는 비운다."""
    expected = expected_artifacts(task, reference)
    if not expected:
        return None
    missing = missing_artifacts(sub_dir, expected)
    if not missing:
        return None
    return format_missing_artifact("", task_id_of(task), missing)


def missing_artifact_message(pack_id, task, sub_dir, reference=None) -> str | None:
    expected = expected_artifacts(task, reference)
    if not expected:
        return None
    missing = missing_artifacts(sub_dir, expected)
    if not missing:
        return None
    return format_missing_artifact(pack_id, task_id_of(task), missing)


def score_is_pass(result) -> bool:
    """채점 봉투의 pass 만 본다. 비-dict·키 없음은 실패."""
    if not isinstance(result, dict):
        return False
    return bool(result.get("pass"))


def normalize_score(result) -> dict:
    """채점 결과를 최소 봉투로. 비-dict 는 pass=False + error."""
    if not isinstance(result, dict):
        return {"pass": False, "error": "채점 결과가 dict 가 아니다"}
    out = {"pass": bool(result.get("pass"))}
    if "error" in result:
        out["error"] = result.get("error")
    if "id" in result:
        out["id"] = result.get("id")
    if "checks" in result:
        out["checks"] = result.get("checks")
    return out


def failed_check_lines(checks) -> list[str]:
    """실패한 검사의 `이름: 이유` 목록. 비-dict 칸은 건너뛴다."""
    if not isinstance(checks, list):
        return []
    failed = []
    for check in checks:
        if not isinstance(check, dict):
            continue
        if check.get("ok"):
            continue
        name = check.get("name") or check.get("op") or "검사"
        failed.append(f"{name}: {check.get('error', '판정 불일치')}")
    return failed


def fold_score_result(result) -> dict:
    """채점 봉투를 통과/실패 자리로 접는다. 순수."""
    if score_is_pass(result):
        return {"ok": True, "kind": "ok", "reason": None, "failedChecks": []}
    if not isinstance(result, dict):
        return {"ok": False, "kind": "failed-score",
                "reason": "채점 결과가 dict 가 아니다", "failedChecks": []}
    if result.get("error"):
        return {"ok": False, "kind": "failed-score",
                "reason": result.get("error"), "failedChecks": []}
    failed = failed_check_lines(result.get("checks") or [])
    reason = "; ".join(failed) if failed else "채점 실패"
    return {"ok": False, "kind": "failed-score", "reason": reason, "failedChecks": failed}


def score_failure_message(pack_id, task_id, result) -> str | None:
    """verify_built_task 가 쓰는 한 줄. 통과면 None."""
    folded = fold_score_result(result)
    if folded["ok"]:
        return None
    return format_task_failure(pack_id, task_id, folded["reason"])


def empty_counts() -> dict:
    return {key: 0 for key in COUNT_KEYS}


def bump_count(counts, key, amount=1) -> dict:
    if not isinstance(counts, dict):
        counts = empty_counts()
    counts[key] = int(counts.get(key) or 0) + amount
    return counts


def format_summary(counts) -> str:
    """사람용 한 줄. 기존 왕복 요약 형식을 유지한다."""
    if not isinstance(counts, dict):
        counts = empty_counts()
    built = int(counts.get("built") or 0)
    failed = int(counts.get("failed") or 0)
    skipped = int(counts.get("skipped") or 0)
    return f"기준 풀이 왕복: 성공 {built} · 실패 {failed} · 기준 풀이 없음 {skipped}"


def format_summary_detail(counts) -> str:
    """실패를 부재 산출·채점 실패·조립 오류로 나눈 부가 줄."""
    if not isinstance(counts, dict):
        counts = empty_counts()
    return (
        f"  내역: 부재 산출 {int(counts.get('missingArtifact') or 0)}"
        f" · 채점 실패 {int(counts.get('failedScore') or 0)}"
        f" · 조립 오류 {int(counts.get('buildError') or 0)}"
    )


def summary_exit(counts) -> int:
    if not isinstance(counts, dict):
        return 1
    return 0 if int(counts.get("failed") or 0) == 0 else 1


def allow_exits_of(step) -> list:
    if not is_step_mapping(step):
        return list(DEFAULT_ALLOW_EXITS)
    raw = step.get("allowExits", list(DEFAULT_ALLOW_EXITS))
    if not isinstance(raw, (list, tuple)):
        return list(DEFAULT_ALLOW_EXITS)
    return list(raw)


def write_json_file(path, body) -> None:
    payload = json.dumps(body, ensure_ascii=False, indent=2) + "\n"
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(payload)


def write_answer_file(sub_dir, answer) -> str | None:
    if not answer:
        return None
    path = os.path.join(sub_dir, ANSWER_FILENAME)
    write_json_file(path, answer)
    return path


def apply_copy_step(step, task, sub_dir) -> str:
    spec = step["copy"]
    src = resolve_copy_source(spec["from"], task, sub_dir)
    dest = resolve(spec["to"], task, sub_dir)
    ensure_parent_dir(dest, sub_dir)
    shutil.copyfile(src, dest)
    return dest


def apply_write_json_step(step, task, sub_dir) -> str:
    spec = step["write_json"]
    path = resolve(spec["to"], task, sub_dir)
    body = resolve_write_json_body(spec["body"], task, sub_dir)
    write_json_file(path, body)
    return path


def apply_keyring_step(step, task, sub_dir) -> str:
    spec = step["keyring_from"]
    with io.open(resolve(spec["key"], task, sub_dir), encoding="utf-8") as fh:
        key = json.load(fh)
    keyring = {"schemaVersion": "1.0", "kind": "keyring",
               "keys": [{"keyId": spec["keyId"], "publicKey": key["publicKey"],
                         "revoked": None}]}
    path = resolve(spec["out"], task, sub_dir)
    write_json_file(path, keyring)
    return path


def apply_answer_step(step, bin_path, task, sub_dir, answer) -> dict:
    for key, spec in step["answer"].items():
        if "const" in spec:
            answer[key] = spec["const"]
            continue
        env = run_step(bin_path, spec["cmd"], task, sub_dir,
                       spec.get("allowExits", list(DEFAULT_ALLOW_EXITS)))
        if env is None:
            raise RuntimeError(f"{task_id_of(task)}: 답안 봉투 파싱 실패")
        value = dig(env, spec.get("path", ""))
        answer[key] = len(value) if spec.get("len") else value
    return answer


def apply_step(step, bin_path, pack_id, task, sub_dir, answer) -> str:
    kind = step_kind(step)
    if kind == "run":
        run_step(bin_path, step["run"], task, sub_dir, allow_exits_of(step))
        return "run"
    if kind == "copy":
        apply_copy_step(step, task, sub_dir)
        return "copy"
    if kind == "write_json":
        apply_write_json_step(step, task, sub_dir)
        return "write_json"
    if kind == "keyring_from":
        apply_keyring_step(step, task, sub_dir)
        return "keyring_from"
    if kind == "answer":
        apply_answer_step(step, bin_path, task, sub_dir, answer)
        return "answer"
    raise RuntimeError(f"{task_id_of(task)}: 알 수 없는 기준 풀이 단계 {list(step)}")


def run_step(bin_path, args, task, sub_dir, allow_exits):
    resolved = resolve_args(args, task, sub_dir)
    proc = subprocess.run([bin_path] + resolved, cwd=ROOT, capture_output=True)
    out = proc.stdout.decode("utf-8", errors="replace")
    if proc.returncode not in allow_exits:
        stderr = proc.stderr.decode("utf-8", "replace")[:ERROR_HEAD_LIMIT]
        raise RuntimeError(
            f"기준 풀이 실패 (exit {proc.returncode}, 허용 {allow_exits}): "
            f"{' '.join(str(item) for item in resolved[:4])}\n  {stderr}")
    try:
        return json.loads(out)
    except ValueError:
        return None


def submission_dir(sub_root, pack_id, task) -> str:
    return os.path.join(sub_root, pack_id, task_id_of(task))


def pack_submission_root(sub_root, pack_id) -> str:
    return os.path.join(sub_root, pack_id)


def build_task(bin_path, pack_id, task, reference, sub_root):
    sub_dir = submission_dir(sub_root, pack_id, task)
    shutil.rmtree(sub_dir, ignore_errors=True)
    os.makedirs(sub_dir, exist_ok=True)
    answer = {}
    steps = steps_of_reference(reference)
    if steps is None:
        raise RuntimeError(f"{task_id_of(task)}: steps 가 목록이 아니다")
    for step in steps:
        apply_step(step, bin_path, pack_id, task, sub_dir, answer)
    write_answer_file(sub_dir, answer)
    return sub_dir


def verify_built_task(bin_path, pack_id, task, sub_root):
    """방금 만든 제출물을 같은 pack 경로에서 실제 채점한다."""
    result = runner.score_task(task, pack_submission_root(sub_root, pack_id), bin_path)
    return score_failure_message(pack_id, task_id_of(task), result)


def inspect_built_task(bin_path, pack_id, task, sub_root, reference=None) -> dict:
    """부재 산출을 먼저 보고, 없으면 채점한다. 순수 접기 + 채점 호출."""
    sub_dir = submission_dir(sub_root, pack_id, task)
    missing = missing_artifact_message(pack_id, task, sub_dir, reference)
    if missing:
        return {
            "ok": False,
            "kind": "missing-artifact",
            "pack": pack_id,
            "task": task_id_of(task),
            "message": missing,
            "missing": missing_artifacts(sub_dir, expected_artifacts(task, reference)),
        }
    failure = verify_built_task(bin_path, pack_id, task, sub_root)
    if failure:
        return {
            "ok": False,
            "kind": "failed-score",
            "pack": pack_id,
            "task": task_id_of(task),
            "message": failure,
        }
    return {
        "ok": True,
        "kind": "ok",
        "pack": pack_id,
        "task": task_id_of(task),
        "message": None,
    }


def reference_path(pack_id, task_id) -> str:
    return os.path.join(PACKS_DIR, pack_id, "reference", f"{task_id}.json")


def load_reference(path):
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def process_one_task(bin_path, pack_id, task, reference, sub_root, counts) -> dict:
    """한 과제를 조립·검증하고 counts 를 갱신한다."""
    try:
        build_task(bin_path, pack_id, task, reference, sub_root)
        inspected = inspect_built_task(bin_path, pack_id, task, sub_root, reference)
    except CATCHABLE_EXCEPTIONS as exc:
        bump_count(counts, "failed")
        bump_count(counts, "buildError")
        message = format_task_failure(pack_id, task_id_of(task), exc)
        print(f"  X {message}")
        return {"ok": False, "kind": "build-error", "message": message}
    if inspected["ok"]:
        bump_count(counts, "built")
        return inspected
    bump_count(counts, "failed")
    if inspected["kind"] == "missing-artifact":
        bump_count(counts, "missingArtifact")
    elif inspected["kind"] == "failed-score":
        bump_count(counts, "failedScore")
    print(f"  X {inspected['message']}")
    return inspected


def process_pack(bin_path, pack_id, sub_root, counts) -> None:
    ref_dir = os.path.join(PACKS_DIR, pack_id, "reference")
    if not os.path.isdir(ref_dir):
        print(f"[{pack_id}] 기준 풀이 없음 — 건너뜀")
        return
    _manifest, tasks = runner.load_pack(pack_id)
    for task in tasks:
        ref_path = reference_path(pack_id, task_id_of(task))
        if not os.path.exists(ref_path):
            bump_count(counts, "skipped")
            continue
        try:
            reference = load_reference(ref_path)
        except CATCHABLE_EXCEPTIONS as exc:
            bump_count(counts, "failed")
            bump_count(counts, "buildError")
            print(f"  X {format_task_failure(pack_id, task_id_of(task), exc)}")
            continue
        process_one_task(bin_path, pack_id, task, reference, sub_root, counts)


def parse_args(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--agent", default="claude-fable-5")
    ap.add_argument("--pack", action="append", default=None)
    ap.add_argument("--bin", default=None)
    return ap.parse_args(argv)


def cli_flag_names() -> tuple[str, ...]:
    return CLI_FLAGS


def main(argv=None):
    a = parse_args(argv)

    bin_path = runner.find_bin(a.bin)
    sub_root = os.path.join(runner.GYM, "submissions", a.agent)
    pack_ids = a.pack or runner.discover_packs()

    counts = empty_counts()
    for pack_id in pack_ids:
        process_pack(bin_path, pack_id, sub_root, counts)
    print(format_summary(counts))
    if int(counts.get("failed") or 0):
        print(format_summary_detail(counts))
    return summary_exit(counts)


if __name__ == "__main__":
    sys.exit(main())
