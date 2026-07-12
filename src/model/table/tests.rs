use super::*;

/// 테스트용 N×M 표 생성 헬퍼
fn make_table(rows: u16, cols: u16) -> Table {
    let cell_width: HwpUnit = 3600; // 약 12.7mm
    let cell_height: HwpUnit = 1000;
    let mut cells = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            cells.push(Cell::new_empty(c, r, cell_width, cell_height, 1));
        }
    }
    let mut table = Table {
        row_count: rows,
        col_count: cols,
        row_sizes: vec![cols as i16; rows as usize],
        border_fill_id: 1,
        cells,
        ..Default::default()
    };
    table.rebuild_grid();
    table
}

fn set_cell_text(table: &mut Table, row: u16, col: u16, text: &str) {
    let idx = table.cell_index_at(row, col).expect("cell index");
    let mut para = Paragraph::new_empty();
    para.insert_text_at(0, text);
    table.cells[idx].paragraphs = vec![para];
}

fn cell_text(table: &Table, row: u16, col: u16) -> String {
    table
        .cell_at(row, col)
        .and_then(|cell| cell.paragraphs.first())
        .map(|para| para.text.clone())
        .unwrap_or_default()
}

#[test]
fn test_table_default() {
    let table = Table::default();
    assert_eq!(table.row_count, 0);
    assert_eq!(table.col_count, 0);
    assert!(table.cells.is_empty());
}

#[test]
fn test_cell_span() {
    let cell = Cell {
        col: 0,
        row: 0,
        col_span: 2,
        row_span: 3,
        ..Default::default()
    };
    assert_eq!(cell.col_span, 2);
    assert_eq!(cell.row_span, 3);
}

#[test]
fn test_cell_new_empty() {
    let cell = Cell::new_empty(2, 3, 3600, 1000, 5);
    assert_eq!(cell.col, 2);
    assert_eq!(cell.row, 3);
    assert_eq!(cell.col_span, 1);
    assert_eq!(cell.row_span, 1);
    assert_eq!(cell.width, 3600);
    assert_eq!(cell.height, 1000);
    assert_eq!(cell.border_fill_id, 5);
    assert_eq!(cell.paragraphs.len(), 1);
    assert_eq!(cell.paragraphs[0].char_count, 1); // 끝 마커(0x000D) 포함
}

#[test]
fn test_get_column_widths() {
    let table = make_table(2, 3);
    let widths = table.get_column_widths();
    assert_eq!(widths, vec![3600, 3600, 3600]);
}

#[test]
fn test_get_row_heights() {
    let table = make_table(2, 3);
    let heights = table.get_row_heights();
    assert_eq!(heights, vec![1000, 1000]);
}

// === insert_row 테스트 ===

#[test]
fn test_insert_row_below() {
    let mut table = make_table(2, 2);
    assert_eq!(table.cells.len(), 4);

    table.insert_row(0, true).unwrap();

    assert_eq!(table.row_count, 3);
    assert_eq!(table.row_sizes.len(), 3);
    assert_eq!(table.cells.len(), 6);

    // 행 0: 원래 첫 행
    assert_eq!(table.cells[0].row, 0);
    assert_eq!(table.cells[1].row, 0);
    // 행 1: 새 행
    assert_eq!(table.cells[2].row, 1);
    assert_eq!(table.cells[3].row, 1);
    // 행 2: 원래 둘째 행 (시프트)
    assert_eq!(table.cells[4].row, 2);
    assert_eq!(table.cells[5].row, 2);
}

#[test]
fn test_insert_row_above() {
    let mut table = make_table(2, 2);

    table.insert_row(0, false).unwrap();

    assert_eq!(table.row_count, 3);
    assert_eq!(table.cells.len(), 6);

    // 행 0: 새 행
    assert_eq!(table.cells[0].row, 0);
    assert_eq!(table.cells[1].row, 0);
    assert_eq!(table.cells[0].paragraphs.len(), 1); // 빈 문단
                                                    // 행 1: 원래 첫 행 (시프트)
    assert_eq!(table.cells[2].row, 1);
    assert_eq!(table.cells[3].row, 1);
}

#[test]
fn test_insert_row_with_merged_cell() {
    let mut table = make_table(3, 2);
    // (0,0) 셀을 row_span=2로 병합
    table.cells[0].row_span = 2;
    table.cells[0].height = 2000;
    // 병합된 (0,1) 셀 제거
    table.cells.retain(|c| !(c.col == 0 && c.row == 1));

    // 행 0 아래에 삽입 → 병합 셀이 삽입 지점을 걸침
    table.insert_row(0, true).unwrap();

    assert_eq!(table.row_count, 4);
    // (0,0) 셀의 row_span이 3으로 확장되어야 함
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.row_span, 3);
    // 새 행의 열 0은 병합 셀에 의해 커버 → 새 셀 없음
    // 새 행의 열 1에만 새 셀 생성
    let new_cells: Vec<_> = table.cells.iter().filter(|c| c.row == 1).collect();
    assert_eq!(new_cells.len(), 1);
    assert_eq!(new_cells[0].col, 1);
}

