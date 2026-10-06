//! 미주용 구성 컨텍스트의 format 경계.

use crate::renderer::typeset::{
    ComposedParagraph, FormattedParagraph, Paragraph, ResolvedStyleSet, TypesetEngine,
};

impl TypesetEngine {
    /// 미주는 저장 LineSeg가 쪽/단 흐름의 정본이므로, HWP3 본문 orphan tail을
    /// 위한 fresh 줄 수 축소를 적용하지 않는다.
    pub(in crate::renderer::typeset) fn format_endnote_paragraph(
        &self,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        column_width_px: Option<f64>,
    ) -> FormattedParagraph {
        self.format_paragraph_for_flow(para, composed, styles, column_width_px, false, false)
    }
}
