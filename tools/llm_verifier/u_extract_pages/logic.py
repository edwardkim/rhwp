from __future__ import annotations

COMMAND = 'extract-pages'
FAMILY = 'window'
CLAIM_ID = 'V-unit-extract-pages'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    req = int(f0); emitted = int(f1); total = int(f2)
    if req < 0 or total < 0 or emitted < 0:
        return "USAGE"
    if req >= total:
        return "USAGE"
    if emitted == 0:
        return "WINDOW_MISS"
    if emitted != 1:
        return "WINDOW_LEAK"
    return "WINDOW_OK"

    raise ValueError(family)
