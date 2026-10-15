//! 미주 참조 해석과 번호 문자 준비. 원본 IR은 변경하지 않는다.

use crate::renderer::typeset::{
    format_number, Control, EndnoteDeferral, EndnoteRef, Paragraph, RenderNumberFormat,
};

/// [미주 배치] `en_ref` 의 미주 본문(en_ctrl)을 해석한다. 현재 구역 미주면 본문
/// paragraphs 에서, END_OF_DOCUMENT 로 문서 끝에 모인 앞 구역 미주면 deferral
/// 목록에서 찾는다. (참조 표시 위첨자는 원 위치에 남고 본문만 여기서 렌더.)
pub(in crate::renderer::typeset) fn resolve_endnote_content<'a>(
    en_ref: &EndnoteRef,
    section_index: usize,
    paragraphs: &'a [Paragraph],
    deferral: &'a EndnoteDeferral<'a>,
) -> Option<&'a crate::model::footnote::Endnote> {
    if en_ref.section_index == section_index {
        match paragraphs
            .get(en_ref.para_index)
            .and_then(|p| p.controls.get(en_ref.control_index))
        {
            Some(Control::Endnote(en_ctrl)) => Some(en_ctrl.as_ref()),
            _ => None,
        }
    } else if let EndnoteDeferral::RenderAll(deferred) = deferral {
        deferred
            .iter()
            .find(|d| {
                d.reff.section_index == en_ref.section_index
                    && d.reff.para_index == en_ref.para_index
                    && d.reff.control_index == en_ref.control_index
            })
            .map(|d| &d.endnote)
    } else {
        None
    }
}

pub(in crate::renderer::typeset) fn note_number_format_from_hwp_code(
    code: u8,
) -> RenderNumberFormat {
    match code {
        0 => RenderNumberFormat::Digit,
        1 => RenderNumberFormat::CircledDigit,
        2 => RenderNumberFormat::RomanUpper,
        3 => RenderNumberFormat::RomanLower,
        4 => RenderNumberFormat::LatinUpper,
        5 => RenderNumberFormat::LatinLower,
        8 => RenderNumberFormat::HangulGaNaDa,
        12 => RenderNumberFormat::HangulNumber,
        13 => RenderNumberFormat::HanjaNumber,
        _ => RenderNumberFormat::Digit,
    }
}

pub(in crate::renderer::typeset) fn note_decoration_char(value: u16) -> Option<char> {
    if value == 0 {
        None
    } else {
        char::from_u32(value as u32).filter(|ch| *ch != '\0')
    }
}

pub(in crate::renderer::typeset) fn format_endnote_marker_text(
    endnote: &crate::model::footnote::Endnote,
) -> String {
    let number = format_number(
        endnote.number,
        note_number_format_from_hwp_code(endnote.number_shape as u8),
    );
    let prefix = note_decoration_char(endnote.before_decoration_letter)
        .map(|ch| ch.to_string())
        .unwrap_or_default();
    let suffix = note_decoration_char(endnote.after_decoration_letter)
        .unwrap_or(')')
        .to_string();
    format!("{}{}{}", prefix, number, suffix)
}

/// 미주 첫 문단을 그릴 때 번호 앞에서 지우는 앞 공백 글자 수.
fn endnote_leading_space_count(para: &Paragraph) -> usize {
    para.text
        .chars()
        .take_while(|ch| matches!(*ch, ' ' | '\u{00A0}' | '\u{2007}'))
        .count()
}

/// 미주 첫 문단 앞에 붙여 그리는 번호 글자(`1) `).
fn endnote_marker_prefix(endnote: &crate::model::footnote::Endnote) -> String {
    format!("{} ", format_endnote_marker_text(endnote))
}

/// 미주 첫 문단의 글자 위치를 번호를 붙여 그린 문단의 글자 위치로 바꾼다.
/// 지운 앞 공백 안의 위치는 번호 바로 뒤로 모은다.
pub(crate) fn endnote_first_para_render_offset(
    endnote: &crate::model::footnote::Endnote,
    char_offset: usize,
) -> usize {
    let leading_spaces = endnote
        .paragraphs
        .first()
        .map_or(0, endnote_leading_space_count);
    char_offset.saturating_sub(leading_spaces) + endnote_marker_prefix(endnote).chars().count()
}

pub(in crate::renderer::typeset) fn prepend_endnote_marker_text(
    para: &mut Paragraph,
    endnote: &crate::model::footnote::Endnote,
) {
    // 미주 번호는 렌더 시점에 가상 텍스트로 붙이므로, line_segs/char_shapes도
    // 같은 UTF-16 stream 기준으로 함께 밀어야 한다.
    let leading_spaces = endnote_leading_space_count(para);
    // 자동번호는 뒤 본문과 다른 글자 모양을 가질 수 있다. 표시 문자의
    // 위치가 아니라 번호가 소유한 원본 UTF-16 슬롯에서 서식을 읽는다.
    let marker_char_shape_id = para
        .controls
        .iter()
        .enumerate()
        .find(|(index, control)| {
            matches!(control, Control::AutoNumber(number)
                if number.number_type == crate::model::control::AutoNumberType::Endnote)
                && para
                    .control_text_positions()
                    .get(*index)
                    .is_some_and(|position| *position <= leading_spaces)
        })
        .and_then(|(index, _)| para.control_utf16_positions().get(index).copied())
        .and_then(|position| {
            para.char_shapes
                .iter()
                .rev()
                .find(|shape| shape.start_pos <= position)
                .map(|shape| shape.char_shape_id)
        })
        .or_else(|| para.char_shape_id_at(leading_spaces));
    let separator_char_shape_id = if leading_spaces > 0 {
        para.char_shape_id_at(0)
    } else {
        marker_char_shape_id
    };
    if leading_spaces > 0 {
        para.delete_text_at(0, leading_spaces);
    }
    let prefix = endnote_marker_prefix(endnote);
    let prefix_len = prefix.chars().count();
    para.insert_text_at(0, &prefix);
    if let Some(char_shape_id) = marker_char_shape_id {
        para.apply_char_shape_range(0, prefix_len - 1, char_shape_id);
    }
    if let Some(char_shape_id) = separator_char_shape_id {
        para.apply_char_shape_range(prefix_len - 1, prefix_len, char_shape_id);
    }
}
