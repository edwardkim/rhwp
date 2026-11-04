//! CanvasKit도 Native/WebCanvas와 같은 비트맵 축소 결과를 소비한다.

use wasm_bindgen::prelude::*;

/// CanvasKit도 실제 출력 변환과 공통 Rust의 그림 외곽 pen 결과를 소비한다.
#[wasm_bindgen(js_name = printPicturePenWidth)]
pub fn print_picture_pen_width(pen: &[f64], transform: &[f64]) -> Option<f64> {
    let pen: [f64; 2] = pen.try_into().ok()?;
    let transform: [f64; 4] = transform.try_into().ok()?;
    crate::renderer::print_picture_pen::raster_width(
        pen,
        crate::renderer::raster_stroke::minimum_width(transform),
    )
}

#[wasm_bindgen(js_name = prepareBitmapRaster)]
pub fn prepare_bitmap_raster(
    bytes: &[u8],
    destination: &[f64],
    matrix: &[f64],
    profile: &str,
    paper_grid: Option<Vec<f64>>,
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
    let paper_grid = paper_grid.and_then(|values| {
        let values: [f64; 8] = values.try_into().ok()?;
        crate::renderer::bitmap_raster::PrintPaperGrid::new(
            [values[0], values[1]],
            values[2..].try_into().ok()?,
        )
    });
    let Some(prepared) = crate::renderer::bitmap_raster::prepare_bitmap(
        bytes,
        crate::renderer::render_tree::BoundingBox::new(x, y, width, height),
        matrix,
        profile,
        paper_grid,
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
