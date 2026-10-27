//! 공통 글자겹침 배치를 일반 SVG 글자와 투명 마스크로 출력한다.
use super::SvgRenderer;
use crate::renderer::char_overlap::{Glyph, Projection};
use crate::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};

impl SvgRenderer {
    fn overlap_glyph(&mut self, id: u32, glyph: &Glyph, mask: bool) {
        let mut run = glyph.run.clone();
        if mask {
            run.style.color = 0;
        }
        self.render_layer_payload_node(id, RenderNodeType::TextRun(run), glyph.bbox, false);
    }

    pub(super) fn render_composed_overlap(&mut self, node: &RenderNode, run: &TextRunNode) -> bool {
        let Some(plan) = crate::renderer::char_overlap::project(run, node.bbox) else {
            return false;
        };
        self.with_layer_text_rotation(run.rotation, node.bbox, |renderer| {
            renderer.composed_overlap(node.id, &plan)
        });
        true
    }

    fn composed_overlap(&mut self, id: u32, plan: &Projection) {
        if plan.reversed {
            let b = plan.bounds;
            self.output.push_str(&format!("<defs><mask id=\"compose-{id}\" maskUnits=\"userSpaceOnUse\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\"/>",b.x,b.y,b.width,b.height,b.x,b.y,b.width,b.height));
            for glyph in &plan.interior {
                self.overlap_glyph(id, glyph, true);
            }
            self.output
                .push_str(&format!("</mask></defs><g mask=\"url(#compose-{id})\">"));
            if let Some(border) = &plan.border {
                self.overlap_glyph(id, border, false);
            }
            self.output.push_str("</g>");
        } else {
            if let Some(border) = &plan.border {
                self.overlap_glyph(id, border, false);
            }
            for glyph in &plan.interior {
                self.overlap_glyph(id, glyph, false);
            }
        }
    }
}
