//! Layout pipeline — turns a styled DOM into positioned boxes.
//!
//! Phase LAYOUT in the v0.3.35 plan, the largest remaining unknown.
//! Lands incrementally:
//!
//! - **LAYOUT-1** (this commit) — box tree construction. DOM ×
//!   ComputedStyles → [`BoxTree`]. No positioning yet; just the
//!   semantic tree the next sub-tasks size and place.
//! - **LAYOUT-2** — `ComputedStyles → taffy::Style` mapping for
//!   block/flex/grid/table modes; Taffy delegates inline measurement
//!   back to us.
//! - **LAYOUT-3** — inline formatting context (line boxes, BiDi, UAX
//!   #14 line breaks, justify, vertical-align, decorations,
//!   `::first-line`/`::first-letter`).
//! - **LAYOUT-4..7** — floats, margin collapsing, multi-column, tables.

pub mod box_tree;
pub mod floats;
pub mod inline;
pub mod margin_collapse;
pub mod multicol;
pub mod tables;
pub mod taffy_style;

pub use box_tree::{
    build_box_tree, BoxId, BoxKind, BoxNode, BoxTree, BoxTreeError, DisplayInside, DisplayOutside,
};
pub use floats::{read_clear, read_float, Clear, FloatBox, FloatContext, FloatSide};
pub use inline::{layout_paragraph, InlineItem, LineBox, LineFragment, TextAlign, WhiteSpace};
pub use margin_collapse::{collapse_margins, parent_child_bottom, parent_child_top, sibling_gap};
pub use multicol::{
    column_rects, distribute_lines_into_columns, read_multicol, ColumnRect, MultiColConfig,
};
pub use tables::{
    compute_column_widths, compute_row_heights, CellHint, LayoutAlgorithm, RowGroupKind, RowHint,
    TableLayout,
};
pub use taffy_style::{run_layout, style_to_taffy, LayoutBox, LayoutResult};
