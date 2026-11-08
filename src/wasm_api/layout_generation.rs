//! Native CLI와 같은 조판 세대 설정을 WASM 세션에 연결한다.

use super::HwpDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl HwpDocument {
    /// 실제 기준 출력의 한컴 세대를 선택한다. 저장 메타데이터로 추정하지 않는다.
    /// 기존 공통 경로가 캐시를 무효화하고 모든 섹션을 다시 조판한다.
    #[wasm_bindgen(js_name = setHangul2024Compat)]
    pub fn set_hangul2024_compat(&mut self, enabled: bool) {
        self.core.set_hangul2024_compat(enabled);
    }
}
