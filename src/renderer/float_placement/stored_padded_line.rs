//! 저장 줄 상자가 셀 안 여백까지 포함해 닫힌 경우의 내용 높이.

use crate::model::table::Cell;

/// 호출자가 실제 저장 줄의 원점·순서·유효성을 먼저 검증한다.
/// 셀 전체를 닫는 단일 줄에서 textHeight가 안 여백 안쪽 높이를 직접 기록하면
/// lineHeight에 같은 여백을 다시 더하지 않는다.
pub(super) fn closed_content_height_hu(cell: &Cell, padding: i32) -> Option<i32> {
    let [paragraph] = cell.paragraphs.as_slice() else {
        return None;
    };
    let [line] = paragraph.line_segs.as_slice() else {
        return None;
    };
    (padding > 0
        && line.vertical_pos == 0
        && line.text_height > 0
        && (i64::from(line.line_height) - i64::from(cell.height)).abs() <= 1
        && (i64::from(line.text_height) + i64::from(padding) - i64::from(line.line_height)).abs()
            <= 1)
        .then_some(line.text_height)
}
