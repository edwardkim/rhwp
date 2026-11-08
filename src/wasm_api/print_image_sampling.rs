//! CanvasKit도 Native·Canvas2D와 같은 장치 좌표 필터를 소비한다.
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(js_name = isPrintMetafile)]
pub fn is_print_metafile(data: &[u8]) -> bool {
    crate::paint::metafile_print::is_prepared(data)
}

/// host의 실제 JPEG codec 결과만 받아 Native와 같은 공통 재생을 수행한다.
#[wasm_bindgen(js_name = rasterizePrintMetafile)]
pub fn rasterize_print_metafile(
    data: &[u8],
    matrix: &[f64],
    decode: &js_sys::Function,
) -> Option<Vec<u8>> {
    let raster = crate::paint::metafile_print::rasterize(data, matrix.try_into().ok()?, |jpeg| {
        let header = crate::renderer::image_header::canvaskit_encoded_image_header(jpeg)
            .filter(|header| header.is_within_decode_limits())?;
        let decoded = decode
            .call1(&JsValue::NULL, &js_sys::Uint8Array::from(jpeg))
            .ok()?;
        let decoded = decoded.dyn_into::<js_sys::Uint8Array>().ok()?;
        let length = u64::from(header.width)
            .checked_mul(u64::from(header.height))?
            .checked_mul(4)?;
        if u64::from(decoded.length()) != length {
            return None;
        }
        image::RgbaImage::from_raw(header.width, header.height, decoded.to_vec())
    })?;
    let mut output = Vec::with_capacity(16 + raster.rgba.len());
    for value in [
        raster.left,
        raster.top,
        raster.width as i32,
        raster.height as i32,
    ] {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output.extend_from_slice(&raster.rgba);
    Some(output)
}

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

#[wasm_bindgen(js_name = mapPrintBitmapImage)]
pub fn map_print_bitmap_image(rectangle: &[f64], transform: &[f64]) -> JsValue {
    let (Ok([x, y, width, height]), Ok(matrix)) = (
        <[f64; 4]>::try_from(rectangle),
        <[f64; 6]>::try_from(transform),
    ) else {
        return JsValue::NULL;
    };
    let source = crate::renderer::render_tree::BoundingBox::new(x, y, width, height);
    let Some(mapped) = crate::paint::device_image::bounds(source, matrix) else {
        return JsValue::NULL;
    };
    js_sys::Float64Array::from([mapped.x, mapped.y, mapped.width, mapped.height].as_slice()).into()
}

#[wasm_bindgen(js_name = mapPrintRectangleFill)]
pub fn map_print_rectangle_fill(rectangle: &[f64], transform: &[f64]) -> JsValue {
    let (Ok([x, y, width, height]), Ok(matrix)) = (
        <[f64; 4]>::try_from(rectangle),
        <[f64; 6]>::try_from(transform),
    ) else {
        return JsValue::NULL;
    };
    let source = crate::renderer::render_tree::BoundingBox::new(x, y, width, height);
    let Some(rectangle) = crate::paint::device_fill::bounds(source, matrix) else {
        return JsValue::NULL;
    };
    js_sys::Float64Array::from(
        [rectangle.x, rectangle.y, rectangle.width, rectangle.height].as_slice(),
    )
    .into()
}

/// CanvasKit의 실제 장치 행렬도 Native/Canvas2D와 같은 clip 계산에 전달한다.
#[wasm_bindgen(js_name = mapPrintBitmapClip)]
pub fn map_print_bitmap_clip(rectangle: &[f64], transform: &[f64]) -> JsValue {
    let (Ok([x, y, width, height]), Ok(matrix)) = (
        <[f64; 4]>::try_from(rectangle),
        <[f64; 6]>::try_from(transform),
    ) else {
        return JsValue::NULL;
    };
    let source = crate::renderer::render_tree::BoundingBox::new(x, y, width, height);
    let Some(clip) = crate::paint::device_clip::rectangle(source, matrix) else {
        return JsValue::NULL;
    };
    js_sys::Float64Array::from([clip.x, clip.y, clip.width, clip.height].as_slice()).into()
}

#[wasm_bindgen(js_name = rasterizePrintImageRGBA)]
pub fn rasterize_print_image_rgba(
    rgba: &[u8],
    source_width: u32,
    source_height: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    page_width: f64,
    page_height: f64,
    scale: f64,
) -> Result<JsValue, JsValue> {
    let Some(raster) = crate::renderer::print_image_sampling::rasterize(
        rgba,
        (source_width, source_height),
        (x, y, width, height),
        (page_width, page_height),
        scale,
    ) else {
        return Ok(JsValue::NULL);
    };
    raster_result(&raster)
}

/// encoded straight RGB와 별도 이진 마스크를 공통 Rust에서 표본화한다.
#[wasm_bindgen(js_name = rasterizePrintMaskedImage)]
pub fn rasterize_print_masked_image(
    encoded: &[u8],
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    page_width: f64,
    page_height: f64,
    scale: f64,
) -> Result<JsValue, JsValue> {
    let Some(raster) = crate::renderer::print_image_sampling::rasterize_masked(
        encoded,
        (x, y, width, height),
        (page_width, page_height),
        scale,
    ) else {
        return Ok(JsValue::NULL);
    };
    raster_result(&raster)
}

fn raster_result(
    raster: &crate::renderer::print_image_sampling::DeviceRaster,
) -> Result<JsValue, JsValue> {
    let result = js_sys::Object::new();
    for (name, value) in [
        ("left", f64::from(raster.left)),
        ("top", f64::from(raster.top)),
        ("width", f64::from(raster.width)),
        ("height", f64::from(raster.height)),
    ] {
        js_sys::Reflect::set(&result, &name.into(), &value.into())?;
    }
    let data = js_sys::Uint8Array::from(raster.rgba.as_slice());
    js_sys::Reflect::set(&result, &"data".into(), &data)?;
    Ok(result.into())
}
