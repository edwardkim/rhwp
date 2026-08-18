"""Wire types for V-abstain field-tuple cases.

Field names follow existing rhwp --json envelopes (knowledge map §2).
This module does not invent a rhwp CLI and does not reimplement V-proto.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any, Mapping, Optional

CLAIM_ID = "V-abstain"
SCHEMA_VERSION = "1.0"
KIND = "abstainOnContradiction"

VERDICT_ABSTAIN = "abstain"
VERDICT_PASS = "pass"
VERDICT_FAIL = "fail"

VERDICTS: tuple[str, ...] = (VERDICT_ABSTAIN, VERDICT_PASS, VERDICT_FAIL)

COMMANDS: tuple[str, ...] = (
    "info",
    "verify",
    "ir-diff",
    "layout-anomaly",
    "replay",
    "fill-fields",
    "render-diff",
)

CASE_COLUMNS: tuple[str, ...] = (
    "case_id",
    "command",
    "exit",
    "identical",
    "has_signal",
    "reproduced",
    "page_count_a",
    "page_count_b",
    "page_count_mismatch",
    "struct_status",
    "struct_node",
    "page_count_node",
    "verify_identical",
    "verify_diff_count",
    "diff_count",
    "fail_count",
    "pass_count",
    "verdict",
    "regression",
    "status",
    "clean",
    "signal_count",
    "valid",
    "finding_count",
    "overflow_count",
    "overlap_count",
    "empty_page_count",
    "expected",
    "contradiction_id",
    "success_tokens",
    "fail_tokens",
    "sample",
    "source_format",
    "agency",
    "doc_kind",
    "year",
    "family",
)

TRUTHY = frozenset({"1", "true", "yes", "y", "on"})
FALSY = frozenset({"0", "false", "no", "n", "off"})


def parse_optional_bool(value: Any) -> Optional[bool]:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int) and value in (0, 1):
        return bool(value)
    text = str(value).strip().lower()
    if text in {"", "-", "null", "none"}:
        return None
    if text in TRUTHY:
        return True
    if text in FALSY:
        return False
    raise ValueError(f"optional boolean expected, got {value!r}")


def parse_optional_int(value: Any) -> Optional[int]:
    if value is None:
        return None
    if isinstance(value, bool):
        raise ValueError("bool is not an int field")
    if isinstance(value, int):
        return value
    text = str(value).strip().lower()
    if text in {"", "-", "null", "none"}:
        return None
    return int(text)


def parse_optional_str(value: Any) -> Optional[str]:
    if value is None:
        return None
    text = str(value)
    if text.strip().lower() in {"", "-", "null", "none"}:
        return None
    return text


def bool_cell(value: Optional[bool]) -> str:
    if value is None:
        return ""
    return "1" if value else "0"


def int_cell(value: Optional[int]) -> str:
    if value is None:
        return ""
    return str(value)


def str_cell(value: Optional[str]) -> str:
    return "" if value is None else value


@dataclass(frozen=True)
class EnvelopeFields:
    """Existing envelope fields a verifier may read. Absent = None."""

    command: str
    exit: int
    identical: Optional[bool] = None
    has_signal: Optional[bool] = None
    reproduced: Optional[bool] = None
    page_count_a: Optional[int] = None
    page_count_b: Optional[int] = None
    page_count_mismatch: Optional[bool] = None
    struct_status: Optional[str] = None
    struct_node: Optional[str] = None
    page_count_node: Optional[str] = None
    verify_identical: Optional[bool] = None
    verify_diff_count: Optional[int] = None
    diff_count: Optional[int] = None
    fail_count: Optional[int] = None
    pass_count: Optional[int] = None
    verdict: Optional[str] = None
    regression: Optional[bool] = None
    status: Optional[str] = None
    clean: Optional[bool] = None
    signal_count: Optional[int] = None
    valid: Optional[bool] = None
    finding_count: Optional[int] = None
    overflow_count: Optional[int] = None
    overlap_count: Optional[int] = None
    empty_page_count: Optional[int] = None

    def replace(self, **changes: Any) -> "EnvelopeFields":
        data = asdict(self)
        data.update(changes)
        return EnvelopeFields(**data)

    def field_tuple(self) -> tuple[Any, ...]:
        return (
            self.command,
            self.exit,
            self.identical,
            self.has_signal,
            self.reproduced,
            self.page_count_a,
            self.page_count_b,
            self.page_count_mismatch,
            self.struct_status,
            self.struct_node,
            self.page_count_node,
            self.verify_identical,
            self.verify_diff_count,
            self.diff_count,
            self.fail_count,
            self.pass_count,
            self.verdict,
            self.regression,
            self.status,
            self.clean,
            self.signal_count,
            self.valid,
            self.finding_count,
            self.overflow_count,
            self.overlap_count,
            self.empty_page_count,
        )

    def pages_equal(self) -> Optional[bool]:
        if self.page_count_a is None or self.page_count_b is None:
            return None
        return self.page_count_a == self.page_count_b

    def same_struct_node(self) -> bool:
        left = self.struct_node or ""
        right = self.page_count_node or ""
        return left == right

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "EnvelopeFields":
        return cls(
            command=str(row.get("command") or "info"),
            exit=int(row.get("exit", 0) or 0),
            identical=parse_optional_bool(row.get("identical")),
            has_signal=parse_optional_bool(row.get("has_signal", row.get("hasSignal"))),
            reproduced=parse_optional_bool(row.get("reproduced")),
            page_count_a=parse_optional_int(row.get("page_count_a", row.get("pageCountA"))),
            page_count_b=parse_optional_int(row.get("page_count_b", row.get("pageCountB"))),
            page_count_mismatch=parse_optional_bool(
                row.get("page_count_mismatch", row.get("pageCountMismatch"))
            ),
            struct_status=parse_optional_str(row.get("struct_status", row.get("structStatus"))),
            struct_node=parse_optional_str(row.get("struct_node", row.get("structNode"))),
            page_count_node=parse_optional_str(
                row.get("page_count_node", row.get("pageCountNode"))
            ),
            verify_identical=parse_optional_bool(
                row.get("verify_identical", row.get("verifyIdentical"))
            ),
            verify_diff_count=parse_optional_int(
                row.get("verify_diff_count", row.get("verifyDiffCount"))
            ),
            diff_count=parse_optional_int(row.get("diff_count", row.get("diffCount"))),
            fail_count=parse_optional_int(row.get("fail_count", row.get("failCount"))),
            pass_count=parse_optional_int(row.get("pass_count", row.get("passCount"))),
            verdict=parse_optional_str(row.get("verdict")),
            regression=parse_optional_bool(row.get("regression")),
            status=parse_optional_str(row.get("status")),
            clean=parse_optional_bool(row.get("clean")),
            signal_count=parse_optional_int(row.get("signal_count", row.get("signalCount"))),
            valid=parse_optional_bool(row.get("valid")),
            finding_count=parse_optional_int(row.get("finding_count", row.get("findingCount"))),
            overflow_count=parse_optional_int(
                row.get("overflow_count", row.get("overflowCount"))
            ),
            overlap_count=parse_optional_int(row.get("overlap_count", row.get("overlapCount"))),
            empty_page_count=parse_optional_int(
                row.get("empty_page_count", row.get("emptyPageCount"))
            ),
        )


@dataclass(frozen=True)
class Decision:
    verdict: str
    contradiction_id: str
    contradictions: tuple[str, ...]
    success_tokens: tuple[str, ...]
    fail_tokens: tuple[str, ...]

    @property
    def abstained(self) -> bool:
        return self.verdict == VERDICT_ABSTAIN


@dataclass(frozen=True)
class AbstainCase:
    case_id: str
    fields: EnvelopeFields
    expected: str
    contradiction_id: str
    success_tokens: str
    fail_tokens: str
    sample: str
    source_format: str
    agency: str
    doc_kind: str
    year: str
    family: str

    def identity_key(self) -> tuple[Any, ...]:
        return self.fields.field_tuple()

    def to_row(self) -> dict[str, str]:
        f = self.fields
        return {
            "case_id": self.case_id,
            "command": f.command,
            "exit": str(f.exit),
            "identical": bool_cell(f.identical),
            "has_signal": bool_cell(f.has_signal),
            "reproduced": bool_cell(f.reproduced),
            "page_count_a": int_cell(f.page_count_a),
            "page_count_b": int_cell(f.page_count_b),
            "page_count_mismatch": bool_cell(f.page_count_mismatch),
            "struct_status": str_cell(f.struct_status),
            "struct_node": str_cell(f.struct_node),
            "page_count_node": str_cell(f.page_count_node),
            "verify_identical": bool_cell(f.verify_identical),
            "verify_diff_count": int_cell(f.verify_diff_count),
            "diff_count": int_cell(f.diff_count),
            "fail_count": int_cell(f.fail_count),
            "pass_count": int_cell(f.pass_count),
            "verdict": str_cell(f.verdict),
            "regression": bool_cell(f.regression),
            "status": str_cell(f.status),
            "clean": bool_cell(f.clean),
            "signal_count": int_cell(f.signal_count),
            "valid": bool_cell(f.valid),
            "finding_count": int_cell(f.finding_count),
            "overflow_count": int_cell(f.overflow_count),
            "overlap_count": int_cell(f.overlap_count),
            "empty_page_count": int_cell(f.empty_page_count),
            "expected": self.expected,
            "contradiction_id": self.contradiction_id,
            "success_tokens": self.success_tokens,
            "fail_tokens": self.fail_tokens,
            "sample": self.sample,
            "source_format": self.source_format,
            "agency": self.agency,
            "doc_kind": self.doc_kind,
            "year": self.year,
            "family": self.family,
        }

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "AbstainCase":
        return cls(
            case_id=str(row["case_id"]),
            fields=EnvelopeFields.from_mapping(row),
            expected=str(row["expected"]),
            contradiction_id=str(row.get("contradiction_id", "")),
            success_tokens=str(row.get("success_tokens", "")),
            fail_tokens=str(row.get("fail_tokens", "")),
            sample=str(row.get("sample", "")),
            source_format=str(row.get("source_format", "")),
            agency=str(row.get("agency", "")),
            doc_kind=str(row.get("doc_kind", "")),
            year=str(row.get("year", "")),
            family=str(row.get("family", "")),
        )
