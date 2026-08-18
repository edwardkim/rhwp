from __future__ import annotations

COMMAND = 'mcp-hwp-doc-search'
FAMILY = 'search'
CLAIM_ID = 'V-w2-mcp-hwp-doc-search'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    match_count = int(f0); array_len = int(f1); page = int(f2); page_count = int(f3)
    if match_count < 0 or array_len < 0 or page_count < 0:
        return "USAGE"
    if match_count != array_len:
        return "COUNT_DRIFT"
    if page < 0 or page >= page_count:
        return "COORD_OOB"
    return "SEARCH_OK"

    raise ValueError(family)