#[test]
fn test_insert_row_out_of_bounds() {
    let mut table = make_table(2, 2);
    assert!(table.insert_row(5, true).is_err());
}

// === insert_column 테스트 ===

#[test]
fn test_insert_column_right() {
    let mut table = make_table(2, 2);

    table.insert_column(0, true).unwrap();

    assert_eq!(table.col_count, 3);
    assert_eq!(table.cells.len(), 6);

    // 열 0: 원래, 열 1: 새로, 열 2: 원래 (시프트)
    let row0: Vec<u16> = table
        .cells
        .iter()
        .filter(|c| c.row == 0)
        .map(|c| c.col)
        .collect();
    assert_eq!(row0, vec![0, 1, 2]);
}

#[test]
fn test_insert_column_left() {
    let mut table = make_table(2, 2);

    table.insert_column(0, false).unwrap();

    assert_eq!(table.col_count, 3);
    assert_eq!(table.cells.len(), 6);

    // 열 0: 새로, 열 1: 원래 (시프트), 열 2: 원래 (시프트)
    let row0: Vec<u16> = table
        .cells
        .iter()
        .filter(|c| c.row == 0)
        .map(|c| c.col)
        .collect();
    assert_eq!(row0, vec![0, 1, 2]);
}

#[test]
fn test_insert_column_with_merged_cell() {
    let mut table = make_table(2, 3);
    // (0,0) 셀을 col_span=2로 병합
    table.cells[0].col_span = 2;
    table.cells[0].width = 7200;
    // 병합된 (1,0) 셀 제거
    table.cells.retain(|c| !(c.col == 1 && c.row == 0));

    // 열 0 오른쪽에 삽입 → 병합 셀이 삽입 지점을 걸침
    table.insert_column(0, true).unwrap();

    assert_eq!(table.col_count, 4);
    // (0,0) 셀의 col_span이 3으로 확장
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 3);
    // 행 0의 새 열에는 셀 없음 (병합에 의해 커버)
    // 행 1의 새 열에 새 셀 생성
    let row1_new: Vec<_> = table
        .cells
        .iter()
        .filter(|c| c.row == 1 && c.col == 1)
        .collect();
    assert_eq!(row1_new.len(), 1);
}

#[test]
fn test_insert_column_out_of_bounds() {
    let mut table = make_table(2, 2);
    assert!(table.insert_column(5, true).is_err());
}

// === set_column_widths 테스트 ===

#[test]
fn test_set_column_widths_basic() {
    let mut table = make_table(2, 3);
    table.set_column_widths(&[2000, 3000, 1000]).unwrap();
    assert_eq!(table.get_column_widths(), vec![2000, 3000, 1000]);
    // 모든 col_span==1 셀이 자기 열 폭으로 설정된다.
    for cell in &table.cells {
        let expected = [2000u32, 3000, 1000][cell.col as usize];
        assert_eq!(cell.width, expected, "col {} 셀 폭", cell.col);
    }
}

#[test]
fn test_set_column_widths_merged_cell() {
    let mut table = make_table(2, 3);
    // (0,0) 셀을 col_span=2로 병합하고 병합된 (1,0) 셀 제거
    table.cells[0].col_span = 2;
    table.cells[0].width = 7200;
    table.cells.retain(|c| !(c.col == 1 && c.row == 0));
    table.rebuild_grid();

    table.set_column_widths(&[2000, 3000, 1000]).unwrap();

    // 병합 셀(col0, span2)은 걸친 두 열 폭의 합(2000+3000)이 된다.
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0 && c.col_span == 2)
        .unwrap();
    assert_eq!(merged.width, 5000);
    // 열 폭은 col_span==1 셀(행 1) 기준으로 정확히 반영된다.
    assert_eq!(table.get_column_widths(), vec![2000, 3000, 1000]);
}

#[test]
fn test_set_column_widths_wrong_len() {
    let mut table = make_table(2, 3);
    assert!(table.set_column_widths(&[1000, 2000]).is_err());
}

// === merge_cells 테스트 ===

#[test]
fn test_merge_cells_2x2_full() {
    let mut table = make_table(2, 2);

    table.merge_cells(0, 0, 1, 1).unwrap();

    // 비주 셀 제거 → 주 셀 1개만 남음
    assert_eq!(table.cells.len(), 1);
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 2);
    assert_eq!(merged.row_span, 2);
    assert_eq!(merged.width, 7200); // 3600 * 2
    assert_eq!(merged.height, 2000); // 1000 * 2
                                     // row_sizes 갱신: 각 행에 셀 1개(행0만 주 셀), 행1은 0개
    assert_eq!(table.row_sizes, vec![1, 0]);
}

