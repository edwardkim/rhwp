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
            "DIAG_CELL_UNIT row={} col={} span={} index={} height={:.6} para={} lines={}..{} hard_break={} stored_frame_break={} page_frame_reset={} source={:?} nested_row={:?} nested_cut={:?}",
            cell.row, cell.col, cell.row_span, index, unit.height, unit.para_idx,
            unit.vis_start, unit.vis_end, unit.hard_break_before,
            unit.stored_frame_break_before, unit.page_frame_reset_before,
            seg.map(|seg| (seg.vertical_pos, seg.line_height, seg.line_spacing)),
            unit.nested_row,
            unit.nested_table_fragment.as_ref().map(|cut| (
                &cut.start_cut,
                &cut.end_cut,
                cut.terminal,
            )),
        );
    }
}

pub(in crate::renderer::layout) fn trace_alignment(
    cell: &Cell,
    position: (f64, f64),
    heights: (f64, f64, f64, f64),
    align: crate::model::table::VerticalAlign,
    owners: &[(&str, bool)],
    cut: Option<(usize, usize)>,
) {
    if selected(cell) {
        eprintln!(
            "DIAG_CELL_ALIGN row={} col={} cut={cut:?} align={align:?} cell_y={:.6} text_y={:.6} cell_h={:.6} pad={:.6} content={:.6} centered={:.6} owners={owners:?}",
            cell.row, cell.col, position.0, position.1,
            heights.0, heights.1, heights.2, heights.3,
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
