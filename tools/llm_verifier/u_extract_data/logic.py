from __future__ import annotations

COMMAND = 'extract-data'
FAMILY = 'kind'
CLAIM_ID = 'V-unit-extract-data'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    kind = f0
    count = int(f1); array_len = int(f2)
    if kind not in {"date", "amount", "number", "all"}:
        return "KIND_UNKNOWN"
    if count < 0 or array_len < 0:
        return "USAGE"
    return "KIND_OK" if count == array_len else "COUNT_DRIFT"

    raise ValueError(family)
