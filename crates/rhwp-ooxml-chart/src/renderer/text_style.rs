//! 제목의 명시 서식과 차트 기본 글자에서 정하는 자동 서식을 구분한다.

use super::{hancom_default, OoxmlChart, PX_PER_PT};

/// 축 간격 측정과 실제 숫자 표시가 같은 글줄 점유 높이를 사용한다.
pub(super) fn axis_line_height(font: f64) -> f64 {
    font * crate::legend_layout::TEXT_LINE_EM
}

/// 제목에 직접 지정한 크기를 우선한다. 자동 제목은 차트 전체 글자의
/// 정수 pt 단계에 따른 제목 프리셋을 사용하며 나머지는 20% 확대한다.
/// 기본 글자 10pt에서 관측한 제목 14pt를 다른 크기에 그대로 적용하지 않는다.
pub(super) fn title_font_px(chart: &OoxmlChart) -> f64 {
    let title_pt = chart.title_size_pt.unwrap_or_else(|| {
        let base = chart
            .text_size_pt
            .unwrap_or(hancom_default::LABEL_PT)
            .trunc();
        match base as u32 {
            8 => 12.0,
            10 => hancom_default::TITLE_PT,
            12 => 18.0,
            18 => 21.0,
            _ => (base * 1.2 * 100.0).trunc() / 100.0,
        }
    });
    title_pt * PX_PER_PT
}
