//! 조합에 속하지 않는 자모의 원점과 표시 프로그램을 같은 런에 보존한다.

use super::{ComposedParagraph, ComposedTextRun, ParagraphBox, ResolvedStyleSet, SpaceMetric};
use crate::model::paragraph::{LineSeg, Paragraph};
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

/// 선택한 무전진 프로그램과 과거 spacing 프로그램의 줄 컷을 혼용하지 않는다.
/// 원문·저장 IR은 보존하고 같은 물리 상자의 현재 줄 결과만 발행한다.
pub(crate) fn project_stored_rows_with_selected_program(
    para: &Paragraph,
    box_: &ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(Vec<LineSeg>, Vec<(u32, SpaceMetric)>)> {
    let first = para.line_segs.first()?;
    if para.line_segs.len() < 2
        || !super::stored_word_tail::stored_rows_match_paragraph_box(para, box_)
        || first.segment_width <= 0
        || !para.controls.is_empty()
        || para.stored_text_partition_is_dirty()
        || para.text.contains(['\n', '\r'])
        || !para
            .text
            .chars()
            .any(crate::renderer::conjoining_jamo::is_mark)
        || para.line_segs.iter().enumerate().any(|(index, line)| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || !line.is_first_segment()
                || !line.is_last_segment()
                || line.column_start != first.column_start
                || line.segment_width != first.segment_width
                || line.line_height <= 0
                || line.line_spacing < 0
                || (index > 0 && (line.is_first_line_of_page() || line.is_first_line_of_column()))
        })
        || para.line_segs.windows(2).any(|pair| {
            pair[0]
                .vertical_pos
                .checked_add(pair[0].line_height)
                .and_then(|end| end.checked_add(pair[0].line_spacing))
                != Some(pair[1].vertical_pos)
        })
        || !replaces_spacing_mark(para, styles)
    {
        return None;
    }
    let mut frame = box_.for_stored_body_rows(para).frame(first.vertical_pos);
    let rows = super::line_breaking::layout_paragraph_in_frame(para, &mut frame, styles, dpi)?;
    // 줄 수가 같아도 프로그램 교체 뒤 글자의 줄 소속은 달라질 수 있다.
    (rows
        .iter()
        .map(|line| line.text_start)
        .ne(para.line_segs.iter().map(|line| line.text_start)))
    .then(|| (rows, frame.project_space_metrics()))
}

fn replaces_spacing_mark(para: &Paragraph, styles: &ResolvedStyleSet) -> bool {
    let chars: Vec<_> = para.text.chars().collect();
    let isolated = crate::renderer::conjoining_jamo::isolated_positions(&chars);
    let mut utf16_position = 0;
    for (index, &ch) in chars.iter().enumerate() {
        let position = para
            .char_offsets
            .get(index)
            .copied()
            .unwrap_or(utf16_position);
        utf16_position += ch.len_utf16() as u32;
        if !isolated[index] {
            continue;
        }
        let Some(reference) = para
            .char_shapes
            .iter()
            .rev()
            .find(|shape| shape.start_pos <= position)
        else {
            continue;
        };
        let mut original = super::resolved_to_text_style(
            styles,
            reference.char_shape_id,
            super::detect_lang_category(ch),
        );
        if original.isolated_jamo_font != Some("HCR Dotum")
            || matches!(
                original
                    .font_family
                    .split(',')
                    .next()
                    .unwrap_or_default()
                    .trim(),
                "HCR Dotum" | "HCRDotum" | "함초롬돋움"
            )
        {
            continue;
        }
        original.isolated_jamo_font = None;
        let prior = super::estimate_text_width_unrounded(&ch.to_string(), &original);
        if prior.is_finite() && prior > 0.0 {
            return true;
        }
    }
    false
}
