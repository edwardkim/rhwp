"""V-shadow: two different mechanical commands must both pass.

This package decides joint pass/fail from a pair of *already published*
rhwp command envelopes. It does not invent a new rhwp CLI. One command
passing is not enough. Same-envelope field contradiction is V-abstain,
not this axis.
"""

try:
    from .checks import (
        CHECKS,
        MechanicalCheck,
        check_by_id,
        command_key,
        iter_checks,
        iter_distinct_pairs,
    )
    from .decide import (
        CLAIM_ID,
        SCHEMA_VERSION,
        Decision,
        DecisionInputs,
        decide,
        decide_row,
    )
except ImportError:
    from checks import (
        CHECKS,
        MechanicalCheck,
        check_by_id,
        command_key,
        iter_checks,
        iter_distinct_pairs,
    )
    from decide import (
        CLAIM_ID,
        SCHEMA_VERSION,
        Decision,
        DecisionInputs,
        decide,
        decide_row,
    )

__all__ = [
    "CHECKS",
    "CLAIM_ID",
    "Decision",
    "DecisionInputs",
    "MechanicalCheck",
    "SCHEMA_VERSION",
    "check_by_id",
    "command_key",
    "decide",
    "decide_row",
    "iter_checks",
    "iter_distinct_pairs",
]
