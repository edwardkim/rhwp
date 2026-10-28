//! 실제 줄 메트릭의 변경을 같은 저장 단 조각의 후속 원점에 투영한다.

use crate::model::footnote::Endnote;
use crate::renderer::style_resolver::ResolvedStyleSet;
use std::borrow::Cow;

pub(super) fn project_equation_rows<'a>(
    original: &'a Endnote,
    styles: &ResolvedStyleSet,
    st: &mut crate::renderer::typeset::TypesetState,
) -> Cow<'a, Endnote> {
    let stored_hwpx_columns =
        st.profile.hwpx_stored_layout() && !st.profile.session_edited() && st.col_count > 1;
    let mut normalized = Cow::Borrowed(original);
    if stored_hwpx_columns {
        for (paragraph_index, paragraph) in original.paragraphs.iter().enumerate() {
            for (index, rows) in paragraph.line_segs.windows(3).enumerate() {
                let before = &rows[0];
                let previous = &rows[1];
                let current = &rows[2];
                let before_advance = before.line_height.checked_add(before.line_spacing);
                let previous_advance = previous.line_height.checked_add(previous.line_spacing);
                // 다음 줄은 새 단 원점에서 직전 줄의 전진만큼 내려간다.
                // 직전 줄의 0 원점만 누락된 저장본은 뒤 줄의 되감김에서
                // 자르면 한 줄을 앞 단에 남긴다. 원문 대신 투영 사본을 고친다.
                if before_advance.is_some_and(|advance| {
                    advance > 0
                        && before.vertical_pos.checked_add(advance) == Some(previous.vertical_pos)
                }) && previous_advance.is_some_and(|advance| {
                    advance > 0
                        && current.vertical_pos == advance
                        && current.vertical_pos < previous.vertical_pos
                }) {
                    normalized.to_mut().paragraphs[paragraph_index].line_segs[index + 1]
                        .vertical_pos = 0;
                }
            }
        }
    }
    match crate::renderer::composer::stored_row_projection::project_equation_rows(
        &normalized.paragraphs,
        styles,
    ) {
        Some(paragraphs) => {
            let first_local_index = st.endnote_paragraphs.len();
            for (index, (source, derived)) in
                normalized.paragraphs.iter().zip(&paragraphs).enumerate()
            {
                // 원래 폭·문자 분할은 유지한다. 실제 높이로 재발행한 세로 좌표만
                // 과거 단 경계의 근거로 사용하지 않는다.
                if source
                    .line_segs
                    .iter()
                    .zip(&derived.line_segs)
                    .any(|(before, after)| {
                        before.vertical_pos != after.vertical_pos
                            || before.line_height != after.line_height
                            || before.line_spacing != after.line_spacing
                    })
                {
                    // 물리 상자 증가가 같은 양의 leading을 소비했을 때만
                    // 뒤 미주 간격의 사용량으로 전달한다. 실제 전진 증가나
                    // 다른 줄 소속을 이 값으로 상쇄하지 않는다.
                    let consumed = source
                        .line_segs
                        .last()
                        .zip(derived.line_segs.last())
                        .filter(|(before, after)| {
                            before.text_start == after.text_start
                                && before
                                    .line_height
                                    .checked_add(before.line_spacing)
                                    .is_some_and(|advance| {
                                        after.line_height.checked_add(after.line_spacing)
                                            == Some(advance)
                                    })
                                && after.line_height >= before.line_height
                        })
                        .map_or(0, |(before, after)| {
                            after.line_height.saturating_sub(before.line_height)
                        });
                    st.record_reissued_endnote_rows(first_local_index + index, consumed);
                }
            }
            let mut derived = normalized.into_owned();
            derived.paragraphs = paragraphs;
            Cow::Owned(derived)
        }
        None => normalized,
    }
}

/// 실제 emit과 fit 재생이 같은 미주 시작점·간격 입력을 사용한다.
pub(super) fn render_paragraph(
    original: &crate::model::paragraph::Paragraph,
    endnote_start: i32,
    compact_spacing: bool,
) -> crate::model::paragraph::Paragraph {
    let mut paragraph = original.clone();
    for line in &mut paragraph.line_segs {
        if compact_spacing {
            line.line_spacing = -(line.line_height / 10);
        }
        line.vertical_pos = line.vertical_pos.saturating_add(endnote_start);
    }
    paragraph
}