#[test]
fn test_merge_cells_partial_row() {
    let mut table = make_table(2, 3);

    // 첫 행의 열 0~1 병합
    table.merge_cells(0, 0, 0, 1).unwrap();

    assert_eq!(table.cells.len(), 5); // 비주 셀 1개 제거
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 2);
    assert_eq!(merged.row_span, 1);
    // row_sizes: 행0=2셀(병합1+col2), 행1=3셀
    assert_eq!(table.row_sizes, vec![2, 3]);
}

#[test]
fn test_merge_cells_partial_column() {
    let mut table = make_table(3, 2);

    // 열 0의 행 0~1 병합
    table.merge_cells(0, 0, 1, 0).unwrap();

    assert_eq!(table.cells.len(), 5); // 비주 셀 1개 제거
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 1);
    assert_eq!(merged.row_span, 2);
    // row_sizes: 행0=2셀(병합1+col1), 행1=1셀(col1만), 행2=2셀
    assert_eq!(table.row_sizes, vec![2, 1, 2]);
}

#[test]
fn test_merge_cells_invalid_range() {
    let mut table = make_table(2, 2);
    assert!(table.merge_cells(1, 0, 0, 0).is_err()); // start > end
    assert!(table.merge_cells(0, 0, 5, 5).is_err()); // 범위 초과
}

#[test]
fn test_merge_cells_overlapping_span() {
    let mut table = make_table(3, 3);
    // (0,0)을 col_span=2로 병합
    table.cells[0].col_span = 2;
    table.cells[0].width = 7200;
    table.cells.retain(|c| !(c.col == 1 && c.row == 0));

    // (0,0)~(0,2) 병합 시도 → 기존 병합이 범위 안에 있으므로 성공
    table.merge_cells(0, 0, 0, 2).unwrap();
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 3);
}

#[test]
fn test_insert_row_single_row() {
    let mut table = make_table(1, 3);

    table.insert_row(0, true).unwrap();

    assert_eq!(table.row_count, 2);
    assert_eq!(table.cells.len(), 6);
}

#[test]
fn test_insert_column_single_column() {
    let mut table = make_table(3, 1);

    table.insert_column(0, true).unwrap();

    assert_eq!(table.col_count, 2);
    assert_eq!(table.cells.len(), 6);
}

// === split_cell 테스트 ===

#[test]
fn test_split_cell_2x2_full() {
    let mut table = make_table(2, 2);
    table.merge_cells(0, 0, 1, 1).unwrap();
    assert_eq!(table.cells.len(), 1);

    // 나누기
    table.split_cell(0, 0).unwrap();

    assert_eq!(table.cells.len(), 4);
    // 모든 셀이 col_span=1, row_span=1
    for cell in &table.cells {
        assert_eq!(cell.col_span, 1);
        assert_eq!(cell.row_span, 1);
    }
    // row_sizes 복원
    assert_eq!(table.row_sizes, vec![2, 2]);
}

#[test]
fn test_split_cell_partial_row() {
    let mut table = make_table(2, 3);
    table.merge_cells(0, 0, 0, 1).unwrap();
    assert_eq!(table.cells.len(), 5);

    table.split_cell(0, 0).unwrap();

    assert_eq!(table.cells.len(), 6);
    let cell = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(cell.col_span, 1);
    assert_eq!(cell.row_span, 1);
    // row_sizes 복원: 각 행 3개 셀
    assert_eq!(table.row_sizes, vec![3, 3]);
}

#[test]
fn test_split_cell_partial_column() {
    let mut table = make_table(3, 2);
    table.merge_cells(0, 0, 1, 0).unwrap();
    assert_eq!(table.cells.len(), 5);

    table.split_cell(0, 0).unwrap();

    assert_eq!(table.cells.len(), 6);
    let cell = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(cell.col_span, 1);
    assert_eq!(cell.row_span, 1);
    // row_sizes 복원: 각 행 2개 셀
    assert_eq!(table.row_sizes, vec![2, 2, 2]);
}

#[test]
fn test_split_cell_not_merged() {
    let mut table = make_table(2, 2);
    // 병합되지 않은 셀 나누기 → 에러
    assert!(table.split_cell(0, 0).is_err());
}

#[test]
fn test_split_cell_width_distribution() {
    let mut table = make_table(2, 3);
    // 열 0~1 병합 (폭: 3600 + 3600 = 7200)
    table.merge_cells(0, 0, 0, 1).unwrap();

    table.split_cell(0, 0).unwrap();

    // 다른 행에 col_span=1 셀이 있으므로 실제 열폭 사용
    let cell0 = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    let cell1 = table
        .cells
        .iter()
        .find(|c| c.col == 1 && c.row == 0)
        .unwrap();
    assert_eq!(cell0.width, 3600);
    assert_eq!(cell1.width, 3600);
}

// === delete_row 테스트 ===

