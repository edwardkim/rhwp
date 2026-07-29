//! OTSL table serialisation for the DocLang writer.
//!
//! Converts an [`ir::table::Table`] into a DocLang `<table>` element whose
//! children are OTSL (Open Table Structure Language) tokens.  The IR stores
//! only *anchor* cells (top-left of each merged region) with explicit
//! `col_span`/`row_span`; this writer reconstructs the full rows×cols grid and
//! emits one token per grid position:
//!
//! | token      | meaning                                                    |
//! |------------|------------------------------------------------------------|
//! | `<ched/>`  | header cell (anchor, `is_header == true`)                  |
//! | `<fcel/>`  | full cell — anchor with non-empty content                  |
//! | `<ecel/>`  | empty cell — anchor with empty content, or a hole in grid  |
//! | `<lcel/>`  | left-merge continuation (covered horizontally by a span)   |
//! | `<ucel/>`  | up-merge continuation (covered vertically by a span)       |
//! | `<xcel/>`  | cross continuation (covered both horizontally and vertically) |
//! | `<nl/>`    | row terminator                                             |
//!
//! Anchor tokens are immediately followed by the cell's serialised block
//! content (recursing into the block writer, which transparently supports
//! nested tables).  Per the DocLang element-head order, an optional
//! `<caption>` is emitted before the OTSL token stream, inside `<table>`.
//!
//! ## Defensive behaviour
//!
//! Malformed input (anchors out of bounds, or overlapping spans) must never
//! panic.  Out-of-bounds anchors are skipped and span extents are clamped to
//! the declared grid; the first cell to claim a grid position wins, and any
//! conflict is recorded as a [`LossKind::Other`] entry rather than crashing.

use crate::doclang::ir::prov::LocationMap;
use crate::doclang::ir::table::Table;
use crate::doclang::loss::{LossEntry, LossKind, LossReport};
use crate::doclang::options::ConvertOptions;

use super::write_blocks;
use super::write_inlines;
use super::{emit_location, resolve_loc};

/// Upper bound on the reconstructed OTSL grid size (`rows * cols`).
///
/// Serialisation is O(rows*cols) in both memory (the occupancy grid) and output
/// tokens, and `rows`/`cols` are `u16` so a malformed table could declare a
/// ~4.3-billion-cell grid and exhaust memory. Tables beyond this cap are almost
/// certainly corrupt; we drop their OTSL content and record the loss instead of
/// risking OOM. One million cells comfortably exceeds any real HWP table.
const MAX_TABLE_GRID_CELLS: usize = 1_000_000;

/// What occupies a single position in the reconstructed table grid.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Occ {
    /// No cell covers this position (a hole in the declared geometry).
    Empty,
    /// The anchor (top-left) of the cell with the given index into `cells`.
    Anchor(usize),
    /// Covered by a span originating from the anchor at `(anchor_row,
    /// anchor_col)`.  `left`/`up` record the direction(s) of coverage relative
    /// to that anchor and select the continuation token.
    Cover { left: bool, up: bool },
}

