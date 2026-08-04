//! Table lowering — rhwp [`Table`] → crate-IR [`ir::Table`].
//!
//! rhwp stores tables as a flat list of **anchor cells** (`Table.cells`), each
//! carrying its zero-based `(col, row)` origin plus `col_span`/`row_span`.
//! Non-anchor (merged-over) grid positions are *not* present in `cells`; they
//! are only referenced by `Table.cell_grid` (a `row*col_count+col` → anchor
//! index map).  Our IR uses the same anchor-only convention
//! ([`ir::Table::cells`] documents this), so the mapping is a direct 1:1
//! transform of `Table.cells` → [`ir::TableCell`] with no grid expansion.
//!
//! # Paragraph-conversion callback contract
//!
//! Each rhwp [`Cell`] owns a `Vec<Paragraph>` whose lowering into block-level
//! [`Block`] nodes lives in the paragraph adapter (task T5).  To keep *this*
//! module free of any dependency on `paragraph.rs` (which is built in a later
//! wave and wired here by T8), the per-cell paragraph conversion is injected as
//! a callback:
//!
//! ```ignore
//! convert_paragraphs: &dyn Fn(&[crate::model::paragraph::Paragraph]) -> Vec<Block>
//! ```
//!
//! The callback is invoked once per cell with that cell's `paragraphs` slice and
//! must return the cell's block content (including any nested
//! [`Block::Table`] for tables-within-tables).  T8 supplies the real closure
//! that delegates to the T5 paragraph adapter; the unit tests below pass a
//! trivial stub.
//!
//! # Caption
//!
//! rhwp's [`Caption`](crate::model::shape::Caption) is a paragraph list.  We
//! flatten every caption paragraph through [`extract_inlines`] and concatenate
//! the results into [`ir::Table::caption`] (`Option<Vec<Inline>>`).  Caption
//! *direction* / *width* / *spacing* have no DocLang counterpart; if a caption
//! is present we record a single [`LossKind::Caption`] entry noting the dropped
//! geometry so lean-mode audits stay honest (plan Critic N4).
//!
//! # Defensive clamping
//!
//! A malformed document could place a cell origin outside the declared
//! `row_count`/`col_count` grid.  Such a cell is clamped into range and a
//! [`LossKind::Other`] entry is recorded; this keeps the OTSL writer's grid
//! walk well-formed instead of panicking on an out-of-bounds anchor.

use std::collections::HashSet;

use crate::model::document::DocInfo;
use crate::model::paragraph::Paragraph;
use crate::model::shape::Caption;
use crate::model::table::Table;

use crate::doclang::ir::{Block, Inline, Table as IrTable, TableCell as IrTableCell};
use crate::doclang::loss::report::{LossEntry, LossKind, LossReport};

use super::inline::extract_inlines;

