//! 중첩 표 원점이 문단 세로 오프셋을 이미 소비했는지 보존한다.

use crate::model::table::Table;

/// 문단 원점에는 offset을 더하지만, 저장 표 원점에는 다시 더하지 않는다.
/// 바깥여백은 기존 자식 표 배치가 소비하며 이 결과에 합치지 않는다.
pub(super) enum NestedTableOrigin {
    Paragraph(f64),
    Table(f64),
}

impl NestedTableOrigin {
    pub(super) fn resolve(self, table: &Table, dpi: f64, owns_entry: bool) -> f64 {
        match self {
            Self::Table(y) => y,
            Self::Paragraph(y) => {
                y + if owns_entry {
                    super::table_layout::para_relative_float_table_lead(table, dpi)
                } else {
                    0.0
                }
            }
        }
    }
}
