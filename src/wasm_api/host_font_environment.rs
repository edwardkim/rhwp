//! 실제 공급 프로그램을 대조한 host 관측을 공통 조판 환경에 전달한다.

use super::HwpDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl HwpDocument {
    #[wasm_bindgen(js_name = getDeclaredFontFallbackRequests)]
    pub fn get_declared_font_fallback_requests_js(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.declared_font_fallback_requests())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// 호출자는 원 face 부재·실제 대체 프로그램·입력 대응을 확인한 관측만 전달한다.
    #[wasm_bindgen(js_name = setObservedFontFallbacks)]
    pub fn set_observed_font_fallbacks_js(&mut self, json: &str) -> Result<bool, JsValue> {
        self.set_observed_font_fallbacks_native(json)
            .map_err(JsValue::from)
    }
}
