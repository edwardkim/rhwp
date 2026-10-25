//! 원본 표의 선언 조각과 공유 글줄 좌표계가 소유하는 텍스트 프레임.

use super::{
    stored_plain_text_paragraph_frame_reset, stored_text_paragraphs_share_vpos_frame, LayoutEngine,
    ResolvedStyleSet,
};
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::table::{Cell, Table, TablePageBreak};

impl LayoutEngine {
    fn original_stored_table_frame_is_usable(&self, table: &Table, frame_reflowed: bool) -> bool {
        let profile = self.profile.get();
        (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            && !profile.session_edited()
            && !table.common.treat_as_char
            && table.page_break == TablePageBreak::RowBreak
            && table.common.height > 0
            && !frame_reflowed
    }

    pub(crate) fn native_stored_table_has_fragment_frames(&self, table: &Table) -> bool {
        self.profile.get().hwp5_stored_pagination_layout()
            && self.original_stored_table_frame_is_usable(
                table,
                self.render_normalization
                    .borrow()
                    .table_text_reflowed(table),
            )
            && Self::stored_row_boxes_exceed_object_frame(table)
    }

    /// HWP/HWPX의 원본 조각은 같은 저장 좌표·문단 간격 인증을 쓴다.
    /// 문단마다 0인 지역 원점만으로 쪽 경계를 만들지 않는다. 선언 행의 합이
    /// 원본 object보다 크거나, 원본 첫 글줄 조각이 object를 정확히 닫아야 한다.
    /// 셀 선언 높이는 패딩만 담을 수도 있으므로 완전한 내용 높이로 가정하지 않는다.
    pub(super) fn original_stored_text_frame_cell(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
    ) -> bool {
        cell.row_span == 1
            && !cell.dirty_flag
            && self.original_stored_table_frame_is_usable(
                table,
                self.render_normalization
                    .borrow()
                    .cell_frame_reflowed(table, cell),
            )
            && stored_text_paragraphs_share_vpos_frame(cell, styles, self.dpi)
            && (Self::stored_row_boxes_exceed_object_frame(table)
                || opening_text_frame_end_hu(cell, table, styles, self.dpi)
                    .is_some_and(|end| (end - i64::from(table.common.height)).abs() <= 2))
    }
}

fn source_paragraph(paragraph: &Paragraph) -> bool {
    paragraph.controls.is_empty()
        && !paragraph.stored_text_partition_is_dirty()
        && !paragraph.cell_format_vpos_dirty
        && !paragraph.line_segs.is_empty()
        && paragraph.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos >= 0
                && line.line_height > 0
        })
}

/// 선행 행은 원본 글줄·패딩·선언 최소 높이로만 계산한다. 되감김이나 개체가
/// 있는 행의 물리 높이를 현재 측정 결과로 추정해 인증에 넣지 않는다.
fn source_row_height_hu(cell: &Cell, table: &Table) -> Option<i64> {
    if cell.dirty_flag || cell.height > i32::MAX as u32 {
        return None;
    }
    let mut previous = None;
    let mut bottom = 0_i64;
    for paragraph in &cell.paragraphs {
        if !source_paragraph(paragraph) {
            return None;
        }
        for line in &paragraph.line_segs {
            if previous.is_some_and(|vpos| line.vertical_pos < vpos) {
                return None;
            }
            previous = Some(line.vertical_pos);
            bottom = bottom.max(i64::from(line.vertical_pos) + i64::from(line.line_height));
        }
    }
    previous?;
    let padding = cell.effective_padding(&table.padding);
    Some((bottom + i64::from(padding.top) + i64::from(padding.bottom)).max(i64::from(cell.height)))
}

/// 첫 문단 간 재시작 직전의 원본 잉크가 소유한 표 좌표계 끝점이다.
/// 뒤 조각의 개체는 첫 조각의 인증을 무효화하지 않는다. 인증 전에 만난 개체나
/// 합성 줄·문단 내부 되감김은 이 문단 간 텍스트 프레임의 증거로 쓰지 않는다.
fn opening_text_frame_end_hu(
    cell: &Cell,
    table: &Table,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<i64> {
    if table.common.height > i32::MAX as u32 {
        return None;
    }
    let mut previous: Option<&LineSeg> = None;
    let mut closing = None;
    'paragraphs: for (index, paragraph) in cell.paragraphs.iter().enumerate() {
        if !source_paragraph(paragraph) {
            return None;
        }
        for (line_index, line) in paragraph.line_segs.iter().enumerate() {
            if let Some(before) = previous {
                if line.vertical_pos < before.vertical_pos {
                    if line_index != 0
                        || index == 0
                        || !stored_plain_text_paragraph_frame_reset(
                            &cell.paragraphs[index - 1],
                            paragraph,
                            styles,
                            dpi,
                        )
                    {
                        return None;
                    }
                    closing = Some(before);
                    break 'paragraphs;
                }
            }
            previous = Some(line);
        }
    }
    let closing = closing?;
    let mut prefix = 0_i64;
    for row in 0..cell.row {
        // 앞 행을 가로지르는 rowspan은 이 독립 행 합으로 증명할 수 없다.
        if table.cells.iter().any(|peer| {
            usize::from(peer.row) <= usize::from(row)
                && usize::from(row) < usize::from(peer.row) + usize::from(peer.row_span)
                && peer.row_span != 1
        }) {
            return None;
        }
        let mut row_height = None;
        for peer in table.cells.iter().filter(|peer| peer.row == row) {
            let height = source_row_height_hu(peer, table)?;
            row_height = Some(row_height.unwrap_or(0_i64).max(height));
        }
        prefix += row_height? + i64::from(table.cell_spacing);
    }
    let padding = cell.effective_padding(&table.padding);
    Some(
        prefix
            + i64::from(closing.vertical_pos)
            + i64::from(closing.line_height)
            + i64::from(padding.top)
            + i64::from(padding.bottom),
    )
}
