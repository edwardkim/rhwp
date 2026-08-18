from __future__ import annotations

COMMAND = 'skill-bulk'
FAMILY = 'isolate'
CLAIM_ID = 'V-unit-skill-bulk'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    n_in = int(f0); n_ok = int(f1); n_fail = int(f2); neighbor = int(f3)
    if min(n_in, n_ok, n_fail) < 0:
        return "USAGE"
    if n_in != n_ok + n_fail:
        return "COUNT_DRIFT"
    return "POISON" if neighbor else "ISOLATED"

    raise ValueError(family)
