"""Wire types for one lineage-chain case.

Required axis tuple (issue #5516):

    (parent_out, child_in, parentOk, lineageOk, brokenAt, verdict)

`parent_out` is the parent capsule `receipt.outputSha256`.
`child_in` is the child capsule `receipt.inputSha256`.
`parentOk` is the existing `rhwp lineage` link field (parent file bytes).
`lineageOk` is the existing link field (parent output == child input).
`brokenAt` is the existing envelope field naming the first broken capsule.
`verdict` is this verifier's closed class — never implementer prose.

Single-job `reproduced` is V-replay and is stored only to prove it is ignored.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from enum import Enum
from typing import Any, Mapping

CASE_COLUMNS: tuple[str, ...] = (
    "case_id",
    "parent_out",
    "child_in",
    "parent_ok",
    "lineage_ok",
    "broken_at",
    "verdict",
    "source",
    "kind",
    "parent_state",
    "valid",
    "reproduced",
    "exit_class",
    "chain_accepted",
    "evidence_kind",
    "head",
    "child_capsule",
    "parent_capsule",
    "depth",
    "agency",
    "doc_type",
    "action",
    "implementer_claim",
    "family",
    "hash_defect",
)

TRUTHY = frozenset({"1", "true", "yes", "y", "on"})
FALSY = frozenset({"0", "false", "no", "n", "off"})


class LineageSource(str, Enum):
    LINEAGE = "lineage"
    CAPSULE = "capsule"
    PROSE = "prose"
    IO = "io"
    USAGE = "usage"


class ParentState(str, Enum):
    OK = "ok"
    ROOT = "root"
    FIELD_MISSING = "field_missing"
    SHA_MISSING = "sha_missing"
    ABSENT = "absent"


def parse_optional_bool(value: Any) -> bool | None:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int) and value in (0, 1):
        return bool(value)
    text = str(value).strip().lower()
    if text in {"", "null", "none"}:
        return None
    if text in TRUTHY:
        return True
    if text in FALSY:
        return False
    raise ValueError(f"expected bool|null, got {value!r}")


def optional_bool_cell(value: bool | None) -> str:
    if value is None:
        return ""
    return "1" if value else "0"


def parse_source(value: Any) -> LineageSource:
    text = str(value or "").strip().lower()
    if text == LineageSource.LINEAGE.value:
        return LineageSource.LINEAGE
    if text == LineageSource.CAPSULE.value:
        return LineageSource.CAPSULE
    if text in {LineageSource.PROSE.value, "implementer", "narrative"}:
        return LineageSource.PROSE
    if text in {LineageSource.IO.value, "missing_head", "head_missing"}:
        return LineageSource.IO
    if text == LineageSource.USAGE.value:
        return LineageSource.USAGE
    raise ValueError(f"unknown lineage source {value!r}")


def parse_parent_state(value: Any) -> ParentState:
    text = str(value or "").strip().lower()
    if text in {"", ParentState.OK.value}:
        return ParentState.OK
    if text == ParentState.ROOT.value:
        return ParentState.ROOT
    if text in {ParentState.FIELD_MISSING.value, "missing_parent", "no_parent_key"}:
        return ParentState.FIELD_MISSING
    if text in {ParentState.SHA_MISSING.value, "no_parent_sha", "parent_sha_bad"}:
        return ParentState.SHA_MISSING
    if text == ParentState.ABSENT.value:
        return ParentState.ABSENT
    raise ValueError(f"unknown parent state {value!r}")


@dataclass(frozen=True)
class LineageObservation:
    parent_out: str
    child_in: str
    parent_ok: bool | None
    lineage_ok: bool | None
    broken_at: str
    source: LineageSource
    kind: str = "lineage"
    parent_state: ParentState = ParentState.OK
    valid: bool | None = None
    reproduced: bool | None = None

    def axis_tuple(self) -> tuple[str, str, bool | None, bool | None, str]:
        return (
            self.parent_out,
            self.child_in,
            self.parent_ok,
            self.lineage_ok,
            self.broken_at,
        )


@dataclass(frozen=True)
class LineageCase:
    case_id: str
    parent_out: str
    child_in: str
    parent_ok: bool | None
    lineage_ok: bool | None
    broken_at: str
    verdict: str
    source: str
    kind: str
    parent_state: str
    valid: bool | None
    reproduced: bool | None
    exit_class: str
    chain_accepted: bool
    evidence_kind: str
    head: str
    child_capsule: str
    parent_capsule: str
    depth: str
    agency: str
    doc_type: str
    action: str
    implementer_claim: str
    family: str
    hash_defect: str

    def observation(self) -> LineageObservation:
        return LineageObservation(
            parent_out=self.parent_out,
            child_in=self.child_in,
            parent_ok=self.parent_ok,
            lineage_ok=self.lineage_ok,
            broken_at=self.broken_at,
            source=parse_source(self.source),
            kind=self.kind,
            parent_state=parse_parent_state(self.parent_state),
            valid=self.valid,
            reproduced=self.reproduced,
        )

    def identity_key(self) -> tuple[Any, ...]:
        return (
            self.parent_out,
            self.child_in,
            self.parent_ok,
            self.lineage_ok,
            self.broken_at,
            self.verdict,
        )

    def to_row(self) -> dict[str, str]:
        data = asdict(self)
        data["parent_ok"] = optional_bool_cell(self.parent_ok)
        data["lineage_ok"] = optional_bool_cell(self.lineage_ok)
        data["valid"] = optional_bool_cell(self.valid)
        data["reproduced"] = optional_bool_cell(self.reproduced)
        data["chain_accepted"] = "1" if self.chain_accepted else "0"
        return {key: "" if data[key] is None else str(data[key]) for key in CASE_COLUMNS}

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "LineageCase":
        return cls(
            case_id=str(row.get("case_id", "")),
            parent_out=str(row.get("parent_out", row.get("parentOut", ""))),
            child_in=str(row.get("child_in", row.get("childIn", ""))),
            parent_ok=parse_optional_bool(row.get("parent_ok", row.get("parentOk"))),
            lineage_ok=parse_optional_bool(row.get("lineage_ok", row.get("lineageOk"))),
            broken_at=str(row.get("broken_at", row.get("brokenAt", ""))),
            verdict=str(row.get("verdict", "")),
            source=str(row.get("source", "")),
            kind=str(row.get("kind", "")),
            parent_state=str(row.get("parent_state", "")),
            valid=parse_optional_bool(row.get("valid")),
            reproduced=parse_optional_bool(row.get("reproduced")),
            exit_class=str(row.get("exit_class", "")),
            chain_accepted=parse_optional_bool(row.get("chain_accepted")) is True,
            evidence_kind=str(row.get("evidence_kind", "")),
            head=str(row.get("head", "")),
            child_capsule=str(row.get("child_capsule", "")),
            parent_capsule=str(row.get("parent_capsule", "")),
            depth=str(row.get("depth", "")),
            agency=str(row.get("agency", "")),
            doc_type=str(row.get("doc_type", "")),
            action=str(row.get("action", "")),
            implementer_claim=str(row.get("implementer_claim", "")),
            family=str(row.get("family", "")),
            hash_defect=str(row.get("hash_defect", "")),
        )
