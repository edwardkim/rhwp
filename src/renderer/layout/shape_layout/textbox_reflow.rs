//! 행렬 글상자도 공통 문단 프레임이 저장 줄 수용과 재조판을 결정한다.

use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::composer::{
    reflow_line_segs, stored_rows_reproduce_frame_expectation, ParagraphBox,
};
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(super) fn in_frame(
    paragraph: &mut Paragraph,
    available_width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    let style = styles.para_styles.get(paragraph.para_shape_id as usize);
    let paragraph_box = ParagraphBox::content_for_style(available_width, style, dpi);
    if !paragraph_box.is_usable() {
        return;
    }
    let top = paragraph
        .line_segs
        .first()
        .map_or(0, |line| line.vertical_pos);
    let mut frame = paragraph_box.frame(top);
    let original_partition = !paragraph.stored_text_partition_is_dirty()
        && !paragraph.layout_object_projection
        && paragraph
            .line_segs
            .iter()
            .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0);
    if original_partition
        && stored_rows_reproduce_frame_expectation(paragraph, &mut frame, styles, dpi)
    {
        // 저장 원점·폭이 실제 프레임과 같으면 자연 폭이 짧거나 상자가 낮아도
        // 원래 글자 구분을 유지한다. 세로 정렬은 같은 저장 줄을 소비한다.
        return;
    }
    // 실제 폭이 변했거나 편집된 구분은 별도의 px fit 없이 공통 채움으로 바꾼다.
    reflow_line_segs(paragraph, paragraph_box, styles, dpi);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::CharShapeRef;
    use crate::renderer::style_resolver::{ResolvedCharStyle, ResolvedParaStyle};

    fn saved_partition() -> (Paragraph, ResolvedStyleSet) {
        let paragraph = Paragraph {
            text: "AB".to_string(),
            char_count: 2,
            char_offsets: vec![0, 1],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: [0, 1]
                .into_iter()
                .map(|start| LineSeg {
                    text_start: start,
                    vertical_pos: start as i32 * 2400,
                    line_height: 1500,
                    text_height: 1500,
                    baseline_distance: 1275,
                    line_spacing: 900,
                    segment_width: 6000,
                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let mut styles = ResolvedStyleSet::default();
        styles.char_styles.push(ResolvedCharStyle {
            font_family: "Arial".to_string(),
            font_families: vec!["Arial".to_string(); 7],
            font_size: 20.0,
            ..Default::default()
        });
        styles.para_styles.push(ResolvedParaStyle::default());
        (paragraph, styles)
    }

    #[test]
    fn matching_frame_keeps_saved_text_partition_and_metrics() {
        let (mut paragraph, styles) = saved_partition();
        let original = paragraph.line_segs.clone();

        // 자연 폭이 한 줄에 들어가도 현재 6000 HU 프레임과 일치하는
        // 저장 구분은 합치지 않는다. 실제 편람의 4줄 반례와 같은 계약이다.
        in_frame(&mut paragraph, 80.0, &styles, 96.0);

        assert_eq!(paragraph.line_segs, original);
    }

    #[test]
    fn dirty_text_partition_reflows_even_in_matching_frame() {
        let (mut paragraph, styles) = saved_partition();
        paragraph.invalidate_layout_inputs();

        in_frame(&mut paragraph, 80.0, &styles, 96.0);

        assert_eq!(paragraph.line_segs.len(), 1);
        assert_eq!(paragraph.line_segs[0].text_start, 0);
        assert_eq!(paragraph.line_segs[0].segment_width, 6000);
        assert!(!paragraph.stored_text_partition_is_dirty());
    }
}
