//! Issue #6771 — 셀·글상자 **안**에 있는 표를 지운다.
//!
//! 공공기관 배포 양식은 작성 안내문을 셀 안 1×1 표(점선 상자)에 담고, 본문에 "안내 박스는
//! 반드시 삭제 후 제출"이라고 적는다. 글자는 `delete_text_in_cell_by_path` 로 지울 수 있었지만
//! **그릇을 지울 길이 없었다**: `delete_control_at` 은 본문 리스트만 다루고
//! (`{"ok":false,"reason":"본문 밖 컨트롤은 아직 다루지 않는다"}`),
//! `delete_table_control` 은 `(구역, 문단, 컨트롤)` 셋만 받아 셀 안을 짚지 못한다.
//!
//! 그래서 안내문을 지운 제출본에 **빈 점선 상자**가 남았다. 이 테스트는 그 자리를 고정한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{CommonObjAttr, DrawingObjAttr, RectangleShape, ShapeObject, TextBox};
use rhwp::model::table::{Cell, Table};

/// 셀 안에 표가 든 공개 샘플 — 섹션0 문단 20, 셀 0의 문단 2가 표를 품는다.
const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";
const PARENT_PARA: usize = 20;

fn load() -> DocumentCore {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    DocumentCore::from_bytes(&bytes).expect("parse sample")
}

/// 셀 안 표가 든 호스트 문단 경로.
const HOST_PATH: &str = r#"[{"controlIndex":0,"cellIndex":0,"cellParaIndex":2}]"#;

#[test]
fn deletes_table_inside_cell() {
    let mut doc = load();

    let deleted = doc.delete_cell_table_control_by_path_native(0, PARENT_PARA, HOST_PATH, 0);
    assert!(deleted.is_ok(), "셀 안 표 삭제 실패: {:?}", deleted.err());
    assert_eq!(deleted.unwrap(), "{\"ok\":true}");

    // 지운 뒤 같은 자리를 다시 지우려 하면 **없다**고 답해야 한다 — 삭제가 실제로 일어난 증거다.
    let again = doc.delete_cell_table_control_by_path_native(0, PARENT_PARA, HOST_PATH, 0);
    assert!(again.is_err(), "표가 남아 있다: {:?}", again.ok());
}

/// 🔴 종류를 지목해서 지운다 — 그림 자리에 표 삭제를 부르면 거절해야 한다(반대도 같다).
#[test]
fn rejects_when_control_is_not_a_table() {
    let mut doc = load();

    // 같은 자리를 그림으로 지목하면 거절한다(그 컨트롤은 표다).
    let wrong = doc.delete_cell_picture_control_by_path_native(0, PARENT_PARA, HOST_PATH, 0);
    assert!(wrong.is_err(), "그림이 아닌데 지워졌다");
    assert!(
        format!("{:?}", wrong.err()).contains("그림이 아닙니다"),
        "오류 문구가 종류를 말하지 않는다",
    );
}

/// 범위 밖 컨트롤 번호는 조용히 성공하지 않는다.
#[test]
fn rejects_out_of_range_control_index() {
    let mut doc = load();
    assert!(doc
        .delete_cell_table_control_by_path_native(0, PARENT_PARA, HOST_PATH, 99)
        .is_err());
}

fn paragraph_at<'a>(
    doc: &'a DocumentCore,
    parent: usize,
    path: &[(usize, usize, usize)],
) -> &'a Paragraph {
    let mut para = &doc.document().sections[0].paragraphs[parent];
    for &(control, cell, paragraph) in path {
        para = match &para.controls[control] {
            Control::Table(table) => &table.cells[cell].paragraphs[paragraph],
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Rectangle(rectangle) => {
                    assert_eq!(cell, 0);
                    &rectangle.drawing.text_box.as_ref().unwrap().paragraphs[paragraph]
                }
                _ => panic!("unexpected shape"),
            },
            _ => panic!("unexpected container"),
        };
    }
    para
}

fn assert_roundtrip_deleted(
    core: &DocumentCore,
    parent: usize,
    path: &[(usize, usize, usize)],
    text: &str,
    remaining: usize,
) {
    for bytes in [
        core.export_hwp_native().expect("save HWP"),
        core.export_hwpx_native().expect("save HWPX"),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).expect("reopen saved document");
        let para = paragraph_at(&reopened, parent, path);
        assert_eq!(para.controls.len(), remaining, "deleted table reappeared");
        assert_eq!(para.text, text, "surrounding text changed");
    }
}

