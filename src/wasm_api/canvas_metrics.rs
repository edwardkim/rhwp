use super::HwpDocument;
use crate::renderer::supplemental_metrics::{MetricBackend, MetricContext};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl HwpDocument {
    #[wasm_bindgen(js_name = collectCanvasMetricRequests)]
    pub fn collect_canvas_metric_requests_js(
        &mut self,
        document: u32,
        fonts: u32,
    ) -> Result<String, JsValue> {
        let batch = self
            .collect_canvas_metric_requests(MetricContext {
                document_generation: document.into(),
                font_generation: fonts.into(),
                backend: MetricBackend::Canvas2d,
            })
            .map_err(|e| JsValue::from_str(&format!("Canvas metrics: {e:?}")))?;
        serde_json::to_string(&batch).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = registerCanvasMetricReplies)]
    pub fn register_canvas_metric_replies_js(
        &mut self,
        document: u32,
        fonts: u32,
        revision: f64,
        json: &str,
    ) -> Result<bool, JsValue> {
        if !revision.is_finite()
            || revision < 0.0
            || revision > 9_007_199_254_740_991.0
            || revision.fract() != 0.0
            || json.len() > 8 * 1024 * 1024
        {
            return Err(JsValue::from_str("Invalid Canvas metric reply envelope"));
        }
        let replies = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Canvas metrics: {e}")))?;
        self.register_canvas_metric_replies(
            MetricContext {
                document_generation: document.into(),
                font_generation: fonts.into(),
                backend: MetricBackend::Canvas2d,
            },
            revision as u64,
            replies,
        )
        .map_err(|e| JsValue::from_str(&format!("Canvas metrics: {e:?}")))
    }

    #[wasm_bindgen(js_name = selectCanvasMetrics)]
    pub fn select_canvas_metrics_js(&mut self, enabled: bool) -> Result<bool, JsValue> {
        self.select_canvas_metrics(enabled)
            .map_err(|e| JsValue::from_str(&format!("Canvas metrics: {e:?}")))
    }

    #[wasm_bindgen(js_name = canvasMetricsActive)]
    pub fn canvas_metrics_active_js(&self) -> bool {
        self.canvas_metrics_active()
    }

    #[wasm_bindgen(js_name = getCanvasPageLayerTree)]
    pub fn get_canvas_page_layer_tree_js(
        &self,
        page: u32,
        profile: &str,
        omit_font_bytes: bool,
    ) -> Result<String, JsValue> {
        let profile = crate::paint::RenderProfile::parse(profile)
            .ok_or_else(|| JsValue::from_str("Invalid render profile"))?;
        self.get_canvas_page_layer_tree_with_options_native(
            page,
            profile,
            crate::paint::LayerJsonOptions {
                omit_image_bytes: false,
                omit_font_bytes,
            },
        )
        .map_err(JsValue::from)
    }
}
