from __future__ import annotations

COMMAND = 'armor'
FAMILY = 'signal'
CLAIM_ID = 'V-unit-armor'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    count = int(f0); has_signal = int(f1)
    if count < 0:
        return "USAGE"
    if bool(has_signal) != (count > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if count == 0 else "ANOMALY"

    raise ValueError(family)
