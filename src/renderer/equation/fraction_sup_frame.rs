//! 한컴의 일반 글자 분수 뒤 위첨자 프레임을 배치와 논리 높이에 함께 공급한다.
//! 같은 15개 크기의 짧은 본체·괄호 분수·보통 분수를 독립 저장/Print로 대조했다.

use super::layout::{LayoutBox, LayoutKind};
use super::logical_font_frame::{contains_prime, font_line};

// 한컴 2020 11.0.0.9136, 저장 737de7ec-3d16-4f4f-ae82-68a34d7ed056,
// Print d5266fcf-23bc-42d1-b80a-964998fcacbb의 HWPUNIT 높이.
// 괄호의 유무에 관계없이 같은 분수·위첨자 프로그램의 기준선은 모두 68%다.
// 문서 ID·표현식 문자열·화면 좌표를 키로 사용하지 않는다.
const FRAMES: [[u32; 2]; 15] = [
    [600, 1465],
    [700, 1666],
    [800, 2016],
    [900, 2217],
    [1000, 2417],
    [1100, 2768],
    [1200, 2968],
    [1300, 3169],
    [1400, 3519],
    [1500, 3720],
    [1600, 3920],
    [1800, 4434],
    [2000, 4985],
    [2200, 5386],
    [2400, 5937],
];

fn contains_cjk(value: &LayoutBox) -> bool {
    match &value.kind {
        LayoutKind::Text(text)
        | LayoutKind::Function(text)
        | LayoutKind::FontGlyph { text, .. } => super::text_has_cjk(text),
        LayoutKind::Row(items) => items.iter().any(contains_cjk),
        LayoutKind::Color { body, .. }
        | LayoutKind::FontStyle { body, .. }
        | LayoutKind::Paren { body, .. } => contains_cjk(body),
        _ => false,
    }
}

pub(super) fn ordinary_line(value: &LayoutBox, fs: f64) -> bool {
    font_line(value, fs) && !contains_prime(value) && !contains_cjk(value)
}

/// 같은 글자 줄과 그 줄을 감싼 근호의 추가 점유다.
/// 다줄·분수·크기 변경은 글자 줄로 접지 않는다. 정상화된 줄은 원래 자식과
/// 독립 폰트 프레임의 기준선 연결을 확인하며 자연 glyph 높이와 구별한다.
pub(super) fn line_extension(value: &LayoutBox, fs: f64, dpi: f64) -> Option<f64> {
    if ordinary_line(value, fs) {
        return Some(0.0);
    }
    match &value.kind {
        LayoutKind::Sqrt { index, body }
            if index
                .as_deref()
                .is_none_or(|index| index.y >= 0.0 && index.y + index.height <= value.height)
                && (value.font_size - fs).abs() <= f64::EPSILON * fs * 4.0 =>
        {
            line_extension(body, fs, dpi)?;
            let (line_height, _) = super::font_frame_metrics::line_at_dpi(fs, dpi)?;
            let extension = value.height - line_height;
            (extension.is_finite() && extension >= 0.0).then_some(extension)
        }
        LayoutKind::Color { body, .. } | LayoutKind::FontStyle { body, .. } => {
            line_extension(body, fs, dpi)
        }
        LayoutKind::Row(items) if items.len() == 1 => {
            let child = &items[0];
            let extension = line_extension(child, fs, dpi)?;
            if extension == 0.0 {
                let (height, baseline) = super::font_frame_metrics::line_at_dpi(fs, dpi)?;
                let tolerance = f64::EPSILON * fs * 16.0;
                if (value.height - height).abs() > tolerance
                    || (value.baseline - baseline).abs() > tolerance
                    || (child.y + child.baseline - baseline).abs() > tolerance
                {
                    return None;
                }
            } else {
                let tolerance = f64::EPSILON * value.height.abs().max(fs) * 16.0;
                if (value.height - child.height).abs() > tolerance
                    || (value.baseline - child.baseline).abs() > tolerance
                    || child.y.abs() > tolerance
                {
                    return None;
                }
            }
            Some(extension)
        }
        _ => None,
    }
}

/// 분자 위쪽 밴드와 분모 아래쪽 점유를 같은 논리 분수 프레임으로 연결한다.
pub(super) fn source_fraction_frame(
    numer: &LayoutBox,
    denom: &LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    let above = line_extension(numer, fs, dpi)?;
    let below = line_extension(denom, fs, dpi)?;
    let (height, baseline) = super::font_frame_metrics::fraction_at_dpi(fs, dpi)?;
    Some((height + above + below, baseline + above))
}

