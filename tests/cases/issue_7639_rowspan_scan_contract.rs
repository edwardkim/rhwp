//! Pure source-span queries; these assertions do not pin rendering geometry.
use rhwp::model::table::{Cell, Table};

fn cell(row: u16, span: u16) -> Cell {
    Cell {
        row,
        row_span: span,
        ..Default::default()
    }
}

#[test]
fn source_spans_preserve_half_open_bounds_and_overlapping_owners() {
    let table = Table {
        // The queried measured range owns its length, not the declaration.
        row_count: 2,
        cells: vec![cell(1, 3), cell(3, 4), cell(0, 0), cell(2, 1), cell(7, 2)],
        ..Default::default()
    };
    assert_eq!(
        table.rows_touched_by_rowspans(5),
        [false, true, true, true, true]
    );
    assert_eq!(
        table.rows_touched_by_rowspans(8),
        [false, true, true, true, true, true, true, true]
    );
    assert!(table.rows_touched_by_rowspans(0).is_empty());
}

#[test]
fn source_span_end_is_not_truncated_to_a_u16_coordinate() {
    let table = Table {
        cells: vec![cell(u16::MAX - 1, u16::MAX)],
        ..Default::default()
    };
    let touched = table.rows_touched_by_rowspans(usize::from(u16::MAX) + 2);
    assert!(touched[..usize::from(u16::MAX - 1)].iter().all(|&v| !v));
    assert!(touched[usize::from(u16::MAX - 1)..].iter().all(|&v| v));
}
