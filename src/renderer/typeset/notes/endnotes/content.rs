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

pub(in crate::renderer::typeset) fn prepend_endnote_marker_text(
    para: &mut Paragraph,
    endnote: &crate::model::footnote::Endnote,
) {
    // 미주 번호는 렌더 시점에 가상 텍스트로 붙이므로, line_segs/char_shapes도
    // 같은 UTF-16 stream 기준으로 함께 밀어야 한다.
    let leading_spaces = para
        .text
        .chars()
        .take_while(|ch| matches!(*ch, ' ' | '\u{00A0}' | '\u{2007}'))
        .count();
    let marker_char_shape_id = para.char_shape_id_at(leading_spaces);
    if leading_spaces > 0 {
        para.delete_text_at(0, leading_spaces);
    }
    let prefix = format!("{} ", format_endnote_marker_text(endnote));
    let prefix_len = prefix.chars().count();
    para.insert_text_at(0, &prefix);
    if let Some(char_shape_id) = marker_char_shape_id {
        para.apply_char_shape_range(0, prefix_len, char_shape_id);
    }
}
