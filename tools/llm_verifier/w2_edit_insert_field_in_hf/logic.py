from __future__ import annotations

COMMAND = 'edit-insert-field-in-hf'
FAMILY = 'mutate'
CLAIM_ID = 'V-w2-edit-insert-field-in-hf'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    before = int(f0); delta = int(f1); after = int(f2); insert = int(f3)
    if before < 0 or after < 0:
        return "USAGE"
    expected = before + delta if insert else before - delta
    if expected < 0:
        return "USAGE"
    return "MUTATE_OK" if after == expected else "MUTATE_DRIFT"

    raise ValueError(family)
