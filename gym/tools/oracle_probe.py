"""[#5207] 라이브 오라클 프로브 — 이중 계산 결정성·자리표·부재 보고.

## 왜 이 도구인가

채점 규약: 정답을 골든 파일로 박제하지 않고 **채점 시점에 rhwp 로 재계산**한다.
그 오라클이 (1) 두 번 돌려도 같은 값인지, (2) `{input}` / 한 문자열의 여러
`{sub:이름}` 이 기준 풀이(`build_baseline.resolve`)와 같이 결정적으로 치환되는지,
(3) 산출물이 없으면 통과로 위장하지 않는지 — 를 독립 도구로 감사한다.

새 CLI 바이너리는 없다. 이 스크립트는 순수 Python 프로브다. `--json` 은 팩·표본
픽스처 없이 구조 자기점검을 내고, `--selftest` 는 내장 프로브를 돌린다.

`build_baseline.py` 를 import 하지 않는다(경로 삽입·스트림 재설정·러너 적재 같은
부수효과가 구조 자기점검을 더럽힌다). 자리표 치환은 그 함수와 같은 규칙으로
이 파일에 복제했고, 테스트가 양쪽 출력을 대조한다.

사용:
  python gym/tools/oracle_probe.py --json       # 픽스처 없는 구조 자기점검
  python gym/tools/oracle_probe.py --selftest   # 내장 프로브
  python gym/tools/oracle_probe.py --json --selftest
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile

KIND = "gymOracleProbe"
SCHEMA_VERSION = "1.0"
INPUT_TOKEN = "{input}"
SUB_MARK = "{sub:"

REQUIRED_EXPORTS = (
    "probe_determinism",
    "probe_placeholders",
    "probe_missing_artifact",
    "resolve_placeholders",
    "json_canonicalize",
)


# ---------------------------------------------------------------------------
# 자리표 — build_baseline.resolve 와 같은 규칙 (#4664 다중 {sub:} 포함)
# ---------------------------------------------------------------------------


def leftover_sub_names(text):
    """치환 뒤에 남은 `{sub:이름}` 의 이름들. 비어 있어야 자리표 프로브가 통과한다."""
    if not isinstance(text, str) or SUB_MARK not in text:
        return []
    found = []
    rest = text
    while SUB_MARK in rest:
        _head, rest = rest.split(SUB_MARK, 1)
        if "}" not in rest:
            found.append(rest)
            break
        name, rest = rest.split("}", 1)
        found.append(name)
    return found


def extract_sub_names(token):
    """토큰에서 `{sub:이름}` 을 왼쪽부터 모은다 — 치환 전 인벤토리."""
    return leftover_sub_names(token)


def is_exact_sub_token(token):
    return (
        isinstance(token, str)
        and token.startswith(SUB_MARK)
        and token.endswith("}")
        and token.count("{") == 1
    )


def _maybe_mkdir(path, fallback=None):
    parent = os.path.dirname(path)
    if fallback is not None:
        parent = parent or fallback
    if parent:
        os.makedirs(parent, exist_ok=True)


def resolve_placeholders(token, task, sub_dir, mkdir=True):
    """`build_baseline.resolve` 와 같은 치환.

    - 토큰 전체가 `{input}` 이면 `task["input"]`.
    - 토큰 전체가 `{sub:이름}` 이면 `join(sub_dir, 이름)` (중첩 폴더는 미리 만든다).
    - 문자열 안의 여러 `{sub:이름}` 은 **전부** 바꾼다. 경로의 `\\` 는 JSON 에
      박힐 수 있으므로 `\\\\` 로 이스케이프한다(#4664).
    - 문자열 안의 `{input}` 도 모두 과제 입력 경로로 바꾼다 — 기준 풀이 조립기와 같다.
    """
    if token == INPUT_TOKEN:
        return task["input"]
    if is_exact_sub_token(token):
        path = os.path.join(sub_dir, token[len(SUB_MARK):-1])
        if mkdir:
            # 정확한 토큰 분기는 dirname 만 만든다(build_baseline 과 동일).
            os.makedirs(os.path.dirname(path), exist_ok=True)
        return path
    if isinstance(token, str) and SUB_MARK in token:
        out = []
        rest = token
        while SUB_MARK in rest:
            head, rest = rest.split(SUB_MARK, 1)
            name, rest = rest.split("}", 1)
            path = os.path.join(sub_dir, name)
            if mkdir:
                _maybe_mkdir(path, fallback=sub_dir)
            out.append(head + path.replace("\\", "\\\\"))
        out.append(rest)
        return "".join(out)
    if isinstance(token, str) and INPUT_TOKEN in token:
        return token.replace(INPUT_TOKEN, task["input"].replace("\\", "/"))
    return token


def resolve_cmd(cmd, task, sub_dir, mkdir=True):
    """라이브 오라클 argv 의 각 토큰을 치환한다 — `build_baseline.run_step` 과 같은 결."""
    if not isinstance(cmd, (list, tuple)):
        raise TypeError(f"cmd 는 인자 목록이어야 한다: {type(cmd).__name__}")
    return [resolve_placeholders(a, task, sub_dir, mkdir=mkdir) for a in cmd]


def probe_placeholders(token, task, sub_dir):
    """한 토큰의 `{sub:}` 가 모두 사라졌는지 본다. 남으면 실패(리터럴 파일명 사고)."""
    if not isinstance(token, str):
        return {
            "ok": False,
            "token": token,
            "resolved": None,
            "leftover": [],
            "names": [],
            "error": f"자리표는 문자열이어야 한다: {type(token).__name__}",
        }
    names = extract_sub_names(token)
    try:
        resolved = resolve_placeholders(token, task if task is not None else {}, sub_dir)
    except (OSError, KeyError, TypeError, ValueError) as exc:
        return {
            "ok": False,
            "token": token,
            "resolved": None,
            "leftover": leftover_sub_names(token),
            "names": names,
            "error": f"{type(exc).__name__}: {exc}",
        }
    leftover = leftover_sub_names(resolved if isinstance(resolved, str) else "")
    return {
        "ok": leftover == [],
        "token": token,
        "resolved": resolved,
        "leftover": leftover,
        "names": names,
        "inputExact": token == INPUT_TOKEN,
    }


def probe_cmd_placeholders(cmd, task, sub_dir):
    """argv 전 토큰을 치환하고, 어느 한 자리에 `{sub:}` 가 남으면 실패한다."""
    tokens = []
    ok = True
    try:
        items = list(cmd)
    except TypeError as exc:
        return {"ok": False, "tokens": [], "error": f"cmd 순회 실패: {exc}"}
    for item in items:
        one = probe_placeholders(item, task, sub_dir)
        tokens.append(one)
        if not one.get("ok"):
            ok = False
    return {"ok": ok, "tokens": tokens, "count": len(tokens)}


# ---------------------------------------------------------------------------
# JSON 정규화 — 키 순서·숫자 문자열을 같게 보고 이중 계산을 비교한다
# ---------------------------------------------------------------------------


def norm_scalar(value):
    """`checks.norm` 과 같은 스칼라 정규화. bool 을 먼저 본다(int 의 하위형)."""
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return float(value)
    if isinstance(value, float):
        if value != value or value in (float("inf"), float("-inf")):
            return None
        return value
    if isinstance(value, str):
        stripped = value.strip()
        try:
            number = float(stripped)
        except ValueError:
            return stripped
        if number != number or number in (float("inf"), float("-inf")):
            return stripped
        return number
    return value


def json_ready(value):
    """비교 가능한 JSON 값으로 접는다. 집합은 정규화 후 정렬한다."""
    if value is None or isinstance(value, bool):
        return value
    if isinstance(value, (int, float, str)):
        return norm_scalar(value)
    if isinstance(value, dict):
        return {str(key): json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    if isinstance(value, set):
        prepared = [json_ready(item) for item in value]
        return sorted(
            prepared,
            key=lambda item: json.dumps(item, sort_keys=True, ensure_ascii=False, default=str),
        )
    if isinstance(value, bytes):
        try:
            return value.decode("utf-8")
        except UnicodeDecodeError:
            return value.hex()
    return str(value)


def json_canonicalize(value):
    """키 정렬·공백 없는 JSON 문자열. 이중 계산 등호의 단일 출처."""
    return json.dumps(
        json_ready(value),
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    )


def snapshot(value):
    """정규화 스냅샷 — 다음 호출이 원본을 변이시켜도 비교 값이 바뀌지 않는다."""
    return json.loads(json_canonicalize(value))


# ---------------------------------------------------------------------------
# 결정성 — 같은 compute 를 두 번(또는 N 번) 돌려 정규화 등호를 본다
# ---------------------------------------------------------------------------


def _run_once(compute_fn):
    if not callable(compute_fn):
        return False, None, f"compute_fn 이 호출 가능하지 않다: {type(compute_fn).__name__}"
    try:
        raw = compute_fn()
    except Exception as exc:  # noqa: BLE001 — 오라클 예외도 데이터
        return False, None, f"{type(exc).__name__}: {exc}"
    try:
        canon = json_canonicalize(raw)
        return True, json.loads(canon), None
    except (TypeError, ValueError) as exc:
        return False, None, f"JSON 정규화 실패: {type(exc).__name__}: {exc}"


def probe_determinism(compute_fn):
    """`compute_fn` 을 두 번 호출하고 JSON 정규화 결과가 같은지 본다.

    한쪽이라도 예외이거나 직렬화에 실패하면 `ok` 는 False 다. 같은 예외를
    두 번 내도 '결정적 성공'으로 위장하지 않는다 — 오라클이 값을 내야 한다.
    """
    ok1, first, err1 = _run_once(compute_fn)
    ok2, second, err2 = _run_once(compute_fn)
    if not ok1 or not ok2:
        return {
            "ok": False,
            "equal": False,
            "runs": 2,
            "first": first,
            "second": second,
            "error": err1 or err2,
            "errors": [e for e in (err1, err2) if e],
        }
    equal = first == second
    return {
        "ok": equal,
        "equal": equal,
        "runs": 2,
        "first": first,
        "second": second,
        "canonicalFirst": json_canonicalize(first),
        "canonicalSecond": json_canonicalize(second),
    }


def probe_determinism_n(compute_fn, n=2):
    """이중 계산의 일반화 — n 회 돌려 전부 같아야 한다. n < 2 는 사용 오류."""
    if not isinstance(n, int) or n < 2:
        return {"ok": False, "equal": False, "runs": n, "error": f"n 은 2 이상이어야 한다: {n!r}"}
    snapshots = []
    errors = []
    for _ in range(n):
        ok, value, err = _run_once(compute_fn)
        snapshots.append(value)
        if not ok:
            errors.append(err)
    if errors:
        return {
            "ok": False,
            "equal": False,
            "runs": n,
            "values": snapshots,
            "error": errors[0],
            "errors": errors,
        }
    equal = all(item == snapshots[0] for item in snapshots[1:])
    return {"ok": equal, "equal": equal, "runs": n, "values": snapshots}


# ---------------------------------------------------------------------------
# 부재 보고 — 파일이 없으면 절대 통과로 위장하지 않는다
# ---------------------------------------------------------------------------


def classify_artifact(path):
    """경로를 present / absent / not-a-file / invalid 로 가른다."""
    if path is None or path == "":
        return "invalid"
    if not isinstance(path, (str, os.PathLike)):
        return "invalid"
    try:
        if os.path.isfile(path):
            return "present"
        if os.path.isdir(path):
            return "not-a-file"
        return "absent"
    except OSError:
        return "absent"


def probe_missing_artifact(path):
    """산출물 경로를 본다. 일반 파일이 아니면 `ok=False` 이고 status 를 남긴다.

    부재를 통과로 위장하지 않는 것이 이 함수의 존재 이유다. 디렉터리·빈 경로·
    잘못된 타입은 모두 실패이며, `status` 가 `absent` / `not-a-file` / `invalid`
    로 이유를 밝힌다.
    """
    status = classify_artifact(path)
    report = {
        "ok": False,
        "present": False,
        "status": status,
        "path": None if path is None else str(path),
    }
    if status == "invalid":
        report["error"] = "경로가 비어 있거나 문자열이 아니다"
        return report
    if status == "present":
        try:
            size = os.path.getsize(path)
        except OSError as exc:
            report["status"] = "absent"
            report["error"] = f"크기 확인 실패: {exc}"
            return report
        report["ok"] = True
        report["present"] = True
        report["size"] = size
        return report
    if status == "not-a-file":
        report["error"] = "디렉터리이지 산출 파일이 아니다"
        return report
    return report


def probe_artifacts(paths):
    """여러 산출물을 같은 규칙으로 본다. 하나라도 없으면 묶음은 실패."""
    items = [probe_missing_artifact(p) for p in list(paths)]
    missing = [item["path"] for item in items if not item.get("ok")]
    return {
        "ok": bool(items) and not missing,
        "items": items,
        "missing": missing,
        "count": len(items),
    }


# ---------------------------------------------------------------------------
# 묶음 — 결정성 + 자리표 + 산출물을 한 오라클 프로브로
# ---------------------------------------------------------------------------


def probe_live_oracle(compute_fn, token=None, task=None, sub_dir=None, artifacts=None):
    """라이브 오라클 한 건: 이중 계산, 선택적 자리표, 선택적 산출물 존재."""
    det = probe_determinism(compute_fn)
    ph = None
    if token is not None:
        ph = probe_placeholders(token, task or {}, sub_dir or "")
    art = probe_artifacts(artifacts) if artifacts is not None else None
    ok = bool(det.get("ok"))
    if ph is not None:
        ok = ok and bool(ph.get("ok"))
    if art is not None:
        ok = ok and bool(art.get("ok"))
    return {
        "ok": ok,
        "determinism": det,
        "placeholders": ph,
        "artifacts": art,
    }


def envelope(**fields):
    body = {"kind": KIND, "schemaVersion": SCHEMA_VERSION}
    body.update(fields)
    return body


# ---------------------------------------------------------------------------
# 자기점검 — 픽스처 없는 구조 검사 + 내장 프로브
# ---------------------------------------------------------------------------


def structural_self_check():
    """팩·표본·바이너리 없이 모듈 표면과 핵심 경로를 확인한다."""
    # importlib 로 다른 이름에 실리면 __name__ 이 sys.modules 에 없을 수 있다.
    namespace = globals()
    issues = []
    exports = []
    for name in REQUIRED_EXPORTS:
        fn = namespace.get(name)
        if fn is None or not callable(fn):
            issues.append(f"필수 함수 없음: {name}")
        else:
            exports.append(name)

    det = probe_determinism(lambda: {"pageCount": 2, "kind": "info"})
    if not det.get("ok"):
        issues.append("안정 계산의 결정성 프로브가 실패했다")

    with tempfile.TemporaryDirectory() as sub_dir:
        token = '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'
        ph = probe_placeholders(token, {"input": "in.hwp"}, sub_dir)
        if not ph.get("ok") or ph.get("leftover"):
            issues.append(f"다중 {{sub:}} 가 남았다: {ph.get('leftover')}")
        if "{sub:" in str(ph.get("resolved", "")):
            issues.append("치환 결과에 {sub: 리터럴이 남았다")

    missing = probe_missing_artifact(
        os.path.join(tempfile.gettempdir(), "rhwp-oracle-probe-absent-no-such-file.json")
    )
    if missing.get("ok") is True:
        issues.append("부재 산출물을 통과로 위장했다")
    if missing.get("status") != "absent":
        issues.append(f"부재 status 가 absent 가 아니다: {missing.get('status')}")

    return envelope(
        ok=not issues,
        mode="structural",
        exports=exports,
        required=list(REQUIRED_EXPORTS),
        issues=issues,
        issueCount=len(issues),
        probes={
            "determinism": {"ok": det.get("ok"), "equal": det.get("equal")},
            "placeholders": {"ok": ph.get("ok"), "leftover": ph.get("leftover")},
            "missingArtifact": {
                "ok": missing.get("ok"),
                "status": missing.get("status"),
                "present": missing.get("present"),
            },
        },
    )


def run_selftest():
    """내장 프로브 — 통과해야 할 것과 실패해야 할 것을 모두 확인한다."""
    checks = []

    def add(name, ok, detail=None):
        item = {"name": name, "ok": bool(ok)}
        if detail is not None:
            item["detail"] = detail
        checks.append(item)

    stable = probe_determinism(lambda: {"n": 1, "label": "fixed"})
    add("determinism-stable", stable.get("ok") is True, {"equal": stable.get("equal")})

    counter = {"i": 0}

    def drift():
        counter["i"] += 1
        return {"n": counter["i"]}

    drifted = probe_determinism(drift)
    add(
        "determinism-drift-detected",
        drifted.get("ok") is False and drifted.get("equal") is False,
        {"first": drifted.get("first"), "second": drifted.get("second")},
    )

    broken = probe_determinism(lambda: (_ for _ in ()).throw(RuntimeError("오라클 붕괴")))
    add("determinism-exception-is-not-pass", broken.get("ok") is False, {"error": broken.get("error")})

    seq = [{"v": 1}, {"v": "1.0"}]
    numeric = probe_determinism(lambda: seq.pop(0) if seq else {"v": 1})
    add("determinism-numeric-norm", numeric.get("ok") is True)

    with tempfile.TemporaryDirectory() as sub_dir:
        token = '{"input": "{sub:capsules/a.hwp}", "output": "{sub:capsules/b.hwp}"}'
        task = {"input": "samples/in.hwp"}
        ph = probe_placeholders(token, task, sub_dir)
        add(
            "placeholders-multi-sub",
            ph.get("ok") is True and not ph.get("leftover") and "{sub:" not in ph.get("resolved", ""),
            {"resolved": ph.get("resolved"), "names": ph.get("names")},
        )
        exact = probe_placeholders("{input}", task, sub_dir)
        add("placeholders-exact-input", exact.get("ok") is True and exact.get("resolved") == task["input"])
        leftover = probe_placeholders("keep {sub:", task, sub_dir)
        add("placeholders-unclosed-is-not-pass", leftover.get("ok") is False)

        present_path = os.path.join(sub_dir, "answer.json")
        with open(present_path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write("{}\n")
        present = probe_missing_artifact(present_path)
        add("artifact-present", present.get("ok") is True and present.get("status") == "present")

        missing_path = os.path.join(sub_dir, "no-such-output.svg")
        missing = probe_missing_artifact(missing_path)
        add(
            "artifact-absent-is-not-pass",
            missing.get("ok") is False and missing.get("status") == "absent" and missing.get("present") is False,
            {"status": missing.get("status")},
        )

        as_dir = probe_missing_artifact(sub_dir)
        add("artifact-directory-is-not-pass", as_dir.get("ok") is False and as_dir.get("status") == "not-a-file")

    add("artifact-empty-path-is-not-pass", probe_missing_artifact("")["ok"] is False)
    add("artifact-none-is-not-pass", probe_missing_artifact(None)["ok"] is False)

    nrun = probe_determinism_n(lambda: {"k": True}, 3)
    add("determinism-n-stable", nrun.get("ok") is True and nrun.get("runs") == 3)
    add("determinism-n-rejects-one", probe_determinism_n(lambda: 1, 1).get("ok") is False)

    failed = [c["name"] for c in checks if not c["ok"]]
    return envelope(
        ok=not failed,
        mode="selftest",
        checks=checks,
        failed=failed,
        issueCount=len(failed),
        checkCount=len(checks),
    )


def render_human(report):
    mode = report.get("mode", "?")
    flag = "통과" if report.get("ok") else "실패"
    lines = [f"라이브 오라클 프로브 [{mode}]: {flag}  (kind={report.get('kind')} schema={report.get('schemaVersion')})"]
    if report.get("mode") == "structural":
        lines.append("  exports: " + ", ".join(report.get("exports") or []))
        probes = report.get("probes") or {}
        for key, body in probes.items():
            lines.append(f"  {key}: {body}")
        for issue in report.get("issues") or []:
            lines.append(f"  ! {issue}")
    else:
        for check in report.get("checks") or []:
            mark = "O" if check.get("ok") else "X"
            lines.append(f"  {mark} {check.get('name')}")
        for name in report.get("failed") or []:
            lines.append(f"  ! {name}")
    return "\n".join(lines)


def parse_args(argv=None):
    ap = argparse.ArgumentParser(description="라이브 오라클 결정성·자리표·부재 프로브 (#5207)")
    ap.add_argument("--json", action="store_true", help="JSON 봉투(kind=gymOracleProbe)를 낸다")
    ap.add_argument("--selftest", action="store_true", help="내장 프로브를 실행한다")
    return ap.parse_args(argv)


def run(argv=None):
    args = parse_args(argv)
    report = run_selftest() if args.selftest else structural_self_check()
    if args.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    else:
        sys.stdout.write(render_human(report) + "\n")
    return 0 if report.get("ok") else 1


def main():
    return run(sys.argv[1:])


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except Exception:
            pass
    sys.exit(main())
