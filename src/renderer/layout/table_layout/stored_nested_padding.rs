//! 원본 중첩 행의 컷 예약과 배치가 소비하는 칸 안 여백.

use crate::model::table::{Cell, Table};

use super::{LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 컷 예약과 배치가 같은 안 여백을 쓴다.
    ///
    /// [#8067] 예전에는 위·아래 안 여백 합이 칸 선언 높이를 넘으면 그 여백을 선언 높이의
    /// 절반까지 줄이는 가드(#501)가 있었고, 이 함수는 원본 행이 자란 경우 그 축소를
    /// 되돌렸다. 한/글 2024 는 자라는 칸·자라지 않는 칸 모두 저장 안 여백을 그대로 쓴다
    /// (합성 대조). 가드를 없앴으므로 유효 안 여백 하나만 남는다.
    pub(crate) fn resolve_stored_nested_cell_padding(
        &self,
        cell: &Cell,
        table: &Table,
        _styles: &ResolvedStyleSet,
    ) -> (f64, f64, f64, f64) {
        self.resolve_cell_padding(cell, table)
    }
}
