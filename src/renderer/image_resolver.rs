use std::io::Cursor;

use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

pub(crate) fn resolve_image_payload(image: &ImageNode) -> Option<ResolvedImagePayload> {
    let data = image.data.as_deref()?;
    let mime = detect_image_mime_type(data);

    match mime {
        "image/bmp" => bmp_bytes_to_png_bytes(data).map(|data| ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::FormatConverted,
            suppress_effects: false,
        }),
        "image/x-pcx" => pcx_bytes_to_png_bytes(data).map(|data| ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::FormatConverted,
            suppress_effects: false,
        }),
        "image/jpeg" if is_watermark_image(image) => {
            watermark_jpeg_bytes_to_hancom_baked_png_bytes(data).map(|data| ResolvedImagePayload {
                data,
                mime: "image/png",
                kind: ResolvedImageKind::BakedWatermark,
                suppress_effects: true,
            })
        }
        _ => None,
    }
}

pub(crate) fn image_node_with_resolved_payload(
    image: &ImageNode,
    resolved: Option<&ResolvedImagePayload>,
) -> ImageNode {
    let mut image = image.clone();
    if let Some(payload) = resolved {
        image.data = Some(payload.data.clone());
        if payload.suppress_effects {
            image.effect = ImageEffect::RealPic;
            image.brightness = 0;
            image.contrast = 0;
        }
    }
    image
}

fn is_watermark_image(image: &ImageNode) -> bool {
    !matches!(image.effect, ImageEffect::RealPic) && (image.brightness != 0 || image.contrast != 0)
}

