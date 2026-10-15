//! 원본의 독립된 TAC 개체 줄에서 실제 펜과 점유 상자를 조회한다.

use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    table::Table,
};
use crate::renderer::{
    composer::ComposedParagraph, hwpunit_to_px, style_resolver::ResolvedStyleSet,
};

pub(in crate::renderer::typeset) struct OwnedRow {
    pub horizontal_pen: f64,
}

/// 줄을 앞 글자의 논리 위치로 압축하지 않고 제어 슬롯의 UTF-16 소속을 사용한다.
/// 같은 줄의 여러 개체나 수정된 줄은 기존 일반 구성 경로가 처리한다.
pub(in crate::renderer::typeset) fn prepare(
    para: &Paragraph,
    table: &Table,
    control: usize,
    composed: Option<&ComposedParagraph>,
    measured_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<OwnedRow> {
    if !table.common.treat_as_char
        || table.caption.is_some()
        || !para.text.chars().all(char::is_whitespace)
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || table.common.height > i32::MAX as u32
    {
        return None;
    }
    let owner = crate::renderer::composer::control_slot_line_index(para, control)?;
    let line = para.line_segs.get(owner)?;
    let height = i64::from(table.common.height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || line.line_spacing < 0
        || i64::from(line.line_height) != height
        || i64::from(line.text_height) != height
        || line.column_start < 0
        || line.segment_width <= 0
        || (measured_height - hwpunit_to_px(table.common.height as i32, dpi)).abs() > dpi / 7200.0
        || para.controls.iter().enumerate().any(|(ci, c)| {
            ci != control
                && matches!(
                    c,
                    Control::Table(_)
                        | Control::Picture(_)
                        | Control::Shape(_)
                        | Control::Equation(_)
                )
                && crate::renderer::composer::control_slot_line_index(para, ci) == Some(owner)
        })
    {
        return None;
    }
    let composed = composed?;
    if composed.lines.len() != para.line_segs.len() {
        return None;
    }
    let position = crate::renderer::composer::find_render_inline_control_positions(para)
        .get(control)
        .copied();
    let leading = crate::renderer::layout::compute_tac_leading_width(
        composed,
        control,
        styles,
        position,
        Some(owner),
    );
    let width = hwpunit_to_px(table.common.width.min(i32::MAX as u32) as i32, dpi)
        + hwpunit_to_px(
            i32::from(table.outer_margin_left) + i32::from(table.outer_margin_right),
            dpi,
        );
    let left = hwpunit_to_px(line.column_start, dpi);
    let available = hwpunit_to_px(line.segment_width, dpi);
    let para_style = styles.para_styles.get(para.para_shape_id as usize)?;
    // 저장 개체 줄도 본문과 같은 줄별 들여쓰기 계약을 소비한다.
    // 첫 줄의 공백 펜을 뒤 줄에 재사용하지 않고 그 줄의 내어쓰기를 복원한다.
    let indent = crate::renderer::equation_tac_flow::paragraph_line_indent_for_source(
        para_style.indent,
        owner,
        Some(para),
        composed.lines.len(),
        true,
    );
    let left = left + indent;
    let available = available - indent;
    let alignment = para_style.alignment;
    let horizontal_pen = match alignment {
        crate::model::style::Alignment::Center => {
            left + (available - leading - width) / 2.0 + leading
        }
        crate::model::style::Alignment::Right => left + available - width,
        _ => left + leading,
    };
    Some(OwnedRow { horizontal_pen })
}
