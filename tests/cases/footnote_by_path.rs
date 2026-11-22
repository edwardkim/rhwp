//! 관계형. 표 셀·글상자 문단에 경로로 각주를 넣고 읽으며, 쪽 각주 정보가 그 경로를 준다.
//! 주석 갱신 2026-11-23 06:38 KST (+0900) · 직전 파일 변경 2026-11-23T06:36:25+09:00 (60b610ae6a44cfb745e90313481f9eed53098c29).
//! 2026-11-23 한컴 Print, 96 DPI 보정 전 2px 실루엣 raw(Native / fresh WASM).
//! 셀 각주·미주 HWP는 2쪽 중 1쪽 99.65% / 99.68%, 2쪽 100.00% / 100.00%로 전쪽 최저 99.65% / 99.68%다.
//! 같은 HWPX의 두 쪽은 각각 100.00% / 100.00%다. 글상자 미주 대조군은 HWP·HWPX 각 1쪽 100.00% / 100.00%다.
//! 이 검사는 합성 문단의 각주 번호·셀과 글상자 경로·저장 후 내용 관계를 확인한다.
//! 인쇄 원본과 전쪽 결과는 mydocs/pr/assets/semanticist21-20261123-table-notes-shadows/hancom-print.tsv 및 focus-pages.tsv에 있다.

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
    let para = cell_paragraph(&core, table_para, table_ctrl);
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
    let box_note = box_paragraph(&core, shape_para, shape_ctrl)
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

/// 표 첫 셀 첫 문단.
fn cell_paragraph(
    core: &DocumentCore,
    para: usize,
    ctrl: usize,
) -> &rhwp::model::paragraph::Paragraph {
    let Control::Table(table) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("표");
    };
    &table.cells[0].paragraphs[0]
}

/// 글상자 첫 문단.
fn box_paragraph(
    core: &DocumentCore,
    para: usize,
    ctrl: usize,
) -> &rhwp::model::paragraph::Paragraph {
    let Control::Shape(shape) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("글상자");
    };
    &shape
        .drawing()
        .and_then(|d| d.text_box.as_ref())
        .expect("글상자 글")
        .paragraphs[0]
}
