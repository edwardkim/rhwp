//! 전진이 없는 글리프의 잉크와 글줄의 논리 폭을 구분한다.
use super::{
    render_tree::{BoundingBox, TextRunNode},
    text_direction::VisualTextPositions,
};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

/// 결합 문자·자소는 0 전진이어도 음수 side bearing에 잉크가 있다.
/// 원문 주소·폭·흐름을 바꾸지 않고 본문 clip과 paint 범위에만 보수적 em을 예약한다.
pub(crate) fn zero_advance_ink_bbox(bbox: BoundingBox, run: &TextRunNode) -> Option<BoundingBox> {
    if run.char_overlap.is_some() || run.is_vertical || run.rotation != 0.0 {
        return None;
    }
    let text = run.display_or_text();
    let positions = run.replay_positions_for(text);
    let visual = VisualTextPositions::for_style(text, &positions, &run.style);
    let (size, baseline) = run
        .style
        .script_draw_metrics(run.style.effective_font_size(), bbox.y + run.baseline);
    let horizontal = size * run.style.ratio.abs();
    if !size.is_finite() || !horizontal.is_finite() || size <= 0.0 || horizontal <= 0.0 {
        return None;
    }
    let mut left = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    for (start, cluster) in super::layout::split_into_clusters(text) {
        let end = start + cluster.chars().count();
        let (Some(&a), Some(&b)) = (positions.get(start), positions.get(end)) else {
            continue;
        };
        if !a.is_finite()
            || !b.is_finite()
            || (b - a).abs() > f64::EPSILON
            || !cluster.chars().any(|ch| {
                !ch.is_whitespace()
                    && !ch.is_control()
                    && ch.general_category() != GeneralCategory::Format
                    && ch != '\u{fffc}'
            })
        {
            continue;
        }
        let origin = bbox.x
            + visual
                .as_ref()
                .and_then(|v| v.span_origin(start, end))
                .unwrap_or(a);
        // 합성 기울임도 glyph의 전진을 바꾸지 않는다. 좌표를 맞추는 보정량이 아니라
        // clip이 정상 음수 bearing을 잘라 내지 않게 하는 글꼴 크기 단위의 범위다.
        let slant = if run.style.italic {
            size * super::text_italic::SKEW_X.abs()
        } else {
            0.0
        };
        left = left.min(origin - horizontal - slant);
        right = right.max(origin + horizontal + slant);
    }
    (left.is_finite() && right.is_finite() && baseline.is_finite())
        .then(|| BoundingBox::new(left, baseline - size, right - left, size * 2.0))
}
