from __future__ import annotations

COMMAND = 'recipe-redact'
FAMILY = 'redact'
CLAIM_ID = 'V-unit-recipe-redact'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    applied = int(f0); before = int(f1); after = int(f2)
    if not applied:
        return "NOT_EVIDENCE"
    if before < 0 or after < 0:
        return "USAGE"
    if after > 0:
        return "STILL_PRESENT"
    return "NOTHING_TO_CLEAR" if before == 0 else "CLEAR_OK"

    raise ValueError(family)
