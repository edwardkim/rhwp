//! 같은 문서에 실제 적재된 프로그램에서 확인한 PUA만 그린다.
use super::WebCanvasRenderer;
use crate::renderer::TextStyle;

impl WebCanvasRenderer {
    pub(super) fn source_glyph(&self, text: &str, style: &TextStyle) -> Option<char> {
        self.hft_outline_fonts
            .as_ref()
            .and_then(|fonts| fonts.pua_coverage.glyph(&style.font_family, text))
    }
}
