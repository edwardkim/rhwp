//! 미주 참조 해석과 번호 문자 준비. 원본 IR은 변경하지 않는다.

use crate::renderer::typeset::{Control, EndnoteDeferral, EndnoteRef, Paragraph};

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
        en_ref.resolve(paragraphs)
    } else if let EndnoteDeferral::RenderAll(deferred) = deferral {
        deferred
            .iter()
            .find(|d| d.reff.same_note(en_ref))
            .map(|d| &d.endnote)
    } else {
        None
    }
}

pub(in crate::renderer::typeset) fn format_endnote_marker_text(
    endnote: &crate::model::footnote::Endnote,
) -> String {
    crate::renderer::note_marker::text(
        &endnote.paragraphs,
        crate::model::control::AutoNumberType::Endnote,
        endnote.number,
        endnote.number_shape,
        endnote.before_decoration_letter,
        endnote.after_decoration_letter,
    )
}

/// 미주 첫 문단을 그릴 때 번호 앞에서 지우는 앞 공백 글자 수.
fn endnote_leading_space_count(para: &Paragraph) -> usize {
    if let Some((index, _)) = crate::renderer::note_marker::leading(
        std::slice::from_ref(para),
        crate::model::control::AutoNumberType::Endnote,
    ) {
        // 저장 자동 번호의 자리표만 치환하고 뒤의 실제 공백은 보존한다.
        let positions = para.control_utf16_positions();
        return usize::from(
            para.text.starts_with(' ') && para.char_offsets.first() == positions.get(index),
        );
    }
    para.text
        .chars()
        .take_while(|ch| matches!(*ch, ' ' | '\u{00A0}' | '\u{2007}'))
        .count()
}

/// 미주 첫 문단 앞에 붙여 그리는 번호 글자(`1) `).
fn endnote_marker_prefix(endnote: &crate::model::footnote::Endnote) -> String {
    let mut prefix = format_endnote_marker_text(endnote);
    if crate::renderer::note_marker::leading(
        &endnote.paragraphs,
        crate::model::control::AutoNumberType::Endnote,
    )
    .is_none()
    {
        prefix.push(' ');
    }
    prefix
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
    let has_separator = crate::renderer::note_marker::leading(
        &endnote.paragraphs,
        crate::model::control::AutoNumberType::Endnote,
    )
    .is_none();
    para.insert_text_at(0, &prefix);
    if let Some(char_shape_id) = marker_char_shape_id {
        para.apply_char_shape_range(0, prefix_len - usize::from(has_separator), char_shape_id);
    }
    if let Some(char_shape_id) = separator_char_shape_id.filter(|_| has_separator) {
        para.apply_char_shape_range(prefix_len - 1, prefix_len, char_shape_id);
    }
}
