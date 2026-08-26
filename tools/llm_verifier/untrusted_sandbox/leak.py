"""Detect document excerpts leaking into verification criteria.

leaked_into_criteria is the corpus yes/no column. Detection here is the
mechanical check a verifier host runs before it treats a string as a
criterion or instruction.
"""

from __future__ import annotations

from dataclasses import dataclass

from .nonce import BEGIN_PREFIX, END_PREFIX, boundary_intact


@dataclass(frozen=True)
class LeakReport:
    leaked: bool
    reason: str
    excerpt: str
    slot: str


def excerpt_in_criteria(criteria: str, excerpt: str) -> bool:
    if not excerpt or not criteria:
        return False
    return excerpt in criteria


def injection_looks_like_criterion(excerpt: str) -> bool:
    needles = (
        "검증 기준",
        "always pass",
        "identical:true",
        "identical: true",
        "합격으로 판정",
        "treat as criterion",
        "ignore previous",
        "앞의 지시는 무시",
        "system prompt",
        "시스템 프롬프트",
        "pass if",
        "expected_block=0",
        "machineVerdict",
    )
    folded = excerpt.lower()
    return any(n.lower() in folded for n in needles)


def detect_leak(*, criteria: str, excerpt: str, nonce: str, wrapped: str = "") -> LeakReport:
    if excerpt_in_criteria(criteria, excerpt):
        # Allowed only if the criteria string is itself a wrapped data citation
        # that still has intact nonce boundaries and does not use excerpt as a rule.
        if wrapped and boundary_intact(wrapped, nonce, excerpt) and criteria == wrapped:
            return LeakReport(False, "wrapped_citation_only", excerpt, "criteria")
        return LeakReport(True, "excerpt_copied_into_criteria", excerpt, "criteria")
    if nonce and nonce in criteria and not wrapped:
        return LeakReport(True, "nonce_used_as_criterion", excerpt, "criteria")
    return LeakReport(False, "no_leak", excerpt, "criteria")


def split_prompt_slots(prompt: str, nonce: str) -> dict[str, str]:
    """Best-effort split of a verifier prompt into criteria vs data."""
    begin = f"{BEGIN_PREFIX}{nonce}]"
    end = f"{END_PREFIX}{nonce}]"
    if begin in prompt and end in prompt:
        before, rest = prompt.split(begin, 1)
        inner, after = rest.split(end, 1)
        return {
            "criteria": before + after,
            "data": f"{begin}{inner}{end}",
        }
    return {"criteria": prompt, "data": ""}