#[test]
fn test_delete_row_basic() {
    let mut table = make_table(3, 2);
    assert_eq!(table.cells.len(), 6);

    table.delete_row(1).unwrap();

    assert_eq!(table.row_count, 2);
    assert_eq!(table.cells.len(), 4);
    // 행 0: 유지, 행 1: 원래 행 2가 시프트
    assert_eq!(table.cells[0].row, 0);
    assert_eq!(table.cells[1].row, 0);
    assert_eq!(table.cells[2].row, 1);
    assert_eq!(table.cells[3].row, 1);
}

#[test]
fn test_delete_row_first() {
    let mut table = make_table(2, 2);

    table.delete_row(0).unwrap();

    assert_eq!(table.row_count, 1);
    assert_eq!(table.cells.len(), 2);
    assert_eq!(table.cells[0].row, 0);
    assert_eq!(table.cells[1].row, 0);
}

#[test]
fn test_delete_row_last() {
    let mut table = make_table(3, 2);

    table.delete_row(2).unwrap();

    assert_eq!(table.row_count, 2);
    assert_eq!(table.cells.len(), 4);
}

#[test]
fn test_delete_row_with_merged_cell() {
    let mut table = make_table(3, 2);
    // (0,0) 셀을 row_span=2로 병합
    table.cells[0].row_span = 2;
    table.cells[0].height = 2000;
    // 병합된 (0,1) 셀 제거
    table.cells.retain(|c| !(c.col == 0 && c.row == 1));
    assert_eq!(table.cells.len(), 5);

    // 행 1 삭제 → 병합 셀 row_span 축소
    table.delete_row(1).unwrap();

    assert_eq!(table.row_count, 2);
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.row_span, 1);
}

#[test]
fn test_delete_row_merged_cell_anchor() {
    let mut table = make_table(3, 2);
    // (0,0) 셀을 row_span=3으로 병합 (전체 열)
    table.cells[0].row_span = 3;
    table.cells[0].height = 3000;
    table
        .cells
        .retain(|c| !(c.col == 0 && (c.row == 1 || c.row == 2)));
    assert_eq!(table.cells.len(), 4);

    // 행 0(앵커 행) 삭제 → 병합 셀이 다음 행으로 이동, row_span 축소
    table.delete_row(0).unwrap();

    assert_eq!(table.row_count, 2);
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.row_span, 2);
}

#[test]
fn test_delete_row_single_row_error() {
    let mut table = make_table(1, 2);
    assert!(table.delete_row(0).is_err());
}

#[test]
fn test_delete_row_out_of_bounds() {
    let mut table = make_table(2, 2);
    assert!(table.delete_row(5).is_err());
}

// === delete_column 테스트 ===

#[test]
fn test_delete_column_basic() {
    let mut table = make_table(2, 3);
    assert_eq!(table.cells.len(), 6);

    table.delete_column(1).unwrap();

    assert_eq!(table.col_count, 2);
    assert_eq!(table.cells.len(), 4);
    // 열 0: 유지, 열 1: 원래 열 2가 시프트
    let row0: Vec<u16> = table
        .cells
        .iter()
        .filter(|c| c.row == 0)
        .map(|c| c.col)
        .collect();
    assert_eq!(row0, vec![0, 1]);
}

#[test]
fn test_delete_column_first() {
    let mut table = make_table(2, 2);

    table.delete_column(0).unwrap();

    assert_eq!(table.col_count, 1);
    assert_eq!(table.cells.len(), 2);
    assert_eq!(table.cells[0].col, 0);
    assert_eq!(table.cells[1].col, 0);
}

#[test]
fn test_delete_column_last() {
    let mut table = make_table(2, 3);

    table.delete_column(2).unwrap();

    assert_eq!(table.col_count, 2);
    assert_eq!(table.cells.len(), 4);
}

#[test]
fn test_delete_column_with_merged_cell() {
    let mut table = make_table(2, 3);
    // (0,0) 셀을 col_span=2로 병합
    table.cells[0].col_span = 2;
    table.cells[0].width = 7200;
    // 병합된 (1,0) 셀 제거
    table.cells.retain(|c| !(c.col == 1 && c.row == 0));
    assert_eq!(table.cells.len(), 5);

    // 열 1 삭제 → 병합 셀 col_span 축소
    table.delete_column(1).unwrap();

    assert_eq!(table.col_count, 2);
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 1);
}

#[test]
fn test_delete_column_merged_cell_anchor() {
    let mut table = make_table(2, 3);
    // (0,0) 셀을 col_span=3으로 병합 (전체 행)
    table.cells[0].col_span = 3;
    table.cells[0].width = 10800;
    table
        .cells
        .retain(|c| !(c.row == 0 && (c.col == 1 || c.col == 2)));
    assert_eq!(table.cells.len(), 4);

    // 열 0(앵커 열) 삭제 → 병합 셀 col_span 축소
    table.delete_column(0).unwrap();

    assert_eq!(table.col_count, 2);
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 2);
}

