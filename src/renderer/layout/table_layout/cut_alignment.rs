//! 선택된 중첩 표 컷의 잉크 점유와 다음 줄 전진을 구분한다.

use super::{CellUnit, Control, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 글줄 범위에 표현되지 않는 중첩 표도 선택된 컷의 정렬 점유에 포함한다.
    pub(in crate::renderer::layout) fn nested_cell_cut_content_height(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start: usize,
        end: usize,
    ) -> f64 {
        let units = self.cell_units(cell, table, styles);
        let lo = start.min(units.len());
        let window = &units[lo..end.clamp(lo, units.len())];
        if window.iter().any(|unit| {
            unit.mixed_nested_fragment
                || unit.nested_row.is_some()
                || unit.nested_table_fragment.is_some()
                || cell.paragraphs.get(unit.para_idx).is_some_and(|paragraph| {
                    paragraph
                        .controls
                        .iter()
                        .any(|control| matches!(control, Control::Table(_)))
                })
        }) {
            let trailing = self.nested_cut_terminal_line_spacing(table, cell, window);
            // [#7870] 다음 조각으로 이어지는 컷의 끝 중첩 표 atom 이송 여백도 정렬 잉크가
            // 아니다. 위 `trailing` 은 끝 유닛이 일반 글줄일 때만이라 둘은 겹치지 않는다.
            let atom_tail =
                self.continued_cut_nested_atom_tail(cell, &units, lo, lo + window.len(), styles);
            window.iter().map(|unit| unit.height).sum::<f64>()
                + self.mixed_nested_flow_extra_from_cut(cell, table, styles, lo, lo + window.len())
                - trailing
                - atom_tail
        } else {
            0.0
        }
    }

    /// 마지막 일반 글줄 뒤의 전진 간격은 같은 조각의 가운데 정렬 잉크가 아니다.
    /// 선택된 유닛이 원본 줄 높이+간격을 실제로 계상한 경우만 분리한다.
    fn nested_cut_terminal_line_spacing(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        selected: &[CellUnit],
    ) -> f64 {
        use crate::model::paragraph::LineSeg;
        let profile = self.profile.get();
        if !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || profile.session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || cell.dirty_flag
        {
            return 0.0;
        }
        let Some(last) = selected.last() else {
            return 0.0;
        };
        let paragraph = &cell.paragraphs[last.para_idx];
        if last.nested_row.is_some()
            || last.nested_table_fragment.is_some()
            || last.mixed_nested_fragment
            || last.mixed_nested_trailing
            || last.empty_spacer
            || last.vis_end != last.vis_start + 1
            || !paragraph.controls.is_empty()
            || paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
        {
            return 0.0;
        }
        let Some(line) = paragraph.line_segs.get(last.vis_start) else {
            return 0.0;
        };
        if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || line.line_height <= 0 {
            return 0.0;
        }
        let advance = super::hwpunit_to_px(line.line_height, self.dpi)
            + super::hwpunit_to_px(line.line_spacing, self.dpi);
        if (last.height - advance).abs() <= super::hwpunit_to_px(2, self.dpi) {
            super::hwpunit_to_px(line.line_spacing, self.dpi)
        } else {
            0.0
        }
    }
}
