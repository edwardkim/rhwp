"""V-replay: third-party labor is accepted only from replay reproduced fields.

LLM-as-Verifier axis for issue #5502. Implementer prose is not evidence.
Only `rhwp replay --expect-output-sha256` / workCapsule `receipt.reproduced`
count. This package wraps that existing contract; it does not rewrite
`.claude/skills/rhwp-work-receipt` or M-rcpt fixtures.
"""

from .decide import (
    CLAIM_ID,
    KIND,
    SCHEMA_VERSION,
    VERDICT_CLASSES,
    Decision,
    decide,
    decide_observation,
    decide_row,
)
from .envelopes import observation_from_capsule, observation_from_replay
from .schema import (
    CASE_COLUMNS,
    ReplayCase,
    ReplayMode,
    ReplayObservation,
    ReplaySource,
    parse_reproduced,
)

__all__ = [
    "CASE_COLUMNS",
    "CLAIM_ID",
    "KIND",
    "SCHEMA_VERSION",
    "VERDICT_CLASSES",
    "Decision",
    "ReplayCase",
    "ReplayMode",
    "ReplayObservation",
    "ReplaySource",
    "decide",
    "decide_observation",
    "decide_row",
    "observation_from_capsule",
    "observation_from_replay",
    "parse_reproduced",
]