#[test]
fn test_delete_column_single_column_error() {
    let mut table = make_table(2, 1);
    assert!(table.delete_column(0).is_err());
}

#[test]
fn test_delete_column_out_of_bounds() {
    let mut table = make_table(2, 2);
    assert!(table.delete_column(5).is_err());
}

// === cell_grid / cell_at 테스트 ===

#[test]
fn test_rebuild_grid_basic() {
    let table = make_table(2, 3);
    // 2×3 표: 6개 셀, 각각 (row, col) 순서대로
    assert_eq!(table.cell_grid.len(), 6);
    for r in 0..2u16 {
        for c in 0..3u16 {
            let gi = (r as usize) * 3 + (c as usize);
            let idx = table.cell_grid[gi].expect("grid entry should exist");
            assert_eq!(table.cells[idx].row, r);
            assert_eq!(table.cells[idx].col, c);
        }
    }
}

#[test]
fn test_rebuild_grid_merged() {
    let mut table = make_table(3, 3);
    // (0,0) 셀을 2×2 병합
    table.merge_cells(0, 0, 1, 1).unwrap();
    // 병합 영역 전체가 같은 셀 인덱스를 가리켜야 함
    let anchor_idx = table.cell_grid[0].unwrap(); // (0,0)
    assert_eq!(table.cell_grid[1].unwrap(), anchor_idx); // (0,1)
    assert_eq!(table.cell_grid[3].unwrap(), anchor_idx); // (1,0)
    assert_eq!(table.cell_grid[4].unwrap(), anchor_idx); // (1,1)
                                                         // 앵커 셀 확인
    let anchor = &table.cells[anchor_idx];
    assert_eq!(anchor.row, 0);
    assert_eq!(anchor.col, 0);
    assert_eq!(anchor.col_span, 2);
    assert_eq!(anchor.row_span, 2);
}

#[test]
fn test_cell_at_basic() {
    let table = make_table(2, 3);
    for r in 0..2u16 {
        for c in 0..3u16 {
            let cell = table.cell_at(r, c).expect("cell should exist");
            assert_eq!(cell.row, r);
            assert_eq!(cell.col, c);
        }
    }
}

#[test]
fn test_cell_at_out_of_bounds() {
    let table = make_table(2, 3);
    assert!(table.cell_at(5, 0).is_none());
    assert!(table.cell_at(0, 10).is_none());
}

#[test]
fn test_cell_at_merged_span() {
    let mut table = make_table(3, 3);
    table.merge_cells(0, 0, 1, 1).unwrap();
    // 비앵커 좌표에서도 앵커 셀 반환
    let anchor = table.cell_at(0, 0).unwrap();
    let from_span = table.cell_at(1, 1).unwrap();
    assert!(std::ptr::eq(anchor, from_span));
}

#[test]
fn test_cell_index_at_basic() {
    let table = make_table(2, 3);
    for r in 0..2u16 {
        for c in 0..3u16 {
            let idx = table.cell_index_at(r, c).expect("should find index");
            assert_eq!(table.cells[idx].row, r);
            assert_eq!(table.cells[idx].col, c);
        }
    }
}

#[test]
fn test_edit_ops_rebuild_grid() {
    // insert_row 후 grid 정합성 확인
    let mut table = make_table(2, 2);
    table.insert_row(0, true).unwrap();
    assert_eq!(table.cell_grid.len(), 3 * 2); // 3행 × 2열
    for r in 0..3u16 {
        for c in 0..2u16 {
            let cell = table
                .cell_at(r, c)
                .expect("cell should exist after insert_row");
            assert_eq!(cell.row, r);
            assert_eq!(cell.col, c);
        }
    }

    // delete_column 후 grid 정합성 확인
    table.delete_column(1).unwrap();
    assert_eq!(table.cell_grid.len(), 3); // 3행 × 1열
    for r in 0..3u16 {
        let cell = table
            .cell_at(r, 0)
            .expect("cell should exist after delete_column");
        assert_eq!(cell.row, r);
        assert_eq!(cell.col, 0);
    }
}

// === split_cell_into 테스트 ===

#[test]
fn test_split_cell_into_1x2() {
    // 2×2 표, (0,0)을 1줄×2칸으로 분할
    let mut table = make_table(2, 2);
    table.split_cell_into(0, 0, 1, 2, true, false).unwrap();

    assert_eq!(table.col_count, 3); // 2+1
    assert_eq!(table.row_count, 2); // 변경 없음
                                    // 서브셀 2개
    let c00 = table.cell_at(0, 0).unwrap();
    let c01 = table.cell_at(0, 1).unwrap();
    assert_eq!(c00.col_span, 1);
    assert_eq!(c01.col_span, 1);
    assert_eq!(c00.width + c01.width, 3600); // 원래 폭 보존
                                             // (0,2): 원래 (0,1)이 이동한 셀
    let c02 = table.cell_at(0, 2).unwrap();
    assert_eq!(c02.col, 2);
    // (1,0): 같은 열 → col_span=2로 확장
    let c10 = table.cell_at(1, 0).unwrap();
    assert_eq!(c10.col_span, 2);
}

