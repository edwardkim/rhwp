//! WASM consumes the same bounded static artwork preparation as Native.
use crate::renderer::render_tree::BoundingBox;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    view_box: [f64; 4],
    viewport: [f64; 4],
    scale: f64,
    current_color: String,
}

#[wasm_bindgen(js_name = prepareStaticSvgRaster)]
pub fn prepare_static_svg_raster(options_json: &str, fragment: &str) -> JsValue {
    if options_json.len() > 4096 || fragment.len() > 1024 * 1024 {
        return JsValue::UNDEFINED;
    }
    let Ok(options) = serde_json::from_str::<Options>(options_json) else {
        return JsValue::UNDEFINED;
    };
    let bbox = |[x, y, width, height]: [f64; 4]| BoundingBox::new(x, y, width, height);
    let Some(prepared) = crate::renderer::static_svg_raster::prepare_static_svg_raster(
        fragment,
        bbox(options.view_box),
        bbox(options.viewport),
        options.scale,
        &options.current_color,
    ) else {
        return JsValue::UNDEFINED;
    };
    let value = js_sys::Object::new();
    let fields = [
        (
            "bytes",
            js_sys::Uint8Array::from(prepared.png.as_slice()).into(),
        ),
        ("byteCost", JsValue::from_f64(prepared.byte_cost as f64)),
        ("width", JsValue::from_f64(prepared.width as f64)),
        ("height", JsValue::from_f64(prepared.height as f64)),
    ];
    for (name, field) in fields {
        if js_sys::Reflect::set(&value, &name.into(), &field).is_err() {
            return JsValue::UNDEFINED;
        }
    }
    value.into()
}
