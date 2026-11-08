//! Strict JS options boundary; the shared frontend owns page-range validation.
use crate::parser::pdf::{PdfImportLayout, PdfImportOptions, PdfImportPolicy, PdfImportProfile};
use wasm_bindgen::{JsCast, JsValue};

pub(super) fn options(
    value: JsValue,
) -> Result<(PdfImportOptions, PdfImportPolicy, PdfImportProfile), JsValue> {
    let invalid = || {
        JsValue::from_str("PDF_IMPORT_OPTIONS: expected { sourcePages?, includeTableInference?, layout?, password?, resourceProfile?, resourceLimits? }")
    };
    if value.is_undefined() {
        return Ok((
            PdfImportOptions::default(),
            PdfImportPolicy::default(),
            PdfImportProfile::Standard,
        ));
    }
    if !plain_record(&value) {
        return Err(invalid());
    }
    // Reject unknown keys rather than silently turning a misspelled selection into all pages.
    let keys = js_sys::Reflect::own_keys(&value).map_err(|_| invalid())?;
    for key in keys.iter() {
        if !matches!(
            key.as_string().as_deref(),
            Some(
                "sourcePages"
                    | "password"
                    | "includeTableInference"
                    | "layout"
                    | "resourceProfile"
                    | "resourceLimits"
            )
        ) {
            return Err(invalid());
        }
    }
    let profile =
        js_sys::Reflect::get(&value, &"resourceProfile".into()).map_err(|_| policy_error())?;
    let profile = if profile.is_undefined() {
        PdfImportProfile::Standard
    } else {
        profile
            .as_string()
            .ok_or_else(policy_error)?
            .parse::<PdfImportProfile>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?
    };
    let mut policy = PdfImportPolicy::for_profile(profile);
    let limits =
        js_sys::Reflect::get(&value, &"resourceLimits".into()).map_err(|_| policy_error())?;
    if !limits.is_undefined() {
        // Build JSON from checked own keys, not JSON.stringify/toJSON: explicit
        // undefined, symbols and non-integers must not disappear into defaults.
        let json = limits_json(&limits, true)?.to_string();
        policy = policy
            .with_limits_json(&json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
    }
    let inference = js_sys::Reflect::get(&value, &JsValue::from_str("includeTableInference"))
        .map_err(|_| invalid())?;
    let include_table_inference = if inference.is_undefined() {
        false
    } else {
        inference.as_bool().ok_or_else(invalid)?
    };
    let layout =
        js_sys::Reflect::get(&value, &JsValue::from_str("layout")).map_err(|_| invalid())?;
    let layout = if layout.is_undefined() {
        PdfImportLayout::default()
    } else {
        layout
            .as_string()
            .as_deref()
            .and_then(PdfImportLayout::parse)
            .ok_or_else(invalid)?
    };
    let password = js_sys::Reflect::get(&value, &"password".into()).map_err(|_| invalid())?;
    let password = if password.is_undefined() {
        None
    } else {
        if !password.is_string() || password.unchecked_ref::<js_sys::JsString>().length() > 1024 {
            return Err(invalid());
        }
        let password = password.as_string().ok_or_else(invalid)?;
        if password.len() > 1024 {
            return Err(invalid());
        }
        Some(password)
    };
    let pages =
        js_sys::Reflect::get(&value, &JsValue::from_str("sourcePages")).map_err(|_| invalid())?;
    if pages.is_undefined() {
        return Ok((
            PdfImportOptions {
                password,
                include_table_inference,
                layout,
                ..PdfImportOptions::default()
            },
            policy,
            profile,
        ));
    }
    let selection_error = || {
        JsValue::from_str("PDF_PAGE_SELECTION: sourcePages must be a nonempty, strictly increasing array of nonnegative page indices")
    };
    let pages = pages
        .dyn_into::<js_sys::Array>()
        .map_err(|_| selection_error())?;
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
    Ok((
        PdfImportOptions {
            password,
            source_pages: Some(result),
            include_table_inference,
            layout,
        },
        policy,
        profile,
    ))
}

fn plain_record(value: &JsValue) -> bool {
    if !value.is_object() || value.is_null() || js_sys::Array::is_array(value) {
        return false;
    }
    let Ok(prototype) = js_sys::Reflect::get_prototype_of(value) else {
        return false;
    };
    prototype.is_null()
        || js_sys::Object::is(
            prototype.as_ref(),
            js_sys::Object::get_prototype_of(&js_sys::Object::new()).as_ref(),
        )
}

fn policy_error() -> JsValue {
    JsValue::from_str(
        "PDF_RESOURCE_POLICY: expected a known profile and positive integer lower limits",
    )
}

fn limits_json(value: &JsValue, root: bool) -> Result<serde_json::Value, JsValue> {
    if !plain_record(value) {
        return Err(policy_error());
    }
    let keys = js_sys::Reflect::own_keys(value).map_err(|_| policy_error())?;
    if keys.length() > 64 {
        return Err(policy_error());
    }
    let mut fields = serde_json::Map::new();
    for key in keys.iter() {
        let name = key.as_string().ok_or_else(policy_error)?;
        if name.len() > 128 {
            return Err(policy_error());
        }
        let entry = js_sys::Reflect::get(value, &key).map_err(|_| policy_error())?;
        let entry = if root && name == "backend" {
            limits_json(&entry, false)?
        } else {
            let number = entry.as_f64().ok_or_else(policy_error)?;
            if !number.is_finite()
                || number.fract() != 0.0
                || number <= 0.0
                || number > 9_007_199_254_740_991.0
            {
                return Err(policy_error());
            }
            serde_json::Value::from(number as u64)
        };
        fields.insert(name, entry);
    }
    Ok(serde_json::Value::Object(fields))
}

pub(super) fn copy_input(
    data: &JsValue,
    policy: &PdfImportPolicy,
    session: &mut crate::PdfImportSession,
) -> Result<Vec<u8>, JsValue> {
    copy_input_limited(data, policy.max_input_bytes(), session)
}

pub(super) fn copy_input_limited(
    data: &JsValue,
    maximum: usize,
    session: &mut crate::PdfImportSession,
) -> Result<Vec<u8>, JsValue> {
    let data = data
        .dyn_ref::<js_sys::Uint8Array>()
        .ok_or_else(|| JsValue::from_str("PDF_INVALID_INPUT: expected Uint8Array"))?;
    let len = data.length() as usize;
    if len > maximum {
        return Err(JsValue::from_str(
            "PDF_RESOURCE_LIMIT: input exceeds request policy before WASM copy",
        ));
    }
    session.reserve_transfer(len)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(len)
        .map_err(|_| JsValue::from_str("PDF_WORKER_MEMORY: input allocation failed"))?;
    bytes.resize(len, 0);
    data.copy_to(&mut bytes);
    Ok(bytes)
}
