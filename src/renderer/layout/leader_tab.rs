//! 점끌기 뒤 내용의 정렬 끝은 탭 정지다. 리더 여백은 이 끝을 옮기지 않는다.

use crate::renderer::composer::{effective_text_for_metrics, ComposedLine};
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::TextStyle;

use super::paragraph_layout::right_tab_block_width;
use super::text_measurement::estimate_text_width_exact;

/// 단 기준의 오른쪽 정렬 목표. 저장 인라인 폭이 없는 자동 우단 경로도 공유한다.
pub(super) fn right_aligned_target(style: &TextStyle, has_stored_inline_tab: bool) -> Option<f64> {
    let stop = style
        .tab_stops
        .iter()
        .rev()
        .find(|tab| tab.tab_type == 1 && tab.fill_type != 0)?;
    let target = if !has_stored_inline_tab && style.available_width > 0.0 {
        style.text_start_offset + style.available_width
    } else {
        stop.position
    };
    target.is_finite().then_some(target)
}

/// 문자 모양·언어 경계로 나뉜 뒤쪽 내용도 하나의 정렬 블록으로 측정한다.
pub(super) fn block_width(
    style: &TextStyle,
    line: &ComposedLine,
    start: usize,
    styles: &ResolvedStyleSet,
) -> f64 {
    right_tab_block_width(
        &line.runs,
        start,
        styles,
        style.default_tab_width,
        &style.tab_stops,
        style.auto_tab_right,
        style.available_width,
        None,
        line.char_start,
    )
}

pub(super) fn apply_cross_run_width(
    style: &mut TextStyle,
    text: &str,
    line: &ComposedLine,
    index: usize,
    styles: &ResolvedStyleSet,
) {
    let Some((_, suffix)) = text.rsplit_once('\t') else {
        return;
    };
    let following = &line.runs[index + 1..];
    if following.is_empty() || following.iter().any(|run| run.text.contains('\t')) {
        return;
    }
    let numeric_tail = suffix.chars().chain(
        following
            .iter()
            .flat_map(|run| effective_text_for_metrics(run).chars()),
    );
    let mut has_digit = false;
    let only_page_number = numeric_tail.into_iter().all(|ch| {
        has_digit |= ch.is_ascii_digit();
        ch.is_ascii_digit() || ch.is_whitespace()
    }) && has_digit;
    if !style.auto_tab_right && !(only_page_number && right_aligned_target(style, true).is_some()) {
        return;
    }
    let tab_byte = text.rfind('\t').expect("suffix requires a tab");
    let mut natural_style = style.for_text_fragment(text[..tab_byte].chars().count() + 1);
    natural_style.right_tab_block_width_override = None;
    style.right_tab_block_width_override = Some(
        estimate_text_width_exact(suffix, &natural_style)
            + block_width(&natural_style, line, index + 1, styles),
    );
}
