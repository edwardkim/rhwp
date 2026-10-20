//! 재조판 흐름과 저장 축 재시작에서 실제로 소비할 문단 간격.
//! 합성 줄의 vpos는 앞 간격을 버릴 근거가 아니며, 부분 문단은 첫 조각만 소비한다.

use crate::model::paragraph::Paragraph;
use crate::renderer::layout::ParagraphVerticalSpacing;
use crate::renderer::style_resolver::ResolvedParaStyle;
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(crate) fn plain_text_spacing(
    para: &Paragraph,
    style: Option<&ResolvedParaStyle>,
    start_line: usize,
) -> Option<ParagraphVerticalSpacing> {
    if !para.controls.is_empty() || !crate::renderer::para_has_no_stored_line_segs(para) {
        return None;
    }
    Some(ParagraphVerticalSpacing {
        before: if start_line == 0 {
            style.map_or(0.0, |s| s.spacing_before)
        } else {
            0.0
        },
        after: style.map_or(0.0, |s| s.spacing_after),
    })
}

/// 첫 저장 줄이 옛 축에 남고 다음 줄이 새 쪽 축에서 시작한 경우의 간격.
/// 호출자가 두 문단의 실제 단 소유와 첫 항목 여부를 확인한다.
pub(crate) fn stored_axis_restart_spacing(
    first: &Paragraph,
    next: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<ParagraphVerticalSpacing> {
    let valid = |para: &Paragraph| {
        !para.stored_text_partition_is_dirty()
            && para.controls.is_empty()
            && !para.line_segs.is_empty()
            && para.line_segs.iter().all(|line| {
                line.vertical_pos >= 0
                    && line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
    };
    if first.line_segs.len() != 1 || !valid(first) || !valid(next) {
        return None;
    }
    let first_style = styles.para_styles.get(first.para_shape_id as usize)?;
    let next_style = styles.para_styles.get(next.para_shape_id as usize)?;
    let first_line = &first.line_segs[0];
    let next_line = &next.line_segs[0];
    let expected_next = crate::renderer::px_to_hwpunit(first_style.spacing_before, dpi)
        .saturating_add(first_line.line_height)
        .saturating_add(first_line.line_spacing)
        .saturating_add(crate::renderer::px_to_hwpunit(
            next_style.spacing_before,
            dpi,
        ));
    (first_style.spacing_before > 0.0
        && first_line.vertical_pos > next_line.vertical_pos
        && next_line.vertical_pos == expected_next)
        .then_some(ParagraphVerticalSpacing {
            before: first_style.spacing_before,
            after: first_style.spacing_after,
        })
}
