from __future__ import annotations

COMMAND = 'digest'
FAMILY = 'bound'
CLAIM_ID = 'V-unit-digest'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    limit = int(f0); actual = int(f1); truncated = int(f2)
    if limit < 0 or actual < 0:
        return "USAGE"
    if actual > limit and not truncated:
        return "BOUND_LIE"
    if actual <= limit and truncated:
        return "BOUND_FALSE_POS"
    return "BOUND_OK"

    raise ValueError(family)
