from __future__ import annotations

COMMAND = 'export-tables'
FAMILY = 'span'
CLAIM_ID = 'V-unit-export-tables'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    rows = int(f0); cols = int(f1); rs = int(f2); cs = int(f3)
    if min(rows, cols, rs, cs) <= 0:
        return "USAGE"
    if rs > rows or cs > cols:
        return "SPAN_OOB"
    return "SPAN_OK"

    raise ValueError(family)
