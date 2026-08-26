"""Decision-case wire types for the V-oracle selection tree."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any, Mapping

CASE_COLUMNS: tuple[str, ...] = (
    "case_id",
    "has_hangul_pdf",
    "versions",
    "page_count_match",
    "render_self_pass",
    "cheap_ok",
    "expected_verdict_class",
    "sample",
    "source_format",
    "oracle_root",
    "variant",
    "rhwp_pages",
    "pdf_pages",
    "page_smoke_verdict",
    "cheap_reason",
    "honest_claim",
    "allowed_tools",
    "blocked_tools",
    "contract_source",
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
    has_hangul_pdf: bool
    versions: str
    page_count_match: bool
    render_self_pass: bool
    cheap_ok: bool
    expected_verdict_class: str
    sample: str
    source_format: str
    oracle_root: str
    variant: str
    rhwp_pages: int
    pdf_pages: int
    page_smoke_verdict: str
    cheap_reason: str
    honest_claim: str
    allowed_tools: str
    blocked_tools: str
    contract_source: str

    def axis_tuple(
        self,
    ) -> tuple[bool, str, bool, bool, bool, str]:
        return (
            self.has_hangul_pdf,
            self.versions,
            self.page_count_match,
            self.render_self_pass,
            self.cheap_ok,
            self.expected_verdict_class,
        )

    def identity_key(self) -> tuple[Any, ...]:
        return (
            self.case_id,
            self.has_hangul_pdf,
            self.versions,
            self.page_count_match,
            self.render_self_pass,
            self.cheap_ok,
            self.sample,
            self.source_format,
            self.oracle_root,
            self.variant,
            self.rhwp_pages,
            self.pdf_pages,
            self.page_smoke_verdict,
        )

    def to_row(self) -> dict[str, str]:
        data = asdict(self)
        data["has_hangul_pdf"] = bool_cell(self.has_hangul_pdf)
        data["page_count_match"] = bool_cell(self.page_count_match)
        data["render_self_pass"] = bool_cell(self.render_self_pass)
        data["cheap_ok"] = bool_cell(self.cheap_ok)
        data["rhwp_pages"] = str(self.rhwp_pages)
        data["pdf_pages"] = str(self.pdf_pages)
        return data

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "DecisionCase":
        return cls(
            case_id=str(row["case_id"]),
            has_hangul_pdf=parse_bool(row["has_hangul_pdf"]),
            versions=str(row["versions"]),
            page_count_match=parse_bool(row["page_count_match"]),
            render_self_pass=parse_bool(row["render_self_pass"]),
            cheap_ok=parse_bool(row["cheap_ok"]),
            expected_verdict_class=str(row["expected_verdict_class"]),
            sample=str(row.get("sample", "")),
            source_format=str(row.get("source_format", "")),
            oracle_root=str(row.get("oracle_root", "")),
            variant=str(row.get("variant", "")),
            rhwp_pages=int(row.get("rhwp_pages", 0) or 0),
            pdf_pages=int(row.get("pdf_pages", 0) or 0),
            page_smoke_verdict=str(row.get("page_smoke_verdict", "")),
            cheap_reason=str(row.get("cheap_reason", "")),
            honest_claim=str(row.get("honest_claim", "")),
            allowed_tools=str(row.get("allowed_tools", "")),
            blocked_tools=str(row.get("blocked_tools", "")),
            contract_source=str(row.get("contract_source", "")),
        )
