from __future__ import annotations

COMMAND = 'export-png-gpu'
FAMILY = 'avail'
CLAIM_ID = 'V-unit-export-png-gpu'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    available = int(f0); exit_code = int(f1)
    if exit_code not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    if not available and exit_code == 2:
        return "UNAVAIL_OK"
    if not available and exit_code != 2:
        return "UNAVAIL_LIE"
    if available and exit_code == 2:
        return "FALSE_UNAVAIL"
    return "AVAIL_RUN"

    raise ValueError(family)
