"""gym 판별력 감사 — 각 과제가 "일 안 한 제출"을 실제로 거부하는가(약한 오라클 색출).

## 왜 이 도구인가 (프론티어를 선제 차단)

2026 벤치마크의 최대 위기는 **false-pass**다: OpenAI 감사에서 SWE-Bench Verified
최난도 과제의 59.4%가 버그를 안 고쳐도 테스트가 통과했다(약한 오라클). 채점이
'일을 했나'가 아니라 '파일이 있나' 만 보면, 아무것도 안 한 제출도 만점을 받는다.

이 감사기는 그 결함을 gym 에 **못 들어오게** 막는다 — 사후에 손으로 발견하는 대신,
각 과제에 **음성 대조**(일 안 한 제출)를 넣어 채점해 **반드시 실패**하는지 본다.

음성 대조 구성(종류는 시험이 고정한다):
- **wrong-answer** — 모든 답 키에 명백한 오답(sentinel). answer_eq 가 진값과
  대조하니 거부해야 한다.
- **input-copy** — artifact 과제의 산출 자리에 입력을 그대로 복사한다.
  `differs_from_input` 이 거부해야 한다.
- **garbage** — 같은 자리에 1KiB 넘는 synthetic garbage 를 쓴다.
  `differs_from_input` 만으로는 통과할 수 있으므로 형식·핵심값 검사도
  함께 요구한다.

음성 대조에 **통과하는** 과제 = 판별력 없는 약한 오라클(false-pass). 이걸
리포트한다. 통과 못 하면(=거부) 그 과제는 진짜 일을 요구하는 것이다.

보고 봉투: `kind=gymDiscrimination`, `schemaVersion=1.0`. 판정·집계·경로
안전·예외 접기는 순수 함수라 `scripts/tests/test_gym_discriminate.py` 가
바이너리 없이 고정한다. 새 CLI 플래그·새 pack 은 없다.

## 사용

    python gym/tools/discriminate.py --bin target/debug/rhwp        # 전 과제 판별 감사
    python gym/tools/discriminate.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, GYM_ROOT)

from core import runner  # noqa: E402

# 진값과 절대 같을 리 없는 오답 — 숫자 진값엔 문자열이라 타입부터 다르고, 문자열
# 진값엔 이 특이 문자열이라 값이 다르다. answer_eq 는 어느 쪽이든 거부한다.
WRONG_SENTINEL = "__NEGATIVE_CONTROL_definitely_wrong__"
GARBAGE_MARKER = b"RHWP_GYM_GARBAGE_NEGATIVE_CONTROL\x00"
GARBAGE_REPEAT = 64
GARBAGE_BYTES = GARBAGE_MARKER * GARBAGE_REPEAT
GARBAGE_MIN_SIZE = 1024

CONTROL_WRONG_ANSWER = "wrong-answer"
CONTROL_INPUT_COPY = "input-copy"
CONTROL_GARBAGE = "garbage"

ANSWER_CONTROLS = (CONTROL_WRONG_ANSWER,)
ARTIFACT_CONTROLS = (CONTROL_INPUT_COPY, CONTROL_GARBAGE)
CONTROL_KINDS = ANSWER_CONTROLS + ARTIFACT_CONTROLS

#: 음성 대조 카탈로그. 문서·시험·도구가 같은 표를 본다.
CONTROL_CATALOG = (
    {
        "id": CONTROL_WRONG_ANSWER,
        "submit": "answer",
        "writes": "answer.json",
        "payload": "WRONG_SENTINEL",
        "mustFail": True,
        "why": "answer_eq 가 진값과 대조하므로 sentinel 은 거부돼야 한다",
    },
    {
        "id": CONTROL_INPUT_COPY,
        "submit": "artifact",
        "writes": "submit.files",
        "payload": "task.input 바이트 복사",
        "mustFail": True,
        "why": "무편집 복사는 일을 하지 않은 제출이다",
    },
    {
        "id": CONTROL_GARBAGE,
        "submit": "artifact",
        "writes": "submit.files",
        "payload": "GARBAGE_BYTES",
        "mustFail": True,
        "why": "입력과 다른 쓰레기만으로는 형식·핵심값 검사가 있어야 거부된다",
    },
)

REPORT_KIND = "gymDiscrimination"
SCHEMA_VERSION = "1.0"
NEGATIVE_DIRNAME = "_negative_control"

REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "taskCount",
    "controlCount",
    "discriminating",
    "falsePass",
    "falsePassControls",
)

OPTIONAL_REPORT_KEYS = (
    "results",
    "loadErrors",
    "scoreErrors",
    "buildErrors",
    "skipped",
    "toolFailed",
    "toolErrors",
    "controlKinds",
)

EXIT_OK = 0
EXIT_FALSE_PASS = 1

FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

CATCHABLE_EXCEPTIONS = (
    OSError,
    ValueError,
    TypeError,
    KeyError,
    IndexError,
    AttributeError,
    json.JSONDecodeError,
    RuntimeError,
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


def is_fatal_exception(exc) -> bool:
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def control_ids() -> tuple[str, ...]:
    """카탈로그 id 튜플. 순서는 answer → copy → garbage."""
    return tuple(item["id"] for item in CONTROL_CATALOG)


def control_spec(control_id: str) -> dict | None:
    """id 로 카탈로그 행을 찾는다. 없으면 None."""
    for item in CONTROL_CATALOG:
        if item["id"] == control_id:
            return dict(item)
    return None


def is_known_control(control_id: str) -> bool:
    return control_id in CONTROL_KINDS


def is_artifact_control(control_id: str) -> bool:
    return control_id in ARTIFACT_CONTROLS


def is_answer_control(control_id: str) -> bool:
    return control_id in ANSWER_CONTROLS


def submit_mapping(task) -> dict:
    """task.submit 이 dict 가 아니면 빈 dict."""
    if not isinstance(task, dict):
        return {}
    submit = task.get("submit")
    return submit if isinstance(submit, dict) else {}


def submit_kind(task) -> str:
    kind = submit_mapping(task).get("kind")
    return kind if isinstance(kind, str) else ""


def is_artifact_task(task) -> bool:
    return submit_kind(task) == "artifact"


def is_answer_task(task) -> bool:
    return submit_kind(task) == "answer"


def is_pair_task(task) -> bool:
    return submit_kind(task) == "pair"


def answer_keys(task: dict) -> set[str]:
    """answer_eq 계열이 지목한 키. 비-dict 검사·빈 checks 는 무시."""
    keys = set()
    if not isinstance(task, dict):
        return keys
    checks = task.get("checks")
    if not isinstance(checks, list):
        return keys
    for check in checks:
        if not isinstance(check, dict):
            continue
        key = check.get("answer")
        if isinstance(key, str) and key:
            keys.add(key)
    return keys


def submit_files(task) -> list[str]:
    """submit.files 중 상대경로로 쓸 수 있는 것만. 순서는 선언 순."""
    raw = submit_mapping(task).get("files")
    if not isinstance(raw, list):
        return []
    out = []
    seen = set()
    for item in raw:
        rel = normalize_rel(item)
        if rel is None or rel in seen:
            continue
        seen.add(rel)
        out.append(rel)
    return out


def controls_for(task) -> tuple[str, ...]:
    """과제에 적용할 음성 대조. artifact 는 copy+garbage, 그 외는 sentinel."""
    if is_artifact_task(task):
        return ARTIFACT_CONTROLS
    return ANSWER_CONTROLS


def sentinel_answers(keys) -> dict:
    """모든 키에 같은 sentinel. 키 순서는 정렬해 결정적으로 쓴다."""
    return {str(k): WRONG_SENTINEL for k in sorted(keys)}


def is_wrong_sentinel(value) -> bool:
    return value == WRONG_SENTINEL


def garbage_size() -> int:
    return len(GARBAGE_BYTES)


def is_garbage_payload(data) -> bool:
    return data == GARBAGE_BYTES


def garbage_meets_minimum() -> bool:
    return garbage_size() >= GARBAGE_MIN_SIZE


def _as_rel_text(rel) -> str | None:
    if not isinstance(rel, str):
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
    if not isinstance(rel, str):
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


def join_sub(sub_dir: str, rel: str) -> str:
    """정규화된 상대경로를 제출 폴더 아래에 붙인다."""
    safe = normalize_rel(rel)
    if safe is None:
        raise ValueError(f"불안전 상대경로: {rel!r}")
    parts = safe.split("/")
    return os.path.join(sub_dir, *parts)


def score_is_pass(result) -> bool:
    """채점 봉투의 pass 만 본다. 비-dict·키 없음은 실패."""
    if not isinstance(result, dict):
        return False
    return bool(result.get("pass"))


def score_discriminates(result) -> bool:
    """음성 대조가 거부되면 판별력이 있다."""
    return not score_is_pass(result)


def normalize_score(result) -> dict:
    """채점 결과를 최소 봉투로. 비-dict 는 pass=False + error."""
    if not isinstance(result, dict):
        return {"pass": False, "error": "채점 결과가 dict 가 아니다"}
    out = {"pass": bool(result.get("pass"))}
    if "error" in result:
        out["error"] = result.get("error")
    if "id" in result:
        out["id"] = result.get("id")
    return out


def false_pass_label(pack_id: str, task_id: str) -> str:
    return f"{pack_id}/{task_id}"


def false_pass_control_label(pack_id: str, task_id: str, control: str) -> str:
    return f"{pack_id}/{task_id} ({control})"


def parse_false_pass_label(label: str) -> tuple[str, str] | None:
    if not isinstance(label, str) or "/" not in label:
        return None
    pack_id, task_id = label.split("/", 1)
    if not pack_id or not task_id:
        return None
    return pack_id, task_id


def split_false_pass_control_label(label: str) -> tuple[str, str, str] | None:
    """`pack/task (control)` → (pack, task, control). 형식이 아니면 None."""
    if not isinstance(label, str) or " (" not in label or not label.endswith(")"):
        return None
    head, tail = label.rsplit(" (", 1)
    parsed = parse_false_pass_label(head)
    if parsed is None:
        return None
    control = tail[:-1]
    if not is_known_control(control):
        return None
    return parsed[0], parsed[1], control


def expected_control_count(task) -> int:
    return len(controls_for(task))


def row_is_false_pass(row) -> bool:
    if not isinstance(row, dict):
        return False
    return not bool(row.get("discriminates"))


def format_json_report(report: dict) -> str:
    return json.dumps(report, ensure_ascii=False, indent=2) + "\n"


def empty_report() -> dict:
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": True,
        "taskCount": 0,
        "controlCount": 0,
        "discriminating": 0,
        "falsePass": [],
        "falsePassControls": [],
        "results": [],
        "loadErrors": [],
        "scoreErrors": [],
        "buildErrors": [],
        "skipped": [],
        "toolFailed": False,
        "toolErrors": [],
        "controlKinds": list(CONTROL_KINDS),
    }


def unique_keep_order(items) -> list:
    seen = set()
    out = []
    for item in items:
        if item in seen:
            continue
        seen.add(item)
        out.append(item)
    return out


def aggregate_rows(rows, task_count: int, extras: dict | None = None) -> dict:
    """대조 행에서 보고 봉투를 만든다. 순수."""
    report = empty_report()
    false_pass = []
    false_pass_controls = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        report["results"].append(row)
        if row.get("discriminates"):
            continue
        pack_id = row.get("pack") or ""
        task_id = row.get("task") or ""
        control = row.get("control") or ""
        label = false_pass_label(pack_id, task_id)
        if label not in false_pass:
            false_pass.append(label)
        false_pass_controls.append(false_pass_control_label(pack_id, task_id, control))
    report["taskCount"] = int(task_count)
    report["controlCount"] = len(report["results"])
    report["falsePass"] = false_pass
    report["falsePassControls"] = false_pass_controls
    report["discriminating"] = report["taskCount"] - len(false_pass)
    report["ok"] = len(false_pass) == 0
    if extras:
        for key in OPTIONAL_REPORT_KEYS:
            if key in extras and extras[key] is not None:
                report[key] = extras[key]
        if extras.get("toolFailed"):
            report["toolFailed"] = True
    return report


def validate_report(report) -> list[str]:
    """보고 봉투의 정직 계약. 문제 문장 목록(없으면 빈 목록)."""
    issues = []
    if not isinstance(report, dict):
        return ["보고가 dict 가 아니다"]
    for key in REPORT_KEYS:
        if key not in report:
            issues.append(f"필수 키 없음: {key}")
    if report.get("kind") != REPORT_KIND:
        issues.append(f"kind 가 {REPORT_KIND} 가 아니다")
    if report.get("schemaVersion") != SCHEMA_VERSION:
        issues.append("schemaVersion 이 1.0 이 아니다")
    if not isinstance(report.get("ok"), bool):
        issues.append("ok 가 bool 이 아니다")
    for key in ("taskCount", "controlCount", "discriminating"):
        value = report.get(key)
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            issues.append(f"{key} 가 음이 아닌 정수가 아니다")
    false_pass = report.get("falsePass")
    controls = report.get("falsePassControls")
    if not isinstance(false_pass, list):
        issues.append("falsePass 가 list 가 아니다")
        false_pass = []
    if not isinstance(controls, list):
        issues.append("falsePassControls 가 list 가 아니다")
        controls = []
    if bool(false_pass) == bool(report.get("ok")) and report.get("ok") is not None:
        # ok 는 falsePass 가 비었을 때만 참이어야 한다.
        if report.get("ok") and false_pass:
            issues.append("ok 가 참인데 falsePass 가 있다")
        if (not report.get("ok")) and (not false_pass):
            issues.append("ok 가 거짓인데 falsePass 가 비었다")
    expected_disc = report.get("taskCount", 0) - len(false_pass)
    if report.get("discriminating") != expected_disc:
        issues.append("discriminating 이 taskCount-len(falsePass) 가 아니다")
    results = report.get("results")
    if results is not None:
        if not isinstance(results, list):
            issues.append("results 가 list 가 아니다")
        elif report.get("controlCount") != len(results):
            issues.append("controlCount 가 results 길이와 다르다")
        for row in results:
            if not isinstance(row, dict):
                issues.append("results 행이 dict 가 아니다")
                continue
            control = row.get("control")
            if control is not None and not is_known_control(control):
                issues.append(f"미지 대조: {control}")
    for label in false_pass:
        if parse_false_pass_label(label) is None:
            issues.append(f"falsePass 라벨 형식 오류: {label!r}")
    for label in controls:
        if not isinstance(label, str) or " (" not in label or not label.endswith(")"):
            issues.append(f"falsePassControls 라벨 형식 오류: {label!r}")
            continue
        head, tail = label.rsplit(" (", 1)
        control = tail[:-1]
        if parse_false_pass_label(head) is None:
            issues.append(f"falsePassControls 머리 형식 오류: {label!r}")
        if not is_known_control(control):
            issues.append(f"falsePassControls 미지 대조: {control}")
    kinds = report.get("controlKinds")
    if kinds is not None and list(kinds) != list(CONTROL_KINDS):
        issues.append("controlKinds 가 카탈로그와 다르다")
    return issues


def human_lines(report: dict) -> list[str]:
    """사람이 읽는 한 줄 요약 + false-pass 목록."""
    if report.get("ok"):
        return [
            f"gym 판별력 감사: {report.get('taskCount', 0)} 과제 전부 음성 대조를 거부 — 약한 오라클 0"
        ]
    lines = [
        f"gym 판별력 감사: 약한 오라클(false-pass) {len(report.get('falsePass') or [])}건 — "
        "일 안 한 제출이 통과한다:"
    ]
    for item in report.get("falsePass") or []:
        lines.append(f"  - {item}")
    extra = report.get("falsePassControls") or []
    if extra:
        lines.append("대조별:")
        for item in extra:
            lines.append(f"  - {item}")
    return lines


def format_human_report(report: dict) -> str:
    return "\n".join(human_lines(report)) + "\n"


def ensure_dir(path: str) -> None:
    os.makedirs(path, exist_ok=True)


def write_json(path: str, obj) -> None:
    parent = os.path.dirname(path)
    if parent:
        ensure_dir(parent)
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(obj, fh, ensure_ascii=False)


def write_bytes(path: str, data: bytes) -> None:
    parent = os.path.dirname(path)
    if parent:
        ensure_dir(parent)
    with open(path, "wb") as fh:
        fh.write(data)


def copy_file(src: str, dst: str) -> None:
    parent = os.path.dirname(dst)
    if parent:
        ensure_dir(parent)
    shutil.copyfile(src, dst)


def load_json(path: str):
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def iter_pack_ids(packs_dir: str) -> list[str]:
    """tasks/ 가 있는 pack id. 사전순. 없거나 읽기 실패면 빈 목록."""
    if not packs_dir or not os.path.isdir(packs_dir):
        return []
    try:
        names = os.listdir(packs_dir)
    except OSError:
        return []
    out = []
    for name in sorted(names):
        pack_dir = os.path.join(packs_dir, name)
        try:
            if os.path.isdir(os.path.join(pack_dir, "tasks")):
                out.append(name)
        except OSError:
            continue
    return out


def iter_task_names(tasks_dir: str) -> list[str]:
    """*.json 파일 이름. 사전순."""
    if not tasks_dir or not os.path.isdir(tasks_dir):
        return []
    try:
        names = os.listdir(tasks_dir)
    except OSError:
        return []
    return sorted(name for name in names if name.endswith(".json"))


def load_task(path: str) -> tuple[dict | None, str | None]:
    """과제 JSON. (task, None) 또는 (None, error)."""
    try:
        task = load_json(path)
    except (OSError, ValueError) as exc:
        return None, f"{os.path.basename(path)} 파싱 실패: {exc}"
    if not isinstance(task, dict):
        return None, f"{os.path.basename(path)} 가 객체가 아니다"
    return task, None


def task_id_of(task) -> str | None:
    if not isinstance(task, dict):
        return None
    tid = task.get("id")
    if isinstance(tid, str) and tid.strip():
        return tid
    return None


def resolve_input_path(task, repo_root: str | None = None) -> str:
    root = repo_root if repo_root is not None else REPO_ROOT
    rel = task.get("input") if isinstance(task, dict) else None
    if not isinstance(rel, str) or not rel:
        return ""
    if os.path.isabs(rel):
        return rel
    return os.path.join(root, *rel.replace("\\", "/").split("/"))


def write_answer_file(sub_dir: str, keys) -> str | None:
    if not keys:
        return None
    path = os.path.join(sub_dir, "answer.json")
    write_json(path, sentinel_answers(keys))
    return path


def write_artifact_file(dst: str, src: str, mode: str) -> str:
    """copied / garbage / skipped / rejected 중 하나."""
    if mode == CONTROL_GARBAGE:
        write_bytes(dst, GARBAGE_BYTES)
        return "garbage"
    if mode == CONTROL_INPUT_COPY:
        if src and os.path.isfile(src):
            copy_file(src, dst)
            return "copied"
        return "skipped"
    return "rejected"


def prepare_submission_dir(neg_pack_dir: str, task_id: str) -> str:
    sub_dir = os.path.join(neg_pack_dir, task_id)
    shutil.rmtree(sub_dir, ignore_errors=True)
    ensure_dir(sub_dir)
    return sub_dir


def build_negative(task: dict, neg_pack_dir: str, artifact_mode: str = "input-copy") -> dict:
    """음성 대조 제출물 — 오답 answer.json + artifact별 무편집/garbage 대조.

    artifact_mode:
      - input-copy: 입력을 산출 자리에 복사. 입력이 없으면 그 파일은 건너뛴다.
      - garbage: GARBAGE_BYTES 를 산출 자리에 쓴다.
      - wrong-answer: 답만 쓰고 산출 파일은 만들지 않는다.
    미지 모드는 artifact 과제에서 ValueError.
    """
    tid = task_id_of(task) or "T"
    sub_dir = prepare_submission_dir(neg_pack_dir, tid)
    keys = answer_keys(task)
    answer_path = write_answer_file(sub_dir, keys)
    files_out = []
    errors = []
    if is_artifact_task(task):
        if artifact_mode not in (CONTROL_INPUT_COPY, CONTROL_GARBAGE, CONTROL_WRONG_ANSWER):
            raise ValueError(f"지원하지 않는 artifact 음성 대조: {artifact_mode}")
        if artifact_mode != CONTROL_WRONG_ANSWER:
            src = resolve_input_path(task)
            raw_files = submit_mapping(task).get("files") or []
            if not isinstance(raw_files, list):
                raw_files = []
            for item in raw_files:
                reason = unsafe_rel_reason(item)
                rel = normalize_rel(item)
                if rel is None:
                    errors.append(f"불안전 산출 경로 거부: {item!r} ({reason})")
                    files_out.append({"rel": item, "action": "rejected", "path": None})
                    continue
                dst = join_sub(sub_dir, rel)
                action = write_artifact_file(dst, src, artifact_mode)
                path = dst if action in ("copied", "garbage") else None
                if action == "skipped":
                    errors.append(f"입력 복사 생략(원본 없음): {task.get('input')!r} → {rel}")
                files_out.append({"rel": rel, "action": action, "path": path})
    return {
        "taskId": tid,
        "mode": artifact_mode,
        "answerKeys": sorted(keys),
        "answerPath": answer_path,
        "files": files_out,
        "errors": errors,
        "dir": sub_dir,
    }


def score_task_safe(task, pack_dir, bin_path, score_fn=None):
    """채점 예외를 봉투로 접는다. 치명 예외는 다시 올린다."""
    fn = score_fn or runner.score_task
    try:
        return normalize_score(fn(task, pack_dir, bin_path))
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return {"pass": False, "error": f"채점 예외: {type(exc).__name__}: {exc}"}
    except Exception as exc:  # noqa: BLE001 — 한 과제가 전수 감사를 죽이면 안 된다
        if is_fatal_exception(exc):
            raise
        return {"pass": False, "error": f"채점 예외: {type(exc).__name__}: {exc}"}


def packs_dir_of(gym_root: str) -> str:
    return os.path.join(gym_root, "packs")


def control_pack_dir(neg_root: str, control: str, pack_id: str) -> str:
    return os.path.join(neg_root, control, pack_id)


def make_result_row(pack_id, task_id, control, discriminates, extra=None) -> dict:
    row = {
        "pack": pack_id,
        "task": task_id,
        "control": control,
        "discriminates": bool(discriminates),
    }
    if extra:
        row.update(extra)
    return row


def discover_task_entries(gym_root: str) -> tuple[list[dict], list[str], list[str]]:
    """(entries, loadErrors, toolErrors). entry: pack, name, task."""
    entries = []
    load_errors = []
    tool_errors = []
    packs_dir = packs_dir_of(gym_root)
    if not os.path.isdir(packs_dir):
        return entries, load_errors, tool_errors
    try:
        pack_ids = iter_pack_ids(packs_dir)
    except OSError as exc:
        tool_errors.append(f"packs 목록 실패: {exc}")
        return entries, load_errors, tool_errors
    for pack_id in pack_ids:
        tasks_dir = os.path.join(packs_dir, pack_id, "tasks")
        try:
            names = iter_task_names(tasks_dir)
        except OSError as exc:
            tool_errors.append(f"{pack_id} tasks 목록 실패: {exc}")
            continue
        for name in names:
            path = os.path.join(tasks_dir, name)
            task, err = load_task(path)
            if err:
                load_errors.append(f"{pack_id}/{name}: {err}")
                continue
            tid = task_id_of(task)
            if tid is None:
                load_errors.append(f"{pack_id}/{name}: 과제 id 가 없다")
                continue
            entries.append({"pack": pack_id, "name": name, "task": task})
    return entries, load_errors, tool_errors


def run_one_control(task, pack_id, control, neg_root, bin_path, score_fn=None) -> tuple[dict, list[str], list[str]]:
    """한 과제의 한 대조. (row, buildErrors, scoreErrors)."""
    build_errors = []
    score_errors = []
    tid = task_id_of(task) or "?"
    dest = control_pack_dir(neg_root, control, pack_id)
    try:
        built = build_negative(task, dest, artifact_mode=control)
    except FATAL_EXCEPTIONS:
        raise
    except (OSError, ValueError, TypeError) as exc:
        build_errors.append(f"{pack_id}/{tid} ({control}): 구성 실패: {exc}")
        row = make_result_row(pack_id, tid, control, True, {"error": f"구성 실패: {exc}"})
        return row, build_errors, score_errors
    for item in built.get("errors") or []:
        build_errors.append(f"{pack_id}/{tid} ({control}): {item}")
    scored = score_task_safe(task, dest, bin_path, score_fn=score_fn)
    if scored.get("error"):
        score_errors.append(f"{pack_id}/{tid} ({control}): {scored['error']}")
    row = make_result_row(
        pack_id,
        tid,
        control,
        score_discriminates(scored),
        {"error": scored.get("error")} if scored.get("error") else None,
    )
    return row, build_errors, score_errors


def discriminate(bin_path: str, gym_root: str, neg_root: str, score_fn=None) -> dict:
    """전 pack 음성 대조. score_fn 이 있으면 runner.score_task 대신 쓴다."""
    extras = {
        "loadErrors": [],
        "scoreErrors": [],
        "buildErrors": [],
        "skipped": [],
        "toolFailed": False,
        "toolErrors": [],
        "controlKinds": list(CONTROL_KINDS),
    }
    try:
        entries, load_errors, tool_errors = discover_task_entries(gym_root)
    except FATAL_EXCEPTIONS:
        raise
    except OSError as exc:
        extras["toolFailed"] = True
        extras["toolErrors"] = [f"과제 탐색 실패: {exc}"]
        return aggregate_rows([], 0, extras)
    extras["loadErrors"] = load_errors
    extras["toolErrors"] = tool_errors
    if tool_errors:
        extras["toolFailed"] = True
    rows = []
    fn = score_fn or runner.score_task
    for entry in entries:
        task = entry["task"]
        pack_id = entry["pack"]
        for control in controls_for(task):
            row, build_err, score_err = run_one_control(
                task, pack_id, control, neg_root, bin_path, score_fn=fn
            )
            rows.append(row)
            extras["buildErrors"].extend(build_err)
            extras["scoreErrors"].extend(score_err)
    return aggregate_rows(rows, len(entries), extras)


def prepare_neg_root(neg_root: str) -> str:
    shutil.rmtree(neg_root, ignore_errors=True)
    ensure_dir(neg_root)
    return neg_root


def default_neg_root(gym_root: str | None = None) -> str:
    root = gym_root if gym_root is not None else GYM_ROOT
    return os.path.join(root, "submissions", NEGATIVE_DIRNAME)


def run_audit(bin_path: str, gym_root: str | None = None, neg_root: str | None = None) -> dict:
    gym = gym_root if gym_root is not None else GYM_ROOT
    dest = neg_root if neg_root is not None else default_neg_root(gym)
    prepare_neg_root(dest)
    return discriminate(bin_path, gym, dest)


def parse_args(argv=None) -> argparse.Namespace:
    ap = argparse.ArgumentParser(description="gym 판별력 감사 — 약한 오라클(false-pass) 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--json", action="store_true")
    return ap.parse_args(argv)


def emit_report(report: dict, as_json: bool, stream=None) -> None:
    out = sys.stdout if stream is None else stream
    if as_json:
        out.write(format_json_report(report))
        return
    out.write(format_human_report(report))


def exit_code(report: dict) -> int:
    return EXIT_OK if report.get("ok") else EXIT_FALSE_PASS


def main(argv=None) -> int:
    args = parse_args(argv)
    bin_path = runner.find_bin(args.bin)
    report = run_audit(bin_path)
    emit_report(report, args.json)
    return exit_code(report)


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
