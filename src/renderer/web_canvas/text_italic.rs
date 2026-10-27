//! CSS의 합성 각도 대신 실제 적재 face의 스타일과 공통 잉크 변환을 사용한다.
use super::WebCanvasRenderer;
use crate::renderer::{text_italic::SKEW_X, TextStyle};

impl WebCanvasRenderer {
    /// 같은 source face의 전진은 보존하고, 정체 face의 잉크만 기울인다.
    /// 실제 italic/oblique 프로그램에는 추가 shear를 적용하지 않는다.
    pub(super) fn prepare_italic_font(&self, text: &str, style: &TextStyle, size: f64) -> bool {
        if !style.italic {
            return false;
        }
        let families = crate::renderer::canvas_text_font::family_chain(style);
        let selected = self.hft_outline_fonts.as_ref().and_then(|fonts| {
            fonts
                .pua_coverage
                .text_faces
                .select(&families, text, style.is_visually_bold(), true)
        });
        let Some(selected) = selected else {
            return false;
        };
        self.ctx.set_font(&format!(
            "{}{}{:.3}px {}",
            if selected.slanted { "italic " } else { "" },
            if style.is_visually_bold() {
                "bold "
            } else {
                ""
            },
            size,
            families
        ));
        !selected.slanted
    }

    pub(super) fn apply_synthetic_italic(&self, synthetic: bool) {
        if synthetic {
            let _ = self.ctx.transform(1.0, 0.0, SKEW_X, 1.0, 0.0, 0.0);
        }
    }
}
