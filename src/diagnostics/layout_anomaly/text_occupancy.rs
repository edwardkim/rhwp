//! Replay 원점과 글리프 점유를 구분한다. 분산 전진의 말미 간격은 잉크가 아니다.

use crate::renderer::render_tree::TextRunNode;

pub(super) fn glyph_band_horizontal(run: &TextRunNode, x: f64, width: f64, _em: f64) -> (f64, f64) {
    let text = run.display_or_text();
    let is_space = |c: char| matches!(c, ' ' | '\u{3000}');
    let count = text.chars().count();
    let start = text.chars().take_while(|&c| is_space(c)).count();
    let end = count - text.chars().rev().take_while(|&c| is_space(c)).count();
    let distributed =
        run.style.extra_char_spacing.is_finite() && run.style.extra_char_spacing > 0.0;
    if start >= end || (start == 0 && end == count && !distributed) {
        return (x, width);
    }
    let positions = run.replay_positions_for(text);
    let Some((&left, &advance_end)) = positions.get(start).zip(positions.get(end)) else {
        return (x, width);
    };
    if !left.is_finite()
        || !advance_end.is_finite()
        || left < 0.0
        || advance_end <= left
        || left >= width
    {
        return (x, width);
    }
    let left = if start == 0 { 0.0 } else { left };
    let mut right = if end == count {
        width
    } else {
        advance_end.min(width)
    };
    if distributed {
        // 마지막 글자의 원점은 배치의 replay를 보존한다. 그 뒤에 붙인 분산 간격만
        // 제외하며, 글자 폭은 동일한 paint 메트릭의 자간 전 자연 전진을 사용한다.
        // 손상된/미확인 메트릭으로 기존 후보를 줄이지 않는다.
        let decisions = crate::renderer::layout::trace_char_width_decisions(text, &run.style);
        if let Some(decision) = decisions.get(end - 1) {
            let ratio = if run.style.ratio > 0.0 {
                run.style.ratio
            } else {
                1.0
            };
            let glyph_width = decision.base_width_px * ratio;
            let occupied_end = positions[end - 1] + glyph_width;
            if glyph_width.is_finite()
                && glyph_width > 0.0
                && occupied_end.is_finite()
                && occupied_end > left
                && occupied_end < right
            {
                right = occupied_end;
            }
        }
    }
    if right <= left {
        return (x, width);
    }
    (x + left, right - left)
}
