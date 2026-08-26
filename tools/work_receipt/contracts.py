#!/usr/bin/env python3
"""Existing replay / audit / lineage CLI contracts.

Mirrors ``src/main.rs`` ``cmd_replay`` / ``cmd_audit`` / ``cmd_lineage``
and ``validated_capsule_plan``. No new CLI is invented. Field names,
exit codes, and error needles are copied from devel.
"""

from __future__ import annotations

import hashlib
import json
import re
from typing import Any

ENVELOPE_SCHEMA_VERSION = "1.0"
CLAIM_ID = "M-rcpt"
ISSUE = 5478
GENERATOR = "tools/work_receipt/fatten_work_receipt.py"
KIND_CATALOG = "workReceiptCatalog"

# #2707 family: judgment is envelope data, not a tool crash.
EXIT_OK = 0
EXIT_RUNTIME = 1
EXIT_USAGE = 2
EXIT_JUDGMENT = 3

REPLAY_REQUIRED = (
    "schemaVersion",
    "mode",
    "input",
    "inputSha256",
    "planSha256",
    "outputSha256",
    "toolVersion",
    "steps",
    "reproduced",
    "expectedOutputSha256",
)
AUDIT_REQUIRED = (
    "schemaVersion",
    "root",
    "total",
    "reproduced",
    "failed",
    "reproducedRate",
)
LINEAGE_REQUIRED = (
    "schemaVersion",
    "head",
    "depth",
    "valid",
    "brokenAt",
    "links",
)
CAPSULE_REQUIRED = (
    "schemaVersion",
    "kind",
    "parent",
    "plan",
    "planText",
    "receipt",
)
PLAN_ACTIONS = ("replace_text", "fill_fields", "set_cell", "set_checkbox")

ZERO64 = "0" * 64
SHA256_RE = re.compile(r"^[0-9a-fA-F]{64}$")

# Exact needles from src/main.rs — tests lock these strings.
NEEDLE = {
    "replay_plan_json_missing": "--plan-json 뒤에 계획 JSON 이 필요합니다",
    "replay_expect_missing": "--expect-output-sha256 뒤에 64자리 16진 해시가 필요합니다",
    "replay_parent_missing": "--parent 뒤에 부모 캡슐 경로가 필요합니다",
    "replay_sign_key_missing": "--sign-key 뒤에 키 파일 경로가 필요합니다",
    "replay_capsule_missing": "--capsule 뒤에 저장 경로가 필요합니다",
    "replay_unknown": "알 수 없는 옵션",
    "replay_expect_not_hex": "--expect-output-sha256 값은 64자리 16진이어야 합니다",
    "replay_sign_without_capsule": "--sign-key 는 --capsule 과 함께 사용합니다",
    "replay_usage": "사용법: rhwp replay",
    "replay_plan_parse": "계획 JSON 파싱 실패",
    "replay_plan_no_input": "계획에 input 이 필요합니다",
    "replay_plan_read": "계획을 읽을 수 없습니다",
    "replay_same_file": "--capsule과 --parent가 같은 기존 파일을 가리킵니다",
    "replay_parent_read": "부모 캡슐을 읽을 수 없습니다",
    "replay_input_read": "입력을 읽을 수 없습니다",
    "replay_engine": "계획 재실행 실패",
    "audit_usage": "사용법: rhwp audit",
    "audit_unknown": "알 수 없는 옵션",
    "audit_dir_read": "폴더를 읽을 수 없습니다",
    "audit_empty": "에 *.capsule.json 이 없습니다",
    "audit_kind": "kind 가 workCapsule 이 아님",
    "audit_output_sha": "receipt.outputSha256 가 없거나 64자리 16진이 아님",
    "audit_input_sha": "receipt.inputSha256 가 없거나 64자리 16진이 아님",
    "audit_plan_text_sha": "planText 와 receipt.planSha256 불일치",
    "audit_plan_vs_text": "plan 과 planText 불일치",
    "audit_steps": "receipt.steps 와 planText.steps 길이 불일치 (plan.steps 길이와 receipt.steps 불일치)",
    "audit_plan_text_missing": "planText 없음",
    "audit_steps_not_int": "receipt.steps 가 음이 아닌 정수가 아님",
    "audit_steps_not_array": "planText.steps/plan.steps 가 배열이 아님",
    "lineage_usage": "사용법: rhwp lineage",
    "lineage_unknown": "알 수 없는 옵션",
    "lineage_keyring_missing": "--keyring 뒤에 키 등록부 경로가 필요합니다",
    "lineage_anchor_missing": "--anchor-log 뒤에 로그 경로가 필요합니다",
    "lineage_head_read": "캡슐을 읽을 수 없습니다",
    "lineage_kind": "kind 가 workCapsule 이 아님",
    "lineage_input_sha": "receipt.inputSha256 가 없거나 64자리 16진이 아님",
    "lineage_output_sha": "receipt.outputSha256 가 없거나 64자리 16진이 아님",
    "lineage_parent_field": "parent 필드 없음",
    "lineage_parent_capsule": "parent.capsule 없음",
    "lineage_parent_sha": "parent.sha256 가 없거나 64자리 16진이 아님",
    "lineage_cycle": "체인 길이 1000 초과 — 순환 의심",
}


