//! 하나의 저장 문단이 여러 물리 단·쪽 프레임에 걸치는지 판정한다.

use crate::model::paragraph::Paragraph;

/// 문단의 마지막 줄뿐 아니라 전체 사다리의 프레임 소유를 확인한다.
/// 같은 높이의 FIRST..LAST 조각은 별도 단 이동으로 보지 않는다.
pub(crate) fn crosses_physical_frame(paragraph: &Paragraph) -> bool {
    paragraph.line_segs.windows(2).any(|pair| {
        pair[1].is_first_line_of_page()
            || pair[1].is_first_line_of_column()
            || pair[1].vertical_pos < pair[0].vertical_pos
    })
}
