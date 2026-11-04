#![cfg(not(target_arch = "wasm32"))]
//! 표 문단에 떠 있는 그림을 붙여도 표와 뒤 문단은 제자리에 남는다.
//!
//! 셀에 그림을 넣으면 그림은 종이 기준 어울림 개체로 표 문단의 형제 제어가 된다(#1151).
//! 그 그림은 앵커 줄을 차지하지 않는데도 문단이 "표 하나만 든 빈 앵커"가 아니라고 판정돼
//! 표 바깥 여백 상자가 흐름에서 빠졌다. 표는 위 여백만큼, 뒤 문단은 위·아래 여백만큼
//! 올라갔고, 저장 뒤 다시 열면 표만 제자리로 돌아와 뒤 문단이 표에 겹쳤다.
use rhwp::wasm_api::HwpDocument;

const CELL: &str = r#"[{"controlIndex":0,"cellIndex":0,"cellParaIndex":0}]"#;

/// (표 위끝, 뒤 문단 글줄 위끝)
fn flow(doc: &HwpDocument) -> (f64, f64) {
    let table: serde_json::Value =
        serde_json::from_str(&doc.get_table_bbox(0, 1, 0).unwrap()).unwrap();
    let next: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect(0, 2, 0).unwrap()).unwrap();
    (table["y"].as_f64().unwrap(), next["y"].as_f64().unwrap())
}

/// 머리 글 아래 문단 1에 2×2 표(바깥 여백 283HU)를 만들고 뒤 문단 2에 글을 둔다.
fn blank_with_table() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, "머리").unwrap();
    doc.split_paragraph(0, 0, 2, None).unwrap();
    doc.create_table(0, 1, 0, 2, 2).unwrap();
    doc.insert_text(0, 2, 0, "뒤").unwrap();
    doc
}

fn assert_flow_kept(cell_path: &str) {
    let mut doc = blank_with_table();
    let before = flow(&doc);
    let png = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/logo/logo-16.png"
    ))
    .unwrap();
    doc.insert_picture(
        0,
        1,
        0,
        cell_path,
        &png,
        1200,
        1200,
        16,
        16,
        "png",
        "",
        Some(7000),
        Some(9000),
    )
    .unwrap();
    assert_eq!(
        flow(&doc),
        before,
        "그림을 붙인 직후 (cell_path={cell_path:?})"
    );
    for (format, bytes) in [
        ("HWP", doc.export_hwp().unwrap()),
        ("HWPX", doc.export_hwpx().unwrap()),
    ] {
        let reopened = HwpDocument::new(&bytes).unwrap();
        assert_eq!(
            flow(&reopened),
            before,
            "{format} 재열기 (cell_path={cell_path:?})"
        );
    }
}

#[test]
fn a_cell_picture_keeps_the_table_and_the_following_paragraph_in_place() {
    assert_flow_kept(CELL);
}

#[test]
fn a_floating_picture_on_the_table_paragraph_keeps_the_flow_in_place() {
    assert_flow_kept("");
}
