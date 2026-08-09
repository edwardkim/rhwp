//! Issue #4323: 셀 합치기 후 병합 셀 텍스트가 합치기 전 폭으로 줄바꿈된다 (reflow 누락).
//!
//! `merge_table_cells_native`(`table_ops.rs`)는 `Table::merge_cells`로 주 셀의 폭을
//! 넓힌 뒤 `recompose_section` + `paginate_if_needed`만 호출하고 `reflow_cell_paragraph`를
//! 부르지 않았다. 폭을 바꾸는 형제 명령인 `set_table_column_widths_native`/
//! `resize_table_cells_native`는 둘 다 부른다 — 셀 합치기만 규약에서 빠져 있었다.
//!
//! 결과: 셀 상자만 넓어지고 저장된 `line_segs`는 합치기 전 좁은 폭 기준 그대로 남아
//! 각 줄 오른쪽이 비고 행 높이도 줄지 않는다.
//!
//! 재현: 1×3 표, 좁은 폭의 첫 셀에 줄바꿈이 필요한 긴 텍스트를 넣고 (0,0)~(0,2)를
//! 합친 뒤 문단의 line_segs 줄 수가 합치기 전보다 줄어드는지(재래핑됐는지) 확인한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;

/// 좁은 폭(HWPUNIT) 3열짜리 1×3 표를 가진 빈 문서. 반환값은 표가 놓인 본문 문단 인덱스.
///
/// `create_table_ex_native`의 `col_widths_hu`는 `treat_as_char=false` 경로
/// (`create_table_native`)에서는 적용되지 않으므로(편집 영역 폭을 열 수로 균등
/// 분배), 생성 후 `set_table_column_widths_native`로 명시적으로 좁혀 폭을 고정한다.
fn doc_with_narrow_table(col_width: u32) -> (DocumentCore, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");
    core.create_table_ex_native(0, 0, 0, 1, 3, false, None, None)
        .expect("1x3 table");

    let para_idx = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .expect("표가 놓인 문단");
    let ctrl_idx = table_control_idx(&core, para_idx);
    core.set_table_column_widths_native(0, para_idx, ctrl_idx, vec![col_width; 3])
        .expect("좁은 열 폭 지정");
    (core, para_idx)
}

fn table_control_idx(core: &DocumentCore, para_idx: usize) -> usize {
    core.document().sections[0].paragraphs[para_idx]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .expect("표 컨트롤")
}

fn target_table(core: &DocumentCore, para_idx: usize, ctrl_idx: usize) -> &Table {
    match &core.document().sections[0].paragraphs[para_idx].controls[ctrl_idx] {
        Control::Table(t) => t,
        other => panic!("controls[{ctrl_idx}] 이 표가 아님: {other:?}"),
    }
}

/// (row, col) 위치의 셀 인덱스.
fn cell_idx_at(table: &Table, row: u16, col: u16) -> usize {
    table
        .cells
        .iter()
        .position(|c| c.row == row && c.col == col)
        .unwrap_or_else(|| panic!("({row},{col}) 셀을 찾을 수 없음"))
}