/// Lower a rhwp [`Table`] into the crate IR [`ir::Table`].
///
/// `convert_paragraphs` lowers one cell's paragraph list into block content
/// (see the module-level contract); it is injected so this module has no
/// dependency on the paragraph adapter (`paragraph.rs`, wired by T8).
///
/// `location` is a human-readable source path (e.g. `"s0/p3/tbl"`) used for
/// loss-entry attribution.
///
/// `loss` accumulates lean-mode loss entries: at most one
/// [`LossKind::Caption`] per table (caption geometry) and one
/// [`LossKind::Other`] per out-of-range cell that had to be clamped.
pub(crate) fn convert_table(
    table: &Table,
    doc_info: &DocInfo,
    location: &str,
    convert_paragraphs: &dyn Fn(&[Paragraph]) -> Vec<Block>,
    loss: &mut LossReport,
) -> IrTable {
    let rows = table.row_count;
    let cols = table.col_count;

    let mut cells: Vec<IrTableCell> = Vec::with_capacity(table.cells.len());
    // Track anchor positions already taken so clamping can flag collisions:
    // two out-of-bounds cells clamped to the same corner would otherwise overlap
    // silently. Well-formed tables never collide (every anchor is distinct).
    let mut occupied: HashSet<(u16, u16)> = HashSet::with_capacity(table.cells.len());
    for cell in &table.cells {
        // Defensive: clamp anchor origins that fall outside the declared grid.
        // `rows`/`cols` of 0 would make every coordinate out of range; guard so
        // the clamp target (`rows - 1`) does not underflow.
        let mut col = cell.col;
        let mut row = cell.row;

        if cols == 0 || row >= rows || col >= cols {
            let new_col = if cols == 0 { 0 } else { col.min(cols - 1) };
            let new_row = if rows == 0 { 0 } else { row.min(rows - 1) };
            loss.push(LossEntry {
                kind: LossKind::Other("table-cell-out-of-bounds".to_string()),
                location: location.to_string(),
                detail: format!(
                    "cell ({},{}) outside {}x{} grid, clamped to ({},{})",
                    row, col, rows, cols, new_row, new_col
                ),
            });
            col = new_col;
            row = new_row;
        }

        // If the (possibly clamped) anchor lands on a position already taken,
        // record the collision so the overlap is auditable rather than silent.
        if !occupied.insert((row, col)) {
            loss.push(LossEntry {
                kind: LossKind::Other("table-cell-collision".to_string()),
                location: location.to_string(),
                detail: format!(
                    "cell anchor ({},{}) collides with an earlier cell",
                    row, col
                ),
            });
        }

        // Spans must be at least 1; a malformed 0 span would yield an empty
        // grid region.  Anchor cells legitimately carry span >= 1 in rhwp.
        let col_span = cell.col_span.max(1);
        let row_span = cell.row_span.max(1);

        cells.push(IrTableCell {
            col,
            row,
            col_span,
            row_span,
            is_header: cell.is_header,
            content: convert_paragraphs(&cell.paragraphs),
        });
    }

    let caption = table
        .caption
        .as_ref()
        .and_then(|cap| convert_caption(cap, doc_info, location, loss));

    IrTable {
        rows,
        cols,
        cells,
        caption,
        // Provenance is stamped by the body driver (paragraph.rs), which knows
        // the (section, para, control) position of the owning Table control.
        // Nested tables (lowered through the cell paragraph callback) stay
        // `None` — their indices are cell-local and carry no global provenance.
        prov: None,
    }
}

