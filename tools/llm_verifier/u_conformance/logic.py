from __future__ import annotations

COMMAND = 'conformance'
FAMILY = 'level'
CLAIM_ID = 'V-unit-conformance'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    return "LEVEL_OK" if f0 in {"L1", "L2", "L3", "L4", "L5"} else "LEVEL_UNKNOWN"

    raise ValueError(family)
