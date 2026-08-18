"""V-bon: rank N candidate outcomes from existing dry-run / --verify / ir-diff envelopes.

Outcome ranking only. process_steps and prose scores are out of scope.
"""

try:
    from .envelopes import lift_envelope, parse_exit_class
    from .rank import (
        CLAIM_ID,
        SCHEMA_VERSION,
        OutcomeKey,
        RankedCandidate,
        RankedSet,
        rank_candidates,
        rank_set,
    )
    from .schema import CandidateOutcome, CommandFamily, Mode
except ImportError:  # script / unittest from this directory
    from envelopes import lift_envelope, parse_exit_class
    from rank import (
        CLAIM_ID,
        SCHEMA_VERSION,
        OutcomeKey,
        RankedCandidate,
        RankedSet,
        rank_candidates,
        rank_set,
    )
    from schema import CandidateOutcome, CommandFamily, Mode

__all__ = [
    "CLAIM_ID",
    "SCHEMA_VERSION",
    "CandidateOutcome",
    "CommandFamily",
    "Mode",
    "OutcomeKey",
    "RankedCandidate",
    "RankedSet",
    "lift_envelope",
    "parse_exit_class",
    "rank_candidates",
    "rank_set",
]
