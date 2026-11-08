//! 조각의 마지막 물리 행 상자를 공유하는 셀의 소유 범위.

/// 원본에서 더 아래까지 이어지는 병합 셀도 현재 조각의 마지막 행을 소유한다.
/// 반복 머리행 여부와 실제 내용·여백에 따른 접기 가능량은 호출자가 확인한다.
pub(super) fn owns_fragment_bottom(first_row: usize, row_span: usize, last_row: usize) -> bool {
    first_row <= last_row && first_row.saturating_add(row_span) > last_row
}
