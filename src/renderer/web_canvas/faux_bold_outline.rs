//! 굵은 face가 없는 글꼴의 합성 굵기를 실제 적재 윤곽의 fill + stroke로 그린다(#8430).
//!
//! 한/글 Print는 같은 윤곽을 PDF `2 Tr`(fill 뒤 stroke)로 내보내고 선 굵기를 `0.02 em`으로
//! 준다([`super::super::faux_bold_stroke_width`]). Native Skia도 같은 윤곽을 한 번에
//! fill+stroke한다. Canvas2D의 `fillText` + `strokeText`는 같은 선 굵기를 주어도 브라우저의
//! 글자 마스크와 글자 stroke가 기하 stroke보다 좁게 합성돼 획이 가늘어진다. 여기서는
//! 브라우저가 고른 것과 같은 face의 윤곽을 path로 만들어 기하 그대로 fill·stroke한다.
//! 전진폭·원점·장평은 호출자가 이미 정한 값을 그대로 쓴다.
use super::WebCanvasRenderer;
use crate::renderer::{PathCommand, TextStyle};
use web_sys::DomMatrix;

impl WebCanvasRenderer {
    /// `stroke_space`는 장평·맞춤·기울임을 적용하기 전의 변환이다. 선 굵기는 PDF처럼
    /// 그 공간에서 등방으로 적용한다. 윤곽을 확정할 수 없으면 false로 기존 경로에 맡긴다.
    pub(super) fn draw_faux_bold_outline(
        &self,
        text: &str,
        style: &TextStyle,
        font_size: f64,
        stroke_space: &DomMatrix,
    ) -> bool {
        let color = super::color_to_css(style.color);
        self.draw_faux_bold_outline_with_color(text, style, font_size, stroke_space, &color)
    }

    /// 그림자·양각 등 효과 pass는 같은 윤곽을 그 pass의 색으로 그린다.
    pub(super) fn draw_faux_bold_outline_with_color(
        &self,
        text: &str,
        style: &TextStyle,
        font_size: f64,
        stroke_space: &DomMatrix,
        color: &str,
    ) -> bool {
        if style.text_paint.is_some() || !(font_size.is_finite() && font_size > 0.0) {
            return false;
        }
        let Some(width) = crate::renderer::faux_bold_stroke_width(style, font_size) else {
            return false;
        };
        let mut chars = text.chars();
        let (Some(character), None) = (chars.next(), chars.next()) else {
            return false;
        };
        // 합성 굵기 run의 Canvas descriptor는 보통 굵기다(CanvasTextFont).
        // 기울임 선택은 `prepare_italic_font`와 같으며, 실제 기울임 face는 기존 경로로 둔다.
        let Some(commands) = self
            .hft_outline_fonts
            .as_ref()
            .and_then(|fonts| fonts.faux_bold_glyph(style, character))
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
                PathCommand::ArcTo(..) => {
                    self.ctx.restore();
                    return false;
                }
            }
        }
        self.ctx.restore();
        self.ctx.save();
        self.ctx.set_fill_style_str(color);
        // TrueType·CFF 윤곽의 겹친 contour는 nonzero로 채운다(브라우저 글자 paint와 같음).
        self.ctx.fill();
        let _ = self.ctx.set_transform(
            stroke_space.a(),
            stroke_space.b(),
            stroke_space.c(),
            stroke_space.d(),
            stroke_space.e(),
            stroke_space.f(),
        );
        // 인쇄 래스터는 선과 같은 PDFium 최소 장치 폭 규칙을 따른다(raster_stroke).
        // 기준 PDF의 `2 Tr` stroke도 PDFium이 한 장치 픽셀보다 가늘게 그리지 않는다.
        let width = if self.print_stroke_adjustment {
            crate::renderer::raster_stroke::width(
                width,
                crate::renderer::raster_stroke::minimum_width([
                    stroke_space.a(),
                    stroke_space.b(),
                    stroke_space.c(),
                    stroke_space.d(),
                ]),
            )
        } else {
            width
        };
        self.ctx.set_stroke_style_str(color);
        self.ctx.set_line_width(width);
        self.ctx.set_line_join("miter");
        self.ctx.set_miter_limit(10.0);
        self.ctx.stroke();
        self.ctx.restore();
        true
    }
}
