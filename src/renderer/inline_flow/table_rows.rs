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
                && table.layout_page_break() == crate::model::table::TablePageBreak::RowBreak)
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
    // [#8507] 텍스트 스트림이 빈 host 는 줄 재조판이 개체 줄을 한 번 정한다. 필드 시작 뒤로
    // 넘긴 표는 빈 필드 줄 + 표 줄의 두 줄이 되며, 그 줄 구성도 이 계획이 다시 나누지 않는다.
    // 작성 개행·공백만 있는 문단은 여러 줄이어도 기존처럼 이 계획이 줄을 소유한다.
    let single_composed_row = para.line_segs.len() == 1
        && para.line_segs[0].tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
            != 0
        && para.controls.iter().enumerate().any(|(control, _)| {
            super::super::composer::owned_rowbreak_tac_height(para, control).is_some()
        });
    let has_composed_object_row = flow_objects == 1
        && !has_text
        && (single_composed_row || super::super::para_has_composed_object_rows(para));
    // 본문 축만 새로 만든 경우 텍스트가 없는 문단의 저장 줄은 여전히
    // 유효한 개체 높이를 소유한다. 줄 하나가 단일 TAC 표의 외곽 상자를
    // 정확히 담을 때만 재구성된 축에서 같은 물리 줄 계획을 사용한다.
    let rebased_single_table_row = para.rebased_single_table_row();
    // 저장 글줄이 남아 있어도 문자 분할이 무효화됐으면 원래 줄 소속을
    // 재사용할 수 없다. 표와 텍스트를 같은 물리 줄 계획에서 다시 구성한다.
    Ok((super::super::para_has_no_stored_line_segs(para)
        || para.stored_text_partition_is_dirty()
        || rebased_single_table_row)
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
            // 머리말·꼬리말은 본문 밖 프레임이 소유한다. 이 정의가 같은
            // 문단에 있어도 저장 줄 없는 본문의 표·글줄 재조판을 막지 않는다.
            Control::Header(_) | Control::Footer(_) => true,
            c => super::super::composer::control_is_width_neutral_marker(c),
        }))
}

/// 절대 쪽·종이 기준 어울림 표의 앵커와 TAC 개체도 원본 control 순서의
/// 같은 글줄 구성에 참여한다. 저장 줄 하나라는 이유로 너비 부족이나
/// 작성 개행을 무시하지 않는다. 캡션·중첩 표·각주는 기존 분할 소유자에게 남긴다.
pub(crate) fn supports_page_square_table_rows(para: &Paragraph) -> bool {
    use crate::model::shape::{TextWrap, VertRelTo};
    let page_square = |table: &crate::model::table::Table| {
        !table.common.treat_as_char
            && table.common.text_wrap == TextWrap::Square
            && matches!(table.common.vert_rel_to, VertRelTo::Page | VertRelTo::Paper)
    };
    !para.text.contains('\t')
        && para
            .controls
            .iter()
            .any(|c| matches!(c, Control::Table(t) if page_square(t)))
        && para
            .controls
            .iter()
            .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
        && para.controls.iter().all(|c| match c {
            Control::Table(t) => {
                (t.common.treat_as_char || page_square(t))
                    && t.caption.is_none()
                    && !t.cells.iter().any(|cell| {
                        cell.paragraphs.iter().any(|p| {
                            p.controls
                                .iter()
                                .any(|c| matches!(c, Control::Footnote(_) | Control::Table(_)))
                        })
                    })
            }
            c => super::super::composer::control_is_width_neutral_marker(c),
        })
}
