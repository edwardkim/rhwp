//! WMF 안의 단일 raster 그림에 대한 한컴 실사 색 처리.
//!
//! 다색 경로는 인코딩 RGB 조정 뒤 RGB555 ordered dither를 거친다.
//! 화면 픽셀에는 8단계 dither의 평균을 가장 가까운8비트 값으로 표현한다.
//! 균일한 회색256단계×5조건은 독립 Print JPEG의 양자화까지 대조했다.

use std::collections::HashSet;

use base64::Engine;

use super::{decode_image_with_format_limited, memoized, Conversion, Cursor};
use crate::renderer::svg_fragment::{
    decode_base64_data_url, find_svg_attr_value, try_parse_single_image_data_url,
};

/// 단일 raster WMF의 색 조정을 공통 바이트에 적용한다.
/// 소스 조정색이256색을 넘는 경로에는 RGB555 평균도 반영한다.
/// 확대 후의 팔레트 선택은 별도 검증 중이며 벡터/ROP 합성은 기존 경로를 쓴다.
pub(super) fn adjusted_wmf_picture_svg(
    data: &[u8],
    brightness: i8,
    contrast: i8,
    grayscale: bool,
) -> Option<Vec<u8>> {
    let brightness = brightness.clamp(-100, 100);
    let contrast = contrast.clamp(-100, 100);
    memoized(
        Conversion::WmfPictureTone {
            brightness,
            contrast,
            grayscale,
        },
        data,
        || {
            let converted = crate::renderer::svg::convert_wmf_to_svg(data)?;
            let svg = std::str::from_utf8(&converted).ok()?.trim();
            let body = svg.strip_prefix("<svg ")?;
            let root_end = body.find('>')?;
            let image = body[root_end + 1..].strip_suffix("</svg>")?.trim();
            // 변환기가 생성한 단일 image만 허용한다. 벡터나 ROP 뒤의 효과는
            // 개별 bitmap에 미리 적용하는 것과 교환할 수 없다.
            if image.matches("href=\"").count() != 1 {
                return None;
            }
            // WMF 변환기는 빈 image를 여닫는 두 태그로 기록하기도 한다.
            // 공용 단일-image 검사는 self-closing 표기로 정규화한 뒤 적용한다.
            let normalized_image = if let Some(start) = image.strip_suffix("</image>") {
                format!("{}/>", start.trim_end().strip_suffix('>')?)
            } else {
                image.to_owned()
            };
            let url = try_parse_single_image_data_url(&normalized_image)?;
            let (mime, png) = decode_base64_data_url(url)?;
            if mime != "image/png" {
                return None;
            }
            let mut pixels =
                decode_image_with_format_limited(&png, image::ImageFormat::Png)?.to_rgba8();
            if pixels.pixels().any(|pixel| pixel[3] != 255) {
                return None;
            }
            adjust_pixels(&mut pixels, brightness, contrast, grayscale);
            let mut adjusted = Vec::new();
            pixels
                .write_to(&mut Cursor::new(&mut adjusted), image::ImageFormat::Png)
                .ok()?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(adjusted);
            // viewBox·image 사각형을 유지하고, 새 payload의 원본 픽셀 좌표계를 명시한다.
            // 크기가 없는 SVG의 natural size로 crop하면 브라우저에서 다른 영역이 확대된다.
            let root = &svg[..svg.find('>')? + 1];
            let adjusted = svg.replacen(url, &format!("data:image/png;base64,{encoded}"), 1);
            let adjusted = if find_svg_attr_value(root, "width").is_none()
                && find_svg_attr_value(root, "height").is_none()
            {
                let view_box: Vec<i32> = find_svg_attr_value(root, "viewBox")?
                    .split_ascii_whitespace()
                    .map(str::parse)
                    .collect::<Result<_, _>>()
                    .ok()?;
                let [_, _, width, height]: [i32; 4] = view_box.try_into().ok()?;
                if width <= 0 || height <= 0 {
                    return None;
                }
                adjusted.replacen(
                    "<svg ",
                    &format!("<svg width=\"{width}\" height=\"{height}\" "),
                    1,
                )
            } else {
                adjusted
            };
            Some(adjusted.into_bytes())
        },
    )
}

/// 원본 SVG fallback과 Print 장치 자원이 같은 명암·색 전달을 소비한다.
/// Print에서는 원본 WMF를 장치에 재생한 뒤 이 함수를 호출한다.
pub(crate) fn adjust_pixels(
    pixels: &mut image::RgbaImage,
    brightness: i8,
    contrast: i8,
    grayscale: bool,
) {
    let brightness = brightness.clamp(-100, 100);
    let contrast = contrast.clamp(-100, 100);
    let mut colors = HashSet::with_capacity(257);
    for pixel in pixels.pixels_mut() {
        if pixel[3] == 0 {
            pixel.0[..3].fill(255);
            continue;
        }
        if grayscale {
            let gray =
                (299 * u32::from(pixel[0]) + 587 * u32::from(pixel[1]) + 114 * u32::from(pixel[2]))
                    / 1000;
            pixel.0[..3].fill(gray as u8);
        }
        for channel in &mut pixel.0[..3] {
            *channel = adjusted_wmf_channel(*channel, brightness, contrast);
        }
        if colors.len() <= 256 {
            colors.insert([pixel[0], pixel[1], pixel[2]]);
        }
    }
    if colors.len() > 256 {
        for pixel in pixels.pixels_mut().filter(|pixel| pixel[3] != 0) {
            for channel in &mut pixel.0[..3] {
                *channel = rgb555_dither_mean(*channel);
            }
        }
    }
}

/// 독립256단계 Print에서 확인한 WMF의 정수 백분율 명암 LUT.
/// 중심은128이며 양의 명암 gain을 백분율 정수로 먼저 확정한다.
fn adjusted_wmf_channel(channel: u8, brightness: i8, contrast: i8) -> u8 {
    if contrast == -100 || contrast == 100 {
        return super::picture_adjustments::adjusted_channel(channel, brightness, contrast);
    }
    let c = i64::from(contrast);
    let gain = if c >= 0 { 10000 / (100 - c) } else { 100 + c };
    let numerator = 200 * gain * i64::from(channel)
        + 25600 * (100 - gain)
        + 255 * i64::from(brightness) * (100 + gain);
    (numerator / 20000).clamp(0, 255) as u8
}

/// 8단계 임계값의 RGB555 bit replication 평균.
/// 5비트의 인접한 두 값과 잔여3비트의 비율로 계산한다.
fn rgb555_dither_mean(channel: u8) -> u8 {
    let low = u16::from(channel >> 3);
    let high = (low + 1).min(31);
    let remainder = u16::from(channel & 7);
    let expand = |value: u16| (value << 3) | (value >> 2);
    ((expand(low) * (8 - remainder) + expand(high) * remainder + 4) / 8) as u8
}
