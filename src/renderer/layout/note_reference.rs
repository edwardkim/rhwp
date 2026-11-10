//! 생성 번호의 원본 슬롯·전진 배열을 일반 문단과 control-only 표 호스트가 공유한다.

use super::text_measurement::resolved_to_text_style;
use super::*;

pub(super) struct Reference {
    pub number: u16,
    pub text: String,
    pub base_style: TextStyle,
    pub control_index: usize,
    pub width: f64,
    pub run: TextRunNode,
}

pub(super) fn reference(
    control: Option<&Control>,
    number: u16,
    styles: &ResolvedStyleSet,
    style_id: u32,
    control_index: usize,
) -> Reference {
    let generated = paragraph_layout::note_marker_from_control(control, number);
    let base_style = resolved_to_text_style(styles, style_id, generated.language_index);
    let mut paint = base_style.clone();
    // 생성 번호의 글꼴 속성은 보존하고, 본문의 줄 채움·탭·장식은 가져오지 않는다.
    paint.font_size = FootnoteMarkerNode::font_size(base_style.font_size);
    // 생성 번호는 본문 자간을 상속하지 않는다. 원본 프로그램 전진과 표시
    // 글리프의 폭 차이는 아래 공통 positions가 담당하며 자간으로 중복 소비하지 않는다.
    paint.letter_spacing = 0.0;
    paint.font_position = None;
    paint.superscript = false;
    paint.subscript = false;
    paint.extra_word_spacing = 0.0;
    paint.extra_char_spacing = 0.0;
    paint.extra_dash_advance = 0.0;
    paint.space_compression = None;
    paint.default_tab_width = 0.0;
    paint.tab_stops.clear();
    paint.inline_tabs.clear();
    paint.tab_leaders.clear();
    paint.auto_tab_right = false;
    paint.auto_left_tab_position = None;
    paint.right_tab_block_width_override = None;
    paint.available_width = 0.0;
    paint.line_x_offset = 0.0;
    paint.text_start_offset = 0.0;
    paint.text_paint = None;
    paint.underline = crate::model::style::UnderlineType::None;
    paint.strikethrough = false;
    paint.outline_type = 0;
    paint.shadow_type = 0;
    paint.continuous_shadow = None;
    paint.emboss = false;
    paint.engrave = false;
    paint.emphasis_dot = 0;
    paint.shade_color = u32::MAX;
    let positions = compute_char_positions(&generated.text, &paint);
    let width = positions.last().copied().unwrap_or(0.0);
    let run = TextRunNode {
        text: generated.text.clone(),
        style: paint,
        char_shape_id: Some(style_id),
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 0.0,
        field_marker: FieldMarkerType::None,
        layout_positions: Some(positions),
        display_text: None,
    };
    Reference {
        number,
        text: generated.text,
        base_style,
        control_index,
        width,
        run,
    }
}

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
            let style_id = para
                .char_shapes
                .first()
                .map_or(0, |shape| shape.char_shape_id);
            Some(reference(Some(control), number, styles, style_id, index))
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
    let previous_table =
        para.controls.iter().take(table_index).rposition(
            |control| matches!(control,Control::Table(table)if table.common.treat_as_char),
        );
    leading(para, styles, 0)
        .into_iter()
        .filter(|reference| {
            reference.control_index < table_index
                && previous_table.is_none_or(|index| reference.control_index > index)
                && control_line_seg_index(para, reference.control_index) == Some(owner)
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
    for reference in references {
        let baseline =
            FootnoteMarkerNode::reference_baseline(reference.base_style.font_size, line.1);
        let mut run = reference.run.clone();
        run.baseline = baseline;
        run.section_index = Some(owner.0);
        run.para_index = Some(owner.1);
        parent.children.push(RenderNode::new(
            tree.next_id(),
            RenderNodeType::FootnoteMarker(FootnoteMarkerNode {
                source_run: Some(Box::new(run)),
                number: reference.number,
                text: reference.text.clone(),
                base_font_size: reference.base_style.font_size,
                baseline,
                font_family: reference.base_style.font_family.clone(),
                color: reference.base_style.color,
                section_index: owner.0,
                para_index: owner.1,
                control_index: reference.control_index,
            }),
            BoundingBox::new(x, origin.1, reference.width, line.0),
        ));
        x += reference.width;
    }
}
