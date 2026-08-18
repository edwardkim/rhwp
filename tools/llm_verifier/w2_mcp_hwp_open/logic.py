from __future__ import annotations

COMMAND = 'mcp-hwp-open'
FAMILY = 'lease'
CLAIM_ID = 'V-w2-mcp-hwp-open'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    open_flag = int(f0); expired = int(f1); exit_code = int(f2)
    if exit_code not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    if not open_flag:
        return "SESSION_CLOSED"
    if expired:
        return "LEASE_STALE"
    return "LEASE_OK" if exit_code == 0 else "LEASE_FAIL"

    raise ValueError(family)
