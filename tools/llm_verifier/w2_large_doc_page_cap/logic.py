from __future__ import annotations

COMMAND = 'large-doc-page-cap'
FAMILY = 'limit'
CLAIM_ID = 'V-w2-large-doc-page-cap'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    size = int(f0); cap = int(f1); accepted = int(f2)
    if size < 0 or cap <= 0:
        return "USAGE"
    if size > cap and accepted:
        return "OVER_ACCEPTED"
    if size > cap:
        return "OVER_REJECT"
    return "UNDER_OK" if accepted else "UNDER_REJECT"

    raise ValueError(family)
