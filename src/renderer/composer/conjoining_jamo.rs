//! 조합에 속하지 않는 자모의 원점과 표시 프로그램을 같은 런에 보존한다.

use super::{ComposedParagraph, ComposedTextRun, ResolvedStyleSet};
use crate::renderer::TextStyle;

pub(super) fn apply_font(run: &ComposedTextRun, style: &mut TextStyle) {
    if style.isolated_jamo_font == Some("HCR Dotum")
        && run.display_text.is_none()
        && !run.text.is_empty()
        && run
            .text
            .chars()
            .all(crate::renderer::conjoining_jamo::is_mark)
        && run.char_overlap.is_none()
        && !run.inserted_control_text
        && run.footnote_marker.is_none()
    {
        style.font_family = "HCR Dotum, 함초롬돋움".to_owned();
        style.metric_font_family = Some("함초롬돋움".to_owned());
        style.font_metric_trusted = true;
        style.source_font_is_hft = false;
        style.hft_hangul_face = false;
        style.hft_font_face = false;
        style.hft_fullwidth_dot = false;
    }
}

pub(super) fn split_runs(composed: &mut ComposedParagraph, styles: &ResolvedStyleSet) {
    for line in &mut composed.lines {
        let mut output = Vec::with_capacity(line.runs.len());
        for run in std::mem::take(&mut line.runs) {
            let program = super::resolved_to_text_style(styles, run.char_style_id, run.lang_index)
                .isolated_jamo_font;
            if program != Some("HCR Dotum")
                || run.display_text.is_some()
                || run.char_overlap.is_some()
                || run.inserted_control_text
                || run.footnote_marker.is_some()
            {
                output.push(run);
                continue;
            }
            let chars: Vec<_> = run.text.chars().collect();
            let marks = crate::renderer::conjoining_jamo::isolated_positions(&chars);
            if !marks.iter().any(|&mark| mark) {
                output.push(run);
                continue;
            }
            let mut start = 0;
            for end in 1..=chars.len() {
                if end == chars.len() || marks[end] != marks[start] {
                    let mut piece = run.clone();
                    piece.text = chars[start..end].iter().collect();
                    output.push(piece);
                    start = end;
                }
            }
        }
        line.runs = output;
    }
}
