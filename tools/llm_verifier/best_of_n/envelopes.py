"""Lift existing rhwp JSON envelopes into the four V-bon rank fields.

Does not run rhwp. Does not invent fields. process_steps is refused.
"""

from __future__ import annotations

from typing import Any, Mapping

try:
    from .schema import FORBIDDEN_KEYS, CandidateOutcome, invalid_is_set
except ImportError:
    from schema import FORBIDDEN_KEYS, CandidateOutcome, invalid_is_set

EXIT_NAME_TO_CODE: dict[str, int] = {
    "ok": 0,
    "success": 0,
    "0": 0,
    "io": 1,
    "runtime": 1,
    "1": 1,
    "usage": 2,
    "2": 2,
    "judgment": 3,
    "judgement": 3,
    "3": 3,
    "page_verify": 4,
    "pageVerify": 4,
    "verify_pages": 4,
    "4": 4,
}


def refuse_process_reward(blob: Mapping[str, Any], *, path: str = "") -> None:
    for key in FORBIDDEN_KEYS:
        if key in blob:
            loc = f"{path}.{key}" if path else key
            raise ValueError(f"V-bon refuses process/prose field {loc} (see #5490)")


def parse_exit_class(raw: Any) -> int:
    if raw is None:
        raise ValueError("exitClass is required for outcome ranking")
    if isinstance(raw, bool):
        raise ValueError("exitClass must not be a boolean")
    if isinstance(raw, int):
        if raw in {0, 1, 2, 3, 4}:
            return raw
        raise ValueError(f"unknown rhwp exitClass {raw}; only 0/1/2/3/4")
    if isinstance(raw, float) and raw.is_integer():
        return parse_exit_class(int(raw))
    if isinstance(raw, str):
        mapped = EXIT_NAME_TO_CODE.get(raw)
        if mapped is not None:
            return mapped
        if raw.isdigit():
            return parse_exit_class(int(raw))
        raise ValueError(f"unknown exitClass name {raw!r}")
    raise ValueError(f"unreadable exitClass {raw!r}")


def _as_int(raw: Any, default: int = 0) -> int:
    if raw is None:
        return default
    if isinstance(raw, bool):
        raise ValueError("numeric envelope field must not be a boolean")
    if isinstance(raw, int):
        return raw
    if isinstance(raw, float) and raw.is_integer():
        return int(raw)
    if isinstance(raw, str) and raw.lstrip("-").isdigit():
        return int(raw)
    raise ValueError(f"not an int: {raw!r}")


def lift_changed_count(envelope: Mapping[str, Any]) -> int:
    if "changedCount" in envelope and envelope["changedCount"] is not None:
        return _as_int(envelope["changedCount"])
    for key in ("filledCount", "replacedCount", "redactedCount", "removedCount"):
        if key in envelope and envelope[key] is not None:
            return _as_int(envelope[key])
    if "diffCount" in envelope and envelope["diffCount"] is not None:
        return _as_int(envelope["diffCount"])
    verify = envelope.get("verify")
    if isinstance(verify, Mapping) and verify.get("diffCount") is not None:
        return _as_int(verify["diffCount"])
    return 0


def lift_verify_identical(envelope: Mapping[str, Any]) -> bool | None:
    verify = envelope.get("verify")
    if isinstance(verify, Mapping) and "identical" in verify:
        ident = verify["identical"]
        if ident is None:
            return None
        if isinstance(ident, bool):
            return ident
        raise ValueError("verify.identical must be bool or null")
    if "identical" in envelope:
        ident = envelope["identical"]
        if ident is None:
            return None
        if isinstance(ident, bool):
            return ident
        raise ValueError("identical must be bool or null")
    return None


def lift_invalid(envelope: Mapping[str, Any]) -> Any:
    if "invalid" in envelope:
        return envelope["invalid"]
    return []


def lift_envelope(
    envelope: Mapping[str, Any],
    *,
    candidate_id: str,
    exit_class: Any = None,
) -> CandidateOutcome:
    refuse_process_reward(envelope)
    if exit_class is None:
        if "exitClass" in envelope:
            exit_class = envelope["exitClass"]
        elif "exit" in envelope:
            exit_class = envelope["exit"]
        else:
            raise ValueError("envelope is missing exitClass")
    return CandidateOutcome(
        candidate_id=candidate_id,
        changed_count=lift_changed_count(envelope),
        invalid=lift_invalid(envelope),
        verify_identical=lift_verify_identical(envelope),
        exit_class=parse_exit_class(exit_class),
        envelope=dict(envelope),
    )


def lift_candidate_record(row: Mapping[str, Any]) -> CandidateOutcome:
    refuse_process_reward(row)
    envelope = row.get("envelope")
    if isinstance(envelope, Mapping):
        refuse_process_reward(envelope, path="envelope")
        exit_raw = row.get("exitClass", envelope.get("exitClass", envelope.get("exit")))
        candidate_id = str(row.get("candidateId") or row.get("id") or "c0")
        lifted = lift_envelope(envelope, candidate_id=candidate_id, exit_class=exit_raw)
        # Rank fields on the record override the inner envelope when present.
        changed = (
            _as_int(row["changedCount"])
            if "changedCount" in row and row["changedCount"] is not None
            else lifted.changed_count
        )
        invalid = row["invalid"] if "invalid" in row else lifted.invalid
        if "verify" in row:
            verify = row["verify"]
            ident = (
                verify.get("identical")
                if isinstance(verify, Mapping)
                else (verify if isinstance(verify, bool) else lifted.verify_identical)
            )
        else:
            ident = lifted.verify_identical
        exit_class = (
            parse_exit_class(row["exitClass"])
            if "exitClass" in row
            else lifted.exit_class
        )
        return CandidateOutcome(
            candidate_id=candidate_id,
            changed_count=changed,
            invalid=invalid,
            verify_identical=ident,
            exit_class=exit_class,
            envelope=dict(envelope),
        )
    candidate_id = str(row.get("candidateId") or row.get("id") or "c0")
    return lift_envelope(row, candidate_id=candidate_id)


def envelope_looks_inconsistent(outcome: CandidateOutcome) -> bool:
    """Field-only consistency notes. Does not change the rank key."""
    if outcome.exit_class == 0 and outcome.verify_identical is False:
        return True
    if outcome.exit_class == 3 and outcome.verify_identical is True and not outcome.is_invalid():
        return True
    if outcome.exit_class == 0 and invalid_is_set(outcome.invalid):
        return True
    if outcome.exit_class in {1, 2} and outcome.verify_identical is True:
        return True
    return False
