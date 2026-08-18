from __future__ import annotations

COMMAND = 'vscode-preview'
FAMILY = 'bytes'
CLAIM_ID = 'V-w2-vscode-preview'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    n = int(f0); empty = int(f1)
    if n < 0:
        return "USAGE"
    if empty:
        return "EMPTY_OUTPUT"
    if n == 0:
        return "ZERO_BYTES"
    return "BYTES_OK"

    raise ValueError(family)
