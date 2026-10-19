//! 본문과 번호·글머리표의 기본 크기가 같은 글줄 높이 예산을 소비한다.
use crate::model::paragraph::Paragraph;
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(super) fn include_marker(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    first_row: bool,
    text_font_size: f64,
) -> f64 {
    if !first_row {
        return text_font_size;
    }
    crate::renderer::layout::list_marker_geometry(para, styles)
        .filter(|geometry| geometry.marker_visible)
        .map_or(text_font_size, |geometry| {
            text_font_size.max(geometry.line_font_size_px)
        })
}
