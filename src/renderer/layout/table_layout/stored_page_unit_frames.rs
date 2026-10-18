//! 재조판 글줄 뒤의 원본 쪽 프레임 점유를 공통 유닛에 보존한다.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::table::{Cell, Table};
use crate::renderer::float_placement::{stored_text_page_frame, StoredTextPageFrame};
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
    pub(super) fn stored_page_frame_for_unit(
        &self,
        cell: &Cell,
        table: &Table,
        units: &[CellUnit],
        start: usize,
        padding: f64,
    ) -> Option<StoredTextPageFrame> {
        if !self.stored_span_cut_preserves_line_slots(table, cell)
            || (hwpunit_to_px(cell.effective_vertical_padding_hu(&table.padding), self.dpi)
                - padding)
                .abs()
                > hwpunit_to_px(1, self.dpi)
        {
            return None;
        }
        let first = units.get(start)?;
        if first.vis_start != 0 || start > 0 && units[start - 1].para_idx == first.para_idx {
            return None;
        }
        stored_text_page_frame(cell, table, first.para_idx)
    }

    pub(super) fn preserve_source_page_frame_extents(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        units: &mut [CellUnit],
    ) {
        if !self.stored_span_cut_preserves_line_slots(table, cell) {
            return;
        }
        let (_, _, top, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
        for start in 0..units.len() {
            let Some(frame) =
                self.stored_page_frame_for_unit(cell, table, units, start, top + bottom)
            else {
                continue;
            };
            let end = units[start..]
                .iter()
                .position(|unit| unit.para_idx >= frame.end_paragraph)
                .map_or(units.len(), |relative| start + relative);
            if end <= start || units[end - 1].para_idx + 1 != frame.end_paragraph {
                continue;
            }
            let advance = units[start..end]
                .iter()
                .map(|unit| unit.height)
                .sum::<f64>();
            let required = hwpunit_to_px(frame.content_height_hu, self.dpi);
            if required > advance {
                // 실제 마지막 글줄·가시 범위는 유지하고, 그 뒤 원본 빈 점유만 전진한다.
                // 일반 행과 블록 컷의 높이 walk가 같은 캐시된 유닛을 읽는다.
                units[end - 1].height += required - advance;
            }
        }
    }
}
