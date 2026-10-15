//! Opt-in source-unit diagnostics; never changes the accepted cut or geometry.

use super::CellUnit;
use crate::model::table::Cell;

fn selected(cell: &Cell) -> bool {
    static CELLS: std::sync::OnceLock<Vec<(u16, u16)>> = std::sync::OnceLock::new();
    CELLS
        .get_or_init(|| {
            std::env::var("RHWP_DIAG_CELL_UNITS")
                .unwrap_or_default()
                .split('|')
                .filter_map(|pair| {
                    let (row, col) = pair.split_once(',')?;
                    Some((row.parse().ok()?, col.parse().ok()?))
                })
                .collect()
        })
        .contains(&(cell.row, cell.col))
}

pub(super) fn trace_units(cell: &Cell, units: &[CellUnit]) {
    if !selected(cell) {
        return;
    }
    for (index, unit) in units.iter().enumerate() {
        let seg = cell.paragraphs[unit.para_idx].line_segs.get(unit.vis_start);
        eprintln!(
            "DIAG_CELL_UNIT row={} col={} span={} index={} height={:.6} para={} lines={}..{} hard_break={} stored_frame_break={} page_frame_reset={} source={:?}",
            cell.row, cell.col, cell.row_span, index, unit.height, unit.para_idx,
            unit.vis_start, unit.vis_end, unit.hard_break_before,
            unit.stored_frame_break_before, unit.page_frame_reset_before,
            seg.map(|seg| (seg.vertical_pos, seg.line_height, seg.line_spacing)),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn trace_span_cut(
    cell: &Cell,
    start: usize,
    end: usize,
    prior: f64,
    height: f64,
    su: usize,
    eu: usize,
) {
    if selected(cell) {
        eprintln!(
            "DIAG_CELL_SPAN row={} col={} rows={}..{} prior={:.6} height={:.6} units={}..{}",
            cell.row, cell.col, start, end, prior, height, su, eu,
        );
    }
}
