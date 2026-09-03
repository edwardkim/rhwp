"""Gym 실행 증적을 검증·seal하고 사람용 파생 보고서를 생성한다 (#6669).

입력 계약, 역할별 정직 판정, 실행 신원 교차검사와 deterministic
``evidence-manifest.json``을 제공한다. 별도 HTML renderer는 이 모듈의 검증된
bundle만 소비한다. JSON 봉투가 판정 정본이며 manifest는 입력 파일 집합의 영수증이다.

사용::

    python3 gym/tools/evidence_report.py --evidence-dir <dir> --seal
    python3 gym/tools/evidence_report.py --evidence-dir <dir> --out <report.html>
"""

from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import tempfile


HERE = Path(__file__).resolve().parent
GYM_ROOT = HERE.parent

MANIFEST_KIND = "gymEvidenceManifest"
MANIFEST_SCHEMA_VERSION = "1.0"
GENERATOR_NAME = "gymEvidenceReport"
GENERATOR_VERSION = "1.0.0"
ERROR_KIND = "gymEvidenceReportError"

EXIT_OK = 0
EXIT_RESULT_NOT_PASS = 1
EXIT_INPUT_INVALID = 2

STATUS_PASS = "PASS"
STATUS_FAIL = "FAIL"
STATUS_INCOMPLETE = "INCOMPLETE"
STATUS_NOT_APPLICABLE = "NOT_APPLICABLE"

MAX_JSON_BYTES = 64 * 1024 * 1024
MAX_AUX_BYTES = 16 * 1024 * 1024

OID_RE = re.compile(r"^[0-9a-fA-F]{40,64}$")
SHA256_RE = re.compile(r"^[0-9a-fA-F]{64}$")
RUN_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
UINT_RE = re.compile(r"^(0|[1-9][0-9]*)$")
SHA256SUM_RE = re.compile(r"^([0-9a-fA-F]{64})[ \t]+\*?(.+)$")

ROLE_SPECS = (
    ("audit", "audit.json", "gymAudit", None),
    ("oracleStructural", "oracle-structural.json", "gymOracleProbe", "structural"),
    ("oracleSelftest", "oracle-selftest.json", "gymOracleProbe", "selftest"),
    ("authorityLedger", "authority-ledger.json", "gymAuthorityLedger", None),
    ("positive", "positive.json", "gymBaselineVerification", None),
    ("discrimination", "discrimination.json", "gymDiscrimination", None),
    ("trajectory", "trajectory.json", "gymTrajectoryNecessity", None),
)

METADATA_FILES = (
    "run-id.txt",
    "gym-runner-head.txt",
    "gym-runner-tree.txt",
    "product-source-head.txt",
    "rhwp-version.txt",
    "rhwp-bin.sha256",
    "run-started.txt",
    "platform.txt",
    "python-version.txt",
    "rust-version.txt",
)

ROLE_BASES = (
    "audit",
    "oracle-structural",
    "oracle-selftest",
    "authority-ledger",
    "positive",
    "discrimination",
    "trajectory",
)

ROLE_SIDECARS = tuple(
    name
    for base in ROLE_BASES
    for name in (f"{base}.stderr", f"{base}.exit", f"{base}.seconds")
)

UNIT_FILES = ("unit.txt", "unit.exit", "unit.seconds")
JSON_FILES = tuple(spec[1] for spec in ROLE_SPECS)
REQUIRED_INPUT_FILES = tuple(sorted(METADATA_FILES + ROLE_SIDECARS + UNIT_FILES + JSON_FILES))


class DuplicateJsonKey(ValueError):
    """JSON object에 같은 key가 두 번 나온 경우."""


class EvidenceError(Exception):
    """증적 입력을 신뢰할 수 없어 seal/report 생성을 거부한다."""

    def __init__(self, errors: list[dict]):
        self.errors = sorted(
            errors,
            key=lambda row: (
                str(row.get("file", "")),
                str(row.get("code", "")),
                str(row.get("message", "")),
            ),
        )
        super().__init__(f"Gym evidence input invalid: {len(self.errors)} error(s)")


def error_record(code: str, *, file: str = "", message: str = "") -> dict:
    row = {"code": code, "message": message or code}
    if file:
        row["file"] = file
    return row


