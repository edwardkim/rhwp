//! 해소한 구두점 언어 슬롯을 측정과 실제 표시가 함께 소비하는 run에 반영한다.

use super::{ComposedLine, ComposedTextRun, ResolvedStyleSet};
use crate::renderer::style_resolver::ResolvedCharStyle;

/// 반각 공백은 상속한 언어의 em·장평으로 전진하며 자간은 영문 슬롯을 쓴다.
/// 글꼴 자체의 공백 폭을 선언한 경로는 그 독립 계약을 유지한다.
fn space_origin(ch: char, inherited: usize, style: Option<&ResolvedCharStyle>) -> Option<usize> {
    let style = style?;
    (matches!(ch, ' ' | '\u{00a0}')
        && inherited != 1
        && style.font_space_em.is_none()
        && (style.font_size_for_lang(inherited) != style.font_size_for_lang(1)
            || style.ratio_for_lang(inherited) != style.ratio_for_lang(1)))
    .then_some(inherited)
}

pub(super) fn apply_space_origin(
    output: &mut super::TextStyle,
    styles: &ResolvedStyleSet,
    style_id: u32,
    origin: usize,
) {
    let Some(source) = styles.char_styles.get(style_id as usize) else {
        return;
    };
    let em = output.font_size * output.ratio;
    if output.font_space_em.is_none() && em > 0.0 {
        // font_space_em은 현재 표시 슬롯 em의 비율이다. 전진의 원래 em만
        // 환산하므로 영문 자간·글자 위치와 원본 글자모양을 바꾸지 않는다.
        output.font_space_em =
            Some(0.5 * source.font_size_for_lang(origin) * source.ratio_for_lang(origin) / em);
    }
}

pub(super) fn character_style(
    ch: char,
    inherited: usize,
    styles: &ResolvedStyleSet,
    style_id: u32,
) -> super::TextStyle {
    let origin = space_origin(ch, inherited, styles.char_styles.get(style_id as usize));
    let lang = metric_slot(ch, inherited, styles, style_id);
    let mut output = super::resolved_to_text_style(styles, style_id, lang);
    if let Some(origin) = origin {
        apply_space_origin(&mut output, styles, style_id, origin);
    }
    output
}

fn slot(ch: char, inherited: usize, regenerated: bool, style: Option<&ResolvedCharStyle>) -> usize {
    let Some(style) = style else {
        return inherited;
    };
    // 같은 face여도 중립 문자의 표시·자간에 쓰는 영문 속성은 한글과 다를 수 있다.
    // 공백 전진의 원래 em은 space_origin에서 보존한다.
    let explicit_metrics_differ = style.font_size_for_lang(inherited)
        != style.font_size_for_lang(1)
        || style.letter_spacing_for_lang(inherited) != style.letter_spacing_for_lang(1)
        || style.ratio_for_lang(inherited) != style.ratio_for_lang(1);
    if style.explicit_latin_slot && super::is_latin_slot_punct(ch) {
        1
    } else if explicit_metrics_differ && (matches!(ch, ' ' | '\u{00a0}') || ch.is_ascii_punctuation()) {
        1
    } else if !style.ascii_punct_latin_slot {
        inherited
    } else if ch.is_ascii_punctuation() {
        1
    } else if regenerated {
        super::reflow_punctuation_slot(ch, inherited, true)
    } else {
        inherited
    }
}

/// 줄 채움과 실제 run 분할이 동일한 중립 문자 슬롯을 소비한다.
pub(super) fn metric_slot(
    ch: char,
    inherited: usize,
    styles: &ResolvedStyleSet,
    style_id: u32,
) -> usize {
    slot(
        ch,
        inherited,
        false,
        styles.char_styles.get(style_id as usize),
    )
}

pub(super) fn resolve_run_slots(
    lines: &mut [ComposedLine],
    styles: &ResolvedStyleSet,
    regenerated: bool,
) {
    for line in lines {
        let mut resolved = Vec::new();
        for run in std::mem::take(&mut line.runs) {
            let style = styles.char_styles.get(run.char_style_id as usize);
            if run.char_overlap.is_some() || !run
                .text
                .chars()
                .any(|ch| slot(ch, run.lang_index, regenerated, style) != run.lang_index)
            {
                resolved.push(run);
                continue;
            }
            // 원문 글자를 바꾸지 않고 같은 run의 언어 경계만 나눈다. 공백도
            // 명시한 영문 속성이 다르면 그 슬롯을 쓰되 뒤 글자의 상속 언어는 유지한다.
            // 겹침 문자·PUA 표시 변환은 이 구성 결과 뒤에서 한 번 수행된다.
            let mut start = 0;
            let mut model_start = 0;
            let mut lang = run.lang_index;
            let mut origin = run.neutral_space_origin;
            for (scalar, (byte, ch)) in run.text.char_indices().enumerate() {
                let next = slot(ch, run.lang_index, regenerated, style);
                let next_origin = space_origin(ch, run.lang_index, style);
                if (next, next_origin) != (lang, origin) {
                    if byte > start {
                        resolved.push(ComposedTextRun {
                            text: run.text[start..byte].to_string(),
                            display_text: run.display_for_model_range(model_start, scalar),
                            lang_index: lang,
                            neutral_space_origin: origin,
                            ..run.clone()
                        });
                    }
                    start = byte;
                    model_start = scalar;
                    lang = next;
                    origin = next_origin;
                }
            }
            if start < run.text.len() {
                resolved.push(ComposedTextRun {
                    display_text: run.display_for_model_range(model_start, run.text.chars().count()),
                    text: run.text[start..].to_string(),
                    lang_index: lang,
                    neutral_space_origin: origin,
                    ..run
                });
            }
        }
        line.runs = resolved;
    }
}
