"""Wire types for a V-shadow decision case."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any, Mapping

CASE_COLUMNS: tuple[str, ...] = (
    "case_id",
    "check_a",
    "check_b",
    "a_pass",
    "b_pass",
    "expected_joint",
    "expected_verdict_class",
    "sample",
    "source_format",
    "family",
    "agency",
    "year",
    "page_a",
    "page_b",
    "command_a",
    "command_b",
    "field_a",
    "field_b",
    "observed_a",
    "observed_b",
    "honest_claim",
    "contract_source",
    "not_abstain",
    "not_repeat",
)

TRUTHY = frozenset({"1", "true", "yes", "y", "on"})
FALSY = frozenset({"0", "false", "no", "n", "off", ""})


def parse_bool(value: Any) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, int) and value in (0, 1):
        return bool(value)
    text = str(value).strip().lower()
    if text in TRUTHY:
        return True
    if text in FALSY:
        return False
    raise ValueError(f"boolean expected, got {value!r}")


def bool_cell(value: bool) -> str:
    return "1" if value else "0"


@dataclass(frozen=True)
class DecisionCase:
    case_id: str
    check_a: str
    check_b: str
    a_pass: bool
    b_pass: bool
    expected_joint: bool
    expected_verdict_class: str
    sample: str
    source_format: str
    family: str
    agency: str
    year: str
    page_a: int
    page_b: int
    command_a: str
    command_b: str
    field_a: str
    field_b: str
    observed_a: str
    observed_b: str
    honest_claim: str
    contract_source: str
    not_abstain: bool
    not_repeat: bool

    def axis_tuple(self) -> tuple[str, str, bool, bool, bool, str]:
        return (
            self.check_a,
            self.check_b,
            self.a_pass,
            self.b_pass,
            self.expected_joint,
            self.expected_verdict_class,
        )

    def identity_key(self) -> tuple[Any, ...]:
        return (
            self.case_id,
            self.check_a,
            self.check_b,
            self.a_pass,
            self.b_pass,
            self.sample,
            self.source_format,
            self.family,
            self.agency,
            self.year,
            self.page_a,
            self.page_b,
            self.observed_a,
            self.observed_b,
        )

    def to_row(self) -> dict[str, str]:
        data = asdict(self)
        data["a_pass"] = bool_cell(self.a_pass)
        data["b_pass"] = bool_cell(self.b_pass)
        data["expected_joint"] = bool_cell(self.expected_joint)
        data["not_abstain"] = bool_cell(self.not_abstain)
        data["not_repeat"] = bool_cell(self.not_repeat)
        data["page_a"] = str(self.page_a)
        data["page_b"] = str(self.page_b)
        return data

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "DecisionCase":
        return cls(
            case_id=str(row["case_id"]),
            check_a=str(row["check_a"]),
            check_b=str(row["check_b"]),
            a_pass=parse_bool(row["a_pass"]),
            b_pass=parse_bool(row["b_pass"]),
            expected_joint=parse_bool(row["expected_joint"]),
            expected_verdict_class=str(row["expected_verdict_class"]),
            sample=str(row.get("sample", "")),
            source_format=str(row.get("source_format", "")),
            family=str(row.get("family", "")),
            agency=str(row.get("agency", "")),
            year=str(row.get("year", "")),
            page_a=int(row.get("page_a", 0) or 0),
            page_b=int(row.get("page_b", 0) or 0),
            command_a=str(row.get("command_a", "")),
            command_b=str(row.get("command_b", "")),
            field_a=str(row.get("field_a", "")),
            field_b=str(row.get("field_b", "")),
            observed_a=str(row.get("observed_a", "")),
            observed_b=str(row.get("observed_b", "")),
            honest_claim=str(row.get("honest_claim", "")),
            contract_source=str(row.get("contract_source", "")),
            not_abstain=parse_bool(row.get("not_abstain", "1")),
            not_repeat=parse_bool(row.get("not_repeat", "1")),
        )
