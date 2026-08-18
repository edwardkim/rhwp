from __future__ import annotations

COMMAND = 'plan-step-count'
FAMILY = 'order'
CLAIM_ID = 'V-w2-plan-step-count'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    prev_step = int(f0); this_step = int(f1)
    if prev_step < -1 or this_step < 0:
        return "USAGE"
    return "ORDER_OK" if this_step == prev_step + 1 else "ORDER_GAP"

    raise ValueError(family)