/// Flatten a rhwp [`Caption`]'s paragraph list into inline content.
///
/// Returns `None` when the caption has no representable inline content (so the
/// IR stores `caption: None` rather than `Some(vec![])`).  Caption geometry
/// (direction / width / spacing) is unrepresentable in DocLang; a single
/// [`LossKind::Caption`] entry records the drop whenever a caption is present.
fn convert_caption(
    caption: &Caption,
    doc_info: &DocInfo,
    location: &str,
    loss: &mut LossReport,
) -> Option<Vec<Inline>> {
    let caption_loc = format!("{}/caption", location);
    let mut inlines: Vec<Inline> = Vec::new();
    for para in &caption.paragraphs {
        inlines.extend(extract_inlines(para, doc_info, &caption_loc, loss));
    }

    // Caption is present → its non-inline geometry is dropped in lean mode.
    loss.push(LossEntry {
        kind: LossKind::Caption,
        location: caption_loc,
        detail: format!(
            "caption direction/width/spacing dropped (direction={:?})",
            caption.direction
        ),
    });

    if inlines.is_empty() {
        None
    } else {
        Some(inlines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::DocInfo;
    use crate::model::paragraph::Paragraph;
    use crate::model::shape::Caption;
    use crate::model::table::{Cell, Table};

    /// Trivial paragraph→block stub: emits one empty `Block::Paragraph` per
    /// rhwp paragraph so tests can assert cell content length without depending
    /// on the (later-wave) paragraph adapter.
    fn stub_convert(paras: &[Paragraph]) -> Vec<Block> {
        paras
            .iter()
            .map(|_| Block::Paragraph {
                content: Vec::new(),
                lost: None,
                prov: None,
            })
            .collect()
    }

    fn anchor(col: u16, row: u16, col_span: u16, row_span: u16) -> Cell {
        Cell {
            col,
            row,
            col_span,
            row_span,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        }
    }

    #[test]
    fn simple_2x2_maps_all_cells() {
        let table = Table {
            row_count: 2,
            col_count: 2,
            cells: vec![
                anchor(0, 0, 1, 1),
                anchor(1, 0, 1, 1),
                anchor(0, 1, 1, 1),
                anchor(1, 1, 1, 1),
            ],
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        assert_eq!(out.rows, 2);
        assert_eq!(out.cols, 2);
        assert_eq!(out.cells.len(), 4);
        // Each cell has one paragraph → one Block via the stub.
        assert!(out.cells.iter().all(|c| c.content.len() == 1));
        assert!(out.cells.iter().all(|c| c.col_span == 1 && c.row_span == 1));
        assert!(out.caption.is_none());
        assert!(loss.is_empty());
    }

    #[test]
    fn colspan_and_rowspan_preserved() {
        let table = Table {
            row_count: 2,
            col_count: 2,
            // One cell spanning both columns of row 0, one cell spanning both
            // rows of column 0 below it, plus a filler anchor.
            cells: vec![anchor(0, 0, 2, 1), anchor(0, 1, 1, 2)],
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        assert_eq!(out.cells.len(), 2);
        assert_eq!(out.cells[0].col_span, 2);
        assert_eq!(out.cells[0].row_span, 1);
        assert_eq!(out.cells[1].col_span, 1);
        assert_eq!(out.cells[1].row_span, 2);
        assert!(loss.is_empty());
    }

    #[test]
    fn is_header_flag_preserved() {
        let mut header = anchor(0, 0, 1, 1);
        header.is_header = true;
        let table = Table {
            row_count: 1,
            col_count: 2,
            cells: vec![header, anchor(1, 0, 1, 1)],
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        assert!(out.cells[0].is_header);
        assert!(!out.cells[1].is_header);
    }

    #[test]
    fn caption_present_extracts_inlines_and_records_loss() {
        // Build a caption paragraph with visible text via the same shape the
        // inline adapter expects (text + char_offsets).
        let text = "Table 1";
        let char_offsets: Vec<u32> = (0..text.chars().count() as u32).collect();
        let cap_para = Paragraph {
            text: text.to_string(),
            char_offsets,
            ..Default::default()
        };
        let table = Table {
            row_count: 1,
            col_count: 1,
            cells: vec![anchor(0, 0, 1, 1)],
            caption: Some(Caption {
                paragraphs: vec![cap_para],
                ..Default::default()
            }),
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        assert_eq!(out.caption, Some(vec![Inline::Text("Table 1".to_string())]));
        // Exactly one Caption loss entry for the dropped geometry.
        assert_eq!(
            loss.iter().filter(|e| e.kind == LossKind::Caption).count(),
            1
        );
    }

    #[test]
    fn out_of_bounds_cell_clamped_and_recorded() {
        let table = Table {
            row_count: 2,
            col_count: 2,
            // Second cell is anchored outside the 2x2 grid.
            cells: vec![anchor(0, 0, 1, 1), anchor(5, 9, 1, 1)],
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        // Clamped to the last valid coordinate (row 1, col 1).
        assert_eq!(out.cells[1].col, 1);
        assert_eq!(out.cells[1].row, 1);
        let other_count = loss
            .iter()
            .filter(|e| matches!(&e.kind, LossKind::Other(s) if s == "table-cell-out-of-bounds"))
            .count();
        assert_eq!(other_count, 1);
    }

    #[test]
    fn clamped_cells_colliding_on_same_anchor_are_recorded() {
        let table = Table {
            row_count: 2,
            col_count: 2,
            // Both out-of-bounds cells clamp to the same corner (1,1) → collision.
            cells: vec![anchor(0, 0, 1, 1), anchor(5, 9, 1, 1), anchor(6, 8, 1, 1)],
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let _ = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        let collisions = loss
            .iter()
            .filter(|e| matches!(&e.kind, LossKind::Other(s) if s == "table-cell-collision"))
            .count();
        assert_eq!(collisions, 1);
    }

    #[test]
    fn well_formed_table_records_no_collision() {
        let table = Table {
            row_count: 2,
            col_count: 2,
            cells: vec![
                anchor(0, 0, 1, 1),
                anchor(1, 0, 1, 1),
                anchor(0, 1, 1, 1),
                anchor(1, 1, 1, 1),
            ],
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let _ = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        assert!(loss
            .iter()
            .all(|e| !matches!(&e.kind, LossKind::Other(s) if s == "table-cell-collision")));
    }

    #[test]
    fn empty_caption_paragraphs_yields_none_caption() {
        let table = Table {
            row_count: 1,
            col_count: 1,
            cells: vec![anchor(0, 0, 1, 1)],
            caption: Some(Caption {
                paragraphs: vec![Paragraph::default()], // no text
                ..Default::default()
            }),
            ..Default::default()
        };
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = convert_table(&table, &di, "s0/tbl", &stub_convert, &mut loss);

        assert!(out.caption.is_none());
        // Geometry loss still recorded even when no inline content survives.
        assert_eq!(
            loss.iter().filter(|e| e.kind == LossKind::Caption).count(),
            1
        );
    }
}
