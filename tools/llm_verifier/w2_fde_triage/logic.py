from __future__ import annotations

COMMAND = 'fde-triage'
FAMILY = 'route'
CLAIM_ID = 'V-w2-fde-triage'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    return "ROUTE_OK" if f0 in {"pdf", "fill", "table", "needs-agent", "fde"} else "ROUTE_UNKNOWN"

    raise ValueError(family)
