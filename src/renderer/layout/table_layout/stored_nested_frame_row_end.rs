//! 칸 안 중첩 표의 저장 쪽 프레임 경계에서 끊긴 바깥 행 조각의 물리 상자.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::table::Table;

impl LayoutEngine {
    /// [#9000] 한/글이 바깥 행의 한 칸 안 중첩 표 행을 다음 쪽 프레임에서 다시
    /// 시작하도록 저장했으면(되감김 + 저장 프레임 유닛) 이번 쪽의 바깥 행 조각은
    /// 내용 끝이 아니라 남은 선언 행 높이만큼 상자를 소유한다(쪽 물리 프레임이 상한,
    /// 내용이 하한). KAF 한/글 2020 PDF 34~42쪽은 이어지는 행 조각의 아래 괘선을 모두
    /// 695px(본문 끝 − 바깥 아래 여백)에 긋고, 앞 조각들이 선언 행 높이를 거의 다
    /// 소비한(남은 21.75px) 43쪽은 내용 끝 681px 에서 닫는다. 글 프레임 칸의 같은
    /// 계약(`stored_ordinary_row_end_frame`)은 글 칸만 인증하므로 중첩 표 칸의 저장
    /// 프레임 유닛을 별도 근거로 읽는다.
    pub(crate) fn stored_nested_frame_row_end(
        &self,
        table: &Table,
        row: usize,
        cut: (&[usize], &[usize]),
        available: f64,
        carried: Option<f64>,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let (start, end) = cut;
        let profile = self.profile.get();
        if !profile.hwp5_stored_pagination_layout()
            || profile.session_edited()
            || !available.is_finite()
            || !self.stored_table_has_physical_text_frames(table, styles)
            || table.cells.iter().any(|cell| {
                cell.row_span > 1
                    && usize::from(cell.row) <= row
                    && row < usize::from(cell.row) + usize::from(cell.row_span)
            })
        {
            return None;
        }
        let owners = self.indexed_row_cut_cell_order(table, row);
        if owners.is_empty() || owners.len() != end.len() {
            return None;
        }
        let opens_next_frame = owners.iter().zip(end).any(|(&index, &next)| {
            let cell = &table.cells[index];
            !cell.dirty_flag
                && self
                    .cell_units(cell, table, styles)
                    .get(next)
                    .is_some_and(|unit| {
                        unit.nested_row.is_some()
                            && unit.hard_break_before
                            && unit.stored_frame_break_before
                    })
        });
        if !opens_next_frame {
            return None;
        }
        // 첫 조각은 원본 행 선언 전체, 이어받은 조각은 앞 조각을 차감한 잔여를 쓴다.
        let full = if start.is_empty() {
            owners
                .iter()
                .map(|&index| table.cells[index].height)
                .filter(|&height| height <= i32::MAX as u32)
                .map(|height| crate::renderer::hwpunit_to_px(height as i32, self.dpi))
                .fold(0.0_f64, f64::max)
        } else {
            carried.filter(|height| height.is_finite() && *height > 0.0)?
        };
        let content = self.row_cut_content_height(table, row, start, end, styles);
        (content <= available + 0.01).then_some(full.min(available).max(content))
    }
}
