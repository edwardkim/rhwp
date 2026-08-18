"""[운동장 차등 오라클] 같은 문서의 두 형식이 같은 답을 내는가.

## 착상

운동장 채점기는 정답을 골든 파일로 박제하지 않는다 — 기대값을 **채점 시점에
rhwp 로 재계산**한다. 이 성질에는 아직 쓰지 않은 쓸모가 하나 있다:

> 같은 문서의 HWP 판과 HWPX 판에 **같은 관측**을 물리면 답이 같아야 한다.
> 다르면, 둘 중 하나의 읽기 경로가 틀린 것이다.

즉 골든 파일 없이도 **차등(differential) 테스트**가 성립한다. 사람이 기대값을
적어둔 자리에서만 회귀를 잡는 보통의 테스트와 달리, 이 방식은 **아무도 기대값을
적어두지 않은 자리**까지 훑는다. 저장소에 쌍둥이 픽스처가 139쌍 있으므로
관측을 N개 얹으면 즉시 139×N 개의 판정이 생긴다.

## 오검출을 막는 관문 (이것이 없으면 도구가 거짓말을 한다)

같은 이름의 두 파일이 **실제로 같은 문서라는 보장이 없다**(개정판을 각각 저장한
경우가 있다). 그래서 관측이 어긋난 쌍은 곧바로 결함으로 부르지 않고 두 관문을
통과시킨다.

1. **본문 동일성** — 공백을 무시한 본문이 바이트로 같아야 한다. 다르면 그냥
   다른 문서다(결함 아님). 본문 해시를 못 구하면 동일 문서로 치지 않는다.
2. **IR 동일성** — `ir-diff` 가 `identical: true` 를 내야 한다. rhwp 자신이
   "두 문서의 구조는 같다" 고 말한 뒤에도 관측이 어긋난다면, 그것은 **내부
   모순**이고 결함 후보다.

이 관문을 세운 실측 근거: 표본 25쌍에서 어긋난 2건 중 1건은 본문 해시부터
달랐다(다른 개정판 — 결함 아님), 나머지 1건은 IR 동일 판정에도 쪽수가 달랐다
(진짜 후보).

보고 봉투: `kind=gymDifferential`, `schemaVersion=1.0`. 판정·집계·보고는 순수
함수라 `scripts/tests/test_gym_differential.py` 가 바이너리 없이 고정한다.

## 예외 경로 (도구가 한 쌍의 CLI 실패로 죽지 않도록)

CLI 부재·권한·시간초과·디코드 실패는 관측 kind 로 접는다. 한 쌍의 예외가
전수 스윕을 멈추게 하지 않는다. 다만 접는 자리에서도 **짝짓기·해시 정직
조항은 그대로**다.

- 본문 해시를 예외로 못 구하면 `None` 이다. `None == None` 을 동일 문서로
  부르지 않는다.
- `ir-diff` 가 예외로 죽으면 `identical=False` 다. IR 을 못 봤는데
  `contradiction` 이라고 부르지 않는다.
- 치명 예외(`KeyboardInterrupt` · `SystemExit` · `MemoryError` ·
  `GeneratorExit`)는 삼키지 않는다. 사용자가 끊었는데 모순 0건이라고 쓰면
  거짓말이다.

사용:
  python gym/tools/differential.py [--limit N] [--bin <경로>] [-o 결과.json]
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

ROOT = runner.ROOT

REPORT_KIND = "gymDifferential"
SCHEMA_VERSION = "1.0"
TWIN_EXTS = (".hwp", ".hwpx")

#: 두 형식에서 같아야 하는 관측. (이름, 인자 템플릿, 봉투 경로)
OBSERVATIONS = [
    ("pageCount", ["info", "{f}", "--json"], "pageCount"),
    ("tableCount", ["export-tables", "{f}", "--json"], "tableCount"),
    ("paragraphCount", ["explain", "{f}", "--json"], "paragraphCount"),
    ("fieldCount", ["fields", "{f}", "--json"], "fieldCount"),
    ("footnoteCount", ["explain", "{f}", "--json"], "footnoteCount"),
    ("endnoteCount", ["explain", "{f}", "--json"], "endnoteCount"),
]

#: 관측 kind 카탈로그. 시험과 문서가 같은 표를 본다.
OBSERVATION_KINDS = (
    "value",
    "nojson",
    "badenv",
    "missing",
    "timeout",
    "missing-bin",
    "permission",
    "os-error",
    "type-error",
    "value-error",
    "decode-error",
    "cli-error",
    "unexpected",
)

#: JSON 보고 고정 키. 분류가 성공한 봉투의 최소 집합.
REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "runner",
    "pairs",
    "observationsCompared",
    "sameNameDifferentDocument",
    "findings",
    "contradictions",
    "reviews",
)

#: 부가 키. 있어도 짝짓기·해시·심각도를 뒤집지 않는다.
OPTIONAL_REPORT_KEYS = (
    "toolFailed",
    "toolErrors",
    "pairErrors",
    "writeError",
    "exit",
)

#: 결함 후보의 심각도. other-doc 은 여기에 없다 — 그건 결함이 아니다.
SEVERITIES = ("contradiction", "review")

#: 관문이 내는 라벨. None 은 갈림 없음.
GATE_LABELS = (None, "other-doc", "contradiction", "review")

#: 종료 코드. 0=모순 없음, 1=도구 실패, 3=IR 동일 모순.
EXIT_OK = 0
EXIT_TOOL_FAILED = 1
EXIT_CONTRADICTION = 3

#: 관측 head / 오류 메시지 머리 길이.
HEAD_LIMIT = 80
ERROR_HEAD_LIMIT = 160

#: CLI 기본 초. 0 이하는 시간제한 없음(기존 동작).
CLI_TIMEOUT_DEFAULT = 0

#: 예외 → 관측 kind. context 가 cli/hash/ir 이면 FileNotFound 는 missing-bin.
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
    KeyError: "value-error",
    IndexError: "value-error",
    AttributeError: "type-error",
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


def exception_kind(exc, context="cli"):
    """예외를 관측 kind 로 접는다. 순수.

    context:
      - cli: run_cli 실패. FileNotFound → missing-bin.
      - hash: export-text 본문 해시. 같은 FileNotFound → missing-bin.
      - ir: ir-diff 실패. 같은 FileNotFound → missing-bin.
      - pair: 한 쌍 루프의 그 외 예외.
      - write / walk / find-bin: 도구 자리. 분류를 바꾸지 않는다.
    """
    if exc is None:
        return "unexpected"
    if isinstance(exc, subprocess.TimeoutExpired) or isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, FileNotFoundError):
        return "missing-bin"
    if isinstance(exc, PermissionError):
        return "permission"
    if isinstance(exc, UnicodeError):
        return "decode-error"
    if isinstance(exc, json.JSONDecodeError):
        return "value-error"
    if isinstance(exc, AttributeError):
        return "type-error"
    if isinstance(exc, TypeError):
        return "type-error"
    if isinstance(exc, (KeyError, IndexError)):
        return "value-error"
    if isinstance(exc, ValueError):
        return "value-error"
    if isinstance(exc, OSError):
        return "os-error"
    if isinstance(exc, RuntimeError):
        return "cli-error"
    return "unexpected"


def exception_observation(exc, context="cli", head=""):
    """예외를 관측 봉투로 접는다. 여기서 예외를 다시 올리지 않는다."""
    kind = exception_kind(exc, context=context)
    return {
        "kind": kind,
        "error": type(exc).__name__ if exc is not None else "NoneType",
        "head": truncate_head(head or (str(exc) if exc is not None else ""), ERROR_HEAD_LIMIT),
    }


def exception_tool_error(exc, where, extra=None):
    """도구 자리 오류 한 줄. 심각도·짝짓기를 바꾸지 않는다."""
    row = {
        "where": where,
        "kind": exception_kind(exc, context=where),
        "error": type(exc).__name__ if exc is not None else "NoneType",
        "head": truncate_head(str(exc) if exc is not None else "", ERROR_HEAD_LIMIT),
    }
    if extra:
        row.update(extra)
    return row


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
    """CLI 초. 변환 불능·비양수는 0 — 호출 측에서 시간제한을 끈다."""
    try:
        n = int(timeout)
    except (TypeError, ValueError):
        return 0
    return n if n > 0 else 0


def normalize_limit(limit):
    """--limit. 변환 불능·음수는 0(전부). 순수."""
    try:
        n = int(limit)
    except (TypeError, ValueError):
        return 0
    return n if n > 0 else 0


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
    """값 관측이 아닌 실패/부재 관측인가. 관문을 바꾸지 않는다 — 대조 대상일 뿐이다."""
    kind = observation_kind_of(obs)
    if kind is None:
        return False
    return kind != "value"


def is_sha256_hex(value):
    """64자 소문자/숫자 hex 인가. 순수. None/짧은 값은 거짓."""
    if not isinstance(value, str) or len(value) != 64:
        return False
    try:
        int(value, 16)
    except ValueError:
        return False
    return value == value.lower()


def decode_cli_stdout(raw):
    """subprocess stdout 을 텍스트로. 바이트가 아니면 빈 문자열로 위장하지 않는다."""
    if raw is None:
        return ""
    if isinstance(raw, str):
        return raw
    if isinstance(raw, memoryview):
        raw = raw.tobytes()
    if isinstance(raw, bytearray):
        raw = bytes(raw)
    if not isinstance(raw, bytes):
        raise TypeError(f"decode_cli_stdout 는 bytes/str 만 받습니다 (got {type(raw).__name__})")
    return raw.decode("utf-8")


def loads_cli_json(text):
    """CLI stdout JSON. 비면 None. JSON 이 아니면 ValueError 를 올린다."""
    if text is None:
        return None
    if not isinstance(text, str):
        raise TypeError(f"loads_cli_json 는 str 만 받습니다 (got {type(text).__name__})")
    stripped = text.strip()
    if not stripped:
        return None
    return json.loads(stripped)


def coerce_run_result(result):
    """주입 run() 의 반환을 (code, env) 로. 형식이 아니면 TypeError.

    compare_twins 가 목 run 을 받을 때의 계약. 예외를 여기서 삼키지 않는다.
    """
    if result is None:
        raise TypeError("run() 이 None 을 반환했다")
    if not isinstance(result, (tuple, list)) or len(result) < 2:
        raise TypeError("run() 은 (code, env) 를 반환해야 한다")
    return result[0], result[1]


def run_cli(bin_path, args, timeout=None):
    """rhwp 실행 → (exit, 봉투 json 또는 None).

    timeout 이 양수면 subprocess 에 넘긴다. 예외는 여기서 삼키지 않는다 —
    호출자가 run_cli_safe 로 접는다. JSON 이 아니면 env=None (기존 계약).
    """
    if not bin_path:
        raise FileNotFoundError("바이너리 경로가 비어 있다")
    kwargs = {"cwd": ROOT, "capture_output": True}
    limit = normalize_timeout(timeout) if timeout is not None else 0
    if limit:
        kwargs["timeout"] = limit
    proc = subprocess.run([bin_path] + list(args), **kwargs)
    try:
        text = decode_cli_stdout(proc.stdout)
        return proc.returncode, loads_cli_json(text)
    except ValueError:
        return proc.returncode, None


def run_cli_safe(bin_path, args, timeout=None):
    """run_cli 를 예외 없이 접는다.

    성공: (code, env) — env 는 dict 또는 None.
    실패: (None, exception_observation). code 가 None 이면 실행 자체가 안 된 것.
    """
    try:
        return run_cli(bin_path, args, timeout=timeout)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return None, exception_observation(exc, context="cli")
    except Exception as exc:
        return None, exception_observation(exc, context="cli")


def observation_from_result(code, env, key):
    """CLI 결과에서 대조 가능한 관측을 뽑는다. 순수.

    종료 코드를 문자열로 붙이면 실제 값과 충돌하므로 kind 로 가른다.
    env 가 이미 예외 관측(kind+error)이면 그대로 돌려 한 겹 더 씌우지 않는다.
    """
    if isinstance(env, dict) and env.get("kind") in OBSERVATION_KINDS and env.get("kind") != "value" and "error" in env:
        return env
    if env is None:
        return {"kind": "nojson", "code": code}
    if not isinstance(env, dict):
        return {"kind": "badenv", "code": code}
    if key not in env:
        return {"kind": "missing", "key": key}
    return {"kind": "value", "value": env[key]}


def observations_equal(left, right):
    """관측 동일성. 숫자 6 과 6.0 은 같고, bool 은 int 로 접히지 않는다."""
    return _values_equal(left, right)


def _values_equal(left, right):
    if left is right:
        return True
    if isinstance(left, bool) or isinstance(right, bool):
        return isinstance(left, bool) and isinstance(right, bool) and left is right
    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        # NaN 은 자기 자신과도 같지 않다. 차등으로 오신고하지 않도록 둘 다 NaN 이면 같다.
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


def observation_display(obs):
    """사람이 읽는 한 칸. 값 관측은 raw, 실패는 exitN."""
    if isinstance(obs, dict):
        kind = obs.get("kind")
        if kind == "value":
            return obs.get("value")
        if kind == "nojson":
            return f"exit{obs.get('code')}"
        if kind == "missing":
            return None
        if kind:
            return kind
    return obs


def pages_text(env):
    """export-text 봉투에서 쪽 본문을 이어 붙인다. 봉투가 아니면 None."""
    if not isinstance(env, dict):
        return None
    pages = env.get("pages")
    if not isinstance(pages, list):
        return None
    parts = []
    for page in pages:
        if isinstance(page, dict):
            parts.append(page.get("text") or "")
    return "".join(parts)


def normalize_body(text):
    """공백(개행·탭 포함)을 무시하는 본문."""
    if text is None:
        return None
    if not isinstance(text, str):
        try:
            text = str(text)
        except Exception:
            return None
    return "".join(text.split())


def hash_text(text):
    """UTF-8 SHA-256 16진. text 가 None 이면 None — 빈 문자열로 위장하지 않는다."""
    if text is None:
        return None
    if not isinstance(text, str):
        raise TypeError(f"hash_text 는 str 만 받습니다 (got {type(text).__name__})")
    return hashlib.sha256(text.encode("utf-8", "replace")).hexdigest()


def body_hash_from_env(env):
    """공백 무시 본문 해시. 봉투가 아니면 None — 없음은 동일이 아니다."""
    text = pages_text(env)
    if text is None:
        return None
    return hash_text(normalize_body(text))


def same_body_hash(left, right):
    """양쪽이 모두 재진 해시이고 같을 때만 참. None==None 은 거짓."""
    return left is not None and right is not None and left == right


def ir_identity(env):
    """(identical, diffCount). 봉투가 없으면 identical=False."""
    if not isinstance(env, dict):
        return False, None
    return bool(env.get("identical")), env.get("diffCount")


def classify_pair(body_same, ir_identical, diverged):
    """관측이 갈린 쌍의 심각도. 갈림 없으면 None, 다른 문서면 other-doc.

    이 함수는 예외를 관측으로 접지 않는다. 호출자가 본문 해시·IR 을 정직하게
    넣어야 한다. 해시를 못 구했으면 body_same=False 로 넣을 것.
    IR 을 못 구했으면 ir_identical=False 로 넣을 것.
    """
    if not diverged:
        return None
    if not body_same:
        return "other-doc"
    return "contradiction" if ir_identical else "review"


def diverged_rows(observed):
    """(label, hwp, hwpx) 목록에서 어긋난 행만, 관측 이름 순."""
    rows = [
        {"observation": label, "hwp": left, "hwpx": right}
        for label, left, right in observed
        if not observations_equal(left, right)
    ]
    rows.sort(key=lambda row: row["observation"])
    return rows


def make_finding(stem, hwp, hwpx, ir_identical, ir_diff_count, diverged):
    return {
        "stem": stem,
        "hwp": hwp,
        "hwpx": hwpx,
        "irIdentical": ir_identical,
        "irDiffCount": ir_diff_count,
        "diverged": list(diverged),
        "severity": "contradiction" if ir_identical else "review",
    }


def finding_severity(ir_identical):
    """IR 동일일 때만 contradiction. IR 을 못 봤으면 review 가 정직하다."""
    return "contradiction" if ir_identical else "review"


def path_rank(rel):
    """같은 줄기에서 대표 경로를 고르는 키 — 얕은 경로, 그다음 사전순."""
    if not isinstance(rel, str):
        rel = "" if rel is None else str(rel)
    norm = rel.replace("\\", "/")
    return (norm.count("/"), norm)


def _string_paths(items):
    """짝짓기에 쓸 경로만. 비문자·빈 값은 순위에서 뺀다. 순서는 호출자가 정한다."""
    out = []
    for item in items or ():
        if isinstance(item, str) and item:
            out.append(item)
    return out


def pick_twin_paths(hwps, hwpxs):
    """한 줄기에 파일이 여러 개일 때 대표 HWP/HWPX 를 결정적으로 고른다.

    같은 디렉터리에 양쪽이 있으면 그 짝(디렉터리 경로가 앞선 것)을 쓰고,
    없으면 얕고 사전순인 경로를 고른다. walk 순서에 의존하지 않는다.

    비문자 경로는 순위에서 뺀다. 남는 유효 경로의 순위 규칙은 그대로다.
    """
    hwps = sorted(_string_paths(hwps), key=path_rank)
    hwpxs = sorted(_string_paths(hwpxs), key=path_rank)
    if not hwps or not hwpxs:
        return None
    hwp_by_dir = {}
    for path in hwps:
        hwp_by_dir.setdefault(os.path.dirname(path.replace("\\", "/")), path)
    hwpx_by_dir = {}
    for path in hwpxs:
        hwpx_by_dir.setdefault(os.path.dirname(path.replace("\\", "/")), path)
    local = sorted(set(hwp_by_dir) & set(hwpx_by_dir))
    if local:
        directory = local[0]
        return hwp_by_dir[directory], hwpx_by_dir[directory]
    return hwps[0], hwpxs[0]


def is_dir_safe(path):
    """os.path.isdir 을 OSError 없이. 예외면 디렉터리가 아니다."""
    try:
        return os.path.isdir(path)
    except OSError:
        return False


def find_twins_in(samples_dir, root=None):
    """(stem, hwp, hwpx) 목록. 줄기 사전순. 상대경로는 `/` 로 정규화.

    디렉터리가 없거나 walk 가 OSError 로 죽으면 빈 목록 — 없는 쌍을
    지어내지 않는다.
    """
    if not is_dir_safe(samples_dir):
        return []
    base = samples_dir if root is None else root
    seen = {}
    try:
        walker = os.walk(samples_dir)
    except OSError:
        return []
    try:
        for dirpath, dirnames, files in walker:
            try:
                dirnames.sort()
            except Exception:
                pass
            try:
                names = sorted(files)
            except Exception:
                names = list(files)
            for name in names:
                try:
                    stem, ext = os.path.splitext(name)
                except Exception:
                    continue
                ext_l = ext.lower() if isinstance(ext, str) else ""
                if ext_l not in TWIN_EXTS:
                    continue
                try:
                    rel = os.path.relpath(os.path.join(dirpath, name), base).replace("\\", "/")
                except (OSError, ValueError, TypeError):
                    continue
                seen.setdefault(stem, {}).setdefault(ext_l, []).append(rel)
    except OSError:
        return []
    pairs = []
    for stem, by_ext in seen.items():
        picked = pick_twin_paths(by_ext.get(".hwp", []), by_ext.get(".hwpx", []))
        if picked:
            pairs.append((stem, picked[0], picked[1]))
    pairs.sort(key=lambda item: (item[0], item[1], item[2]))
    return pairs


def find_twins():
    return find_twins_in(os.path.join(ROOT, "samples"), root=ROOT)


def find_twins_safe(samples_dir=None, root=None):
    """짝 탐색을 예외 없이 접는다. 실패하면 ([], error)."""
    try:
        if samples_dir is None:
            pairs = find_twins()
        else:
            pairs = find_twins_in(samples_dir, root=root)
        return pairs, None
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return [], exception_tool_error(exc, "walk")
    except Exception as exc:
        return [], exception_tool_error(exc, "walk")


def select_pairs(pairs, limit):
    """limit<=0 이면 전부. 앞부분은 정렬된 입력의 접두라 결정적이다."""
    try:
        items = list(pairs)
    except TypeError:
        return []
    n = normalize_limit(limit)
    if not n:
        return items
    return items[:n]


def observe(bin_path, path, args, key, timeout=None):
    code, env = run_cli_safe(bin_path, [a.replace("{f}", path) for a in args], timeout=timeout)
    return observation_from_result(code, env, key)


def observe_with_run(run, path, args, key):
    """주입 run 으로 한 관측. 예외는 관측으로 접는다. 치명 예외는 올린다."""
    try:
        rendered = [a.replace("{f}", path) for a in args]
        code, env = coerce_run_result(run(rendered))
        return observation_from_result(code, env, key)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        return exception_observation(exc, context="cli")
    except Exception as exc:
        return exception_observation(exc, context="cli")


def body_hash(bin_path, path, timeout=None):
    """공백을 무시한 본문의 해시 — 두 파일이 같은 문서인지 가르는 1차 관문.

    CLI 예외는 None. 없음은 동일이 아니다.
    """
    try:
        _code, env = run_cli(bin_path, ["export-text", path, "--json"], timeout=timeout)
        return body_hash_from_env(env)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS:
        return None
    except Exception:
        return None


def body_hash_with_run(run, path):
    """주입 run 으로 본문 해시. 예외·형식 오류는 None — 동일 문서로 위장하지 않는다."""
    try:
        _code, env = coerce_run_result(run(["export-text", path, "--json"]))
        return body_hash_from_env(env)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS:
        return None
    except Exception:
        return None


def ir_identity_with_run(run, hwp, hwpx):
    """주입 run 으로 IR 동일성. 예외면 (False, None) — contradiction 으로 위장하지 않는다."""
    try:
        _code, env = coerce_run_result(run(["ir-diff", hwp, hwpx, "--json"]))
        return ir_identity(env)
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS:
        return False, None
    except Exception:
        return False, None


def compare_twins(pairs, run, observations=None, errors_out=None):
    """쌍 목록을 대조한다. 순수 — run(args)->(code, env) 만 주입하면 된다.

    반환: (관측대조건수, 이름만같은다른문서, findings)

    한 쌍의 예외는 그 쌍만 건너뛴다. 본문 해시를 못 구하면 other-doc.
    IR 을 못 구하면 review. contradiction 은 irIdentical=True 일 때만.
    """
    detailed = compare_twins_detailed(pairs, run, observations=observations)
    if errors_out is not None:
        errors_out.extend(detailed["pairErrors"])
    return detailed["compared"], detailed["other_doc"], detailed["findings"]


def compare_twins_detailed(pairs, run, observations=None):
    """compare_twins 의 상세 봉투. 3튜플 계약을 깨지 않으려고 갈라 둔다."""
    observations = OBSERVATIONS if observations is None else observations
    findings = []
    other_doc = 0
    compared = 0
    pair_errors = []
    skipped_pairs = 0
    try:
        pair_list = list(pairs)
    except TypeError:
        return {
            "compared": 0,
            "other_doc": 0,
            "findings": [],
            "pairErrors": [exception_tool_error(TypeError("pairs"), "pair")],
            "skippedPairs": 0,
        }
    for item in pair_list:
        try:
            stem, hwp, hwpx = item
        except (TypeError, ValueError) as exc:
            pair_errors.append(exception_tool_error(exc, "pair", extra={"stem": "?"}))
            skipped_pairs += 1
            continue
        try:
            observed = []
            for label, args, key in observations:
                left = observe_with_run(run, hwp, args, key)
                right = observe_with_run(run, hwpx, args, key)
                compared += 1
                observed.append((label, left, right))
            diverged = diverged_rows(observed)
            if not diverged:
                continue
            ha = body_hash_with_run(run, hwp)
            hb = body_hash_with_run(run, hwpx)
            if not same_body_hash(ha, hb):
                other_doc += 1
                continue
            identical, diff_count = ir_identity_with_run(run, hwp, hwpx)
            findings.append(make_finding(stem, hwp, hwpx, identical, diff_count, diverged))
        except FATAL_EXCEPTIONS:
            raise
        except CATCHABLE_EXCEPTIONS as exc:
            pair_errors.append(exception_tool_error(exc, "pair", extra={"stem": stem}))
            skipped_pairs += 1
        except Exception as exc:
            pair_errors.append(exception_tool_error(exc, "pair", extra={"stem": stem}))
            skipped_pairs += 1
    findings.sort(key=lambda row: (row["stem"], row["hwp"], row["hwpx"]))
    return {
        "compared": compared,
        "other_doc": other_doc,
        "findings": findings,
        "pairErrors": pair_errors,
        "skippedPairs": skipped_pairs,
    }


def build_report(*, bin_name, pairs_count, compared, other_doc, findings,
                 tool_errors=None, pair_errors=None, write_error=None):
    ordered = sorted(findings, key=lambda row: (row.get("stem") or "", row.get("hwp") or ""))
    contradictions = sum(1 for row in ordered if row.get("severity") == "contradiction")
    reviews = sum(1 for row in ordered if row.get("severity") == "review")
    tool_errors = list(tool_errors or [])
    pair_errors = list(pair_errors or [])
    tool_failed = bool(tool_errors)
    report = {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": contradictions == 0,
        "runner": {"bin": bin_name},
        "pairs": pairs_count,
        "observationsCompared": compared,
        "sameNameDifferentDocument": other_doc,
        "findings": ordered,
        "contradictions": contradictions,
        "reviews": reviews,
        "exit": EXIT_TOOL_FAILED if tool_failed else (EXIT_OK if contradictions == 0 else EXIT_CONTRADICTION),
        "toolFailed": tool_failed,
    }
    if tool_errors:
        report["toolErrors"] = tool_errors
    if pair_errors:
        report["pairErrors"] = pair_errors
    if write_error:
        report["writeError"] = write_error
    return report


def status_exit(report):
    """보고 봉투의 종료 코드. 도구 실패가 모순보다 앞선다.

    도구가 죽었는데 모순 0건이라고 exit 0 을 내면, 못 본 것을 본 척하는
    것이다. contradiction 이 있어도 도구 실패면 1 — 모순 집계는 봉투에
    그대로 남긴다.
    """
    if not isinstance(report, dict):
        return EXIT_TOOL_FAILED
    if report.get("toolFailed"):
        return EXIT_TOOL_FAILED
    if report.get("ok"):
        return EXIT_OK
    return EXIT_CONTRADICTION


def validate_report(report):
    """보고 봉투의 정직 계약. 문제 문자열 목록(비면 통과).

    - kind / schemaVersion 고정.
    - ok 는 contradictions==0 과만 같다.
    - contradictions / reviews 는 findings 의 severity 집계와 같다.
    - contradiction 행은 irIdentical 이 참이어야 한다.
    - review 행은 irIdentical 이 거짓이어야 한다.
    - other-doc 은 findings 에 없다.
    - toolFailed 가 참이면 exit 는 1. 그래도 심각도 집계는 뒤집지 않는다.
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
    findings = report.get("findings")
    if not isinstance(findings, list):
        issues.append("findings 가 list 가 아니다")
        findings = []
    contradictions = sum(1 for row in findings if isinstance(row, dict) and row.get("severity") == "contradiction")
    reviews = sum(1 for row in findings if isinstance(row, dict) and row.get("severity") == "review")
    if report.get("contradictions") != contradictions:
        issues.append("contradictions 가 findings 집계와 다르다")
    if report.get("reviews") != reviews:
        issues.append("reviews 가 findings 집계와 다르다")
    if report.get("ok") != (contradictions == 0):
        issues.append("ok 는 contradictions==0 과만 같아야 한다")
    for row in findings:
        if not isinstance(row, dict):
            issues.append("finding 이 dict 가 아니다")
            continue
        sev = row.get("severity")
        if sev == "other-doc":
            issues.append("other-doc 을 finding 으로 부르면 안 된다")
        if sev == "contradiction" and not row.get("irIdentical"):
            issues.append(f"{row.get('stem')}: contradiction 인데 irIdentical 이 거짓이다")
        if sev == "review" and row.get("irIdentical"):
            issues.append(f"{row.get('stem')}: review 인데 irIdentical 이 참이다")
        if sev not in SEVERITIES and sev is not None:
            issues.append(f"{row.get('stem')}: 알 수 없는 severity {sev!r}")
        diverged = row.get("diverged")
        if not isinstance(diverged, list) or not diverged:
            issues.append(f"{row.get('stem')}: finding 에 diverged 가 비어 있다")
    if report.get("toolFailed"):
        if report.get("exit") != EXIT_TOOL_FAILED:
            issues.append("toolFailed 의 exit 는 1 이어야 한다")
    else:
        expected = EXIT_OK if contradictions == 0 else EXIT_CONTRADICTION
        if report.get("exit") not in (None, expected):
            issues.append("exit 가 ok/contradictions 계약과 다르다")
    return issues


