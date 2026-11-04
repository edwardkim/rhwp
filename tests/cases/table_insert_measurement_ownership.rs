//! 관계형·고정형: table_insert_measurement_ownership의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-04T00:23:31+09:00; 직전 파일 변경 커밋: 2026-11-03T21:45:03+09:00 39b4f81f6aab4cc08c6145ed6911db2a8fe5125e.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. ownership fresh-reference/loaded-before/empty-host-after/after-table-after 각 1쪽, Native/fresh WASM 전쪽 최저 100.00%. 선언된 서로 다른 표의 높이와 삽입 후 소유 이동·빈 후속 문단을 관계로 검사한다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/validation.md 및 전쪽 TSV·확대 PNG.
//! 새 표를 삽입할 때 기존 표의 측정 하한은 원래 표를 따라 이동한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn height(doc: &HwpDocument, para: u32, ctrl: u32) -> f64 {
    let bbox: Value = serde_json::from_str(&doc.get_table_bbox(0, para, ctrl).unwrap()).unwrap();
    bbox["height"].as_f64().unwrap()
}

fn create(doc: &mut HwpDocument, para: u32) -> (u32, u32) {
    let result: Value = serde_json::from_str(&doc.create_table(0, para, 0, 2, 1).unwrap()).unwrap();
    (
        result["paraIdx"].as_u64().unwrap() as u32,
        result["controlIdx"].as_u64().unwrap() as u32,
    )
}

fn loaded_tables() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, "머리").unwrap();
    doc.split_paragraph(0, 0, 2, None).unwrap();
    doc.split_paragraph(0, 1, 0, None).unwrap();
    assert_eq!(create(&mut doc, 2), (2, 0));
    assert_eq!(create(&mut doc, 3), (3, 0));
    for (para, size) in [(2, 5000), (3, 9000)] {
        for cell in 0..2 {
            doc.set_cell_properties(0, para, 0, cell, &format!(r#"{{"height":{size}}}"#))
                .unwrap();
        }
    }
    HwpDocument::new(&doc.export_hwp().unwrap()).unwrap()
}

#[test]
fn inserting_into_an_empty_host_keeps_each_existing_tables_own_height() {
    let mut doc = loaded_tables();
    let before = [height(&doc, 2, 0), height(&doc, 3, 0)];
    assert!(before[1] > before[0] + 1.0);
    assert_eq!(create(&mut doc, 1), (1, 0));
    for (index, para) in [3, 4].into_iter().enumerate() {
        assert!(
            (height(&doc, para, 0) - before[index]).abs() <= 0.1,
            "기존 표 {index}가 다른 표의 행 높이를 받으면 안 된다"
        );
    }
}

#[test]
fn a_new_table_after_a_table_does_not_inherit_the_next_tables_height() {
    let mut reference = HwpDocument::create_empty();
    reference.create_blank_document().unwrap();
    let (para, ctrl) = create(&mut reference, 0);
    let expected = height(&reference, para, ctrl);

    let mut doc = loaded_tables();
    let before = [height(&doc, 2, 0), height(&doc, 3, 0)];
    assert_eq!(create(&mut doc, 2), (3, 0));
    assert!(
        (height(&doc, 3, 0) - expected).abs() <= 0.1,
        "새 표가 다음 표의 행 높이를 계승하면 안 된다"
    );
    for (index, para) in [2, 5].into_iter().enumerate() {
        assert!((height(&doc, para, 0) - before[index]).abs() <= 0.1);
    }
}
