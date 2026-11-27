//! 이월된 그림 띠와 다음 글줄이 함께 소유한 저장 프레임의 원점.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::shape::{TextWrap, VertAlign, VertRelTo};
use crate::model::table::{Cell, TablePageBreak};
use crate::model::{control::Control, paragraph::LineSeg, paragraph::Paragraph, table::Table};
use crate::renderer::float_placement::{
    cell_no_overlap_square_stack, parallel_cell_picture_band_height, StoredEmptyOpeningRowFrame,
};
use crate::renderer::layout::picture_flow_frame_size_hu;
use crate::renderer::{hwpunit_to_px, px_to_hwpunit};

fn original_lines(para: &Paragraph) -> bool {
    !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && !para.line_segs.is_empty()
        && para.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos >= 0
                && line.line_height > 0
                && line.line_spacing >= 0
        })
}

impl LayoutEngine {
    /// 나란한 그림 띠가 이월될 때 첫 조각에는 저장 빈 줄과 셀 안여백이 남는다.
    /// 후속 조각에서 첫 행의 그림 유닛 전체를 수용할 높이도 함께 전달한다.
    pub(super) fn saved_parallel_picture_row_empty_opening_frame(
        &self,
        host: &Paragraph,
        successor: &Paragraph,
        table: &Table,
        first: &Cell,
    ) -> Option<StoredEmptyOpeningRowFrame> {
        if !self.profile.get().hwp5_stored_pagination_layout()
            || table.row_count < 2
            || table.caption.is_some()
            || first.is_header
            || host.line_segs.first()?.vertical_pos <= 0
            || successor.line_segs.first()?.vertical_pos != 0
        {
            return None;
        }
        let [anchor] = first.paragraphs.as_slice() else {
            return None;
        };
        let [line] = anchor.line_segs.as_slice() else {
            return None;
        };
        parallel_cell_picture_band_height(anchor, self.dpi)?;
        self.stored_empty_picture_cell_frame(first, table)?;
        let (_, _, top, bottom) = self.resolve_cell_padding(first, table);
        let opening_height = hwpunit_to_px(line.line_height, self.dpi) + top + bottom;
        if top < 0.0
            || bottom < 0.0
            || hwpunit_to_px(i32::try_from(first.height).ok()?, self.dpi) >= opening_height
        {
            return None;
        }
        // 이월된 그림들은 새 조각의 안여백에서 함께 시작한다. 원래 문단에
        // 붙은 개별 오프셋은 새 조각의 그림과 행 높이에 다시 반영하지 않는다.
        let picture_height_hu = anchor
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Picture(picture) => Some(picture_flow_frame_size_hu(picture).1),
                _ => None,
            })
            .max()?;
        let picture_band_height = hwpunit_to_px(picture_height_hu, self.dpi).max(
            cell_no_overlap_square_stack(&anchor.controls, self.dpi, true)
                .map_or(0.0, |stack| stack.bottom),
        );
        let continuation_height = picture_band_height + top + bottom;
        (opening_height < continuation_height).then_some(StoredEmptyOpeningRowFrame {
            opening_height,
            continuation_height,
            deferred_picture_band_height: Some(picture_band_height),
        })
    }

    /// 앞 조각에 빈 앵커 줄을 이미 수용한 경우, 새 조각의 첫 상자는
    /// 단독 그림이다. 앞 조각의 오프셋과 겹침 공제를 이 상자에 재적용하지 않는다.
    pub(super) fn stored_continued_square_picture(
        &self,
        cell: &Cell,
        table: &Table,
        units: &[CellUnit],
        start: usize,
    ) -> Option<(usize, f64)> {
        let first = units.get(start)?;
        let host = units.get(start.checked_sub(1)?)?;
        let profile = self.profile.get();
        if host.para_idx != first.para_idx
            || host.vis_start != 0
            || host.vis_end != 1
            || host.reflowed_line_slot
            || host.non_inline_control_range.is_some()
            || !Self::is_non_inline_control_flow_unit(first)
            || first.non_inline_control_range != Some((0, 0))
            || !profile.hwp5_stored_pagination_layout()
            || profile.session_edited()
            || table.common.treat_as_char
            || table.layout_page_break() != TablePageBreak::RowBreak
            || cell.dirty_flag
            || cell.row_span != 1
            || cell.col_span != table.col_count
            || self
                .render_normalization
                .borrow()
                .cell_frame_reflowed(table, cell)
        {
            return None;
        }
        let paragraph = cell.paragraphs.get(first.para_idx)?;
        let [Control::Picture(picture)] = paragraph.controls.as_slice() else {
            return None;
        };
        let [line] = paragraph.line_segs.as_slice() else {
            return None;
        };
        let common = &picture.common;
        if !paragraph.text.is_empty()
            || !original_lines(paragraph)
            || line.vertical_pos == 0
            || first.para_idx + 1 != cell.paragraphs.len()
            || common.treat_as_char
            || !common.flow_with_text
            || common.vert_rel_to != VertRelTo::Para
            || common.vert_align != VertAlign::Top
            || common.text_wrap != TextWrap::Square
            || (common.vertical_offset as i32) <= 0
            || common.margin.top != 0
            || picture.caption.is_some()
            || units[start..].iter().any(|unit| {
                unit.para_idx != first.para_idx
                    || unit.non_inline_control_range != Some((0, 0))
                    || !Self::is_non_inline_control_flow_unit(unit)
            })
        {
            return None;
        }
        let height = self.cell_non_inline_control_flow_height(common);
        let charged_advance =
            host.height + units[start..].iter().map(|unit| unit.height).sum::<f64>();
        // 저장 띠의 부족분이 앞 조각의 줄과 같아야 그 겹침만 복원할 수 있다.
        let roundoff = f64::EPSILON * (units.len() - start + 2) as f64 * height;
        (height > 0.0 && (charged_advance - height).abs() <= roundoff)
            .then_some((first.para_idx, height))
    }

    pub(in crate::renderer::layout) fn stored_continued_square_picture_cut(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        start: usize,
        end: usize,
    ) -> Option<usize> {
        if start == 0 {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        if end < units.len() {
            return None;
        }
        self.stored_continued_square_picture(cell, table, &units, start)
            .map(|(paragraph, _)| paragraph)
    }

    /// 앞 프레임의 줄 뒤에 다음 줄을 담을 수 없고, 양수 재시작 원점이 앞서
    /// 앵커된 그림의 전체 예약 띠와 같으면 그림과 다음 글줄은 새 프레임을 함께
    /// 소유한다. 문단 자체의 양수 로컬 원점이나 재조판 좌표는 이 근거가 아니다.
    /// 이 신호는 그림을 다음 본문 앞으로 이월하는 순서를 정하며, 그림과 본문
    /// 사이에서 다시 강제 분할하는 글줄 내부 경계로 승격하지 않는다.
    pub(super) fn stored_deferred_picture_frame_before(
        &self,
        cell: &Cell,
        table: &Table,
        index: usize,
        effective: &Paragraph,
    ) -> bool {
        let profile = self.profile.get();
        if !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || table.common.treat_as_char
            || table.layout_page_break() != TablePageBreak::RowBreak
            || cell.row_span != 1
            || cell.col_span != table.col_count
        {
            return false;
        }
        let Some(current) = cell.paragraphs.get(index) else {
            return false;
        };
        let Some(previous) = index.checked_sub(1).and_then(|i| cell.paragraphs.get(i)) else {
            return false;
        };
        if !original_lines(current)
            || !original_lines(previous)
            || current.line_segs != effective.line_segs
            || !current.controls.is_empty()
        {
            return false;
        }
        let first = &current.line_segs[0];
        let last = previous.line_segs.last().expect("원본 줄 확인");
        if first.vertical_pos <= 0 || first.vertical_pos >= last.vertical_pos {
            return false;
        }
        let (_, _, top, bottom) = self.resolve_cell_padding(cell, table);
        let closing = hwpunit_to_px(last.vertical_pos.saturating_add(last.line_height), self.dpi)
            + top
            + bottom;
        let next_advance = hwpunit_to_px(
            last.line_spacing.saturating_add(first.line_height),
            self.dpi,
        );
        let body = self.current_body_area.get().3;
        if body <= 0.0 || closing > body || closing + next_advance <= body {
            return false;
        }

        // 같은 저장 프레임의 문단만 거슬러 찾는다. 다른 물리 경계나 재조판
        // 문단을 넘어서 우연히 높이가 같은 그림을 원점의 소유자로 고르지 않는다.
        let mut later_origin = last.vertical_pos;
        for anchor in cell.paragraphs[..index].iter().rev() {
            if !original_lines(anchor) {
                return false;
            }
            let end = anchor.line_segs.last().expect("원본 줄 확인");
            if end.vertical_pos > later_origin {
                return false;
            }
            if let Some(band) =
                crate::renderer::float_placement::deferred_picture_band_height(anchor, self.dpi)
            {
                return (i64::from(first.vertical_pos) - i64::from(px_to_hwpunit(band, self.dpi)))
                    .abs()
                    <= 1;
            }
            if anchor
                .controls
                .iter()
                .any(|c| !matches!(c, Control::Field(_)))
            {
                return false;
            }
            later_origin = anchor.line_segs[0].vertical_pos;
        }
        false
    }
}
