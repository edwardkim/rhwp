"""Merge occupancy for export-tables / csv-to-table.

export-tables emits *anchors only*. Covered cells are the grid minus that
anchor set. table-to-csv fills those holes with empty strings so the CSV
stays a rectangle. csv-to-table rejects a non-empty value in a covered
cell (`coveredCellNotEmpty`). This module computes that matrix. It does
not split or merge cells.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Iterable


@dataclass(frozen=True)
class Anchor:
    row: int
    col: int
    row_span: int = 1
    col_span: int = 1
    text: str = ""
    is_header: bool = False
    nested: bool = False

    @property
    def merged(self) -> bool:
        return self.row_span > 1 or self.col_span > 1

    @property
    def area(self) -> int:
        return self.row_span * self.col_span

    def covers(self, row: int, col: int) -> bool:
        return (
            self.row <= row < self.row + self.row_span
            and self.col <= col < self.col + self.col_span
        )

    def to_cell_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {
            "row": self.row,
            "col": self.col,
            "rowSpan": self.row_span,
            "colSpan": self.col_span,
            "isHeader": self.is_header,
            "text": self.text,
        }
        if self.nested:
            data["nested"] = True
        return data


@dataclass
class Occupied:
    kind: str  # anchor | covered | empty
    row: int
    col: int
    anchor_row: int | None
    anchor_col: int | None
    row_span: int
    col_span: int
    text: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "kind": self.kind,
            "row": self.row,
            "col": self.col,
            "anchorRow": self.anchor_row,
            "anchorCol": self.anchor_col,
            "rowSpan": self.row_span,
            "colSpan": self.col_span,
            "text": self.text,
        }


@dataclass
class Occupancy:
    rows: int
    cols: int
    anchors: list[Anchor]
    cells: list[list[Occupied]]

    @property
    def cell_count(self) -> int:
        return len(self.anchors)

    @property
    def covered_count(self) -> int:
        return sum(1 for row in self.cells for cell in row if cell.kind == "covered")

    @property
    def merged_anchor_count(self) -> int:
        return sum(1 for anchor in self.anchors if anchor.merged)

    @property
    def area_sum(self) -> int:
        return sum(anchor.area for anchor in self.anchors)

    def get(self, row: int, col: int) -> Occupied:
        return self.cells[row][col]

    def is_covered(self, row: int, col: int) -> bool:
        return self.cells[row][col].kind == "covered"

    def covered_coords(self) -> list[tuple[int, int]]:
        return [
            (cell.row, cell.col)
            for row in self.cells
            for cell in row
            if cell.kind == "covered"
        ]

    def anchor_coords(self) -> list[tuple[int, int]]:
        return [(anchor.row, anchor.col) for anchor in self.anchors]

    def grid_texts(self) -> list[list[str]]:
        """table-to-csv rectangle: covered cells become empty strings."""
        out: list[list[str]] = []
        for r in range(self.rows):
            row: list[str] = []
            for c in range(self.cols):
                cell = self.cells[r][c]
                row.append(cell.text if cell.kind == "anchor" else "")
            out.append(row)
        return out

    def to_public_dict(self, *, slim: bool = True) -> dict[str, Any]:
        merged = [anchor.to_cell_dict() for anchor in self.anchors if anchor.merged]
        covered = [
            {
                "row": row,
                "col": col,
                "anchorRow": self.cells[row][col].anchor_row,
                "anchorCol": self.cells[row][col].anchor_col,
            }
            for row, col in self.covered_coords()
        ]
        data = {
            "rows": self.rows,
            "cols": self.cols,
            "cellCount": self.cell_count,
            "coveredCount": self.covered_count,
            "mergedAnchorCount": self.merged_anchor_count,
            "areaSum": self.area_sum,
            "gridArea": self.rows * self.cols,
            "areaFits": self.area_sum <= self.rows * self.cols,
            "mergedAnchors": merged,
            "covered": covered,
        }
        if not slim:
            data["anchors"] = [anchor.to_cell_dict() for anchor in self.anchors]
        return data


class OccupancyError(ValueError):
    pass


def build_occupancy(rows: int, cols: int, anchors: Iterable[Anchor]) -> Occupancy:
    if rows < 1 or cols < 1:
        raise OccupancyError(f"invalid grid {rows}x{cols}")
    cells: list[list[Occupied | None]] = [[None] * cols for _ in range(rows)]
    ordered = sorted(anchors, key=lambda a: (a.row, a.col))
    for anchor in ordered:
        if anchor.row_span < 1 or anchor.col_span < 1:
            raise OccupancyError(f"bad span at ({anchor.row},{anchor.col})")
        if (
            anchor.row < 0
            or anchor.col < 0
            or anchor.row + anchor.row_span > rows
            or anchor.col + anchor.col_span > cols
        ):
            raise OccupancyError(
                f"span ({anchor.row},{anchor.col}) {anchor.row_span}x{anchor.col_span} "
                f"escapes {rows}x{cols}"
            )
        for r in range(anchor.row, anchor.row + anchor.row_span):
            for c in range(anchor.col, anchor.col + anchor.col_span):
                if cells[r][c] is not None:
                    raise OccupancyError(
                        f"overlap at ({r},{c}) with existing {cells[r][c]}"
                    )
                if r == anchor.row and c == anchor.col:
                    cells[r][c] = Occupied(
                        "anchor",
                        r,
                        c,
                        anchor.row,
                        anchor.col,
                        anchor.row_span,
                        anchor.col_span,
                        anchor.text,
                    )
                else:
                    cells[r][c] = Occupied(
                        "covered",
                        r,
                        c,
                        anchor.row,
                        anchor.col,
                        0,
                        0,
                        "",
                    )
    filled_anchors = list(ordered)
    for r in range(rows):
        for c in range(cols):
            if cells[r][c] is None:
                cells[r][c] = Occupied("anchor", r, c, r, c, 1, 1, "")
                filled_anchors.append(Anchor(r, c, 1, 1, "", False))
    filled_anchors.sort(key=lambda a: (a.row, a.col))
    concrete: list[list[Occupied]] = [[cell for cell in row] for row in cells]  # type: ignore[misc]
    return Occupancy(rows, cols, filled_anchors, concrete)


def anchors_from_spans(
    rows: int,
    cols: int,
    spans: Iterable[tuple[int, int, int, int]],
    texts: dict[tuple[int, int], str] | None = None,
    headers: Iterable[tuple[int, int]] | None = None,
) -> list[Anchor]:
    header_set = set(headers or ())
    text_map = texts or {}
    claimed = set()
    anchors: list[Anchor] = []
    for row, col, row_span, col_span in spans:
        for r in range(row, row + row_span):
            for c in range(col, col + col_span):
                claimed.add((r, c))
        anchors.append(
            Anchor(
                row,
                col,
                row_span,
                col_span,
                text_map.get((row, col), ""),
                (row, col) in header_set,
            )
        )
    for r in range(rows):
        for c in range(cols):
            if (r, c) in claimed:
                continue
            anchors.append(
                Anchor(r, c, 1, 1, text_map.get((r, c), ""), (r, c) in header_set)
            )
    anchors.sort(key=lambda a: (a.row, a.col))
    return anchors


def csv_roundtrip_for(occupancy: Occupancy, container: str | None, nested: bool) -> str:
    if container:
        return "forbidden"
    if nested:
        return "outer-only"
    if occupancy.rows == 1 and occupancy.cols == 1:
        return "skip"
    if occupancy.merged_anchor_count > 0:
        return "extract-only"
    return "allowed"
