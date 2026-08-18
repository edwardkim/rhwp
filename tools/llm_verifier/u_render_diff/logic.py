from __future__ import annotations

COMMAND = 'render-diff'
FAMILY = 'px'
CLAIM_ID = 'V-unit-render-diff'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    delta = int(f0); threshold = int(f1); struct_mismatch = int(f2)
    if delta < 0 or threshold < 0:
        return "USAGE"
    if struct_mismatch:
        return "STRUCT"
    return "PX_FAIL" if delta > threshold else "PX_OK"

    raise ValueError(family)
