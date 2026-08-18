from __future__ import annotations

COMMAND = 'handoff-outgoing'
FAMILY = 'triad'
CLAIM_ID = 'V-w2-handoff-outgoing'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    hexset = set("0123456789abcdef")
    for token in (f0, f1, f2):
        if not token:
            return "TRIAD_MISS"
        if len(token) != 64 or any(ch not in hexset for ch in token):
            return "HASH_DEFECT"
    return "TRIAD_OK"

    raise ValueError(family)
