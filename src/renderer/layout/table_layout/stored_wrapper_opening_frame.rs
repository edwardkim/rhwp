//! 자식의 저장 쪽 경계와 함께 닫히는 부모 래퍼의 첫 물리 상자.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::{
    control::Control,
    paragraph::LineSeg,
    shape::{TextWrap, VertAlign, VertRelTo},
    table::{Table, TablePageBreak},
};
use crate::renderer::{float_placement::signed_hwpunit, hwpunit_to_px};

pub(in crate::renderer::layout) struct StoredWrapperOpeningFrame {
    pub(in crate::renderer::layout) height: f64,
    pub(in crate::renderer::layout) child_top: f64,
}

impl LayoutEngine {
    /// 선언 높이를 임의 하한으로 쓰지 않는다. 선택한 자식 컷의 물리 높이가
    /// 자식의 첫 저장 상자와 같고, 그 상자·오프셋·여백이 부모 선언을 닫아야 한다.
    pub(in crate::renderer::layout) fn stored_wrapper_opening_frame(
        &self,
        table: &Table,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<StoredWrapperOpeningFrame> {
        let profile = self.profile.get();
        let [cell] = table.cells.as_slice() else {
            return None;
        };
        let [para] = cell.paragraphs.as_slice() else {
            return None;
        };
        let [Control::Table(child)] = para.controls.as_slice() else {
            return None;
        };
        let [host] = para.line_segs.as_slice() else {
            return None;
        };
        let [end] = end_cut else {
            return None;
        };
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || table.row_count != 1
            || table.col_count != 1
            || table.common.treat_as_char
            || table.page_break != TablePageBreak::RowBreak
            || cell.dirty_flag
            || !para.text.trim().is_empty()
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || host.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || host.vertical_pos != 0
            || host.line_height <= 0
            || start_cut.iter().any(|cut| *cut != 0)
            || start_cut.len() > 1
            || table.caption.is_some()
            || child.caption.is_some()
            || child.common.treat_as_char
            || !child.common.flow_with_text
            || child.common.allow_overlap
            || child.common.vert_rel_to != VertRelTo::Para
            || child.common.vert_align != VertAlign::Top
            || !matches!(
                child.common.text_wrap,
                TextWrap::TopAndBottom | TextWrap::Square
            )
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(child)
            || child.cells.iter().any(|cell| {
                cell.dirty_flag
                    || cell.paragraphs.iter().any(|para| {
                        para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty
                    })
            })
        {
            return None;
        }
        let height = i32::try_from(table.common.height).ok().filter(|h| *h > 0)?;
        let child_height = i32::try_from(child.common.height).ok().filter(|h| *h > 0)?;
        let offset = signed_hwpunit(child.common.vertical_offset);
        let padding = cell.effective_padding(&table.padding);
        let bands = [
            padding.top,
            padding.bottom,
            child.outer_margin_top,
            child.outer_margin_bottom,
        ];
        if offset < 0 || bands.iter().any(|value| *value < 0) {
            return None;
        }
        let outer = i64::from(child_height)
            + i64::from(offset)
            + bands.iter().map(|value| i64::from(*value)).sum::<i64>();
        if outer != i64::from(height) {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let next = units.get(*end)?;
        if *end == 0 || !next.page_frame_reset_before || !next.hard_break_before {
            return None;
        }
        let split = self.nested_table_split_from_cut_units(cell, table, styles, 0, *end, 0)?;
        let cut = split.recursive_cut.as_ref()?;
        if cut.start_row != 0 || !cut.start_cut.is_empty() || split.terminal {
            return None;
        }
        let child_frame = self.nested_cut_physical_height(child, cut, styles);
        // 정수 원본을 px로 변환한 뒤의 누적 반올림만 허용한다.
        if !child_frame.is_finite()
            || (child_frame - hwpunit_to_px(child_height, self.dpi)).abs()
                > hwpunit_to_px(2, self.dpi)
        {
            return None;
        }
        Some(StoredWrapperOpeningFrame {
            height: hwpunit_to_px(height, self.dpi),
            child_top: hwpunit_to_px(
                i32::from(padding.top) + offset + i32::from(child.outer_margin_top),
                self.dpi,
            ),
        })
    }
}
