//! 셀 생성용 기본 높이와 병합 칸이 정한 행을 구분한다.

use crate::model::table::Table;

use super::LayoutEngine;

impl LayoutEngine {
    /// 내용·선언 제약을 모두 적용한 뒤에도 높이가 없는 행의 높이다.
    ///
    /// 여러 행에 걸친 칸이 덮는 행은 `solve_rowspan_row_heights` 가 경계를 위에서부터 가장
    /// 이른 위치로 정한 결과다. 걸친 칸이 그 행을 요구하지 않으면 0 이 정답이다 — 한/글
    /// 2022 Print 는 `46551_도로교통법 시행규칙 일부개정령안(입법예고안).hwp` 28쪽 표의 rs=1 칸
    /// 없는 행 16·17·19·24·26 을 0.0~0.3px 로 그린다. `1342000_edu_curriculum_map.hwp` 95쪽
    /// 표의 행 28 에 보이는 1.28px 띠도 기본값이 아니라 행 27~28 칸(c1·c6)의 내용 높이
    /// 22.2px 가 행 27(20.9px)을 넘는 만큼이며, 병합 칸 내용 요구 단계가 그 행에 더한다.
    ///
    /// 어떤 칸도 높이를 정하지 않은 행에만 셀 생성용 400HU 상자를 쓴다.
    pub(crate) fn unresolved_row_default_hu(table: &Table, row: usize) -> i32 {
        let mut spanned = false;
        for cell in &table.cells {
            let start = usize::from(cell.row);
            let end = start + usize::from(cell.row_span.max(1));
            if !(start <= row && row < end) {
                continue;
            }
            if cell.row_span <= 1 {
                return 400;
            }
            spanned = true;
        }
        if spanned {
            0
        } else {
            400
        }
    }
}
