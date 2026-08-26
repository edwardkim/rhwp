"""gym 능력 인증서 — 능력 점수를 재현 가능하게 봉인해 위조 불가능한 신뢰 원본으로.

## 왜 이 도구인가 (증명 가능한 능력)

점수를 재는 것과 그 점수가 **진짜**임을 증명하는 것은 다르다. 누가 "내 에이전트가
gym 을 만점 통과했다"고 주장해도, 그게 몰래 축소한 벤치마크나 다른 바이너리로 낸
것이면 거짓이다. 리포트(report.py)만으로는 그 위조를 막지 못한다.

이 인증서가 막는다:

- **벤치마크 지문** — 전 pack 정의(pack.json·tasks·reference)의 sha256. 인증서가
  '무엇을' 재고 채점했는지 못박는다. 벤치마크를 몰래 줄이면 지문이 바뀌어 들킨다.
- **바이너리 신원** — `capabilitiesSha256`. '어느 바이너리로' 냈는지 못박는다.
- **재현 = 증명** — 같은 바이너리 + 같은 pack 정의면 누구나 같은 점수를 재현한다.
  `--verify` 가 다시 돌려 인증서와 대조한다: 재현되면 진짜, 아니면 위조.

암호 서명이 아니라 **결정론적 재현**이 증명 원리다 — reproducible-build attestation 과
같은 계열이라 키 관리 없이 누구나 검증할 수 있다.

예외 세 자리(이슈 #5275)는 스택이 아니라 kind 로 남긴다.

- **없는 스코어카드** — report.py 가 scorecard.json 없이 끝나면 `missing-scorecard`.
  빈 인증서를 지어 만점인 척하지 않는다.
- **깨진 JSON** — 인증서·리포트 stdout 이 객체가 아니면 `malformed-json` /
  `malformed-cert` / `malformed-report`. 파싱 실패를 재현 성공으로 바꾸지 않는다.
- **미가용 pack** — 리포트의 `packsUnavailable` 을 인증서 `unavailablePacks` 와
  `unavailable-pack` 예외 칸에 옮긴다. 부재는 위조가 아니지만 숨기면 위조다.

카탈로그·봉투 계약은 `gym/docs/certify_report.md` 가 정본이다. 작업 기록은
`mydocs/working/gym_certify_report.md`. 시험은 `scripts/tests/test_gym_certify.py`.

새 CLI 플래그는 없다. `--bin` `--verify` `--out` `--at` 만 쓴다.

## 사용

    python gym/certify.py --bin target/debug/rhwp --out cert.json      # 인증서 발급
    python gym/certify.py --verify cert.json --bin target/debug/rhwp    # 재현 검증(exit 0/1)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = HERE
REPO_ROOT = os.path.dirname(GYM_ROOT)

CERT_KIND = "gymCapabilityCertificate"
CERT_SCHEMA = "1.0"
REPORT_KIND = "gymCapabilityReport"

# 종료 코드. 0=발급 성공 또는 재현 통과, 1=재현 불일치, 2=도구 실패.
EXIT_OK = 0
EXIT_VERIFY_FAIL = 1
EXIT_TOOL_FAILED = 2

# 새 플래그 없음. 시험이 argparse 옵션 집합을 이 튜플과 대조한다.
CERT_CLI_FLAGS = ("--bin", "--verify", "--out", "--at")

# 지문에 넣는 측정 입력. 문서·mydocs 는 점수를 바꾸지 않아 넣지 않는다.
FINGERPRINT_TREES = ("packs", "core", "profiles", "tools")
FINGERPRINT_FILES = ("score.py", "report.py", "certify.py")
FINGERPRINT_SKIP_DIRS = frozenset({"__pycache__"})
FINGERPRINT_SKIP_SUFFIXES = (".pyc",)

CERT_KEYS = (
    "kind",
    "schemaVersion",
    "benchmarkFingerprint",
    "report",
    "proof",
    "certifiedAt",
    "exceptions",
    "exceptionCount",
    "unavailablePacks",
    "trusted",
)

CERT_LIST_KEYS = ("exceptions", "unavailablePacks")
CERT_INT_KEYS = ("exceptionCount",)
CERT_BOOL_KEYS = ("trusted",)

REPRODUCIBLE_CORE_KEYS = (
    "benchmarkFingerprint",
    "capabilitiesSha256",
    "accuracy",
    "coverage",
    "axisProfile",
)

COVERAGE_CORE_KEYS = ("percent", "covered", "agentFacingTotal")

PROOF_TEXT = "reproduce: 같은 bin + 같은 pack 정의로 --verify 하면 core 가 일치한다"

# 예외 kind 카탈로그. 문서·시험이 같은 표를 본다.
EXCEPTION_KINDS = (
    "missing-cert",
    "missing-bin",
    "missing-scorecard",
    "missing-report",
    "malformed-json",
    "malformed-cert",
    "malformed-report",
    "wrong-kind",
    "unavailable-pack",
    "fingerprint-empty",
    "report-tool-failed",
    "verify-mismatch",
    "permission",
    "os-error",
    "decode-error",
    "write-error",
    "type-error",
    "value-error",
    "unexpected",
)

EXCEPTION_KIND_HELP = {
    "missing-cert": (
        "--verify 경로에 인증서 파일이 없다. 없는 파일을 재현 성공으로 부르지 않는다."
    ),
    "missing-bin": (
        "--bin 이 비었거나 경로형 바이너리가 없어 report.py 가 시작되지 않는다."
    ),
    "missing-scorecard": (
        "report.py 가 스코어카드 없이 끝났다. stderr 에 missing-scorecard 가 "
        "보이거나 scorecard.json 부재 문구가 있다. 빈 인증서를 발급하지 않는다."
    ),
    "missing-report": (
        "인증서에 report 칸이 없거나 객체가 아니다. 재현 core 를 만들 수 없다."
    ),
    "malformed-json": (
        "인증서 파일 또는 report.py stdout 이 UTF-8 JSON 이 아니다. "
        "잘린 객체·빈 파일·트레일링 콤마."
    ),
    "malformed-cert": (
        "인증서가 JSON 객체(dict)가 아니다. 배열·문자열은 인증서가 아니다."
    ),
    "malformed-report": (
        "리포트가 JSON 객체가 아니다. report.py 가 카드 텍스트를 stdout 에 "
        "흘리면 --json 계약이 깨진 것이다."
    ),
    "wrong-kind": (
        f"인증서 kind 가 {CERT_KIND} 가 아니다. 다른 봉투를 인증서로 들이밀면 "
        "재현 실패(exit 1)다."
    ),
    "unavailable-pack": (
        "리포트 packsUnavailable 의 pack. 부재는 위조가 아니지만 인증서가 "
        "숨기면 축소로 오해된다. 발급 시 예외 칸에 남기고, 재현 때 집합이 "
        "달라지면 verify-mismatch 다."
    ),
    "fingerprint-empty": (
        "지문에 넣을 파일이 하나도 없다. gym_root 가 빈 디렉터리이거나 "
        "packs/core/profiles/tools 와 score/report/certify 가 모두 없다."
    ),
    "report-tool-failed": (
        "report.py 가 비-0 으로 끝났고 더 구체적인 kind 로 분류되지 않았다."
    ),
    "verify-mismatch": (
        "재현 core 가 인증 시점과 다르다. 지문·바이너리 신원·정확도·커버리지·"
        "축·미가용 pack 집합 중 하나."
    ),
    "permission": "인증서 파일 또는 산출 경로의 읽기·쓰기 권한이 없다.",
    "os-error": "그 밖의 OSError. 디스크·경로·잠금.",
    "decode-error": "인증서 파일이 UTF-8 이 아니다.",
    "write-error": "--out 경로에 인증서를 쓰지 못했다.",
    "type-error": "값 타입이 계약과 다르다.",
    "value-error": "값은 있는데 형태가 틀렸다.",
    "unexpected": "분류되지 않은 운영 예외. 치명 예외는 여기로 접지 않는다.",
}

FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

CATCHABLE_EXCEPTIONS = (
    FileNotFoundError,
    PermissionError,
    IsADirectoryError,
    NotADirectoryError,
    UnicodeError,
    json.JSONDecodeError,
    ValueError,
    TypeError,
    KeyError,
    IndexError,
    AttributeError,
    OSError,
    subprocess.SubprocessError,
)

INFORMATIONAL_KINDS = frozenset({"unavailable-pack", "fingerprint-empty"})

VERIFY_PASS_MESSAGE = "✅ 인증서 재현 검증 통과 — 벤치마크·바이너리·전 점수가 재현된다(진짜)"
VERIFY_FAIL_PREFIX = "❌ 인증서 재현 검증 실패:"

KIND_MARKERS = (
    "missing-scorecard",
    "missing-bin",
    "malformed-json",
    "malformed-scorecard",
    "malformed-report",
    "report-tool-failed",
    "permission",
    "decode-error",
)


class CertifyError(Exception):
    """인증서 도구가 접는 운영 예외. kind 는 EXCEPTION_KINDS 중 하나."""

    def __init__(self, kind: str, message: str, **extra: object) -> None:
        if kind not in EXCEPTION_KINDS:
            kind = "unexpected"
        self.kind = kind
        self.message = message
        self.extra = extra
        super().__init__(message)

    def as_record(self) -> dict:
        return exception_record(self.kind, self.message, **self.extra)


def is_json_object(value: object) -> bool:
    return isinstance(value, dict) and not isinstance(value, bool)


def is_json_array(value: object) -> bool:
    return isinstance(value, list)


def is_known_exception_kind(kind: object) -> bool:
    return isinstance(kind, str) and kind in EXCEPTION_KINDS


def is_fatal_exception(exc: BaseException) -> bool:
    return isinstance(exc, FATAL_EXCEPTIONS)


def is_catchable_exception(exc: BaseException) -> bool:
    if is_fatal_exception(exc):
        return False
    return isinstance(exc, CATCHABLE_EXCEPTIONS) or isinstance(exc, CertifyError)


def describe_exception_kind(kind: str) -> str:
    if kind in EXCEPTION_KIND_HELP:
        return EXCEPTION_KIND_HELP[kind]
    return EXCEPTION_KIND_HELP["unexpected"]


def exception_record(kind: str, message: str, **extra: object) -> dict:
    rec: dict = {"kind": kind if is_known_exception_kind(kind) else "unexpected",
                 "message": message}
    for key in ("where", "path", "pack", "role"):
        if extra.get(key) not in (None, ""):
            rec[key] = extra[key]
    return rec


def is_informational_kind(kind: str) -> bool:
    return kind in INFORMATIONAL_KINDS


def structural_exceptions(exceptions: list[dict]) -> list[dict]:
    return [e for e in exceptions if not is_informational_kind(e.get("kind", ""))]


def error_head(exc: BaseException, limit: int = 240) -> str:
    text = str(exc).strip() or type(exc).__name__
    if len(text) > limit:
        return text[:limit]
    return text


def classify_os_error(exc: BaseException, *, role: str = "cert") -> str:
    if isinstance(exc, CertifyError):
        return exc.kind
    if isinstance(exc, json.JSONDecodeError):
        return "malformed-json"
    if isinstance(exc, UnicodeError):
        return "decode-error"
    if isinstance(exc, PermissionError):
        return "permission"
    if isinstance(exc, FileNotFoundError):
        return "missing-cert" if role == "cert" else "missing-bin"
    if isinstance(exc, IsADirectoryError):
        return "malformed-cert" if role == "cert" else "malformed-report"
    if isinstance(exc, TypeError):
        return "type-error"
    if isinstance(exc, ValueError):
        return "value-error"
    if isinstance(exc, subprocess.SubprocessError):
        return "report-tool-failed"
    if isinstance(exc, OSError):
        return "os-error"
    return "unexpected"


def wrap_exception(exc: BaseException, *, role: str = "cert",
                   where: str = "", path: str = "") -> CertifyError:
    if isinstance(exc, CertifyError):
        return exc
    if is_fatal_exception(exc):
        raise exc
    kind = classify_os_error(exc, role=role)
    return CertifyError(kind, error_head(exc), role=role, where=where, path=path)


def load_text(path: str, *, role: str = "cert") -> str:
    if not isinstance(path, str) or not path.strip():
        kind = "missing-cert" if role == "cert" else "missing-bin"
        raise CertifyError(kind, f"{role} 경로가 비었다", role=role, path=path)
    if os.path.isdir(path):
        kind = "malformed-cert" if role == "cert" else "malformed-report"
        raise CertifyError(kind, f"{role} 경로가 디렉터리다: {path}",
                           role=role, path=path)
    if not os.path.isfile(path):
        kind = "missing-cert" if role == "cert" else "missing-bin"
        raise CertifyError(kind, f"{role} 파일이 없다: {path}",
                           role=role, path=path, where="load_text")
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except PermissionError as e:
        raise CertifyError("permission", f"{role} 권한 없음: {error_head(e)}",
                           role=role, path=path)
    except UnicodeError as e:
        raise CertifyError("decode-error", f"{role} UTF-8 디코드 실패: {error_head(e)}",
                           role=role, path=path)
    except OSError as e:
        raise CertifyError("os-error", f"{role} 읽기 실패: {error_head(e)}",
                           role=role, path=path)


def parse_json_text(text: str | bytes, *, role: str = "cert") -> object:
    if text is None:
        raise CertifyError("malformed-json", f"{role} 본문이 없다", role=role)
    if isinstance(text, bytes):
        try:
            text = text.decode("utf-8")
        except UnicodeError as e:
            raise CertifyError("decode-error", f"{role} UTF-8 디코드 실패: {error_head(e)}",
                               role=role)
    try:
        return json.loads(text)
    except json.JSONDecodeError as e:
        raise CertifyError("malformed-json",
                           f"{role} JSON 파싱 실패: {error_head(e)}",
                           role=role, where=f"line {e.lineno}")


def load_json_object(path: str, *, role: str = "cert") -> dict:
    raw = load_text(path, role=role)
    data = parse_json_text(raw, role=role)
    if not is_json_object(data):
        kind = "malformed-cert" if role == "cert" else "malformed-report"
        raise CertifyError(kind, f"{role} 가 JSON 객체가 아니다",
                           role=role, path=path)
    return data


def write_text(path: str, text: str) -> None:
    try:
        parent = os.path.dirname(path)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
    except PermissionError as e:
        raise CertifyError("write-error", f"산출 권한 없음: {error_head(e)}", path=path)
    except OSError as e:
        raise CertifyError("write-error", f"산출 기록 실패: {error_head(e)}", path=path)


def should_skip_fingerprint_name(name: str) -> bool:
    return name.endswith(FINGERPRINT_SKIP_SUFFIXES)


def add_fingerprint_file(entries: list[tuple[str, bytes]], gym_root: str, path: str) -> None:
    if not os.path.isfile(path):
        return
    rel = os.path.relpath(path, gym_root).replace(os.sep, "/")
    with open(path, "rb") as fh:
        entries.append((rel, fh.read()))


def add_fingerprint_tree(entries: list[tuple[str, bytes]], gym_root: str, rel_dir: str) -> None:
    root = os.path.join(gym_root, rel_dir)
    if not os.path.isdir(root):
        return
    for current, dirs, files in os.walk(root):
        dirs[:] = sorted(d for d in dirs if d not in FINGERPRINT_SKIP_DIRS)
        for name in sorted(files):
            if not should_skip_fingerprint_name(name):
                add_fingerprint_file(entries, gym_root, os.path.join(current, name))


def collect_fingerprint_entries(gym_root: str) -> list[tuple[str, bytes]]:
    """지문에 들어가는 (상대경로, 바이트) 목록. 해시는 정렬 뒤 누적한다."""
    entries: list[tuple[str, bytes]] = []
    # `packs`는 과제 선언뿐 아니라 그 과제가 실제로 읽는 asset도 포함한다.
    # 나머지는 report.py가 점수를 재는 코드 경로와 인증서 판정 자체다.
    for rel_dir in FINGERPRINT_TREES:
        add_fingerprint_tree(entries, gym_root, rel_dir)
    for name in FINGERPRINT_FILES:
        add_fingerprint_file(entries, gym_root, os.path.join(gym_root, name))
    return entries


def fingerprint_rel_paths(gym_root: str) -> list[str]:
    return sorted({rel for rel, _data in collect_fingerprint_entries(gym_root)})


def fingerprint_entry_count(gym_root: str) -> int:
    return len(collect_fingerprint_entries(gym_root))


def fingerprint_is_empty(gym_root: str) -> bool:
    return fingerprint_entry_count(gym_root) == 0


def hash_fingerprint_entries(entries: list[tuple[str, bytes]]) -> str:
    h = hashlib.sha256()
    for rel, data in sorted(entries):
        h.update(rel.encode("utf-8"))
        h.update(b"\0")
        h.update(hashlib.sha256(data).digest())
    return h.hexdigest()


def benchmark_fingerprint(gym_root: str) -> str:
    """실제 측정 입력·프로토콜의 결정론적 sha256.

    과제 선언만이 아니라 pack asset, 채점기, 기준 풀이 조립기, 리포트·커버리지
    코드도 결과를 바꾼다. 이 중 하나라도 바뀌면 같은 점수라도 다른 benchmark
    certification으로 취급해야 한다.
    """
    return hash_fingerprint_entries(collect_fingerprint_entries(gym_root))


def mapping_or_empty(value: object) -> dict:
    return value if is_json_object(value) else {}


def coverage_core(coverage: object) -> dict:
    cov = mapping_or_empty(coverage)
    return {k: cov.get(k) for k in COVERAGE_CORE_KEYS}


def reproducible_core(report: dict, fingerprint: str) -> dict:
    """인증서에서 재현으로 대조하는 필드 — 변동 메타(git commit·시각·agent 이름)는 뺀다."""
    report_obj = mapping_or_empty(report)
    runner = mapping_or_empty(report_obj.get("runner"))
    cov = mapping_or_empty(report_obj.get("coverage"))
    return {
        "benchmarkFingerprint": fingerprint,
        "capabilitiesSha256": runner.get("capabilitiesSha256"),
        "accuracy": report_obj.get("accuracy"),
        "coverage": coverage_core(cov),
        "axisProfile": report_obj.get("axisProfile"),
    }


def extract_unavailable(report: object) -> list[str]:
    report_obj = mapping_or_empty(report)
    raw = report_obj.get("packsUnavailable")
    if not is_json_array(raw):
        return []
    ids: list[str] = []
    for item in raw:
        if isinstance(item, str) and item.strip():
            ids.append(item)
    return ids


def exceptions_for_unavailable(pack_ids: list[str]) -> list[dict]:
    notes: list[dict] = []
    for pid in pack_ids:
        notes.append(exception_record(
            "unavailable-pack",
            f"pack {pid} 는 요구 명령 부재로 채점되지 않았다",
            pack=pid,
            where="report.packsUnavailable",
        ))
    return notes


def classify_report_failure(returncode: int, stdout: str, stderr: str) -> CertifyError:
    """report.py 비-0 을 kind 로. 스코어카드 부재를 일반 실패로 뭉개지 않는다."""
    blob = f"{stderr}\n{stdout}"
    for marker in KIND_MARKERS:
        if marker in blob:
            message = f"report.py 실패({marker}): {stderr[:300]}"
            return CertifyError(marker if marker in EXCEPTION_KINDS else "report-tool-failed",
                                message, where="report.py")
    if "scorecard.json" in blob and ("없다" in blob or "남기지" in blob):
        return CertifyError(
            "missing-scorecard",
            f"report.py 실패: 스코어카드 부재 — {stderr[:300]}",
            where="report.py",
        )
    if "JSON" in blob and ("파싱" in blob or "Decode" in blob or "decode" in blob):
        return CertifyError(
            "malformed-json",
            f"report.py 실패: JSON — {stderr[:300]}",
            where="report.py",
        )
    if returncode != 0:
        return CertifyError(
            "report-tool-failed",
            f"report.py 실패: {stderr[:300]}",
            where="report.py",
        )
    return CertifyError("report-tool-failed", "report.py 가 빈 산출을 냈다",
                        where="report.py")


def load_report_json(text: str | bytes) -> dict:
    data = parse_json_text(text, role="report")
    if not is_json_object(data):
        raise CertifyError("malformed-report",
                           "report.py stdout 이 JSON 객체가 아니다",
                           role="report")
    return data


def _run_report(bin_path: str) -> dict:
    if not isinstance(bin_path, str) or not bin_path.strip():
        raise CertifyError("missing-bin", "바이너리 경로가 비었다", role="bin")
    out = subprocess.run(
        [sys.executable, os.path.join(GYM_ROOT, "report.py"), "--bin", bin_path, "--json"],
        cwd=REPO_ROOT, capture_output=True,
    )
    stderr = out.stderr.decode("utf-8", "replace")
    stdout = out.stdout.decode("utf-8", "replace")
    if out.returncode != 0:
        raise classify_report_failure(out.returncode, stdout, stderr)
    if not stdout.strip():
        raise CertifyError("malformed-report", "report.py stdout 이 비었다",
                           role="report")
    return load_report_json(stdout)


def validate_cert(cert: object) -> list[str]:
    """인증서 뼈대. 실패 이유를 사람 문구 목록으로 낸다(verify 계약)."""
    if not is_json_object(cert):
        return ["인증서가 JSON 객체가 아니다"]
    diffs: list[str] = []
    if cert.get("kind") != CERT_KIND:
        diffs.append(f"kind 가 {CERT_KIND} 가 아니다: {cert.get('kind')}")
    report = cert.get("report")
    if report is None:
        diffs.append("인증서에 report 칸이 없다")
    elif not is_json_object(report):
        diffs.append("인증서 report 가 JSON 객체가 아니다")
    fp = cert.get("benchmarkFingerprint")
    if not isinstance(fp, str) or not fp:
        diffs.append("인증서 benchmarkFingerprint 가 비었다")
    return diffs


def compare_core(claimed: dict, fresh: dict) -> list[str]:
    """재현 core 다섯 칸. 문구는 예전 시험을 깨지 않게 접두를 유지한다."""
    diffs: list[str] = []
    if claimed.get("benchmarkFingerprint") != fresh.get("benchmarkFingerprint"):
        diffs.append("벤치마크 지문 불일치 — pack 정의가 인증 시점과 다르다(축소·변조 가능)")
    if claimed.get("capabilitiesSha256") != fresh.get("capabilitiesSha256"):
        diffs.append("바이너리 신원(capabilitiesSha256) 불일치 — 다른 바이너리다")
    if claimed.get("accuracy") != fresh.get("accuracy"):
        diffs.append(f"정확도 불일치: 인증 {claimed.get('accuracy')} vs 재현 {fresh.get('accuracy')}")
    if claimed.get("coverage") != fresh.get("coverage"):
        diffs.append(f"커버리지 불일치: 인증 {claimed.get('coverage')} vs 재현 {fresh.get('coverage')}")
    if claimed.get("axisProfile") != fresh.get("axisProfile"):
        diffs.append("축별 프로파일 불일치")
    return diffs


def compare_unavailable(claimed_ids: list[str], fresh_ids: list[str]) -> list[str]:
    if set(claimed_ids) != set(fresh_ids):
        return [f"미가용 pack 불일치: 인증 {sorted(claimed_ids)} vs 재현 {sorted(fresh_ids)}"]
    return []


def attach_cert_exceptions(cert: dict, notes: list[dict]) -> dict:
    cert["exceptions"] = notes
    cert["exceptionCount"] = len(notes)
    cert["trusted"] = len(structural_exceptions(notes)) == 0
    return cert


def certify(bin_path: str, measured_at: str | None = None) -> dict:
    report = _run_report(bin_path)
    fp = benchmark_fingerprint(GYM_ROOT)
    unavailable = extract_unavailable(report)
    notes = exceptions_for_unavailable(unavailable)
    if fingerprint_is_empty(GYM_ROOT):
        notes.append(exception_record(
            "fingerprint-empty",
            "벤치마크 지문에 넣을 측정 입력이 없다",
            where="benchmark_fingerprint",
        ))
    cert = {
        "kind": CERT_KIND,
        "schemaVersion": CERT_SCHEMA,
        "benchmarkFingerprint": fp,
        "report": report,
        "proof": PROOF_TEXT,
        "unavailablePacks": unavailable,
    }
    attach_cert_exceptions(cert, notes)
    if measured_at:
        cert["certifiedAt"] = measured_at
    return cert


def verify(cert: dict, bin_path: str) -> tuple[bool, list[str]]:
    """인증서를 재발급해 재현 core 를 대조한다 — 위조·환경 변화를 잡는다.

    예전 계약: 성공은 (True, []), 실패는 (False, 이유 목록). 깨진 인증서는
    예외를 올리지 않고 False 로 접는다. report.py 운영 실패는 kind 를 이유에 넣는다.
    """
    skeleton = validate_cert(cert)
    if skeleton and (not is_json_object(cert) or cert.get("kind") != CERT_KIND
                     or not is_json_object(cert.get("report"))):
        # kind 불일치는 예전 문구를 그대로 쓴다.
        if is_json_object(cert) and cert.get("kind") != CERT_KIND:
            return False, [f"kind 가 {CERT_KIND} 가 아니다: {cert.get('kind')}"]
        return False, skeleton
    report = mapping_or_empty(cert.get("report") if is_json_object(cert) else None)
    claimed = reproducible_core(report, cert.get("benchmarkFingerprint", "") if is_json_object(cert) else "")
    try:
        fresh_report = _run_report(bin_path)
        fresh = reproducible_core(fresh_report, benchmark_fingerprint(GYM_ROOT))
    except CertifyError as e:
        return False, [f"{e.kind}: {e}"]
    diffs = compare_core(claimed, fresh)
    diffs.extend(compare_unavailable(extract_unavailable(report),
                                     extract_unavailable(fresh_report)))
    return (len(diffs) == 0), diffs


def load_cert(path: str) -> dict:
    return load_json_object(path, role="cert")


def dump_cert_json(cert: dict) -> str:
    return json.dumps(cert, ensure_ascii=False, indent=2) + "\n"


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(description="gym 능력 인증서 — 발급/재현 검증")
    ap.add_argument("--bin", required=True, help="rhwp 바이너리")
    ap.add_argument("--verify", help="검증할 인증서 JSON")
    ap.add_argument("--out", help="발급 인증서 출력 파일(생략 시 stdout)")
    ap.add_argument("--at", help="certifiedAt 메타(재현 core 에 미포함)")
    return ap


def cli_flag_names(parser: argparse.ArgumentParser | None = None) -> tuple[str, ...]:
    ap = parser or build_parser()
    names: list[str] = []
    for action in ap._actions:
        for opt in action.option_strings:
            if opt.startswith("--") and opt not in ("--help",):
                names.append(opt)
    return tuple(names)


def format_issue_summary(cert: dict) -> str:
    report = mapping_or_empty(cert.get("report"))
    accuracy = mapping_or_empty(report.get("accuracy"))
    pct = accuracy.get("percent")
    fp = str(cert.get("benchmarkFingerprint") or "")
    return f"정확도 {pct}% · 지문 {fp[:12]}"


def main(argv: list[str] | None = None) -> int:
    ap = build_parser()
    a = ap.parse_args(argv)
    try:
        if a.verify:
            cert = load_cert(a.verify)
            ok, diffs = verify(cert, a.bin)
            if ok:
                print(VERIFY_PASS_MESSAGE)
                return EXIT_OK
            print(VERIFY_FAIL_PREFIX)
            for item in diffs:
                print(f"  - {item}")
            return EXIT_VERIFY_FAIL

        cert = certify(a.bin, a.at)
        text = dump_cert_json(cert)
        if a.out:
            write_text(a.out, text)
            print(f"발급: {a.out} · {format_issue_summary(cert)}")
        else:
            sys.stdout.write(text)
        return EXIT_OK
    except CertifyError as e:
        print(f"{e.kind}: {e}", file=sys.stderr)
        return EXIT_TOOL_FAILED
    except CATCHABLE_EXCEPTIONS as e:
        wrapped = wrap_exception(e, role="cert", where="main")
        print(f"{wrapped.kind}: {wrapped}", file=sys.stderr)
        return EXIT_TOOL_FAILED


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
