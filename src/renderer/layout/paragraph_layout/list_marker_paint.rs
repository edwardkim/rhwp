//! 일반 본문과 인라인 표 문단이 같은 글머리표·번호 글리프를 발행한다.

use super::{
    marker_text_width, picture_bullet, BinDataContent, BoundingBox, CellContext, FieldMarkerType,
    PageLayoutContext, RenderNode, RenderNodeType, ResolvedStyleSet, TextRunNode, TextStyle,
};

pub(super) struct MarkerPaint<'a> {
    pub text: &'a str,
    pub style: TextStyle,
    pub para_style_id: u16,
    pub source: (usize, usize),
    pub cell_context: Option<CellContext>,
    pub x: f64,
    pub text_y: f64,
    pub line_height: f64,
    pub baseline: f64,
    pub picture_line_bottom: f64,
}

pub(super) fn node(
    tree: &mut PageLayoutContext,
    styles: &ResolvedStyleSet,
    bin_data: Option<&[BinDataContent]>,
    paint: MarkerPaint<'_>,
) -> Option<RenderNode> {
    let id = tree.next_id();
    let (section_index, para_index) = paint.source;
    if let Some(image) = picture_bullet::image(
        styles,
        paint.para_style_id as usize,
        bin_data,
        section_index,
        para_index,
        paint.cell_context.as_ref(),
    ) {
        return Some(RenderNode::new(
            id,
            RenderNodeType::Image(image),
            BoundingBox::new(
                paint.x,
                paint.picture_line_bottom - paint.style.font_size,
                paint.style.font_size,
                paint.style.font_size,
            ),
        ));
    }
    if picture_bullet::image_reference(styles, paint.para_style_id as usize).is_some() {
        // 외부 그림 참조의 빈 슬롯에 문자 마커를 대신 그리지 않는다.
        return None;
    }
    let width = marker_text_width(paint.text, &paint.style);
    Some(RenderNode::new(
        id,
        RenderNodeType::TextRun(TextRunNode {
            text: paint.text.to_owned(),
            style: paint.style,
            char_shape_id: None,
            para_shape_id: Some(paint.para_style_id),
            section_index: Some(section_index),
            para_index: Some(para_index),
            char_start: None,
            cell_context: paint.cell_context,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: paint.baseline,
            field_marker: FieldMarkerType::None,
            layout_positions: None,
            display_text: None,
        }),
        BoundingBox::new(paint.x, paint.text_y, width, paint.line_height),
    ))
}
