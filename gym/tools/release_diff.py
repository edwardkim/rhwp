"""[#4661] 릴리스 간 차등 회귀 — 두 바이너리로 같은 과제를 돌려 동작 변화를 잡는다.

## 착상

운동장 채점기는 정답을 박제하지 않고 채점 시점에 rhwp 로 재계산한다(#4653).
그러니 **같은 제출물을 두 바이너리로 채점하면 답이 같아야 한다** — 다르면 그
사이 릴리스에서 동작이 바뀐 것이다. #4658(교차형식 차등)이 검증한 원리를
형식축에서 시간축으로 돌리는 것뿐이다. 새 메커니즘 0.

리더보드 총점은 통과/실패의 이진값이라 둔감하다. 대신 각 과제 검사의 **관측값**
(봉투에서 길어낸 raw — 쪽수·표수·필드값·해시·판정 문자열)을 두 바이너리에서
뽑아 대조한다. 골든 없이, 관측이 갈리는 지점이 곧 회귀 후보다.

## 오검출 관문 (도구가 거짓말하지 않도록)

1. **명령 표면 대조** — 두 바이너리의 capabilities digest 가 같으면 관측 변화는
   순수 동작 회귀(regression). 다르면 표면이 바뀐 릴리스(surface-changed)로
   분류 — 의도된 변경일 수 있어 사람 판정 몫.
2. **판정성 종료 코드 허용** — exit 3(판정 데이터)은 실패가 아니다.
3. **비결정 관측 배제** — 파일 경로·산출 파일 크기처럼 릴리스와 무관하게
   흔들리는 자리는 대조에서 뺀다(파일 산출 과제의 file_exists 는 관측이 아니라
   존재 여부라 애초에 raw 비교 대상이 아니다).

분류·관측 동일성·보고 조립은 순수 함수라
`scripts/tests/test_gym_release_diff.py` 가 바이너리 없이 고정한다.

## 정직 조항

이 도구는 "무엇이 바뀌었나" 를 가리키지 "어느 쪽이 옳은가" 를 판정하지 않는다
(한컴 정답지 없음 — #4658 과 같은 결). 판정은 사람이 한다.

분류 함수 `classify` 는 오직 stable / regression / surface-changed 만 낸다.
바이너리를 부르지 못해 표면을 재지 못하면 그 세 값 중 아무것으로도 위장하지
않는다 — `probe-failed` 는 분류가 아니라 **도구 실패 상태**다.

사용:
  python gym/tools/release_diff.py --old <구 바이너리> --new <신 바이너리>
                                   [--pack <id> ...] [-o 리포트.json]
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402
from gym.core import checks as check_registry  # noqa: E402

ROOT = runner.ROOT

REPORT_KIND = "gymReleaseDiff"
SCHEMA_VERSION = "1.0"

#: 봉투를 부르지 않는 파일 연산자 — 관측이 아니라 존재/동일성이라 raw 대조 제외.
FILE_OPS = {"file_exists", "same_hash", "differs_from_input", "files_differ"}

#: 분류 삼원. 이 튜플에 probe-failed 를 넣지 않는다 — 그건 분류가 아니다.
CLASSIFICATIONS = ("stable", "regression", "surface-changed")
EXIT_BY_CLASS = {"stable": 0, "regression": 3, "surface-changed": 2}
CLASSIFICATION_REASON = {
    "stable": "명령 표면과 관측이 같다",
    "regression": "명령 표면은 같고 관측이 갈렸다 — 순수 동작 변화",
    "surface-changed": "명령 표면(capabilities digest)이 달라 사람 판정이 필요하다",
}

#: 표면을 재지 못했을 때의 도구 상태. classify() 가 이 값을 내는 일은 없다.
STATUS_PROBE_FAILED = "probe-failed"
EXIT_PROBE_FAILED = 1
PROBE_FAILED_REASON = (
    "capabilities digest 를 구하지 못해 분류하지 않는다 "
    "— stable/regression/surface-changed 로 위장하지 않는다"
)

#: capabilities 프로브 기본 초. 0 이하는 시간제한 없음(호출 측에서 거른다).
DIGEST_TIMEOUT = 30

#: 관측 head / 오류 메시지 머리 길이.
HEAD_LIMIT = 80
ERROR_HEAD_LIMIT = 160

#: 관측 kind 카탈로그. 시험과 문서가 같은 표를 본다.
OBSERVATION_KINDS = (
    "value",
    "exit",
    "nojson",
    "digfail",
    "no-cmd",
    "resolve-error",
    "cli-error",
    "timeout",
    "missing-bin",
    "permission",
    "os-error",
    "type-error",
    "value-error",
    "decode-error",
    "unexpected",
)

#: JSON 보고 고정 키. 분류가 성공한 봉투의 최소 집합.
REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "old",
    "new",
    "surfaceChanged",
    "tasksCompared",
    "observationsCompared",
    "observationsSkipped",
    "divergences",
    "classification",
    "classificationReason",
    "exit",
    "ok",
    "reviewRequired",
    "diffs",
)

#: 프로브 실패 봉투에 추가로 붙는 키.
PROBE_REPORT_KEYS = (
    "probeFailed",
    "probeErrors",
)

#: 분류가 성공한 뒤에도 남을 수 있는 부가 키(팩 읽기 실패 등).
OPTIONAL_REPORT_KEYS = (
    "packErrors",
    "taskErrors",
    "writeError",
)

#: 예외 → 관측/프로브 kind. context 가 digest 이면 FileNotFound 는 missing-bin.
EXCEPTION_KIND_BY_TYPE = {
    FileNotFoundError: "missing-bin",
    PermissionError: "permission",
    TimeoutError: "timeout",
    subprocess.TimeoutExpired: "timeout",
    UnicodeError: "decode-error",
    UnicodeDecodeError: "decode-error",
    UnicodeEncodeError: "decode-error",
    ValueError: "value-error",
    TypeError: "type-error",
    KeyError: "digfail",
    IndexError: "digfail",
    AttributeError: "digfail",
    OSError: "os-error",
}

#: 삼키면 안 되는 예외 — 도구를 죽이는 것이 정직하다.
FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

#: 관측/프로브에서 잡는 예외. BaseException 전부가 아니다.
CATCHABLE_EXCEPTIONS = (
    FileNotFoundError,
    PermissionError,
    TimeoutError,
    subprocess.TimeoutExpired,
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


def is_fatal_exception(exc):
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def exception_kind(exc, context="observe"):
    """예외를 관측/프로브 kind 로 접는다. 순수.

    context:
      - digest: 바이너리 capabilities 프로브. FileNotFound → missing-bin.
      - resolve: 제출물 경로 해석. FileNotFound → resolve-error (기존 계약).
      - cli: run_cli 실패. FileNotFound → missing-bin.
      - observe / dig: 경로 평가. KeyError/IndexError → digfail.
    """
    if exc is None:
        return "unexpected"
    if isinstance(exc, subprocess.TimeoutExpired) or isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, FileNotFoundError):
        if context == "resolve":
            return "resolve-error"
        return "missing-bin"
    if isinstance(exc, PermissionError):
        return "permission"
    if isinstance(exc, UnicodeError):
        return "decode-error"
    if isinstance(exc, json.JSONDecodeError):
        return "value-error"
    if isinstance(exc, (KeyError, IndexError, AttributeError)):
        return "digfail"
    if isinstance(exc, TypeError):
        return "type-error"
    if isinstance(exc, ValueError):
        return "value-error"
    if isinstance(exc, OSError):
        return "os-error"
    if isinstance(exc, RuntimeError):
        return "cli-error"
    return "unexpected"


def exception_observation(exc, context="observe", head=""):
    """예외를 관측 봉투로 접는다. 여기서 예외를 다시 올리지 않는다."""
    kind = exception_kind(exc, context=context)
    return {
        "kind": kind,
        "error": type(exc).__name__ if exc is not None else "NoneType",
        "head": truncate_head(head or (str(exc) if exc is not None else ""), ERROR_HEAD_LIMIT),
    }


def exception_probe(exc, bin_path, role):
    """capabilities 프로브 실패 한 줄. 분류에 쓰지 않는다."""
    return {
        "role": role,
        "bin": os.path.basename(bin_path) if bin_path else "",
        "kind": exception_kind(exc, context="digest"),
        "error": type(exc).__name__ if exc is not None else "NoneType",
        "head": truncate_head(str(exc) if exc is not None else "", ERROR_HEAD_LIMIT),
    }


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


def normalize_timeout(timeout):
    """프로브 초. 변환 불능·비양수는 0 — 호출 측에서 시간제한을 끈다."""
    try:
        n = int(timeout)
    except (TypeError, ValueError):
        return 0
    return n if n > 0 else 0


def sha256_bytes(payload):
    """바이트열 SHA-256 16진. 비-바이트는 빈 바이트로 접지 않는다 — TypeError."""
    if isinstance(payload, memoryview):
        payload = payload.tobytes()
    if isinstance(payload, bytearray):
        payload = bytes(payload)
    if not isinstance(payload, bytes):
        raise TypeError(f"sha256_bytes 는 bytes-like 만 받습니다 (got {type(payload).__name__})")
    return hashlib.sha256(payload).hexdigest()


def is_sha256_hex(value):
    """64자 소문자/숫자 hex 인가. 순수. None/짧은 값은 거짓."""
    if not isinstance(value, str) or len(value) != 64:
        return False
    try:
        int(value, 16)
    except ValueError:
        return False
    return value == value.lower()


def normalize_digest(value):
    """digest 후보를 소문자 hex 로. 형식이 아니면 None — 빈 문자열로 위장하지 않는다."""
    if not isinstance(value, str):
        return None
    folded = value.strip().lower()
    return folded if is_sha256_hex(folded) else None


def observation_kind_of(obs):
    """관측 봉투의 kind. 형식이 아니면 None."""
    if isinstance(obs, dict):
        kind = obs.get("kind")
        if isinstance(kind, str) and kind:
            return kind
    return None


def is_known_observation_kind(kind):
    return kind in OBSERVATION_KINDS


def is_error_observation(obs):
    """값 관측이 아닌 실패/부재 관측인가. 분류를 바꾸지 않는다 — 대조 대상일 뿐이다."""
    kind = observation_kind_of(obs)
    if kind is None:
        return False
    return kind != "value"


def capabilities_digest(bin_path, timeout=None):
    """capabilities stdout 의 SHA-256. 성공 경로의 기존 계약.

    timeout 이 양수면 subprocess 에 넘긴다. 예외는 여기서 삼키지 않는다 —
    호출자가 probe_capabilities 로 접는다. stdout 이 비어 있어도 해시한다
    (기존 동작: 빈 출력의 digest 는 고정값이지 오류가 아니다).
    """
    kwargs = {"cwd": ROOT, "capture_output": True}
    limit = normalize_timeout(timeout) if timeout is not None else 0
    if limit:
        kwargs["timeout"] = limit
    proc = subprocess.run([bin_path, "capabilities"], **kwargs)
    payload = proc.stdout
    if isinstance(payload, str):
        payload = payload.encode("utf-8", errors="replace")
    if not isinstance(payload, (bytes, bytearray, memoryview)):
        payload = b""
    return sha256_bytes(bytes(payload) if not isinstance(payload, bytes) else payload)


def probe_capabilities(bin_path, timeout=DIGEST_TIMEOUT):
    """capabilities digest 를 예외 없이 접는다.

    성공: {ok: True, digest, kind: "digest", exit}.
    실패: {ok: False, digest: None, kind, error, head}.
    digest 가 None 이면 classify/surface_changed 에 넣지 않는다.
    """
    try:
        if not bin_path:
            raise FileNotFoundError("바이너리 경로가 비어 있다")
        kwargs = {"cwd": ROOT, "capture_output": True}
        limit = normalize_timeout(timeout)
        if limit:
            kwargs["timeout"] = limit
        proc = subprocess.run([bin_path, "capabilities"], **kwargs)
        payload = proc.stdout
        if isinstance(payload, str):
            payload = payload.encode("utf-8", errors="replace")
        if not isinstance(payload, (bytes, bytearray, memoryview)):
            payload = b""
        digest = sha256_bytes(bytes(payload) if not isinstance(payload, bytes) else payload)
        return {
            "ok": True,
            "kind": "digest",
            "digest": digest,
            "exit": proc.returncode,
            "error": None,
            "head": "",
        }
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return {
            "ok": False,
            "kind": exception_kind(exc, context="digest"),
            "digest": None,
            "exit": None,
            "error": type(exc).__name__,
            "head": truncate_head(str(exc), ERROR_HEAD_LIMIT),
        }
    except Exception as exc:
        return {
            "ok": False,
            "kind": "unexpected",
            "digest": None,
            "exit": None,
            "error": type(exc).__name__,
            "head": truncate_head(str(exc), ERROR_HEAD_LIMIT),
        }


def surface_changed(old_digest, new_digest):
    """capabilities digest 가 다르면 표면이 바뀐 것이다. 순수.

    None 과 문자열을 비교하면 True 가 된다. 프로브 실패의 None 을 여기에
    넣지 말라 — can_classify_surface 가 막는다. classify 는 이 함수의
    결과만 보고 삼원을 고른다.
    """
    return old_digest != new_digest


def can_classify_surface(old_digest, new_digest):
    """두 digest 가 모두 재졌을 때만 분류할 수 있다. 순수.

    한쪽이라도 None/비문자면 표면을 모르는 것이지 표면이 바뀐 것이 아니다.
    """
    return isinstance(old_digest, str) and isinstance(new_digest, str)


def classify(surface, divergences):
    """오검출 관문. 표면 변경이 회귀보다 앞선다.

    divergences 는 분기 목록·건수·bool 모두 받는다. 표면이 바뀌면 분기 유무와
    무관하게 surface-changed — 의도된 명령 추가를 회귀로 오신고하지 않는다.

    이 함수는 probe-failed 를 내지 않는다. 표면을 모를 때는 호출하지 말라.
    """
    if surface:
        return "surface-changed"
    if divergences:
        return "regression"
    return "stable"


def classify_or_probe_failed(old_digest, new_digest, divergences):
    """분류 가능하면 classify, 아니면 probe-failed. classify 자체는 건드리지 않는다."""
    if not can_classify_surface(old_digest, new_digest):
        return STATUS_PROBE_FAILED
    return classify(surface_changed(old_digest, new_digest), divergences)


def exit_for(classification):
    """stable=0, surface-changed=2(사람 판정), regression=3(회귀).

    probe-failed 는 이 표에 없다. status_exit 를 쓴다.
    """
    return EXIT_BY_CLASS[classification]


def status_exit(status):
    """보고 상태의 종료 코드. 분류 삼원은 EXIT_BY_CLASS, 도구 실패는 1."""
    if status == STATUS_PROBE_FAILED:
        return EXIT_PROBE_FAILED
    return exit_for(status)


def reason_for(status):
    if status == STATUS_PROBE_FAILED:
        return PROBE_FAILED_REASON
    return CLASSIFICATION_REASON[status]


def expected_exits(check):
    return check.get("expect_exits") or [check.get("expect_exit", 0)]


def should_observe(check):
    if not isinstance(check, dict):
        return False
    return check.get("op") not in FILE_OPS


def _values_equal(left, right):
    """숫자 6 과 6.0 은 같고, bool 은 int 로 접히지 않는다."""
    if left is right:
        return True
    if isinstance(left, bool) or isinstance(right, bool):
        return isinstance(left, bool) and isinstance(right, bool) and left is right
    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        # NaN 은 자기 자신과도 같지 않다. 회귀로 오신고하지 않도록 둘 다 NaN 이면 같다.
        if isinstance(left, float) and isinstance(right, float):
            if left != left and right != right:
                return True
        return float(left) == float(right)
    if isinstance(left, str) and isinstance(right, str):
        return left == right
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(
            _values_equal(a, b) for a, b in zip(left, right)
        )
    if isinstance(left, tuple) and isinstance(right, tuple):
        return len(left) == len(right) and all(
            _values_equal(a, b) for a, b in zip(left, right)
        )
    if isinstance(left, dict) and isinstance(right, dict):
        if set(left) != set(right):
            return False
        return all(_values_equal(left[k], right[k]) for k in left)
    return left == right


def observations_equal(left, right):
    """관측 동일성. 종류가 다르면 값이 같아도 같지 않다."""
    return _values_equal(left, right)


def observation_display(obs):
    """사람이 읽는 한 칸. 값 관측은 raw, 그 외는 kind 또는 exitN."""
    if isinstance(obs, dict):
        kind = obs.get("kind")
        if kind == "value":
            return obs.get("value")
        if kind == "exit":
            return f"exit{obs.get('code')}"
        if kind:
            return kind
    return obs


def observation_from_result(code, env, head, check, dig_fn=None, find_cell_fn=None):
    """CLI 결과에서 대조 가능한 관측을 뽑는다. 순수.

    종료 코드·JSON 부재·경로 실패를 kind 로 가른다. 판정이 아니라 값이다.
    dig 경로의 ValueError(비정수 인덱스)도 도구를 죽이지 않고 digfail 로 접는다.
    """
    if code not in expected_exits(check):
        return {"kind": "exit", "code": code, "head": truncate_head(head or "", HEAD_LIMIT)}
    if env is None:
        return {"kind": "nojson", "head": truncate_head(head or "", HEAD_LIMIT)}
    dig_fn = check_registry.dig if dig_fn is None else dig_fn
    try:
        val = dig_fn(env, check.get("path", ""))
    except CATCHABLE_EXCEPTIONS as e:
        return {"kind": "digfail", "error": type(e).__name__}
    except FATAL_EXCEPTIONS:
        raise
    except Exception as e:
        return {"kind": "digfail", "error": type(e).__name__}
    if check.get("op") == "cell_text_eq":
        find_cell_fn = check_registry.find_cell if find_cell_fn is None else find_cell_fn
        try:
            cell = find_cell_fn(val, check["table"], check["row"], check["col"])
        except CATCHABLE_EXCEPTIONS as e:
            return {"kind": "digfail", "error": type(e).__name__}
        except FATAL_EXCEPTIONS:
            raise
        except Exception as e:
            return {"kind": "digfail", "error": type(e).__name__}
        val = None if cell is None else cell.get("text")
    return {"kind": "value", "value": val}


def observe(bin_path, check, task, sub_dir):
    """한 검사의 관측값을 뽑는다 — 봉투의 지목된 자리(raw). 판정이 아니라 값.

    resolve_args / run_cli / 경로 평가 예외는 관측 상태로 접는다. 한 검사가
    실패해도 차등 도구 전체가 멈추지 않는다. KeyboardInterrupt 는 삼키지 않는다.
    """
    if not isinstance(check, dict):
        return {"kind": "type-error", "error": "TypeError", "head": "check 가 dict 가 아니다"}
    cmd = check.get("cmd")
    if not cmd:
        return {"kind": "no-cmd"}
    try:
        args = runner.resolve_args(cmd, task, sub_dir)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as e:
        # 제출물 부재도 구/신 양쪽에서 비교할 수 있는 관측 상태다. 여기서
        # 예외를 내면 legacy baseline이 비어 있는 한 차등 도구 전체가 멈춘다.
        return exception_observation(e, context="resolve")
    except Exception as e:
        return exception_observation(e, context="resolve")
    try:
        code, env, head = runner.run_cli(bin_path, args)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as e:
        return exception_observation(e, context="cli")
    except Exception as e:
        return exception_observation(e, context="cli")
    try:
        return observation_from_result(code, env, head, check)
    except FATAL_EXCEPTIONS:
        raise
    except Exception as e:
        return exception_observation(e, context="observe")


def make_diff_row(task_id, check, old_obs, new_obs):
    op = ""
    name = ""
    path = ""
    if isinstance(check, dict):
        op = check.get("op", "")
        name = check.get("name", op)
        path = check.get("path", "")
    return {
        "task": task_id,
        "check": name,
        "op": op,
        "path": path,
        "old": old_obs,
        "new": new_obs,
    }


def resolve_submission_dir(sub_root, pack_id, task_id):
    """pack/과제 제출 폴더. 평면 제출 호환. 예외를 올리지 않는다."""
    if not sub_root or not task_id:
        return ""
    packed = os.path.join(sub_root, pack_id, task_id) if pack_id else ""
    try:
        if packed and os.path.isdir(packed):
            return packed
    except OSError:
        packed = ""
    flat = os.path.join(sub_root, task_id)
    try:
        if os.path.isdir(flat):
            return flat
    except OSError:
        return packed or flat
    return packed or flat


def diff_task(old_bin, new_bin, task, sub_root, pack_id):
    if not isinstance(task, dict):
        return []
    task_id = task.get("id", "")
    sub_dir = resolve_submission_dir(sub_root, pack_id, task_id)
    rows = []
    checks = task.get("checks", [])
    if not isinstance(checks, list):
        return []
    for check in checks:
        try:
            if not should_observe(check):
                continue
            o = observe(old_bin, check, task, sub_dir)
            n = observe(new_bin, check, task, sub_dir)
            if not observations_equal(o, n):
                rows.append(make_diff_row(task_id, check, o, n))
        except FATAL_EXCEPTIONS:
            raise
        except Exception:
            # 한 검사의 예외는 행을 건너뛴다. 도구를 죽이지 않되 거짓 분기도 만들지 않는다.
            continue
    return rows


def count_observable_checks(task):
    """관측 대상 검사 수와 건너뛴(파일 연산) 수. 순수."""
    if not isinstance(task, dict):
        return 0, 0
    checks = task.get("checks", [])
    if not isinstance(checks, list):
        return 0, 0
    seen = 0
    skipped = 0
    for check in checks:
        if should_observe(check):
            seen += 1
        else:
            skipped += 1
    return seen, skipped


def load_pack_safe(pack_id):
    """pack 하나를 읽는다. 실패하면 (None, [], error) — 전체를 멈추지 않는다."""
    try:
        manifest, tasks = runner.load_pack(pack_id)
        if not isinstance(tasks, list):
            return manifest, [], f"pack {pack_id}: tasks 가 list 가 아니다"
        return manifest, tasks, None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return None, [], f"pack {pack_id}: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"
    except Exception as exc:
        return None, [], f"pack {pack_id}: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"


def discover_packs_safe(explicit):
    """--pack 목록 또는 탐색. 탐색 예외는 빈 목록 + 오류 문자열."""
    if explicit:
        return list(explicit), None
    try:
        return list(runner.discover_packs()), None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return [], f"discover_packs: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"
    except Exception as exc:
        return [], f"discover_packs: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"


def build_report(old_bin, old_digest, new_bin, new_digest,
                 tasks_compared, observations_compared, diffs,
                 observations_skipped=0):
    """릴리스 차등 JSON 봉투. 순수 — 바이너리를 부르지 않는다.

    digest 가 둘 다 문자열일 때만 classify 한다. 한쪽이라도 없으면
    probe-failed 로 두고 삼원 분류를 위장하지 않는다.
    """
    if not can_classify_surface(old_digest, new_digest):
        return build_probe_failed_report(
            old_bin, {"digest": old_digest, "ok": False, "kind": "missing-digest",
                      "error": "digest-missing", "head": ""},
            new_bin, {"digest": new_digest, "ok": False, "kind": "missing-digest",
                      "error": "digest-missing", "head": ""},
            tasks_compared=tasks_compared,
            observations_compared=observations_compared,
            observations_skipped=observations_skipped,
            diffs=diffs,
        )
    surface = surface_changed(old_digest, new_digest)
    classification = classify(surface, diffs)
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "old": {"bin": os.path.basename(old_bin), "capabilitiesSha256": old_digest},
        "new": {"bin": os.path.basename(new_bin), "capabilitiesSha256": new_digest},
        "surfaceChanged": surface,
        "tasksCompared": tasks_compared,
        "observationsCompared": observations_compared,
        "observationsSkipped": observations_skipped,
        "divergences": len(diffs) if hasattr(diffs, "__len__") else int(bool(diffs)),
        "classification": classification,
        "classificationReason": CLASSIFICATION_REASON[classification],
        "exit": exit_for(classification),
        "ok": classification == "stable",
        "reviewRequired": classification == "surface-changed",
        "diffs": list(diffs),
    }


def build_probe_failed_report(old_bin, old_probe, new_bin, new_probe,
                              tasks_compared=0, observations_compared=0,
                              observations_skipped=0, diffs=None):
    """표면을 재지 못했을 때의 봉투. classification 은 probe-failed.

    ok/reviewRequired/surfaceChanged 는 모두 거짓이다. 모르는 것을 안정·회귀·
    표면변경으로 부르지 않는다.
    """
    diffs = list(diffs or [])
    old_probe = old_probe or {}
    new_probe = new_probe or {}
    errors = []
    if not old_probe.get("ok"):
        errors.append({
            "role": "old",
            "bin": os.path.basename(old_bin) if old_bin else "",
            "kind": old_probe.get("kind") or "unexpected",
            "error": old_probe.get("error") or "digest-missing",
            "head": old_probe.get("head") or "",
        })
    if not new_probe.get("ok"):
        errors.append({
            "role": "new",
            "bin": os.path.basename(new_bin) if new_bin else "",
            "kind": new_probe.get("kind") or "unexpected",
            "error": new_probe.get("error") or "digest-missing",
            "head": new_probe.get("head") or "",
        })
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "old": {
            "bin": os.path.basename(old_bin) if old_bin else "",
            "capabilitiesSha256": old_probe.get("digest"),
        },
        "new": {
            "bin": os.path.basename(new_bin) if new_bin else "",
            "capabilitiesSha256": new_probe.get("digest"),
        },
        "surfaceChanged": False,
        "tasksCompared": tasks_compared,
        "observationsCompared": observations_compared,
        "observationsSkipped": observations_skipped,
        "divergences": len(diffs),
        "classification": STATUS_PROBE_FAILED,
        "classificationReason": PROBE_FAILED_REASON,
        "exit": EXIT_PROBE_FAILED,
        "ok": False,
        "reviewRequired": False,
        "diffs": diffs,
        "probeFailed": True,
        "probeErrors": errors,
    }


def empty_report(old_bin="", new_bin=""):
    """대조 전 빈 봉투. digest 가 없어 probe-failed 가 정직하다."""
    return build_probe_failed_report(
        old_bin, {"ok": False, "kind": "missing-digest", "error": "empty", "head": ""},
        new_bin, {"ok": False, "kind": "missing-digest", "error": "empty", "head": ""},
    )


def validate_report(report):
    """보고 봉투의 정직 계약. 문제 문자열 목록(비면 통과).

    classify 삼원일 때:
      - exit 는 EXIT_BY_CLASS 와 같다.
      - ok 는 stable 과만 같다.
      - reviewRequired 는 surface-changed 와만 같다.
      - surfaceChanged 는 분류가 surface-changed 일 때 참, 그 외 거짓.
      - probeFailed 가 참이면 안 된다.
    probe-failed 일 때:
      - exit 는 1.
      - ok / reviewRequired / surfaceChanged 는 거짓.
      - 분류는 CLASSIFICATIONS 에 없다.
    """
    issues = []
    if not isinstance(report, dict):
        return ["report 가 dict 가 아니다"]
    for key in REPORT_KEYS:
        if key not in report:
            issues.append(f"키 없음: {key}")
    if report.get("kind") != REPORT_KIND:
        issues.append(f"kind 가 {REPORT_KIND} 가 아니다")
    if report.get("schemaVersion") != SCHEMA_VERSION:
        issues.append(f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다")
    classification = report.get("classification")
    diffs = report.get("diffs")
    if isinstance(diffs, list):
        if report.get("divergences") != len(diffs):
            issues.append("divergences 가 diffs 길이와 다르다")
    else:
        issues.append("diffs 가 list 가 아니다")
    if classification in CLASSIFICATIONS:
        if report.get("exit") != EXIT_BY_CLASS[classification]:
            issues.append(f"exit 가 {classification} 계약과 다르다")
        if report.get("ok") != (classification == "stable"):
            issues.append("ok 는 stable 과만 같아야 한다")
        if report.get("reviewRequired") != (classification == "surface-changed"):
            issues.append("reviewRequired 는 surface-changed 와만 같아야 한다")
        if report.get("surfaceChanged") != (classification == "surface-changed"):
            issues.append("surfaceChanged 는 분류와 어긋난다")
        if report.get("probeFailed"):
            issues.append("삼원 분류에 probeFailed 가 붙으면 안 된다")
        if classification == "regression" and not report.get("divergences"):
            issues.append("regression 인데 divergences 가 0 이다")
        if classification == "stable" and report.get("divergences"):
            issues.append("stable 인데 divergences 가 있다")
    elif classification == STATUS_PROBE_FAILED:
        if report.get("exit") != EXIT_PROBE_FAILED:
            issues.append("probe-failed 의 exit 는 1 이어야 한다")
        if report.get("ok"):
            issues.append("probe-failed 를 ok 로 위장하면 안 된다")
        if report.get("reviewRequired"):
            issues.append("probe-failed 를 reviewRequired 로 위장하면 안 된다")
        if report.get("surfaceChanged"):
            issues.append("probe-failed 를 surfaceChanged 로 위장하면 안 된다")
        if not report.get("probeFailed"):
            issues.append("probe-failed 봉투에 probeFailed 표지가 없다")
    else:
        issues.append(f"알 수 없는 classification: {classification!r}")
    return issues


def write_report(report, path):
    """UTF-8 · BOM 없음 · LF. 같은 입력이면 바이트가 같다."""
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(report, ensure_ascii=False, indent=2))
        fh.write("\n")


def write_report_safe(report, path):
    """쓰기 예외를 접는다. 성공이면 None, 실패면 오류 문자열."""
    try:
        write_report(report, path)
        return None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return f"write_report: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"
    except Exception as exc:
        return f"write_report: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"


def render_summary(report, out_path):
    surface = report.get("surfaceChanged")
    classification = report.get("classification")
    if classification == STATUS_PROBE_FAILED:
        lines = [
            f"과제 {report.get('tasksCompared', 0)} · 관측 대조 {report.get('observationsCompared', 0)}건",
            "명령 표면(capabilities): 프로브 실패 → 분류하지 않음",
            f"관측 분기: {report.get('divergences', 0)}건 → 상태 [{classification}]",
            f"이유: {report.get('classificationReason', PROBE_FAILED_REASON)}",
        ]
        for err in report.get("probeErrors") or []:
            lines.append(
                f"  프로브[{err.get('role')}]: {err.get('kind')} · {err.get('error')}"
            )
    else:
        lines = [
            f"과제 {report.get('tasksCompared', 0)} · 관측 대조 {report.get('observationsCompared', 0)}건",
            f"명령 표면(capabilities): {'다름 → surface-changed' if surface else '같음'}",
            f"관측 분기: {report.get('divergences', 0)}건 → 분류 [{classification}]",
            f"이유: {report.get('classificationReason', '')}",
        ]
    for row in (report.get("diffs") or [])[:30]:
        ov = observation_display(row.get("old"))
        nv = observation_display(row.get("new"))
        pack = row.get("pack", "")
        loc = f"{pack}/{row.get('task')}" if pack else row.get("task")
        lines.append(f"  {loc} · {row.get('check')}: {ov!r} → {nv!r}")
    if report.get("packErrors"):
        for err in report["packErrors"][:10]:
            lines.append(f"  pack 오류: {err}")
    if report.get("writeError"):
        lines.append(f"  쓰기 오류: {report['writeError']}")
    lines.append(f"→ {out_path}")
    return lines


def find_bin_safe(path):
    """runner.find_bin 을 예외 없이 접는다."""
    try:
        found = runner.find_bin(path)
        return found, None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return path, f"find_bin: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"
    except Exception as exc:
        return path, f"find_bin: {type(exc).__name__}: {truncate_head(str(exc), ERROR_HEAD_LIMIT)}"


def compare_packs(old_bin, new_bin, pack_ids, sub_root):
    """pack 목록을 돌며 분기 행과 집계를 모은다. 한 pack 실패는 나머지를 남긴다."""
    diffs, tasks_seen, checks_seen, skipped = [], 0, 0, 0
    pack_errors = []
    task_errors = []
    for pack_id in pack_ids:
        _manifest, tasks, err = load_pack_safe(pack_id)
        if err:
            pack_errors.append(err)
            continue
        for task in tasks:
            try:
                tasks_seen += 1
                seen, skip = count_observable_checks(task)
                checks_seen += seen
                skipped += skip
                for row in diff_task(old_bin, new_bin, task, sub_root, pack_id):
                    row["pack"] = pack_id
                    diffs.append(row)
            except FATAL_EXCEPTIONS:
                raise
            except Exception as exc:
                tid = task.get("id") if isinstance(task, dict) else "?"
                task_errors.append(
                    f"{pack_id}/{tid}: {type(exc).__name__}: "
                    f"{truncate_head(str(exc), ERROR_HEAD_LIMIT)}"
                )
    return {
        "diffs": diffs,
        "tasksCompared": tasks_seen,
        "observationsCompared": checks_seen,
        "observationsSkipped": skipped,
        "packErrors": pack_errors,
        "taskErrors": task_errors,
    }


def attach_collection_errors(report, collection):
    """팩/과제 오류를 보고에 붙인다. 분류는 바꾸지 않는다."""
    if collection.get("packErrors"):
        report["packErrors"] = list(collection["packErrors"])
    if collection.get("taskErrors"):
        report["taskErrors"] = list(collection["taskErrors"])
    return report


def parse_args(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--old", required=True, help="구 rhwp 바이너리 경로")
    ap.add_argument("--new", required=True, help="신 rhwp 바이너리 경로")
    ap.add_argument("--agent", default="claude-fable-5", help="관측에 쓸 제출물")
    ap.add_argument("--pack", action="append", default=None)
    ap.add_argument("-o", "--out", default=None)
    ap.add_argument("--digest-timeout", type=int, default=DIGEST_TIMEOUT,
                    help="capabilities 프로브 초(기본 30, 0 이하는 무제한)")
    return ap.parse_args(argv)


def main(argv=None):
    a = parse_args(argv)

    old_bin, old_find_err = find_bin_safe(a.old)
    new_bin, new_find_err = find_bin_safe(a.new)

    timeout = normalize_timeout(getattr(a, "digest_timeout", DIGEST_TIMEOUT)) or None
    old_probe = probe_capabilities(old_bin, timeout=timeout if timeout else DIGEST_TIMEOUT)
    new_probe = probe_capabilities(new_bin, timeout=timeout if timeout else DIGEST_TIMEOUT)
    if old_find_err and old_probe.get("ok"):
        old_probe = {
            "ok": False,
            "kind": "os-error",
            "digest": None,
            "exit": None,
            "error": "find_bin",
            "head": old_find_err,
        }
    if new_find_err and new_probe.get("ok"):
        new_probe = {
            "ok": False,
            "kind": "os-error",
            "digest": None,
            "exit": None,
            "error": "find_bin",
            "head": new_find_err,
        }

    sub_root = os.path.join(runner.GYM, "submissions", a.agent)
    pack_ids, discover_err = discover_packs_safe(a.pack)
    collection = compare_packs(old_bin, new_bin, pack_ids, sub_root)
    if discover_err:
        collection["packErrors"] = [discover_err] + list(collection.get("packErrors") or [])

    if not (old_probe.get("ok") and new_probe.get("ok")):
        report = build_probe_failed_report(
            old_bin, old_probe, new_bin, new_probe,
            tasks_compared=collection["tasksCompared"],
            observations_compared=collection["observationsCompared"],
            observations_skipped=collection["observationsSkipped"],
            diffs=collection["diffs"],
        )
    else:
        report = build_report(
            old_bin, old_probe["digest"], new_bin, new_probe["digest"],
            collection["tasksCompared"], collection["observationsCompared"],
            collection["diffs"],
            observations_skipped=collection["observationsSkipped"],
        )
    attach_collection_errors(report, collection)

    out = a.out or os.path.join(runner.GYM, "release-diff.json")
    write_err = write_report_safe(report, out)
    if write_err:
        report["writeError"] = write_err

    for line in render_summary(report, out):
        print(line)
    return report["exit"]


if __name__ == "__main__":
    sys.exit(main())
