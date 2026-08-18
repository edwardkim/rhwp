from __future__ import annotations

COMMAND = 'chart-to-csv'
FAMILY = 'count_eq'
CLAIM_ID = 'V-unit-chart-to-csv'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    declared = int(f0); actual = int(f1)
    if declared < 0 or actual < 0:
        return "USAGE"
    return "COUNT_OK" if declared == actual else "COUNT_DRIFT"

    raise ValueError(family)
