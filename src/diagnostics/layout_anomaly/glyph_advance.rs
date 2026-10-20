/// 렌더러의 replay 위치로 앞뒤 공백의 전진폭만 제거한다.
/// 장평·자간·탭·유효한 layout_positions를 같은 경로로 해석한다.
/// 표시 문자열이 달라진 필드도 backend와 같은 검증/fallback을 거친다.
pub(super) fn glyph_band_horizontal(
    run: &crate::renderer::render_tree::TextRunNode,
    x: f64,
    width: f64,
    _em: f64,
) -> (f64, f64) {
    let text = run.display_or_text();
    let is_space = |c: char| matches!(c, ' ' | '\u{3000}');
    let start = text.chars().take_while(|&c| is_space(c)).count();
    let end = text.chars().count() - text.chars().rev().take_while(|&c| is_space(c)).count();
    let positive_tracking =
        run.style.extra_char_spacing.is_finite() && run.style.extra_char_spacing > 0.0;
    if start >= end || (!positive_tracking && start == 0 && end == text.chars().count()) {
        return (x, width);
    }
    let positions = run.replay_positions_for(text);
    let Some((&left, &right)) = positions.get(start).zip(positions.get(end)) else {
        return (x, width);
    };
    // bbox와 전진폭의 계약이 맞지 않을 때 근거 없는 축소를 하지 않는다.
    if !left.is_finite() || !right.is_finite() || left < 0.0 || right <= left || left >= width {
        return (x, width);
    }
    // 공백 없는 쪽 끝은 원상자를 유지한다. 비공백 advance 차이까지 줄이지 않는다.
    let left = if start == 0 { 0.0 } else { left };
    let mut right = if end == text.chars().count() {
        width
    } else {
        right.min(width)
    };
    if let Some(ink_end) = tracked_glyph_end(run, text, &positions, width) {
        right = right.min(ink_end);
    }
    if right <= left {
        return (x, width);
    }
    (x + left, right - left)
}

/// 배분 간격은 다음 글자의 원점만 전진시키고 마지막 글리프를 늘리지 않는다.
/// SVG/Canvas와 같은 클러스터·replay·glyph-fit 소비 결과에서 보이는 끝을 구한다.
/// 음수 자간은 실제 glyph가 advance보다 넓을 수 있으므로 그 여유를 보존한다.
fn tracked_glyph_end(
    run: &crate::renderer::render_tree::TextRunNode,
    text: &str,
    positions: &[f64],
    width: f64,
) -> Option<f64> {
    let style = &run.style;
    if !style.extra_char_spacing.is_finite()
        || style.extra_char_spacing <= 0.0
        || !style.letter_spacing.is_finite()
        || !width.is_finite()
    {
        return None;
    }
    // 저장 replay와 bbox의 폭 계약이 다른 런을 임의로 좁히지 않는다.
    let advance = *positions.last()?;
    if (advance - width).abs() > 1e-6 * width.abs().max(1.0) {
        return None;
    }
    let fits = crate::renderer::glyph_fit_positions(
        text,
        style,
        run.layout_positions.as_deref(),
        positions,
    );
    let mut right: f64 = 0.0;
    for (start, cluster) in crate::renderer::layout::split_into_clusters(text) {
        if cluster.chars().all(char::is_whitespace) {
            continue;
        }
        let count = cluster.chars().count();
        let fit_advance = fits.get(start + count)? - fits.get(start)?;
        let glyph_width =
            style.glyph_fit_advance(fit_advance)? + (-style.letter_spacing).max(0.0) * count as f64;
        let end = positions.get(start)? + glyph_width;
        if !end.is_finite() {
            return None;
        }
        right = right.max(end);
    }
    (right > 0.0).then_some(right)
}
