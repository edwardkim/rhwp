from __future__ import annotations

COMMAND = 'csv-to-table'
FAMILY = 'dims'
CLAIM_ID = 'V-unit-csv-to-table'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    rows_in = int(f0); cols_in = int(f1); rows_out = int(f2); cols_out = int(f3)
    if min(rows_in, cols_in, rows_out, cols_out) <= 0:
        return "USAGE"
    if rows_in != rows_out or cols_in != cols_out:
        return "DIM_DRIFT"
    return "DIM_OK"

    raise ValueError(family)