/// BMP 바이트를 PNG 바이트로 재인코딩한다. 실패 시 None 반환.
///
/// 브라우저는 SVG `<image>` 내부의 `data:image/bmp` URI를 표준 지원하지 않으므로,
/// SVG 임베딩 전에 PNG로 변환해 호환성을 확보한다.
pub(crate) fn bmp_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    use image::{load_from_memory_with_format, ImageFormat};

    let img = load_from_memory_with_format(data, ImageFormat::Bmp).ok()?;
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// PCX 바이트를 PNG 바이트로 재인코딩한다. 실패 시 None 반환.
///
/// 브라우저는 PCX 포맷을 native 렌더링하지 못하므로 (구형 ZSoft Paintbrush 포맷),
/// SVG 임베딩 전에 PNG로 변환해 호환성을 확보한다.
/// paletted PCX (8bpp) 와 RGB PCX (24bpp) 모두 지원.
///
/// **투명 처리**: PCX 자체는 알파 채널을 지원하지 않지만, HWP 의 PCX 임베드는
/// 보통 BehindText (글뒤로) 배경/로고 용도로 흰색 (255,255,255) 영역을 투명으로
/// 보여야 한다 (한컴 호환). 변환 시 흰색 픽셀을 투명 알파로 매핑한 RGBA PNG 를
/// 출력한다.
pub(crate) fn pcx_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    use image::{ImageFormat, RgbaImage};

    let mut reader = pcx::Reader::new(Cursor::new(data)).ok()?;
    let width = reader.width() as u32;
    let height = reader.height() as u32;
    if width == 0 || height == 0 {
        return None;
    }
    let pixel_count = (width as usize) * (height as usize);
    let mut rgba = vec![0u8; pixel_count * 4];
    if reader.is_paletted() {
        let row_bytes = width as usize;
        let mut indices = vec![0u8; row_bytes * height as usize];
        for y in 0..height as usize {
            reader
                .next_row_paletted(&mut indices[y * row_bytes..(y + 1) * row_bytes])
                .ok()?;
        }
        let mut palette = vec![0u8; 256 * 3];
        reader.read_palette(&mut palette).ok()?;
        for (dst, &idx) in rgba.chunks_exact_mut(4).zip(indices.iter()) {
            let p = idx as usize * 3;
            let r = palette[p];
            let g = palette[p + 1];
            let b = palette[p + 2];
            dst[0] = r;
            dst[1] = g;
            dst[2] = b;
            dst[3] = if r == 255 && g == 255 && b == 255 {
                0
            } else {
                255
            };
        }
    } else {
        let row_bytes_rgb = width as usize * 3;
        let mut rgb_row = vec![0u8; row_bytes_rgb];
        for y in 0..height as usize {
            reader.next_row_rgb(&mut rgb_row).ok()?;
            for (x, src) in rgb_row.chunks_exact(3).enumerate() {
                let dst = &mut rgba[(y * width as usize + x) * 4..(y * width as usize + x) * 4 + 4];
                dst[0] = src[0];
                dst[1] = src[1];
                dst[2] = src[2];
                dst[3] = if src[0] == 255 && src[1] == 255 && src[2] == 255 {
                    0
                } else {
                    255
                };
            }
        }
    }
    let img = RgbaImage::from_raw(width, height, rgba)?;
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// 워터마크 JPEG 를 한컴 PDF 정답지에 가까운 회색 톤 PNG 로 변환한다.
pub(crate) fn watermark_jpeg_bytes_to_hancom_baked_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    use image::{load_from_memory_with_format, ImageFormat};

    let mut img = load_from_memory_with_format(data, ImageFormat::Jpeg)
        .ok()?
        .to_rgba8();
    let width = img.width();
    let height = img.height();
    if width == 0 || height == 0 {
        return None;
    }

    fn is_near_white(px: [u8; 4]) -> bool {
        px[0] >= 245 && px[1] >= 245 && px[2] >= 245
    }

    let mut border_total = 0u64;
    let mut border_near_white = 0u64;
    for x in 0..width {
        for y in [0, height - 1] {
            border_total += 1;
            if is_near_white(img.get_pixel(x, y).0) {
                border_near_white += 1;
            }
        }
    }
    if height > 2 {
        for y in 1..height - 1 {
            for x in [0, width - 1] {
                border_total += 1;
                if is_near_white(img.get_pixel(x, y).0) {
                    border_near_white += 1;
                }
            }
        }
    }

    let mut all_near_white = 0u64;
    for px in img.pixels() {
        if is_near_white(px.0) {
            all_near_white += 1;
        }
    }

    let pixel_total = (width as u64) * (height as u64);
    if (border_near_white as f64 / border_total as f64) < 0.85
        || (all_near_white as f64 / pixel_total as f64) < 0.20
    {
        return None;
    }

    fn map_watermark_gray(gray: f64) -> u8 {
        let value = if gray < 50.0 {
            198.0 + 0.46 * gray
        } else if gray < 80.0 {
            221.0 + 0.47 * (gray - 50.0)
        } else if gray < 100.0 {
            235.1 + 0.14 * (gray - 80.0)
        } else if gray < 120.0 {
            237.9 + 0.385 * (gray - 100.0)
        } else if gray < 160.0 {
            245.6 + 0.1625 * (gray - 120.0)
        } else {
            252.1 + 0.032 * (gray - 160.0)
        };
        value.clamp(0.0, 255.0).round() as u8
    }

    for px in img.pixels_mut() {
        if is_near_white(px.0) {
            px.0 = [255, 255, 255, 255];
        } else {
            let gray = 0.299 * px.0[0] as f64 + 0.587 * px.0[1] as f64 + 0.114 * px.0[2] as f64;
            let mapped = map_watermark_gray(gray);
            px.0 = [mapped, mapped, mapped, 255];
        }
    }

    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// 이미지 데이터에서 MIME 타입 감지
pub(crate) fn detect_image_mime_type(data: &[u8]) -> &'static str {
    if data.len() >= 8 {
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return "image/png";
        }
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return "image/jpeg";
        }
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return "image/gif";
        }
        if data.starts_with(&[0x42, 0x4D]) {
            return "image/bmp";
        }
        if data.starts_with(&[0xD7, 0xCD, 0xC6, 0x9A])
            || data.starts_with(&[0x01, 0x00, 0x09, 0x00])
        {
            return "image/x-wmf";
        }
        if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
            || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        {
            return "image/tiff";
        }
    }
    if data.len() >= 2 && data.starts_with(&[0x0A, 0x05]) {
        return "image/x-pcx";
    }
    "application/octet-stream"
}