def _tool_module(name: str):
    """기존 Gym 도구의 순수 validator를 exact 경로에서 불러온다."""
    cache_name = f"_gym_evidence_{name}"
    cached = sys.modules.get(cache_name)
    if cached is not None:
        return cached
    path = HERE / f"{name}.py"
    spec = importlib.util.spec_from_file_location(cache_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Gym validator를 불러올 수 없다: {name}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[cache_name] = module
    spec.loader.exec_module(module)
    return module


def _renderer_module():
    """표현 책임을 분리한 renderer를 저장소의 exact 경로에서 불러온다."""
    cache_name = "_gym_evidence_html"
    cached = sys.modules.get(cache_name)
    if cached is not None:
        return cached
    path = GYM_ROOT / "core" / "evidence_html.py"
    spec = importlib.util.spec_from_file_location(cache_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError("Gym evidence HTML renderer를 불러올 수 없다")
    module = importlib.util.module_from_spec(spec)
    sys.modules[cache_name] = module
    spec.loader.exec_module(module)
    return module


def _pairs_without_duplicates(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise DuplicateJsonKey(f"중복 JSON key: {key}")
        obj[key] = value
    return obj


def _reject_nonfinite(token: str):
    raise ValueError(f"비유한 JSON 숫자: {token}")


def canonical_json_bytes(value) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode("utf-8")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def is_nonnegative_int(value) -> bool:
    return type(value) is int and value >= 0


def _file_limit(name: str) -> int:
    return MAX_JSON_BYTES if name in JSON_FILES else MAX_AUX_BYTES


def _read_required_inputs(root: Path) -> tuple[dict[str, bytes], list[dict]]:
    files = {}
    errors = []
    for name in REQUIRED_INPUT_FILES:
        path = root / name
        try:
            if path.is_symlink():
                errors.append(error_record(
                    "input-symlink", file=name,
                    message="필수 입력은 증적 디렉터리 밖을 가리키는 symlink일 수 없다",
                ))
                continue
            if not path.is_file():
                errors.append(error_record(
                    "missing-input", file=name, message="필수 증적 파일이 없다",
                ))
                continue
            size = path.stat().st_size
            limit = _file_limit(name)
            if size > limit:
                errors.append(error_record(
                    "oversized-input", file=name,
                    message=f"입력 크기 {size}가 상한 {limit}을 넘는다",
                ))
                continue
            data = path.read_bytes()
            if len(data) != size:
                errors.append(error_record(
                    "input-size-drift", file=name,
                    message="stat 이후 파일 크기가 바뀌었다",
                ))
                continue
            files[name] = data
        except OSError as exc:
            errors.append(error_record(
                "input-read", file=name,
                message=f"필수 증적 파일을 읽을 수 없다: {type(exc).__name__}",
            ))
    return files, errors


def _decode_text(data: bytes, name: str, errors: list[dict]) -> str | None:
    if data.startswith(b"\xef\xbb\xbf"):
        errors.append(error_record(
            "unexpected-bom", file=name, message="UTF-8 BOM을 허용하지 않는다",
        ))
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        errors.append(error_record(
            "invalid-utf8", file=name, message="UTF-8로 해석할 수 없다",
        ))
        return None


def _one_line(files: dict[str, bytes], name: str, errors: list[dict]) -> str | None:
    text = _decode_text(files[name], name, errors)
    if text is None:
        return None
    value = text.rstrip("\r\n")
    if not value or "\n" in value or "\r" in value or "\x00" in value:
        errors.append(error_record(
            "invalid-metadata", file=name,
            message="metadata는 NUL 없는 비어 있지 않은 한 줄이어야 한다",
        ))
        return None
    return value


def _unsigned(files: dict[str, bytes], name: str, errors: list[dict]) -> int | None:
    value = _one_line(files, name, errors)
    if value is None:
        return None
    if not UINT_RE.fullmatch(value):
        errors.append(error_record(
            "invalid-unsigned", file=name, message="음이 아닌 정수 한 줄이어야 한다",
        ))
        return None
    return int(value)


def _json_object(files: dict[str, bytes], name: str, errors: list[dict]) -> dict | None:
    text = _decode_text(files[name], name, errors)
    if text is None:
        return None
    try:
        value = json.loads(
            text,
            object_pairs_hook=_pairs_without_duplicates,
            parse_constant=_reject_nonfinite,
        )
    except (json.JSONDecodeError, DuplicateJsonKey, ValueError, RecursionError) as exc:
        errors.append(error_record(
            "malformed-json", file=name,
            message=f"JSON을 결정적으로 해석할 수 없다: {type(exc).__name__}",
        ))
        return None
    if not isinstance(value, dict):
        errors.append(error_record(
            "json-not-object", file=name, message="최상위 JSON 봉투가 객체가 아니다",
        ))
        return None
    return value


def _validate_metadata(files: dict[str, bytes], errors: list[dict]) -> dict:
    values = {name: _one_line(files, name, errors) for name in METADATA_FILES}

    run_id = values["run-id.txt"]
    if run_id is not None and not RUN_ID_RE.fullmatch(run_id):
        errors.append(error_record(
            "invalid-run-id", file="run-id.txt", message="실행 ID 형식이 계약과 다르다",
        ))

    for name in ("gym-runner-head.txt", "gym-runner-tree.txt", "product-source-head.txt"):
        value = values[name]
        if value is not None and not OID_RE.fullmatch(value):
            errors.append(error_record(
                "invalid-oid", file=name, message="40~64자리 hex OID가 아니다",
            ))

    started = values["run-started.txt"]
    if started is not None:
        try:
            parsed = datetime.fromisoformat(started.replace("Z", "+00:00"))
            if parsed.tzinfo is None:
                raise ValueError("timezone missing")
        except ValueError:
            errors.append(error_record(
                "invalid-started-at", file="run-started.txt",
                message="timezone이 있는 ISO-8601 시각이 아니다",
            ))

    digest = None
    bin_path = None
    sha_line = values["rhwp-bin.sha256"]
    if sha_line is not None:
        match = SHA256SUM_RE.fullmatch(sha_line)
        if match is None:
            errors.append(error_record(
                "invalid-binary-sha", file="rhwp-bin.sha256",
                message="SHA-256과 실행 당시 binary path 형식이 아니다",
            ))
        else:
            digest = match.group(1).lower()
            bin_path = match.group(2)
            if not bin_path.strip():
                errors.append(error_record(
                    "invalid-binary-path", file="rhwp-bin.sha256",
                    message="binary path가 비었다",
                ))

    return {
        "runId": run_id,
        "runnerHead": values["gym-runner-head.txt"],
        "runnerTree": values["gym-runner-tree.txt"],
        "productSourceHead": values["product-source-head.txt"],
        "rhwpVersion": values["rhwp-version.txt"],
        "binarySha256": digest,
        "binaryPath": bin_path,
        "runStarted": started,
        "platform": values["platform.txt"],
        "pythonVersion": values["python-version.txt"],
        "rustVersion": values["rust-version.txt"],
    }


def _require_nonnegative(report: dict, keys: tuple[str, ...], name: str, errors: list[dict]):
    for key in keys:
        if not is_nonnegative_int(report.get(key)):
            errors.append(error_record(
                "invalid-envelope", file=name,
                message=f"{key}가 음이 아닌 정수가 아니다",
            ))


def _validate_envelope_header(
    report: dict,
    *,
    name: str,
    expected_kind: str,
    expected_mode: str | None,
    errors: list[dict],
):
    if report.get("kind") != expected_kind:
        errors.append(error_record(
            "kind-mismatch", file=name,
            message=f"kind가 {expected_kind}가 아니다",
        ))
    if report.get("schemaVersion") != "1.0":
        errors.append(error_record(
            "schema-mismatch", file=name, message="schemaVersion이 1.0이 아니다",
        ))
    if expected_mode is not None and report.get("mode") != expected_mode:
        errors.append(error_record(
            "mode-mismatch", file=name,
            message=f"mode가 {expected_mode}가 아니다",
        ))
    if not isinstance(report.get("ok"), bool):
        errors.append(error_record(
            "invalid-envelope", file=name, message="ok가 bool이 아니다",
        ))


def _validator_errors(module_name: str, report: dict) -> list[str]:
    module = _tool_module(module_name)
    return list(module.validate_report(report))


def _validate_audit(report: dict, name: str, errors: list[dict]):
    for detail in _validator_errors("audit", report):
        errors.append(error_record("invalid-envelope", file=name, message=detail))
    _require_nonnegative(
        report,
        ("packCount", "taskCount", "referenceCount", "issueCount", "exit"),
        name,
        errors,
    )
    packs = report.get("packs")
    if not isinstance(packs, list):
        return
    if is_nonnegative_int(report.get("packCount")) and report["packCount"] != len(packs):
        errors.append(error_record(
            "invalid-envelope", file=name, message="packCount가 packs 길이와 다르다",
        ))
    ids = []
    task_sum = 0
    reference_sum = 0
    for row in packs:
        if not isinstance(row, dict) or not isinstance(row.get("id"), str) or not row["id"]:
            errors.append(error_record(
                "invalid-envelope", file=name, message="packs 행의 id가 유효하지 않다",
            ))
            continue
        ids.append(row["id"])
        if not is_nonnegative_int(row.get("taskCount")) or not is_nonnegative_int(row.get("referenceCount")):
            errors.append(error_record(
                "invalid-envelope", file=name, message="packs 행의 task/reference 집계가 유효하지 않다",
            ))
            continue
        task_sum += row["taskCount"]
        reference_sum += row["referenceCount"]
    if len(ids) != len(set(ids)):
        errors.append(error_record(
            "invalid-envelope", file=name, message="packs id가 중복된다",
        ))
    if is_nonnegative_int(report.get("taskCount")) and task_sum != report["taskCount"]:
        errors.append(error_record(
            "invalid-envelope", file=name, message="taskCount가 packs 집계와 다르다",
        ))
    if is_nonnegative_int(report.get("referenceCount")) and reference_sum != report["referenceCount"]:
        errors.append(error_record(
            "invalid-envelope", file=name, message="referenceCount가 packs 집계와 다르다",
        ))


def _validate_oracle(report: dict, name: str, expected_mode: str, errors: list[dict]):
    _require_nonnegative(report, ("issueCount",), name, errors)
    issues = report.get("issues") if expected_mode == "structural" else report.get("failed")
    if not isinstance(issues, list):
        errors.append(error_record(
            "invalid-envelope", file=name,
            message="oracle issue/failed 목록이 list가 아니다",
        ))
        return
    if is_nonnegative_int(report.get("issueCount")) and report["issueCount"] != len(issues):
        errors.append(error_record(
            "invalid-envelope", file=name, message="oracle issueCount가 목록 길이와 다르다",
        ))
    expected_ok = len(issues) == 0
    if isinstance(report.get("ok"), bool) and report["ok"] != expected_ok:
        errors.append(error_record(
            "invalid-envelope", file=name, message="oracle ok가 issue/failed 목록과 다르다",
        ))
    if expected_mode == "selftest":
        checks = report.get("checks")
        if not isinstance(checks, list) or not is_nonnegative_int(report.get("checkCount")):
            errors.append(error_record(
                "invalid-envelope", file=name, message="selftest check 목록·집계가 유효하지 않다",
            ))
        elif report["checkCount"] != len(checks):
            errors.append(error_record(
                "invalid-envelope", file=name, message="checkCount가 checks 길이와 다르다",
            ))


def _validate_authority(report: dict, name: str, errors: list[dict]):
    _require_nonnegative(
        report,
        ("taskCount", "referenceCount", "entryCount", "issueCount", "exit"),
        name,
        errors,
    )
    entries = report.get("entries")
    issues = report.get("issues")
    summary = report.get("summary")
    if not isinstance(entries, list) or not isinstance(issues, list) or not isinstance(summary, dict):
        errors.append(error_record(
            "invalid-envelope", file=name,
            message="authority entries/issues/summary 형식이 유효하지 않다",
        ))
        return
    if is_nonnegative_int(report.get("entryCount")) and report["entryCount"] != len(entries):
        errors.append(error_record(
            "invalid-envelope", file=name, message="entryCount가 entries 길이와 다르다",
        ))
    if is_nonnegative_int(report.get("issueCount")) and report["issueCount"] != len(issues):
        errors.append(error_record(
            "invalid-envelope", file=name, message="issueCount가 issues 길이와 다르다",
        ))
    expected_summary = _tool_module("authority_ledger").recompute_summary(entries)
    if summary != expected_summary:
        errors.append(error_record(
            "invalid-envelope", file=name, message="summary가 entries 재계산과 다르다",
        ))
    keys = [row.get("key") for row in entries if isinstance(row, dict)]
    if len(keys) != len(entries) or any(not isinstance(key, str) or not key for key in keys):
        errors.append(error_record(
            "invalid-envelope", file=name, message="authority entry key가 유효하지 않다",
        ))
    elif len(keys) != len(set(keys)):
        errors.append(error_record(
            "invalid-envelope", file=name, message="authority entry key가 중복된다",
        ))
    expected_ok = not issues and report.get("taskCount") == report.get("entryCount")
    if isinstance(report.get("ok"), bool) and report["ok"] != expected_ok:
        errors.append(error_record(
            "invalid-envelope", file=name, message="authority ok가 issue/entry 집계와 다르다",
        ))
    expected_exit = 0 if expected_ok else 1
    if report.get("exit") != expected_exit:
        errors.append(error_record(
            "invalid-envelope", file=name, message="authority exit가 판정과 다르다",
        ))


def _validate_positive(report: dict, name: str, errors: list[dict]):
    count_keys = (
        "taskCount", "built", "failed", "skipped", "missingArtifact",
        "failedScore", "buildError", "exit",
    )
    _require_nonnegative(report, count_keys, name, errors)
    if not isinstance(report.get("agent"), str) or not report["agent"]:
        errors.append(error_record(
            "invalid-envelope", file=name, message="agent가 비어 있지 않은 문자열이 아니다",
        ))
    if not isinstance(report.get("binPath"), str) or not report["binPath"]:
        errors.append(error_record(
            "invalid-envelope", file=name, message="binPath가 비어 있지 않은 문자열이 아니다",
        ))
    packs = report.get("packs")
    results = report.get("results")
    if not isinstance(packs, list) or not packs or any(not isinstance(item, str) or not item for item in packs):
        errors.append(error_record(
            "invalid-envelope", file=name, message="packs가 비어 있지 않은 문자열 목록이 아니다",
        ))
        packs = []
    elif len(packs) != len(set(packs)):
        errors.append(error_record(
            "invalid-envelope", file=name, message="positive pack id가 중복된다",
        ))
    if not isinstance(results, list):
        errors.append(error_record(
            "invalid-envelope", file=name, message="results가 list가 아니다",
        ))
        return
    if is_nonnegative_int(report.get("taskCount")) and report["taskCount"] != len(results):
        errors.append(error_record(
            "invalid-envelope", file=name, message="taskCount가 results 길이와 다르다",
        ))
    kinds = Counter()
    keys = []
    built = 0
    failed = 0
    skipped = 0
    for row in results:
        if not isinstance(row, dict):
            errors.append(error_record(
                "invalid-envelope", file=name, message="positive result 행이 객체가 아니다",
            ))
            continue
        pack = row.get("pack")
        task = row.get("task")
        kind = row.get("kind")
        if not isinstance(pack, str) or not pack or not isinstance(task, str) or not task:
            errors.append(error_record(
                "invalid-envelope", file=name, message="positive result pack/task가 유효하지 않다",
            ))
            continue
        if packs and pack not in packs:
            errors.append(error_record(
                "invalid-envelope", file=name, message="result pack이 packs 목록에 없다",
            ))
        keys.append((pack, task))
        if kind not in ("ok", "missing-reference", "missing-artifact", "failed-score", "build-error"):
            errors.append(error_record(
                "invalid-envelope", file=name, message="positive result kind가 카탈로그 밖이다",
            ))
            continue
        kinds[kind] += 1
        if kind == "ok" and row.get("ok") is True:
            built += 1
        elif kind == "missing-reference" and row.get("ok") is False:
            skipped += 1
        elif kind != "ok" and row.get("ok") is False:
            failed += 1
        else:
            errors.append(error_record(
                "invalid-envelope", file=name, message="positive result ok와 kind가 모순된다",
            ))
    if len(keys) != len(set(keys)):
        errors.append(error_record(
            "invalid-envelope", file=name, message="positive pack/task가 중복된다",
        ))
    expected = {
        "built": built,
        "failed": failed,
        "skipped": skipped,
        "missingArtifact": kinds["missing-artifact"],
        "failedScore": kinds["failed-score"],
        "buildError": kinds["build-error"],
    }
    for key, value in expected.items():
        if report.get(key) != value:
            errors.append(error_record(
                "invalid-envelope", file=name,
                message=f"{key}가 result 재계산과 다르다",
            ))
    expected_ok = bool(results) and built == len(results) and failed == 0 and skipped == 0
    if isinstance(report.get("ok"), bool) and report["ok"] != expected_ok:
        errors.append(error_record(
            "invalid-envelope", file=name, message="positive ok가 result 집계와 다르다",
        ))
    expected_exit = 0 if expected_ok else 1
    if report.get("exit") != expected_exit:
        errors.append(error_record(
            "invalid-envelope", file=name, message="positive exit가 판정과 다르다",
        ))


def _validate_discrimination(report: dict, name: str, errors: list[dict]):
    for detail in _validator_errors("discriminate", report):
        errors.append(error_record("invalid-envelope", file=name, message=detail))
    results = report.get("results")
    if not isinstance(results, list):
        return
    keys = []
    for row in results:
        if not isinstance(row, dict):
            continue
        pack = row.get("pack")
        task = row.get("task")
        control = row.get("control")
        if not all(isinstance(item, str) and item for item in (pack, task, control)):
            errors.append(error_record(
                "invalid-envelope", file=name,
                message="discrimination result pack/task/control이 유효하지 않다",
            ))
            continue
        if not isinstance(row.get("discriminates"), bool):
            errors.append(error_record(
                "invalid-envelope", file=name,
                message="discriminates가 bool이 아니다",
            ))
        if "error" in row and not isinstance(row.get("error"), str):
            errors.append(error_record(
                "invalid-envelope", file=name,
                message="result error가 문자열이 아니다",
            ))
        keys.append((pack, task, control))
    if len(keys) != len(set(keys)):
        errors.append(error_record(
            "invalid-envelope", file=name,
            message="discrimination pack/task/control이 중복된다",
        ))
    for key in ("loadErrors", "scoreErrors", "buildErrors", "skipped", "toolErrors"):
        values = report.get(key, [])
        if not isinstance(values, list) or any(not isinstance(item, (str, dict)) for item in values):
            errors.append(error_record(
                "invalid-envelope", file=name, message=f"{key}가 유효한 목록이 아니다",
            ))
    if not isinstance(report.get("toolFailed", False), bool):
        errors.append(error_record(
            "invalid-envelope", file=name, message="toolFailed가 bool이 아니다",
        ))


def _validate_trajectory(report: dict, name: str, errors: list[dict]):
    for detail in _validator_errors("trajectory", report):
        errors.append(error_record("invalid-envelope", file=name, message=detail))
    for key in ("taskCount", "loadBearing", "exceptionCount", "skipCount", "exit"):
        if key in report and not is_nonnegative_int(report.get(key)):
            errors.append(error_record(
                "invalid-envelope", file=name, message=f"{key}가 음이 아닌 정수가 아니다",
            ))
    results = report.get("results", [])
    skipped = report.get("skipped", [])
    if not isinstance(results, list) or not isinstance(skipped, list):
        errors.append(error_record(
            "invalid-envelope", file=name, message="trajectory results/skipped가 list가 아니다",
        ))
        return
    keys = []
    for row in results:
        if not isinstance(row, dict):
            errors.append(error_record(
                "invalid-envelope", file=name, message="trajectory result 행이 객체가 아니다",
            ))
            continue
        pack = row.get("pack")
        task = row.get("task")
        if not isinstance(pack, str) or not pack or not isinstance(task, str) or not task:
            errors.append(error_record(
                "invalid-envelope", file=name, message="trajectory result pack/task가 유효하지 않다",
            ))
            continue
        keys.append((pack, task))
    if len(keys) != len(set(keys)):
        errors.append(error_record(
            "invalid-envelope", file=name, message="trajectory pack/task가 중복된다",
        ))


def _validate_role_documents(files: dict[str, bytes], errors: list[dict]) -> dict[str, dict]:
    documents = {}
    for role, name, kind, mode in ROLE_SPECS:
        report = _json_object(files, name, errors)
        if report is None:
            continue
        documents[role] = report
        _validate_envelope_header(
            report,
            name=name,
            expected_kind=kind,
            expected_mode=mode,
            errors=errors,
        )
        if role == "audit":
            _validate_audit(report, name, errors)
        elif role.startswith("oracle"):
            _validate_oracle(report, name, mode or "", errors)
        elif role == "authorityLedger":
            _validate_authority(report, name, errors)
        elif role == "positive":
            _validate_positive(report, name, errors)
        elif role == "discrimination":
            _validate_discrimination(report, name, errors)
        elif role == "trajectory":
            _validate_trajectory(report, name, errors)
    return documents


def _validate_process_sidecars(files: dict[str, bytes], documents: dict, errors: list[dict]) -> dict:
    processes = {
        "unit": {
            "exit": _unsigned(files, "unit.exit", errors),
            "seconds": _unsigned(files, "unit.seconds", errors),
        }
    }
    role_by_base = {spec[1][:-5]: spec[0] for spec in ROLE_SPECS}
    for base in ROLE_BASES:
        role = role_by_base[base]
        process_exit = _unsigned(files, f"{base}.exit", errors)
        seconds = _unsigned(files, f"{base}.seconds", errors)
        processes[role] = {"exit": process_exit, "seconds": seconds}
        report = documents.get(role)
        if report is None or process_exit is None:
            continue
        if role in ("audit", "authorityLedger", "positive", "trajectory"):
            if report.get("exit") != process_exit:
                errors.append(error_record(
                    "process-exit-mismatch", file=f"{base}.exit",
                    message="process exit가 JSON envelope exit와 다르다",
                ))
        else:
            expected = 0 if report.get("ok") is True else 1
            if process_exit != expected:
                errors.append(error_record(
                    "process-exit-mismatch", file=f"{base}.exit",
                    message="process exit가 JSON envelope 판정과 다르다",
                ))
    return processes


def _score_error_accounting(report: dict) -> dict:
    reported = Counter(report.get("scoreErrors") or [])
    intended = Counter()
    row_errors = Counter()
    unsafe_rows = []
    for row in report.get("results") or []:
        if not isinstance(row, dict) or not row.get("error"):
            continue
        text = f"{row.get('pack')}/{row.get('task')} ({row.get('control')}): {row['error']}"
        row_errors[text] += 1
        if row.get("discriminates") is True:
            intended[text] += 1
        else:
            unsafe_rows.append(text)

    matched = reported & intended
    unexpected_reported = reported - intended
    missing_reported = row_errors - reported
    unexplained = []
    for text, count in sorted(unexpected_reported.items()):
        unexplained.append({"kind": "unmatched-score-error", "text": text, "count": count})
    for text, count in sorted(missing_reported.items()):
        unexplained.append({"kind": "missing-score-error-record", "text": text, "count": count})
    for text in sorted(unsafe_rows):
        unexplained.append({"kind": "error-row-false-pass", "text": text, "count": 1})
    return {
        "reportedCount": sum(reported.values()),
        "intendedCount": sum(matched.values()),
        "unexplainedCount": sum(item["count"] for item in unexplained),
        "unexplained": unexplained,
    }


def _role_statuses(documents: dict, processes: dict, score_accounting: dict) -> dict:
    statuses = {}

    unit_exit = processes["unit"]["exit"]
    statuses["unit"] = {
        "status": STATUS_PASS if unit_exit == 0 else STATUS_INCOMPLETE,
        "reasons": [] if unit_exit == 0 else ["unit process exit가 0이 아니다"],
    }

    audit = documents["audit"]
    if audit.get("toolFailed") or audit.get("missingPacksRoot"):
        statuses["audit"] = {"status": STATUS_INCOMPLETE, "reasons": ["audit tool failure"]}
    elif audit.get("ok") is not True:
        statuses["audit"] = {"status": STATUS_FAIL, "reasons": ["audit issue가 있다"]}
    else:
        statuses["audit"] = {"status": STATUS_PASS, "reasons": []}

    for role in ("oracleStructural", "oracleSelftest"):
        report = documents[role]
        statuses[role] = {
            "status": STATUS_PASS if report.get("ok") is True else STATUS_FAIL,
            "reasons": [] if report.get("ok") is True else ["oracle probe가 실패했다"],
        }

    authority = documents["authorityLedger"]
    statuses["authorityLedger"] = {
        "status": STATUS_PASS if authority.get("ok") is True else STATUS_FAIL,
        "reasons": [] if authority.get("ok") is True else ["authority issue가 있다"],
    }

    positive = documents["positive"]
    statuses["positive"] = {
        "status": STATUS_PASS if positive.get("ok") is True else STATUS_FAIL,
        "reasons": [] if positive.get("ok") is True else ["positive baseline 실패가 있다"],
    }

    discrimination = documents["discrimination"]
    incomplete_reasons = []
    for key in ("loadErrors", "buildErrors", "toolErrors", "skipped"):
        if discrimination.get(key):
            incomplete_reasons.append(f"{key}가 비어 있지 않다")
    if discrimination.get("toolFailed"):
        incomplete_reasons.append("toolFailed=true")
    if score_accounting["unexplainedCount"]:
        incomplete_reasons.append("미설명 score error가 있다")
    if incomplete_reasons:
        disc_status = STATUS_INCOMPLETE
    elif discrimination.get("ok") is not True:
        disc_status = STATUS_FAIL
    else:
        disc_status = STATUS_PASS
    statuses["discrimination"] = {"status": disc_status, "reasons": incomplete_reasons}

    trajectory = documents["trajectory"]
    trajectory_incomplete = []
    if trajectory.get("exceptions"):
        trajectory_incomplete.append("trajectory exception이 있다")
    if trajectory.get("toolErrors") or trajectory.get("toolFailed"):
        trajectory_incomplete.append("trajectory tool failure가 있다")
    if trajectory.get("missingBin"):
        trajectory_incomplete.append("binary가 없다")
    unknown_skips = [
        row for row in trajectory.get("skipped") or []
        if not isinstance(row, dict) or row.get("reason") != "single-step"
    ]
    if unknown_skips:
        trajectory_incomplete.append("알 수 없는 trajectory skip이 있다")
    if trajectory.get("trusted") is not True:
        trajectory_incomplete.append("trajectory trusted=false")
    if trajectory_incomplete:
        trajectory_status = STATUS_INCOMPLETE
    elif trajectory.get("ok") is not True:
        trajectory_status = STATUS_FAIL
    else:
        trajectory_status = STATUS_PASS
    statuses["trajectory"] = {
        "status": trajectory_status,
        "reasons": list(dict.fromkeys(trajectory_incomplete)),
        "notApplicable": sum(
            1 for row in trajectory.get("skipped") or []
            if isinstance(row, dict) and row.get("reason") == "single-step"
        ),
    }

    values = [row["status"] for row in statuses.values()]
    if STATUS_INCOMPLETE in values:
        overall = STATUS_INCOMPLETE
    elif STATUS_FAIL in values:
        overall = STATUS_FAIL
    else:
        overall = STATUS_PASS
    return {"overall": overall, "roles": statuses}


def _pack_ids_from_audit(report: dict) -> list[str]:
    return sorted(row["id"] for row in report.get("packs") or [] if isinstance(row, dict) and row.get("id"))


def _validate_identity(metadata: dict, documents: dict, errors: list[dict]):
    positive = documents.get("positive", {})
    discrimination = documents.get("discrimination", {})
    trajectory = documents.get("trajectory", {})
    run_id = metadata.get("runId")
    if run_id and positive.get("agent") != f"maintainer-{run_id}":
        errors.append(error_record(
            "run-identity-mismatch", file="positive.json",
            message="positive agent가 run-id와 대응하지 않는다",
        ))

    bin_paths = [
        positive.get("binPath"),
        discrimination.get("binPath"),
        trajectory.get("binPath"),
    ]
    if any(not isinstance(path, str) or not path for path in bin_paths):
        errors.append(error_record(
            "run-identity-mismatch", message="세 실행 축에 binPath가 모두 있어야 한다",
        ))
    elif len(set(bin_paths)) != 1:
        errors.append(error_record(
            "run-identity-mismatch", message="세 실행 축의 binPath가 서로 다르다",
        ))
    elif metadata.get("binaryPath") != bin_paths[0]:
        errors.append(error_record(
            "run-identity-mismatch", file="rhwp-bin.sha256",
            message="binary hash 기록의 path가 실행 축 binPath와 다르다",
        ))

    audit = documents.get("audit", {})
    authority = documents.get("authorityLedger", {})
    reports = (audit, authority, positive, discrimination)
    all_claim_pass = all(report.get("ok") is True for report in reports)
    if all_claim_pass:
        task_counts = {
            audit.get("taskCount"), authority.get("taskCount"),
            authority.get("entryCount"), positive.get("taskCount"),
            discrimination.get("taskCount"),
        }
        if len(task_counts) != 1:
            errors.append(error_record(
                "run-cardinality-mismatch",
                message="PASS 봉투들의 task/entry 수가 달라 실행 혼합을 배제할 수 없다",
            ))
        reference_counts = {audit.get("referenceCount"), authority.get("referenceCount")}
        if len(reference_counts) != 1:
            errors.append(error_record(
                "run-cardinality-mismatch",
                message="PASS 봉투들의 reference 수가 다르다",
            ))
        audit_packs = _pack_ids_from_audit(audit)
        positive_packs = sorted(positive.get("packs") or [])
        if audit_packs != positive_packs:
            errors.append(error_record(
                "run-cardinality-mismatch",
                message="PASS audit와 positive의 pack 집합이 다르다",
            ))


def _identity(metadata: dict, documents: dict) -> dict:
    bin_path = metadata.get("binaryPath") or ""
    binary_name = re.split(r"[\\/]", bin_path)[-1] if bin_path else ""
    audit = documents["audit"]
    authority = documents["authorityLedger"]
    positive = documents["positive"]
    discrimination = documents["discrimination"]
    trajectory = documents["trajectory"]
    return {
        "runId": metadata["runId"],
        "runnerHead": str(metadata["runnerHead"]).lower(),
        "runnerTree": str(metadata["runnerTree"]).lower(),
        "productSourceHead": str(metadata["productSourceHead"]).lower(),
        "binaryName": binary_name,
        "binarySha256": metadata["binarySha256"],
        "binaryPathSha256": sha256_bytes(bin_path.encode("utf-8")),
        "runStarted": metadata["runStarted"],
        "counts": {
            "auditTasks": audit.get("taskCount"),
            "auditReferences": audit.get("referenceCount"),
            "authorityTasks": authority.get("taskCount"),
            "authorityReferences": authority.get("referenceCount"),
            "authorityEntries": authority.get("entryCount"),
            "positiveTasks": positive.get("taskCount"),
            "discriminationTasks": discrimination.get("taskCount"),
            "discriminationControls": discrimination.get("controlCount"),
            "trajectoryTasks": trajectory.get("taskCount"),
        },
    }


def load_evidence(evidence_dir: str | os.PathLike[str]) -> dict:
    root = Path(evidence_dir)
    errors = []
    try:
        if root.is_symlink() or not root.is_dir():
            raise EvidenceError([
                error_record(
                    "invalid-evidence-dir",
                    message="증적 경로가 실재하는 일반 디렉터리가 아니다",
                )
            ])
    except OSError as exc:
        raise EvidenceError([
            error_record(
                "invalid-evidence-dir",
                message=f"증적 디렉터리를 확인할 수 없다: {type(exc).__name__}",
            )
        ]) from exc

    files, read_errors = _read_required_inputs(root)
    errors.extend(read_errors)
    if read_errors:
        raise EvidenceError(errors)

    metadata = _validate_metadata(files, errors)
    documents = _validate_role_documents(files, errors)
    processes = _validate_process_sidecars(files, documents, errors)
    if len(documents) == len(ROLE_SPECS):
        _validate_identity(metadata, documents, errors)
    if errors:
        raise EvidenceError(errors)

    score_accounting = _score_error_accounting(documents["discrimination"])
    status = _role_statuses(documents, processes, score_accounting)
    identity = _identity(metadata, documents)
    identity_fingerprint = sha256_bytes(canonical_json_bytes(identity))
    return {
        "root": root,
        "files": files,
        "metadata": metadata,
        "documents": documents,
        "processes": processes,
        "scoreErrorAccounting": score_accounting,
        "status": status,
        "identity": identity,
        "identityFingerprint": identity_fingerprint,
    }


def build_manifest(bundle: dict) -> dict:
    documents_by_file = {
        spec[1]: bundle["documents"][spec[0]] for spec in ROLE_SPECS
    }
    inputs = []
    for name in REQUIRED_INPUT_FILES:
        data = bundle["files"][name]
        row = {
            "path": name,
            "bytes": len(data),
            "sha256": sha256_bytes(data),
        }
        report = documents_by_file.get(name)
        if report is not None:
            row["kind"] = report["kind"]
            row["schemaVersion"] = report["schemaVersion"]
            if report.get("mode") is not None:
                row["mode"] = report["mode"]
        inputs.append(row)
    return {
        "kind": MANIFEST_KIND,
        "schemaVersion": MANIFEST_SCHEMA_VERSION,
        "generator": {
            "name": GENERATOR_NAME,
            "version": GENERATOR_VERSION,
        },
        "identity": bundle["identity"],
        "identityFingerprint": bundle["identityFingerprint"],
        "resultStatus": bundle["status"]["overall"],
        "roleStatus": {
            role: row["status"] for role, row in sorted(bundle["status"]["roles"].items())
        },
        "scoreErrorAccounting": {
            key: bundle["scoreErrorAccounting"][key]
            for key in ("reportedCount", "intendedCount", "unexplainedCount")
        },
        "inputs": inputs,
    }


def _atomic_write(
    path: Path,
    data: bytes,
    *,
    error_code: str = "manifest-write",
    artifact_label: str = "manifest",
):
    tmp_name = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="wb",
            prefix=f".{path.name}.",
            dir=path.parent,
            delete=False,
        ) as handle:
            tmp_name = handle.name
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(tmp_name, path)
    except OSError as exc:
        if tmp_name is not None:
            try:
                os.unlink(tmp_name)
            except OSError:
                pass
        raise EvidenceError([
            error_record(
                error_code, file=path.name,
                message=f"{artifact_label}를 원자적으로 쓸 수 없다: {type(exc).__name__}",
            )
        ]) from exc


def _verify_input_snapshot(bundle: dict):
    """load와 manifest 교체 사이의 입력 변경을 seal 성공으로 보고하지 않는다."""
    errors = []
    root = bundle["root"]
    for name in REQUIRED_INPUT_FILES:
        path = root / name
        try:
            current = path.read_bytes()
        except OSError as exc:
            errors.append(error_record(
                "input-read", file=name,
                message=f"seal 직전 입력을 다시 읽을 수 없다: {type(exc).__name__}",
            ))
            continue
        if current != bundle["files"][name]:
            errors.append(error_record(
                "input-snapshot-drift", file=name,
                message="검증과 manifest 기록 사이에 입력 바이트가 바뀌었다",
            ))
    if errors:
        raise EvidenceError(errors)


def seal_evidence(evidence_dir: str | os.PathLike[str]) -> tuple[dict, dict]:
    bundle = load_evidence(evidence_dir)
    manifest = build_manifest(bundle)
    _verify_input_snapshot(bundle)
    _atomic_write(bundle["root"] / "evidence-manifest.json", canonical_json_bytes(manifest))
    return bundle, manifest


def _load_manifest(path: Path) -> dict:
    try:
        if path.is_symlink() or not path.is_file():
            raise EvidenceError([
                error_record(
                    "missing-manifest", file=path.name,
                    message="seal manifest가 없다",
                )
            ])
        data = path.read_bytes()
    except EvidenceError:
        raise
    except OSError as exc:
        raise EvidenceError([
            error_record(
                "manifest-read", file=path.name,
                message=f"manifest를 읽을 수 없다: {type(exc).__name__}",
            )
        ]) from exc
    errors = []
    report = _json_object({path.name: data}, path.name, errors)
    if errors or report is None:
        raise EvidenceError(errors)
    return report


def verify_seal(evidence_dir: str | os.PathLike[str]) -> tuple[dict, dict]:
    bundle = load_evidence(evidence_dir)
    actual = _load_manifest(bundle["root"] / "evidence-manifest.json")
    expected = build_manifest(bundle)
    errors = []
    if actual.get("kind") != MANIFEST_KIND:
        errors.append(error_record(
            "manifest-kind", file="evidence-manifest.json",
            message=f"manifest kind가 {MANIFEST_KIND}가 아니다",
        ))
    if actual.get("schemaVersion") != MANIFEST_SCHEMA_VERSION:
        errors.append(error_record(
            "manifest-schema", file="evidence-manifest.json",
            message=f"manifest schema가 {MANIFEST_SCHEMA_VERSION}가 아니다",
        ))
    if actual != expected:
        errors.append(error_record(
            "manifest-mismatch", file="evidence-manifest.json",
            message="seal 이후 입력·신원·생성기 버전 중 하나가 달라졌다",
        ))
    if errors:
        raise EvidenceError(errors)
    return bundle, actual


def _validate_output_path(bundle: dict, output_path: str | os.PathLike[str]) -> Path:
    raw = os.fspath(output_path)
    if not raw or "\x00" in raw:
        raise EvidenceError([
            error_record("invalid-output", message="출력 경로가 비었거나 NUL을 포함한다")
        ])
    path = Path(raw)
    root = Path(os.path.abspath(bundle["root"]))
    absolute = Path(os.path.abspath(path))
    protected = {root / name for name in REQUIRED_INPUT_FILES}
    protected.add(root / "evidence-manifest.json")
    if absolute in protected:
        raise EvidenceError([
            error_record("protected-output", message="seal 또는 필수 입력을 출력으로 덮어쓸 수 없다")
        ])
    if path.suffix.lower() != ".html":
        raise EvidenceError([
            error_record("invalid-output", message="출력 파일 확장자는 .html이어야 한다")
        ])
    try:
        if path.is_symlink() or path.is_dir():
            raise EvidenceError([
                error_record("invalid-output", message="출력은 symlink나 디렉터리일 수 없다")
            ])
    except OSError as exc:
        raise EvidenceError([
            error_record(
                "invalid-output",
                message=f"출력 경로를 확인할 수 없다: {type(exc).__name__}",
            )
        ]) from exc
    return path


def render_evidence(
    evidence_dir: str | os.PathLike[str],
    output_path: str | os.PathLike[str],
) -> tuple[dict, dict, Path, str]:
    """검증된 seal을 정적 HTML로 쓰고 결과 상태와 hash를 돌려준다."""
    bundle, manifest = verify_seal(evidence_dir)
    path = _validate_output_path(bundle, output_path)
    renderer = _renderer_module()
    data = renderer.render_html(bundle, manifest)

    # 검증 뒤 renderer가 도는 동안 입력 또는 manifest가 바뀐 경우도 출력하지 않는다.
    _verify_input_snapshot(bundle)
    if _load_manifest(bundle["root"] / "evidence-manifest.json") != manifest:
        raise EvidenceError([
            error_record(
                "manifest-snapshot-drift",
                file="evidence-manifest.json",
                message="HTML 생성 중 seal manifest가 바뀌었다",
            )
        ])
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        raise EvidenceError([
            error_record(
                "report-directory", message=f"출력 디렉터리를 준비할 수 없다: {type(exc).__name__}"
            )
        ]) from exc
    _atomic_write(path, data, error_code="report-write", artifact_label="HTML report")
    return bundle, manifest, path, sha256_bytes(data)


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Gym JSON 증적을 seal하거나 검증된 정적 HTML로 변환한다 (#6669)",
    )
    parser.add_argument("--evidence-dir", required=True)
    operation = parser.add_mutually_exclusive_group(required=True)
    operation.add_argument("--seal", action="store_true")
    operation.add_argument("--out", metavar="REPORT.html")
    return parser.parse_args(argv)


def _error_envelope(exc: EvidenceError) -> dict:
    return {
        "kind": ERROR_KIND,
        "schemaVersion": MANIFEST_SCHEMA_VERSION,
        "ok": False,
        "errorCount": len(exc.errors),
        "errors": exc.errors,
    }


def main(argv=None) -> int:
    args = parse_args(argv)
    try:
        if args.seal:
            bundle, manifest = seal_evidence(args.evidence_dir)
            summary = {
                "kind": "gymEvidenceSeal",
                "schemaVersion": MANIFEST_SCHEMA_VERSION,
                "ok": True,
                "manifest": "evidence-manifest.json",
                "identityFingerprint": manifest["identityFingerprint"],
                "resultStatus": bundle["status"]["overall"],
            }
            exit_code = EXIT_OK
        else:
            bundle, manifest, path, report_sha256 = render_evidence(args.evidence_dir, args.out)
            summary = {
                "kind": "gymEvidenceReport",
                "schemaVersion": MANIFEST_SCHEMA_VERSION,
                "generated": True,
                "report": path.name,
                "reportSha256": report_sha256,
                "identityFingerprint": manifest["identityFingerprint"],
                "resultStatus": bundle["status"]["overall"],
            }
            exit_code = (
                EXIT_OK if bundle["status"]["overall"] == STATUS_PASS
                else EXIT_RESULT_NOT_PASS
            )
    except EvidenceError as exc:
        sys.stderr.buffer.write(canonical_json_bytes(_error_envelope(exc)))
        return EXIT_INPUT_INVALID
    sys.stdout.buffer.write(canonical_json_bytes(summary))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