/// Serialise `table` into a `<table>` element, appending to `out`.
///
/// `loc` is a human-readable source path used for any [`LossEntry`]; `loss`
/// accumulates fidelity losses.  This mirrors the other block writers in
/// [`super`]: it appends to `out` and never fails.
pub(crate) fn write_table(
    out: &mut String,
    table: &Table,
    loc: &str,
    opts: &ConvertOptions,
    locs: &LocationMap,
    loss: &mut LossReport,
) {
    let rows = table.rows as usize;
    let cols = table.cols as usize;

    out.push_str("<table>");

    // Element-head order: location precedes caption, which precedes content.
    emit_location(out, resolve_loc(opts, locs, table.prov));

    // Element-head order: caption precedes the OTSL content.
    if let Some(caption) = &table.caption {
        out.push_str("<caption>");
        write_inlines(out, caption);
        out.push_str("</caption>");
    }

    // A zero-sized grid has no tokens to emit.
    if rows == 0 || cols == 0 {
        // If anchors exist despite a zero-sized grid, the geometry is
        // inconsistent; flag it rather than silently dropping content.
        if !table.cells.is_empty() {
            loss.push(LossEntry {
                kind: LossKind::Other("table-empty-grid".to_string()),
                location: loc.to_string(),
                detail: format!(
                    "table declares {rows}x{cols} grid but carries {} cell(s); cells dropped",
                    table.cells.len()
                ),
            });
        }
        out.push_str("</table>");
        return;
    }

    // Guard against pathological geometries: the grid allocation and token
    // stream are O(rows*cols). A malformed table declaring a huge grid could
    // exhaust memory, so cap the reconstructed size — beyond it, record the loss
    // and emit an empty table rather than risking OOM.
    let grid_cells = rows.saturating_mul(cols);
    if grid_cells > MAX_TABLE_GRID_CELLS {
        loss.push(LossEntry {
            kind: LossKind::Other("table-too-large".to_string()),
            location: loc.to_string(),
            detail: format!(
                "table grid {rows}x{cols} ({grid_cells} cells) exceeds the \
                 {MAX_TABLE_GRID_CELLS}-cell cap; OTSL content dropped"
            ),
        });
        out.push_str("</table>");
        return;
    }

    // Build the occupancy grid.  `grid[r * cols + c]` describes position (r, c).
    let mut grid = vec![Occ::Empty; rows * cols];

    for (idx, cell) in table.cells.iter().enumerate() {
        let ar = cell.row as usize;
        let ac = cell.col as usize;

        // Defensive: anchors outside the declared grid cannot be placed.
        if ar >= rows || ac >= cols {
            loss.push(LossEntry {
                kind: LossKind::Other("table-cell-out-of-bounds".to_string()),
                location: loc.to_string(),
                detail: format!(
                    "cell anchor ({ar},{ac}) lies outside {rows}x{cols} grid; cell dropped"
                ),
            });
            continue;
        }

        // Clamp the span extent to the grid edges (span of 0 is treated as 1).
        let row_span = (cell.row_span.max(1) as usize).min(rows - ar);
        let col_span = (cell.col_span.max(1) as usize).min(cols - ac);

        let mut conflict = false;

        for r in ar..ar + row_span {
            for c in ac..ac + col_span {
                let pos = r * cols + c;
                // First writer wins; later overlaps are recorded, not applied.
                if grid[pos] != Occ::Empty {
                    conflict = true;
                    continue;
                }
                grid[pos] = if r == ar && c == ac {
                    Occ::Anchor(idx)
                } else {
                    Occ::Cover {
                        left: c > ac,
                        up: r > ar,
                    }
                };
            }
        }

        if conflict {
            loss.push(LossEntry {
                kind: LossKind::Other("table-overlapping-span".to_string()),
                location: loc.to_string(),
                detail: format!(
                    "cell anchor ({ar},{ac}) span {row_span}x{col_span} overlaps an earlier cell; overlap clamped"
                ),
            });
        }
    }

    // Emit tokens row-major.
    for r in 0..rows {
        for c in 0..cols {
            match grid[r * cols + c] {
                Occ::Empty => out.push_str("<ecel/>"),
                Occ::Anchor(idx) => {
                    let cell = &table.cells[idx];
                    if cell.is_header {
                        out.push_str("<ched/>");
                    } else if cell.content.is_empty() {
                        out.push_str("<ecel/>");
                    } else {
                        out.push_str("<fcel/>");
                    }
                    // Anchor token is followed by the serialised cell content.
                    // Recurse through the block writer so nested tables and any
                    // other block element render correctly.
                    let cell_loc = format!("{loc}/cell[{},{}]", cell.row, cell.col);
                    write_blocks(out, &cell.content, &cell_loc, opts, locs, loss);
                }
                Occ::Cover { left, up } => {
                    let token = match (left, up) {
                        (true, true) => "<xcel/>",
                        (true, false) => "<lcel/>",
                        (false, true) => "<ucel/>",
                        // Should be unreachable: a cover position is always
                        // displaced from its anchor in at least one axis.
                        (false, false) => "<ecel/>",
                    };
                    out.push_str(token);
                }
            }
        }
        out.push_str("<nl/>");
    }

    out.push_str("</table>");
}

