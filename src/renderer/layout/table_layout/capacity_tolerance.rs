//! 저장 줄 복원 관용과 새로 조판한 텍스트 줄의 합산 오차를 구분한다.

use super::CellUnit;
use crate::model::table::Cell;

/// 재조판 줄은 실제 내용 예산을 넘겨 수용하지 않는다. 부동소수 합산의
/// 연산 횟수와 값의 크기로 설명되는 오차만 남긴다. 기존 저장 프레임과
/// 중첩 개체의 복원 관용은 해당 경로의 계약을 유지한다.
pub(super) fn for_next_unit(
    cell: &Cell,
    unit: &CellUnit,
    unit_count: usize,
    next_height: f64,
    available: f64,
    stored_tolerance: f64,
) -> f64 {
    let composed_text = unit.nested_row.is_none()
        && !unit.mixed_nested_fragment
        && !unit.top_and_bottom_flow
        && cell.paragraphs.get(unit.para_idx).is_some_and(|para| {
            para.controls.is_empty() && crate::renderer::para_has_no_stored_line_segs(para)
        });
    let operations = unit_count.saturating_add(1) as f64;
    let tolerance = if composed_text {
        (next_height.abs() + available.abs()) * f64::EPSILON * operations
    } else {
        stored_tolerance
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        static TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if *TRACE.get_or_init(|| std::env::var_os("RHWP_DIAG_REFLOW_CAPACITY").is_some())
            && (next_height - available).abs() <= unit.height.abs()
        {
            let paragraph = cell.paragraphs.get(unit.para_idx);
            eprintln!(
                "DIAG_REFLOW_CAPACITY row={} col={} para={} no_ls={} controls={:?} nested={:?} mixed={} topbottom={} next={:.12} available={:.12} tolerance={:.12}",
                cell.row,
                cell.col,
                unit.para_idx,
                paragraph.is_some_and(crate::renderer::para_has_no_stored_line_segs),
                paragraph.map(|para| para.controls.len()),
                unit.nested_row,
                unit.mixed_nested_fragment,
                unit.top_and_bottom_flow,
                next_height,
                available,
                tolerance,
            );
        }
    }
    tolerance
}
