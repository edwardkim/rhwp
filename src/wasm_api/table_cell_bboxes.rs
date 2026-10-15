use super::HwpDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl HwpDocument {
    /// 지정 페이지의 셀 경계만 조회한다. 전체 셀 선택 API의 범위는 유지한다.
    #[wasm_bindgen(js_name = getTableCellBboxesAtPage)]
    pub fn get_table_cell_bboxes_at_page(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        page: u32,
    ) -> Result<String, JsValue> {
        self.get_table_cell_bboxes_at_page_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            page,
        )
        .map_err(JsValue::from)
    }
}