#[test]
fn test_split_cell_into_2x1() {
    // 2×2 표, (0,0)을 2줄×1칸으로 분할
    let mut table = make_table(2, 2);
    table.split_cell_into(0, 0, 2, 1, true, false).unwrap();

    assert_eq!(table.row_count, 3); // 2+1
    assert_eq!(table.col_count, 2); // 변경 없음
    let c00 = table.cell_at(0, 0).unwrap();
    let c10 = table.cell_at(1, 0).unwrap();
    assert_eq!(c00.row_span, 1);
    assert_eq!(c10.row_span, 1);
    assert_eq!(c00.height + c10.height, 1000); // 원래 높이 보존
                                               // (0,1): 같은 행 → row_span=2로 확장
    let c01 = table.cell_at(0, 1).unwrap();
    assert_eq!(c01.row_span, 2);
}

#[test]
fn test_split_cell_into_2x2() {
    // 3×3 표, (1,1)을 2줄×2칸으로 분할
    let mut table = make_table(3, 3);
    table.split_cell_into(1, 1, 2, 2, true, false).unwrap();

    assert_eq!(table.col_count, 4); // 3+1
    assert_eq!(table.row_count, 4); // 3+1
                                    // 4개 서브셀 확인
    for r in 1..=2u16 {
        for c in 1..=2u16 {
            let cell = table.cell_at(r, c).unwrap();
            assert_eq!(cell.col_span, 1);
            assert_eq!(cell.row_span, 1);
        }
    }
    // (0,1): 같은 열 → col_span=2
    let c01 = table.cell_at(0, 1).unwrap();
    assert_eq!(c01.col_span, 2);
    // (1,0): 같은 행 → row_span=2
    let c10 = table.cell_at(1, 0).unwrap();
    assert_eq!(c10.row_span, 2);
}

#[test]
fn test_split_cell_into_noop() {
    let mut table = make_table(2, 2);
    table.split_cell_into(0, 0, 1, 1, true, false).unwrap();
    assert_eq!(table.col_count, 2);
    assert_eq!(table.row_count, 2);
    assert_eq!(table.cells.len(), 4);
}

#[test]
fn test_split_cell_into_width_distribution() {
    // 1×1 표 (단일 셀, 폭=7200), 1줄×3칸으로 분할
    let mut table = make_table(1, 1);
    // 기본 폭 3600 → 7200으로 변경
    table.cells[0].width = 7200;
    table.split_cell_into(0, 0, 1, 3, true, false).unwrap();

    assert_eq!(table.col_count, 3);
    let w0 = table.cell_at(0, 0).unwrap().width;
    let w1 = table.cell_at(0, 1).unwrap().width;
    let w2 = table.cell_at(0, 2).unwrap().width;
    assert_eq!(w0 + w1 + w2, 7200);
    assert_eq!(w0, 2400);
    assert_eq!(w1, 2400);
    assert_eq!(w2, 2400);
}

#[test]
fn test_split_cell_into_merged_merge_first() {
    // 2×3 표, (0,0)-(0,1) 병합 후 merge_first로 1줄×3칸 분할
    let mut table = make_table(2, 3);
    table.merge_cells(0, 0, 0, 1).unwrap();

    // 병합된 셀: col_span=2
    let merged = table
        .cells
        .iter()
        .find(|c| c.col == 0 && c.row == 0)
        .unwrap();
    assert_eq!(merged.col_span, 2);

    table.split_cell_into(0, 0, 1, 3, true, true).unwrap();

    // 병합 해제 후 (0,0) span=1x1 → 1×3 분할: extra_cols=2
    assert_eq!(table.col_count, 5); // 3 + 2
    let c00 = table.cell_at(0, 0).unwrap();
    assert_eq!(c00.col_span, 1);
}

#[test]
fn test_split_cells_in_range_2x2_into_1x2() {
    // 3×3 표, (0,0)~(1,1) 범위의 4개 셀을 각각 1줄×2칸으로 분할
    let mut table = make_table(3, 3);
    table.split_cells_in_range(0, 0, 1, 1, 1, 2, true).unwrap();

    // 열 0, 1 각각 2분할 → extra_cols = 2
    assert_eq!(table.col_count, 5); // 3 + 2
    assert_eq!(table.row_count, 3);

    // (0,0)과 (0,1): 분할된 서브셀 (span=1)
    let c00 = table.cell_at(0, 0).unwrap();
    let c01 = table.cell_at(0, 1).unwrap();
    assert_eq!(c00.col_span, 1);
    assert_eq!(c01.col_span, 1);

    // (2,0): 선택 범위 밖 → col_span=2 (열 0의 서브열 흡수)
    let c20 = table.cell_at(2, 0).unwrap();
    assert_eq!(c20.col_span, 2);

    // (0,4): 원래 (0,2)가 시프트된 셀
    let c04 = table.cell_at(0, 4).unwrap();
    assert_eq!(c04.col_span, 1);
}

