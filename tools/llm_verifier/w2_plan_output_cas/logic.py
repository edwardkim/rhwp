from __future__ import annotations

COMMAND = 'plan-output-cas'
FAMILY = 'cas'
CLAIM_ID = 'V-w2-plan-output-cas'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    present = int(f0); extra = int(f1); expected = f2; actual = f3
    if extra:
        return "USAGE"
    if not present:
        return "SKIP"
    hexset = set("0123456789abcdef")
    if len(expected) != 64 or len(actual) != 64:
        return "USAGE"
    if any(c not in hexset for c in expected + actual):
        return "USAGE"
    return "CAS_OK" if expected == actual else "CAS_MISMATCH"

    raise ValueError(family)
