//! 참조 번호의 선두 슬롯을 일반 문단과 control-only 표 호스트가 공유한다.

use super::text_measurement::resolved_to_text_style;
use super::*;

pub(super) type Reference = (u16, String, TextStyle, usize, f64);

pub(super) fn leading(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    line_start: usize,
) -> Vec<Reference> {
    let positions = para.control_text_positions();
    para.controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| {
            let number = match control {
                Control::Footnote(note) => note.number,
                Control::Endnote(note) => note.number,
                _ => return None,
            };
            let position = if para.text.is_empty() {
                0
            } else {
                *positions.get(index)?
            };
            if position != line_start {
                return None;
            }
            let text = paragraph_layout::note_marker_text_from_control(Some(control), number);
            let style_id = para
                .char_shapes
                .first()
                .map_or(0, |shape| shape.char_shape_id);
            let style = resolved_to_text_style(styles, style_id, 0);
            let marker_style = TextStyle {
                font_size: FootnoteMarkerNode::font_size(style.font_size),
                ..style.clone()
            };
            let width = estimate_text_width(&text, &marker_style);
            Some((number, text, style, index, width))
        })
        .collect()
}

pub(super) fn before_table(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    table_index: usize,
) -> Vec<Reference> {
    let Some(owner) = control_line_seg_index(para, table_index) else {
        return Vec::new();
    };
    let previous_table = para.controls.iter().take(table_index).rposition(
        |control| matches!(control, Control::Table(table) if table.common.treat_as_char),
    );
    leading(para, styles, 0)
        .into_iter()
        .filter(|reference| {
            reference.3 < table_index
                && previous_table.is_none_or(|index| reference.3 > index)
                && control_line_seg_index(para, reference.3) == Some(owner)
        })
        .collect()
}

pub(super) fn append(
    tree: &mut PageLayoutContext,
    parent: &mut RenderNode,
    references: &[Reference],
    origin: (f64, f64),
    line: (f64, f64),
    owner: (usize, usize),
) {
    let mut x = origin.0;
    for (number, text, style, control_index, width) in references {
        parent.children.push(RenderNode::new(
            tree.next_id(),
            RenderNodeType::FootnoteMarker(FootnoteMarkerNode {
                number: *number,
                text: text.clone(),
                base_font_size: style.font_size,
                baseline: FootnoteMarkerNode::reference_baseline(style.font_size, line.1),
                font_family: style.font_family.clone(),
                color: style.color,
                section_index: owner.0,
                para_index: owner.1,
                control_index: *control_index,
            }),
            BoundingBox::new(x, origin.1, *width, line.0),
        ));
        x += width;
    }
}