def write_report(report, path):
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


def format_finding_detail(finding):
    return ", ".join(
        f"{row['observation']} {observation_display(row['hwp'])}≠{observation_display(row['hwpx'])}"
        for row in finding.get("diverged", [])
    )


def render_summary(report, out_path=None):
    lines = [
        f"쌍둥이 {report.get('pairs', 0)}쌍 · 관측 대조 {report.get('observationsCompared', 0)}건",
        f"이름만 같은 다른 문서(제외): {report.get('sameNameDifferentDocument', 0)}쌍",
        f"결함 후보: {len(report.get('findings') or [])}건 (그중 IR 동일 모순 {report.get('contradictions', 0)}건)",
    ]
    if report.get("toolFailed"):
        lines.append("도구 실패 → 분류를 위장하지 않음 (exit 1)")
        for err in report.get("toolErrors") or []:
            lines.append(
                f"  도구[{err.get('where')}]: {err.get('kind')} · {err.get('error')}"
            )
    for finding in report.get("findings") or []:
        mark = "!!" if finding.get("severity") == "contradiction" else "  "
        lines.append(
            f" {mark} {str(finding.get('stem') or '')[:46]:48} irIdentical={finding.get('irIdentical')} | "
            f"{format_finding_detail(finding)}"
        )
    if report.get("pairErrors"):
        for err in report["pairErrors"][:10]:
            lines.append(f"  쌍 오류: {err.get('stem', '?')} · {err.get('kind')} · {err.get('error')}")
    if report.get("writeError"):
        lines.append(f"  쓰기 오류: {report['writeError']}")
    if out_path:
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
        return path, exception_tool_error(exc, "find-bin")
    except Exception as exc:
        return path, exception_tool_error(exc, "find-bin")


