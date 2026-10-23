//! RowBreak 조각의 내용 수용 하단과 빈 종료 띠의 하단.

use super::{hwpunit_to_px, single_cell_page_fragment_bottom, Table};

pub(crate) struct RowbreakFragmentBounds {
    /// 실제 글줄을 수용하는 본문 경계. 빈 띠의 선언 몫과 구분한다.
    pub(crate) content_bottom: f64,
    /// 내용 뒤의 빈 띠가 예약하고 이어받는 종료선.
    pub(crate) blank_bottom: f64,
}

pub(crate) fn rowbreak_fragment_bounds(
    table: &Table,
    body_bottom: f64,
    dpi: f64,
    body_filling_source_frame: bool,
) -> RowbreakFragmentBounds {
    let outer_bottom = body_bottom - hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let blank_bottom = if body_filling_source_frame {
        outer_bottom
    } else {
        single_cell_page_fragment_bottom(table, body_bottom, dpi)
    };
    RowbreakFragmentBounds {
        // 단독 셀은 쪽 조각 자체가 이 프레임을 소유한다. 일반 격자의 실제
        // 글줄에는 빈 종료 띠를 위한 추가 inset을 수용 예산으로 중복 적용하지 않는다.
        content_bottom: if table.row_count == 1 && table.col_count == 1 {
            blank_bottom
        } else {
            outer_bottom
        },
        blank_bottom,
    }
}

pub(crate) fn rowbreak_fragment_frame_bottom(
    table: &Table,
    body_bottom: f64,
    dpi: f64,
    body_filling_source_frame: bool,
) -> f64 {
    rowbreak_fragment_bounds(table, body_bottom, dpi, body_filling_source_frame).content_bottom
}
