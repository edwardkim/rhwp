//! 원문 겹침의 공통 배치를 일반 글자 출력으로 소비한다.
use super::SkiaTextReplay;
use crate::renderer::char_overlap::{Glyph, Projection};
use crate::renderer::render_tree::BoundingBox;
use skia_safe::{canvas::SaveLayerRec, BlendMode, Paint};

impl SkiaTextReplay<'_> {
    pub(crate) fn draw_composed_overlap(
        &self,
        plan: &Projection,
        rotation: f64,
        bbox: BoundingBox,
    ) {
        self.canvas.save();
        if rotation != 0.0 {
            self.canvas.rotate(
                rotation as f32,
                Some(
                    (
                        (bbox.x + bbox.width / 2.0) as f32,
                        (bbox.y + bbox.height / 2.0) as f32,
                    )
                        .into(),
                ),
            );
        }
        let draw = |g: &Glyph| {
            let r = &g.run;
            self.draw_text(
                &r.text,
                g.bbox,
                &r.style,
                r.baseline,
                0.0,
                false,
                None,
                false,
                false,
                false,
                r.layout_positions.as_deref(),
                0,
                false,
                false,
            );
        };
        if plan.reversed {
            self.canvas.save_layer(&SaveLayerRec::default());
            if let Some(border) = &plan.border {
                draw(border);
            }
            let mut erase = Paint::default();
            erase.set_blend_mode(BlendMode::DstOut);
            self.canvas
                .save_layer(&SaveLayerRec::default().paint(&erase));
            for g in &plan.interior {
                draw(g);
            }
            self.canvas.restore();
            self.canvas.restore();
        } else {
            if let Some(border) = &plan.border {
                draw(border);
            }
            for g in &plan.interior {
                draw(g);
            }
        }
        self.canvas.restore();
    }
}
