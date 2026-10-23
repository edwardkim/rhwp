//! CanvasKit도 Native·Canvas2D와 같은 장치 좌표 필터를 소비한다.
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = resamplePrintFrameRgba)]
pub fn resample_print_frame_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    frame: &[f64],
    scale: &[f64],
) -> Option<Vec<u8>> {
    let frame =
        crate::renderer::print_raster::PrintRasterFrame::from_values(frame.try_into().ok()?)?;
    let [sx, sy] = *scale else {
        return None;
    };
    let raster = frame.rasterize(rgba, (width, height), [sx, sy, 0.0, 0.0])?;
    let mut result = Vec::with_capacity(16 + raster.rgba.len());
    for value in [
        raster.left,
        raster.top,
        raster.width as i32,
        raster.height as i32,
    ] {
        result.extend_from_slice(&value.to_le_bytes());
    }
    result.extend_from_slice(&raster.rgba);
    Some(result)
}

#[wasm_bindgen(js_name = resamplePrintBitmapRgba)]
pub fn resample_print_bitmap_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    destination: &[f64],
) -> Option<Vec<u8>> {
    let [x, y, destination_width, destination_height] = *destination else {
        return None;
    };
    crate::renderer::anisotropic_sampling::rasterize(
        rgba,
        (width, height),
        (x, y, destination_width, destination_height),
    )
    .map(|raster| raster.rgba)
}
