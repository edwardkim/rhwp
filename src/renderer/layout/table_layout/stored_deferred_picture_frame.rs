//! 이월된 그림 띠와 다음 글줄이 함께 소유한 저장 프레임의 원점.

use super::LayoutEngine;
use crate::model::table::{Cell, TablePageBreak};
use crate::model::{control::Control, paragraph::LineSeg, paragraph::Paragraph, table::Table};
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
            || table.page_break != TablePageBreak::RowBreak
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