/// 셀 합치기 후 주 셀 문단이 새 폭으로 재래핑되어 줄 수가 줄어든다.
#[test]
fn merge_table_cells_reflows_widened_primary_cell() {
    // 한 글자(전각) 당 대략 1000 HWPUNIT 안팎으로 잡고, 좁은 열 폭에서
    // 여러 줄로 감기게 충분히 긴 한글 문자열을 준비한다.
    let narrow_col_width: u32 = 4200;
    let long_text: String = "가나다라마바사아자차카타파하".repeat(3); // 42자

    let (mut core, para_idx) = doc_with_narrow_table(narrow_col_width);
    let ctrl_idx = table_control_idx(&core, para_idx);

    let cell0 = cell_idx_at(target_table(&core, para_idx, ctrl_idx), 0, 0);
    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, cell0, 0, 0, &long_text)
        .expect("긴 텍스트 삽입");

    let before_table = target_table(&core, para_idx, ctrl_idx);
    let before_cell = &before_table.cells[cell0];
    let before_width = before_cell.width;
    let before_lines = before_cell.paragraphs[0].line_segs.len();
    assert!(
        before_lines > 1,
        "재현 전제 실패: 좁은 폭({before_width})에서 줄바꿈이 일어나지 않음 \
         (line_segs={before_lines}). 텍스트를 더 늘리거나 폭을 더 좁혀야 함."
    );

    // (0,0)~(0,2) 병합 — 주 셀 폭이 3배로 넓어진다.
    core.merge_table_cells_native(0, para_idx, ctrl_idx, 0, 0, 0, 2)
        .expect("셀 병합 (0,0)~(0,2)");

    let after_table = target_table(&core, para_idx, ctrl_idx);
    let after_cell_idx = cell_idx_at(after_table, 0, 0);
    let after_cell = &after_table.cells[after_cell_idx];
    assert_eq!(
        after_cell.width,
        before_width * 3,
        "병합 후 주 셀 폭이 3배가 되어야 함"
    );

    let after_lines = after_cell.paragraphs[0].line_segs.len();
    assert!(
        after_lines < before_lines,
        "#4323 회귀: 병합 후 주 셀 폭이 {}(->{})로 넓어졌는데도 line_segs 줄 수가 \
         {before_lines}->{after_lines}로 줄지 않음 — reflow_cell_paragraph 가 호출되지 \
         않아 합치기 전 좁은 폭 기준 줄바꿈이 그대로 남아 있다.",
        before_width,
        after_cell.width,
    );

    // 재래핑된 각 줄의 segment_width 는 새 셀 내부 폭(패딩 차감) 이하여야 하고,
    // 옛 좁은 폭(before_width) 근처에 멈춰 있으면 안 된다 — stale 값 회귀 가드.
    for seg in &after_cell.paragraphs[0].line_segs {
        assert!(
            (seg.segment_width as i64) <= after_cell.width as i64,
            "재래핑된 세그먼트 폭({})이 새 셀 폭({})을 넘음",
            seg.segment_width,
            after_cell.width
        );
    }
    let widened_at_least_one = after_cell.paragraphs[0]
        .line_segs
        .iter()
        .any(|seg| (seg.segment_width as i64) > (before_width as i64));
    assert!(
        widened_at_least_one,
        "#4323 회귀: 재래핑 후에도 모든 줄 세그먼트 폭이 옛 좁은 폭({before_width}) 이하 \
         — reflow 가 실제로 새 폭을 반영하지 않음"
    );
}

/// 형제 셀(합치기 범위 밖)의 line_segs 는 병합에 영향받지 않는다.
#[test]
fn merge_table_cells_does_not_touch_untouched_cell() {
    let narrow_col_width: u32 = 4200;
    let long_text: String = "가나다라마바사아자차카타파하".repeat(3);

    let (mut core, para_idx) = doc_with_narrow_table(narrow_col_width);
    let ctrl_idx = table_control_idx(&core, para_idx);

    // 4행 3열이 아니라 1행 3열 표라서 병합은 (0,0)~(0,1)만 하고 (row0,col2)는 그대로 둔다.
    let table = target_table(&core, para_idx, ctrl_idx);
    let cell0 = cell_idx_at(table, 0, 0);
    let cell2 = cell_idx_at(table, 0, 2);
    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, cell0, 0, 0, &long_text)
        .expect("cell0 텍스트 삽입");
    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, cell2, 0, 0, &long_text)
        .expect("cell2 텍스트 삽입");

    let before_table = target_table(&core, para_idx, ctrl_idx);
    let before_cell2_lines = before_table.cells[cell_idx_at(before_table, 0, 2)].paragraphs[0]
        .line_segs
        .len();

    core.merge_table_cells_native(0, para_idx, ctrl_idx, 0, 0, 0, 1)
        .expect("셀 병합 (0,0)~(0,1)");

    let after_table = target_table(&core, para_idx, ctrl_idx);
    let after_cell2 = &after_table.cells[cell_idx_at(after_table, 0, 2)];
    assert_eq!(
        after_cell2.width, narrow_col_width,
        "병합 범위 밖 셀의 폭은 그대로여야 함"
    );
    assert_eq!(
        after_cell2.paragraphs[0].line_segs.len(),
        before_cell2_lines,
        "병합 범위 밖 셀의 줄 수는 그대로여야 함"
    );
}
