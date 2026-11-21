//! 같은 문서에 실제 적재된 프로그램에서 확인한 PUA만 그린다.
use super::WebCanvasRenderer;
use crate::paint::RenderProfile;
use crate::renderer::{contains_old_hangul_jamo, pua_glyph, TextStyle};

impl WebCanvasRenderer {
    pub(super) fn source_glyph(&self, text: &str, style: &TextStyle) -> Option<char> {
        if !contains_old_hangul_jamo(text) {
            return None;
        }
        let fonts = self.hft_outline_fonts.as_ref()?;
        if let Some(glyph) = fonts.pua_coverage.glyph(&style.font_family, text) {
            return Some(glyph);
        }
        // 같은 family에 여러 프로그램이 있어도 실제 Print 주 face에서 확인한 윤곽은
        // 사용할 수 있다. 앞 face나 그 동반 프로그램의 소유가 불명인 경우는 제외한다.
        if self.render_profile != RenderProfile::Print || !self.hft_outline_enabled {
            return None;
        }
        pua_glyph::candidate(text)
            .into_iter()
            .chain(pua_glyph::fallback_candidate(text))
            .find(|&glyph| fonts.print_primary_glyph(style, glyph).is_some())
    }
}
