//! Pure persisted-width validity queries, without rendering geometry or goldens.
use rhwp::model::table::{Cell, Table};
use std::collections::BTreeSet;

fn cell(row: u16, col: u16, span: u16, width: u32) -> Cell {
    Cell {
        row,
        col,
        col_span: span,
        row_span: 1,
        width,
        ..Default::default()
    }
}

#[test]
fn complete_row_widths_preserve_tolerance_and_source_column_ownership() {
    let mut table = Table {
        row_count: 6,
        col_count: 2,
        ..Default::default()
    };
    table.common.width = 1000; // Format tolerance = 1% = 10 HU.
    table.cells = vec![
        cell(0, 1, 1, 510),
        cell(0, 0, 1, 500), // exactly tolerance: valid
        cell(1, 1, 1, 511),
        cell(1, 0, 1, 500),  // complete but exceeds tolerance
        cell(2, 1, 1, 1500), // missing first column: not a complete declaration
        cell(3, 0, 2, 1100), // merged columns still form a complete row
        cell(4, 0, 1, 2000),
        cell(4, 0, 1, 2000), // duplicate column: incomplete
        cell(5, 0, 0, 2000), // zero span: incomplete
        cell(6, 0, 2, 2000), // outside declared row domain: excluded
    ];
    assert_eq!(table.invalid_declared_width_rows(), BTreeSet::from([1, 3]));
    table.common.width = 0;
    assert!(table.invalid_declared_width_rows().is_empty());
}

#[test]
fn sparse_rows_exclude_vertical_spans_and_keep_large_source_widths() {
    let mut table = Table {
        row_count: u16::MAX,
        col_count: 2,
        ..Default::default()
    };
    table.common.width = 1000;
    let mut spanning = cell(2, 0, 2, u32::MAX);
    spanning.row_span = 2;
    table.cells = vec![spanning, cell(u16::MAX - 1, 0, 2, u32::MAX)];
    assert_eq!(
        table.invalid_declared_width_rows(),
        BTreeSet::from([u16::MAX - 1])
    );
}
