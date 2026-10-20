//! Native raster → 불투명 Sixel. 문서 조판과 terminal 상태를 변경하지 않는다.

use crate::error::HwpError;
use crate::renderer::layer_renderer::{LayerRenderResult, RasterRenderOptions};
use crate::renderer::raster_pixels::RasterPixels;
use icy_sixel::{EncodeOptions, QuantizeMethod, SixelImage};

/// Wu 256색, 디더링 없음: 글자·표 경계에 새 점을 확산시키지 않는다.
/// 실제 터미널의 팔레트 지원은 사용자가 선택한 terminal의 계약이다.
pub fn encode_sixel(pixels: &RasterPixels) -> LayerRenderResult<Vec<u8>> {
    pixels.validate()?;
    let limits = RasterRenderOptions::default();
    // sixel의 마지막 6줄 밴드까지 포함하여 인코더의 64 Mi-pixel 한도를 확인한다.
    let band_height = (pixels.height as u64).div_ceil(6) * 6;
    if pixels.width > limits.max_dimension
        || pixels.height > limits.max_dimension
        || pixels.width as u64 * band_height > limits.max_pixels
    {
        return Err(HwpError::RenderError(
            "Sixel raster size exceeds output limits".into(),
        ));
    }
    let image = SixelImage::try_from_rgba(
        pixels.opaque_rgba_on_white()?,
        pixels.width as usize,
        pixels.height as usize,
    )
    .map_err(|error| HwpError::RenderError(format!("Sixel input: {error}")))?;
    image
        .encode_with(&EncodeOptions {
            max_colors: 256,
            diffusion: 0.0,
            quantize_method: QuantizeMethod::Wu,
        })
        .map(String::into_bytes)
        .map_err(|error| HwpError::RenderError(format!("Sixel encoding: {error}")))
}
