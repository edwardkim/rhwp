use crate::model::paragraph::tab_ext_is_placeholder;
use crate::model::style::Alignment;
use crate::renderer::TabStop;

/// 재계산이 필요한 저장 캐시와 유효한 인라인 전진폭을 구분한다.
pub(super) fn geometry_inline_tabs(
    extensions: &[[u16; 7]],
    first: usize,
    stored_geometry_valid: bool,
) -> Vec<[u16; 7]> {
    if stored_geometry_valid {
        extensions.iter().skip(first).copied().collect()
    } else {
        Vec::new()
    }
}

/// 왼쪽/기본 정지로 자연 줄을 구성한 뒤에는 줄 전체를 정렬 이동한다.
/// 단에 맞추는 오른쪽/가운데 정지는 해당 단의 좌표 계약을 유지한다.
pub(super) fn tab_frame_translates_with_line(
    alignment: Alignment,
    auto_tab_right: bool,
    stops: &[TabStop],
    extensions: &[[u16; 7]],
) -> bool {
    matches!(alignment, Alignment::Center | Alignment::Right)
        && !auto_tab_right
        && !stops.iter().any(|stop| matches!(stop.tab_type, 1 | 2))
        && !extensions.iter().any(|extension| {
            !tab_ext_is_placeholder(extension)
                && (matches!(extension[2] >> 8, 2 | 3) || matches!(extension[2], 1 | 2))
        })
}
