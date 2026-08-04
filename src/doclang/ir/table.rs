use super::block::Block;
use super::inline::Inline;
use super::prov::Prov;

/// A table captured from an HWP `Table` control.
///
/// Row and column counts are stored explicitly; `cells` is in row-major order
/// (row 0 col 0, row 0 col 1, …, row 1 col 0, …).  Merged regions are
/// described by `col_span`/`row_span` on the anchor cell; non-anchor positions
/// in the grid are omitted from this vector (the writer reconstructs them via
/// the OTSL algorithm).
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// Number of rows.
    pub rows: u16,
    /// Number of columns.
    pub cols: u16,
    /// Anchor cells only, in row-major order.
    pub cells: Vec<TableCell>,
    /// Optional caption (maps to DocLang `<caption>` inside the table element).
    pub caption: Option<Vec<Inline>>,
    /// Model provenance for v2 `<location>` join; `None` when location is
    /// disabled or the table is nested (cell-local, no global provenance).
    pub prov: Option<Prov>,
}

/// A single table cell (anchor position only).
///
/// Non-anchor cells created by merging are not stored; the OTSL writer
/// reconstructs `lcel`/`ucel`/`xcel` tokens from the span values.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    /// Zero-based column index of the anchor position.
    pub col: u16,
    /// Zero-based row index of the anchor position.
    pub row: u16,
    /// Number of columns spanned (1 = no horizontal merge).
    pub col_span: u16,
    /// Number of rows spanned (1 = no vertical merge).
    pub row_span: u16,
    /// Whether this cell is a header cell (`<ched>` in OTSL).
    pub is_header: bool,
    /// Block-level content of the cell.  May contain nested `Block::Table` for
    /// tables-within-tables.
    pub content: Vec<Block>,
}
