//! 저장 줄이 없는 TAC 표의 물리 줄 계획 지원 범위.

use crate::model::{control::Control, paragraph::Paragraph};

/// 현재 단에 전체 계획이 들어가지 않으면 기존 행 분할 소유자가 처리한다.
pub(crate) fn standalone_table_fragment_owner(para: &Paragraph) -> bool {
    match standalone_table_fragment_owner_with_storage(para, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn standalone_table_fragment_owner_with_storage<S: crate::storage::Storage>(
    para: &Paragraph,
    storage: &S,
) -> Result<bool, S::Error> {
    let flow_objects = para
        .controls
        .iter()
        .filter(|c| matches!(c, Control::Table(_)))
        .count();
    let has_text =
        super::super::composer::expand_pua_display_text_with_storage(&para.text, storage)?
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace());
    Ok(flow_objects == 1
        && !has_text
        && para.controls.iter().any(|control| {
            matches!(control, Control::Table(table)
                if table.common.treat_as_char
                && table.row_count > 1
                && table.page_break == crate::model::table::TablePageBreak::RowBreak)
        }))
}

/// Reflowed TAC rows also own the text preceding, separating and following
/// their objects. Stored rows and controls with other owners retain that path.
pub(crate) fn supports_table_text_rows(para: &Paragraph) -> bool {
    match supports_table_text_rows_with_storage(para, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn supports_table_text_rows_with_storage<S: crate::storage::Storage>(
    para: &Paragraph,
    storage: &S,
) -> Result<bool, S::Error> {
    // 현재 단에 들어가는 전체 물리 줄은 같은 계획으로 측정·배치한다.
    // 분할이 필요하면 호출자가 기존 행 분할 소유자에게 반환한다.
    // 이미 구성한 합성 개체 줄의 소유는 유지한다.
    let flow_objects = para
        .controls
        .iter()
        .filter(|control| matches!(control, Control::Table(_)))
        .count();
    let has_text =
        super::super::composer::expand_pua_display_text_with_storage(&para.text, storage)?
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace());
    let has_composed_object_row = flow_objects == 1
        && !has_text
        && para.line_segs.len() == 1
        && para.line_segs[0].tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
            != 0
        && para.controls.iter().enumerate().any(|(control, _)| {
            super::super::composer::owned_rowbreak_tac_height(para, control).is_some()
        });
    Ok(super::super::para_has_no_stored_line_segs(para)
        && !has_composed_object_row
        && !para.text.contains('\t')
        && para
            .controls
            .iter()
            .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
        && para.controls.iter().all(|c| match c {
            Control::Table(t) => {
                // Captions and cell notes have their own placement/reservation owner.
                t.caption.is_none()
                    && !t.cells.iter().any(|cell| {
                        cell.paragraphs.iter().any(|p| {
                            p.controls
                                .iter()
                                .any(|c| matches!(c, Control::Footnote(_) | Control::Table(_)))
                        })
                    })
                    && (t.common.treat_as_char
                        || (matches!(
                            t.common.text_wrap,
                            crate::model::shape::TextWrap::InFrontOfText
                                | crate::model::shape::TextWrap::BehindText
                        ) && matches!(
                            t.common.vert_rel_to,
                            crate::model::shape::VertRelTo::Para
                        ) && matches!(
                            t.common.vert_align,
                            crate::model::shape::VertAlign::Top
                                | crate::model::shape::VertAlign::Inside
                        ) && matches!(
                            t.common.horz_rel_to,
                            crate::model::shape::HorzRelTo::Column
                                | crate::model::shape::HorzRelTo::Para
                        )))
            }
            c => super::super::composer::control_is_width_neutral_marker(c),
        }))
}
