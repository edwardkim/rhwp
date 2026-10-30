//! 실제 공급 프로그램을 대조한 host 관측을 공통 조판 환경에 전달한다.

use super::HwpDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl HwpDocument {
    /// 같은 cmap 판정으로 아직 공급되지 않은 글리프만 다음 후보에 전달한다.
    #[wasm_bindgen(js_name = fontMissingText)]
    pub fn font_missing_text_js(
        &self,
        bytes: &[u8],
        face_index: u32,
        text: &str,
    ) -> Option<String> {
        crate::renderer::font_symbol_coverage::font_missing_text(bytes, face_index, text)
    }

    /// 공급 후보를 등록하기 전에 실제 cmap를 대조한다. CSS 이름 검사가 아니다.
    #[wasm_bindgen(js_name = fontSupportsText)]
    pub fn font_supports_text_js(&self, bytes: &[u8], face_index: u32, text: &str) -> bool {
        crate::renderer::font_symbol_coverage::font_supports_text(bytes, face_index, text)
    }

    /// FontFace와 Native fallback이 같은 실제 기호 공급 판정을 사용한다.
    #[wasm_bindgen(js_name = getFontSymbolUnicodeRange)]
    pub fn get_font_symbol_unicode_range_js(&self, bytes: &[u8], face_index: u32) -> String {
        crate::renderer::font_symbol_coverage::font_symbol_unicode_range(bytes, face_index)
    }

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
