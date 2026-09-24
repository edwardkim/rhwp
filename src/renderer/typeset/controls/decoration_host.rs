//! 데코레이션 표 host의 텍스트 소유 여부와 줄 전진량 조회.
//! 확정 항목/높이만 반환하며 페이지 상태와 원본 IR은 변경하지 않는다.
use super::super::paragraph::{metrics::FormattedParagraph, placement::ParagraphFragment};
use super::super::{para_has_non_whitespace_text, para_has_visible_text};
use crate::model::paragraph::Paragraph;
use crate::renderer::{hwpunit_to_px, pagination::PageItem, style_resolver::ResolvedStyleSet};

#[allow(clippy::too_many_arguments)]
pub(super) fn plan(
    para_idx: usize,
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    fmt: &FormattedParagraph,
    decoration_host_text_pending: bool,
    flow_table_owns_host_text: bool,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_layout: impl FnOnce() -> bool,
) -> Option<ParagraphFragment> {
    if decoration_host_text_pending
        && !flow_table_owns_host_text
        && para_has_non_whitespace_text(para)
    {
        let total_lines = fmt.line_heights.len();
        if total_lines > 0 {
            return Some(ParagraphFragment {
                item: PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: 0,
                    end_line: total_lines,
                },
                height: fmt.line_advances_sum(0..total_lines),
            });
        }
    } else if decoration_host_text_pending && !flow_table_owns_host_text {
        // [#7047] **글자 없는** 데코레이션 표 host 도 자기 줄을 흐름에 낸다.
        //
        // `#703` 단축은 표만 방출하고 흐름을 0 소비한다. 위 블록이 가시 텍스트를
        // 가진 host 를 보완했지만, 글자가 없는 host 는 어떤 `PageItem` 도 남기지
        // 않는다. 그러면 렌더가 그 문단을 건너뛰어 흐름이 한 픽셀도 전진하지 않고,
        // 항목이 없으니 저장 vpos 보정조차 받지 못해 lazy base 가 결손을 그대로
        // 세탁한다(임대차계약서양식 3쪽 pi=57 기준 1788 → pi=58 기준 2538,
        // 정확히 750 HWPUNIT = 줄높이 450 + 다음 앞간격 300). 그 결과 뒤따르는
        // 개체가 host 줄만큼 위에 놓인다 — 11.8px · 23.9px.
        //
        // 저장 사다리가 `줄높이 + 줄간격 + host 뒤간격 + 다음 앞간격` 을
        // **1 HWPUNIT 안에서** 증언할 때만 낸다. `#703` 의 0 소비가 옳은
        // 문서(사다리 델타가 0 이거나 줄간격뿐인 데코레이션 앵커)는 등식이 깨져
        // 종전 경로가 그대로 남는다 — 광역 규칙이 아닌 문단 단위 자기 게이트다.
        if let Some(advance_hu) =
            stored_decoration_host_line_advance_hu(stored_layout(), para, next_para, styles, dpi)
        {
            return Some(ParagraphFragment {
                item: PageItem::FullParagraph {
                    para_index: para_idx,
                },
                height: hwpunit_to_px(advance_hu, dpi),
            });
        }
    }
    None
}

/// [#7047] 글자 없는 데코레이션(글앞/글뒤) 표 host 문단이 저장 사다리에서 차지한
/// 줄 전진량(HWPUNIT).
///
/// `#703` 단축이 흐름을 0 소비하는 바람에 이 문단은 `PageItem` 이 하나도 없고, 렌더도
/// 저장 vpos 보정도 그 문단을 보지 못한다. 사다리가 `줄높이 + 줄간격 + host 뒤간격 +
/// 다음 앞간격` 을 1 HWPUNIT 안에서 증언하면 그 값이 한/글의 전진량이다.
fn stored_decoration_host_line_advance_hu(
    stored_layout: bool,
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<i32> {
    if !stored_layout || para_has_visible_text(para) {
        return None;
    }
    let next = next_para?;
    let synth = crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    // 합성 사다리(재조판 산물)는 저장 증거가 아니다.
    let real_first = |p: &Paragraph| -> Option<(i32, i32, i32)> {
        p.line_segs
            .iter()
            .find(|s| s.tag & synth == 0 && s.line_height > 0)
            .map(|s| (s.vertical_pos, s.line_height, s.line_spacing))
    };
    let (host_vpos, host_line_height, host_line_spacing) = real_first(para)?;
    let (next_vpos, _, _) = real_first(next)?;
    let delta = next_vpos - host_vpos;
    if delta <= 0 {
        return None;
    }
    let spacing_hu = |p: &Paragraph, after: bool| -> i32 {
        styles
            .para_styles
            .get(p.para_shape_id as usize)
            .map(|ps| {
                let px = if after {
                    ps.spacing_after
                } else {
                    ps.spacing_before
                };
                ((px * 7200.0 / dpi).round() as i32).max(0)
            })
            .unwrap_or(0)
    };
    let expected = host_line_height
        + host_line_spacing.max(0)
        + spacing_hu(para, true)
        + spacing_hu(next, false);
    ((delta - expected).abs() <= 1).then_some(delta)
}
