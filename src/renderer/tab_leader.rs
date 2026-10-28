//! 점 자체의 glyph 원점과 리더가 채우는 폭을 분리한다.
//!
//! 독립 한양/Poppy 9·13·18pt, 세 접두 길이·세 자리 수 표본의 Print는
//! 1/4em 셀 수를 먼저 정하고 남는 폭을 그 셀에 배분한다. 원 glyph의
//! 가운뎃점 중심은 원점에서 0.192em이다. 글줄·탭 정지의 위치는 바꾸지 않는다.

pub(crate) const MAX_DOT_CENTERS: usize = 4096;

pub(crate) struct DotLeaderGeometry {
    pub centers: Vec<f64>,
    pub radius: f64,
}

pub(crate) fn dot_geometry(font_size: f64, start: f64, end: f64) -> Option<DotLeaderGeometry> {
    if !font_size.is_finite() || font_size <= 0.0 || !start.is_finite() || !end.is_finite() {
        return None;
    }
    let width = (end - start).max(0.0);
    let count = (width / (font_size * 0.25)).floor();
    if !count.is_finite() || count > MAX_DOT_CENTERS as f64 {
        return None;
    }
    let count = count as usize;
    let step = if count == 0 {
        0.0
    } else {
        width / count as f64
    };
    Some(DotLeaderGeometry {
        centers: (0..count)
            .map(|index| start + index as f64 * step + font_size * 0.192)
            .collect(),
        radius: font_size * super::render_tree::MIDDLE_DOT_RADIUS_EM,
    })
}
