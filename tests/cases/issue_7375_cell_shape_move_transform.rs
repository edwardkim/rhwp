//! 관계형: 번호 도형을 이동·복원할 때 원본 행렬과 표시 경계의 가역성을 검사한다.
//! 주석 갱신: 2026-11-05 20:26 KST.
//! 입력: issue7333/aaaaaa.hwp, 독립 한컴 2020 Print의 27·28쪽 번호 도형.
//! 원본 27쪽 Native/fresh WASM 99.99897%/99.99691%, 28쪽 99.99893%/99.99678%.
//! 같은 PDF·96dpi·compat2022의 정본 Visual Sweep 결과다. 실물 px 기대값을 고정하지 않고
//! 같은 객체의 변경 전후 관계·원본 데이터 보존과 사양의 곡률 백분율을 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
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
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let saved = DocumentCore::from_bytes(&bytes).unwrap();
        let properties: Value = serde_json::from_str(
            &saved
                .get_cell_shape_properties_by_path_native(0, para, PATH, 0)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(properties["horzOffset"], moved["horzOffset"]);
        assert_eq!(
            properties["vertOffset"], moved["vertOffset"],
            "이동을 저장·재열기해도 원본 패스스루로 되돌아가지 않는다"
        );
    }
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
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let saved = DocumentCore::from_bytes(&bytes).unwrap();
        let Control::Table(table) = &saved.document().sections[0].paragraphs[383].controls[0]
        else {
            panic!("저장·재열기한 문서의 외부 표가 사라졌다");
        };
        assert_eq!(
            table.cells[0].paragraphs[0].controls.len(),
            3,
            "삭제한 번호 도형이 원본 패스스루 때문에 저장 결과에 다시 나타나지 않는다"
        );
    }
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

#[test]
fn corner_percentages_survive_both_formats_and_drive_the_same_rectangle_geometry() {
    fn radius(node: &RenderNode) -> Option<(f64, f64)> {
        if let RenderNodeType::Rectangle(rect) = &node.node_type {
            if rect.section_index == Some(0)
                && rect.para_index == Some(383)
                && rect.control_index == Some(0)
                && rect.cell_index == Some(0)
            {
                return Some((rect.corner_radius, node.bbox.width.min(node.bbox.height)));
            }
        }
        node.children.iter().find_map(radius)
    }
    // HWP5 표 94: 직각 0·둥근 모양 20·반원 50. 37은 명시 곡률 보존 대조군이다.
    for rate in [0, 20, 37, 50] {
        let mut core = load();
        core.set_cell_shape_properties_by_path_native(
            0,
            383,
            PATH,
            0,
            &json!({"roundRate":rate}).to_string(),
        )
        .unwrap();
        let tree = core.build_page_render_tree(26).unwrap();
        let (actual, shorter_side) = radius(&tree.root).expect("원본 번호 사각형");
        assert_eq!(
            actual,
            shorter_side * (f64::from(rate) / 100.0),
            "공통 RectangleNode는 곡률 백분율을 한 번만 적용한다"
        );
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let restored = DocumentCore::from_bytes(&bytes).unwrap();
            let props: Value = serde_json::from_str(
                &restored
                    .get_cell_shape_properties_by_path_native(0, 383, PATH, 0)
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                props["roundRate"], rate,
                "HWP/HWPX 저장·재열기 모두 같은 곡률을 보존한다"
            );
        }
    }
}
