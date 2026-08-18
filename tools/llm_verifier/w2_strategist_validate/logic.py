from __future__ import annotations

COMMAND = 'strategist-validate'
FAMILY = 'rate'
CLAIM_ID = 'V-w2-strategist-validate'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    ok = int(f0); total = int(f1)
    if ok < 0 or total < 0:
        return "USAGE"
    if ok > total:
        return "RATE_IMPOSSIBLE"
    return "RATE_OK"

    raise ValueError(family)
