//! 한컴 Print에서 확인한 그림의 인코딩 RGB 색조. 모든 출력 경로가 같은 픽셀을 쓴다.

use super::{decode_image_with_format_limited, memoized, Conversion, Cursor, ImageEffect};
use crate::renderer::render_tree::PageBackgroundImage;

fn supports_adjustment(effect: ImageEffect, brightness: i8, contrast: i8) -> bool {
    (brightness != 0 || contrast != 0)
        && matches!(
            effect,
            ImageEffect::RealPic | ImageEffect::GrayScale | ImageEffect::BlackWhite
        )
        && !(effect == ImageEffect::BlackWhite && brightness != 0 && contrast != 0)
}

/// 쪽 배경의 RealPic도 저장된 밝기·대비를 원본 해상도에서 한 번만 적용한다.
/// 독립 Print는 보정된 RGB를 불투명으로 전달하며 별도의 워터마크 alpha를 쓰지 않는다.
/// 원본 model은 보존하고, 모든 backend가 소비할 render-node 픽셀과 인자만 정규화한다.
pub(crate) fn resolve_page_background_tone(mut image: PageBackgroundImage) -> PageBackgroundImage {
    if image.effect == ImageEffect::RealPic {
        let (brightness, contrast) = image.display_brightness_contrast();
        if let Some(data) = adjusted_picture_bytes(&image.data, image.effect, brightness, contrast)
        {
            image.data = data;
            image.brightness = 0;
            image.contrast = 0;
        }
    }
    image
}

/// Inline payload와 지연 로딩 키가 공유하는 일반 raster 색조 조건이다.
pub(crate) fn picture_tone_parameters(image: &super::ImageNode) -> Option<(i8, i8, u8)> {
    if super::is_watermark_image(image)
        || image.is_real_picture_watermark_tone_preset()
        || !supports_adjustment(image.effect, image.brightness, image.contrast)
        || !matches!(
            super::detect_image_mime_type(image.data.as_deref()?),
            "image/png"
                | "image/jpeg"
                | "image/bmp"
                | "image/tiff"
                | "image/x-pcx"
                | "image/gif"
                | "image/webp"
        )
    {
        return None;
    }
    Some((
        image.brightness.clamp(-100, 100),
        image.contrast.clamp(-100, 100),
        image.effect as u8,
    ))
}

pub(super) fn picture_tone_effect(effect: u8) -> Option<ImageEffect> {
    match effect {
        0 => Some(ImageEffect::RealPic),
        1 => Some(ImageEffect::GrayScale),
        2 => Some(ImageEffect::BlackWhite),
        _ => None,
    }
}

pub(super) fn resolve_picture_tone(
    image: &super::ImageNode,
    base: Option<super::ResolvedImagePayload>,
) -> Option<super::ResolvedImagePayload> {
    if let Some((brightness, contrast, effect)) = picture_tone_parameters(image) {
        let data = base
            .as_ref()
            .map(|p| p.data.as_slice())
            .or(image.data.as_deref());
        if let Some(data) = data.and_then(|data| {
            adjusted_picture_bytes(data, picture_tone_effect(effect)?, brightness, contrast)
        }) {
            return Some(super::ResolvedImagePayload {
                data,
                mime: "image/png",
                kind: super::ResolvedImageKind::FormatConverted,
                suppress_effects: true,
            });
        }
    }
    base
}

/// 일반 raster/회색조/흑백의 밝기·대비를 원본 해상도에서 한 번 적용한다.
/// 워터마크 프리셋 등 이미 bake된 결과는 호출자가 제외한다.
pub(crate) fn adjusted_picture_bytes(
    data: &[u8],
    effect: ImageEffect,
    brightness: i8,
    contrast: i8,
) -> Option<Vec<u8>> {
    if !supports_adjustment(effect, brightness, contrast) {
        return None;
    }
    let brightness = brightness.clamp(-100, 100);
    let contrast = contrast.clamp(-100, 100);
    memoized(
        Conversion::PictureAdjustment {
            effect: effect as u8,
            brightness,
            contrast,
        },
        data,
        || {
            let format = image::guess_format(data).ok()?;
            let mut pixels = decode_image_with_format_limited(data, format)?.to_rgba8();
            for pixel in pixels.pixels_mut() {
                if matches!(effect, ImageEffect::GrayScale | ImageEffect::BlackWhite) {
                    // 한컴은 회색조를 먼저 바이트로 확정한 뒤 밝기·대비를 적용한다.
                    let gray = (299 * u32::from(pixel[0])
                        + 587 * u32::from(pixel[1])
                        + 114 * u32::from(pixel[2]))
                        / 1000;
                    pixel.0[..3].fill(gray as u8);
                }
                for channel in &mut pixel.0[..3] {
                    *channel = adjusted_channel(*channel, brightness, contrast);
                }
                if effect == ImageEffect::BlackWhite {
                    // 독립 Print의 B=0/10/50 대조군: 색조를 적용한 회색 바이트의
                    // 128 이하는 검정, 초과는 흰색이다. 먼저 이산화하면 중간 회색이 남는다.
                    let value = if pixel[0] <= 128 { 0 } else { 255 };
                    pixel.0[..3].fill(value);
                }
            }
            let mut png = Vec::new();
            pixels
                .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
                .ok()?;
            Some(png)
        },
    )
}

pub(super) fn adjusted_channel(channel: u8, brightness: i8, contrast: i8) -> u8 {
    let value = i64::from(channel);
    let b = i64::from(brightness);
    let c = i64::from(contrast);
    // a = c>=0 ? 1/(1-c) : 1+c
    // d = (1-a)/2 + b*(1+a)/2, 인코딩 RGB의 a*x + 255*d.
    // 정수 유리식으로 계산해 마지막 floor 전에 부동소수점 반올림을 피한다.
    // 독립 Print의 부호 교차8조건×11색은 이 식과±1 이내다(2단계 실행 기록).
    // 끝값은 중간 범위 선형식의 극한이 아니다. 실제 한컴 Print의 별도
    // 양자화 결과(2단계 boundary job)를 보존한 뒤 밝기를 적용한다.
    let (numerator, denominator) = if c == 100 || c == -100 {
        let quantized = if c == -100 {
            1
        } else if channel > 128 {
            255
        } else {
            0
        };
        return ((quantized * 100 + b * 255) / 100).clamp(0, 255) as u8;
    } else if c >= 0 {
        (
            20000 * value + 255 * (b * (200 - c) - 100 * c),
            200 * (100 - c),
        )
    } else {
        (
            200 * (100 + c) * value + 255 * (b * (200 + c) - 100 * c),
            20000,
        )
    };
    (numerator / denominator).clamp(0, 255) as u8
}
