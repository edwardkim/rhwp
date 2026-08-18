from __future__ import annotations

COMMAND = 'edit-apply-hf-template'
FAMILY = 'reread'
CLAIM_ID = 'V-w2-edit-apply-hf-template'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    verify = int(f0); written = int(f1); reread = int(f2)
    if not verify:
        return "NOT_EVIDENCE"
    if written < 0 or reread < 0:
        return "USAGE"
    return "REREAD_OK" if written == reread else "REREAD_DRIFT"

    raise ValueError(family)
