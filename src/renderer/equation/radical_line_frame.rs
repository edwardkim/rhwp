//! 같은 한컴 글자 줄을 감싸는 근호의 추가 점유다.
//! 독립 2024 Save/Print의 15개 크기, 무지수·근 지수·중첩 근호를 대조했다.
//! 저장/Print job과 입력·산출 SHA는 #7642 Stage 171의 진단 기록에 있다.

use super::layout::LayoutBox;

// BaseUnit, 일반 글줄 위의 근호 밴드, 이미 근호를 포함한 글줄의 추가 밴드(HU).
// 글자 크기는 폰트 프로그램의 축이다. 실물 문서 좌표나 수식 ID로 선택하지 않는다.
const BANDS: [[u32; 3]; 15] = [
    [600, 90, 90],
    [700, 103, 101],
    [800, 114, 116],
    [900, 135, 135],
    [1000, 148, 146],
    [1100, 156, 158],
    [1200, 180, 180],
    [1300, 193, 191],
    [1400, 198, 200],
    [1500, 225, 225],
    [1600, 238, 236],
    [1800, 270, 270],
    [2000, 282, 284],
    [2200, 328, 326],
    [2400, 360, 360],
];

pub(super) fn top_band(body: &LayoutBox, fs: f64, dpi: f64) -> Option<f64> {
    let extension = super::fraction_sup_frame::line_extension(body, fs, dpi)?;
    let unit = super::font_frame_metrics::base_unit_at_dpi(fs, dpi);
    let row = BANDS.iter().find(|row| row[0] == unit)?;
    let index = if extension > 0.0 { 2 } else { 1 };
    Some(crate::renderer::hwpunit_to_px(row[index] as i32, dpi))
}
