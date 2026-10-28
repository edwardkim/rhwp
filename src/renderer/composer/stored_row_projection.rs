//! 재발행한 수식 줄 메트릭을 같은 저장 단 조각의 후속 원점까지 전달한다.

use crate::model::paragraph::Paragraph;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 이미 구성한 실제 줄 결과가 저장 수식 줄의 전진을 바꿨는지 확인한다.
/// 중첩 미주 소유나 폭 재조판의 수용 조건으로 세로 메트릭 갱신을 추측하지 않는다.
pub(crate) fn reissued_equation_rows(
    original: &Paragraph,
    composed: &super::ComposedParagraph,
) -> bool {
    !original.stored_text_partition_is_dirty()
        && original.controls.iter().any(|control| {
            matches!(control, crate::model::control::Control::Equation(equation)
                if equation.common.treat_as_char)
        })
        && composed.lines.len() == original.line_segs.len()
        && original.line_segs.iter().all(|line| {
            line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.line_height > 0
        })
        && original
            .line_segs
            .iter()
            .zip(&composed.lines)
            .any(|(source, actual)| {
                source.line_height != actual.line_height
                    || source.line_spacing != actual.line_spacing
            })
}

pub(crate) fn project_equation_rows(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
) -> Option<Vec<Paragraph>> {
    match project_equation_rows_with_storage(original, styles, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn project_equation_rows_with_storage<S: crate::storage::Storage>(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    crate::storage::with_optional(|failure| {
        let mut derived = crate::storage::optional(
            storage.try_collect(
                original
                    .iter()
                    .map(|paragraph| storage.clone_paragraph(paragraph)),
            ),
            failure,
        )?;
        let mut shift = 0_i32;
        let mut previous_source_vpos = None;
        let mut changed = false;
        for paragraph in &mut derived {
            let composed = crate::storage::optional(
                crate::renderer::composer::compose_paragraph_in_context_with_storage(
                    paragraph, styles, storage,
                ),
                failure,
            )?;
            // 줄 소유·분할이 달라진 경로는 이 좌표 투영의 계약이 아니다.
            // 같은 저장 줄에 실제 상자 메트릭을 재발행한 경우만 소비한다.
            if composed.lines.len() != paragraph.line_segs.len() {
                return None;
            }
            let source_positions: Vec<_> = crate::storage::optional(
                storage.collect(paragraph.line_segs.iter().map(|line| line.vertical_pos)),
                failure,
            )?;
            for (index, (source, actual)) in paragraph
                .line_segs
                .iter_mut()
                .zip(&composed.lines)
                .enumerate()
            {
                let source_vpos = source.vertical_pos;
                if previous_source_vpos.is_some_and(|previous| source_vpos < previous) {
                    // 저장 단/쪽 되감김은 새 좌표 원점이다. 앞 조각의 재조판량을
                    // 새 단 원점까지 누적하거나 명시적 나눔을 지우지 않는다.
                    shift = 0;
                }
                previous_source_vpos = Some(source_vpos);
                let projected_vpos = source_vpos.saturating_add(shift);
                changed |= projected_vpos != source_vpos;
                source.vertical_pos = projected_vpos;
                if actual.line_height != source.line_height
                    || actual.baseline_distance != source.baseline_distance
                    || actual.line_spacing != source.line_spacing
                {
                    // 저장 선언 높이에 뒤 줄 개체가 포함됐어도 앞줄의 전진은
                    // 저장 사다리가 가리킨다. 원래 내용 높이+간격으로 이미 전진한
                    // 줄은 그 높이를 기준으로 재조판 차이를 구한다. 선언 높이의
                    // 차이를 빼면 중복 예약분까지 다시 당겨 다음 줄이 겹친다.
                    let source_advance_height = if source.line_height > source.text_height
                        && source_positions.get(index + 1).is_some_and(|next| {
                            source_vpos
                                .checked_add(source.text_height)
                                .and_then(|end| end.checked_add(source.line_spacing))
                                == Some(*next)
                        }) {
                        source.text_height
                    } else {
                        source.line_height
                    };
                    let height_delta = actual.line_height.saturating_sub(source_advance_height);
                    let spacing_delta = actual.line_spacing.saturating_sub(source.line_spacing);
                    // 저장 사다리가 이미 실제 줄의 전진값이면 뒤 원점을 다시 당기지
                    // 않는다. 낡은 상자의 전진을 저장한 경우만 재조판량을 투영한다.
                    let already_projected = source_positions.get(index + 1).is_some_and(|next| {
                        source_vpos
                            .checked_add(actual.line_height)
                            .and_then(|end| end.checked_add(actual.line_spacing))
                            == Some(*next)
                    });
                    if !already_projected {
                        shift = shift
                            .saturating_add(height_delta)
                            .saturating_add(spacing_delta);
                    }
                    source.line_height = actual.line_height;
                    source.text_height = actual.line_height;
                    source.baseline_distance = actual.baseline_distance;
                    source.line_spacing = actual.line_spacing;
                    changed = true;
                }
            }
        }
        if changed {
            Some(derived)
        } else {
            None
        }
    })
}