#[cfg(test)]
mod tests {
    use crate::doclang::ir::block::Block;
    use crate::doclang::ir::inline::Inline;
    use crate::doclang::ir::table::{Table, TableCell};
    use crate::doclang::loss::{LossKind, LossReport};

    /// Build a simple body-paragraph block carrying `text`.
    fn para(text: &str) -> Block {
        Block::Paragraph {
            content: vec![Inline::Text(text.into())],
            lost: None,
            prov: None,
        }
    }

    /// Convenience anchor-cell constructor.
    fn cell(
        row: u16,
        col: u16,
        row_span: u16,
        col_span: u16,
        is_header: bool,
        content: Vec<Block>,
    ) -> TableCell {
        TableCell {
            row,
            col,
            row_span,
            col_span,
            is_header,
            content,
        }
    }

    /// Serialise a table in isolation and return `(xml, loss)`.
    fn emit(table: &Table) -> (String, LossReport) {
        let mut out = String::new();
        let mut loss = LossReport::new();
        let opts = crate::doclang::options::ConvertOptions::default();
        let locs = crate::doclang::ir::prov::LocationMap::new();
        super::write_table(
            &mut out,
            table,
            "section[0]/block[0]",
            &opts,
            &locs,
            &mut loss,
        );
        (out, loss)
    }

