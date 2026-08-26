"""Best-of-N outcome ranker.

Order is a closed lexicographic key of existing envelope fields:

    (invalid, exitClass, verify.identical, |changedCount-intended|, changedCount, id)

Lower is better. There is no prose score and no per-step process reward.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Iterable, Mapping, Sequence

try:
    from .envelopes import envelope_looks_inconsistent, lift_candidate_record
    from .schema import (
        FORBIDDEN_KEYS,
        RANK_FIELDS,
        CandidateOutcome,
        CandidateSet,
        CommandFamily,
        Mode,
        invalid_is_set,
    )
except ImportError:
    from envelopes import envelope_looks_inconsistent, lift_candidate_record
    from schema import (
        FORBIDDEN_KEYS,
        RANK_FIELDS,
        CandidateOutcome,
        CandidateSet,
        CommandFamily,
        Mode,
        invalid_is_set,
    )

CLAIM_ID = "V-bon"
SCHEMA_VERSION = "1.0"
KIND = "bestOfNRanking"

# Smaller is better. 0 = rhwp success, 3 = judgment (tool ran), 4/1/2 worse.
EXIT_RANK: dict[int, int] = {
    0: 0,
    3: 1,
    4: 2,
    1: 3,
    2: 4,
}


@dataclass(frozen=True)
class OutcomeKey:
    invalid_rank: int
    exit_rank: int
    identical_rank: int
    change_delta: int
    changed_count: int
    candidate_id: str

    def as_tuple(self) -> tuple[int, int, int, int, int, str]:
        return (
            self.invalid_rank,
            self.exit_rank,
            self.identical_rank,
            self.change_delta,
            self.changed_count,
            self.candidate_id,
        )

    def to_json(self) -> dict[str, int | str]:
        return {
            "invalidRank": self.invalid_rank,
            "exitRank": self.exit_rank,
            "identicalRank": self.identical_rank,
            "changeDelta": self.change_delta,
            "changedCount": self.changed_count,
            "candidateId": self.candidate_id,
        }


@dataclass(frozen=True)
class RankedCandidate:
    candidate: CandidateOutcome
    key: OutcomeKey
    expected_rank: int
    inconsistent: bool

    def to_json(self) -> dict[str, Any]:
        return {
            "candidateId": self.candidate.candidate_id,
            "expectedRank": self.expected_rank,
            "changedCount": self.candidate.changed_count,
            "invalid": self.candidate.invalid,
            "verify": (
                None
                if self.candidate.verify_identical is None
                else {"identical": self.candidate.verify_identical}
            ),
            "exitClass": self.candidate.exit_class,
            "inconsistent": self.inconsistent,
            "key": self.key.to_json(),
        }


@dataclass(frozen=True)
class RankedSet:
    set_id: str
    command: str
    mode: str
    n: int
    intended_changed_count: int
    ranking: tuple[RankedCandidate, ...]
    winner_id: str

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM_ID,
            "kind": KIND,
            "setId": self.set_id,
            "command": self.command,
            "mode": self.mode,
            "n": self.n,
            "intendedChangedCount": self.intended_changed_count,
            "winnerId": self.winner_id,
            "rankFields": list(RANK_FIELDS),
            "ranking": [row.to_json() for row in self.ranking],
        }


def identical_rank(value: bool | None) -> int:
    if value is True:
        return 0
    if value is None:
        return 1
    return 2


def outcome_key(candidate: CandidateOutcome, intended_changed_count: int) -> OutcomeKey:
    changed = candidate.changed_count
    if changed < 0:
        raise ValueError(f"changedCount must be >= 0, got {changed}")
    return OutcomeKey(
        invalid_rank=1 if invalid_is_set(candidate.invalid) else 0,
        exit_rank=EXIT_RANK.get(candidate.exit_class, 5),
        identical_rank=identical_rank(candidate.verify_identical),
        change_delta=abs(changed - intended_changed_count),
        changed_count=changed,
        candidate_id=candidate.candidate_id,
    )


def rank_candidates(
    candidates: Sequence[CandidateOutcome],
    *,
    intended_changed_count: int,
    set_id: str = "",
    command: str = "",
    mode: str = "",
) -> RankedSet:
    if not candidates:
        raise ValueError("Best-of-N requires at least one candidate")
    if intended_changed_count < 0:
        raise ValueError("intendedChangedCount must be >= 0")
    ids = [c.candidate_id for c in candidates]
    if len(ids) != len(set(ids)):
        raise ValueError(f"duplicate candidateId in set {set_id or ids}")

    keyed = [(outcome_key(c, intended_changed_count), c) for c in candidates]
    keyed.sort(key=lambda item: item[0].as_tuple())

    ranked: list[RankedCandidate] = []
    prev_cmp: tuple[int, int, int, int, int] | None = None
    rank = 0
    for index, (key, cand) in enumerate(keyed, start=1):
        cmp = (
            key.invalid_rank,
            key.exit_rank,
            key.identical_rank,
            key.change_delta,
            key.changed_count,
        )
        if prev_cmp != cmp:
            rank = index
            prev_cmp = cmp
        ranked.append(
            RankedCandidate(
                candidate=cand,
                key=key,
                expected_rank=rank,
                inconsistent=envelope_looks_inconsistent(cand),
            )
        )
    return RankedSet(
        set_id=set_id,
        command=command,
        mode=mode,
        n=len(ranked),
        intended_changed_count=intended_changed_count,
        ranking=tuple(ranked),
        winner_id=ranked[0].candidate.candidate_id,
    )


def rank_set(candidate_set: CandidateSet) -> RankedSet:
    return rank_candidates(
        candidate_set.candidates,
        intended_changed_count=candidate_set.intended_changed_count,
        set_id=candidate_set.set_id,
        command=candidate_set.command.value,
        mode=candidate_set.mode.value,
    )


def rank_mapping(blob: Mapping[str, Any]) -> RankedSet:
    for key in FORBIDDEN_KEYS:
        if key in blob:
            raise ValueError(f"V-bon refuses process/prose field {key} (see #5490)")
    raw_candidates = blob.get("candidates")
    if not isinstance(raw_candidates, Iterable) or isinstance(raw_candidates, (str, bytes)):
        raise ValueError("candidates must be a list")
    candidates = [lift_candidate_record(row) for row in raw_candidates]
    command = str(blob.get("command") or "")
    mode = str(blob.get("mode") or "")
    intended = blob.get("intendedChangedCount", 0)
    if not isinstance(intended, int) or isinstance(intended, bool):
        raise ValueError("intendedChangedCount must be an int")
    return rank_candidates(
        candidates,
        intended_changed_count=intended,
        set_id=str(blob.get("setId") or blob.get("id") or ""),
        command=command,
        mode=mode,
    )


def expected_ranks_match(blob: Mapping[str, Any]) -> list[str]:
    """Return mismatches of recorded expectedRank vs the ranker."""
    computed = rank_mapping(blob)
    by_id = {row.candidate.candidate_id: row.expected_rank for row in computed.ranking}
    errors: list[str] = []
    for row in blob.get("candidates") or ():
        cid = str(row.get("candidateId") or row.get("id") or "")
        if "expectedRank" not in row:
            errors.append(f"{cid}: missing expectedRank")
            continue
        got = int(row["expectedRank"])
        want = by_id[cid]
        if got != want:
            errors.append(f"{cid}: expectedRank={got} ranker={want}")
    return errors


def command_mode_ok(command: str, mode: str) -> bool:
    try:
        fam = CommandFamily(command)
    except ValueError:
        return False
    try:
        md = Mode(mode)
    except ValueError:
        return False
    if md is Mode.IR_DIFF:
        return fam is CommandFamily.IR_DIFF
    if md is Mode.DRY_RUN:
        return fam.has_dry_run
    if md is Mode.VERIFY:
        return fam.has_verify
    return False
