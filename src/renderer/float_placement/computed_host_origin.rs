//! 같은 앵커의 표 하나가 전체 호스트 글줄의 원점을 인증할 수는 없다.

use crate::model::table::Table;
use crate::model::{control::Control, paragraph::Paragraph};

use super::{ParagraphFloatFlow, ParagraphFloatPlacement, ParagraphHostLine};

impl ParagraphFloatPlacement {
    /// 표 본체의 위끝이 아닌 원문 위 바깥여백까지 포함한 점유 위끝이다.
    pub(crate) fn exclusion_top(self, table: &Table, dpi: f64) -> f64 {
        self.table_top - super::hwpunit_to_px(i32::from(table.outer_margin_top), dpi)
    }
}

pub(super) fn shares_float_anchor(para: &Paragraph, char_pos: usize) -> bool {
    let positions = para.control_text_positions();
    para.controls
        .iter()
        .zip(positions)
        .filter(|(control, position)| {
            *position == char_pos
                && matches!(control, Control::Table(table)
                    if super::is_para_topbottom_float(&table.common))
        })
        .take(2)
        .count()
        > 1
}

/// 같은 계산 글줄을 막는 형제 표는 본체와 바깥여백으로만 흐름을 전진시킨다.
/// 별도 호스트 글줄의 줄간격은 형제 표 사이에 반복 예약하지 않는다.
pub(super) fn stacked_frame(
    para: &Paragraph,
    table: &Table,
    char_pos: usize,
    origin: f64,
    lines: &[ParagraphHostLine],
    height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    if lines.len() != 1 || table.common.allow_overlap || !shares_float_anchor(para, char_pos) {
        return None;
    }
    let top = origin + super::hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let bottom = top + height + super::hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    (top.is_finite() && bottom.is_finite()).then_some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::CoanchoredTable,
        anchor_y: origin,
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top: top,
        occupied_bottom: bottom,
    })
}