#[test]
fn test_split_cells_in_range_single_cell() {
    // 범위 = 단일 셀: split_cell_into와 동일 동작
    let mut table = make_table(2, 2);
    table.split_cells_in_range(0, 0, 0, 0, 1, 3, true).unwrap();
    assert_eq!(table.col_count, 4); // 2 + 2
}

// === transpose copy/paste 테스트 ===

#[test]
fn test_transpose_copy_paste_4x2_to_2x4() {
    let mut table = make_table(4, 6);
    for r in 0..4u16 {
        for c in 0..2u16 {
            set_cell_text(&mut table, r, c, &format!("s{r}{c}"));
        }
    }
    set_cell_text(&mut table, 0, 2, "target");

    let data = table.copy_transpose_range(0, 0, 3, 1).unwrap();
    let changed = table.paste_transposed_cells(0, 2, &data).unwrap();

    assert_eq!(data.source_rows, 4);
    assert_eq!(data.source_cols, 2);
    assert_eq!(changed.len(), 8);
    assert_eq!(cell_text(&table, 0, 2), "s00");
    assert_eq!(cell_text(&table, 0, 3), "s10");
    assert_eq!(cell_text(&table, 0, 4), "s20");
    assert_eq!(cell_text(&table, 0, 5), "s30");
    assert_eq!(cell_text(&table, 1, 2), "s01");
    assert_eq!(cell_text(&table, 1, 3), "s11");
    assert_eq!(cell_text(&table, 1, 4), "s21");
    assert_eq!(cell_text(&table, 1, 5), "s31");

    // 원본 범위는 정적 복사이므로 유지된다.
    assert_eq!(cell_text(&table, 3, 1), "s31");
}

#[test]
fn test_transpose_full_table_in_place_4x2_to_2x4() {
    let mut table = make_table(4, 2);
    for r in 0..4u16 {
        for c in 0..2u16 {
            set_cell_text(&mut table, r, c, &format!("s{r}{c}"));
        }
    }

    let changed = table.transpose_unmerged_table_in_place().unwrap();

    assert_eq!(table.row_count, 2);
    assert_eq!(table.col_count, 4);
    assert_eq!(changed.len(), 8);
    assert_eq!(cell_text(&table, 0, 0), "s00");
    assert_eq!(cell_text(&table, 0, 1), "s10");
    assert_eq!(cell_text(&table, 0, 2), "s20");
    assert_eq!(cell_text(&table, 0, 3), "s30");
    assert_eq!(cell_text(&table, 1, 0), "s01");
    assert_eq!(cell_text(&table, 1, 1), "s11");
    assert_eq!(cell_text(&table, 1, 2), "s21");
    assert_eq!(cell_text(&table, 1, 3), "s31");
}

#[test]
fn test_transpose_paste_out_of_bounds_fails() {
    let mut table = make_table(2, 2);
    set_cell_text(&mut table, 0, 0, "a");
    set_cell_text(&mut table, 0, 1, "b");
    set_cell_text(&mut table, 1, 0, "c");
    set_cell_text(&mut table, 1, 1, "d");

    let data = table.copy_transpose_range(0, 0, 1, 1).unwrap();

    assert!(table.paste_transposed_cells(1, 1, &data).is_err());
    assert_eq!(cell_text(&table, 1, 1), "d");
}

#[test]
fn test_transpose_rejects_merged_cells() {
    let mut table = make_table(3, 3);
    table.merge_cells(0, 0, 0, 1).unwrap();
    assert!(table.copy_transpose_range(0, 0, 1, 1).is_err());

    let mut target_table = make_table(3, 3);
    set_cell_text(&mut target_table, 0, 0, "a");
    set_cell_text(&mut target_table, 1, 0, "b");
    let data = target_table.copy_transpose_range(0, 0, 1, 0).unwrap();
    target_table.merge_cells(0, 1, 0, 2).unwrap();
    assert!(target_table.paste_transposed_cells(0, 1, &data).is_err());
}

// [Task #1716] leading_header_rows: 상단 연속 제목행 블록만 반환하는지 검증
#[test]
fn test_leading_header_rows_scattered_body_headers() {
    // 상단 1행 header + 본문(행 2·4)에 흩어진 header → [0] 만
    let mut t = make_table(6, 3);
    for c in 0..3 {
        let i = t.cell_index_at(0, c).unwrap();
        t.cells[i].is_header = true;
    }
    for &r in &[2u16, 4] {
        let i = t.cell_index_at(r, 0).unwrap();
        t.cells[i].is_header = true;
    }
    assert_eq!(t.leading_header_rows(), vec![0]);
}

