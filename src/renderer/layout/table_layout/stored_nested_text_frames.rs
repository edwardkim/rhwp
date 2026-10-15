//! 확인된 부모·자식 저장 프레임을 원본 유닛 투영과 실제 배치가 공유한다.

use super::{LayoutEngine, NestedFlowFragment, RecursiveBlockPreludeRole, ResolvedStyleSet};
use crate::{
    model::table::Table,
    renderer::{hwpunit_to_px, render_normalization::StoredTextFrame},
};

impl LayoutEngine {
    pub(super) fn stored_nested_text_frame_fragments(
        &self,
        table: &Table,
        styles: &ResolvedStyleSet,
    ) -> Option<Vec<NestedFlowFragment>> {
        let frames = self
            .render_normalization
            .borrow()
            .stored_nested_text_frames(table)?;
        let cell = table.cells.first()?;
        let units = self.cell_units(cell, table, styles);
        if units.len() != frames.units.len()
            || units.iter().zip(&frames.units).any(|(unit, source)| {
                unit.para_idx != source.paragraph
                    || unit.vis_start != source.line
                    || unit.vis_end != source.line + 1
                    || unit.empty_spacer
                    || unit.nested_row.is_some()
                    || unit.mixed_nested_fragment
            })
        {
            return None;
        }
        Some(
            frames
                .units
                .iter()
                .enumerate()
                .map(|(index, unit)| NestedFlowFragment {
                    height: hwpunit_to_px(unit.physical_height_hu, self.dpi),
                    hard_break_before: index > 0 && unit.starts_frame,
                    stored_frame_break_before: index > 0 && unit.starts_frame,
                    trailing: false,
                    content_height: hwpunit_to_px(
                        cell.paragraphs[unit.paragraph].line_segs[unit.line].line_height,
                        self.dpi,
                    ),
                    recursive: true,
                    starts_after_table: false,
                    source_para_idx: Some(unit.paragraph),
                    recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                })
                .collect(),
        )
    }

    pub(in crate::renderer::layout) fn stored_nested_text_frame_for_cut(
        &self,
        table: &Table,
        start: &[usize],
        end: &[usize],
        start_is_block: bool,
        end_is_block: bool,
    ) -> Option<StoredTextFrame> {
        if start_is_block || end_is_block || start.len() > 1 || end.len() > 1 {
            return None;
        }
        let frames = self
            .render_normalization
            .borrow()
            .stored_nested_text_frames(table)?;
        frames.frame_for_cut(
            start.first().copied().unwrap_or(0),
            end.first().copied().unwrap_or(usize::MAX),
        )
    }
}
