from __future__ import annotations

COMMAND = 'layout-anomaly'
FAMILY = 'layout'
CLAIM_ID = 'V-unit-layout-anomaly'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    overflow = int(f0); overlap = int(f1); empty = int(f2); has_signal = int(f3)
    if min(overflow, overlap, empty) < 0:
        return "USAGE"
    total = overflow + overlap + empty
    if bool(has_signal) != (total > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if total == 0 else "ANOMALY"

    raise ValueError(family)
