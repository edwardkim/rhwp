//! 공통 글자겹침 배치와 반전형의 투명 레이어 합성.
use super::{HtmlCanvasElement, JsCast, TextRunNode, WebCanvasRenderer};
use crate::renderer::render_tree::BoundingBox;

impl WebCanvasRenderer {
    pub(super) fn render_composed_overlap(
        &mut self,
        bbox: &BoundingBox,
        run: &TextRunNode,
    ) -> bool {
        let Some(plan) = crate::renderer::char_overlap::project(run, *bbox) else {
            return false;
        };
        self.ctx.save();
        if run.rotation != 0.0 {
            let (x, y) = (bbox.x + bbox.width / 2.0, bbox.y + bbox.height / 2.0);
            let _ = self.ctx.translate(x, y);
            let _ = self.ctx.rotate(run.rotation.to_radians());
            let _ = self.ctx.translate(-x, -y);
        }
        if !plan.reversed {
            if let Some(border) = &plan.border {
                self.render_text_run(&border.bbox, &border.run);
            }
            for glyph in &plan.interior {
                self.render_text_run(&glyph.bbox, &glyph.run);
            }
        } else {
            let Some(document) = web_sys::window().and_then(|w| w.document()) else {
                self.ctx.restore();
                return false;
            };
            let Some(canvas) = document
                .create_element("canvas")
                .ok()
                .and_then(|c| c.dyn_into::<HtmlCanvasElement>().ok())
            else {
                self.ctx.restore();
                return false;
            };
            let b = plan.bounds;
            let scale = self.scale.max(1.0);
            canvas.set_width((b.width * scale).ceil() as u32 + 2);
            canvas.set_height((b.height * scale).ceil() as u32 + 2);
            let Ok(mut layer) = WebCanvasRenderer::new(&canvas) else {
                self.ctx.restore();
                return false;
            };
            let _ = layer.ctx.scale(scale, scale);
            let _ = layer.ctx.translate(-b.x, -b.y);
            layer.hft_outline_enabled = self.hft_outline_enabled;
            layer.hft_outline_fonts = self.hft_outline_fonts.take();
            if let Some(border) = &plan.border {
                layer.render_text_run(&border.bbox, &border.run);
            }
            // 내부 글자를 뺀 테두리만 주 Canvas에 합성한다. 본문과 배경은 보존한다.
            let _ = layer.ctx.set_global_composite_operation("destination-out");
            for glyph in &plan.interior {
                layer.render_text_run(&glyph.bbox, &glyph.run);
            }
            self.hft_outline_fonts = layer.hft_outline_fonts.take();
            let _ = self.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                &canvas,
                b.x,
                b.y,
                f64::from(canvas.width()) / scale,
                f64::from(canvas.height()) / scale,
            );
        }
        self.ctx.restore();
        true
    }
}
