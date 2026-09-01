//! [#6557] 병합 셀이 낀 셀 선택 열 경계 드래그의 local_resize 마킹 계약.
//!
//! studio 마우스 드래그는 폭 보존 update(경계 왼쪽 셀 +d, 오른쪽 이웃 -d)를
//! `resizeTableCells` 로 보낸다. 적용 결과가 전 행 균일하면 행 단위 resize 로
//! 마킹하지 않아야 한다 — 세로 병합 셀의 delta 는 홈 행에만 집계되므로, 종전
//! count/합 휴리스틱은 병합 셀이 걸치지 않은 행만 일부 마킹했고 base grid
//! 추출(local 행 제외, col_span==1 max)에서 병합 열이 폭 소스를 잃어 기본값
//! 1800 으로 무너졌다 (studio 실측: 병합 셀 279.7px → 24px, 잔여 폭은 마지막
//! 열으로 쏠림). 결과가 실제로 갈라진 행만 종전대로 마킹한다.

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn table_of(doc: &HwpDocument, para_idx: usize) -> &rhwp::model::table::Table {
    doc.document().sections[0].paragraphs[para_idx]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표 컨트롤")
}

fn cell_index(doc: &HwpDocument, para_idx: usize, row: u16, col: u16) -> usize {
    table_of(doc, para_idx)
        .cells
        .iter()
        .enumerate()
        .find(|(_, c)| c.row == row && c.col == col)
        .map(|(idx, _)| idx)
        .expect("셀 인덱스")
}

fn table_para_idx(created: &str) -> usize {
    let parsed: serde_json::Value = serde_json::from_str(created).expect("createTable JSON");
    parsed["paraIdx"].as_u64().expect("paraIdx") as usize
}

#[test]
fn uniform_result_with_vertical_merge_keeps_base_grid() {
    let mut doc = HwpDocument::create_empty();
    let created = doc.create_table_native(0, 0, 0, 3, 2).expect("3x2 표");
    let para_idx = table_para_idx(&created);
    doc.merge_table_cells_native(0, para_idx, 0, 0, 0, 1, 0)
        .expect("세로 병합 (rows0-1, col0)");

    let merged_idx = cell_index(&doc, para_idx, 0, 0);
    let r0c1 = cell_index(&doc, para_idx, 0, 1);
    let r1c1 = cell_index(&doc, para_idx, 1, 1);
    let r2c0 = cell_index(&doc, para_idx, 2, 0);
    let r2c1 = cell_index(&doc, para_idx, 2, 1);
    let (base_w0, base_w1) = {
        let table = table_of(&doc, para_idx);
        (table.cells[merged_idx].width, table.cells[r0c1].width)
    };

    let delta = 3570i32;
    let neg = -delta;
    let payload = format!(
        r#"[{{"cellIdx":{merged_idx},"widthDelta":{delta}}},{{"cellIdx":{r0c1},"widthDelta":{neg}}},{{"cellIdx":{r1c1},"widthDelta":{neg}}},{{"cellIdx":{r2c0},"widthDelta":{delta}}},{{"cellIdx":{r2c1},"widthDelta":{neg}}}]"#
    );
    doc.resize_table_cells(0, para_idx as u32, 0, &payload)
        .expect("경계 드래그 적용");

    let table = table_of(&doc, para_idx);
    assert!(
        table.local_resize_rows.is_empty(),
        "전 행 균일 결과는 행 단위 resize 마킹 대상이 아니다: {:?}",
        table.local_resize_rows
    );
    let widths = table.get_column_widths();
    assert_eq!(
        widths[0] as i64,
        base_w0 as i64 + delta as i64,
        "col0 폭 = 원래 + delta"
    );
    assert_eq!(
        widths[1] as i64,
        base_w1 as i64 - delta as i64,
        "col1 폭 = 원래 - delta"
    );
    for cell in &table.cells {
        if cell.col_span == 1 {
            assert_eq!(
                cell.width, widths[cell.col as usize],
                "모든 행이 base grid 와 같은 폭을 가져야 한다 (row{} col{})",
                cell.row, cell.col
            );
        }
    }
}

#[test]
fn divergent_row_still_marks_local_resize() {
    // 대조군: 결과가 실제로 갈라지면(선택 행만 이동) 종전과 같이 그 행을
    // local_resize 로 마킹한다. 병합 없는 3x2 표에서 row1 만 +d/-d 를 받으면
    // row1 의 폭 벡터가 base grid 와 달라진다.
    let mut doc = HwpDocument::create_empty();
    let created = doc.create_table_native(0, 0, 0, 3, 2).expect("3x2 표");
    let para_idx = table_para_idx(&created);

    let r1c0 = cell_index(&doc, para_idx, 1, 0);
    let r1c1 = cell_index(&doc, para_idx, 1, 1);

    let payload = format!(
        r#"[{{"cellIdx":{r1c0},"widthDelta":900}},{{"cellIdx":{r1c1},"widthDelta":-900}}]"#
    );
    doc.resize_table_cells(0, para_idx as u32, 0, &payload)
        .expect("행 한정 드래그 적용");

    let table = table_of(&doc, para_idx);
    assert_eq!(
        table.local_resize_rows,
        vec![1],
        "갈라진 행은 종전대로 행 단위 resize 로 마킹된다"
    );
}
