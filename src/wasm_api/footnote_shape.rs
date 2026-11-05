//! 각주 모양 조회·적용 바인딩 (주석 모양 대화상자의 각주 모양 탭).
use super::HwpDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl HwpDocument {
    /// 구역의 각주 모양을 조회한다. 필드는 `getEndnoteShape`와 같고 `numbering`은
    /// continue·restartSection·restartPage, `placement`는 eachColumn·mergedColumn·rightColumn이다.
    #[wasm_bindgen(js_name = getFootnoteShape)]
    pub fn get_footnote_shape(&self, section_idx: u32) -> Result<String, JsValue> {
        self.core
            .get_footnote_shape_native(section_idx as usize)
            .map_err(|e| e.into())
    }

    /// 구역의 각주 모양을 바꾸고 각주 번호·모양을 맞춘다. 빠진 필드는 그대로 둔다.
    #[wasm_bindgen(js_name = applyFootnoteShape)]
    pub fn apply_footnote_shape(
        &mut self,
        section_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.core
            .apply_footnote_shape_native(section_idx as usize, props_json)
            .map_err(|e| e.into())
    }
}
