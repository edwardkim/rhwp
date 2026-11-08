//! 정렬의 가시 끝 뒤에는 다음 글자와의 자간이 없다.

use super::super::composer::ComposedLine;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::TextStyle;
use super::text_measurement::estimate_text_width_unrounded;

/// 같은 표시 글리프의 측정에서 실제 자간 기여만 분리한다.
/// 글꼴 크기에 대한 선언 비율을 좁은 글리프의 끝 자간으로 대신하지 않는다.
pub(crate) fn glyph_letter_spacing_advance(text: &str, style: &TextStyle) -> f64 {
    if style.letter_spacing == 0.0
        && style
            .ascii_punct_latin
            .as_ref()
            .is_none_or(|latin| latin.letter_spacing == 0.0)
    {
        return 0.0;
    }
    let display = super::super::composer::one_to_one_pua_replay_text(text);
    let tracked = estimate_text_width_unrounded(&display, style);
    let mut natural = style.clone();
    natural.letter_spacing = 0.0;
    if let Some(latin) = natural.ascii_punct_latin.as_mut() {
        latin.letter_spacing = 0.0;
    }
    tracked - estimate_text_width_unrounded(&display, &natural)
}

/// 마지막 가시 글자 뒤의 자간을 반환한다. 음수 자간도 부호를 보존한다.
/// 공백과 개체의 전진량 자체는 바꾸지 않고 정렬 폭에서만 제외한다.
pub(super) fn terminal_tracking_for_alignment(
    line: &ComposedLine,
    styles: &ResolvedStyleSet,
    tac_offsets: &[(usize, f64, usize)],
    trim_trailing_spaces: bool,
) -> f64 {
    if line.runs.iter().any(|run| {
        run.char_overlap.is_some()
                || run.display_text.as_ref().is_some_and(|display| {
                    super::super::composer::one_to_one_pua_replay_text(&run.text).as_ref()
                        != display
                })
                // 컨트롤 표제는 표시 글자와 원문 scalar의 좌표가 다르다.
                || run.inserted_control_text
    }) {
        return 0.0;
    }
    let mut text_end = line.char_start
        + line
            .runs
            .iter()
            .map(|run| run.source_scalar_count())
            .sum::<usize>();
    for run in line.runs.iter().rev() {
        let text = if trim_trailing_spaces {
            run.text.trim_end_matches(' ')
        } else {
            run.text.as_str()
        };
        text_end -= run.text.chars().count() - text.chars().count();
        if text.is_empty() {
            continue;
        }
        // 마지막 글자 뒤에 개체가 있으면 자간 다음의 내용이 실제로 존재한다.
        if tac_offsets.iter().any(|(pos, _, _)| *pos >= text_end) {
            return 0.0;
        }
        let Some(last) =
            unicode_segmentation::UnicodeSegmentation::graphemes(text, true).next_back()
        else {
            return 0.0;
        };
        if last.chars().any(char::is_whitespace) {
            return 0.0;
        }
        let style = run.text_style(styles);
        if style.kerning {
            return 0.0;
        }
        if super::text_measurement::has_hft_bold_tracking(&style) {
            // 가시 끝 다음에는 HFT 굵기 전진도 소비할 다음 글자가 없다.
            // 같은 측정 walk에서 원본 자간과 굵기 전진을 함께 제외한 끝을 쓴다.
            let mut intrinsic = style.clone();
            intrinsic.letter_spacing = 0.0;
            if let Some(latin) = intrinsic.ascii_punct_latin.as_mut() {
                latin.letter_spacing = 0.0;
            }
            let intrinsic_width =
                super::text_measurement::compute_glyph_fit_positions(last, &intrinsic)
                    .last()
                    .copied()
                    .unwrap_or(0.0);
            return estimate_text_width_unrounded(last, &style) - intrinsic_width;
        }
        return glyph_letter_spacing_advance(last, &style);
    }
    0.0
}
