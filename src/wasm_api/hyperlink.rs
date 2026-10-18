//! Studio 하이퍼링크의 엄격한 JSON options 어댑터 (#6963).
use super::HwpDocument;
use crate::document_core::hyperlink::HyperlinkTarget;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

fn parse<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, JsValue> {
    serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InsertOptions {
    target: HyperlinkTarget,
    start: usize,
    end: usize,
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    bookmark: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateOptions {
    target: HyperlinkTarget,
    field_id: u32,
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    bookmark: Option<String>,
}

/// 링크 대상은 웹 주소 `uri` 또는 문서 내부 책갈피 이름 `bookmark` 중 정확히 하나다.
fn one_destination_error() -> JsValue {
    JsValue::from_str("uri와 bookmark 중 하나만 지정해야 합니다")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TextOptions {
    target: HyperlinkTarget,
    field_id: u32,
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RemoveOptions {
    target: HyperlinkTarget,
    field_id: u32,
    #[serde(default)]
    restore_formatting: bool,
}

#[wasm_bindgen]
impl HwpDocument {
    /// 표시 문자는 Unicode scalar 축이다. 대상 누락 시 본문으로 폴백하지 않는다.
    #[wasm_bindgen(js_name = getHyperlinkContext)]
    pub fn get_hyperlink_context(&self, target_json: &str) -> Result<String, JsValue> {
        let target: HyperlinkTarget = parse(target_json)?;
        let paragraph = self.core.hyperlink_paragraph(&target)?;
        Ok(serde_json::json!({
            "text": paragraph.text,
            "links": self.core.hyperlinks_native(&target)?,
        })
        .to_string())
    }

    #[wasm_bindgen(js_name = insertHyperlinkEx)]
    pub fn insert_hyperlink_ex(&mut self, options_json: &str) -> Result<u32, JsValue> {
        let o: InsertOptions = parse(options_json)?;
        match (&o.uri, &o.bookmark) {
            (Some(uri), None) => self
                .core
                .insert_hyperlink_native(&o.target, o.start, o.end, uri),
            (None, Some(name)) => self
                .core
                .insert_bookmark_hyperlink_native(&o.target, o.start, o.end, name),
            _ => return Err(one_destination_error()),
        }
        .map_err(Into::into)
    }

    #[wasm_bindgen(js_name = updateHyperlinkEx)]
    pub fn update_hyperlink_ex(&mut self, options_json: &str) -> Result<bool, JsValue> {
        let o: UpdateOptions = parse(options_json)?;
        match (&o.uri, &o.bookmark) {
            (Some(uri), None) => self
                .core
                .update_hyperlink_native(&o.target, o.field_id, uri),
            (None, Some(name)) => self
                .core
                .update_bookmark_hyperlink_native(&o.target, o.field_id, name),
            _ => return Err(one_destination_error()),
        }
        .map_err(Into::into)
    }

    #[wasm_bindgen(js_name = replaceHyperlinkTextEx)]
    pub fn replace_hyperlink_text_ex(&mut self, options_json: &str) -> Result<bool, JsValue> {
        let o: TextOptions = parse(options_json)?;
        self.core
            .replace_hyperlink_text_native(&o.target, o.field_id, &o.text)
            .map_err(Into::into)
    }

    #[wasm_bindgen(js_name = removeHyperlinkEx)]
    pub fn remove_hyperlink_ex(&mut self, options_json: &str) -> Result<(), JsValue> {
        let o: RemoveOptions = parse(options_json)?;
        self.core
            .remove_hyperlink_with_format_native(&o.target, o.field_id, o.restore_formatting)
            .map_err(Into::into)
    }
}
