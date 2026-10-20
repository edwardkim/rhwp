//! 저장 줄 없는 중첩 행이 내용에 따라 성장하면 현재 글줄 컷을 보존한다.

use super::{CellUnit, LayoutEngine};
use crate::model::table::Cell;

impl LayoutEngine {
    pub(super) fn reflowed_nested_row_content_grew(
        &self,
        row_cells: &[&Cell],
        driver_units: &[CellUnit],
        driver: usize,
        declared_height: f64,
    ) -> bool {
        let Some(cell) = row_cells.get(driver) else {
            return false;
        };
        if driver_units.len() <= 1
            || cell.paragraphs.is_empty()
            || !cell.paragraphs.iter().all(|paragraph| {
                paragraph.controls.is_empty()
                    && crate::renderer::para_has_no_stored_line_segs(paragraph)
            })
            || !cell
                .paragraphs
                .iter()
                .any(|paragraph| paragraph.text.chars().any(|ch| !ch.is_whitespace()))
        {
            return false;
        }
        // 부모의 현재 내용 유닛을 다시 재거나 선언 크기 비율로 추정하지 않는다.
        // 같은 유닛의 전진이 선언 최소 높이를 채웠는지만 비교한다.
        let content_height: f64 = driver_units.iter().map(|unit| unit.height).sum();
        let roundoff =
            f64::EPSILON * content_height.abs().max(declared_height.abs()).max(1.0) * 8.0;
        content_height > declared_height + roundoff
    }
}