    #[test]
    fn simple_2x2() {
        let table = Table {
            rows: 2,
            cols: 2,
            cells: vec![
                cell(0, 0, 1, 1, false, vec![para("a")]),
                cell(0, 1, 1, 1, false, vec![para("b")]),
                cell(1, 0, 1, 1, false, vec![para("c")]),
                cell(1, 1, 1, 1, false, vec![para("d")]),
            ],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(
            xml,
            "<table>\
                <fcel/><text>a</text><fcel/><text>b</text><nl/>\
                <fcel/><text>c</text><fcel/><text>d</text><nl/>\
             </table>"
        );
        assert!(loss.is_empty());
    }

    #[test]
    fn header_row_uses_ched() {
        let table = Table {
            rows: 2,
            cols: 2,
            cells: vec![
                cell(0, 0, 1, 1, true, vec![para("H1")]),
                cell(0, 1, 1, 1, true, vec![para("H2")]),
                cell(1, 0, 1, 1, false, vec![para("x")]),
                cell(1, 1, 1, 1, false, vec![para("y")]),
            ],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(
            xml,
            "<table>\
                <ched/><text>H1</text><ched/><text>H2</text><nl/>\
                <fcel/><text>x</text><fcel/><text>y</text><nl/>\
             </table>"
        );
        assert!(loss.is_empty());
    }

    #[test]
    fn colspan_two_emits_lcel() {
        // Row 0: one cell spanning both columns. Row 1: two normal cells.
        let table = Table {
            rows: 2,
            cols: 2,
            cells: vec![
                cell(0, 0, 1, 2, false, vec![para("wide")]),
                cell(1, 0, 1, 1, false, vec![para("c")]),
                cell(1, 1, 1, 1, false, vec![para("d")]),
            ],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(
            xml,
            "<table>\
                <fcel/><text>wide</text><lcel/><nl/>\
                <fcel/><text>c</text><fcel/><text>d</text><nl/>\
             </table>"
        );
        assert!(loss.is_empty());
    }

    #[test]
    fn rowspan_two_emits_ucel() {
        // Col 0: one cell spanning both rows. Plus a cell in each row at col 1.
        let table = Table {
            rows: 2,
            cols: 2,
            cells: vec![
                cell(0, 0, 2, 1, false, vec![para("tall")]),
                cell(0, 1, 1, 1, false, vec![para("b")]),
                cell(1, 1, 1, 1, false, vec![para("d")]),
            ],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(
            xml,
            "<table>\
                <fcel/><text>tall</text><fcel/><text>b</text><nl/>\
                <ucel/><fcel/><text>d</text><nl/>\
             </table>"
        );
        assert!(loss.is_empty());
    }

    #[test]
    fn merged_block_emits_lcel_ucel_xcel() {
        // A single 2x2 cell occupies the whole table.
        let table = Table {
            rows: 2,
            cols: 2,
            cells: vec![cell(0, 0, 2, 2, false, vec![para("big")])],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(
            xml,
            "<table>\
                <fcel/><text>big</text><lcel/><nl/>\
                <ucel/><xcel/><nl/>\
             </table>"
        );
        assert!(loss.is_empty());
    }

    #[test]
    fn nested_table_in_cell() {
        let inner = Table {
            rows: 1,
            cols: 1,
            cells: vec![cell(0, 0, 1, 1, false, vec![para("inner")])],
            caption: None,
            prov: None,
        };
        let outer = Table {
            rows: 1,
            cols: 1,
            cells: vec![cell(0, 0, 1, 1, false, vec![Block::Table(inner)])],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&outer);
        assert_eq!(
            xml,
            "<table>\
                <fcel/>\
                <table><fcel/><text>inner</text><nl/></table>\
                <nl/>\
             </table>"
        );
        assert!(loss.is_empty());
    }

    #[test]
    fn empty_cells_use_ecel() {
        // Anchor with empty content -> <ecel/>; missing grid position -> <ecel/>.
        let table = Table {
            rows: 1,
            cols: 2,
            // Only the first position has a (content-less) anchor; col 1 is a hole.
            cells: vec![cell(0, 0, 1, 1, false, vec![])],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(xml, "<table><ecel/><ecel/><nl/></table>");
        assert!(loss.is_empty());
    }

    #[test]
    fn caption_emitted_before_content() {
        let table = Table {
            rows: 1,
            cols: 1,
            cells: vec![cell(0, 0, 1, 1, false, vec![para("x")])],
            caption: Some(vec![Inline::Text("Table 1: title".into())]),
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(
            xml,
            "<table>\
                <caption>Table 1: title</caption>\
                <fcel/><text>x</text><nl/>\
             </table>"
        );
        assert!(loss.is_empty());
    }

    #[test]
    fn out_of_bounds_cell_is_recorded_not_panicked() {
        let table = Table {
            rows: 1,
            cols: 1,
            cells: vec![
                cell(0, 0, 1, 1, false, vec![para("ok")]),
                cell(5, 5, 1, 1, false, vec![para("nope")]),
            ],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(xml, "<table><fcel/><text>ok</text><nl/></table>");
        assert_eq!(loss.len(), 1);
        assert_eq!(
            loss.iter().next().unwrap().kind,
            LossKind::Other("table-cell-out-of-bounds".to_string())
        );
    }

    #[test]
    fn overlapping_span_is_clamped_and_recorded() {
        // Second cell's anchor lands inside the first cell's 2x2 span.
        let table = Table {
            rows: 2,
            cols: 2,
            cells: vec![
                cell(0, 0, 2, 2, false, vec![para("big")]),
                cell(1, 1, 1, 1, false, vec![para("conflict")]),
            ],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        // First cell wins the whole grid; (1,1) stays a cross continuation.
        assert_eq!(
            xml,
            "<table>\
                <fcel/><text>big</text><lcel/><nl/>\
                <ucel/><xcel/><nl/>\
             </table>"
        );
        assert_eq!(loss.len(), 1);
        assert_eq!(
            loss.iter().next().unwrap().kind,
            LossKind::Other("table-overlapping-span".to_string())
        );
    }

    #[test]
    fn oversized_table_grid_is_capped_not_allocated() {
        // 2000 x 2000 = 4,000,000 cells > MAX_TABLE_GRID_CELLS: must NOT allocate
        // the grid; instead emit an empty table and record the loss.
        let table = Table {
            rows: 2000,
            cols: 2000,
            cells: vec![cell(0, 0, 1, 1, false, vec![para("a")])],
            caption: None,
            prov: None,
        };
        let (xml, loss) = emit(&table);
        assert_eq!(xml, "<table></table>");
        assert_eq!(loss.len(), 1);
        assert_eq!(
            loss.iter().next().unwrap().kind,
            LossKind::Other("table-too-large".to_string())
        );
    }
}
