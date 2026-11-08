//! 한컴이 재생성한 괄호·prime·이탤릭 f의 논리 전진폭.
//! 프레임·저장 줄이 없는 56개 수식, MCP2020 job
//! 32f965f4-1f99-4c5b-b290-b2864ecca45c와 Print의 글리프 원점에 근거한다.
//! 폰트 바이트·윤곽이나 실물 문서의 px 위치를 포함하지 않는다.

// BaseUnit, 둥근 괄호, 단일 prime, 이탤릭 f의 HWPUNIT 전진폭.
const ADVANCES: [[u32; 4]; 14] = [
    [600, 225, 150, 300],
    [700, 300, 225, 375],
    [800, 300, 225, 375],
    [900, 375, 300, 450],
    [1000, 375, 300, 600],
    [1100, 375, 300, 600],
    [1200, 450, 375, 675],
    [1300, 525, 375, 750],
    [1400, 525, 375, 750],
    [1500, 600, 375, 825],
    [1600, 675, 450, 900],
    [1800, 675, 450, 975],
    [2000, 750, 525, 1050],
    [2400, 900, 675, 1275],
];

pub(super) fn measured(fs: f64, dpi: f64, index: usize) -> Option<f64> {
    let size = super::super::font_frame_metrics::base_unit_at_dpi(fs, dpi);
    let row = ADVANCES.iter().find(|row| row[0] == size)?;
    Some(crate::renderer::hwpunit_to_px(row[index] as i32, dpi))
}

pub(super) fn paren(fs: f64, dpi: f64) -> f64 {
    // 독립 22pt LEFT(x)의 3075HU에서 x 1275HU를 뺀 두 슬롯이다.
    // MCP 정상 저장·Print job: 2a652b98-863c-4397-97a0-d19ea108ad1e.
    if super::super::font_frame_metrics::base_unit_at_dpi(fs, dpi) == 2200 {
        return crate::renderer::hwpunit_to_px(900, dpi);
    }
    measured(fs, dpi, 1).unwrap_or_else(|| super::source_font_text::logical_advance(398, fs, dpi))
}

/// CASES와 LEFT/RIGHT curly는 같은 논리 em의 절반을 예약한다.
/// 독립 22pt 3/4행 저장 폭에서 각 슬롯 1125HU를 확인했다.
pub(super) fn curly_slot(fs: f64, dpi: f64) -> f64 {
    let em = super::source_roman_advance::em_advance(fs, dpi).unwrap_or(fs);
    super::source_font_text::logical_advance(512, em, dpi)
}

/// 독립 한컴 저장·Print에서 확인한 절댓값 구분자의 논리 슬롯이다.
pub(super) fn vertical_slot(fs: f64, dpi: f64) -> Option<f64> {
    let size = super::super::font_frame_metrics::base_unit_at_dpi(fs, dpi);
    let advance = match size {
        900 => 450,
        1100 => 525,
        1600 => 900,
        2200 => 1125,
        _ => return None,
    };
    Some(crate::renderer::hwpunit_to_px(advance, dpi))
}
