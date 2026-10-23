//! Strict JS options boundary; the shared frontend owns page-range validation.
use crate::parser::pdf::{PdfImportOptions, PdfImportPolicy};
use wasm_bindgen::{JsCast, JsValue};

pub(super) fn options(value: JsValue) -> Result<PdfImportOptions, JsValue> {
    let invalid = || {
        JsValue::from_str("PDF_IMPORT_OPTIONS: expected { sourcePages?: number[], includeTableInference?: boolean }")
    };
    if value.is_undefined() {
        return Ok(PdfImportOptions::default());
    }
    if !value.is_object() || value.is_null() || js_sys::Array::is_array(&value) {
        return Err(invalid());
    }
    // Reject unknown keys rather than silently turning a misspelled selection into all pages.
    let keys = js_sys::Reflect::own_keys(&value).map_err(|_| invalid())?;
    for key in keys.iter() {
        if !matches!(
            key.as_string().as_deref(),
            Some("sourcePages" | "includeTableInference")
        ) {
            return Err(invalid());
        }
    }
    let inference = js_sys::Reflect::get(&value, &JsValue::from_str("includeTableInference"))
        .map_err(|_| invalid())?;
    let include_table_inference = if inference.is_undefined() {
        false
    } else {
        inference.as_bool().ok_or_else(invalid)?
    };
    let pages =
        js_sys::Reflect::get(&value, &JsValue::from_str("sourcePages")).map_err(|_| invalid())?;
    if pages.is_undefined() {
        return Ok(PdfImportOptions {
            include_table_inference,
            ..PdfImportOptions::default()
        });
    }
    let selection_error = || {
        JsValue::from_str("PDF_PAGE_SELECTION: sourcePages must be a nonempty, strictly increasing array of nonnegative page indices")
    };
    let pages = pages
        .dyn_into::<js_sys::Array>()
        .map_err(|_| selection_error())?;
    let policy = PdfImportPolicy::default();
    if pages.length() == 0 || pages.length() as usize > policy.max_selected_pages {
        return Err(selection_error());
    }
    let mut result = Vec::with_capacity(pages.length() as usize);
    for page in pages.iter() {
        let page = page.as_f64().ok_or_else(selection_error)?;
        // Source indices and selected count have separate bounds before wasm32 narrowing.
        if !page.is_finite()
            || page.fract() != 0.0
            || page < 0.0
            || page >= policy.max_source_pages as f64
        {
            return Err(selection_error());
        }
        let page = page as usize;
        if result.last().is_some_and(|previous| *previous >= page) {
            return Err(selection_error());
        }
        result.push(page);
    }
    Ok(PdfImportOptions {
        source_pages: Some(result),
        include_table_inference,
    })
}