#[test]
fn real_sample_deletion_survives_hwp_and_hwpx_save() {
    let mut core = load();
    let path = [(0, 0, 2)];
    let original = paragraph_at(&core, PARENT_PARA, &path);
    let text = original.text.clone();
    let remaining = original.controls.len() - 1;
    core.delete_cell_table_control_by_path_native(0, PARENT_PARA, HOST_PATH, 0)
        .unwrap();
    assert_roundtrip_deleted(&core, PARENT_PARA, &path, &text, remaining);
    let events: serde_json::Value = serde_json::from_str(&core.serialize_event_log()).unwrap();
    let last = events["events"].as_array().unwrap().last().unwrap();
    assert_eq!(last["type"], "CellTableDeleted");
    assert_eq!(last["section"], 0);
    assert_eq!(last["para"], PARENT_PARA);
    assert_eq!(last["innerControlIndex"], 0);
    assert_eq!(
        last["cellPath"],
        serde_json::from_str::<serde_json::Value>(HOST_PATH).unwrap()
    );
}

fn table_with(paragraph: Paragraph) -> Control {
    Control::Table(Box::new(Table {
        common: CommonObjAttr {
            width: 12_000,
            height: 4_000,
            treat_as_char: true,
            ..Default::default()
        },
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            row_span: 1,
            col_span: 1,
            width: 12_000,
            height: 4_000,
            paragraphs: vec![paragraph],
            ..Default::default()
        }],
        ..Default::default()
    }))
}

fn control_paragraph(control: Control) -> Paragraph {
    Paragraph {
        char_count: 8,
        controls: vec![control],
        ..Default::default()
    }
}

fn synthetic_core(target: Control, textbox: bool) -> DocumentCore {
    let inner = Paragraph {
        text: "LEFTRIGHT".to_string(),
        char_count: 17,
        char_offsets: (0..9).map(|i| if i >= 4 { i + 8 } else { i }).collect(),
        controls: vec![target],
        ..Default::default()
    };
    let nested = control_paragraph(table_with(inner));
    let outer = if textbox {
        Control::Shape(Box::new(ShapeObject::Rectangle(RectangleShape {
            common: CommonObjAttr {
                width: 20_000,
                height: 12_000,
                treat_as_char: true,
                ..Default::default()
            },
            drawing: DrawingObjAttr {
                text_box: Some(TextBox {
                    max_width: 20_000,
                    paragraphs: vec![nested],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })))
    } else {
        table_with(nested)
    };
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank template");
    let mut doc = core.document().clone();
    // 첫 문단은 구역/단 정의를 위한 문단으로 유지한다. 저장기가 첫 문단에
    // 정의 컨트롤을 보강하더라도 시험 대상의 중첩 경로는 바뀌지 않아야 한다.
    doc.sections[0].paragraphs.truncate(1);
    doc.sections[0].paragraphs.push(control_paragraph(outer));
    doc.sections[0].raw_stream = None;
    core.set_document(doc);
    core
}

const SYNTHETIC_PARENT: usize = 1;
const DEEP_PATH: &str = r#"[{"controlIndex":0,"cellIndex":0,"cellParaIndex":0},{"controlIndex":0,"cellIndex":0,"cellParaIndex":0}]"#;

#[test]
fn nested_cell_and_textbox_deletion_preserve_text_and_offsets() {
    for textbox in [false, true] {
        let mut core = synthetic_core(table_with(Paragraph::default()), textbox);
        core.delete_cell_table_control_by_path_native(0, SYNTHETIC_PARENT, DEEP_PATH, 0)
            .expect("delete nested table");
        let path = [(0, 0, 0), (0, 0, 0)];
        let para = paragraph_at(&core, SYNTHETIC_PARENT, &path);
        assert!(para.controls.is_empty());
        assert_eq!(para.text, "LEFTRIGHT");
        assert_eq!(para.char_offsets, (0..9).collect::<Vec<_>>());
        assert_roundtrip_deleted(&core, SYNTHETIC_PARENT, &path, "LEFTRIGHT", 0);
    }
}

#[test]
fn table_api_rejects_picture_and_picture_api_keeps_its_event() {
    let mut core = synthetic_core(Control::Picture(Box::default()), false);
    let events_before = core.serialize_event_log();
    assert!(core
        .delete_cell_table_control_by_path_native(0, SYNTHETIC_PARENT, DEEP_PATH, 0)
        .is_err());
    let path = [(0, 0, 0), (0, 0, 0)];
    let para = paragraph_at(&core, SYNTHETIC_PARENT, &path);
    assert!(matches!(para.controls[0], Control::Picture(_)));
    assert_eq!(para.text, "LEFTRIGHT");
    assert_eq!(core.serialize_event_log(), events_before);
    core.delete_cell_picture_control_by_path_native(0, SYNTHETIC_PARENT, DEEP_PATH, 0)
        .unwrap();
    assert!(paragraph_at(&core, SYNTHETIC_PARENT, &path)
        .controls
        .is_empty());
    let events: serde_json::Value = serde_json::from_str(&core.serialize_event_log()).unwrap();
    assert_eq!(
        events["events"].as_array().unwrap().last().unwrap()["type"],
        "PictureDeleted"
    );
    assert_roundtrip_deleted(&core, SYNTHETIC_PARENT, &path, "LEFTRIGHT", 0);
}
