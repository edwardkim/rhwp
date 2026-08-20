//! Table command ownership and shared coordinate resolution.

mod coordinates;
mod grid;
mod layout;
mod structure;

use coordinates::resolve_top_table;
pub(crate) use coordinates::{resolve_table_cell, CellResolveError};
pub(super) use grid::{
    edit_delete_col, edit_delete_row, edit_insert_col, edit_insert_row, edit_merge_cells,
    edit_split_cell, edit_split_cell_into,
};
pub(super) use layout::{
    edit_fit_table, edit_move_table, edit_resize_table, edit_resize_table_cell,
    edit_set_column_widths, edit_set_table_props,
};
pub(super) use structure::{
    edit_delete_table, edit_insert_table, edit_merge_table, edit_split_table, edit_transpose_table,
};
