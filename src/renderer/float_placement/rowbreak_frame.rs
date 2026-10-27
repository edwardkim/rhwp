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
    has_original_page_frames: bool,
) -> RowbreakFragmentBounds {
    let outer_bottom = body_bottom - hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let blank_bottom = if body_filling_source_frame {
        outer_bottom
    } else {
        single_cell_page_fragment_bottom(table, body_bottom, dpi)
    };
    // 분할 저장된 한 열 본문의 복제 빈 peer는 독립 내용 프레임이 아니다.
    // 원본 폭·문단 복제 출처를 인증한 소유자는 같은 쪽 조각의 물리 하단을 쓴다.
    // 일반 두 열 격자에는 이 출처 판별이 적용되지 않는다.
    let reparsed_body_owner = table.col_count == 2
        && table.cells.iter().any(|cell| {
            cell.col == 1
                && cell.paragraphs.len() == 1
                && cell.paragraphs[0].text.is_empty()
                && cell.paragraphs[0].controls.is_empty()
                && crate::renderer::typeset::is_reparsed_single_column_cell_split_row(
                    table,
                    usize::from(cell.row),
                )
        });
    // 원본 쪽 프레임이 없다는 사실만으로 재구성된 저장 사다리가 있다고
    // 추정하지 않는다. NO_LS 본문은 새 글줄을 만들며 일반 격자 하단을 쓴다.
    // 내용 없는 앵커의 저장 줄은 본문 좌표계의 증거가 아니다.
    let has_saved_text_coordinates = table.cells.iter().any(|cell| {
        cell.paragraphs.iter().any(|paragraph| {
            !paragraph.text.trim().is_empty()
                && !crate::renderer::para_has_no_stored_line_segs(paragraph)
        })
    });
    RowbreakFragmentBounds {
        // 단독 셀과 출처가 인증된 분할 본문은 쪽 조각 자체가 이 프레임을 소유한다.
        // 일반 격자의 글줄에는 빈 종료 띠의 inset을 수용 예산으로 중복 적용하지 않는다.
        content_bottom: if (!has_original_page_frames && has_saved_text_coordinates)
            || (table.row_count == 1 && table.col_count == 1)
            || reparsed_body_owner
        {
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
    rowbreak_fragment_bounds(table, body_bottom, dpi, body_filling_source_frame, true)
        .content_bottom
}
