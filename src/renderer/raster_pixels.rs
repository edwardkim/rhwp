//! 인코더와 무관한 CPU 래스터 계약. N32의 플랫폼별 채널 순서를 노출하지 않는다.

use crate::error::HwpError;
use crate::renderer::layer_renderer::{LayerRenderResult, RasterColorSpace};

/// 채널 순서는 바이트 R, G, B, A이며 RGB에 alpha가 이미 곱해져 있다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterPixelFormat {
    Rgba8Premultiplied,
}

/// 위에서 아래로 저장한 sRGB 픽셀. `stride`는 한 행의 바이트 수이며 패딩을 포함한다.
/// PNG는 alpha를 유지하고, 불투명 출력은 인코딩 전에 명시적으로 합성한다.
#[derive(Debug, Clone, PartialEq)]
pub struct RasterPixels {
    pub bytes: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub stride: usize,
    pub format: RasterPixelFormat,
    pub color_space: RasterColorSpace,
    pub dpi: Option<f64>,
}

impl RasterPixels {
    pub fn validate(&self) -> LayerRenderResult<()> {
        let invalid =
            || HwpError::RenderError("invalid RGBA raster dimensions/stride/buffer".into());
        if self.width <= 0 || self.height <= 0 {
            return Err(invalid());
        }
        let row_bytes = (self.width as usize).checked_mul(4).ok_or_else(invalid)?;
        let buffer_bytes = self
            .stride
            .checked_mul(self.height as usize)
            .ok_or_else(invalid)?;
        if self.stride < row_bytes || self.bytes.len() != buffer_bytes {
            return Err(invalid());
        }
        Ok(())
    }

    /// 선형화하지 않은 sRGB 바이트 공간에서 흰색에 source-over 합성한다.
    /// Skia의 기존 N32 canvas 합성과 같은 표현이다. premul RGB + (255-alpha).
    /// 완전 투명 픽셀은 흰색, 반투명 픽셀은 halo 없이 불투명 RGBA가 된다.
    pub fn opaque_rgba_on_white(&self) -> LayerRenderResult<Vec<u8>> {
        self.validate()?;
        let mut rgba = Vec::with_capacity(self.width as usize * self.height as usize * 4);
        for row in self.bytes.chunks_exact(self.stride) {
            for pixel in row[..self.width as usize * 4].chunks_exact(4) {
                if pixel[..3].iter().any(|channel| *channel > pixel[3]) {
                    return Err(HwpError::RenderError(
                        "invalid premultiplied RGBA channel".into(),
                    ));
                }
                let white = 255 - pixel[3];
                rgba.extend_from_slice(&[
                    pixel[0] + white,
                    pixel[1] + white,
                    pixel[2] + white,
                    255,
                ]);
            }
        }
        Ok(rgba)
    }
}
