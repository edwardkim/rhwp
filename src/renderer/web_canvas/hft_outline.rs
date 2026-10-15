//! 실제 FontFace와 같은 바이트에서 얻은 윤곽만 현재 paint 변환에 재생한다.

use super::WebCanvasRenderer;
use crate::renderer::{hft_outline::supports_style, PathCommand, TextStyle};

impl WebCanvasRenderer {
    pub(super) fn draw_hft_outline(&self, text: &str, style: &TextStyle, font_size: f64) -> bool {
        if !self.hft_outline_enabled || !supports_style(style) {
            return false;
        }
        let mut chars = text.chars();
        let Some(character) = chars.next() else {
            return false;
        };
        if chars.next().is_some() {
            return false;
        }
        let Some(commands) = self
            .hft_outline_fonts
            .as_ref()
            .and_then(|fonts| fonts.glyph(&style.font_family, character))
        else {
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
