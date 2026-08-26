"""Hangul-year token grammar consumed from oracle_public contracts.

``oracle_resolver`` pair years are 2018/2020/2022/2024.
``multiver_index`` also records 2010 as a directory-default year.
A ``+`` join means page counts agree across years.
A ``!`` join means ``multiver_index`` reported page_count_disagree.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

# oracle_public/oracle_resolver.py HANCOM_YEARS + schema enum
RESOLVER_HANCOM_YEARS: frozenset[str] = frozenset({"2018", "2020", "2022", "2024"})
# oracle_public/multiver_index.py HANGUL_YEARS (includes 2010)
ALLOWED_HANCOM_YEARS: frozenset[str] = frozenset({"2010", "2018", "2020", "2022", "2024"})

NONE_TOKENS: frozenset[str] = frozenset({"", "none", "-", "null", "unmatched"})
UNKNOWN_TOKENS: frozenset[str] = frozenset({"unknown", "unparsed", "unversioned", "?"})

VersionKind = Literal["none", "unknown", "invalid", "agree", "disagree"]


@dataclass(frozen=True)
class ParsedVersions:
    raw: str
    kind: VersionKind
    years: tuple[str, ...]
    agree: bool
    out_of_contract: tuple[str, ...]

    @property
    def has_official_year(self) -> bool:
        return self.kind in {"agree", "disagree"} and bool(self.years)

    @property
    def canonical(self) -> str:
        if self.kind == "none":
            return "none"
        if self.kind == "unknown":
            return "unknown"
        if self.kind == "invalid":
            return "invalid:" + ",".join(self.out_of_contract)
        joiner = "+" if self.agree else "!"
        return joiner.join(self.years)


def _split_years(raw: str) -> tuple[list[str], bool | None]:
    """Split a versions token. None agreement means no joiner was used."""
    text = raw.strip()
    if "!" in text and "+" in text:
        return [part for part in text.replace("+", "!").split("!") if part], False
    if "!" in text:
        return [part for part in text.split("!") if part], False
    if "+" in text:
        return [part for part in text.split("+") if part], True
    if "," in text:
        return [part for part in text.split(",") if part], True
    if text:
        return [text], True
    return [], None


def parse_versions(raw: str | None) -> ParsedVersions:
    token = "" if raw is None else str(raw).strip()
    folded = token.lower()
    if folded in NONE_TOKENS:
        return ParsedVersions(raw=token, kind="none", years=(), agree=True, out_of_contract=())
    if folded in UNKNOWN_TOKENS:
        return ParsedVersions(
            raw=token, kind="unknown", years=(), agree=True, out_of_contract=()
        )

    parts, agreement = _split_years(token)
    years: list[str] = []
    invalid: list[str] = []
    for part in parts:
        year = part.strip()
        if not year:
            continue
        if year in ALLOWED_HANCOM_YEARS:
            if year not in years:
                years.append(year)
        else:
            if year not in invalid:
                invalid.append(year)

    if invalid:
        return ParsedVersions(
            raw=token,
            kind="invalid",
            years=tuple(years),
            agree=agreement is not False,
            out_of_contract=tuple(invalid),
        )
    if not years:
        return ParsedVersions(
            raw=token, kind="unknown", years=(), agree=True, out_of_contract=()
        )
    agree = True if agreement is None else agreement
    if len(years) == 1:
        agree = True
    return ParsedVersions(
        raw=token,
        kind="disagree" if not agree else "agree",
        years=tuple(years),
        agree=agree,
        out_of_contract=(),
    )


def iter_agree_encodings() -> tuple[str, ...]:
    singles = tuple(sorted(ALLOWED_HANCOM_YEARS))
    pairs = (
        "2010+2018",
        "2010+2020",
        "2010+2022",
        "2010+2024",
        "2018+2020",
        "2018+2022",
        "2018+2024",
        "2020+2022",
        "2020+2024",
        "2022+2024",
        "2018+2020+2022",
        "2018+2020+2024",
        "2018+2022+2024",
        "2020+2022+2024",
        "2010+2018+2020+2022+2024",
        "2018+2020+2022+2024",
    )
    return singles + pairs


def iter_disagree_encodings() -> tuple[str, ...]:
    return (
        "2010!2018",
        "2010!2020",
        "2010!2024",
        "2018!2020",
        "2018!2022",
        "2018!2024",
        "2020!2022",
        "2020!2024",
        "2022!2024",
        "2018!2020!2022",
        "2018!2020!2024",
        "2020!2022!2024",
        "2010!2020!2024",
        "2018!2020!2022!2024",
    )


def iter_invalid_encodings() -> tuple[str, ...]:
    return (
        "2016",
        "2025",
        "2026",
        "1998",
        "hancom-next",
        "office2019",
        "2018+2025",
        "2020!2030",
    )


def iter_unknown_encodings() -> tuple[str, ...]:
    return ("unknown", "unparsed", "unversioned", "?")


def iter_none_encodings() -> tuple[str, ...]:
    return ("none", "", "-", "unmatched")
