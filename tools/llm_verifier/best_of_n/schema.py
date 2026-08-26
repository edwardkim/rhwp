"""Closed types for V-bon outcome ranking.

Field names follow existing rhwp JSON envelopes (knowledge map §2-2).
This module does not invent a CLI or a process-step reward.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Mapping


class CommandFamily(str, Enum):
    FILL_FIELDS = "fill-fields"
    CSV_TO_TABLE = "csv-to-table"
    IR_DIFF = "ir-diff"
    CONVERT = "convert"
    REPLACE_TEXT = "replace-text"
    REDACT = "redact"
    SET_CELL = "set-cell"
    CSV_TO_CHART = "csv-to-chart"
    SANITIZE = "sanitize"
    RUN = "run"

    @property
    def argv_head(self) -> tuple[str, ...]:
        return {
            CommandFamily.FILL_FIELDS: ("edit", "fill-fields"),
            CommandFamily.CSV_TO_TABLE: ("csv-to-table",),
            CommandFamily.IR_DIFF: ("ir-diff",),
            CommandFamily.CONVERT: ("convert",),
            CommandFamily.REPLACE_TEXT: ("edit", "replace-text"),
            CommandFamily.REDACT: ("edit", "redact"),
            CommandFamily.SET_CELL: ("edit", "set-cell"),
            CommandFamily.CSV_TO_CHART: ("csv-to-chart",),
            CommandFamily.SANITIZE: ("edit", "sanitize"),
            CommandFamily.RUN: ("run",),
        }[self]

    @property
    def has_dry_run(self) -> bool:
        return self is not CommandFamily.IR_DIFF

    @property
    def has_verify(self) -> bool:
        return self is not CommandFamily.IR_DIFF


class Mode(str, Enum):
    DRY_RUN = "dry-run"
    VERIFY = "verify"
    IR_DIFF = "ir-diff"

    def argv_flag(self) -> tuple[str, ...]:
        if self is Mode.DRY_RUN:
            return ("--dry-run", "--json")
        if self is Mode.VERIFY:
            return ("--verify", "--json")
        return ("--json",)


RANK_FIELDS: tuple[str, ...] = (
    "changedCount",
    "invalid",
    "verify.identical",
    "exitClass",
)

# process_steps is V-step (#5490). V-bon must not read or emit it.
FORBIDDEN_KEYS: tuple[str, ...] = (
    "process_steps",
    "processSteps",
    "proseScore",
    "llmScore",
    "rubricScore",
    "stepReward",
)


def invalid_is_set(invalid: Any) -> bool:
    """True when the existing envelope says the candidate is invalid."""
    if invalid is True:
        return True
    if invalid is False or invalid is None:
        return False
    if isinstance(invalid, (list, tuple)):
        return len(invalid) > 0
    if isinstance(invalid, dict):
        return True
    if isinstance(invalid, str):
        return bool(invalid)
    return bool(invalid)


def invalid_fingerprint(invalid: Any) -> str:
    if not invalid_is_set(invalid):
        return "clean"
    if invalid is True:
        return "bool:true"
    if isinstance(invalid, (list, tuple)):
        reasons = []
        for item in invalid:
            if isinstance(item, Mapping):
                reasons.append(str(item.get("reason", item.get("code", "item"))))
            else:
                reasons.append(str(item))
        return "list:" + ",".join(reasons) if reasons else "list"
    if isinstance(invalid, Mapping):
        return f"obj:{invalid.get('reason', 'obj')}"
    return f"other:{invalid!r}"


@dataclass(frozen=True)
class CandidateOutcome:
    """One final candidate. Ranking reads only the four envelope fields."""

    candidate_id: str
    changed_count: int
    invalid: Any
    verify_identical: bool | None
    exit_class: int
    envelope: Mapping[str, Any] = field(default_factory=dict)

    def is_invalid(self) -> bool:
        return invalid_is_set(self.invalid)

    def rank_tuple_payload(self) -> dict[str, Any]:
        return {
            "candidateId": self.candidate_id,
            "changedCount": self.changed_count,
            "invalid": self.invalid,
            "verify": (
                None
                if self.verify_identical is None
                else {"identical": self.verify_identical}
            ),
            "exitClass": self.exit_class,
        }


@dataclass(frozen=True)
class CandidateSet:
    set_id: str
    command: CommandFamily
    mode: Mode
    intended_changed_count: int
    candidates: tuple[CandidateOutcome, ...]
    sample: str = ""
    source_format: str = "hwpx"

    @property
    def n(self) -> int:
        return len(self.candidates)
