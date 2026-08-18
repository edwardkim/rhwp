from __future__ import annotations

COMMAND = 'eq-supsub'
FAMILY = 'parse'
CLAIM_ID = 'V-unit-eq-supsub'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    has_space = int(f0); parsed = int(f1)
    if has_space and not parsed:
        return "PARSE_DROP"
    return "PARSE_OK" if parsed else "PARSE_FAIL"

    raise ValueError(family)
