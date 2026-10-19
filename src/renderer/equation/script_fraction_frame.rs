//! 독립 저장·Print로 확인한 분수 첨자의 부모/축소 글자 프레임.
//! 공개 MATH의 분수 축과 원 face의 논리 첨자 프레임을 혼용하지 않는다.
use super::layout::{LayoutBox, LayoutKind};

// 한컴 2020 11.0.0.9136, 저장 bda862b8-b4d3-4a9d-a8aa-86d73f1e6443,
// Print 97ef7c4e-7a3f-4806-b189-b87b65b0551f. 글자 크기, 높이, 기준선 %.
const PARENT: [[u32; 3]; 7] = [
    [600, 1172, 93],
    [612, 1325, 94],
    [900, 1834, 93],
    [1020, 2083, 92],
    [1496, 3155, 93],
    [1500, 3156, 93],
    [2200, 4537, 93],
];
// 실제 축소 글자 크기. 저장 f0a37aee-9f15-4305-8f81-38bd6419c4c5,
// Print 1c26f3fb-2c6e-4543-b4cd-40db98f0851a. 일반 줄 H/BL, 분수 H/BL.
const REDUCED: [[u32; 5]; 7] = [
    [408, 375, 88, 872, 67],
    [416, 450, 83, 1025, 65],
    [612, 600, 86, 1384, 66],
    [694, 675, 86, 1558, 66],
    [1017, 1050, 84, 2405, 65],
    [1020, 1050, 85, 2406, 65],
    [1496, 1500, 85, 3449, 66],
];

pub(super) fn reduced(fs: f64, dpi: f64, fraction: bool) -> Option<(f64, f64)> {
    let unit = super::font_frame_metrics::base_unit_at_dpi(fs, dpi);
    let row = REDUCED.iter().find(|row| row[0] == unit)?;
    let index = if fraction { 3 } else { 1 };
    let height = crate::renderer::hwpunit_to_px(row[index] as i32, dpi);
    Some((height, height * f64::from(row[index + 1]) / 100.0))
}

fn fraction_line(value: &LayoutBox, fs: f64) -> bool {
    match &value.kind {
        LayoutKind::Fraction { numer, denom, .. } => {
            super::fraction_sup_frame::ordinary_line(numer, fs)
                && super::fraction_sup_frame::ordinary_line(denom, fs)
        }
        LayoutKind::Row(items) => {
            let count = items.iter().filter(|v| fraction_line(v, fs)).count();
            count == 1
                && items.iter().all(|v| {
                    fraction_line(v, fs) || super::fraction_sup_frame::ordinary_line(v, fs)
                })
        }
        _ => false,
    }
}

fn trailing_space(value: &LayoutBox) -> bool {
    match &value.kind {
        LayoutKind::Space(_) | LayoutKind::Empty => true,
        LayoutKind::Row(items) => items.last().is_some_and(trailing_space),
        _ => false,
    }
}

pub(super) fn frame(base: &LayoutBox, sup: &LayoutBox, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    if !(super::fraction_sup_frame::ordinary_line(base, fs)
        || matches!(base.kind, LayoutKind::Empty))
        || !fraction_line(sup, sup.font_size)
        || (sup.font_size - fs * 0.68).abs() > f64::EPSILON * fs * 4.0
    {
        return None;
    }
    let unit = super::font_frame_metrics::base_unit_at_dpi(fs, dpi);
    let row = PARENT.iter().find(|row| row[0] == unit)?;
    let line_height = super::font_frame_metrics::line_at_dpi(fs, dpi)?.0;
    // 명시 공백의 nominal strut와 글리프 논리 줄의 차이를 반영한다.
    // 독립 612/2200HU에서 각각 6/12HU이고 나머지 대조 크기는 0이다.
    let strut = if trailing_space(base) {
        ((f64::from(unit) - line_height * crate::renderer::HWPUNIT_PER_INCH / dpi).max(0.0) / 2.0)
            .floor() as i32
    } else {
        0
    };
    let height = crate::renderer::hwpunit_to_px(row[1] as i32 + strut, dpi);
    Some((height, height * f64::from(row[2]) / 100.0))
}

pub(super) fn shift(base: &LayoutBox, sup: &LayoutBox, fs: f64, dpi: f64) -> Option<f64> {
    let (_, baseline) = frame(base, sup, fs, dpi)?;
    Some(baseline - sup.baseline)
}
