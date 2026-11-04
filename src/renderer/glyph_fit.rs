//! 글자 원점을 옮기는 간격과 글리프 윤곽의 폭을 분리한다.

use super::{layout, legacy_font_advances, validated_replay_positions, TextStyle};
use std::borrow::Cow;

fn has_positive_source_tracking(style: &TextStyle) -> bool {
    style.letter_spacing > 0.0
        || style
            .ascii_punct_latin
            .as_ref()
            .is_some_and(|latin| latin.letter_spacing > 0.0)
}

fn intrinsic_style(style: &TextStyle) -> Cow<'_, TextStyle> {
    if !has_positive_source_tracking(style) {
        return Cow::Borrowed(style);
    }
    let mut intrinsic = style.clone();
    if intrinsic.letter_spacing > 0.0 {
        intrinsic.letter_spacing = 0.0;
    }
    if let Some(latin) = intrinsic.ascii_punct_latin.as_mut() {
        if latin.letter_spacing > 0.0 {
            latin.letter_spacing = 0.0;
        }
    }
    Cow::Owned(intrinsic)
}

/// 글리프 폭 맞춤(SVG `textLength`, Canvas `scaleX`)에 쓸 문자 경계를 돌려준다.
///
/// 원본의 양수 자간·커닝·배분 간격은 글자 원점에만 적용한다. 그 경계로 폭을
/// 맞추면 글리프 윤곽도 늘거나 눌리므로, 같은 측정기에서 원본 자간을 제외한다.
/// 기존 음수 간격의 최소 전진폭
/// clamp를 역산하지 않고 같은 측정기로 다시 계산하며, 그리는 원점은 보존한다.
pub(crate) fn glyph_fit_positions<'a>(
    replay_text: &str,
    style: &TextStyle,
    layout_positions: Option<&[f64]>,
    char_positions: &'a [f64],
) -> std::borrow::Cow<'a, [f64]> {
    let positive_source_tracking = has_positive_source_tracking(style);
    let intrinsic = intrinsic_style(style);
    let style = intrinsic.as_ref();
    if legacy_font_advances::has_fixed_ascii_bold_tracking(style) {
        // 원점은 확정 layout positions를 그대로 쓴다. 윤곽 폭만 측정이 실제
        // 적용한 tracking 기여를 제외하며, 다른 언어 슬롯·탭·음수 자간도
        // 같은 CharWidthDecision 결과에서 소비한다.
        let mut fit_style = style.clone();
        fit_style.extra_char_spacing = fit_style.extra_char_spacing.max(0.0);
        let decisions = layout::trace_char_width_decisions(replay_text, &fit_style);
        let mut positions = Vec::with_capacity(decisions.len() + 1);
        let mut x = 0.0;
        positions.push(x);
        for decision in decisions {
            x += decision.final_width_px - decision.bold_tracking_px;
            positions.push(x);
        }
        return std::borrow::Cow::Owned(positions);
    }
    if layout::has_hft_bold_tracking(style) {
        // 원본 HFT의 굵기 간격은 글자 원점만 전진시키며 intrinsic 폭은 유지한다.
        // backend는 원래 char_positions를 위치로, 이 경계를 glyph-fit 폭으로 쓴다.
        {
            let mut fit_style = style.clone();
            fit_style.extra_char_spacing = fit_style.extra_char_spacing.max(0.0);
            std::borrow::Cow::Owned(layout::compute_glyph_fit_positions(replay_text, &fit_style))
        }
    } else if (style.kerning && validated_replay_positions(replay_text, layout_positions).is_some())
        || positive_source_tracking
        || style.extra_char_spacing < 0.0
        || legacy_font_advances::has_bold_tracking(style)
    {
        let mut fit_style = style.clone();
        fit_style.extra_char_spacing = fit_style.extra_char_spacing.max(0.0);
        if legacy_font_advances::has_bold_tracking(style) {
            // 원점만 전진시키는 옛 프로그램의 굵은 자간을 글리프 확대에 쓰지 않는다.
            // 표시 글꼴·굵기와 자연 메트릭은 유지한다.
            fit_style.metric_font_family = None;
        }
        std::borrow::Cow::Owned(layout::compute_char_positions(replay_text, &fit_style))
    } else {
        std::borrow::Cow::Borrowed(char_positions)
    }
}

/// 문단 또는 명시적 개행 끝의 밑줄은 마지막 글리프 뒤의 자간을 포함하지 않는다.
/// 원본 run과 본문 원점은 보존하며 장식 replay의 마지막 경계만 바꾼다.
pub(crate) fn terminal_underline_positions(
    run: &super::render_tree::TextRunNode,
) -> Option<Vec<f64>> {
    if run.is_vertical
        || !(run.is_para_end || run.is_line_break_end)
        || !has_positive_source_tracking(&run.style)
    {
        return None;
    }
    let text = super::composer::one_to_one_pua_replay_text(run.display_or_text());
    // 말미 공백·탭의 범위는 기존 장식 trim/탭 계약이 결정한다.
    if text.chars().last()?.is_whitespace() {
        return None;
    }
    let mut positions = run.replay_positions_for(&text).into_owned();
    validated_replay_positions(&text, Some(&positions))?;
    let (start, cluster) = layout::split_into_clusters(&text).pop()?;
    let end = start + cluster.chars().count();
    let fit = glyph_fit_positions(
        &text,
        &run.style,
        run.layout_positions.as_deref(),
        &positions,
    );
    let natural = run.style.glyph_fit_advance(fit[end] - fit[start])?;
    let visible_end = positions[start] + natural;
    if !visible_end.is_finite() || visible_end < positions[end - 1] {
        return None;
    }
    drop(fit);
    positions[end] = visible_end;
    Some(positions)
}
