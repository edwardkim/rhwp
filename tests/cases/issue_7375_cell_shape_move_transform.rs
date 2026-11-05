//! 관계형: 번호 도형을 이동·복원할 때 원본 행렬과 표시 경계의 가역성을 검사한다.
//! 주석 갱신: 2026-11-05 17:49 KST.
//! 입력: issue7333/aaaaaa.hwp, 독립 한컴 2020 Print의 27·28쪽 번호 도형.
//! Native 집중 Visual Sweep의 27쪽은 100%다. fresh WASM 일치율은 이 검사 작성 시 미측정이며,
//! 실물 px 기대값을 고정하지 않고 같은 객체의 변경 전후 관계·원본 데이터 보존을 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::DocumentCore;
use serde_json::{json, Value};

const PATH: &str = r#"[{"controlIndex":0,"cellIndex":0,"cellParaIndex":0}]"#;

fn load() -> DocumentCore {
    DocumentCore::from_bytes(include_bytes!("../../samples/issue7333/aaaaaa.hwp")).unwrap()
}

fn matrix(core: &DocumentCore, para: usize) -> Vec<u8> {
    let Control::Table(table) = &core.document().sections[0].paragraphs[para].controls[0] else {
        panic!("원본 표가 없다");
    };
    let Control::Shape(shape) = &table.cells[0].paragraphs[0].controls[0] else {
        panic!("원본 번호 도형이 없다");
    };
    shape.shape_attr().raw_rendering.clone()
}

fn bounds(core: &mut DocumentCore, page: u32, para: usize) -> Value {
    let layout: Value =
        serde_json::from_str(&core.get_page_control_layout_native(page).unwrap()).unwrap();
    let control = layout["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["type"] == "shape"
                && c["paraIdx"] == para
                && c["controlIdx"] == 0
                && c["cellPath"].as_array().is_some_and(|p| p.len() == 1)
        })
        .expect("번호 도형의 실제 표시 경계");
    json!([control["x"], control["y"], control["w"], control["h"]])
}

fn move_and_restore(page: u32, para: usize) {
    let mut core = load();
    assert_eq!(core.page_count(), 50);
    let before_bounds = bounds(&mut core, page, para);
    let before_matrix = matrix(&core, para);
    assert!(!before_matrix.is_empty(), "원본 변환 행렬이 있는 대조군");
    let before: Value = serde_json::from_str(
        &core
            .get_cell_shape_properties_by_path_native(0, para, PATH, 0)
            .unwrap(),
    )
    .unwrap();
    let moved = json!({
        "horzOffset": before["horzOffset"].as_u64().unwrap().wrapping_add(1800) as u32,
        "vertOffset": before["vertOffset"].as_u64().unwrap().wrapping_add(1350) as u32,
    });
    core.set_cell_shape_properties_by_path_native(0, para, PATH, 0, &moved.to_string())
        .unwrap();
    assert_ne!(
        bounds(&mut core, page, para),
        before_bounds,
        "실제 이동이 표시 위치를 바꾼다"
    );
    assert_eq!(
        matrix(&core, para),
        before_matrix,
        "위치만 바꿀 때 로컬 변환 행렬을 버리지 않는다"
    );
    let restore = json!({"horzOffset": before["horzOffset"], "vertOffset": before["vertOffset"]});
    core.set_cell_shape_properties_by_path_native(0, para, PATH, 0, &restore.to_string())
        .unwrap();
    assert_eq!(
        bounds(&mut core, page, para),
        before_bounds,
        "Undo가 저장 좌표뿐 아니라 화면 위치도 복원한다"
    );
    assert_eq!(matrix(&core, para), before_matrix);
    assert_eq!(core.page_count(), 50);
}

#[test]
fn translated_number_restores_original_transform_and_display_bounds() {
    move_and_restore(26, 383);
}

#[test]
fn sibling_page_number_keeps_the_same_reversible_move_contract() {
    move_and_restore(27, 392);
}

#[test]
fn deleting_cell_number_preserves_the_outer_table_and_sibling_controls() {
    let mut core = load();
    let before = match &core.document().sections[0].paragraphs[383].controls[0] {
        Control::Table(t) => t.cells[0].paragraphs[0].controls.clone(),
        _ => panic!("외부 표"),
    };
    assert_eq!(before.len(), 4, "번호·강조 상자·화살표·그림 원본");
    core.delete_cell_shape_control_by_path_native(0, 383, PATH, 0)
        .unwrap();
    let Control::Table(table) = &core.document().sections[0].paragraphs[383].controls[0] else {
        panic!("번호 도형 삭제가 외부 표를 지웠다");
    };
    assert_eq!(
        format!("{:?}", table.cells[0].paragraphs[0].controls),
        format!("{:?}", &before[1..]),
        "번호 도형만 지우고 그림과 형제 도형의 데이터는 그대로 보존한다"
    );
    assert_eq!(core.page_count(), 50);
}

#[test]
fn shape_delete_rejects_a_picture_without_mutating_its_owner() {
    let mut core = load();
    let before = format!("{:?}", core.document().sections[0].paragraphs[383]);
    assert!(core
        .delete_cell_shape_control_by_path_native(0, 383, PATH, 3)
        .is_err());
    assert_eq!(
        format!("{:?}", core.document().sections[0].paragraphs[383]),
        before,
        "종류가 다른 주소의 삭제 실패는 원자적이다"
    );
}

#[test]
fn restoring_shape_offsets_does_not_reinstate_stale_source_after_another_edit() {
    let mut core = load();
    let before: Value = serde_json::from_str(
        &core
            .get_cell_shape_properties_by_path_native(0, 383, PATH, 0)
            .unwrap(),
    )
    .unwrap();
    core.set_cell_shape_properties_by_path_native(0, 383, PATH, 0, r#"{"horzOffset":15000}"#)
        .unwrap();
    core.insert_text_native(0, 0, 0, "보존할 편집").unwrap();
    let restore = json!({"horzOffset":before["horzOffset"],"vertOffset":before["vertOffset"]});
    core.set_cell_shape_properties_by_path_native(0, 383, PATH, 0, &restore.to_string())
        .unwrap();
    assert!(core.document().sections[0].paragraphs[0]
        .text
        .starts_with("보존할 편집"));
    assert!(
        core.document().sections[0].raw_stream.is_none(),
        "다른 편집이 남은 구역에 원본 raw를 재사용하지 않는다"
    );
    assert!(core.document().layout_profile().session_edited());
}
