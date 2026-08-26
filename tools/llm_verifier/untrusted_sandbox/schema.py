"""Wire types for V-nonce untrusted-content sandbox cases.

Corpus columns are the issue contract:

    (excerpt, nonce, slot, leaked_into_criteria, expected_block)

plus tracing fields so each row stays a distinct document-derived placement,
not comment padding. Document text is data. It is never a verification
criterion and never an instruction.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any, Mapping

CASE_COLUMNS: tuple[str, ...] = (
    "case_id",
    "excerpt",
    "nonce",
    "slot",
    "leaked_into_criteria",
    "expected_block",
    "field_path",
    "command",
    "source_label",
    "source_label_kind",
    "wrap_state",
    "nonce_kind",
    "untrusted_content",
    "fail_kinds",
    "sample",
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


def parse_fail_kinds(raw: str) -> tuple[str, ...]:
    text = (raw or "").strip()
    if not text:
        return ()
    return tuple(part for part in text.split("|") if part)


@dataclass(frozen=True)
class SandboxCase:
    case_id: str
    excerpt: str
    nonce: str
    slot: str
    leaked_into_criteria: bool
    expected_block: bool
    field_path: str
    command: str
    source_label: str
    source_label_kind: str
    wrap_state: str
    nonce_kind: str
    untrusted_content: bool
    fail_kinds: str
    sample: str

    def contract_tuple(self) -> tuple[str, str, str, bool, bool]:
        return (
            self.excerpt,
            self.nonce,
            self.slot,
            self.leaked_into_criteria,
            self.expected_block,
        )

    def identity_key(self) -> tuple[Any, ...]:
        return (
            self.excerpt,
            self.nonce,
            self.slot,
            self.leaked_into_criteria,
            self.expected_block,
            self.field_path,
            self.command,
            self.source_label,
            self.wrap_state,
            self.nonce_kind,
            self.sample,
        )

    def to_row(self) -> dict[str, str]:
        data = asdict(self)
        data["leaked_into_criteria"] = bool_cell(self.leaked_into_criteria)
        data["expected_block"] = bool_cell(self.expected_block)
        data["untrusted_content"] = bool_cell(self.untrusted_content)
        return data

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "SandboxCase":
        return cls(
            case_id=str(row["case_id"]),
            excerpt=str(row["excerpt"]),
            nonce=str(row["nonce"]),
            slot=str(row["slot"]),
            leaked_into_criteria=parse_bool(row["leaked_into_criteria"]),
            expected_block=parse_bool(row["expected_block"]),
            field_path=str(row["field_path"]),
            command=str(row["command"]),
            source_label=str(row["source_label"]),
            source_label_kind=str(row["source_label_kind"]),
            wrap_state=str(row["wrap_state"]),
            nonce_kind=str(row["nonce_kind"]),
            untrusted_content=parse_bool(row["untrusted_content"]),
            fail_kinds=str(row.get("fail_kinds", "")),
            sample=str(row["sample"]),
        )
