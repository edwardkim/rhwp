//! 세로쓰기 칸의 최종 높이 축에서 공통 글줄을 구성한다.

use super::{
    compose_paragraph_in_context, recompose_cell_lines_in_frame, ComposedParagraph, ParagraphBox,
};
use crate::model::paragraph::Paragraph;
use crate::model::provenance::StoredGeometryPolicy;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 로드 과정의 가로 축 합성 줄을 세로쓰기의 원본 저장 줄로 소비하지 않는다.
/// 원본 IR과 유효한 저장 줄, 별도 개체 흐름이 발행한 줄은 그대로 보존한다.
pub(crate) fn compose_vertical_cell_lines(
    paragraph: &Paragraph,
    styles: &ResolvedStyleSet,
    inner_height: f64,
    dpi: f64,
    stored_policy: StoredGeometryPolicy,
) -> ComposedParagraph {
    let projected;
    let source = if !paragraph.line_segs.is_empty()
        && crate::renderer::para_has_no_stored_line_segs(paragraph)
        && !paragraph.layout_float_band
        && !paragraph.layout_object_projection
    {
        projected = {
            let mut value = paragraph.clone();
            value.line_segs.clear();
            value
        };
        &projected
    } else {
        paragraph
    };
    let mut composed = compose_paragraph_in_context(source, styles);
    recompose_cell_lines_in_frame(
        &mut composed,
        source,
        ParagraphBox::content_width_px(inner_height, dpi),
        styles,
        dpi,
        stored_policy,
    );
    composed
}
