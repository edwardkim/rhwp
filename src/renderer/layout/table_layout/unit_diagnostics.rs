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
            "DIAG_CELL_UNIT row={} col={} span={} index={} height={:.6} following={:.6} reflowed_slot={} para={} lines={}..{} hard_break={} stored_frame_break={} page_frame_reset={} source={:?} nested_row={:?} nested_cut={:?}",
            cell.row, cell.col, cell.row_span, index, unit.height,
            unit.following_line_advance, unit.reflowed_line_slot, unit.para_idx,
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

/// 저장 혼합 컷의 인증이 끝난 지점을 선택한 셀에 한해 기록한다.
pub(super) fn trace_mixed_opening(cell: &Cell, stage: &str) {
    if selected(cell) {
        eprintln!(
            "DIAG_MIXED_OPENING row={} col={} stage={stage}",
            cell.row, cell.col
        );
    }
}

/// 로컬 원점 복원과 독립적인 물리 페이지 경계의 판정을 구분한다.
pub(super) fn trace_reset_cut(
    cell: &Cell,
    cut: (usize, usize),
    budget: (f64, f64),
    unit: &CellUnit,
    strict: bool,
    zero_origin: impl FnOnce() -> bool,
) {
    if selected(cell) && unit.page_frame_reset_before {
        eprintln!(
            "DIAG_RESET_CUT row={} col={} start={} next={} height={:.6} available={:.6} unit_height={:.6} para={} line={} hard={} stored={} strict={} local_zero_origin={}",
            cell.row, cell.col, cut.0, cut.1, budget.0, budget.1, unit.height,
            unit.para_idx, unit.vis_start, unit.hard_break_before,
            unit.stored_frame_break_before, strict, zero_origin(),
        );
    }
}

/// 원점 되감김과 현재 물리 컷의 인증 근거를 선택한 셀에서 분리해 기록한다.
pub(super) fn trace_reset_owners(
    cell: &Cell,
    cursor: (usize, usize),
    unit: &CellUnit,
    owners: &[(&str, bool)],
) {
    if selected(cell)
        && (unit.page_frame_reset_before
            || unit.stored_frame_break_before
            || unit.hard_break_before)
    {
        eprintln!(
            "DIAG_RESET_OWNERS row={} col={} start={} next={} para={} line={} owners={owners:?}",
            cell.row, cell.col, cursor.0, cursor.1, unit.para_idx, unit.vis_start,
        );
    }
}

/// 같은 종료 행의 요구에서 실제 상자와 추정 띠 중 어느 원장을 썼는지 확인한다.
pub(super) fn trace_span_remainder(
    cell: &Cell,
    cursor_row: usize,
    accepted: Option<f64>,
    heights: (f64, f64, f64, f64, f64),
) {
    if selected(cell) {
        let (required, estimated_prefix, actual_prefix, band, remaining) = heights;
        eprintln!(
            "DIAG_SPAN_REMAINDER row={} col={} cursor={} accepted={accepted:?} required={required:.6} estimated_prefix={estimated_prefix:.6} actual_prefix={actual_prefix:.6} band={band:.6} remaining={remaining:.6}",
            cell.row, cell.col, cursor_row,
        );
    }
}

/// 수용 전 원장과 현재 PageItem snapshot의 같은 종료 요구를 대조한다.
pub(super) fn trace_continuation_demand(
    cell: &Cell,
    heights: (f64, f64),
    ledger: (Option<f64>, Option<f64>),
    snapshot: bool,
) {
    if selected(cell) {
        eprintln!(
            "DIAG_CONTINUATION_DEMAND row={} col={} content={:.6} physical={:.6} prefix={:?} through={:?} snapshot={snapshot}",
            cell.row, cell.col, heights.0, heights.1, ledger.0, ledger.1,
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
