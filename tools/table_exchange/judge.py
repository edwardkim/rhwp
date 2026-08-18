"""csv-to-table / table-to-csv / export-tables envelope judge.

Mirrors the documented CLI contract. Collects *all* invalid reasons
before refusing a write. Does not invent merge writers or table resize.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from . import SCHEMA_VERSION, SKILL
from .catalog import (
    COL_MISMATCH_MESSAGE,
    CONTROL_CHARS,
    CONTROL_MESSAGE,
    COVERED_MESSAGE,
    CSV_PARSE_MESSAGE,
    OUTPUT_FORMATS,
    ROW_MISMATCH_MESSAGE,
    UNTRUSTED_FIELDS,
    output_format_for,
)
from .csv_codec import CsvParseResult, read_csv, write_csv
from .occupancy import Occupancy


@dataclass
class InvalidItem:
    reason: str
    message: str
    row: int | None = None
    col: int | None = None
    expected: int | None = None
    actual: int | None = None
    anchor_row: int | None = None
    anchor_col: int | None = None

    def to_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {"reason": self.reason, "message": self.message}
        if self.row is not None:
            data["row"] = self.row
        if self.col is not None:
            data["col"] = self.col
        if self.expected is not None:
            data["expected"] = self.expected
        if self.actual is not None:
            data["actual"] = self.actual
        if self.anchor_row is not None:
            data["anchorRow"] = self.anchor_row
        if self.anchor_col is not None:
            data["anchorCol"] = self.anchor_col
        return data


@dataclass
class ChangedItem:
    row: int
    col: int
    old_text: str
    new_text: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "row": self.row,
            "col": self.col,
            "oldText": self.old_text,
            "newText": self.new_text,
        }


@dataclass
class Judgment:
    parse: CsvParseResult
    invalid: list[InvalidItem] = field(default_factory=list)
    changed: list[ChangedItem] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return self.parse.ok and not self.invalid

    @property
    def exit_code(self) -> int:
        if not self.parse.ok or self.invalid:
            return 2
        return 0

    @property
    def writes(self) -> bool:
        return self.ok


def collect_invalid(
    occupancy: Occupancy,
    records: tuple[tuple[str, ...], ...] | None,
    parse: CsvParseResult,
    table_index: int,
) -> list[InvalidItem]:
    items: list[InvalidItem] = []
    if not parse.ok:
        items.append(InvalidItem("csvParse", parse.message or CSV_PARSE_MESSAGE))
        return items
    assert records is not None
    if len(records) != occupancy.rows:
        items.append(
            InvalidItem(
                "rowCountMismatch",
                ROW_MISMATCH_MESSAGE.format(
                    actual=len(records), table=table_index, expected=occupancy.rows
                ),
                expected=occupancy.rows,
                actual=len(records),
            )
        )
    for row_idx, row in enumerate(records):
        if len(row) != occupancy.cols:
            items.append(
                InvalidItem(
                    "colCountMismatch",
                    COL_MISMATCH_MESSAGE.format(
                        row=row_idx,
                        actual=len(row),
                        table=table_index,
                        expected=occupancy.cols,
                    ),
                    row=row_idx,
                    expected=occupancy.cols,
                    actual=len(row),
                )
            )
    # Covered / control only make sense on in-bound cells.
    for row_idx, row in enumerate(records):
        if row_idx >= occupancy.rows:
            continue
        for col_idx, value in enumerate(row):
            if col_idx >= occupancy.cols:
                continue
            cell = occupancy.get(row_idx, col_idx)
            if cell.kind == "covered" and value != "":
                items.append(
                    InvalidItem(
                        "coveredCellNotEmpty",
                        COVERED_MESSAGE.format(
                            row=row_idx,
                            col=col_idx,
                            anchor_row=cell.anchor_row,
                            anchor_col=cell.anchor_col,
                        ),
                        row=row_idx,
                        col=col_idx,
                        anchor_row=cell.anchor_row,
                        anchor_col=cell.anchor_col,
                    )
                )
            if any(ch in value for ch in CONTROL_CHARS):
                items.append(
                    InvalidItem(
                        "controlCharacter",
                        CONTROL_MESSAGE,
                        row=row_idx,
                        col=col_idx,
                    )
                )
    return items


def collect_changed(
    occupancy: Occupancy, records: tuple[tuple[str, ...], ...]
) -> list[ChangedItem]:
    old = occupancy.grid_texts()
    changed: list[ChangedItem] = []
    for r, row in enumerate(records):
        for c, value in enumerate(row):
            if occupancy.get(r, c).kind == "covered":
                continue
            if old[r][c] != value:
                changed.append(ChangedItem(r, c, old[r][c], value))
    return changed


def judge_csv_to_table(
    occupancy: Occupancy,
    csv_text: str,
    table_index: int,
) -> Judgment:
    parse = read_csv(csv_text)
    records = parse.records if parse.ok else None
    invalid = collect_invalid(occupancy, records, parse, table_index)
    changed: list[ChangedItem] = []
    if parse.ok and not invalid and records is not None:
        changed = collect_changed(occupancy, records)
    return Judgment(parse=parse, invalid=invalid, changed=changed)


def csv_to_table_envelope(
    *,
    source: str,
    table_index: int,
    occupancy: Occupancy,
    judgment: Judgment,
    csv_name: str,
    mode: str,
    output: str | None,
    verify_diff_count: int | None = None,
) -> dict[str, Any]:
    dry_run = mode == "dry-run"
    do_verify = mode == "verify"
    ok = judgment.ok
    if dry_run:
        changed_pages: list[int] | None = None
        output_value: str | None = None
        verify: dict[str, Any] | None = None
        exit_code = 2 if not ok else 0
        output_kept = False
    elif not ok:
        changed_pages = None
        output_value = None
        verify = None
        exit_code = 2
        output_kept = False
    else:
        changed_pages = [0] if judgment.changed else []
        output_value = output
        exit_code = 0
        output_kept = True
        if do_verify:
            diff = 0 if verify_diff_count is None else verify_diff_count
            verify = {"diffCount": diff, "identical": diff == 0}
            if diff != 0:
                exit_code = 3
        else:
            verify = None

    envelope: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "source": source,
        "table": table_index,
        "rowCount": occupancy.rows,
        "colCount": occupancy.cols,
        "csv": csv_name,
        "dryRun": dry_run,
        "changed": [item.to_dict() for item in judgment.changed] if ok else [],
        "changedCount": len(judgment.changed) if ok else 0,
        "changedPages": changed_pages,
        "invalid": [item.to_dict() for item in judgment.invalid],
        "output": output_value,
        "verify": verify,
        "untrustedContent": True if ok and judgment.changed else False,
        "untrustedFields": list(UNTRUSTED_FIELDS["csv-to-table"]) if ok and judgment.changed else [],
    }
    if output_value and not dry_run and ok:
        envelope["outputFormat"] = output_format_for(output_value)
    envelope["_skillMeta"] = {
        "skill": SKILL,
        "command": "csv-to-table",
        "exit": exit_code,
        "outputKept": output_kept,
        "mode": mode,
        "writes": bool(ok and not dry_run),
    }
    return envelope


def export_tables_envelope(
    *,
    source: str,
    tables: list[dict[str, Any]],
    exit_code: int = 0,
    stdout_bytes: int | None = None,
) -> dict[str, Any]:
    envelope: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "source": source,
        "tableCount": len(tables),
        "tables": tables,
        "untrustedContent": True,
        "untrustedFields": list(UNTRUSTED_FIELDS["export-tables"]),
        "_skillMeta": {
            "skill": SKILL,
            "command": "export-tables",
            "exit": exit_code,
        },
    }
    if stdout_bytes is not None:
        envelope["_skillMeta"]["stdoutBytes"] = stdout_bytes
    return envelope


def table_to_csv_envelope(
    *,
    source: str,
    tables: list[dict[str, Any]],
    bom: bool,
    output: str | None,
    output_is_dir: bool,
    exit_code: int = 0,
    stdout_bytes: int | None = None,
) -> dict[str, Any]:
    envelope: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "source": source,
        "tableCount": len(tables),
        "tables": tables,
        "bom": bom,
        "untrustedContent": True,
        "untrustedFields": list(UNTRUSTED_FIELDS["table-to-csv"]),
        "_skillMeta": {
            "skill": SKILL,
            "command": "table-to-csv",
            "exit": exit_code,
            "envelopeCsvStartsWithBom": False,
        },
    }
    if output is not None:
        envelope["output"] = output
        envelope["outputFormat"] = "csv" if not output_is_dir else "dir"
    if bom:
        envelope["_filePrefix"] = [0xEF, 0xBB, 0xBF]
        envelope["_envelopeCsvStartsWithBom"] = False
    if stdout_bytes is not None:
        envelope["_skillMeta"]["stdoutBytes"] = stdout_bytes
    return envelope


def table_csv_entry(occupancy: Occupancy, index: int, output: str | None = None) -> dict[str, Any]:
    csv_text = write_csv(occupancy.grid_texts())
    entry: dict[str, Any] = {
        "index": index,
        "rowCount": occupancy.rows,
        "colCount": occupancy.cols,
        "csv": csv_text,
    }
    if output is not None:
        entry["output"] = output
    return entry


def export_table_entry(
    occupancy: Occupancy,
    index: int,
    *,
    section: int = 0,
    paragraph: int = 0,
    control: int = 0,
    container: list[dict[str, Any]] | None = None,
    include_full_cells: bool = True,
) -> dict[str, Any]:
    entry: dict[str, Any] = {
        "index": index,
        "section": section,
        "paragraph": paragraph,
        "control": control,
        "rows": occupancy.rows,
        "cols": occupancy.cols,
        "cellCount": occupancy.cell_count,
    }
    if container:
        entry["containerPath"] = container
    if include_full_cells:
        entry["cells"] = [anchor.to_cell_dict() for anchor in occupancy.anchors]
    else:
        entry["cells"] = [
            anchor.to_cell_dict() for anchor in occupancy.anchors if anchor.merged
        ]
        entry["_mergeNote"] = (
            f"전체 {occupancy.cell_count} 앵커. 여기선 병합 앵커만. "
            f"면적 합 {occupancy.area_sum} <= {occupancy.rows * occupancy.cols}."
        )
    return entry


def silent_usage(command: str, argv: list[str]) -> dict[str, Any]:
    return {
        "schemaVersion": SCHEMA_VERSION,
        "stdout": "",
        "_skillMeta": {
            "skill": SKILL,
            "command": command,
            "exit": 2,
            "stdoutBytes": 0,
            "argv": argv,
            "branch": "usage-silent",
        },
    }


def silent_runtime(command: str, argv: list[str], message: str) -> dict[str, Any]:
    return {
        "schemaVersion": SCHEMA_VERSION,
        "stdout": "",
        "stderrContains": message,
        "_skillMeta": {
            "skill": SKILL,
            "command": command,
            "exit": 1,
            "stdoutBytes": 0,
            "argv": argv,
            "branch": "runtime-silent",
        },
    }


# Keep OUTPUT_FORMATS imported for catalog tests.
_ = OUTPUT_FORMATS
