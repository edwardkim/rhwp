from __future__ import annotations

COMMAND = 'nols-2024'
FAMILY = 'trap'
CLAIM_ID = 'V-unit-nols-2024'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    year = int(f0); nols = int(f1); used = int(f2)
    if year >= 2024 and nols and used:
        return "TRAP"
    if year >= 2024 and nols:
        return "FLAGGED"
    return "SAFE"

    raise ValueError(family)
