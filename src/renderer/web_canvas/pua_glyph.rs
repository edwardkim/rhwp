//! 같은 문서에 실제 적재된 프로그램에서 확인한 PUA만 그린다.
use super::WebCanvasRenderer;
use crate::paint::RenderProfile;
use crate::renderer::{contains_old_hangul_jamo, pua_glyph, TextStyle};

impl WebCanvasRenderer {
    pub(super) fn source_glyph(&self, text: &str, style: &TextStyle) -> Option<char> {
        if !contains_old_hangul_jamo(text) {
            return None;
        }
        let fonts = self.legacy_font_outlines.as_ref()?;
        if let Some((glyph, _)) = pua_glyph::source_program_glyph(style, text) {
            if (crate::renderer::legacy_font_outline::supports_style(style)
                && fonts.glyph(&style.font_family, glyph).is_some())
                || fonts.print_primary_glyph(style, glyph).is_some()
            {
                return Some(glyph);
            }
        }
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
            .find(|&glyph| {
                if fonts.print_primary_glyph(style, glyph).is_some() {
                    return true;
                }
                // 줄 측정과 같은 공급자가 적재한 가시 글리프인지 확인한다.
                // 알려지지 않은 앞 face를 건너뛰어 CSS의 실제 선택을 바꾸지 않는다.
                let Some((metric, _)) =
                    crate::renderer::font_metrics_data::declared_old_hangul_fallback(style, glyph)
                else {
                    return false;
                };
                let coverage = &fonts.pua_coverage;
                coverage
                    .text_faces
                    .select(
                        &style.font_family,
                        &glyph.to_string(),
                        style.is_visually_bold(),
                        style.italic,
                    )
                    .is_some_and(|face| {
                        face.predecessors_known
                            && face.family.eq_ignore_ascii_case(metric.requested_name)
                            && coverage.glyph(&face.family, text) == Some(glyph)
                    })
            })
    }
}
