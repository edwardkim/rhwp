"""V-lineage: a work chain is accepted only when parent output == child input.

LLM-as-Verifier axis for issue #5516. Implementer prose is not evidence.
Only existing `rhwp lineage` fields `parentOk`, `lineageOk`, `brokenAt`
plus the two hashes decide. This package wraps that contract; it does not
rewrite `.agents/skills/rhwp-work-receipt` and it does not reimplement
V-replay (`reproduced` / `--expect-output-sha256`).
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
from .envelopes import observation_from_capsule_pair, observation_from_lineage
from .schema import (
    CASE_COLUMNS,
    LineageCase,
    LineageObservation,
    LineageSource,
    ParentState,
    parse_optional_bool,
)

__all__ = [
    "CASE_COLUMNS",
    "CLAIM_ID",
    "KIND",
    "SCHEMA_VERSION",
    "VERDICT_CLASSES",
    "Decision",
    "LineageCase",
    "LineageObservation",
    "LineageSource",
    "ParentState",
    "decide",
    "decide_observation",
    "decide_row",
    "observation_from_capsule_pair",
    "observation_from_lineage",
    "parse_optional_bool",
]
