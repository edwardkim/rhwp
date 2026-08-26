"""RFC 4180 reader/writer independent of the rhwp binary.

`tests/table_csv_contract.rs` keeps its own reader so a shared bug cannot
hide. This module is that same idea in Python: fixtures and tests parse
CSV without calling `rhwp table-to-csv`.
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class CsvParseResult:
    ok: bool
    records: tuple[tuple[str, ...], ...]
    message: str | None = None


def read_csv(text: str) -> CsvParseResult:
    """Parse RFC 4180. Unclosed quotes become csvParse, not a panic."""
    if text.startswith("\ufeff"):
        text = text[1:]
    chars = list(text)
    records: list[list[str]] = []
    record: list[str] = []
    field: list[str] = []
    quoted = False
    i = 0
    while i < len(chars):
        ch = chars[i]
        if quoted:
            if ch == '"':
                if i + 1 < len(chars) and chars[i + 1] == '"':
                    field.append('"')
                    i += 1
                else:
                    quoted = False
            else:
                field.append(ch)
        elif ch == '"' and not field:
            quoted = True
        elif ch == ",":
            record.append("".join(field))
            field = []
        elif ch == "\n":
            record.append("".join(field))
            field = []
            records.append(record)
            record = []
        elif ch != "\r":
            field.append(ch)
        i += 1
    if quoted:
        return CsvParseResult(False, (), "CSV 를 읽지 못했습니다 — 닫히지 않은 따옴표.")
    if field or record:
        record.append("".join(field))
        records.append(record)
    return CsvParseResult(True, tuple(tuple(row) for row in records))


def needs_quote(value: str) -> bool:
    return any(ch in value for ch in (',', '"', '\r', '\n'))


def quote_field(value: str) -> str:
    if needs_quote(value):
        return '"' + value.replace('"', '""') + '"'
    return value


def write_csv(records: list[list[str]] | tuple[tuple[str, ...], ...]) -> str:
    lines: list[str] = []
    for record in records:
        lines.append(",".join(quote_field(field) for field in record))
    if not lines:
        return ""
    return "\r\n".join(lines) + "\r\n"


def rectangle(records: list[list[str]] | tuple[tuple[str, ...], ...]) -> bool:
    if not records:
        return True
    width = len(records[0])
    return all(len(row) == width for row in records)


def row_widths(records: list[list[str]] | tuple[tuple[str, ...], ...]) -> list[int]:
    return [len(row) for row in records]
