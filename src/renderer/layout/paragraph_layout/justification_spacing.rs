//! 양쪽 정렬의 낱말 공백 한계와 글자 사이 압축을 같은 측정으로 계산한다.

use super::{
    effective_text_for_metrics, estimate_text_width_exact, LeadingSpacePlan, ResolvedStyleSet,
    TextStyle,
};
use unicode_segmentation::UnicodeSegmentation;

pub(super) struct LineMeasure {
    pub trailing_spaces: usize,
    pub trailing_width: f64,
    pub slack: f64,
}

/// 줄의 배분 여부와 별개로 같은 양쪽 정렬 fit 여백을 측정한다.
/// 마지막 줄도 음수 여백일 때는 같은 압축 계약을 소비한다.
pub(super) fn measure(
    plan: &LeadingSpacePlan<'_>,
    styles: &ResolvedStyleSet,
    total_width: f64,
    available: f64,
    tab_width: f64,
    trailing_limit: usize,
    terminal_tracking: f64,
) -> LineMeasure {
    let trailing_spaces = plan
        .spacing_chars()
        .iter()
        .rev()
        .take_while(|(ch, _)| *ch == ' ')
        .count()
        .min(trailing_limit);
    let trailing_width = plan.line.runs.last().map_or(0.0, |run| {
        let mut style = run.text_style(styles);
        style.default_tab_width = tab_width;
        estimate_text_width_exact(&" ".repeat(trailing_spaces), &style)
    });
    LineMeasure {
        trailing_spaces,
        trailing_width,
        slack: available - (total_width - trailing_width - terminal_tracking),
    }
}

/// 한컴 Print의 공백 축소 한계는 원본 자간을 제외한 전진의 20%다.
/// 혼합 글자모양 줄에서는 실제 배분 대상 중 가장 작은 공백을 기준으로 한다.
/// 공백에도 extra_char_spacing이 붙으므로 그 몫은 word spacing에서 되돌린다.
pub(super) fn compress(
    plan: &LeadingSpacePlan<'_>,
    styles: &ResolvedStyleSet,
    trailing_spaces: usize,
    tab_width: f64,
    slack: f64,
    terminal_ink_overhang: f64,
) -> (f64, f64) {
    let count = plan.spacing_chars().len();
    let end = count.saturating_sub(trailing_spaces);
    let mut position = 0;
    let mut runs = Vec::new();
    let mut smallest_space = f64::INFINITY;
    let mut spaces = 0;
    let mut glyphs = 0;
    let mut last = None;
    for (index, run) in plan.line.runs.iter().enumerate() {
        // 필드의 표시 길이는 spacing_chars의 모델 한 칸과 다르다.
        let field = plan.body_field_runs.get(index).copied().unwrap_or(false);
        let display = effective_text_for_metrics(run);
        let length = if field { 1 } else { display.chars().count() };
        let take = end.saturating_sub(position).min(length);
        let text: String = if field && take > 0 {
            display.to_owned()
        } else {
            display.chars().take(take).collect()
        };
        position += length;
        let distributed = plan.spacing(index, 1.0) != 0.0;
        let mut style = run.text_style(styles);
        style.default_tab_width = tab_width;
        if distributed && text.contains(' ') {
            let mut untracked = style.clone();
            untracked.letter_spacing = 0.0;
            if let Some(latin) = untracked.ascii_punct_latin.as_mut() {
                latin.letter_spacing = 0.0;
            }
            smallest_space = smallest_space.min(estimate_text_width_exact(" ", &untracked));
            spaces += text.chars().filter(|&ch| ch == ' ').count();
        }
        for glyph in text.graphemes(true) {
            let eligible = distributed
                && !glyph
                    .chars()
                    .any(|ch| ch.is_whitespace() || ch == '\u{fffc}')
                && estimate_text_width_exact(glyph, &style) > 0.0;
            glyphs += usize::from(eligible);
            last = Some((glyph.to_owned(), style.clone(), eligible));
        }
        runs.push((text, style, distributed));
    }
    if spaces == 0 || !smallest_space.is_finite() {
        return (0.0, 0.0);
    }
    let word_delta = (slack / spaces as f64).max(-smallest_space * 0.2);
    let leftover = slack - word_delta * spaces as f64;
    let terminal = last.filter(|(_, _, eligible)| *eligible && !plan.terminal_inline_object);
    let slots = glyphs.saturating_sub(usize::from(terminal.is_some()));
    if leftover >= 0.0 || slots == 0 {
        return (word_delta, 0.0);
    }
    let measure = |word: f64, character: f64| -> f64 {
        runs.iter()
            .map(|(text, style, distributed)| {
                let mut measured = style.clone();
                if *distributed {
                    measured.extra_word_spacing = word;
                    measured.extra_char_spacing = character;
                }
                estimate_text_width_exact(text, &measured)
            })
            .sum::<f64>()
    };
    let natural_width = measure(0.0, 0.0);
    let terminal_recovery = |character: f64| -> f64 {
        let Some((glyph, style, _)) = &terminal else {
            return 0.0;
        };
        let mut measured: TextStyle = style.clone();
        measured.extra_char_spacing = character;
        let actual_delta =
            estimate_text_width_exact(glyph, &measured) - estimate_text_width_exact(glyph, style);
        // 같은 글리프 측정으로 좁은 글자 전진의 최소 폭도 반영한다.
        (terminal_ink_overhang - actual_delta).max(0.0) - terminal_ink_overhang
    };
    let mut character = leftover / slots as f64;
    for _ in 0..8 {
        let word = word_delta - character;
        let consumed = measure(word, character) - natural_width + terminal_recovery(character);
        let remainder = slack - consumed;
        if remainder.abs() <= 1.0 / 75.0 {
            break;
        }
        let next = (character + remainder / slots as f64).min(0.0);
        if (next - character).abs() <= f64::EPSILON {
            break;
        }
        character = next;
    }
    (word_delta - character, character)
}