def sha256_hex(data: bytes | str) -> str:
    if isinstance(data, str):
        data = data.encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def is_sha256_hex(value: object) -> bool:
    return isinstance(value, str) and bool(SHA256_RE.fullmatch(value))


def canonical_json(data: Any) -> str:
    return json.dumps(data, ensure_ascii=False, indent=2) + "\n"


def plan_text_of(plan: dict[str, Any]) -> str:
    """Inline --plan-json bytes: compact UTF-8, key order preserved."""
    return json.dumps(plan, ensure_ascii=False, separators=(",", ":"))


def classify_expect_hash(value: str | None) -> tuple[int, str | None]:
    if value is None:
        return EXIT_OK, None
    lowered = value.strip().lower()
    if len(lowered) != 64 or any(ch not in "0123456789abcdef" for ch in lowered):
        return EXIT_USAGE, NEEDLE["replay_expect_not_hex"]
    return EXIT_OK, None


def classify_replay(
    *,
    has_plan: bool,
    plan_parse_ok: bool,
    has_input: bool,
    sign_key: bool,
    capsule: bool,
    same_file: bool,
    expect: str | None,
    reproduced: bool | None,
    io_error: bool = False,
    engine_fail: bool = False,
) -> tuple[int, str]:
    """Return (exit, mode) for the existing replay CLI."""
    if io_error:
        return EXIT_RUNTIME, "error"
    if engine_fail:
        return EXIT_RUNTIME, "error"
    if sign_key and not capsule:
        return EXIT_USAGE, "usage"
    if same_file:
        return EXIT_USAGE, "usage"
    if not has_plan:
        return EXIT_USAGE, "usage"
    if not plan_parse_ok:
        return EXIT_USAGE, "usage"
    if not has_input:
        return EXIT_USAGE, "usage"
    code, _ = classify_expect_hash(expect)
    if code != EXIT_OK:
        return EXIT_USAGE, "usage"
    if expect is None:
        return EXIT_OK, "attest"
    if reproduced is False:
        return EXIT_JUDGMENT, "verify"
    return EXIT_OK, "verify"


def classify_audit(*, dir_exists: bool, total: int, failed: int) -> int:
    if not dir_exists:
        return EXIT_RUNTIME
    if total == 0:
        return EXIT_USAGE
    if failed > 0:
        return EXIT_JUDGMENT
    return EXIT_OK


def audit_rate(reproduced: int, total: int) -> float:
    if total <= 0:
        raise ValueError("empty audit folder is usage, not rate 0")
    return reproduced / total


def classify_lineage(
    *,
    has_head_arg: bool,
    head_readable: bool,
    valid: bool,
    usage_error: bool = False,
    io_error: bool = False,
) -> int:
    if usage_error or not has_head_arg:
        return EXIT_USAGE
    if io_error or not head_readable:
        return EXIT_RUNTIME
    if not valid:
        return EXIT_JUDGMENT
    return EXIT_OK


def validated_capsule_plan(capsule: dict[str, Any]) -> tuple[dict[str, Any], int] | str:
    """Same fail-closed checks as ``validated_capsule_plan`` in main.rs."""
    plan_text = capsule.get("planText")
    if not isinstance(plan_text, str):
        return NEEDLE["audit_plan_text_missing"]
    expected = capsule.get("receipt", {}).get("planSha256")
    if not is_sha256_hex(expected):
        return "receipt.planSha256 가 없거나 64자리 16진이 아님"
    if sha256_hex(plan_text) != expected:
        return NEEDLE["audit_plan_text_sha"]
    try:
        plan = json.loads(plan_text)
    except json.JSONDecodeError as exc:
        return f"planText JSON 파싱 실패: {exc}"
    if not isinstance(plan, dict):
        return "planText 계획 객체 없음"
    if capsule.get("plan") != plan:
        return NEEDLE["audit_plan_vs_text"]
    steps = capsule.get("receipt", {}).get("steps")
    if not isinstance(steps, int) or isinstance(steps, bool) or steps < 0:
        return NEEDLE["audit_steps_not_int"]
    plan_steps = plan.get("steps")
    if not isinstance(plan_steps, list):
        return NEEDLE["audit_steps_not_array"]
    if steps != len(plan_steps):
        return NEEDLE["audit_steps"]
    return plan, steps


def lineage_ok(parent_output_sha: str, child_input_sha: str) -> bool:
    """Chronicle invariant: parent output bytes == child input bytes."""
    return parent_output_sha == child_input_sha


def parent_ok(recorded_parent_sha: str, parent_file_sha: str) -> bool:
    return recorded_parent_sha == parent_file_sha


def stdout_silent_on_failure(exit_code: int, has_json_error_envelope: bool) -> bool:
    """Failure paths are silent unless replay engine-error JSON is opted in.

    Audit empty / usage and lineage missing-head keep stdout at 0 bytes.
    Replay engine failure in --json emits a tiny {schemaVersion, error} object.
    """
    if exit_code in (EXIT_OK, EXIT_JUDGMENT):
        return False
    return not has_json_error_envelope
