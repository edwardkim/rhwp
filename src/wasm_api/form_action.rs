use super::HwpDocument;
use crate::document_core::queries::form_action::FormAddress;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl HwpDocument {
    #[wasm_bindgen(js_name = getFormActionContext)]
    pub fn get_form_action_context(&self, address_json: &str) -> Result<String, JsValue> {
        if address_json.len() > 4096 {
            return Err(JsValue::from_str("FORM_ACTION_INVALID"));
        }
        let address: FormAddress = serde_json::from_str(address_json)
            .map_err(|_| JsValue::from_str("FORM_ACTION_INVALID"))?;
        self.core
            .form_action_context_native(&address)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    #[wasm_bindgen(js_name = applyFormActionUpdates)]
    pub fn apply_form_action_updates(&mut self, updates_json: &str) -> Result<(), JsValue> {
        self.core
            .apply_form_action_updates_native(updates_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
