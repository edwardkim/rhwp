//! 같은 런의 오른쪽 탭도 명시적 정지와 자동 우측 탭을 구분한다.

use crate::renderer::TextStyle;

/// 측정과 문자 배치가 소비할 탭 뒤 정렬 단위의 시작점이다.
/// 리더는 채움 모양이며, 명시적 정지를 셀 우단으로 바꾸지 않는다.
pub(super) fn text_start(
    style: &TextStyle,
    current: f64,
    stored_target: f64,
    segment_width: f64,
) -> f64 {
    let (_, _, default_tab_width) = super::style_params(style);
    let (position, kind, _) = super::find_next_tab_stop(
        style.line_x_offset + current,
        &style.tab_stops,
        default_tab_width,
        style.auto_tab_right,
        style.available_width,
        style.text_start_offset,
    );
    if kind == 1 {
        (position - style.line_x_offset - segment_width).max(current)
    } else {
        // 명시적 RIGHT 정지가 없는 유효한 inline 폭은 이미 정렬된 전진이다.
        stored_target.max(current)
    }
}
