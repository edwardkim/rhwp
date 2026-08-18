from __future__ import annotations

COMMAND = 'boundary-path'
FAMILY = 'path'
CLAIM_ID = 'V-unit-boundary-path'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    escaped = int(f0); outside = int(f1)
    if outside:
        return "BREACH"
    return "ESCAPE" if escaped else "PATH_OK"

    raise ValueError(family)
