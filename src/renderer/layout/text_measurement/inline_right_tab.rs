//! 수용된 저장 RIGHT 전진과 재조판의 정렬 정지를 구분한다.

use crate::renderer::TextStyle;

/// 측정과 문자 배치가 소비할 탭 뒤 정렬 단위의 시작점이다.
/// 리더는 채움 모양이며, 명시적 정지를 셀 우단으로 바꾸지 않는다.
pub(super) fn text_start(
    style: &TextStyle,
    current: f64,
    stored_target: f64,
    segment_width: f64,
    extension: &[u16; 7],
) -> f64 {
    // 교차 런의 StoredStart와 같은 계약이다. 저장 줄의 폭은 정렬을
    // 이미 마친 전진이며, 같은 런이라는 이유로 정지점에서 다시 빼지 않는다.
    // 저장 줄 부재·재조판에서는 생산자가 inline 확장을 제거한다.
    let stored_width_hu = u32::from(extension[0]) | (u32::from(extension[1]) << 16);
    if extension[2] >> 8 == 2
        && stored_width_hu > 0
        && !crate::model::paragraph::tab_ext_is_placeholder(extension)
    {
        return current + f64::from(stored_width_hu) * 96.0 / 7200.0;
    }
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
