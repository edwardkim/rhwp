from __future__ import annotations

COMMAND = 'agent-surface-stateless'
FAMILY = 'layer'
CLAIM_ID = 'V-w2-agent-surface-stateless'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    rpc_error = int(f0); is_error = int(f1); exit_code = int(f2)
    if rpc_error:
        return "RPC_FAIL"
    if is_error:
        return "TOOL_FAIL"
    if exit_code == 0:
        return "ENV_OK"
    if exit_code in (1, 2, 3, 4):
        return "ENV_JUDGE"
    return "ENV_UNKNOWN"

    raise ValueError(family)