#[test]
fn test_leading_header_rows_contiguous_multi() {
    // 상단 연속 2행 header → [0,1] (#1022 다중 머리행 보존)
    let mut t = make_table(5, 3);
    for r in 0..2 {
        for c in 0..3 {
            let i = t.cell_index_at(r, c).unwrap();
            t.cells[i].is_header = true;
        }
    }
    assert_eq!(t.leading_header_rows(), vec![0, 1]);
}

#[test]
fn test_leading_header_rows_rowspan_header() {
    // rowspan=2 header 셀이 행 0..2 를 덮음 → [0,1]
    let mut t = make_table(4, 2);
    let i = t.cell_index_at(0, 0).unwrap();
    t.cells[i].is_header = true;
    t.cells[i].row_span = 2;
    assert_eq!(t.leading_header_rows(), vec![0, 1]);
}

#[test]
fn test_leading_header_rows_none_and_all() {
    let t = make_table(3, 2);
    assert_eq!(t.leading_header_rows(), Vec::<usize>::new());
    let mut all = make_table(3, 2);
    for r in 0..3 {
        for c in 0..2 {
            let i = all.cell_index_at(r, c).unwrap();
            all.cells[i].is_header = true;
        }
    }
    assert_eq!(all.leading_header_rows(), vec![0, 1, 2]);
}

/// 삽입 지점의 열(행)에 비병합 셀이 하나도 없으면 insert_row / insert_column 의
/// 템플릿 탐색이 전부 실패했고, Cell::new_empty() 로 후퇴해 para_shape_id/style_id=0,
/// char_shapes 가 빈 셀이 만들어졌다 (저장 시 charPrIDRef="0").
/// 이제 표의 아무 셀이나 템플릿으로 쓴다.
fn shape_cell(mut cell: Cell) -> Cell {
    cell.paragraphs[0].para_shape_id = 12;
    cell.paragraphs[0].style_id = 3;
    cell.paragraphs[0].char_shapes = vec![crate::model::paragraph::CharShapeRef {
        start_pos: 0,
        char_shape_id: 7,
    }];
    cell
}

/// 모든 셀이 가로 병합(col_span=2) — 어떤 열에도 비병합 셀이 없다.
/// insert_row 의 템플릿 탐색(`col_span == 1`)이 전부 실패한다.
fn col_merged_table(rows: u16) -> Table {
    let cells = (0..rows)
        .map(|r| {
            let mut cell = Cell::new_empty(0, r, 7200, 1000, 1);
            cell.col_span = 2;
            shape_cell(cell)
        })
        .collect();
    Table {
        row_count: rows,
        col_count: 2,
        row_sizes: vec![1; rows as usize],
        border_fill_id: 1,
        cells,
        ..Default::default()
    }
}

/// 모든 셀이 세로 병합(row_span=2) — 어떤 행에도 비병합 셀이 없다.
/// insert_column 의 템플릿 탐색(`row_span == 1`)이 전부 실패한다.
fn row_merged_table(cols: u16) -> Table {
    let cells = (0..cols)
        .map(|c| {
            let mut cell = Cell::new_empty(c, 0, 3600, 2000, 1);
            cell.row_span = 2;
            shape_cell(cell)
        })
        .collect();
    Table {
        row_count: 2,
        col_count: cols,
        row_sizes: vec![cols as i16, 0],
        border_fill_id: 1,
        cells,
        ..Default::default()
    }
}

fn assert_inherited(cell: &Cell, where_: &str) {
    let p = &cell.paragraphs[0];
    assert_eq!(p.para_shape_id, 12, "{}: para_shape_id 상속", where_);
    assert_eq!(p.style_id, 3, "{}: style_id 상속", where_);
    assert_eq!(
        p.char_shapes.first().map(|cs| cs.char_shape_id),
        Some(7),
        "{}: char_shapes 상속 (빈 채로 두면 charPrIDRef=0)",
        where_
    );
}

#[test]
fn insert_row_inherits_shape_when_column_has_only_merged_cells() {
    let mut table = col_merged_table(2);
    table.insert_row(1, false).unwrap();

    let new_cells: Vec<&Cell> = table
        .cells
        .iter()
        .filter(|c| c.row == 1 && c.col_span == 1)
        .collect();
    assert_eq!(new_cells.len(), 2, "새 행에 셀 2개");
    for cell in new_cells {
        assert_inherited(cell, "insert_row");
    }
}

#[test]
fn insert_column_inherits_shape_when_row_has_only_merged_cells() {
    let mut table = row_merged_table(2);
    table.insert_column(1, true).unwrap();

    let new_cells: Vec<&Cell> = table.cells.iter().filter(|c| c.row_span == 1).collect();
    assert!(!new_cells.is_empty(), "새 열 셀이 생성되어야 한다");
    for cell in new_cells {
        assert_inherited(cell, "insert_column");
    }
}
