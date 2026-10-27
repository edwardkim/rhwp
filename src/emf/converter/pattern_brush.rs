//! DIB 브러시를 장치 좌표의 반복 타일로 재생한다.

use super::device_context::BitmapPattern;
use crate::emf::parser::records::DibPatternBrush;

pub(super) fn decode(brush: &DibPatternBrush) -> Option<BitmapPattern> {
    // DIB_PAL_COLORS는 별도의 논리 팔레트가 필요하다. RGB 팔레트로 오해하지 않는다.
    if brush.usage != 0 || brush.bmi.len() < 12 {
        return None;
    }
    let header_size = u32::from_le_bytes(brush.bmi[..4].try_into().ok()?);
    let (width, height) = if header_size == 12 {
        (
            u32::from(u16::from_le_bytes(brush.bmi[4..6].try_into().ok()?)),
            u32::from(u16::from_le_bytes(brush.bmi[6..8].try_into().ok()?)),
        )
    } else if header_size >= 40 && brush.bmi.len() >= 40 {
        let width = i32::from_le_bytes(brush.bmi[4..8].try_into().ok()?);
        let height = i32::from_le_bytes(brush.bmi[8..12].try_into().ok()?);
        (u32::try_from(width).ok()?, height.unsigned_abs())
    } else {
        return None;
    };
    if width == 0 || height == 0 {
        return None;
    }
    let data_url = super::player::dib_to_bmp_data_url(&brush.bmi, &brush.bits);
    data_url
        .starts_with("data:image/png;")
        .then_some(BitmapPattern {
            width,
            height,
            data_url,
        })
}

/// 원본 장치 타일을 현재 논리 좌표로 역변환한다. 이후 도형의 변환과 상쇄되므로
/// 월드 변환이 바뀌어도 무늬가 도형과 함께 늘어나지 않는다.
pub(super) fn definition(
    pattern: &BitmapPattern,
    id: &str,
    device_to_logical: super::device_stroke::Matrix,
) -> String {
    let [a, b, c, d, e, f] = device_to_logical;
    let width = pattern.width;
    let height = pattern.height;
    format!(
        "<defs><pattern id=\"{id}\" patternUnits=\"userSpaceOnUse\" width=\"{width}\" height=\"{height}\" patternTransform=\"matrix({a} {b} {c} {d} {e} {f})\"><image width=\"{width}\" height=\"{height}\" preserveAspectRatio=\"none\" href=\"{}\"/></pattern></defs>",
        pattern.data_url,
    )
}
