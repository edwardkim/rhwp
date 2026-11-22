//! 관계형. 본문·표 셀·글상자 주석의 소유 경로, 번호 순서와 편집 좌표를 비교한다.
//! 주석 갱신 2026-11-23 04:40 KST (+0900) · 직전 파일 변경 2026-11-23T04:06:47+09:00 (0af8e7e5db10dc51f308c5566df8e6b9e3487f1b).
//! 2026-11-23 한컴 Print 대조. 96 DPI, 보정 전 2px 실루엣 raw의 전체 쪽 최저값(Native / fresh WASM).
//! table-cell-notes-hwp 전체 2쪽 99.65% / 99.68% (1쪽 99.65% / 99.68%, 2쪽 100.00% / 100.00%).
//! table-cell-notes-hwpx 전체 2쪽 100.00% / 100.00% (1쪽 100.00% / 100.00%, 2쪽 100.00% / 100.00%).
//! textbox-notes-hwp 전체 1쪽 100.00% / 100.00%.
//! textbox-notes-hwpx 전체 1쪽 100.00% / 100.00%.
//! 같은 동작을 실제 파일로 저장한 인쇄 대조군이다. 합성 모델의 값은 테스트가 구성한 입력과 독립 기대값으로 검사한다.
//! 전쪽 결과와 원본은 mydocs/pr/assets/semanticist21-20261123-table-notes-shadows/focus-pages.tsv 및 hancom-print.tsv에 기록했다.
//!
//! 표 셀·글상자 안 미주 넣기·편집 위치와 글상자 미주 그리기.
//!
//! 한/글 12.30 Mac은 표 셀과 글상자 글에서도 입력 > 주석 > 미주로 캐럿 자리에 미주를 넣고, 미주
//! 번호를 문서 차례로 매기며, 글상자 안 미주 내용도 본문·셀 미주와 함께 문서 끝에 그린다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

/// 본문 "a[미주 1]b", 그 아래 1×1 표 "셀글", 글자처럼 취급한 글상자 "상자끝".
/// 반환: (문서, 표 문단, 표 컨트롤, 글상자 문단, 글상자 컨트롤).
fn document() -> (HwpDocument, usize, usize, usize, usize) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "ab").unwrap();
    doc.insert_endnote_native(0, 0, 1).unwrap();
    let table: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 0, 2, 1, 1).unwrap()).unwrap();
    let table_para = table["paraIdx"].as_u64().unwrap() as usize;
    let table_ctrl = table["controlIdx"].as_u64().unwrap() as usize;
    doc.insert_text_in_cell_native(0, table_para, table_ctrl, 0, 0, 0, "셀글")
        .unwrap();
    let last = doc.document().sections[0].paragraphs.len() - 1;
    let shape: serde_json::Value = serde_json::from_str(
        &doc.create_shape_control_native(
            0,
            last,
            0,
            12000,
            5000,
            0,
            0,
            true,
            "Square",
            "textbox",
            false,
            false,
            &[],
        )
        .unwrap(),
    )
    .unwrap();
    let shape_para = shape["paraIdx"].as_u64().unwrap() as usize;
    let shape_ctrl = shape["controlIdx"].as_u64().unwrap() as usize;
    doc.insert_text_in_cell_by_path(0, shape_para, &[(shape_ctrl, 0, 0)], 0, "상자끝")
        .unwrap();
    (doc, table_para, table_ctrl, shape_para, shape_ctrl)
}

/// 모든 쪽에 그린 글.
fn drawn_text(doc: &HwpDocument) -> String {
    (0..doc.page_count())
        .map(|page| doc.get_page_text_layout_native(page).unwrap())
        .collect()
}

