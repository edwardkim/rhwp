from __future__ import annotations

COMMAND = 'verify'
FAMILY = 'diff'
CLAIM_ID = 'V-unit-verify'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    diff_count = int(f0); items = int(f1)
    if diff_count < 0 or items < 0:
        return "USAGE"
    return "DIFF_OK" if diff_count == items else "COUNT_DRIFT"

    raise ValueError(family)
