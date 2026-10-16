//! CanvasKit도 Native/WebCanvas와 같은 비트맵 축소 결과를 소비한다.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = prepareBitmapRaster)]
pub fn prepare_bitmap_raster(
    bytes: &[u8],
    destination: &[f64],
    matrix: &[f64],
    profile: &str,
) -> JsValue {
    let (Ok(destination), Ok(matrix)) = (
        <[f64; 4]>::try_from(destination),
        <[f64; 6]>::try_from(matrix),
    ) else {
        return JsValue::UNDEFINED;
    };
    let profile = match profile {
        "print" => crate::paint::RenderProfile::Print,
        "highQuality" => crate::paint::RenderProfile::HighQuality,
        _ => return JsValue::UNDEFINED,
    };
    let [x, y, width, height] = destination;
    let Some(prepared) = crate::renderer::bitmap_raster::prepare_bitmap(
        bytes,
        crate::renderer::render_tree::BoundingBox::new(x, y, width, height),
        matrix,
        profile,
    ) else {
        return JsValue::UNDEFINED;
    };
    let value = js_sys::Object::new();
    let bytes = js_sys::Uint8Array::from(prepared.bytes.as_slice());
    let bbox = js_sys::Float64Array::from(
        [
            prepared.bbox.x,
            prepared.bbox.y,
            prepared.bbox.width,
            prepared.bbox.height,
        ]
        .as_slice(),
    );
    if js_sys::Reflect::set(&value, &"bytes".into(), &bytes).is_err()
        || js_sys::Reflect::set(&value, &"bbox".into(), &bbox).is_err()
    {
        return JsValue::UNDEFINED;
    }
    value.into()
}
