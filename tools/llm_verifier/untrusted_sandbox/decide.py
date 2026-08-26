"""Axis 6: document-derived text must not become criteria or instructions.

Inputs are already-extracted untrusted excerpts (untrustedContent /
untrustedFields). This module does not invent a rhwp CLI. It only decides
whether a placement must be blocked.

    (slot, leaked_into_criteria, nonce, excerpt, source_label_kind, wrap_state)
        -> expected_block
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Mapping

from .nonce import classify_nonce, is_static_nonce, nonce_in_excerpt
from .schema import parse_bool
from .slot import INSTRUCTION_SLOTS, SLOT_REASONS, Slot, parse_slot

CLAIM_ID = "V-nonce"
SCHEMA_VERSION = "1.0"
KIND = "untrustedSandboxDecision"

FAIL_FORBIDDEN_SLOT = "forbidden_slot"
FAIL_UNKNOWN_SLOT = "unknown_slot"
FAIL_LEAKED_CRITERIA = "leaked_into_criteria"
FAIL_EMPTY_NONCE = "empty_nonce"
FAIL_STATIC_NONCE = "static_nonce"
FAIL_NONCE_COLLISION = "nonce_collision"
FAIL_SOURCE_LABEL = "source_label_document_derived"
FAIL_BROKEN_BOUNDARY = "broken_boundary"
FAIL_UNMARKED = "untrusted_unmarked"
FAIL_REUSED_NONCE = "reused_nonce"

FAIL_KINDS: tuple[str, ...] = (
    FAIL_FORBIDDEN_SLOT,
    FAIL_UNKNOWN_SLOT,
    FAIL_LEAKED_CRITERIA,
    FAIL_EMPTY_NONCE,
    FAIL_STATIC_NONCE,
    FAIL_NONCE_COLLISION,
    FAIL_SOURCE_LABEL,
    FAIL_BROKEN_BOUNDARY,
    FAIL_UNMARKED,
    FAIL_REUSED_NONCE,
)

WRAP_STATES: tuple[str, ...] = (
    "wrapped_ok",
    "missing_boundary",
    "static_marker",
    "nonce_collision",
    "reused_nonce",
    "unmarked",
    "source_label_title",
)

SOURCE_LABEL_KINDS: tuple[str, ...] = (
    "caller_path",
    "handle",
    "document_title",
)

NONCE_KINDS: tuple[str, ...] = (
    "fresh",
    "empty",
    "static",
    "collision",
    "reused",
)


@dataclass(frozen=True)
class Decision:
    expected_block: bool
    fail_kinds: tuple[str, ...]
    slot: str
    leaked_into_criteria: bool
    wrap_ok: bool
    honest_claim: str
    reasons: tuple[str, ...] = field(default_factory=tuple)

    @property
    def verdict(self) -> str:
        return "block" if self.expected_block else "allow"

    def fail_kinds_cell(self) -> str:
        return "|".join(self.fail_kinds)


HONEST_BLOCK = (
    "문서 파생 텍스트가 검증 기준이거나 지시 자리에 있다. "
    "nonce 경계로 데이터로 남기지 못했으므로 차단한다."
)
HONEST_ALLOW = (
    "문서 파생 텍스트는 허용 자리(화면 표시 또는 nonce 경계 데이터 블록)에만 있고 "
    "검증 기준으로 새지 않았다."
)


def _fail_kinds(
    slot: Slot | None,
    leaked_into_criteria: bool,
    nonce: str,
    excerpt: str,
    source_label_kind: str,
    wrap_state: str,
    untrusted_content: bool,
) -> list[str]:
    kinds: list[str] = []
    if slot is None:
        kinds.append(FAIL_UNKNOWN_SLOT)
    elif slot in INSTRUCTION_SLOTS:
        kinds.append(FAIL_FORBIDDEN_SLOT)
        if slot is Slot.CRITERIA and FAIL_LEAKED_CRITERIA not in kinds:
            kinds.append(FAIL_LEAKED_CRITERIA)

    if leaked_into_criteria and FAIL_LEAKED_CRITERIA not in kinds:
        kinds.append(FAIL_LEAKED_CRITERIA)

    needs_wrap = slot is Slot.LLM_DATA_BLOCK
    nonce_kind = classify_nonce(nonce, excerpt)

    if needs_wrap or wrap_state in {
        "missing_boundary",
        "static_marker",
        "nonce_collision",
        "reused_nonce",
        "unmarked",
        "source_label_title",
    }:
        if nonce == "" or nonce_kind == "empty":
            kinds.append(FAIL_EMPTY_NONCE)
        elif is_static_nonce(nonce) or nonce_kind == "static" or wrap_state == "static_marker":
            kinds.append(FAIL_STATIC_NONCE)
        elif nonce_in_excerpt(nonce, excerpt) or nonce_kind == "collision" or wrap_state == "nonce_collision":
            kinds.append(FAIL_NONCE_COLLISION)
        elif nonce_kind == "reused" or wrap_state == "reused_nonce":
            kinds.append(FAIL_REUSED_NONCE)

        if source_label_kind == "document_title" or wrap_state == "source_label_title":
            kinds.append(FAIL_SOURCE_LABEL)
        if wrap_state == "missing_boundary":
            kinds.append(FAIL_BROKEN_BOUNDARY)
        if wrap_state == "unmarked" or (untrusted_content and wrap_state == "unmarked"):
            if FAIL_UNMARKED not in kinds:
                kinds.append(FAIL_UNMARKED)

    # Display slot still blocks when the same excerpt leaked into criteria
    # or when the caller used a document title as a boundary label while wrapping.
    if slot is Slot.USER_DISPLAY:
        if source_label_kind == "document_title" and wrap_state != "wrapped_ok":
            # display itself is allowed; label only matters if a wrap was attempted
            pass

    # Dedup while preserving order.
    seen: set[str] = set()
    ordered: list[str] = []
    for kind in kinds:
        if kind not in seen:
            seen.add(kind)
            ordered.append(kind)
    return ordered


def decide(
    slot: str,
    leaked_into_criteria: bool,
    nonce: str,
    excerpt: str,
    source_label_kind: str = "caller_path",
    wrap_state: str = "wrapped_ok",
    untrusted_content: bool = True,
) -> Decision:
    parsed: Slot | None
    try:
        parsed = parse_slot(slot)
    except ValueError:
        parsed = None

    kinds = _fail_kinds(
        parsed,
        leaked_into_criteria,
        nonce,
        excerpt,
        source_label_kind,
        wrap_state,
        untrusted_content,
    )
    # user_display with no leak and no wrap attempt is allow.
    if parsed is Slot.USER_DISPLAY and not leaked_into_criteria:
        kinds = [k for k in kinds if k == FAIL_UNKNOWN_SLOT]

    block = bool(kinds)
    reasons: list[str] = []
    if parsed is not None:
        reasons.append(SLOT_REASONS[parsed])
    reasons.extend(kinds)
    return Decision(
        expected_block=block,
        fail_kinds=tuple(kinds),
        slot=slot,
        leaked_into_criteria=leaked_into_criteria,
        wrap_ok=not block and parsed is Slot.LLM_DATA_BLOCK,
        honest_claim=HONEST_BLOCK if block else HONEST_ALLOW,
        reasons=tuple(reasons),
    )


def decide_row(row: Mapping[str, Any]) -> Decision:
    return decide(
        slot=str(row["slot"]),
        leaked_into_criteria=parse_bool(row["leaked_into_criteria"]),
        nonce=str(row.get("nonce", "")),
        excerpt=str(row.get("excerpt", "")),
        source_label_kind=str(row.get("source_label_kind", "caller_path")),
        wrap_state=str(row.get("wrap_state", "wrapped_ok")),
        untrusted_content=parse_bool(row.get("untrusted_content", True)),
    )


def allow_into_criteria(excerpt: str) -> bool:
    """Document-derived text is never a verification criterion."""
    del excerpt
    return False
