//! 변환된 글상자의 무효화된 줄 구분은 공통 문단 재조판으로 결정한다.

use crate::model::paragraph::Paragraph;
use crate::renderer::composer::{reflow_line_segs, ParagraphBox};
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
    // 호출자는 행렬·높이 변화로 저장 구분을 무효화한 경로다. 가로 키가
    // 같다는 이유로 그 구분을 다시 수용하지 않는다. 줄 합침의 별도 px 여유
    // 없이 공통 채움의 실제 글자 폭과 문단 여백으로 다시 구성한다.
    reflow_line_segs(paragraph, paragraph_box, styles, dpi);
}
