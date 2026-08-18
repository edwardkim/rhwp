from __future__ import annotations

COMMAND = 'edit-split-cell'
FAMILY = 'coord'
CLAIM_ID = 'V-w2-edit-split-cell'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    row = int(f0); col = int(f1); rows = int(f2); cols = int(f3)
    if min(row, col, rows, cols) < 0:
        return "USAGE"
    if rows == 0 or cols == 0:
        return "USAGE"
    if row >= rows or col >= cols:
        return "COORD_OOB"
    return "COORD_OK"

    raise ValueError(family)
