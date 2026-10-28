//! 본문 선두의 미주 참조 번호는 본문 문자열과 별개의 표시 런이다.

use super::*;

/// 첫 본문 줄 앞의 번호 표시와 그 줄 채움이 소비하는 공통 점유.
pub(crate) struct LeadingEndnotePrefixGeometry {
    pub advance: f64,
    pub height: f64,
    pub baseline: f64,
}

pub(crate) fn leading_endnote_prefix_geometry(
    paragraph: &Paragraph,
    styles: &ResolvedStyleSet,
    first_char: usize,
) -> LeadingEndnotePrefixGeometry {
    let positions = paragraph.control_text_positions();
    let style_id = paragraph.char_shapes.first().map_or(0, |s| s.char_shape_id);
    let base_style = resolved_to_text_style(styles, style_id, 0);
    // 정상 한컴 Print의 본문 참조 번호는 본문 글자의 3/4 크기다.
    let scale = 0.75;
    let rise = base_style.font_size * (1.0 - scale) * TAC_OBJECT_ASCENT_RATIO;
    let mut geometry = LeadingEndnotePrefixGeometry {
        advance: 0.0,
        height: 0.0,
        baseline: base_style.font_size * TAC_OBJECT_ASCENT_RATIO - rise,
    };
    for (control_index, control) in paragraph.controls.iter().enumerate() {
        let Control::Endnote(note) = control else {
            continue;
        };
        let Some(&position) = positions.get(control_index) else {
            continue;
        };
        if !is_leading_note_marker_rendered_as_prefix(
            Some(paragraph),
            control_index,
            0,
            0,
            position,
            first_char,
        ) {
            continue;
        }
        // 원문에 공백을 추가하지 않는다. 언어별 표시 face와 실제 폭도 공유한다.
        let label = note_marker_text_from_control(Some(control), note.number);
        let runs = crate::renderer::composer::split_runs_by_lang(vec![ComposedTextRun {
            text: label,
            char_style_id: style_id,
            ..Default::default()
        }]);
        for run in runs {
            let mut style = resolved_to_text_style(styles, style_id, run.lang_index);
            style.font_size *= scale;
            let width = estimate_text_width_exact(&run.text, &style);
            geometry.advance += width;
            geometry.height = geometry.height.max(style.font_size);
        }
    }
    geometry
}
