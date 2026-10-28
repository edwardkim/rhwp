//! Native PNG의 Kitty graphics 직접 전송. 조판이나 terminal 응답을 다루지 않는다.

use std::io::{self, Write};

use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::error::HwpError;
use crate::renderer::layer_renderer::{LayerRenderResult, RasterRenderOptions};
use crate::renderer::raster_pixels::RasterPixels;

/// 기존 premultiplied RGBA를 흰색에 합성하고 기존 Skia PNG 인코더에 전달한다.
pub fn encode_kitty_png(pixels: &RasterPixels) -> LayerRenderResult<Vec<u8>> {
    pixels.validate()?;
    let limits = RasterRenderOptions::default();
    if pixels.width > limits.max_dimension
        || pixels.height > limits.max_dimension
        || pixels.width as u64 * pixels.height as u64 > limits.max_pixels
    {
        return Err(HwpError::RenderError(
            "Kitty raster size exceeds output limits".into(),
        ));
    }
    let opaque = RasterPixels {
        bytes: pixels.opaque_rgba_on_white()?,
        width: pixels.width,
        height: pixels.height,
        stride: pixels.width as usize * 4,
        format: pixels.format,
        color_space: pixels.color_space,
        dpi: pixels.dpi,
    };
    crate::renderer::skia::SkiaLayerRenderer::encode_pixels_png(&opaque).map(|output| output.bytes)
}

/// 인코딩이 끝난 PNG를 최대 4096바이트 Base64 payload로 전송한다.
/// 호출자는 쓰기 실패와 flush를 처리한다. 문서 이름·본문은 제어 데이터에 포함하지 않는다.
pub fn write_kitty_png<W: Write>(png: &[u8], writer: &mut W) -> io::Result<()> {
    if png.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty Kitty PNG",
        ));
    }
    // 3072 raw bytes become exactly 4096 Base64 bytes, without line wrapping.
    let mut encoded = [0u8; 4096];
    let count = png.len().div_ceil(3072);
    for (index, chunk) in png.chunks(3072).enumerate() {
        let more = u8::from(index + 1 < count);
        if index == 0 {
            write!(writer, "\x1b_Ga=T,f=100,t=d,q=2,m={more};")?;
        } else {
            write!(writer, "\x1b_Gm={more},q=2;")?;
        }
        let length = STANDARD
            .encode_slice(chunk, &mut encoded)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        writer.write_all(&encoded[..length])?;
        writer.write_all(b"\x1b\\")?;
    }
    writer.write_all(b"\r\n")
}
