//! Table command ownership and shared coordinate resolution.

mod coordinates;

pub(crate) use coordinates::{resolve_table_cell, CellResolveError};