#[test]
fn endnotes_inserted_by_path_into_table_cell_and_text_box() {
    let (mut doc, table_para, table_ctrl, shape_para, shape_ctrl) = document();
    let cell = [(table_ctrl, 0, 0)];
    let text_box = [(shape_ctrl, 0, 0)];

    let result = doc
        .insert_endnote_by_path_native(0, table_para, &cell, 1)
        .unwrap();
    assert!(result.contains("\"controlIdx\":0"), "{result}");
    assert!(result.contains("\"endnoteNumber\":2"), "{result}");
    let result = doc
        .insert_endnote_by_path_native(0, shape_para, &text_box, 2)
        .unwrap();
    assert!(result.contains("\"endnoteNumber\":3"), "{result}");
    assert!(doc
        .insert_endnote_by_path_native(0, table_para, &cell, 9)
        .is_err());

    // 미주 안 글은 셀 경로에 미주 단계(셀 0)를 이어 넣는다.
    doc.insert_text_in_cell_by_path(0, table_para, &[(table_ctrl, 0, 0), (0, 0, 0)], 2, "셀미주")
        .unwrap();
    let box_note = [(shape_ctrl, 0, 0), (0, 0, 0)];
    doc.insert_text_in_cell_by_path(0, shape_para, &box_note, 2, "상자미주")
        .unwrap();
    let info = doc
        .get_footnote_info_by_path_native(0, table_para, &cell, 0)
        .unwrap();
    assert!(info.contains("\"number\":2"), "{info}");
    assert!(info.contains("셀미주"), "{info}");

    // 셀·글상자 미주 모두 문서 끝에 그린다.
    let drawn = drawn_text(&doc);
    assert!(drawn.contains("셀미주"), "{drawn}");
    assert!(drawn.contains("상자미주"), "{drawn}");

    // 모아 그린 미주 가상 문단은 원본 미주의 셀 경로를 준다.
    let base = doc.document().sections[0].paragraphs.len();
    let sources: Vec<String> = (0..4)
        .map(|k| doc.get_endnote_source_at_native(0, base + k).unwrap())
        .collect();
    assert!(
        sources[0].contains("\"notePath\":[],\"noteControlIdx\":"),
        "{}",
        sources[0]
    );
    assert!(
        sources[1].contains(&format!(
            "\"paraIdx\":{table_para},\"notePath\":[{{\"controlIndex\":{table_ctrl},\"cellIndex\":0,\"cellParaIndex\":0}}],\"noteControlIdx\":0,\"noteParaIdx\":0"
        )),
        "{}",
        sources[1]
    );
    assert!(
        sources[2].contains(&format!(
            "\"paraIdx\":{shape_para},\"notePath\":[{{\"controlIndex\":{shape_ctrl},\"cellIndex\":0,\"cellParaIndex\":0}}],\"noteControlIdx\":0,\"noteParaIdx\":0"
        )),
        "{}",
        sources[2]
    );
    assert_eq!(sources[3], "{\"ok\":false}");
    assert_eq!(
        doc.get_endnote_source_at_native(0, 0).unwrap(),
        "{\"ok\":false}"
    );

    // 경로로 짚은 미주 안 캐럿은 그 미주를 그린 줄에 선다.
    let rect = |path: &[(usize, usize, usize)], para: usize, offset: usize| -> serde_json::Value {
        serde_json::from_str(
            &doc.get_cursor_rect_in_note_by_path_native(0, para, path, 0, 0, offset)
                .unwrap(),
        )
        .unwrap()
    };
    let start = rect(&cell, table_para, 2);
    let end = rect(&cell, table_para, 5);
    assert_eq!(start["pageIndex"], end["pageIndex"]);
    assert_eq!(start["y"], end["y"]);
    assert!(end["x"].as_f64().unwrap() > start["x"].as_f64().unwrap());
    let in_box = rect(&text_box, shape_para, 2);
    assert!(in_box["y"].as_f64().unwrap() > start["y"].as_f64().unwrap());
    assert!(doc
        .get_cursor_rect_in_note_by_path_native(0, table_para, &cell, 1, 0, 2)
        .is_err());

    // HWP로 저장해 다시 열어도 번호와 글이 그대로다.
    let reloaded = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    let info = reloaded
        .get_footnote_info_by_path_native(0, shape_para, &text_box, 0)
        .unwrap();
    assert!(info.contains("\"number\":3"), "{info}");
    assert!(info.contains("상자미주"), "{info}");
    assert!(drawn_text(&reloaded).contains("상자미주"));
}
