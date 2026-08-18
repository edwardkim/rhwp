from __future__ import annotations

COMMAND = 'hwp5-roundtrip'
FAMILY = 'round'
CLAIM_ID = 'V-unit-hwp5-roundtrip'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    before = int(f0); after = int(f1); same = int(f2)
    if before < 0 or after < 0:
        return "USAGE"
    if not same:
        return "FORMAT_NA"
    return "ROUND_OK" if before == after else "ROUND_DRIFT"

    raise ValueError(family)