fn fraction_extensions(value: &LayoutBox, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    match &value.kind {
        LayoutKind::Fraction { numer, denom, .. } => Some((
            line_extension(numer, fs, dpi)?,
            line_extension(denom, fs, dpi)?,
        )),
        LayoutKind::Color { body, .. }
        | LayoutKind::FontStyle { body, .. }
        | LayoutKind::Paren { body, .. } => fraction_extensions(body, fs, dpi),
        LayoutKind::Row(items) if items.len() == 1 => fraction_extensions(&items[0], fs, dpi),
        _ => None,
    }
}

pub(super) fn frame(base: &LayoutBox, sup: &LayoutBox, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    let (above, below) = fraction_extensions(base, fs, dpi)?;
    if !ordinary_line(sup, sup.font_size)
        || (sup.font_size - fs * 0.68).abs() > f64::EPSILON * fs * 4.0
    {
        return None;
    }
    let unit = super::font_frame_metrics::base_unit_at_dpi(fs, dpi);
    let row = FRAMES.iter().find(|row| row[0] == unit)?;
    let height = crate::renderer::hwpunit_to_px(row[1] as i32, dpi);
    // 한컴 2024 재생성·Print에서 근호 분모의 600/900/1500HU 높이는
    // 보통 분수 첨자보다 각각 90/135/225HU 크다. 하강 점유를 늘리되
    // 첨자를 본체 중앙의 일반 글자 기준선으로 되돌리지 않는다.
    Some((height + above + below, height * 0.68 + above))
}

pub(super) fn shift(base: &LayoutBox, sup: &LayoutBox, fs: f64, dpi: f64) -> Option<f64> {
    let (_, fraction_baseline) = frame(base, sup, fs, dpi)?;
    let (sup_height, sup_baseline) = super::font_frame_metrics::superscript_at_dpi(fs, dpi)?;
    let (line_height, line_baseline) = super::font_frame_metrics::line_at_dpi(fs, dpi)?;
    // 독립 Print에서 짧은 본체와 분수의 첨자 상단은 같은 글리프 프로그램이다.
    // 기존 단일 글자 첨자의 기준선 위치를 소비해 큰 본체의 shift를 해소한다.
    let plain_shift = sup_height - line_height + line_baseline * (1.0 - sup.font_size / fs);
    Some(fraction_baseline - sup_baseline + plain_shift)
}

/// 독립 출력에서 분수의 아래첨자는 일반 아래첨자와 같은 하강 확장을 예약한다.
pub(super) fn subscript_frame(
    base: &LayoutBox,
    sub: &LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    if !ordinary_line(sub, sub.font_size)
        || (sub.font_size - fs * 0.68).abs() > f64::EPSILON * fs * 4.0
    {
        return None;
    }
    let (above, below) = fraction_extensions(base, fs, dpi)?;
    let (fraction_height, fraction_baseline) = super::font_frame_metrics::fraction_at_dpi(fs, dpi)?;
    let (sub_height, _) = super::font_frame_metrics::subscript_at_dpi(fs, dpi)?;
    let (line_height, _) = super::font_frame_metrics::line_at_dpi(fs, dpi)?;
    Some((
        fraction_height + above + below + sub_height - line_height,
        fraction_baseline + above,
    ))
}

pub(super) fn subscript_geometry(
    base: &LayoutBox,
    sub: &LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    let (height, _) = subscript_frame(base, sub, fs, dpi)?;
    let (line_height, line_baseline) = super::font_frame_metrics::line_at_dpi(fs, dpi)?;
    let sub_depth = (line_height - line_baseline) * sub.font_size / fs;
    Some((height - sub_depth - sub.baseline, height))
}

pub(super) fn subsup_frame(
    base: &LayoutBox,
    sub: &LayoutBox,
    sup: &LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    subscript_frame(base, sub, fs, dpi)?;
    let (height, baseline) = frame(base, sup, fs, dpi)?;
    let (sub_height, _) = super::font_frame_metrics::subscript_at_dpi(fs, dpi)?;
    let (line_height, _) = super::font_frame_metrics::line_at_dpi(fs, dpi)?;
    Some((height + sub_height - line_height, baseline))
}
