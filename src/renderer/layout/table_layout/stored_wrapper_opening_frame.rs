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
    /// 보이는 호스트 줄 뒤의 래퍼는 자식 저장 쪽 경계에서 바깥 위 여백을 다시 연다.
    /// 첫 조각의 선언 상자 또는 마지막 조각의 저장 빈 줄이 그 경계를 증언해야 한다.
    pub(crate) fn stored_nested_wrapper_cut_reopens_outer_top(
        &self,
        table: &Table,
        start_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let [cell] = table.cells.as_slice() else {
            return false;
        };
        let [host, _] = cell.paragraphs.as_slice() else {
            return false;
        };
        table.outer_margin_top > 0
            && table.common.flow_with_text
            && !table.common.allow_overlap
            && table.common.vert_rel_to == VertRelTo::Para
            && table.common.vert_align == VertAlign::Top
            && table.common.text_wrap == TextWrap::TopAndBottom
            && !host.text.trim().is_empty()
            && (self
                .stored_wrapper_opening_frame(table, &[], start_cut, styles)
                .is_some()
                || self
                    .stored_wrapper_closing_height(table, start_cut, styles)
                    .is_some())
    }

    pub(super) fn transparent_single_cell_wrapper(
        table: &Table,
        styles: &ResolvedStyleSet,
    ) -> Option<()> {
        let [cell] = table.cells.as_slice() else {
            return None;
        };
        let border = styles
            .border_styles
            .get(usize::from(cell.border_fill_id.checked_sub(1)?))?;
        let table_style = styles
            .border_styles
            .get(usize::from(table.border_fill_id.checked_sub(1)?))?;
        let has_fill = |style: &crate::renderer::style_resolver::ResolvedBorderStyle| {
            style.fill_color.is_some()
                || style.pattern.is_some()
                || style.gradient.is_some()
                || style.image_fill.is_some()
        };
        if has_fill(border)
            || has_fill(table_style)
            || !table.zones.is_empty()
            || border.break_cell_separate_line
            || border.diagonal_attr != 0
            || border.center_line != crate::model::style::CenterLine::None
            || border
                .borders
                .iter()
                .any(|line| line.line_type != crate::model::style::BorderLineType::None)
        {
            return None;
        }
        Some(())
    }

    /// 테두리 없는 래퍼를 닫을 때는 보이는 자식의 끝과 다음 저장 글줄을
    /// 대조한다. 첫 컷에서 확인한 래퍼 좌표로 같은 자식의 마지막 컷을 읽는다.
    pub(crate) fn stored_wrapper_terminal_visible_end(
        &self,
        table: &Table,
        start_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let opening = self.stored_wrapper_opening_frame(table, &[], start_cut, styles)?;
        Self::transparent_single_cell_wrapper(table, styles)?;
        let [cell] = table.cells.as_slice() else {
            return None;
        };
        let [host] = cell.paragraphs.as_slice() else {
            return None;
        };
        let [Control::Table(child)] = host.controls.as_slice() else {
            return None;
        };
        let [start] = start_cut else { return None };
        let units = self.cell_units(cell, table, styles);
        let split =
            self.nested_table_split_from_cut_units(cell, table, styles, *start, units.len(), 0)?;
        let cut = split.recursive_cut.as_ref()?;
        if !split.terminal || !cut.end_cut.is_empty() || cut.end_row != usize::from(child.row_count)
        {
            return None;
        }
        Some(opening.child_top + self.nested_cut_physical_height(child, cut, styles))
    }

    /// 보이는 호스트 글줄과 별도로, 자식 뒤의 저장 빈 줄도 닫힌 셀의 상자를 차지한다.
    /// 빈 줄 원점이 현재 조각의 실제 자식 끝과 일치할 때만 그 줄 상자를 더한다.
    pub(in crate::renderer::layout) fn stored_wrapper_closing_height(
        &self,
        table: &Table,
        start_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let [start] = start_cut else { return None };
        let [cell] = table.cells.as_slice() else {
            return None;
        };
        let [host, following] = cell.paragraphs.as_slice() else {
            return None;
        };
        let [Control::Table(child)] = host.controls.as_slice() else {
            return None;
        };
        let [line] = following.line_segs.as_slice() else {
            return None;
        };
        let profile = self.profile.get();
        if *start == 0
            || profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || table.row_count != 1
            || table.col_count != 1
            || table.caption.is_some()
            || !matches!(table.layout_page_break(), TablePageBreak::RowBreak)
            || table.common.treat_as_char
            || cell.dirty_flag
            || host.text.trim().is_empty()
            || host.stored_text_partition_is_dirty()
            || host.cell_format_vpos_dirty
            || !following.text.trim().is_empty()
            || !following.controls.is_empty()
            || following.stored_text_partition_is_dirty()
            || following.cell_format_vpos_dirty
            || following
                .source_line_seg_vertical_pos
                .as_ref()
                .is_some_and(|source| source.first() != Some(&line.vertical_pos))
            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || line.vertical_pos <= 0
            || line.line_height <= 0
            || child.caption.is_some()
            || child.common.treat_as_char
            || !child.common.flow_with_text
            || child.common.allow_overlap
            || child.common.vert_rel_to != VertRelTo::Para
            || child.common.vert_align != VertAlign::Top
            || child.common.text_wrap != TextWrap::TopAndBottom
        {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let split =
            self.nested_table_split_from_cut_units(cell, table, styles, *start, units.len(), 0)?;
        let cut = split.recursive_cut.as_ref()?;
        if !split.terminal
            || cut.is_block_split
            || cut.start_cut_is_block
            || !cut.end_cut.is_empty()
            || cut.end_row != usize::from(child.row_count)
            || !self.row_cut_ends_at_original_plain_text_reset(
                child,
                cut.start_row,
                &[],
                &cut.start_cut,
                styles,
            )
        {
            return None;
        }
        let child_height = self.nested_cut_physical_height(child, cut, styles);
        let child_outer_height = child_height
            + hwpunit_to_px(
                i32::from(child.outer_margin_top) + i32::from(child.outer_margin_bottom),
                self.dpi,
            );
        // 행 상자 합산의 부동소수점 오차가 원래 정수 좌표 하나를 절삭하지 않도록
        // 반 HWPUNIT 안에서 같은 저장 원점인지 확인한다.
        let source_origin = hwpunit_to_px(line.vertical_pos, self.dpi);
        let same_source_origin = child_outer_height.is_finite()
            && (child_outer_height - source_origin).abs() < hwpunit_to_px(1, self.dpi) * 0.5;
        if !same_source_origin {
            return None;
        }
        let padding = cell.effective_padding(&table.padding);
        let height = line
            .vertical_pos
            .checked_add(line.line_height)?
            .checked_add(i32::from(padding.top) + i32::from(padding.bottom))?;
        Some(hwpunit_to_px(height, self.dpi))
    }

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
        let para = match cell.paragraphs.as_slice() {
            [para] if para.text.trim().is_empty() => para,
            [para, following]
                if !para.text.trim().is_empty()
                    && following.text.trim().is_empty()
                    && following.controls.is_empty()
                    && !following.stored_text_partition_is_dirty()
                    && !following.cell_format_vpos_dirty
                    && matches!(following.line_segs.as_slice(), [line]
                        if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                            && i64::from(line.vertical_pos) >= i64::from(table.common.height)) =>
            {
                para
            }
            _ => return None,
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
            || table.layout_page_break() != TablePageBreak::RowBreak
            || cell.dirty_flag
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
        if offset < 0
            || (!para.text.trim().is_empty() && host.line_height > offset)
            || bands.iter().any(|value| *value < 0)
        {
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
        if *end == 0 {
            return None;
        }
        let split = self.nested_table_split_from_cut_units(cell, table, styles, 0, *end, 0)?;
        let cut = split.recursive_cut.as_ref()?;
        if cut.start_row != 0 || !cut.start_cut.is_empty() || split.terminal {
            return None;
        }
        // 글줄이 먼저 있는 호스트도 끝 컷이 자식 글줄의 원본 재시작을
        // 가리키면 같은 선언 상자를 닫는다.
        if !(next.page_frame_reset_before && next.hard_break_before)
            && (para.text.trim().is_empty()
                || !self.row_cut_ends_at_original_plain_text_reset(
                    child,
                    cut.end_row.checked_sub(1)?,
                    &[],
                    &cut.end_cut,
                    styles,
                ))
        {
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
