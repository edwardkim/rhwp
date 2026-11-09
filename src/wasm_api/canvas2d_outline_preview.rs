//! Explicit document-owned Canvas2D outline preview; no JSON paint importer.

use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

use crate::paint::RenderProfile;
use crate::renderer::layer_renderer::LayerRenderer;
use crate::renderer::web_canvas::WebCanvasRenderer;

use super::{normalize_canvas_scale, scaled_canvas_extent, HwpDocument};

pub(super) fn render(
    document: &HwpDocument,
    page: u32,
    canvas: &HtmlCanvasElement,
    scale: f64,
    profile: &str,
) -> Result<String, JsValue> {
    let profile = RenderProfile::parse(profile)
        .ok_or_else(|| JsValue::from_str(&format!("unsupported render profile: {profile}")))?;
    crate::renderer::image_resolver::with_decoded_handoff(|| {
        let tree = document
            .build_canvas_page_layer_tree_with_profile(page, profile)
            .map_err(JsValue::from)?;
        let scale = normalize_canvas_scale(tree.page_width, tree.page_height, scale)
            .map_err(JsValue::from_str)?;
        canvas.set_width(scaled_canvas_extent(tree.page_width, scale));
        canvas.set_height(scaled_canvas_extent(tree.page_height, scale));
        let mut renderer = WebCanvasRenderer::new(canvas)?;
        renderer.set_hft_outline_fonts(&document.hft_outline_fonts);
        renderer.set_scale(scale);
        renderer.set_strict_glyph_outline_replay(true);
        renderer.render_page(&tree).map_err(JsValue::from)?;
        let reports = renderer
            .get_text_variant_selection_diagnostics()
            .iter()
            .map(|report| {
                serde_json::json!({
                    "backend": report.backend.as_str(),
                    "equivalenceGroup": report.equivalence_group,
                    "selectedVariantId": report.selected_variant_id,
                    "selectedVariantKind": report.selected_variant_kind.map(|kind| kind.as_str()),
                    "selectedReason": report.selected_reason.as_str(),
                    "fallbackRequired": report.fallback_required,
                    "rejectedVariants": report.rejected_variants.iter().map(|rejected| {
                        serde_json::json!({
                            "variantId": rejected.variant_id,
                            "variantKind": rejected.variant_kind.as_str(),
                            "reasons": rejected.reasons.iter().map(|reason| reason.as_str()).collect::<Vec<_>>(),
                        })
                    }).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>();
        Ok(serde_json::json!({
            "backend": "canvas2d",
            "strictGlyphOutlineReplay": true,
            "reports": reports,
            "reportsDropped": renderer.text_variant_selection_diagnostics_dropped(),
        })
        .to_string())
    })
}
