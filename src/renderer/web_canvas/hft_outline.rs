//! 실제 FontFace와 같은 바이트에서 얻은 윤곽만 현재 paint 변환에 재생한다.

use super::WebCanvasRenderer;
use crate::renderer::{
    hft_outline::{has_natural_hft_program, supports_style},
    PathCommand, TextStyle,
};

impl WebCanvasRenderer {
    pub(super) fn draw_hft_outline(&self, text: &str, style: &TextStyle, font_size: f64) -> bool {
        // 이 cluster는 호출자가 old_hangul_descriptor로 전환해 그린다.
        // 원래 family의 CFF에도 glyph가 있어도 우선 선택된 전용 face를 덮지 않는다.
        if !self.hft_outline_enabled || crate::renderer::contains_old_hangul_jamo(text) {
            return false;
        }
        let mut chars = text.chars();
        let Some(character) = chars.next() else {
            return false;
        };
        if chars.next().is_some() {
            return false;
        }
        let Some(commands) = self.hft_outline_fonts.as_ref().and_then(|fonts| {
            let original = supports_style(style)
                .then(|| fonts.glyph(&style.font_family, character))
                .flatten();
            if original.is_some() {
                original
            } else if self.render_profile == crate::paint::RenderProfile::Print
                && !crate::renderer::contextual_text::is_contextual(text)
            {
                // 실제 CSS 주 프로그램의 소유를 먼저 확인한다. 명시 자연 HFT의
                // 정적 TTF 재생도 보존하며 화면·문맥 shaping으로 확대하지 않는다.
                fonts.print_primary_glyph(style, character).or_else(|| {
                    (self.print_text_geometry || has_natural_hft_program(style))
                        .then(|| fonts.print_fallback_glyph(style, character))
                        .flatten()
                })
            } else {
                None
            }
        }) else {
            return false;
        };
        self.ctx.save();
        let _ = self.ctx.scale(font_size, font_size);
        self.ctx.begin_path();
        for command in commands.iter() {
            match *command {
                PathCommand::MoveTo(x, y) => self.ctx.move_to(x, y),
                PathCommand::LineTo(x, y) => self.ctx.line_to(x, y),
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    self.ctx.bezier_curve_to(x1, y1, x2, y2, x, y)
                }
                PathCommand::ClosePath => self.ctx.close_path(),
                _ => {
                    self.ctx.restore();
                    return false;
                }
            }
        }
        self.ctx.fill();
        self.ctx.restore();
        true
    }
}
