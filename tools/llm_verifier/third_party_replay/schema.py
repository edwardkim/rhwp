"""Wire types for one third-party replay case.

Required axis tuple (issue #5502):

    (plan, expect_sha, reproduced, toolVersion, verdict)

`plan` is the plan *text* the third party must replay (`planSha256` target).
`expect_sha` is `--expect-output-sha256` / `expectedOutputSha256`.
`reproduced` is the envelope bool (`null` on attest).
`toolVersion` is the receipt field that must be compared before replay.
`verdict` is this verifier's closed class — never implementer prose.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from enum import Enum
from typing import Any, Mapping

CASE_COLUMNS: tuple[str, ...] = (
    "case_id",
    "plan",
    "expect_sha",
    "reproduced",
    "tool_version",
    "verdict",
    "mode",
    "source",
    "plan_sha256",
    "output_sha256",
    "expected_tool_version",
    "input_path",
    "action",
    "exit_class",
    "labor_accepted",
    "evidence_kind",
    "implementer_claim",
    "sha_defect",
    "family",
)

TRUTHY = frozenset({"1", "true", "yes", "y", "on"})
FALSY = frozenset({"0", "false", "no", "n", "off"})


class ReplayMode(str, Enum):
    VERIFY = "verify"
    ATTEST = "attest"
    ABSENT = "absent"


class ReplaySource(str, Enum):
    REPLAY = "replay"
    CAPSULE = "capsule"
    PROSE = "prose"


def parse_reproduced(value: Any) -> bool | None:
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
    raise ValueError(f"reproduced expected bool|null, got {value!r}")


def reproduced_cell(value: bool | None) -> str:
    if value is None:
        return ""
    return "1" if value else "0"


def parse_mode(value: Any) -> ReplayMode:
    text = str(value or "").strip().lower()
    if text in {"", "none", "absent"}:
        return ReplayMode.ABSENT
    if text == ReplayMode.VERIFY.value:
        return ReplayMode.VERIFY
    if text == ReplayMode.ATTEST.value:
        return ReplayMode.ATTEST
    raise ValueError(f"unknown replay mode {value!r}")


def parse_source(value: Any) -> ReplaySource:
    text = str(value or "").strip().lower()
    if text == ReplaySource.REPLAY.value:
        return ReplaySource.REPLAY
    if text == ReplaySource.CAPSULE.value:
        return ReplaySource.CAPSULE
    if text in {ReplaySource.PROSE.value, "implementer", "narrative"}:
        return ReplaySource.PROSE
    raise ValueError(f"unknown replay source {value!r}")


@dataclass(frozen=True)
class ReplayObservation:
    plan: str
    expect_sha: str
    reproduced: bool | None
    tool_version: str
    mode: ReplayMode
    source: ReplaySource
    expected_tool_version: str = ""

    def axis_tuple(self) -> tuple[str, str, bool | None, str]:
        return (self.plan, self.expect_sha, self.reproduced, self.tool_version)


@dataclass(frozen=True)
class ReplayCase:
    case_id: str
    plan: str
    expect_sha: str
    reproduced: bool | None
    tool_version: str
    verdict: str
    mode: str
    source: str
    plan_sha256: str
    output_sha256: str
    expected_tool_version: str
    input_path: str
    action: str
    exit_class: str
    labor_accepted: bool
    evidence_kind: str
    implementer_claim: str
    sha_defect: str
    family: str

    def observation(self) -> ReplayObservation:
        return ReplayObservation(
            plan=self.plan,
            expect_sha=self.expect_sha,
            reproduced=self.reproduced,
            tool_version=self.tool_version,
            mode=parse_mode(self.mode),
            source=parse_source(self.source),
            expected_tool_version=self.expected_tool_version,
        )

    def identity_key(self) -> tuple[Any, ...]:
        return (
            self.plan,
            self.expect_sha,
            self.reproduced,
            self.tool_version,
            self.verdict,
        )

    def to_row(self) -> dict[str, str]:
        data = asdict(self)
        data["reproduced"] = reproduced_cell(self.reproduced)
        data["labor_accepted"] = "1" if self.labor_accepted else "0"
        return {key: "" if data[key] is None else str(data[key]) for key in CASE_COLUMNS}

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "ReplayCase":
        return cls(
            case_id=str(row.get("case_id", "")),
            plan=str(row.get("plan", "")),
            expect_sha=str(row.get("expect_sha", "")),
            reproduced=parse_reproduced(row.get("reproduced")),
            tool_version=str(row.get("tool_version", row.get("toolVersion", ""))),
            verdict=str(row.get("verdict", "")),
            mode=str(row.get("mode", "")),
            source=str(row.get("source", "")),
            plan_sha256=str(row.get("plan_sha256", "")),
            output_sha256=str(row.get("output_sha256", "")),
            expected_tool_version=str(row.get("expected_tool_version", "")),
            input_path=str(row.get("input_path", "")),
            action=str(row.get("action", "")),
            exit_class=str(row.get("exit_class", "")),
            labor_accepted=parse_reproduced(row.get("labor_accepted")) is True,
            evidence_kind=str(row.get("evidence_kind", "")),
            implementer_claim=str(row.get("implementer_claim", "")),
            sha_defect=str(row.get("sha_defect", "")),
            family=str(row.get("family", "")),
        )
