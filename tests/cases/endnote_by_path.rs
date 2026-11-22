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

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

fn footnote_numbers(core: &DocumentCore) -> Vec<u16> {
    fn walk(paras: &[rhwp::model::paragraph::Paragraph], numbers: &mut Vec<u16>) {
        for para in paras {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Footnote(note) => numbers.push(note.number),
                    Control::Table(table) => {
                        for cell in &table.cells {
                            walk(&cell.paragraphs, numbers);
                        }
                    }
                    Control::Shape(shape) => {
                        if let Some(text_box) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                            walk(&text_box.paragraphs, numbers);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let mut numbers = Vec::new();
    walk(&core.document().sections[0].paragraphs, &mut numbers);
    numbers
}

/// 표 첫 셀 첫 문단의 첫 각주 안 첫 문단.
fn cell_note_paragraph(
    core: &mut DocumentCore,
    table_para: usize,
    table_ctrl: usize,
) -> &mut rhwp::model::paragraph::Paragraph {
    let Control::Table(table) =
        &mut core.document_mut().sections[0].paragraphs[table_para].controls[table_ctrl]
    else {
        panic!("표");
    };
    let Some(Control::Footnote(note)) = table.cells[0].paragraphs[0].controls.first_mut() else {
        panic!("셀 각주");
    };
    &mut note.paragraphs[0]
}

/// 표 셀·글상자 문단에 경로로 각주를 넣으면 문서 차례로 번호가 붙고, 경로로 그 각주 정보를
/// 읽으며, 쪽 각주 정보가 그 경로를 준다. 저장해 다시 읽어도 같다.
#[test]
fn insert_footnote_by_path_in_table_cell_and_text_box() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "ab").unwrap();
    core.insert_footnote_native(0, 0, 1).unwrap();
    let table: serde_json::Value =
        serde_json::from_str(&core.create_table_native(0, 0, 2, 1, 1).unwrap()).unwrap();
    let table_para = table["paraIdx"].as_u64().unwrap() as usize;
    let table_ctrl = table["controlIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_cell_native(0, table_para, table_ctrl, 0, 0, 0, "셀글")
        .unwrap();
    let last = core.document().sections[0].paragraphs.len() - 1;
    let shape: serde_json::Value = serde_json::from_str(
        &core
            .create_shape_control_native(
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
    core.insert_text_in_cell_by_path(0, shape_para, &[(shape_ctrl, 0, 0)], 0, "상자끝")
        .unwrap();

    let cell_path = [(table_ctrl, 0, 0)];
    let cell = core
        .insert_footnote_by_path_native(0, table_para, &cell_path, 1)
        .unwrap();
    assert!(cell.contains("\"controlIdx\":0"), "{cell}");
    assert!(cell.contains("\"footnoteNumber\":2"), "{cell}");
    let box_path = [(shape_ctrl, 0, 0)];
    let text_box = core
        .insert_footnote_by_path_native(0, shape_para, &box_path, 2)
        .unwrap();
    assert!(text_box.contains("\"footnoteNumber\":3"), "{text_box}");
    assert_eq!(footnote_numbers(&core), vec![1, 2, 3]);
    assert!(core
        .insert_footnote_by_path_native(0, table_para, &cell_path, 9)
        .is_err());

    // 셀 문단 글은 그대로이고 번호가 글자 사이 한 칸을 차지한다.
    let Control::Table(table) =
        &core.document().sections[0].paragraphs[table_para].controls[table_ctrl]
    else {
        panic!("표");
    };
    let para = &table.cells[0].paragraphs[0];
    assert_eq!(para.text, "셀글");
    assert_eq!(para.char_offsets, vec![0, 9]);

    // 각주 안 글을 채운다(셀 각주의 첫 문단, 번호 뒤).
    cell_note_paragraph(&mut core, table_para, table_ctrl).insert_text_at(2, "셀주석");
    assert_eq!(
        cell_note_paragraph(&mut core, table_para, table_ctrl).text,
        "  셀주석"
    );

    let info = core.get_page_footnote_info_native(0, 1).unwrap();
    assert!(info.contains(&format!(
        "\"notePath\":[{{\"controlIndex\":{table_ctrl},\"cellIndex\":0,\"cellParaIndex\":0}}],\"noteControlIdx\":0"
    )), "{info}");
    let Control::Shape(shape) =
        &core.document().sections[0].paragraphs[shape_para].controls[shape_ctrl]
    else {
        panic!("글상자");
    };
    let text_box = shape.drawing().unwrap().text_box.as_ref().unwrap();
    let box_note = text_box.paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Footnote(_)))
        .unwrap();
    let info = core.get_page_footnote_info_native(0, 2).unwrap();
    assert!(
        info.contains(&format!(
            "\"notePath\":[{{\"controlIndex\":{shape_ctrl},\"cellIndex\":0,\"cellParaIndex\":0}}],\"noteControlIdx\":{box_note}"
        )),
        "{info}"
    );
    let body = core.get_page_footnote_info_native(0, 0).unwrap();
    assert!(body.contains("\"notePath\":[],"), "{body}");

    let info = core
        .get_footnote_info_by_path_native(0, table_para, &cell_path, 0)
        .unwrap();
    assert!(info.contains("\"number\":2"), "{info}");
    assert!(info.contains("셀주석"), "{info}");
    assert!(core
        .get_footnote_info_by_path_native(0, table_para, &cell_path, 1)
        .is_err());

    let mut reloaded = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_eq!(footnote_numbers(&reloaded), vec![1, 2, 3]);
    assert_eq!(
        cell_note_paragraph(&mut reloaded, table_para, table_ctrl).text,
        "  셀주석"
    );
}
