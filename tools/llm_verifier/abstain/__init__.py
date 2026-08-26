"""V-abstain: envelope field contradiction => abstain, never invent pass/fail."""

from .decide import decide, decide_row, named_contradictions, success_tokens, fail_tokens
from .schema import (
    CLAIM_ID,
    EnvelopeFields,
    Decision,
    VERDICT_ABSTAIN,
    VERDICT_FAIL,
    VERDICT_PASS,
)

__all__ = [
    "CLAIM_ID",
    "Decision",
    "EnvelopeFields",
    "VERDICT_ABSTAIN",
    "VERDICT_FAIL",
    "VERDICT_PASS",
    "decide",
    "decide_row",
    "fail_tokens",
    "named_contradictions",
    "success_tokens",
]
