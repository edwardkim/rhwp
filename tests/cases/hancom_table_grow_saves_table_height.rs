//! 셀 내용이 넘친 표는 한컴처럼 칸 높이는 두고 표 높이(sz)만 키워 저장한다.
//! 기준 파일은 한컴에서 만든 3행 표(행 높이 10mm)와, 가운데 행에 긴 글을 넣어 저장한 결과다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

const BEFORE: &[u8] = include_bytes!("../fixtures/hancom_table_grow/grow3-before.hwpx");
const AFTER: &[u8] = include_bytes!("../fixtures/hancom_table_grow/grow3-after.hwpx");
const TEXT: &str = "가나다라마바사아자차카타파하가나다라마바사아자차카타파하";

fn table(core: &DocumentCore) -> (usize, u32, Vec<u32>) {
    let paragraph = &core.document().sections[0].paragraphs[0];
    let (index, table) = paragraph
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::Table(table) => Some((index, table)),
            _ => None,
        })
        .unwrap();
    (
        index,
        table.common.height,
        table.cells.iter().map(|cell| cell.height).collect(),
    )
}

/// 첫 행 "위"와 마지막 행 "아래"의 y 차이 = 첫 행 + 둘째 행 높이.
fn first_to_last_row(core: &DocumentCore) -> f64 {
    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    let y = |text: &str| {
        layout["runs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|run| run["text"] == text)
            .unwrap()["y"]
            .as_f64()
            .unwrap()
    };
    y("아래") - y("위")
}

#[test]
fn overflowing_cell_text_saves_the_grown_table_height_and_keeps_cell_heights() {
    let hancom = DocumentCore::from_bytes(AFTER).unwrap();
    let (_, hancom_height, hancom_cells) = table(&hancom);
    assert_eq!(hancom_height, 11748);
    assert_eq!(hancom_cells, vec![2834, 2834, 2834]);

    let mut core = DocumentCore::from_bytes(BEFORE).unwrap();
    let (control, ..) = table(&core);
    core.insert_text_in_cell_native(0, 0, control, 1, 0, 0, TEXT)
        .unwrap();
    for saved in [
        core.export_hwpx_native().unwrap(),
        core.export_hwp_with_adapter_snapshot().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&saved).unwrap();
        let (_, height, cells) = table(&reopened);
        assert!(
            height.abs_diff(hancom_height) <= 2,
            "저장 표 높이는 한컴처럼 행 높이 합이다: {height}"
        );
        assert_eq!(cells, hancom_cells, "칸 높이는 선언값을 지킨다");
        assert!((first_to_last_row(&reopened) - first_to_last_row(&hancom)).abs() < 0.2);
    }
}

#[test]
fn grown_table_reopens_with_only_the_overflowing_row_taller() {
    // 표 높이가 행 합보다 커도 넘친 행만 커지고 첫 행은 선언 높이 10mm를 지킨다.
    let before = DocumentCore::from_bytes(BEFORE).unwrap();
    let hancom = DocumentCore::from_bytes(AFTER).unwrap();
    let row = first_to_last_row(&before) / 2.0;
    let second_row = 6080.0 * 96.0 / 7200.0;
    assert!((first_to_last_row(&hancom) - (row + second_row)).abs() < 0.2);
}

#[test]
fn unchanged_table_keeps_its_saved_height() {
    let core = DocumentCore::from_bytes(BEFORE).unwrap();
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(table(&reopened).1, 8502);
}
