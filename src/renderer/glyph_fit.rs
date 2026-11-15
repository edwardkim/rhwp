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

/// Canvas 폰트의 glyph를 레이아웃 advance 칸에 놓을 `(가로 배율, x 이동)`을 계산한다.
///
/// 양수 문자 간격은 다음 cluster의 시작 위치를 바꾸는 값이므로 glyph 폭 맞춤과
/// 분리한다. 음수 간격은 #2189 셀 오버플로우 보정의 기존 glyph-fit 계약을
/// 유지하고, 양수 배분 간격만 `glyph_fit_advance`로 제외한다.
///
/// 대체 glyph를 advance로 늘리거나 상자 폭 기준으로 누르면 모양이 망가진다.
/// - advance보다 좁은 ASCII는 늘리지 않고 칸 가운데에 둔다. 고정폭 원 글꼴의
///   `1`·`i`·`l`이 최대 2배로 늘어 굵은 글자처럼 보였다.
/// - advance보다 넓은 기호는 잉크(`ink_width`)가 칸에 들어가면 누르지 않고, 넘칠
///   때만 잉크를 칸 폭에 맞춘 뒤 상자를 칸 가운데에 둔다. 전각 대체 글꼴의 `μ`·`•`
///   같은 기호는 넓은 상자 가운데에 좁은 잉크가 있어, 상자 폭으로 누르면 잉크가
///   0.1~0.5배 실선처럼 눌렸다.
/// - 한글 음절·한자는 글자 상자를 채우도록 그린 glyph라 종전처럼 상자 폭을
///   advance에 맞춘다.
/// - `fixed_glyph_advances` 텍스트(HFT 자동 쪽번호)는 원본 폭 점유 계약이라
///   종전처럼 상자 폭을 맞추고, ASCII는 양방향으로 맞춘다.
pub(crate) fn canvas_cluster_fit(
    style: &TextStyle,
    cluster: &str,
    layout_cluster_advance: f64,
    visual_width: f64,
    ink_width: Option<f64>,
    pin_ascii_advance: bool,
) -> Option<(f64, f64)> {
    // 원 HFT의 중심 막대와 표시용 glyph의 자연 전진은 서로 다른 소유다.
    // 좁은 표시 glyph는 원래 모양 그대로 원본 em 칸 가운데에 놓는다.
    // 자간·배분 간격은 뒤 원점만 옮기며 이 intrinsic 칸의 중심을 바꾸지 않는다.
    if style.source_font_is_hft {
        let mut characters = cluster.chars();
        if let Some(character) = characters.next().filter(|_| characters.next().is_none()) {
            let source = style
                .original_glyph_metric_family
                .as_deref()
                .map(String::as_str)
                .or(style.metric_font_family.as_deref())
                .unwrap_or(&style.font_family);
            if let Some(em) = super::font_metrics_data::original_hft_centered_bar_advance_em(
                source,
                style.bold,
                style.italic,
                character,
            ) {
                let (size, _) = style.script_draw_metrics(style.effective_font_size(), 0.0);
                let advance = size * style.ratio_draw_params(1.0).1 * em;
                if advance.is_finite()
                    && visual_width.is_finite()
                    && visual_width > 0.0
                    && visual_width <= advance
                {
                    return Some((1.0, (advance - visual_width) / 2.0));
                }
            }
        }
    }
    // 원본 TTF에 없는 작은 수학 기호는 대체 face로 그리되 원본의 반각
    // 칸을 사용한다. 음수 자간은 윤곽을 누르지 않지만, 전각 대체 face의
    // 양쪽 빈 공간은 반각 칸 중앙에 맞춰야 독립 Print와 같은 원점이 된다.
    if style.letter_spacing < 0.0
        && matches!(cluster, "\u{2218}" | "\u{2219}")
        && style.native_otf_metrics.as_ref().is_some_and(|font| {
            cluster.chars().next().is_some_and(|ch| {
                // HWP5의 유형 미상은 TTF 선언이 아니다. 실제 첫 TTF가 결측이면
                // 뒤 표시 후보가 glyph를 갖더라도 같은 반각 칸의 정렬을 보존한다.
                // CFF/HFT의 원본 전진 계약을 이 조건으로 승격하지 않는다.
                (style.source_font_is_ttf && !font.has_glyph(ch))
                    || (!style.source_font_is_ttf
                        && !style.source_font_is_hft
                        && font.primary_truetype_lacks_glyph(ch))
            })
        })
    {
        let advance =
            style.glyph_fit_advance(layout_cluster_advance)? * style.script_advance_scale();
        if advance > 0.0
            && visual_width > advance + 0.25
            // Skia 경계의 반올림 오차는 아래 상자 초과 판정과 같은 0.25px을 허용한다.
            && ink_width.is_some_and(|ink| {
                ink.is_finite() && ink > 0.0 && ink <= advance * 1.1 + 0.25
            })
        {
            return Some((1.0, (advance - visual_width) / 2.0));
        }
    }
    // 명시 대체의 실제 잉크가 원본 메트릭의 좁은 칸을 차지하는 경우도 있다.
    // 기호의 음수 자간은 다음 원점만 이동시킨다. 그 간격을 glyph 칸 폭으로
    // 쓰거나, 잉크 보존을 이유로 전각 표시 프로그램의 여백까지 보존하지 않는다.
    // 독립 Print의 U+2219는 반각 advance에 원래 원형 잉크를 그대로 놓는다.
    // 실제 원본 TTF/HFT와 결측·추정 폭은 이 대체 프로그램 계약에 포함하지 않는다.
    if style.letter_spacing < 0.0
        && style.preserve_glyph_ink
        && !style.fixed_glyph_advances
        && !style.source_font_is_ttf
        && !style.source_font_is_hft
        && matches!(cluster, "\u{2218}" | "\u{2219}")
    {
        let ch = cluster.chars().next()?;
        let decision = layout::char_width_decision(&[ch], &[1], 0, style, true);
        if decision.width_source == "embeddedMetric" && decision.character_match == "hit" {
            let advance = decision.base_width_px * style.ratio_draw_params(1.0).1;
            if advance > 0.0
                && visual_width > advance + 0.25
                && ink_width
                    .is_some_and(|ink| ink.is_finite() && ink > 0.0 && ink <= advance * 1.1 + 0.25)
            {
                return Some((1.0, (advance - visual_width) / 2.0));
            }
        }
    }
    // 호출자가 선택한 프로그램은 전진과 잉크 폭을 구분한다. 원본 HFT 선언이나
    // 폭 표의 신뢰만으로 일반 대체 face의 기존 맞춤까지 차단하지 않는다.
    if !style.fixed_glyph_advances && style.preserve_glyph_ink {
        return None;
    }
    let cluster_advance =
        style.glyph_fit_advance(layout_cluster_advance)? * style.script_advance_scale();
    if cluster_advance <= 0.0 || visual_width <= 0.0 || style.letter_spacing < 0.0 {
        return None;
    }
    let box_scale = cluster_advance / visual_width;
    if style.fixed_glyph_advances && pin_ascii_advance {
        // 생성자가 원본 글자 폭을 paint 계약으로 명시한 텍스트(HFT 자동 쪽번호)는
        // 점유 폭을 원본 폭에 맞춘다.
        return Some((box_scale.clamp(0.1, 2.0), 0.0));
    }
    let overflow_tolerance = if pin_ascii_advance { 0.0 } else { 0.25 };
    if visual_width > cluster_advance + overflow_tolerance {
        let box_scale = box_scale.clamp(0.1, 1.0);
        // 한글 음절·한자(통합·확장 A·호환)
        let fills_em_box = |ch: char| {
            ('\u{AC00}'..='\u{D7A3}').contains(&ch)
                || ('\u{3400}'..='\u{9FFF}').contains(&ch)
                || ('\u{F900}'..='\u{FAFF}').contains(&ch)
        };
        let keeps_box_fit = style.fixed_glyph_advances || cluster.chars().any(fills_em_box);
        let ink_width = ink_width.filter(|width| width.is_finite() && *width > 0.0);
        let Some(ink_width) = ink_width.filter(|_| !keeps_box_fit) else {
            return Some((box_scale, 0.0));
        };
        let scale = (cluster_advance / ink_width).clamp(box_scale, 1.0);
        return Some((scale, (cluster_advance - visual_width * scale) / 2.0));
    }
    if pin_ascii_advance {
        return Some((1.0, (cluster_advance - visual_width) / 2.0));
    }
    None
}