def parse_args(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0, help="검사할 쌍 수 (0=전부)")
    ap.add_argument("--bin", default=None)
    ap.add_argument("-o", "--out", default=None)
    ap.add_argument(
        "--cli-timeout",
        type=int,
        default=CLI_TIMEOUT_DEFAULT,
        help="관측 CLI 초(기본 0=무제한)",
    )
    return ap.parse_args(argv)


def default_out_path():
    return os.path.join(runner.GYM, "differential-report.json")


def attach_write_error(report, write_err):
    """쓰기 오류를 보고에 붙인다. 심각도 집계는 바꾸지 않는다."""
    if write_err:
        report["writeError"] = write_err
    return report


def main(argv=None):
    try:
        a = parse_args(argv)
    except FATAL_EXCEPTIONS:
        raise
    except SystemExit:
        raise
    except Exception:
        return EXIT_TOOL_FAILED

    tool_errors = []
    bin_path, find_err = find_bin_safe(a.bin)
    if find_err:
        tool_errors.append(find_err)

    pairs, walk_err = find_twins_safe()
    if walk_err:
        tool_errors.append(walk_err)
    pairs = select_pairs(pairs, a.limit)

    timeout = normalize_timeout(getattr(a, "cli_timeout", CLI_TIMEOUT_DEFAULT)) or None

    def run(args):
        return run_cli(bin_path, args, timeout=timeout)

    pair_errors = []
    try:
        compared, other_doc, findings = compare_twins(
            pairs, run, errors_out=pair_errors
        )
    except FATAL_EXCEPTIONS:
        raise
    except CATCHABLE_EXCEPTIONS as exc:
        tool_errors.append(exception_tool_error(exc, "compare"))
        compared, other_doc, findings = 0, 0, []
    except Exception as exc:
        tool_errors.append(exception_tool_error(exc, "compare"))
        compared, other_doc, findings = 0, 0, []

    report = build_report(
        bin_name=os.path.basename(bin_path) if bin_path else "",
        pairs_count=len(pairs),
        compared=compared,
        other_doc=other_doc,
        findings=findings,
        tool_errors=tool_errors,
        pair_errors=pair_errors,
    )
    out = a.out or default_out_path()
    write_err = write_report_safe(report, out)
    attach_write_error(report, write_err)
    for line in render_summary(report, out):
        print(line)
    return status_exit(report)


if __name__ == "__main__":
    sys.exit(main())
